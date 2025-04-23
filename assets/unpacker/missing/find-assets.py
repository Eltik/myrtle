#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# LZHAM Direct Extractor for Ark-Unpacker
# This script directly extracts LZHAM-compressed AB files using the existing lz4ak decompression code

import os
import sys
import json
import time
import shutil
import signal
import argparse
import traceback
import threading
import multiprocessing
from typing import List, Dict, Any, Optional

# Get the directory of the current script
script_dir = os.path.dirname(os.path.abspath(__file__))
# Calculate the path to the directory containing the 'src' folder
# (Script is assets/unpacker/missing, src is in assets/unpacker/unpacker/src)
src_parent_dir = os.path.abspath(os.path.join(script_dir, '..', 'unpacker'))
# Add the src parent directory to the Python path
if src_parent_dir not in sys.path:
    sys.path.insert(0, src_parent_dir)

try:
    import UnityPy
    from UnityPy.enums.BundleFile import CompressionFlags
    from UnityPy.helpers import CompressionHelper
except ImportError:
    print("Error: UnityPy not found. Installing...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "UnityPy"])
    import UnityPy
    from UnityPy.enums.BundleFile import CompressionFlags
    from UnityPy.helpers import CompressionHelper

# Import the lz4ak decompression module
# sys.path.append(os.path.dirname(os.path.abspath(__file__))) # Removed this line
try:
    from src.lz4ak.Block import decompress_lz4ak
    from src.utils.Logger import Logger
    from src.utils.GlobalMethods import print as custom_print, rmdir
    print = custom_print  # Use Ark-Unpacker's custom print function
    
    # Also try to import the true LZHAM decompression module
    has_lzham = False
    try:
        from src.lzham.Block import decompress_lzham
        has_lzham = True
        print("Found LZHAM decompression module - full LZHAM support enabled")
    except ImportError as e:
        has_lzham = False
        print(f"LZHAM decompression module not available: {str(e)}")
        print("Only LZ4AK (mislabeled as LZHAM) will be supported (sufficient for current game versions)")
except ImportError as e:
    print(f"Error importing required modules: {e}")
    print("Make sure the 'src' directory is present in the project root and accessible.")
    sys.exit(1)

# Apply the LZHAM patch directly
CompressionHelper.DECOMPRESSION_MAP[CompressionFlags.LZHAM] = decompress_lz4ak
print("Applied LZ4AK decompression patch to UnityPy for current LZHAM-labeled assets")

# Add true LZHAM support if available
if has_lzham:
    # Define a custom LZHAM compression flag (may need to be adjusted)
    LZHAM_COMPRESSION_FLAG = 4
    CompressionHelper.DECOMPRESSION_MAP[LZHAM_COMPRESSION_FLAG] = decompress_lzham
    print("Applied true LZHAM decompression patch to UnityPy for future releases")

# Helper to create directories safely
def ensure_dir(directory):
    """Create directory if it doesn't exist"""
    if not os.path.exists(directory):
        os.makedirs(directory, exist_ok=True)

class TimeoutError(Exception):
    """Exception raised when a function call times out"""
    pass

def run_with_timeout(func, args=(), kwargs={}, timeout_duration=60):
    """Run a function with a timeout"""
    result = None
    error = None
    timed_out = False
    
    def target():
        nonlocal result, error
        try:
            result = func(*args, **kwargs)
        except Exception as e:
            error = e
    
    thread = threading.Thread(target=target)
    thread.daemon = True
    
    start_time = time.time()
    thread.start()
    thread.join(timeout_duration)
    
    if thread.is_alive():
        timed_out = True
        # We can't forcibly terminate a thread in Python, but we can note that it timed out
        return None, TimeoutError(f"Function call timed out after {timeout_duration} seconds"), True
    
    if error:
        return None, error, False
        
    return result, None, False

def analyze_ab_file(ab_file: str, verbose: bool = False, timeout: int = 60) -> Dict:
    """
    Analyze an AB file to determine what types of assets it contains
    
    Returns a dictionary with asset information
    """
    if not os.path.exists(ab_file):
        print(f"File not found: {ab_file}", c=3)
        return {"error": "File not found", "path": ab_file}
    
    def _do_analyze():
        env = UnityPy.load(ab_file)
        
        # Count objects by type
        type_counts = {}
        object_names = []
        
        # Process each object in the bundle
        for obj in env.objects:
            # Count by type
            obj_type = obj.type.name
            if obj_type not in type_counts:
                type_counts[obj_type] = 0
            type_counts[obj_type] += 1
            
            # Collect some object names for debugging
            try:
                data = obj.read()
                if hasattr(data, "name") and data.name:
                    object_names.append(f"{obj_type}:{data.name}")
                elif hasattr(data, "m_Name") and data.m_Name:
                    object_names.append(f"{obj_type}:{data.m_Name}")
            except Exception as e:
                pass
        
        # Prepare analysis result
        result = {
            "path": ab_file,
            "types": type_counts,
            "total_objects": len(env.objects),
            "sample_names": object_names[:10] if object_names else []
        }
        
        # Prediction of content type
        if "Texture2D" in type_counts:
            result["likely_content"] = "images"
        elif "TextAsset" in type_counts:
            result["likely_content"] = "text"
        elif "AudioClip" in type_counts:
            result["likely_content"] = "audio"
        elif "Mesh" in type_counts and "Material" in type_counts:
            result["likely_content"] = "models"
        else:
            result["likely_content"] = "misc"
        
        return result
    
    try:
        result, error, timed_out = run_with_timeout(_do_analyze, timeout_duration=timeout)
        
        if timed_out:
            print(f"Analysis of {ab_file} timed out after {timeout} seconds", c=3)
            return {
                "error": "timeout",
                "path": ab_file,
                "timeout": timeout
            }
        
        if error:
            print(f"Error analyzing {ab_file}: {error}", c=1)
            if verbose:
                traceback.print_exc()
            return {"error": str(error), "path": ab_file}
            
        return result
    
    except Exception as e:
        print(f"Error analyzing {ab_file}: {e}", c=1)
        if verbose:
            traceback.print_exc()
        return {"error": str(e), "path": ab_file}

def raw_extract_objects(env, extract_dir):
    """
    Extract all objects from an environment using their raw data
    This bypasses the type-specific logic and just dumps everything
    """
    extraction_count = 0
    
    for obj in env.objects:
        try:
            # Skip container objects
            if obj.type.name in ["AssetBundle"]:
                continue
                
            # Try to read the object
            data = obj.read()
            
            # Try to determine a name
            name = None
            if hasattr(data, "name") and data.name:
                name = data.name
            elif hasattr(data, "m_Name") and data.m_Name:
                name = data.m_Name
            else:
                name = f"unknown_{obj.path_id}"
            
            # Clean the name for file system
            name = "".join(c if c.isalnum() or c in "._- " else "_" for c in name)
            
            # Type-specific handling
            if obj.type.name == "Texture2D":
                if hasattr(data, "image") and data.image:
                    dest = os.path.join(extract_dir, f"{name}.png")
                    try:
                        data.image.save(dest)
                        extraction_count += 1
                    except Exception as e:
                        print(f"Failed to save image {name}: {e}", c=3)
                
                # Also save the raw data as a fallback
                dest = os.path.join(extract_dir, f"{name}.tex")
                with open(dest, "wb") as f:
                    if hasattr(data, "m_RawData") and data.m_RawData:
                        f.write(data.m_RawData)
                        extraction_count += 1
                    elif hasattr(data, "data") and data.data:
                        f.write(data.data)
                        extraction_count += 1
            
            elif obj.type.name == "Sprite":
                if hasattr(data, "image") and data.image:
                    dest = os.path.join(extract_dir, f"{name}.png")
                    try:
                        data.image.save(dest)
                        extraction_count += 1
                    except Exception as e:
                        print(f"Failed to save sprite {name}: {e}", c=3)
            
            elif obj.type.name == "TextAsset":
                if hasattr(data, "script") and data.script:
                    ext = ".txt"
                    # Try to determine the correct extension
                    if name.endswith((".json", ".xml", ".csv", ".html", ".htm", ".yaml", ".yml")):
                        ext = ""
                    elif data.script.startswith(b"<?xml"):
                        ext = ".xml"
                    elif data.script.startswith(b"{") or data.script.startswith(b"["):
                        ext = ".json"
                    
                    dest = os.path.join(extract_dir, f"{name}{ext}")
                    with open(dest, "wb") as f:
                        f.write(data.script)
                        extraction_count += 1
            
            elif obj.type.name == "AudioClip":
                if hasattr(data, "samples") and data.samples:
                    for sample_name, sample_data in data.samples.items():
                        ext = sample_name.split(".")[-1] if "." in sample_name else "dat"
                        dest = os.path.join(extract_dir, f"{name}.{ext}")
                        with open(dest, "wb") as f:
                            f.write(sample_data)
                            extraction_count += 1
            
            elif obj.type.name == "Mesh":
                # Save mesh data as OBJ if possible
                if hasattr(data, "export") and callable(getattr(data, "export")):
                    try:
                        obj_data = data.export()
                        dest = os.path.join(extract_dir, f"{name}.obj")
                        with open(dest, "w") as f:
                            f.write(obj_data)
                            extraction_count += 1
                    except Exception as e:
                        print(f"Failed to export mesh {name}: {e}", c=3)
            
            # For all other types, save data attribute if available
            elif hasattr(data, "data") and data.data:
                dest = os.path.join(extract_dir, f"{name}.{obj.type.name.lower()}")
                with open(dest, "wb") as f:
                    f.write(data.data)
                    extraction_count += 1
                    
            # Try to handle serialized data for unknown types
            elif hasattr(data, "dump") and callable(getattr(data, "dump")):
                try:
                    dump_data = data.dump()
                    if dump_data:
                        dest = os.path.join(extract_dir, f"{name}.dump.json")
                        with open(dest, "w") as f:
                            json.dump(dump_data, f, indent=2)
                            extraction_count += 1
                except Exception as e:
                    pass  # Silently ignore dump failures

        except Exception as e:
            continue  # Skip this object on error
            
    return extraction_count

def extract_ab_file(ab_file: str, output_dir: str, verbose: bool = False, force_raw: bool = False, timeout: int = 120) -> Dict:
    """
    Extract a single AB file with LZHAM compression
    
    Returns a dictionary with extraction results
    """
    if not os.path.exists(ab_file):
        return {"status": "error", "error": "File not found", "path": ab_file}
    
    # Create the output directory
    ab_name = os.path.splitext(os.path.basename(ab_file))[0]
    extract_dir = os.path.join(output_dir, ab_name)
    ensure_dir(extract_dir)
    
    if verbose:
        print(f"Extracting {ab_file} to {extract_dir}...", c=6)
    
    def _do_extract():
        # Load the bundle with our patched decompression
        env = UnityPy.load(ab_file)
        
        if force_raw:
            # Use raw extraction for all objects
            extracted_count = raw_extract_objects(env, extract_dir)
            return {"status": "success" if extracted_count > 0 else "empty", "count": extracted_count}
        else:
            # Standard extraction process
            extracted_count = 0
            
            # Process each object in the bundle
            for obj in env.objects:
                try:
                    if obj.type.name in ["Texture2D", "Sprite"]:
                        # Handle image assets
                        data = obj.read()
                        if hasattr(data, "name") and data.name:
                            name = data.name
                            if hasattr(data, "image") and data.image:
                                # Save as PNG
                                dest = os.path.join(extract_dir, f"{name}.png")
                                try:
                                    img = data.image
                                    img.save(dest)
                                    extracted_count += 1
                                except Exception as e:
                                    if verbose:
                                        print(f"Error saving image {name}: {e}", c=3)
                    
                    elif obj.type.name == "TextAsset":
                        # Handle text assets
                        data = obj.read()
                        if hasattr(data, "name") and data.name:
                            name = data.name
                            if hasattr(data, "script"):
                                # Save as text file
                                dest = os.path.join(extract_dir, f"{name}.txt")
                                with open(dest, "wb") as f:
                                    f.write(data.script)
                                    extracted_count += 1
                    
                    elif obj.type.name == "AudioClip":
                        # Handle audio assets
                        data = obj.read()
                        if hasattr(data, "name") and data.name:
                            name = data.name
                            # Save audio data
                            if hasattr(data, "samples") and data.samples:
                                for sample_name, sample_data in data.samples.items():
                                    ext = sample_name.split(".")[-1] if "." in sample_name else "dat"
                                    dest = os.path.join(extract_dir, f"{name}.{ext}")
                                    with open(dest, "wb") as f:
                                        f.write(sample_data)
                                        extracted_count += 1
                
                except Exception as e:
                    if verbose:
                        print(f"Error extracting object {obj.type.name}: {e}", c=3)
                    continue
            
            # If standard extraction failed, try raw extraction as fallback
            if extracted_count == 0:
                if verbose:
                    print("Standard extraction failed, trying raw extraction...", c=3)
                extracted_count = raw_extract_objects(env, extract_dir)
                
            return {"status": "success" if extracted_count > 0 else "empty", "count": extracted_count}
    
    try:
        start_time = time.time()
        result, error, timed_out = run_with_timeout(_do_extract, timeout_duration=timeout)
        elapsed = time.time() - start_time
        
        if timed_out:
            print(f"Extraction of {ab_file} timed out after {timeout} seconds", c=3)
            # Create a marker in the extraction directory to indicate timeout
            with open(os.path.join(extract_dir, "_EXTRACTION_TIMEOUT.txt"), "w") as f:
                f.write(f"Extraction timed out after {timeout} seconds")
            return {
                "status": "timeout", 
                "path": ab_file, 
                "timeout": timeout,
                "elapsed": elapsed
            }
        
        if error:
            print(f"Error extracting {ab_file}: {error}", c=1)
            if verbose:
                traceback.print_exc()
            return {"status": "error", "error": str(error), "path": ab_file, "elapsed": elapsed}
            
        result["path"] = ab_file
        result["elapsed"] = elapsed
        
        # Report what was extracted
        if verbose:
            print(f"Extracted {result['count']} files from {ab_file} in {elapsed:.1f}s", 
                  c=2 if result['count'] > 0 else 3)
        
        return result
    
    except Exception as e:
        print(f"Error extracting {ab_file}: {e}", c=1)
        if verbose:
            traceback.print_exc()
        return {"status": "error", "error": str(e), "path": ab_file}

def process_lzham_files(lzham_list_file: str, output_dir: str, verbose: bool = False, 
                      force_raw: bool = False, analyze_only: bool = False,
                      timeout: int = 120, max_failures: int = None):
    """
    Process a list of LZHAM files for extraction
    
    Args:
        lzham_list_file: Path to a JSON file with a list of LZHAM compressed files
        output_dir: Directory to extract files to
        verbose: Show verbose output
        force_raw: Force raw extraction mode
        analyze_only: Only analyze files, don't extract
        timeout: Timeout in seconds for extraction
        max_failures: Maximum number of failures before stopping
        
    Returns:
        Summary dictionary
    """
    print("LZHAM Direct Extractor for Ark-Unpacker")
    print("========================================")
    print(f"LZHAM assets list: {lzham_list_file}")
    print(f"Output directory: {output_dir}")
    
    if analyze_only:
        print("Mode: Analysis only")
    else:
        print("Mode: Extraction")
        print(f"Extraction method: {'Raw' if force_raw else 'Auto'}")
        print(f"Timeout: {timeout} seconds per file")
        print(f"Max failures: {'No limit' if max_failures is None else max_failures}")
        print(f"Verbose output: {'Yes' if verbose else 'No'}")
    
    # Load the list of LZHAM files
    try:
        with open(lzham_list_file, 'r') as f:
            lzham_data = json.load(f)
            lzham_files = lzham_data.get("lzham_assets", [])
            print(f"Found {len(lzham_files)} LZHAM-compressed files to process")
    except Exception as e:
        print(f"Error loading LZHAM list: {e}")
        return {"status": "error", "error": str(e)}
    
    # Create output directory
    ensure_dir(output_dir)
    
    # Process each file
    results = []
    successful = 0
    failed = 0
    timed_out = 0
    empty = 0
    failure_count = 0
    total_files = len(lzham_files)
    
    # Initialize i to handle the case of empty list
    i = -1
    
    for i, file_path in enumerate(lzham_files):
        if not os.path.exists(file_path):
            print(f"File not found: {file_path}", c=3)
            failed += 1
            results.append({"file": file_path, "status": "error", "error": "File not found"})
            continue
        
        print(f"Processing file {i+1}/{total_files}: {file_path}", c=6)
        
        if analyze_only:
            # Just analyze the file
            result = analyze_ab_file(file_path, verbose=verbose, timeout=timeout)
        else:
            # Extract the file
            result = extract_ab_file(file_path, output_dir, verbose=verbose, force_raw=force_raw, timeout=timeout)
        
        results.append(result)
        
        # Handle result status
        if "error" in result:
            print(f"Error: {result['error']}", c=1)
            failed += 1
            failure_count += 1
            
            if "timeout" in result:
                timed_out += 1
                
            if max_failures and failure_count >= max_failures:
                print(f"Reached maximum failure count ({max_failures}). Stopping.", c=1)
                break
        elif result.get("status") == "empty":
            print("No assets extracted", c=3)
            empty += 1
        else:
            print(f"Successfully extracted {result.get('count', 0)} assets", c=2)
            successful += 1
        
        print(f"Total files processed: {i+1 if i >= 0 else 0} of {total_files}", c=6)
        print(f"Success: {successful}, Failed: {failed}, Empty: {empty}, Timed out: {timed_out}", c=6)
    
    # Compile summary
    print("\nProcessing complete!", c=6)
    summary = {
        "total": total_files,
        "successful": successful,
        "failed": failed,
        "empty": empty,
        "timed_out": timed_out,
        "detailed_results": results
    }
    
    # Save summary to file
    summary_file = os.path.join(output_dir, "lzham_extraction_summary.json")
    with open(summary_file, 'w') as f:
        json.dump(summary, f, indent=2)
    
    print(f"Summary saved to: {summary_file}", c=6)
    return summary

def extract_by_type(content_type: str, analysis_file: str, output_dir: str, verbose: bool = False, 
                  force_raw: bool = False, timeout: int = 120, max_failures: int = None):
    """Extract files based on their content type from analysis results"""
    try:
        with open(analysis_file, 'r') as f:
            analysis = json.load(f)
    except Exception as e:
        print(f"Error loading analysis file: {e}", c=3)
        return
    
    if "content_types" not in analysis:
        print("Invalid analysis file format", c=3)
        return
    
    if content_type not in analysis["content_types"]:
        print(f"No files found with content type: {content_type}", c=3)
        print(f"Available types: {', '.join(analysis['content_types'].keys())}", c=6)
        return
    
    files_to_extract = analysis["content_types"][content_type]
    print(f"Found {len(files_to_extract)} files with content type: {content_type}", c=6)
    
    # Prepare for extraction
    type_output_dir = os.path.join(output_dir, content_type)
    ensure_dir(type_output_dir)
    
    # Results for tracking successes and failures
    results = {
        "successful": [],
        "failed": [],
        "timeout": [],
        "empty": []
    }
    
    # Tracking variables
    successful = 0
    failed = 0
    timeouts = 0
    empty = 0
    start_time = time.time()
    failures_limit_reached = False
    
    # Process each file
    for i, file_path in enumerate(files_to_extract):
        # Check if we've hit the maximum failure limit
        if max_failures and (failed + timeouts) >= max_failures:
            print(f"\nMaximum failure limit ({max_failures}) reached. Stopping processing.", c=3)
            failures_limit_reached = True
            break
            
        # Show progress
        if verbose or i % 10 == 0:
            print(f"Extracting file {i+1}/{len(files_to_extract)}: {os.path.basename(file_path)}", c=6)
            elapsed = time.time() - start_time
            estimated_total = (elapsed / (i+1)) * len(files_to_extract) if i > 0 else 0
            estimated_remaining = estimated_total - elapsed if estimated_total > 0 else 0
            print(f"Progress: {i/len(files_to_extract)*100:.1f}% - {successful} successful, {failed} failed, {timeouts} timeouts", c=6)
            print(f"Time elapsed: {elapsed:.1f}s, Est. remaining: {estimated_remaining:.1f}s", c=6)
        
        # Extract the file
        result = extract_ab_file(file_path, type_output_dir, verbose, force_raw, timeout=timeout)
        
        if result["status"] == "success":
            successful += 1
            results["successful"].append(file_path)
        elif result["status"] == "timeout":
            timeouts += 1
            results["timeout"].append(file_path)
        elif result["status"] == "empty":
            empty += 1
            results["empty"].append(file_path)
        else:
            failed += 1
            results["failed"].append(file_path)
    
    # Show final stats
    elapsed = time.time() - start_time
    print(f"\nExtraction {'stopped early' if failures_limit_reached else 'complete'}!", s=1)
    print(f"Total files processed: {i+1 if i < len(files_to_extract) else len(files_to_extract)} of {len(files_to_extract)}", c=6)
    print(f"Successfully extracted: {successful}", c=2)
    print(f"Failed to extract: {failed}", c=3 if failed > 0 else 6)
    print(f"Timeouts: {timeouts}", c=3 if timeouts > 0 else 6)
    print(f"Empty files (no content): {empty}", c=6)
    print(f"Time elapsed: {elapsed:.1f} seconds ({elapsed/60:.1f} minutes)", c=6)
    print(f"\nExtracted files are in: {type_output_dir}", c=2)
    
    # Save results
    results_file = os.path.join(type_output_dir, "extraction_results.json")
    with open(results_file, 'w') as f:
        json.dump(results, f, indent=2)
    print(f"Results saved to: {results_file}", c=6)
    
    # Return summary
    return {
        "total": len(files_to_extract),
        "processed": i+1 if i < len(files_to_extract) else len(files_to_extract),
        "successful": successful,
        "failed": failed,
        "timeouts": timeouts,
        "empty": empty
    }

# --- Helper function to check if a directory contains actual files (recursive) ---
def dir_contains_files(directory_path, verbose=False, is_debug_file=False):
    if not directory_path or not isinstance(directory_path, str) or not os.path.isdir(directory_path):
         return False
    try:
        for root, dirs, files_in_dir in os.walk(directory_path):
            if files_in_dir:
                non_system_files = [f for f in files_in_dir if f != '.DS_Store' and not f.endswith( ('_EXTRACTION_TIMEOUT.txt', '.log', '.json') )]
                if non_system_files:
                    return True
    except OSError as e:
         if verbose: print(f"    Warning: Could not walk directory {directory_path} fully: {e}", c=3)
         return False
    return False
# --- End Helper ---

# --- Helper: Check for relevant files (returns core type found flag) ---
def dir_contains_relevant_files(directory_path, potential_base_names_set, verbose=False, is_debug_file=False):
    if not directory_path or not isinstance(directory_path, str) or not os.path.isdir(directory_path):
        if verbose and is_debug_file: print(f"        [relevant_files] Input path invalid or not a directory: {directory_path}")
        return False, None, False

    found_relevant = False
    first_found_path = None
    found_core_spine_type = False
    core_type_path = None # Store path of the file that triggered core type match
    spine_extensions = {'.atlas', '.json', '.skel'}
    if verbose and is_debug_file: print(f"        [relevant_files] Starting walk in: {directory_path} for names {potential_base_names_set}")

    try:
        for root, dirs, files_in_dir in os.walk(directory_path):
            # If verbose and is_debug_file: print(f"          [relevant_files] Checking dir: {root}, Files: {files_in_dir[:5]}...") # Can be very verbose
            for file in files_in_dir:
                if file == '.DS_Store' or file.endswith(('_EXTRACTION_TIMEOUT.txt', '.log', '.json')):
                    continue

                current_file_path = os.path.join(root, file)
                file_name_lower = file.lower()
                # --- Explicit Extension Logging ---
                try:
                    file_ext = os.path.splitext(file_name_lower)[1]
                except Exception as e:
                    if verbose and is_debug_file: print(f"          [relevant_files] Error getting extension for {file}: {e}")
                    file_ext = "?.error" # Assign dummy extension on error

                # Check if file name starts with any potential base name
                base_name_match = any(file_name_lower.startswith(base_name.lower()) for base_name in potential_base_names_set)

                if base_name_match:
                    if not found_relevant:
                        found_relevant = True
                        first_found_path = current_file_path
                        if verbose and is_debug_file: print(f"          [relevant_files] Found *first* relevant file: {current_file_path}")

                    # --- Explicit Core Type Check Logging ---
                    is_core_type = file_ext in spine_extensions
                    if verbose and is_debug_file:
                         print(f"          [relevant_files] Checking file: {file}, Ext: '{file_ext}', IsCoreType={is_core_type}")

                    if is_core_type:
                         if not found_core_spine_type: # Log only the first time
                             if verbose and is_debug_file: print(f"          [relevant_files] *** CORE TYPE MATCH FOUND ***: {current_file_path}. Setting FoundCoreType=True.")
                         found_core_spine_type = True
                         core_type_path = current_file_path
                         # Continue searching for now

    except OSError as e:
         if verbose: print(f"    Warning: Could not walk directory {directory_path} fully for relevance check: {e}", c=3)

    if verbose and is_debug_file:
        print(f"        [relevant_files] Finished walk for {directory_path}. Final Result: FoundRelevant={found_relevant}, FoundCoreType={found_core_spine_type}, FirstPath={first_found_path}, CorePath={core_type_path}")
    return found_relevant, first_found_path, found_core_spine_type
# --- End Helper ---


# --- Refined Helper function for specific content checks ---
def check_specific_content(base_path, asset_category, potential_base_names_set, verbose=False, is_debug_file=False, is_fallback_check=False):
    if not os.path.isdir(base_path): return False

    if asset_category == "chararts" or asset_category == "skinpack":
        has_image = False
        has_spine = False # Initialize

        # --- Image Check (remains the same) ---
        try:
            image_found_path = None
            image_search_paths = [base_path]
            if not is_fallback_check:
                for sub_dir_name in ['Texture2D', 'Sprite']:
                    sub_dir_path = os.path.join(base_path, sub_dir_name)
                    if os.path.isdir(sub_dir_path): image_search_paths.append(sub_dir_path)
            if verbose and is_debug_file: print(f"      Image Check: Searching paths: {image_search_paths} for names like {potential_base_names_set}")
            for search_path in image_search_paths:
                 try:
                     for entry in os.scandir(search_path):
                         if entry.is_file() and entry.name.lower().endswith('.png'):
                             entry_name_no_ext = os.path.splitext(entry.name)[0].lower()
                             if any(entry_name_no_ext.startswith(base_name.lower()) for base_name in potential_base_names_set):
                                 has_image = True
                                 image_found_path = entry.path
                                 if verbose and is_debug_file: print(f"        -> Found relevant image: {entry.path}")
                                 break
                 except OSError: pass
                 if has_image: break
            if verbose and is_debug_file: print(f"      Image Check (Relevant): Found={has_image} (Path: {image_found_path or 'N/A'})")
        except Exception as e:
            if verbose and is_debug_file: print(f"      Image Check Error: {e}")
            pass
        # --- End Image Check ---

        # --- Spine Check (Adapts based on fallback flag, now stricter fallback) ---
        spine_found_path = None
        # has_spine starts False
        found_core_in_back = False
        found_core_in_front = False
        found_core_in_building = False
        # Track overall relevance for logging
        found_any_relevant_spine_file = False
        spine_subdirs_to_check = ['BattleBack', 'BattleFront', 'Building']

        if is_fallback_check:
            # Fallback Mode: Stricter check - Back/Front required if dirs exist, Building optional
            if verbose and is_debug_file: print(f"      Spine Check (Fallback - Strict): Checking direct subdirs in {base_path} for relevant CORE files (.atlas/.json/.skel) matching {potential_base_names_set}: {spine_subdirs_to_check}")

            # Check each location
            for sub in spine_subdirs_to_check:
                sub_path = os.path.join(base_path, sub)
                # Check if the specific subdir (e.g., Unpacked/chararts/BattleBack) exists before checking content
                if not os.path.isdir(sub_path):
                    if verbose and is_debug_file: print(f"        -> Subdir '{sub}' does not exist at {sub_path}. Skipping.")
                    continue # Skip if the directory itself doesn't exist

                # Subdir exists, now check for relevant core files within it
                found_relevant, first_path, found_core_type = dir_contains_relevant_files(sub_path, potential_base_names_set, verbose, is_debug_file)

                if found_relevant and not found_any_relevant_spine_file:
                     spine_found_path = first_path # Log the first relevant file found overall
                     found_any_relevant_spine_file = True

                if found_core_type:
                    # Record which subdir had the core type
                    if sub == 'BattleBack': found_core_in_back = True
                    elif sub == 'BattleFront': found_core_in_front = True
                    elif sub == 'Building': found_core_in_building = True
                    if verbose and is_debug_file: print(f"        -> Found relevant CORE Spine content in direct subdir: {sub} (File: {first_path})")
                # else: # Don't need to log failure for each sub, final check handles it
                    # if verbose and is_debug_file: print(f"        -> No relevant CORE spine content found in {sub}.")

            # Determine final fallback spine status based on strict rules
            back_dir_exists = os.path.isdir(os.path.join(base_path, 'BattleBack'))
            front_dir_exists = os.path.isdir(os.path.join(base_path, 'BattleFront'))

            # Rule: Pass if Building has content OR (Back exists implies Back has content AND Front exists implies Front has content)
            building_ok = found_core_in_building
            back_ok = not back_dir_exists or found_core_in_back
            front_ok = not front_dir_exists or found_core_in_front

            if building_ok or (back_ok and front_ok):
                has_spine = True
                if verbose and is_debug_file: print(f"      Spine Check (Fallback - Strict): Criteria met (BuildingOK={building_ok}, BackOK={back_ok}, FrontOK={front_ok}). Setting has_spine = True.")
            else:
                # has_spine remains False
                if verbose and is_debug_file:
                    print(f"      Spine Check (Fallback - Strict): Criteria NOT met (BuildingOK={building_ok}, BackOK={back_ok}, FrontOK={front_ok}). has_spine remains False.")
                    if not back_ok: print(f"        -> Reason: BattleBack directory exists but no relevant core files found.")
                    if not front_ok: print(f"        -> Reason: BattleFront directory exists but no relevant core files found.")
                    if not building_ok and not (back_ok and front_ok): print(f"        -> Reason: Building has no core files and Back/Front requirements not met.")

        else: # Primary Mode (logic unchanged)
            spine_dir = os.path.join(base_path, 'Spine')
            if not os.path.isdir(spine_dir):
                 if verbose and is_debug_file: print(f"      Spine Check (Primary): Base Spine directory not found at {spine_dir}")
            else:
                 if verbose and is_debug_file: print(f"      Spine Check (Primary): Checking subdirs in {spine_dir} for *any* content: {spine_subdirs_to_check}")
                 for sub in spine_subdirs_to_check:
                     sub_path = os.path.join(spine_dir, sub)
                     if dir_contains_files(sub_path, verbose, is_debug_file):
                         has_spine = True
                         spine_found_path = sub_path
                         if verbose and is_debug_file: print(f"        -> Found *any* content in {spine_dir}/{sub}")
                         break
                 if not has_spine and verbose and is_debug_file: print(f"      Spine Check (Primary): No content found in required subdirectories within {spine_dir}.")

        # --- Final Logging & Return ---
        if verbose and is_debug_file:
             print(f"      Spine Check: Final state before Complete check: Found={has_spine} (Path: {spine_found_path or 'N/A'}, Mode: {'Fallback-Strict' if is_fallback_check else 'Primary'})")

        is_complete = has_image and has_spine
        if verbose and is_debug_file: print(f"      Overall Content Check (char/skin): has_image={has_image}, has_spine={has_spine} -> Complete={is_complete}")
        return is_complete

    # --- Other Categories (logic remains the same) ---
    elif asset_category == "dynchars":
        dynillust_path = os.path.join(base_path, 'DynIllust')
        has_dynillust = dir_contains_files(dynillust_path, verbose, is_debug_file)
        if verbose and is_debug_file: print(f"      Content Check (dynchars): DynIllust Path={dynillust_path}, HasContent={has_dynillust}")
        return has_dynillust

    elif asset_category == "audio":
        has_audio = dir_contains_files(base_path, verbose, is_debug_file)
        if not has_audio:
             audio_clip_path = os.path.join(base_path, "AudioClip")
             has_audio = dir_contains_files(audio_clip_path, verbose, is_debug_file)
             if verbose and is_debug_file and has_audio: print(f"      Content Check (audio): Found content in AudioClip subdir")
        if verbose and is_debug_file: print(f"      Content Check (audio): Found Content={has_audio}")
        return has_audio

    else: # General category
        has_general_content = dir_contains_files(base_path, verbose, is_debug_file)
        if verbose and is_debug_file: print(f"      Content Check (general): Found Content={has_general_content}")
        return has_general_content
# --- End Helper ---


def find_missing_assets(source_dir: str, extracted_dir: str, output_dir: str, verbose: bool = False):
    print(f"Finding missing assets...", c=6)
    source_dir = os.path.abspath(source_dir)
    extracted_dir = os.path.abspath(extracted_dir)
    output_dir = os.path.abspath(output_dir)
    print(f"Source directory: {source_dir}", c=6)
    print(f"Extracted directory: {extracted_dir}", c=6)
    print(f"Output directory: {output_dir}", c=6)

    ab_files = []
    for root, _, files in os.walk(source_dir):
        for file in files:
            if file.endswith('.ab'):
                ab_files.append(os.path.join(root, file))

    if not ab_files:
        print(f"No AB files found in source directory", c=3)
        return []

    print(f"Found {len(ab_files)} AB files in source directory", c=6)

    # Pre-scan optimization
    parent_dir_contents = {}
    if os.path.exists(extracted_dir):
        print("Scanning extracted directory contents (using os.scandir)...")
        queue = [extracted_dir]
        processed_dirs = set()
        while queue:
            current_dir = queue.pop(0)
            if current_dir in processed_dirs: continue
            processed_dirs.add(current_dir)
            try:
                immediate_subdirs = []
                for entry in os.scandir(current_dir):
                    if entry.is_dir():
                        immediate_subdirs.append(entry.name)
                        if entry.path not in processed_dirs: queue.append(entry.path)
                parent_dir_contents[current_dir] = immediate_subdirs
            except OSError as e:
                print(f"Warning: Could not scan directory {current_dir}: {e}", c=3)
                parent_dir_contents[current_dir] = []
        print(f"Finished scanning extracted directory. Found {len(parent_dir_contents)} scanned directories.")


    missing_assets = []
    already_extracted_count = 0
    partial_or_empty_count = 0
    common_prefixes = ['[uc]', '[pack]', '[ui]', 'ui_', 'bg_', 'char_', 'effect_', 'battle_', 'level_']


    # --- Main Loop ---
    print("Comparing source assets to extracted data...")
    for idx, source_file in enumerate(ab_files):
        # --- Debug specific file ---
        debug_ab_files = ['char_003_kalts.ab', 'm_bat_abyssalhunters.ab', 'char_4062_totter.ab', 'char_4087_ines_ambiencesynesthesia#5.ab', 'char_4119_wanqin.ab']
        source_basename = os.path.basename(source_file)
        is_debug_file = source_basename in debug_ab_files
        if verbose and is_debug_file:
             print(f"\n--- DEBUG: Processing {source_basename} ---")
             print(f"  Source Path: {source_file}")
        # --- End Debug ---

        # Paths, Names, Category
        relative_path = os.path.relpath(source_file, source_dir)
        relative_path_dir = os.path.dirname(relative_path)
        if relative_path_dir == '.': relative_path_dir = ""
        ab_name_no_ext = os.path.splitext(source_basename)[0]
        expected_parent_dir = os.path.join(extracted_dir, relative_path_dir)
        asset_category = "general"
        normalized_relative_path = relative_path.replace("\\", "/")
        if normalized_relative_path.startswith("chararts/"): asset_category = "chararts"
        elif normalized_relative_path.startswith("skinpack/"): asset_category = "skinpack"
        elif normalized_relative_path.startswith("arts/dynchars/"): asset_category = "dynchars"
        elif normalized_relative_path.startswith("audio/"): asset_category = "audio"
        potential_base_names = set([ab_name_no_ext])
        normalized_ab_name_lower = ab_name_no_ext.lower()
        for prefix in common_prefixes:
            if normalized_ab_name_lower.startswith(prefix.lower()):
                potential_base_names.add(ab_name_no_ext[len(prefix):])
                break
        potential_normalized_names = set(name.lower().replace('#','') for name in potential_base_names)

        if verbose and is_debug_file:
             print(f"  Asset Category: {asset_category}")
             print(f"  Expected Parent Dir: {expected_parent_dir}")
             print(f"  Potential Base Names: {potential_base_names}")
             print(f"  Potential Normalized Names for Matching: {potential_normalized_names}")


        # --- Check Logic ---
        status = "not_found"
        preliminary_status = "not_found"
        matched_actual_path = None
        found_primary_match = False

        actual_dirs_in_parent = parent_dir_contents.get(expected_parent_dir, [])
        if verbose and is_debug_file: print(f"  Actual Dirs in Parent: {actual_dirs_in_parent}")

        # 1. Check subdirectories using startswith matching (Primary Method)
        for actual_dir_name in actual_dirs_in_parent:
            normalized_actual_name = actual_dir_name.lower().replace('#','')
            is_potential_match = any(normalized_actual_name.startswith(pnn) for pnn in potential_normalized_names)

            if is_potential_match:
                found_primary_match = True
                matched_actual_path_candidate = os.path.join(expected_parent_dir, actual_dir_name)
                if verbose and is_debug_file: print(f"    Checking Primary Candidate SubDir: {matched_actual_path_candidate}")

                # Call content check with is_fallback_check=False
                has_required_content = check_specific_content(
                    matched_actual_path_candidate, asset_category, potential_base_names, verbose, is_debug_file, is_fallback_check=False
                )

                if has_required_content:
                    status = "found_complete"
                    matched_actual_path = matched_actual_path_candidate
                    if verbose and is_debug_file: print(f"      -> Found complete relevant content in primary subdir.")
                    break
                else:
                    preliminary_status = "found_empty"
                    if matched_actual_path is None: matched_actual_path = matched_actual_path_candidate
                    if verbose and is_debug_file: print(f"      -> Found primary subdir, but relevant content check failed.")
                    # Continue checking other potential primary matches

        # 2. Fallback Check: If NO complete primary match was found
        if status != "found_complete":
            if verbose and is_debug_file:
                if not found_primary_match: print(f"    No primary subdirectory match candidates found.")
                elif preliminary_status == "found_empty": print(f"    Primary subdirectory match(es) found but were incomplete (Preliminary Status: {preliminary_status}).")
                else: print(f"    No primary subdirectory match found completion (Preliminary Status: {preliminary_status}).")
                print(f"    Performing Fallback Check on Parent Dir: {expected_parent_dir}...")

            parent_exists = os.path.isdir(expected_parent_dir)
            if parent_exists:
                 # Call content check with is_fallback_check=True
                 has_required_content_in_parent = check_specific_content(
                     expected_parent_dir, asset_category, potential_base_names, verbose, is_debug_file, is_fallback_check=True
                 )

                 if has_required_content_in_parent:
                      if preliminary_status == "not_found":
                           status = "found_complete"
                           matched_actual_path = expected_parent_dir
                           if verbose and is_debug_file: print(f"      -> Found complete relevant content directly in parent dir (fallback).")
                      elif preliminary_status == "found_empty":
                           status = "found_complete" # Override previous empty status
                           matched_actual_path = expected_parent_dir
                           if verbose and is_debug_file: print(f"      -> WARNING: Found 'empty' primary match, but parent check IS complete (fallback). Marking complete.")
                 else:
                      if verbose and is_debug_file: print(f"      -> Parent dir check (fallback) also failed to find complete relevant content.")
                      status = preliminary_status # Stick with 'found_empty' or 'not_found'
            else:
                 if verbose and is_debug_file: print(f"    Parent directory does not exist.")
                 status = preliminary_status

            if verbose and is_debug_file: print(f"    Final decision after fallback: Status = {status}")


        # --- Tally Results ---
        if verbose and is_debug_file: print(f"  Final Status for {source_basename}: {status} (Matched Path: {matched_actual_path or 'None'})")

        if status == "found_complete":
            already_extracted_count += 1
        elif status == "found_empty":
            partial_or_empty_count += 1
            missing_assets.append(source_file)
        else: # status == "not_found"
            missing_assets.append(source_file)


    # --- Print Summary and Save Results ---
    print(f"\nComparison complete:", c=6)
    print(f"  - Total Source AB Files: {len(ab_files)}")
    print(f"  - Successfully Extracted (Found Matching Path with Required Relevant Content): {already_extracted_count}")
    print(f"  - Partially Extracted / Empty Dirs Found (Missing Required Content): {partial_or_empty_count}")
    print(f"  - Missing Assets (Path Not Found or Incomplete): {len(missing_assets)}")

    ensure_dir(output_dir)
    missing_file = os.path.join(output_dir, "missing_assets.json")

    # --- Save the list of ABSOLUTE paths ---
    # Note: The missing_assets list already contains absolute paths from os.walk
    with open(missing_file, 'w') as f:
        json.dump(missing_assets, f, indent=2) # <<< Save the original list with absolute paths

    print(f"List of missing assets saved to: {missing_file} (Format: JSON List of Absolute Paths)", c=2)
    return missing_assets # Return the absolute paths

def main():
    parser = argparse.ArgumentParser(description="Direct extractor for LZHAM-compressed AB files")
    parser.add_argument("--lzham-list", "-l", default="lzham_assets.json",
                      help="JSON file containing list of LZHAM-compressed assets")
    parser.add_argument("--output", "-o", default="Unpacked_LZHAM",
                      help="Output directory for extracted files")
    parser.add_argument("--verbose", "-v", action="store_true", 
                      help="Show detailed output during extraction")
    parser.add_argument("--clean", action="store_true",
                      help="Clean output directory before extraction")
    parser.add_argument("--analyze-only", "-a", action="store_true",
                      help="Only analyze files, don't extract")
    parser.add_argument("--force-raw", "-r", action="store_true",
                      help="Force raw extraction mode for all files")
    parser.add_argument("--extract-type", "-t", 
                      help="Extract only files of specific content type (requires analysis file)")
    parser.add_argument("--analysis-file", 
                      help="Analysis file to use with --extract-type (default: <output_dir>/asset_analysis.json)")
    parser.add_argument("--timeout", type=int, default=120,
                      help="Timeout in seconds for processing each file (default: 120)")
    parser.add_argument("--max-failures", type=int,
                      help="Maximum number of failures before stopping (default: no limit)")
    parser.add_argument("--find-missing", action="store_true",
                      help="Find assets that haven't been extracted yet")
    parser.add_argument("--source-dir", 
                      help="Source directory containing .ab files (for --find-missing)")
    parser.add_argument("--extracted-dir", 
                      help="Directory containing already extracted files (for --find-missing)")
    
    args = parser.parse_args()
    
    # Find missing assets
    if args.find_missing:
        if not args.source_dir or not args.extracted_dir:
            print("Error: --find-missing requires --source-dir and --extracted-dir", c=3)
            return 1
        
        missing_assets = find_missing_assets(args.source_dir, args.extracted_dir, args.output, args.verbose)
        print(f"Found {len(missing_assets)} missing assets", c=6)
        return 0
    
    # Validate arguments
    if args.extract_type and not (args.analysis_file or os.path.exists(os.path.join(args.output, "asset_analysis.json"))):
        print("Error: --extract-type requires an existing analysis file", c=3)
        print("Run with --analyze-only first to generate the analysis file", c=6)
        return 1
    
    # Clean output if requested
    if args.clean and os.path.exists(args.output) and not args.analyze_only:
        print(f"Cleaning output directory: {args.output}")
        rmdir(args.output)
    
    # Create output directory
    ensure_dir(args.output)
    
    print("LZHAM Direct Extractor for Ark-Unpacker", s=1)
    print("========================================")
    print(f"LZHAM assets list: {args.lzham_list}")
    print(f"Output directory: {args.output}")
    print(f"Mode: {'Analysis only' if args.analyze_only else 'Extraction'}")
    print(f"Extraction method: {'Raw (forced)' if args.force_raw else 'Auto'}")
    print(f"Timeout: {args.timeout} seconds per file")
    print(f"Max failures: {args.max_failures if args.max_failures else 'No limit'}")
    print(f"Verbose output: {'Yes' if args.verbose else 'No'}")
    
    # Extract by type if specified
    if args.extract_type:
        analysis_file = args.analysis_file or os.path.join(args.output, "asset_analysis.json")
        summary = extract_by_type(
            args.extract_type, 
            analysis_file, 
            args.output, 
            args.verbose, 
            args.force_raw,
            args.timeout,
            args.max_failures
        )
    else:
        # Process LZHAM files
        summary = process_lzham_files(
            args.lzham_list, 
            args.output, 
            args.verbose, 
            args.force_raw, 
            args.analyze_only,
            args.timeout,
            args.max_failures
        )
    
    if not args.analyze_only:
        print("\nExtraction complete! You can find extracted files in:", c=2)
        print(os.path.abspath(args.output), c=6)
        
        if summary and summary.get("timeouts", 0) > 0:
            print(f"\nNote: {summary['timeouts']} files timed out during extraction.", c=3)
            print(f"Check extraction_results.json for the list of these files.", c=6)
            print(f"You can try increasing the timeout with --timeout <seconds>", c=6)

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nOperation cancelled by user")
        sys.exit(1)
    except Exception as e:
        print(f"An unexpected error occurred: {e}", c=1)
        traceback.print_exc()
        sys.exit(1) 