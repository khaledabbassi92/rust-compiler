@echo off
setlocal

set "CARGO=C:\Users\khaled\.cargo\bin\cargo.exe"
set "NASM=C:\Users\khaled\AppData\Local\bin\NASM\nasm.exe"
set "GPP=C:\Users\khaled\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.MCF.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin\g++.exe"

echo [1/4] Running compiler...
"%CARGO%" run
if errorlevel 1 (
    echo Compiler failed.
    pause
    exit /b 1
)

echo.
echo [2/4] Assembling...
"%NASM%" -f win64 output.asm -o output.obj
if errorlevel 1 (
    echo Assembly failed.
    pause
    exit /b 1
)

echo.
echo [3/4] Linking...
"%GPP%" -nostdlib output.obj -o output.exe -lkernel32 -Wl,-e,mainCRTStartup
if errorlevel 1 (
    echo Linking failed.
    pause
    exit /b 1
)

echo.
echo ===== PROGRAM OUTPUT =====
echo.

echo [4/4] Running generated program...
echo.

.\output.exe

echo.
pause