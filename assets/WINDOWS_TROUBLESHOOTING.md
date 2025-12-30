# Windows Build Troubleshooting Guide

Complete guide for building the Arknights Assets toolchain on Windows.

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Common Issues and Solutions](#common-issues-and-solutions)
4. [Complete Setup Guide](#complete-setup-guide)
5. [FMOD Installation](#fmod-installation)
6. [Verification Steps](#verification-steps)
7. [Quick Reference](#quick-reference)

---

## Overview

This project was originally developed on macOS. Building on Windows requires specific toolchain setup due to differences in compilers, linkers, and library formats.

**Key Requirements:**
- **Rust:** 1.70.0 or later
- **Toolchain:** MinGW-w64 (GNU toolchain) - **NOT MSVC**
- **FMOD:** Engine 2.02.22 (for audio extraction)
- **Node.js:** For running `build.js`

---

## Prerequisites

### Required Software

1. **Rust (1.70.0+)**
   - Download: https://rustup.rs/
   - Install with default settings
   - Verify: `rustc --version`

2. **Node.js**
   - Download: https://nodejs.org/
   - Verify: `node --version`

3. **MSYS2 + MinGW-w64**
   - Download: https://www.msys2.org/
   - Required for MinGW toolchain
   - See [Complete Setup Guide](#complete-setup-guide)

4. **FMOD Engine 2.02.22**
   - Download: https://www.fmod.com/download (requires free account)
   - Required version: 2.02.22 or any 2.02.xx
   - See [FMOD Installation](#fmod-installation)

---

## Common Issues and Solutions

### Issue 1: "Cargo is not installed"

**Error:**
```
ERROR: Cargo is not installed (should come with Rust)
```

**Cause:** Rust installed but cargo not in PATH

**Solution:**
```powershell
# Verify Rust installation
rustc --version
cargo --version

# If cargo not found, add to PATH:
$env:Path += ";$env:USERPROFILE\.cargo\bin"
```

---

### Issue 2: "linking with `link.exe` failed" (Git's link conflicting)

**Error:**
```
link: extra operand 'file.o'
note: program not found
```

**Cause:** Git Bash's `link.exe` is being used instead of MSVC linker

**Solution:** Use MinGW instead of MSVC (see [Complete Setup Guide](#complete-setup-guide))

---

### Issue 3: "linker `link.exe` not found" (MSVC)

**Error:**
```
note: the msvc targets depend on the msvc linker but `link.exe` was not found
note: please ensure that Visual Studio 2017 or later is installed
```

**Cause:** MSVC toolchain requires Visual Studio Build Tools which is ~8 GB download

**Solution:** Use MinGW instead (simpler, smaller, no Visual Studio needed)

---

### Issue 4: "cannot find -lfmod_vc" (MinGW + FMOD)

**Error:**
```
ld.exe: cannot find -lfmod_vc: No such file or directory
ld.exe: cannot find -lfmodstudio_vc: No such file or directory
```

**Cause:**
- FMOD libraries need to be created for MinGW
- The Rust `libfmod` crate expects MSVC-style naming

**Solution:** See [FMOD Installation](#fmod-installation) section

---

### Issue 5: "error calling dlltool 'dlltool.exe': program not found"

**Error:**
```
error: error calling dlltool 'dlltool.exe': program not found
```

**Cause:** MinGW-w64 toolchain not installed or not in PATH

**Solution:**
1. Install MinGW via MSYS2 (see [Complete Setup Guide](#complete-setup-guide))
2. Add `C:\msys64\mingw64\bin` to Windows PATH
3. Restart terminal and verify: `dlltool --version`

---

## Complete Setup Guide

### Step 1: Install MSYS2

1. Download from https://www.msys2.org/
2. Run `msys2-x86_64-latest.exe`
3. Install to default location: `C:\msys64`
4. Complete installation

### Step 2: Update MSYS2 and Install MinGW

1. Open **"MSYS2 MSYS"** from Start Menu
2. Update package database:
   ```bash
   pacman -Syu
   ```
3. Close terminal when prompted, reopen "MSYS2 MSYS", and run again:
   ```bash
   pacman -Syu
   ```
4. Install MinGW-w64 toolchain:
   ```bash
   pacman -S mingw-w64-x86_64-toolchain
   ```
   Press Enter to install all packages

### Step 3: Add MinGW to Windows PATH

1. Press **Windows Key + R**
2. Type: `sysdm.cpl` and press Enter
3. Click **Advanced** tab → **Environment Variables**
4. Under "System variables", find and select **Path**
5. Click **Edit** → **New**
6. Add: `C:\msys64\mingw64\bin`
7. Click **OK** on all dialogs

**Alternative (PowerShell):**
```powershell
[Environment]::SetEnvironmentVariable('Path',
  [Environment]::GetEnvironmentVariable('Path', 'User') + ';C:\msys64\mingw64\bin',
  'User')
```

### Step 4: Switch Rust to GNU Toolchain

1. Close ALL terminal windows
2. Open a NEW PowerShell
3. Install GNU toolchain (if not already installed):
   ```powershell
   rustup toolchain install stable-x86_64-pc-windows-gnu
   ```
4. Set as default:
   ```powershell
   rustup default stable-x86_64-pc-windows-gnu
   ```
5. Verify:
   ```powershell
   rustup show
   # Should show: stable-x86_64-pc-windows-gnu (default)
   ```

### Step 5: Verify Installation

Close all terminals, open a fresh PowerShell, and verify:

```powershell
# Check Rust
rustc --version
cargo --version

# Check MinGW tools
gcc --version
dlltool --version

# Check toolchain
rustup show
# Should show: stable-x86_64-pc-windows-gnu (default)
```

All commands should succeed.

---

## FMOD Installation

FMOD is required for audio extraction. The project uses **FMOD Engine 2.02.22**.

### Why FMOD Installation is Complex on Windows with MinGW

- FMOD officially provides only MSVC libraries (`.lib` files)
- MinGW requires GNU-style import libraries (`.a` files)
- We must create these manually using MinGW tools

### Installation Steps

#### 1. Download FMOD Engine

1. Go to https://www.fmod.com/download
2. Create a free account (required)
3. Download **FMOD Engine 2.02** for Windows
   - Any 2.02.xx version works (2.02.22, 2.02.25, 2.02.32, etc.)
   - Do NOT use 2.00.xx or 2.01.xx (too old)
   - Do NOT use 2.03+ (may have breaking changes)

#### 2. Extract Required DLL Files

After installation, locate these files:

**FMOD Core API:** `<FMOD_Install>\api\core\lib\x64\`
- `fmod.dll` - Release version (required)
- `fmodL.dll` - Debug version (optional)

**FMOD Studio API:** `<FMOD_Install>\api\studio\lib\x64\`
- `fmodstudio.dll` - Release version (required)
- `fmodstudioL.dll` - Debug version (optional)

#### 3. Create MinGW Import Libraries

We've provided `create-fmod-libs.bat` to automate this.

**Method A: Use the Automated Script (Recommended)**

1. Copy the 4 DLL files to a temporary folder
2. Copy `create-fmod-libs.bat` to the same folder
3. Run the script:
   ```cmd
   create-fmod-libs.bat
   ```
4. This will create:
   - `libfmod.a`
   - `libfmodstudio.a`
   - `libfmodL.a` (if fmodL.dll exists)
   - `libfmodstudioL.a` (if fmodstudioL.dll exists)

**Method B: Manual Creation**

```bash
# Add MinGW tools to PATH
set PATH=C:\msys64\mingw64\bin;%PATH%

# Generate .def files from DLLs
gendef fmod.dll
gendef fmodstudio.dll

# Create import libraries
dlltool -k --output-lib libfmod.a --def fmod.def --dllname fmod.dll
dlltool -k --output-lib libfmodstudio.a --def fmodstudio.def --dllname fmodstudio.dll
```

#### 4. Install to Project

Copy files to: `unity-rs\src\resources\FMOD\Windows\x64\`

**Required files:**
- `fmod.dll`
- `fmodstudio.dll`
- `libfmod.a`
- `libfmodstudio.a`

#### 5. Create MSVC-Named Copies (CRITICAL!)

The `libfmod` Rust crate expects MSVC naming conventions. Create copies:

```cmd
cd unity-rs\src\resources\FMOD\Windows\x64

copy libfmod.a libfmod_vc.a
copy libfmodstudio.a libfmodstudio_vc.a
```

**Why?** The Rust crate links with `-lfmod_vc`, which looks for `libfmod_vc.a` (or `fmod_vc.lib` on MSVC).

### Verify FMOD Installation

Your `unity-rs\src\resources\FMOD\Windows\x64\` folder should contain:

```
fmod.dll
fmodstudio.dll
libfmod.a
libfmod_vc.a          ← Copy of libfmod.a
libfmodstudio.a
libfmodstudio_vc.a    ← Copy of libfmodstudio.a
```

---

## Verification Steps

After completing all setup:

### 1. Verify Tools

```powershell
# Rust toolchain
rustup show
# Should show: stable-x86_64-pc-windows-gnu (default)

# Compilers and tools
gcc --version
dlltool --version
cargo --version
```

### 2. Verify FMOD

```powershell
dir unity-rs\src\resources\FMOD\Windows\x64
```

Should list:
- `fmod.dll`
- `fmodstudio.dll`
- `libfmod_vc.a`
- `libfmodstudio_vc.a`

### 3. Run the Build

```powershell
cd C:\Users\ianle\Documents\myrtle\assets
node build.js
```

Or if PATH is not updated in current session:

```cmd
build-mingw.bat
```

### Expected Output

```
✓ Checking rust installation...
  Found Rust version: 1.92.0
✓ Rust version is sufficient
✓ Installing Rust...
  Found Cargo version: 1.92.0
✓ Cargo is available
✓ Checking FMOD library...
  Found FMOD at: C:\Users\ianle\Documents\myrtle\assets\unity-rs\src\resources\FMOD\Windows\x64
✓ FMOD library is available.

✓ Building unity-rs library...
  Building in release mode (optimized)...
  Compiling...
  [builds successfully]
```

---

## Quick Reference

### Essential Files (Keep These)

- **`build.js`** - Main build script
- **`build-mingw.bat`** - Convenience script to run build with correct PATH
- **`create-fmod-libs.bat`** - Tool to create FMOD import libraries
- **`FMOD_INSTALLATION_GUIDE.md`** - Detailed FMOD setup instructions
- **`MINGW_SETUP_GUIDE.md`** - MinGW installation guide
- **`WINDOWS_TROUBLESHOOTING.md`** - This file

### Key Commands

```powershell
# Switch to GNU toolchain
rustup default stable-x86_64-pc-windows-gnu

# Verify toolchain
rustup show

# Build project
node build.js

# Or with PATH fix
build-mingw.bat

# Check for updates
rustup update
```

### Important Paths

| What | Path |
|------|------|
| MinGW binaries | `C:\msys64\mingw64\bin` |
| Cargo binaries | `%USERPROFILE%\.cargo\bin` |
| FMOD resources | `unity-rs\src\resources\FMOD\Windows\x64\` |
| Build output | `*/target/release/` |

### Environment Variables

```powershell
# Add MinGW to PATH (current session only)
$env:Path = "C:\msys64\mingw64\bin;" + $env:Path

# Add Cargo to PATH (current session only)
$env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
```

---

## Need More Help?

### Check These Files

1. **`MINGW_SETUP_GUIDE.md`** - Step-by-step MinGW installation
2. **`FMOD_INSTALLATION_GUIDE.md`** - Detailed FMOD setup
3. **Unity-rs README** - `unity-rs/README.md`
4. **Unpacker README** - `unpacker/README.md`

### Common Resources

- **MSYS2 Documentation:** https://www.msys2.org/docs/
- **Rust Windows Guide:** https://rust-lang.github.io/rustup/installation/windows.html
- **FMOD Downloads:** https://www.fmod.com/download
- **MinGW-w64:** https://www.mingw-w64.org/

### Debug Information

If you encounter issues, gather this information:

```powershell
# System info
rustc --version --verbose
cargo --version
gcc --version
node --version

# Toolchain info
rustup show
rustup toolchain list

# Path info
echo $env:Path

# FMOD files
dir unity-rs\src\resources\FMOD\Windows\x64
```

---

## Architecture Notes

### Why MinGW Instead of MSVC?

**MSVC Issues Encountered:**
- Git Bash's `link.exe` conflicts with MSVC linker
- Requires Visual Studio Build Tools (~8 GB download)
- Requires Windows SDK installation
- Complex PATH management

**MinGW Benefits:**
- Self-contained toolchain (~1 GB)
- No Visual Studio required
- Standard GNU toolchain (cross-platform consistency)
- Works well with the project's Unix-oriented build scripts

### FMOD Library Format Differences

| Platform | Library Format | Linker Flag | MinGW Support |
|----------|---------------|-------------|---------------|
| MSVC | `.lib` | `/link foo.lib` | ❌ No |
| MinGW | `.a` | `-lfoo` | ✅ Yes |
| macOS | `.dylib` | `-lfoo` | N/A |
| Linux | `.so` | `-lfoo` | N/A |

FMOD officially provides only MSVC `.lib` files for Windows. We create MinGW-compatible `.a` files using:
1. `gendef` - Generates `.def` (definition) files from DLLs
2. `dlltool` - Creates import libraries from `.def` files

---

## Version History

- **Initial macOS development** - build.js created for macOS
- **Windows support added** - MinGW toolchain support
- **FMOD 2.02.22** - Updated from 2.00.10 to match `libfmod` crate requirements
- **Current** - Full Windows MinGW support with FMOD

---

## Summary Checklist

Before building, ensure:

- [ ] MSYS2 installed at `C:\msys64`
- [ ] MinGW-w64 toolchain installed via `pacman -S mingw-w64-x86_64-toolchain`
- [ ] `C:\msys64\mingw64\bin` added to Windows PATH
- [ ] Rust toolchain set to `stable-x86_64-pc-windows-gnu`
- [ ] FMOD DLLs installed to `unity-rs\src\resources\FMOD\Windows\x64\`
- [ ] FMOD import libraries (`.a`) created and copied
- [ ] MSVC-named copies created (`libfmod_vc.a`, `libfmodstudio_vc.a`)
- [ ] Terminal restarted to pick up PATH changes
- [ ] `gcc --version` and `dlltool --version` work

Then run: `node build.js`

---

**Last Updated:** 2025-12-29

---

## Running the Built Executables

### Issue: STATUS_DLL_NOT_FOUND (Exit Code 0xc0000135)

After building successfully, running the executables directly may fail with:

error: process didn't exit successfully (exit code: 0xc0000135, STATUS_DLL_NOT_FOUND)


**Cause:** The executables need runtime DLLs that aren't in your PATH:
- **python313.dll** (from Python installation) - required by pyo3
- **fmod.dll, fmodstudio.dll** (from FMOD) - required by libfmod
- **MinGW runtime DLLs** (libgcc_s_seh-1.dll, libwinpthread-1.dll, etc.)

### Solution: Use the Helper Scripts

I've created  helper scripts in each project directory:

**Downloader:**
cmd
cd downloader
run.bat --help


**Unpacker:**
cmd
cd unpacker
run.bat --help


These scripts automatically add the necessary DLL directories to PATH before running.

### Alternative: Add to System PATH Permanently

Add these directories to your Windows PATH:
1. C:\Users\ianle\AppData\Local\Programs\Python\Python313
2. C:\Users\ianle\Documents\myrtle\assets\unity-rs\src\resources\FMOD\Windows\x64
3. C:\msys64\mingw64\bin (already added during setup)

Then you can run cargo run directly without the helper scripts.

### Manual PATH Setup (Per Session)

**PowerShell:**
powershell
:Path
:Path
cd downloader
cargo run -- --help


**Command Prompt:**
cmd
set PATH=C:\Users\ianle\AppData\Local\Programs\Python\Python313;%PATH%
set PATH=C:\Users\ianle\Documents\myrtle\assets\unity-rs\src\resources\FMOD\Windows\x64;%PATH%
cd downloader
cargo run -- --help


---

**Last Updated:** 2025-12-29
