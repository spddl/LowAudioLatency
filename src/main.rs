// #![windows_subsystem = "windows"]

use windows::Win32::System::Threading::*;
mod sound;

#[cfg(windows)]
fn main() {
    // https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setpriorityclass
	unsafe {
		SetPriorityClass(GetCurrentProcess(), PROCESS_MODE_BACKGROUND_BEGIN);
	}

    let mut handles = Vec::new();
    for data in [
        (
            windows::Win32::Media::Audio::eRender,
            windows::Win32::Media::Audio::eConsole,
        ),
        (
            windows::Win32::Media::Audio::eCapture,
            windows::Win32::Media::Audio::eCommunications,
        ),
    ]
    .iter()
    {
        handles.push(std::thread::spawn(move || {
            sound::apply_audio_settings(data.0, data.1)
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
