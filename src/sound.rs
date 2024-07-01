use windows::Win32::Media::Audio::{
    EDataFlow, ERole, IAudioClient3, IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{CoCreateInstance, StructuredStorage, CLSCTX_ALL, STGM_READ};
use windows::Win32::System::Variant::{VARENUM, VT_LPWSTR};
use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, PROPERTYKEY};

#[allow(non_upper_case_globals)]
const PKEY_Device_FriendlyName: PROPERTYKEY = PROPERTYKEY {
    fmtid: windows::core::GUID::from_values(
        0xa45c254e,
        0xdf1c,
        0x4efd,
        [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0],
    ),
    pid: 14,
};

pub fn apply_audio_settings(
    audiothread_tx: &std::sync::mpsc::Sender<bool>,
    edataflow: EDataFlow,
    erole: ERole,
    p_min_period_in_frames: u32,
) {
    unsafe {
        let device_enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .expect("CoCreateInstance Failed");

        // https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nf-mmdeviceapi-immdeviceenumerator-getdefaultaudioendpoint
        let default_audio_endpoint = device_enumerator.GetDefaultAudioEndpoint(edataflow, erole);
        if default_audio_endpoint.is_err() {
            println!(
                "GetDefaultAudioEndpoint Failed: {:?}",
                default_audio_endpoint.unwrap_err()
            );
            audiothread_tx.send(false).unwrap();
            return;
        }

        let endpoint = default_audio_endpoint.unwrap();

        let p_audio_client: IAudioClient3 = endpoint
            .Activate(CLSCTX_ALL, None)
            .expect("Activate Failed");

        // https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudioclient-getmixformat
        let p_format = p_audio_client.GetMixFormat().unwrap();

        // https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nf-mmdeviceapi-immdevice-openpropertystore
        let property_store = endpoint
            .OpenPropertyStore(STGM_READ)
            .expect("OpenPropertyStore Failed");

        let friendly_name = get_property_vt_lpwstr(&property_store, &PKEY_Device_FriendlyName);

        let n_channels = (*p_format).nChannels;
        let w_bits_per_sample = (*p_format).wBitsPerSample;
        let n_samples_per_sec = (*p_format).nSamplesPerSec;
        let n_avg_bytes_per_sec = (*p_format).nAvgBytesPerSec;

        let mut pdefaultperiodinframes: u32 = 0;
        let mut pfundamentalperiodinframes: u32 = 0;
        let mut pminperiodinframes: u32 = 0;
        let mut pmaxperiodinframes: u32 = 0;

        // https://docs.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudioclient3-getsharedmodeengineperiod
        p_audio_client
            .GetSharedModeEnginePeriod(
                p_format,
                &mut pdefaultperiodinframes,
                &mut pfundamentalperiodinframes,
                &mut pminperiodinframes,
                &mut pmaxperiodinframes,
            )
            .expect("GetSharedModeEnginePeriod Failed");

        println!("Friendly Name...........{}", friendly_name);
        println!("Channels{:.>17}", n_channels);
        println!("Bits per sample{:.>11} Bit", w_bits_per_sample);
        println!("Samples per sec{:.>11} kHz", n_samples_per_sec / 1000);
        println!("Average{:.>23} bytes/s", n_avg_bytes_per_sec);

        let n_samples_per_sec_float: f64 = n_samples_per_sec as f64;
        println!(
            "Buffer size (default){:.>6} samples (about {} milliseconds)",
            pdefaultperiodinframes,
            pdefaultperiodinframes as f64 / n_samples_per_sec_float * 1000.0
        );
        println!(
            "Buffer size (min){:.>10} samples (about {} milliseconds)",
            pminperiodinframes,
            pminperiodinframes as f64 / n_samples_per_sec_float * 1000.0
        );
        println!(
            "Buffer size (max){:.>10} samples (about {} milliseconds)",
            pmaxperiodinframes,
            pmaxperiodinframes as f64 / n_samples_per_sec_float * 1000.0
        );

        if p_min_period_in_frames == 0 {
            if pminperiodinframes >= pdefaultperiodinframes {
                println!("no change necessary, exit");
                audiothread_tx.send(false).unwrap();
                return;
            }
        } else {
            pminperiodinframes = p_min_period_in_frames;
            println!(
                "Buffer new (min) size{:.>6} samples (about {} milliseconds)",
                pminperiodinframes,
                pminperiodinframes as f64 / n_samples_per_sec_float * 1000.0
            );
        }

        // https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudioclient3-initializesharedaudiostream
        p_audio_client
            .InitializeSharedAudioStream(0, pminperiodinframes, p_format, None)
            .expect("p_audio_client.InitializeSharedAudioStream Failed");

        // https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudioclient-start
        p_audio_client.Start().expect("p_audio_client.Start Failed");

        // Thread is ready
        audiothread_tx.send(true).unwrap();

        std::thread::park();

        // unreachable code
        // https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudioclient-stop
        p_audio_client.Stop().expect("Stop Failed");
    }
}

fn get_property_vt_lpwstr(store: &IPropertyStore, props_key: &PROPERTYKEY) -> String {
    #[allow(unused_assignments)]
    let mut result = String::from("");

    unsafe {
        let mut property_value = store.GetValue(props_key as *const _ as *const _).unwrap();
        let prop_variant = property_value.as_raw().Anonymous.Anonymous;

        if !VT_LPWSTR.eq(&VARENUM(prop_variant.vt)) {
            println!(
                "property store produced invalid data: {:?}",
                prop_variant.vt
            );
        }

        let ptr_utf16 = *(&prop_variant.Anonymous as *const _ as *const *const u16);

        // Find the length of the friendly name.
        let mut len = 0;
        while *ptr_utf16.offset(len) != 0 {
            len += 1;
        }

        // Create the utf16 slice and convert it into a string.
        let name_slice = std::slice::from_raw_parts(ptr_utf16, len as usize);
        let name_os_string: std::ffi::OsString =
            std::os::windows::ffi::OsStringExt::from_wide(name_slice);
        let name_string = match name_os_string.into_string() {
            Ok(string) => string,
            Err(os_string) => os_string.to_string_lossy().into(),
        };

        StructuredStorage::PropVariantClear(&mut property_value).ok();

        result = name_string
    }
    result
}
