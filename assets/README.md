# Arknights Assets Toolkit

A comprehensive toolkit for downloading and unpacking Arknights game assets.

## Table of Contents
- [Requirements](#requirements)
- [Installation](#installation)
- [Asset Downloader](#asset-downloader)
  - [How it Works](#how-it-works)
  - [Usage](#usage)
  - [Arguments](#arguments)
- [Asset Unpacker](#asset-unpacker)
  - [Building](#building)
  - [Usage](#usage-1)
  - [Arguments](#arguments-1)
- [Helper Utilities](#helper-utilities)
  - [Combining Alpha Channels](#combining-alpha-channels)
- [Credits](#credits)

## Requirements

### For Asset Downloader
- Python 3.8+
- Required Python packages (see [Installation](#installation))

### For Asset Unpacker
- Python 3.8+
- Poetry (for dependency management)
- On Windows: Python build tools
- On MacOS: Xcode Command Line Tools

## Installation

### Installing the Asset Downloader

1. Clone this repository:
   ```
   git clone https://github.com/yourusername/arknights-assets-toolkit.git
   cd arknights-assets-toolkit
   ```

2. Install required packages for the downloader:
   ```
   pip install -r assets/downloader/requirements.txt
   ```
   
   This will install:
   - bson ~= 0.5
   - keyboard >= 0.13.5
   - requests >= 2.25.1
   - UnityPy ~= 1.22
   - pycryptodome ~= 3.22
   - tqdm >= 4.61.0
   - Pillow ~= 9.5

### Installing the Asset Unpacker

1. Ensure you have Poetry installed:
   ```
   curl -sSL https://install.python-poetry.org | python3 -
   ```

2. Navigate to the unpacker directory:
   ```
   cd assets/unpacker/unpacker
   ```

3. Install dependencies with Poetry:
   ```
   poetry install
   ```

## Asset Downloader

### How it Works

The downloader script connects to the Arknights game servers to retrieve asset data. It can download from either the Official or Bilibili servers. The downloader automatically resolves the asset versions and manages the download process.

The script features:
- Multi-threaded downloading
- Progress tracking
- Resume capability
- Selective downloading by asset category

### Usage

Run the downloader with:

```
python assets/downloader/ark-downloader.py [options]
```

#### Interactive Mode

When run without arguments, the script enters interactive mode where you can:
1. Choose between downloading all assets or specific categories
2. Navigate through categories using arrow keys
3. Select/deselect categories with right/left arrows
4. Confirm your selection with the ESC key

### Arguments

- `--server`: Choose server type (0 for Official, 1 for Bilibili)
- `--output`: Specify the output directory
- `--threads`: Number of download threads (default: 6)
- `--categories`: Specify categories to download (comma-separated list)

Example:
```
python assets/downloader/ark-downloader.py --server 0 --output ArkAssets --threads 8 --categories other,charpack,skinpack
```

## Asset Unpacker

The unpacker extracts and processes the downloaded Arknights assets.

### Building

#### On Windows

Run the build script:
```
cd assets/unpacker/unpacker
python Build.py
```

#### On MacOS

MacOS requires a special build script:
```
cd assets/unpacker/unpacker
chmod +x build_macos.sh
./build_macos.sh
```

Note: Linux is not officially supported.

The build process will create a `build` folder containing the executable in `build/dist/ArkUnpacker-v4.0.0`.

### Usage

After building, run the unpacker with:

```
./build/dist/ArkUnpacker-v4.0.0 [options]
```

### Arguments

Basic unpacking:
```
./build/dist/ArkUnpacker-v4.0.0 -m ab -i ../ArkAssets --image --text --audio --spine -o Unpacked
```

- `-m, --mode`: Unpacking mode (`ab` for AssetBundle)
- `-i, --input`: Input directory containing downloaded assets
- `-o, --output`: Output directory for unpacked assets
- Resource type flags:
  - `--image`: Extract image assets
  - `--text`: Extract text assets
  - `--audio`: Extract audio assets
  - `--spine`: Extract Spine animation models
- `--separate`: Group exports by source file
- `--no-del`: Don't delete existing files in output directory
- `--resume`: Resume from last interrupted position
- `--skip-problematic`: Skip files that caused problems previously
- `--timeout`: Set processing timeout per file (seconds)

## Helper Utilities

### Combining Alpha Channels

Many Arknights images have separate alpha channel files. The toolkit includes a utility to combine these:

```
python assets/unpacker/helper/combine_alpha.py --input-dir Unpacked --delete-alpha
```

#### Arguments

- `--input-dir`: Directory containing unpacked images
- `--output-dir`: Directory for combined images (optional)
- `--overwrite`: Overwrite existing files
- `--delete-alpha`: Delete alpha files after combining
- `--threads`: Number of processing threads

## Credits

This toolkit is based on several open-source projects with slight modifications:

- [isHarryh/Ark-Unpacker](https://github.com/isHarryh/Ark-Unpacker) - The primary asset unpacker for Arknights resources
- [isHarryh/Ark-FBS-Py](https://github.com/isHarryh/Ark-FBS-Py) - Tools for handling Arknights FlatBuffers encoded data
- [ChaomengOrion/ArkAssetsTool](https://github.com/ChaomengOrion/ArkAssetsTool) - The foundation for the download script

The original code was created by these developers, and only slight modifications have been made to adapt it for this toolkit. 