use ntapi::ntexapi::{NtSetSystemInformation, SystemAllowedCpuSetsInformation};
use windows::Win32::Foundation::{GetLastError, HANDLE};
use windows::Win32::System::SystemInformation::{
    GetSystemCpuSetInformation, SYSTEM_CPU_SET_INFORMATION,
};

/// https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getsystemcpusetinformation
/// GetSystemCpuSetInformation
pub unsafe fn get_system_cpu_set_information(
    handle: HANDLE,
) -> Vec<std::mem::MaybeUninit<SYSTEM_CPU_SET_INFORMATION>> {
    let cpu_num: usize = number_of_processors();

    let buf_len = std::mem::size_of::<SYSTEM_CPU_SET_INFORMATION>() * cpu_num;
    let mut ret_len: u32 = 0;
    let mut infos: Vec<std::mem::MaybeUninit<SYSTEM_CPU_SET_INFORMATION>> =
        Vec::with_capacity(cpu_num);
    infos.set_len(cpu_num);
    let r = GetSystemCpuSetInformation(
        Some(infos.as_ptr() as *mut SYSTEM_CPU_SET_INFORMATION),
        buf_len as u32,
        &mut ret_len,
        handle,
        0,
    );
    if !r.as_bool() {
        println!("Err GetSystemCpuSetInformation");
    }

    infos
}

// https://learn.microsoft.com/en-us/windows/win32/sysinfo/ntsetsysteminformation
pub unsafe fn system_allowed_cpu_sets_information(cpusets: Vec<u64>) {
    let length = (cpusets.len() * std::mem::size_of::<u64>()) as u32;
    let status = NtSetSystemInformation(
        SystemAllowedCpuSetsInformation,
        cpusets.as_ptr() as *mut ntapi::winapi::ctypes::c_void,
        length,
    );
    if status != 0 {
        println!(
            "Failed to change system CPU set (NTSTATUS: 0x{:08X}, Win32 Error: {:?}).",
            status,
            GetLastError()
        );
    }
}

pub unsafe fn number_of_processors() -> usize {
    let mut info: windows::Win32::System::SystemInformation::SYSTEM_INFO = std::mem::zeroed();
    windows::Win32::System::SystemInformation::GetSystemInfo(&mut info);
    return info.dwNumberOfProcessors as usize;
}
