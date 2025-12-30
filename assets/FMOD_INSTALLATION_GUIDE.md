# FMOD Installation Guide for MinGW (Windows)

## Required Version

Your codebase requires: **FMOD Engine 2.02.22** (or any 2.02.xx version)

- Crate: `libfmod = "2.222"` → FMOD 2.02.22
- Download: https://www.fmod.com/download

---

## Installation Steps

### Step 1: Download FMOD Engine

1. Go to https://www.fmod.com/download
2. Create a free account (required to download)
3. Download **FMOD Engine 2.02** for Windows
   - Click "FMOD Engine" section
   - Select Windows platform
   - Download the installer or ZIP

### Step 2: Extract FMOD Libraries

After installing/extracting FMOD, locate these folders:

**FMOD Core API:**
```
<FMOD Install>\api\core\lib\x64\
```

Contains:
- `fmod.dll` - Release version
- `fmodL.dll` - Debug version with logging

**FMOD Studio API:**
```
<FMOD Install>\api\studio\lib\x64\
```

Contains:
- `fmodstudio.dll` - Release version
- `fmodstudioL.dll` - Debug version

### Step 3: Create MinGW Import Libraries

FMOD doesn't provide MinGW-compatible `.a` libraries, so we need to create them.

1. **Copy all 4 DLL files** to a temporary folder (or work in place)

2. **Run the script** I created:
   ```cmd
   cd C:\Users\ianle\Documents\myrtle\assets
   create-fmod-libs.bat
   ```

   Or manually run these commands:
   ```cmd
   REM Add MinGW to PATH
   set PATH=C:\msys64\mingw64\bin;%PATH%

   REM Generate .def files
   gendef fmod.dll
   gendef fmodstudio.dll

   REM Create import libraries
   dlltool -k --output-lib libfmod.a --def fmod.def --dllname fmod.dll
   dlltool -k --output-lib libfmodstudio.a --def fmodstudio.def --dllname fmodstudio.dll
   ```

### Step 4: Install Files to Project

Copy the files to your project's FMOD resource directory:

```
C:\Users\ianle\Documents\myrtle\assets\unity-rs\src\resources\FMOD\Windows\x64\
```

**Files to copy:**
- `fmod.dll`
- `fmodstudio.dll`
- `libfmod.a`
- `libfmodstudio.a`

Optional (for debug builds):
- `fmodL.dll`
- `fmodstudioL.dll`
- `libfmodL.a`
- `libfmodstudioL.a`

### Step 5: Fix Library Naming (IMPORTANT!)

The `libfmod` Rust crate looks for MSVC library names (`fmod_vc`), but we have MinGW libraries.

**Two options:**

**Option A: Create symlinks/copies with MSVC names**
```cmd
cd C:\Users\ianle\Documents\myrtle\assets\unity-rs\src\resources\FMOD\Windows\x64

REM Copy with MSVC-style names
copy libfmod.a libfmod_vc.a
copy libfmodstudio.a libfmodstudio_vc.a
```

**Option B: Set environment variable** (tells linker to find them)
```cmd
REM This tells the linker to treat -lfmod_vc as -lfmod
set RUSTFLAGS=-L C:\Users\ianle\Documents\myrtle\assets\unity-rs\src\resources\FMOD\Windows\x64
```

**Recommended: Use Option A** (simpler and permanent)

### Step 6: Test the Build

After copying files and creating the MSVC-named copies:

```powershell
cd C:\Users\ianle\Documents\myrtle\assets
node build.js
```

---

## Quick Summary

```bash
# 1. Download FMOD Engine 2.02.xx from https://www.fmod.com/download
# 2. Extract DLLs from api/core/lib/x64/ and api/studio/lib/x64/
# 3. Create import libraries:
cd <folder-with-dlls>
gendef fmod.dll
gendef fmodstudio.dll
dlltool -k --output-lib libfmod.a --def fmod.def --dllname fmod.dll
dlltool -k --output-lib libfmodstudio.a --def fmodstudio.def --dllname fmodstudio.dll

# 4. Copy to project:
copy *.dll C:\Users\ianle\Documents\myrtle\assets\unity-rs\src\resources\FMOD\Windows\x64\
copy *.a C:\Users\ianle\Documents\myrtle\assets\unity-rs\src\resources\FMOD\Windows\x64\

# 5. Create MSVC-named copies:
cd C:\Users\ianle\Documents\myrtle\assets\unity-rs\src\resources\FMOD\Windows\x64
copy libfmod.a libfmod_vc.a
copy libfmodstudio.a libfmodstudio_vc.a

# 6. Build!
cd C:\Users\ianle\Documents\myrtle\assets
node build.js
```

---

## Troubleshooting

### "gendef not found"
- Make sure MinGW-w64 is installed via MSYS2
- Add `C:\msys64\mingw64\bin` to PATH
- Restart terminal

### "cannot find -lfmod_vc"
- You forgot Step 5! Create copies with MSVC names:
  ```
  copy libfmod.a libfmod_vc.a
  copy libfmodstudio.a libfmodstudio_vc.a
  ```

### Different FMOD version?
- 2.02.xx versions should work (e.g., 2.02.22, 2.02.25, 2.02.32)
- Don't use 2.00.xx or 2.01.xx (too old)
- Don't use 2.03+ (might have API changes)

---

## Why This is Needed

- **libfmod crate 2.222** = FMOD Engine **2.02.22**
- FMOD officially provides only MSVC libraries for Windows (.lib files)
- MinGW needs `.a` import libraries
- We create them using `gendef` + `dlltool` from the provided DLLs
- The Rust crate expects MSVC naming (`fmod_vc`), so we create copies
