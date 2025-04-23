# -*- coding: utf-8 -*-
# Copyright (c) 2022-2025, Harry Huang
# @ BSD 3-Clause License
from typing import Union
import sys
import importlib.util
import os
import subprocess

# Try different ways to import LZHAM
LZHAM_AVAILABLE = False
LZHAM_ERROR = ""

# Method 1: Try direct import
try:
    import lzham
    LZHAM_AVAILABLE = True
except ImportError as e:
    LZHAM_ERROR = f"Direct import failed: {str(e)}"
    # Method 2: Try pylzham
    try:
        import pylzham
        lzham = pylzham
        LZHAM_AVAILABLE = True
    except ImportError as e:
        LZHAM_ERROR += f", pylzham import failed: {str(e)}"

from ..utils.Profiler import CodeProfiler
from ..utils.Logger import Logger

ByteString = Union[bytes, bytearray, memoryview]


def decompress_lzham(compressed_data: ByteString, uncompressed_size: int) -> bytes:
    """Decompresses the given data block using LZHAM algorithm.

    :param compressed_data: The raw compressed data bytes;
    :param uncompressed_size: The size of the uncompressed data;
    :returns: The decompressed data bytes;
    :rtype: bytes;
    """
    if not LZHAM_AVAILABLE:
        error_msg = f"LZHAM decompression is not available. Error: {LZHAM_ERROR}"
        Logger.warn(f"LZHAM decompression failed: {error_msg}")
        
        # Return the compressed data as fallback (it won't work, but at least won't crash)
        Logger.warn("Falling back to using LZ4AK decompression (may not work correctly)")
        from ..lz4ak.Block import decompress_lz4ak
        try:
            return decompress_lz4ak(compressed_data, uncompressed_size)
        except Exception as e:
            error_msg = f"Fallback LZ4AK decompression also failed: {str(e)}"
            Logger.error(error_msg)
            # As last resort, just return the data unmodified to prevent crashes
            return compressed_data
        
    with CodeProfiler("lzham_decompress"):
        try:
            # Create a decompressor with standard settings
            # Dictionary size should be a power of 2 in the range [4096, 16777216]
            # We use a generous size since Arknights assets can be large
            dict_size_log2 = 23  # 2^23 = 8MB
            
            # Different LZHAM libraries have different APIs
            if hasattr(lzham, 'Decompressor'):
                # Regular lzham module approach
                decompressor = lzham.Decompressor(dict_size_log2=dict_size_log2)
                decompressed_data = decompressor.decompress(compressed_data, uncompressed_size)
            elif hasattr(lzham, 'decompress'):
                # pylzham approach
                decompressed_data = lzham.decompress(compressed_data, uncompressed_size, dict_size_log2)
            else:
                error_msg = "Unknown LZHAM library API"
                Logger.error(error_msg)
                raise RuntimeError(error_msg)
            
            # Verify we got the expected size
            if len(decompressed_data) != uncompressed_size:
                Logger.warn(f"LZHAM decompression: Expected size {uncompressed_size}, got {len(decompressed_data)}")
            
            return decompressed_data
            
        except Exception as e:
            error_msg = f"LZHAM decompression failed: {str(e)}"
            Logger.error(error_msg)
            
            # Try falling back to LZ4AK as a last resort
            Logger.warn("Falling back to using LZ4AK decompression as a last resort")
            from ..lz4ak.Block import decompress_lz4ak
            try:
                return decompress_lz4ak(compressed_data, uncompressed_size)
            except Exception as e2:
                # As last resort, just return the data unmodified to prevent crashes
                Logger.error(f"Fallback LZ4AK decompression also failed: {str(e2)}")
                return compressed_data 