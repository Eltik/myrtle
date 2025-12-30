@echo off
REM Script to create MinGW-compatible FMOD import libraries
REM Place this in the folder with fmod.dll and fmodstudio.dll

echo Creating MinGW import libraries for FMOD...
echo.

REM Add MinGW tools to PATH
set "PATH=C:\msys64\mingw64\bin;%PATH%"

REM Check if gendef and dlltool are available
where gendef >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: gendef not found. Make sure MinGW-w64 is installed and in PATH.
    pause
    exit /b 1
)

where dlltool >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: dlltool not found. Make sure MinGW-w64 is installed and in PATH.
    pause
    exit /b 1
)

REM Create .def files from DLLs
echo Creating .def files...
if exist fmod.dll (
    gendef fmod.dll
    echo   - Created fmod.def
) else (
    echo   WARNING: fmod.dll not found
)

if exist fmodL.dll (
    gendef fmodL.dll
    echo   - Created fmodL.def
) else (
    echo   WARNING: fmodL.dll not found
)

if exist fmodstudio.dll (
    gendef fmodstudio.dll
    echo   - Created fmodstudio.def
) else (
    echo   WARNING: fmodstudio.dll not found
)

if exist fmodstudioL.dll (
    gendef fmodstudioL.dll
    echo   - Created fmodstudioL.def
) else (
    echo   WARNING: fmodstudioL.dll not found
)

echo.
echo Creating .a import libraries...

REM Create import libraries from .def files
if exist fmod.def (
    dlltool -k --output-lib libfmod.a --def fmod.def --dllname fmod.dll
    echo   - Created libfmod.a
)

if exist fmodL.def (
    dlltool -k --output-lib libfmodL.a --def fmodL.def --dllname fmodL.dll
    echo   - Created libfmodL.a
)

if exist fmodstudio.def (
    dlltool -k --output-lib libfmodstudio.a --def fmodstudio.def --dllname fmodstudio.dll
    echo   - Created libfmodstudio.a
)

if exist fmodstudioL.def (
    dlltool -k --output-lib libfmodstudioL.a --def fmodstudioL.def --dllname fmodstudioL.dll
    echo   - Created libfmodstudioL.a
)

echo.
echo Done! Import libraries created successfully.
echo.
echo Next steps:
echo 1. Copy all .dll files to: unity-rs\src\resources\FMOD\Windows\x64\
echo 2. Copy all .a files to: unity-rs\src\resources\FMOD\Windows\x64\
echo.
pause
