# Myrtle Backend

Robust Arknights API inspired by ArkPRTS.

## Features

- Game data retrieval and parsing
- Asset downloading and unpacking
- LZ4 decompression support for Unity asset bundles
- Structured logging for better debugging
- Various Arknights-specific utilities

## Game Data

This backend interfaces with Arknights game data and assets.

## Asset Handling

The backend includes utilities for downloading and processing Arknights assets, including:

- Asset downloading from official servers
- Unity asset bundle unpacking
- Text, image, and audio asset extraction
- LZ4 decompression for compressed assets

## Configuration

The system can be configured using environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| ARKNIGHTS_LOG_LEVEL | Logging level (DEBUG, INFO, WARN, ERROR) | INFO |

## Development

### Prerequisites

- [Bun](https://bun.sh/) runtime
- Node.js 18+

### Setup

```bash
bun install
```

### Running

```bash
# Development
bun dev

# Production
bun start
```

### Building

```bash
bun run build
```

## LZ4 Decompression

The system now supports full LZ4 decompression for Unity asset bundles. This feature enables extracting game assets that are compressed using the LZ4 algorithm.

Implementation details:
- Detects LZ4 compression from Unity bundle headers
- Uses the lz4js JavaScript library for decompression (no native dependencies)
- Compatible with Bun runtime (avoids Node-API native modules)
- Implements full Unity asset bundle block handling:
  - Properly decompresses the blocks info data
  - Parses block metadata (sizes, compression flags)
  - Processes each data block individually
  - Handles both compressed and uncompressed blocks
- Provides detailed logging of the decompression process
- Includes fallback mechanism for handling edge cases

### Compatibility

The implementation uses a pure JavaScript LZ4 implementation (lz4js) which:
- Works natively with Bun without requiring Node-API or native modules
- Provides the same compression/decompression capabilities as the C++ implementation
- Has a simpler API that avoids buffer allocation issues

## Logging System

A structured logging system has been implemented to improve debugging capabilities:

- Multiple log levels (DEBUG, INFO, WARN, ERROR)
- Component-based logging for better context
- Color-coded output for improved readability
- Configurable verbosity via environment variables

You can set the log level using the `ARKNIGHTS_LOG_LEVEL` environment variable. 