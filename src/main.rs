// #![windows_subsystem = "windows"]
use windows_dll::dll;

enum CVoid {}
#[dll("kernel32.dll")]
extern "system" {
    #[allow(non_snake_case)]
    fn GetCurrentProcess() -> *mut CVoid;

    #[allow(non_snake_case)]
    fn SetPriorityClass(
        hProcess: *mut CVoid,
        dwPriorityClass: u32,
    ) -> i32;
}
mod sound;

#[cfg(windows)]
fn main() {
    // https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setpriorityclass
    unsafe { SetPriorityClass(GetCurrentProcess(), 0x00100000); } // PROCESS_MODE_BACKGROUND_BEGIN

    sound::apply_audio_settings();
}
