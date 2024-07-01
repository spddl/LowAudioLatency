# LowAudioLatency

[![Downloads][1]][2] [![GitHub stars][3]][4]

[1]: https://img.shields.io/github/downloads/spddl/LowAudioLatency/total.svg
[2]: https://github.com/spddl/LowAudioLatency/releases "Downloads"
[3]: https://img.shields.io/github/stars/spddl/LowAudioLatency.svg
[4]: https://github.com/spddl/LowAudioLatency/stargazers "GitHub stars"

## About This Project

LowAudioLatency sets the Windows audio buffer to the smallest possible value, similar to [miniant-git/REAL](https://github.com/miniant-git/REAL). LAL not only checks the output devices (headphones, speakers) but also the input devices (microphones). Additionally, it removes the real-time connection to the first CPU thread, as it is not necessary for this function. If the smallest buffer size is already the default buffer size, the program will terminate.

> [!IMPORTANT]
> Please note that not every audio driver supports this feature, and not all hardware has a driver with this capability.

## Usage

Simply run the executable file to minimize the audio latency for the output and input device:

```
low_audio_latency.exe
```

You can also specify EDataFlow and ERole yourself with an optional buffer value:

```
low_audio_latency.exe eRender,eConsole,336 eCapture,eCommunications,336
```

## Supported parameters

| Name                                                                                                                                                      | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | Required | Allowed values                                                    |
| --------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- | ----------------------------------------------------------------- |
| [EDataFlow](https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/ne-mmdeviceapi-edataflow#constants)                                           | The EDataFlow enumeration defines constants that indicate the direction in which audio data flows between an audio endpoint device and an application.                                                                                                                                                                                                                                                                                                                                          | Yes      | Enumeration ID or `eRender`, `eCapture`, or `eAll`                |
| [ERole](https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/ne-mmdeviceapi-erole#constants)                                                   | The ERole enumeration defines constants that indicate the role that the system has assigned to an audio endpoint device.                                                                                                                                                                                                                                                                                                                                                                        | Yes      | Enumeration ID or `eConsole`, `eMultimedia`, or `eCommunications` |
| [pMinPeriodInFrames](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudioclient3-initializesharedaudiostream#parameters) | Periodicity requested by the client. This value must be an integral multiple of the value returned in the <i>pFundamentalPeriodInFrames</i> parameter to <a href="/windows/desktop/api/audioclient/nf-audioclient-iaudioclient3-getsharedmodeengineperiod">IAudioClient3::GetSharedModeEnginePeriod</a>. <i>PeriodInFrames</i> must also be greater than or equal to the value returned in <i>pMinPeriodInFrames</i> and less than or equal to the value returned in <i>pMaxPeriodInFrames</i>. | No       | 0 determines the lowest value itself                              |

> [!Note]
>
> ### If a driver supports small buffer sizes, will all applications in Windows 10 and later automatically use small buffers to render and capture audio?
>
> No, by default all applications in Windows 10 and later will use 10-ms buffers to render and capture audio. If an application needs to use small buffers, then it needs to use the new AudioGraph settings or the WASAPI IAudioClient3 interface, in order to do so. However, if one application requests the usage of small buffers, then the audio engine will start transferring audio using that particular buffer size. In that case, all applications that use the same endpoint and mode will automatically switch to that small buffer size. When the low latency application exits, the audio engine will switch to 10-ms buffers again.
>
> quote: [Low-Latency Audio FAQ](https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/low-latency-audio#faq)

## Requirement

Windows 10 for the core task [IAudioClient3](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nn-audioclient-iaudioclient3) and Windows 11 for [ProcessPowerThrottling](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/ne-processthreadsapi-process_information_class)
