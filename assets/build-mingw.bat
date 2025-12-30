@echo off
REM Add MinGW to PATH and run build
set "PATH=C:\msys64\mingw64\bin;%PATH%"
node build.js %*
