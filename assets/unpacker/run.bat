@echo off
REM Helper script to run assets-unpacker with correct PATH

REM Add all required DLL directories to PATH
set "PATH=C:\Users\ianle\AppData\Local\Programs\Python\Python313;%PATH%"
set "PATH=C:\Users\ianle\Documents\myrtle\assets\unity-rs\src\resources\FMOD\Windows\x64;%PATH%"
set "PATH=C:\msys64\mingw64\bin;%PATH%"

REM Run cargo with all arguments passed to this script
cargo run -- %*
