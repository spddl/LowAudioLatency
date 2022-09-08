# LowAudioLatency

only a rewrite of the original [miniant-git/REAL](https://github.com/miniant-git/REAL)

The program checks the smallest buffer size and sets it.
If the smallest buffer size is the default buffer size, the program terminates itself.

>**If a driver supports small buffer sizes (<10ms buffers), will all applications in Windows 10 automatically use small buffers to render and capture audio?**
>
>No. By default, all applications in Windows 10 will use 10ms buffers to render and capture audio. If an application needs to use small buffers, then it needs to use the new AudioGraph settings or the WASAPI IAudioClient3 interface, in order to do so. However, if one application in Windows 10 requests the usage of small buffers, then the Audio Engine will start transferring audio using that particular buffer size. In that case, all applications that use the same endpoint and mode will automatically switch to that small buffer size. When the low latency application exits, the Audio Engine will switch to 10ms buffers again.

quote: https://docs.microsoft.com/en-us/windows-hardware/drivers/audio/low-latency-audio