use windows::Win32::Media::Audio::{
    IAudioClient3, IMMDeviceEnumerator, MMDeviceEnumerator,
    eConsole, eRender,
};
use windows::Win32::{
    System::Com::{
        CoCreateInstance, CoInitialize, CLSCTX_ALL,
    },
};
use windows::core::GUID;

pub fn apply_audio_settings() {
    unsafe {
        CoInitialize(std::ptr::null_mut()).expect("CoInitialize Failed");

        let imm_device_enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .expect("CoCreateInstance Failed");

        let endpoint = imm_device_enumerator
            .GetDefaultAudioEndpoint(eRender, eConsole)
            .expect("GetDefaultAudioEnpoint Failed");

        let p_audio_client : IAudioClient3 = endpoint.Activate(
            CLSCTX_ALL,
            std::ptr::null_mut(),
        ).expect("Activate Failed");

        let p_format = p_audio_client.GetMixFormat().unwrap();

        let n_channels  = (*p_format).nChannels;
        let w_bits_per_sample = (*p_format).wBitsPerSample;
        let n_samples_per_sec = (*p_format).nSamplesPerSec;
        let n_avg_bytes_per_sec = (*p_format).nAvgBytesPerSec;

        let mut pdefaultperiodinframes : u32 = 0;
        let mut pfundamentalperiodinframes : u32 = 0;
        let mut pminperiodinframes : u32 = 0;
        let mut pmaxperiodinframes : u32 = 0;
        // https://docs.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudioclient3-getsharedmodeengineperiod
        p_audio_client.GetSharedModeEnginePeriod(
            p_format,
            &mut pdefaultperiodinframes,
            &mut pfundamentalperiodinframes,
            &mut pminperiodinframes,
            &mut pmaxperiodinframes,
        ).expect("GetSharedModeEnginePeriod Failed");

        println!("Channels{:.>17}", n_channels);
        println!("Bits per sample{:.>11} Bit", w_bits_per_sample);
        println!("Samples per sec{:.>11} kHz", n_samples_per_sec/1000);
        println!("Average{:.>23} bytes/s", n_avg_bytes_per_sec);

        let n_samples_per_sec_float: f64 = n_samples_per_sec as f64;
        println!("Buffer size (default){:.>6} samples (about {} milliseconds)", pdefaultperiodinframes, pdefaultperiodinframes as f64 / n_samples_per_sec_float * 1000.0);
        println!("Buffer size (min){:.>10} samples (about {} milliseconds)", pminperiodinframes, pminperiodinframes as f64 / n_samples_per_sec_float * 1000.0);
        println!("Buffer size (max){:.>10} samples (about {} milliseconds)", pmaxperiodinframes, pmaxperiodinframes as f64 / n_samples_per_sec_float * 1000.0);

        if pminperiodinframes >= pdefaultperiodinframes {
            println!("no change necessary, exit");
            return
        }

        const NULL_GUID : GUID =GUID{ data1: 0, data2: 0, data3: 0, data4: [0, 0, 0, 0, 0, 0, 0, 0] };
        p_audio_client.InitializeSharedAudioStream(
            0,
            pminperiodinframes,
            p_format,
            &NULL_GUID,
        ).expect("InitializeSharedAudioStream Failed");

        p_audio_client.Start()
            .expect("Start Failed");
    }

    std::thread::park();
}
