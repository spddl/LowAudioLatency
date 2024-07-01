ECHO OFF
@REM cargo clean

:loop
CLS

IF exist "%~dp0\target\release\low_audio_latency.exe" (
    FOR /F "usebackq" %%A IN ('%~dp0\target\release\low_audio_latency.exe') DO SET /A beforeSize=%%~zA
) ELSE (
    SET /A beforeSize=0
)

cargo build --release

FOR /F "usebackq" %%A IN ('%~dp0\target\release\low_audio_latency.exe') DO SET /A size=%%~zA
SET /A diffSize = %size% - %beforeSize%
SET /A size=(%size%/1024)+1
IF "%diffSize%" EQU "0" (
    ECHO %size% kb
) ELSE (
    IF "%diffSize%" GTR "0" (
        ECHO %size% kb [+%diffSize% b]
    ) ELSE (
        ECHO %size% kb [%diffSize% b]
    )
)

PAUSE
GOTO loop