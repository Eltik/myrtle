# Assets

A comprehensive toolkit for downloading and unpacking Arknights game assets.

## Table of Contents
- [Features](#features)
- [Prerequisites](#prerequisites)
  - [For Asset Downloader](#for-asset-downloader)
  - [For Asset Unpacker](#for-asset-unpacker)
- [Installation](#installation)
- [Development](#development)
  - [Asset Downloader](#asset-downloader)
  - [Asset Unpacker](#asset-unpacker)
- [Project Structure](#project-structure)
- [Helper Utilities](#helper-utilities)
  - [Combining Alpha Channels](#combining-alpha-channels)
- [Credits](#credits)

## Features

- **Asset Downloader**: Tool to download game assets from official servers
- **Asset Unpacker**: Tool to extract and process downloaded assets
- **Multi-threaded downloading**: Efficient parallel download capabilities
- **Resume capability**: Support for resuming interrupted downloads
- **Selective downloading**: Download specific asset categories as needed
- **Cross-platform support**: Works on Windows and MacOS

## Prerequisites

### For Asset Downloader
- [Python 3.8+](https://www.python.org/downloads/)
- Required Python packages:
  - [`bson`](https://pypi.org/project/bson/) ~= 0.5
  - [`keyboard`](https://pypi.org/project/keyboard/) >= 0.13.5
  - [`requests`](https://pypi.org/project/requests/) >= 2.25.1
  - [`UnityPy`](https://pypi.org/project/UnityPy/) ~= 1.22
  - [`pycryptodome`](https://pypi.org/project/pycryptodome/) ~= 3.22
  - [`tqdm`](https://pypi.org/project/tqdm/) >= 4.61.0
  - [`Pillow`](https://pypi.org/project/Pillow/) ~= 9.5

### For Asset Unpacker
- [Python 3.8+](https://www.python.org/downloads/)
- [Poetry](https://python-poetry.org/docs/#installation) (for dependency management)
- On Windows: Python build tools
- On MacOS: Xcode Command Line Tools

## Installation

1. Move into the `./assets` directory:
```bash
cd assets
```

2. Install required packages for the downloader:
```bash
pip install -r assets/downloader/requirements.txt
```

3. For the unpacker, install Poetry:
```bash
curl -sSL https://install.python-poetry.org | python3 -
```

4. Install unpacker dependencies:
```bash
cd assets/unpacker/unpacker
poetry install
```

## Development

### Asset Downloader

The downloader script connects to the Arknights game servers to retrieve asset data. 
It can download from either the Official or Bilibili servers.

#### Usage

Run the downloader with:
```bash
python assets/downloader/ark-downloader.py [options]
```

#### Available Options

- `--server`: Choose server type (0 for Official, 1 for Bilibili)
- `--output`: Specify the output directory
- `--threads`: Number of download threads (default: 6)
- `--categories`: Specify categories to download (comma-separated list)

Example:
```bash
python assets/downloader/ark-downloader.py --server 0 --output ArkAssets --threads 8 --categories other,charpack,skinpack
```

### Asset Unpacker

The unpacker extracts and processes the downloaded Arknights assets.

#### Building

For Windows:
```bash
cd assets/unpacker/unpacker
python Build.py
```

For MacOS:
```bash
cd assets/unpacker/unpacker
chmod +x build_macos.sh
./build_macos.sh
```

The build process will create a `build` folder containing the executable.

#### Usage

After building, run the unpacker with:
```bash
./build/dist/ArkUnpacker-v4.0.0 [options]
```

Basic unpacking example:
```bash
./build/dist/ArkUnpacker-v4.0.0 -m ab -i ../ArkAssets --image --text --audio --spine -o Unpacked
```

#### Available Options

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

## Project Structure

```
assets/
├── downloader/
│   ├── ark-downloader.py     # Main downloader script
│   └── requirements.txt      # Downloader dependencies
├── unpacker/
│   ├── unpacker/             # Unpacker source code
│   │   ├── Build.py          # Windows build script
│   │   └── build_macos.sh    # MacOS build script
│   └── helper/
│       └── combine_alpha.py  # Alpha channel combiner tool
```

## Helper Utilities

### Combining Alpha Channels

Many Arknights images have separate alpha channel files. The toolkit includes a utility to combine these:

```bash
python assets/unpacker/helper/combine_alpha.py --input-dir Unpacked --delete-alpha
```

#### Available Options

- `--input-dir`: Directory containing unpacked images
- `--output-dir`: Directory for combined images (optional)
- `--overwrite`: Overwrite existing files
- `--delete-alpha`: Delete alpha files after combining
- `--threads`: Number of processing threads

### Finding Missing Assets

Sometimes when unpacking downloaded assets, there will be assets that did not get extracted. To find the missing assets, you can run the `find-assets.py` file:

```bash
python unpacker/missing/find-assets.py --find-missing --source-dir Input-Dir --extracted-dir Unpacked-Dir -o Output-Dir
```

This will compare the directory containing the original `.ab` files with the output directory and find the missing assets until finally writing a `missing-assets.json` file to a directory.

#### Available Options

- `--source-dir`: The directory containing the original resource files (aka `.ab` files)
- `--extracted-dir`: The directory containing all the unpacked assets
- `-o`: The directory where the `missing-assets.json` file gets written to

### Extract Targeted Assets

If you have a list of the missing assets using the helper script [above](#finding-missing-assets), then you can run the command below which uses the `Main.py` file.

```bash
python unpacker/Main.py -m ab -i Input-Dir -o Unpacked-Dir --target-list missing_assets.json --image --spine --text --audio --resume --skip-problematic --timeout <num>
```

#### Available Options

- `-m, --mode`: Unpacking mode (`ab` for AssetBundle)
- `-i, --input`: Input directory containing downloaded assets
- `-o, --output`: Output directory for unpacked assets
- `--target-list`: Path to the JSON file listing specific assets to extract
- Resource type flags:
  - `--image`: Extract image assets
  - `--text`: Extract text assets
  - `--audio`: Extract audio assets
  - `--spine`: Extract Spine animation models
- `--resume`: Resume from last interrupted position
- `--skip-problematic`: Skip files that caused problems previously
- `--timeout`: Set processing timeout per file (seconds)

## Credits

This toolkit is based on several open-source projects with slight modifications:

- [isHarryh/Ark-Unpacker](https://github.com/isHarryh/Ark-Unpacker) - The primary asset unpacker for Arknights resources
- [isHarryh/Ark-FBS-Py](https://github.com/isHarryh/Ark-FBS-Py) - Tools for handling Arknights FlatBuffers encoded data
- [ChaomengOrion/ArkAssetsTool](https://github.com/ChaomengOrion/ArkAssetsTool) - The foundation for the download script

The original code was created by these developers, and only slight modifications have been made to adapt it for this toolkit. 