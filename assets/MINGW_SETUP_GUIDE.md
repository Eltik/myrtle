# MinGW Setup Guide for Windows Build

## Step-by-Step Installation

### Step 1: Download MSYS2

1. Open your browser and go to: **https://www.msys2.org/**
2. Download: **msys2-x86_64-latest.exe**
3. Run the installer
4. Install to default location: `C:\msys64`
5. Click through the installer (keep all defaults)
6. When finished, **UNCHECK** "Run MSYS2 now" (we'll do it manually)

### Step 2: Update MSYS2 Package Database

1. Open Start Menu → Search for **"MSYS2 MSYS"** → Open it
2. In the MSYS2 terminal, run:
   ```bash
   pacman -Syu
   ```
3. When it asks `Proceed with installation? [Y/n]` → Type **Y** and press Enter
4. If it says to close the terminal, **close it** and open a new **MSYS2 MSYS** terminal
5. Run the update again:
   ```bash
   pacman -Syu
   ```

### Step 3: Install MinGW-w64 Toolchain

1. In the MSYS2 terminal, run:
   ```bash
   pacman -S mingw-w64-x86_64-toolchain
   ```
2. When it asks `Enter a selection (default=all)` → Just press **Enter**
3. When it asks `Proceed with installation? [Y/n]` → Type **Y** and press Enter
4. Wait for installation (~500 MB download)

### Step 4: Add MinGW to Windows PATH

1. Press **Windows Key + R**
2. Type: `sysdm.cpl` and press Enter
3. Click **Advanced** tab
4. Click **Environment Variables** button
5. Under "System variables", find and select **Path**
6. Click **Edit**
7. Click **New**
8. Add: `C:\msys64\mingw64\bin`
9. Click **OK** on all dialogs

### Step 5: Verify Installation

1. **Close ALL PowerShell/CMD windows**
2. Open a **NEW PowerShell** window
3. Run these commands to verify:
   ```powershell
   gcc --version
   dlltool --version
   ```

   You should see version information for both.

### Step 6: Switch Rust Toolchain

In PowerShell, run:
```powershell
rustup default stable-x86_64-pc-windows-gnu
```

### Step 7: Build Your Project!

```powershell
cd C:\Users\ianle\Documents\myrtle\assets
node build.js
```

---

## Troubleshooting

### "gcc not found" after Step 5

- Make sure you closed ALL terminal windows after adding to PATH
- Open a FRESH PowerShell window
- Try again

### Build still fails

Run with verbose output:
```powershell
$env:RUST_BACKTRACE="full"
node build.js
```

---

## What Gets Installed

- **MSYS2**: Package manager for Windows (~100 MB)
- **MinGW-w64 toolchain**: GCC compiler, linker, tools (~500 MB)
- **Total disk space**: ~1 GB

Much lighter than Visual Studio Build Tools (~8 GB)!
