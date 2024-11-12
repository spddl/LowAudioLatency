// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cpuset;
mod sound;

use std::sync::mpsc::channel;
use windows::Win32::Foundation::{CloseHandle, GetLastError, FALSE, HANDLE, LUID};
use windows::Win32::Media::Audio::*;
use windows::Win32::Security::{
    AdjustTokenPrivileges, GetTokenInformation, LookupPrivilegeValueW, TokenElevation,
    SE_INC_BASE_PRIORITY_NAME, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_ELEVATION,
    TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
use windows::Win32::System::SystemInformation::{
    SYSTEM_CPU_SET_INFORMATION_ALLOCATED, SYSTEM_CPU_SET_INFORMATION_ALLOCATED_TO_TARGET_PROCESS,
    SYSTEM_CPU_SET_INFORMATION_REALTIME,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, OpenProcessToken, ProcessPowerThrottling, SetPriorityClass,
    SetProcessInformation, IDLE_PRIORITY_CLASS, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
    PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
};

#[cfg(windows)]
fn main() {
    unsafe {
        let current_process = GetCurrentProcess();

        // https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setpriorityclass
        SetPriorityClass(current_process, IDLE_PRIORITY_CLASS).expect("Err SetPriorityClass");

        // ControlMask selects the mechanism and StateMask declares which mechanism should be on or off.
        let state = PROCESS_POWER_THROTTLING_STATE {
            Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
            ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
            StateMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        };

        // only works with Windows Build 22000 or higher
        let state_pointer: *const std::ffi::c_void = std::ptr::addr_of!(state).cast();
        let _ = SetProcessInformation(
            current_process,
            ProcessPowerThrottling,
            state_pointer,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        );

        // https://learn.microsoft.com/de-de/windows/win32/api/objbase/nf-objbase-coinitialize
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        if hr.is_err() {
            println!("hr: {:?}", hr)
        }
    }

    let opt = parse_args(std::env::args().skip(1).collect());
    let opt_len = opt.len();

    let mut handles = Vec::new();
    let (audio_tx, audio_rx) = channel();
    for data in opt {
        let audiothread_tx = audio_tx.clone();
        handles.push(std::thread::spawn(move || {
            sound::apply_audio_settings(&audiothread_tx, data.0, data.1, data.2);
        }));
    }

    // Wait for all audio streams to start or fail
    for _ in 0..opt_len {
        let _ = audio_rx.recv().unwrap();
    }

    unsafe {
        let infos = cpuset::get_system_cpu_set_information(GetCurrentProcess());
        if let Some(core0) = infos.first() {
            let flags = core0.Anonymous.CpuSet.Anonymous1.AllFlags;

            // This process is allocated a core in the CpuSet with realtime flag
            if (flags & SYSTEM_CPU_SET_INFORMATION_ALLOCATED as u8 != 0
                || flags & SYSTEM_CPU_SET_INFORMATION_ALLOCATED_TO_TARGET_PROCESS as u8 != 0)
                && flags & SYSTEM_CPU_SET_INFORMATION_REALTIME as u8 != 0
            {
                println!("This process is allocated a core in the CpuSet with realtime flag");
                enable_debug_privilege();
                let bitmask: Vec<u64> = vec![(1 << cpuset::number_of_processors()) - 1];
                cpuset::system_allowed_cpu_sets_information(bitmask);
            }
        }
    }

    // keeps the parked threads alive
    for handle in handles {
        handle.join().unwrap();
    }
}

// https://github.com/mstange/samply/blob/eab5ffb44a23d92fa35aa64d1dc0ad31f6a9ae78/samply/src/windows/winutils.rs#L18
fn is_elevated() -> bool {
    unsafe {
        let mut handle: HANDLE = Default::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle).ok();

        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
        GetTokenInformation(
            handle,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut std::ffi::c_void),
            size,
            &mut size,
        )
        .ok();

        elevation.TokenIsElevated != 0
    }
}

// https://github.com/mstange/samply/blob/eab5ffb44a23d92fa35aa64d1dc0ad31f6a9ae78/samply/src/windows/winutils.rs#L39
fn enable_debug_privilege() {
    if !is_elevated() {
        eprintln!(
            "You must run samply as an Administrator so that it can enable SeDebugPrivilege. \
            Try using 'sudo' on recent Windows."
        );
        std::process::exit(1);
    }

    unsafe {
        let mut h_token: HANDLE = Default::default();
        let mut tp: TOKEN_PRIVILEGES = std::mem::zeroed();

        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut h_token,
        )
        .is_err()
        {
            panic!("OpenProcessToken failed. Error: {:?}", GetLastError());
        }

        let mut luid: LUID = std::mem::zeroed();

        if LookupPrivilegeValueW(
            windows::core::PCWSTR::null(),
            SE_INC_BASE_PRIORITY_NAME,
            &mut luid,
        )
        .is_err()
        {
            panic!("LookupPrivilegeValue failed. Error: {:?}", GetLastError());
        }

        tp.PrivilegeCount = 1;
        tp.Privileges[0].Luid = luid;
        tp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

        if AdjustTokenPrivileges(
            h_token,
            FALSE,
            Some(&tp),
            std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
            None,
            None,
        )
        .is_err()
        {
            panic!("AdjustTokenPrivileges failed. Error: {:?}", GetLastError());
        }

        if GetLastError().is_err() {
            eprintln!(
                "AdjustTokenPrivileges succeeded, but the error result is failure. Likely \
                the token does not have the specified privilege, which means you are not running \
                as Administrator. GetLastError: {:?}",
                GetLastError()
            );
            std::process::exit(1);
        }

        CloseHandle(h_token).ok();
    }
}

fn parse_args(args: Vec<String>) -> Vec<(EDataFlow, ERole, u32)> {
    if args.is_empty() {
        return vec![
            (
                eRender,  // EDataFlow
                eConsole, // ERole
                0,        // pMinPeriodInFrames
            ),
            (
                eCapture,        // EDataFlow
                eCommunications, // ERole
                0,               // pMinPeriodInFrames
            ),
        ];
    }

    args.iter()
        .map(|data| {
            let params: Vec<&str> = data.split(',').collect();
            (
                match params.get(0).unwrap_or(&"0").to_lowercase().as_str() {
                    "erender" => eRender,
                    "ecapture" => eCapture,
                    "eall" => eAll,
                    _ => EDataFlow(0), // eRender
                },
                match params.get(1).unwrap_or(&"0").to_lowercase().as_str() {
                    "econsole" => eConsole,
                    "emultimedia" => eMultimedia,
                    "ecommunications" => eCommunications,
                    _ => ERole(0), // eConsole
                },
                params.get(2).unwrap_or(&"0").parse().unwrap_or(0),
            )
        })
        .collect()
}
