#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# Alpha channel combiner for Ark-Unpacker
# This script finds [alpha] files and combines them with their RGB counterparts

import os
import re
import argparse
from PIL import Image
import logging
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler("combine_alpha.log"),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)

# Regex patterns for alpha images, similar to the Ark-Unpacker code
ALPHA_PATTERNS = [
    re.compile(r"(.+)\[alpha\](\$[0-9]+)?"),  # Match char_108_silent_sweep#1[alpha]
    re.compile(r"(.+)_alpha(\$[0-9]+)?"),
    re.compile(r"(.+)alpha(\$[0-9]+)?"),
    re.compile(r"(.+)a(\$[0-9]+)?"),
]

def is_image_file(path):
    """Check if a file is an image based on its extension."""
    ext = os.path.splitext(path)[1].lower()
    return ext in ['.png', '.jpg', '.jpeg', '.bmp', '.tga']

def get_base_name(alpha_path):
    """Get the base name of an alpha file, removing the alpha suffix."""
    basename = os.path.basename(alpha_path)
    name, ext = os.path.splitext(basename)
    
    # Check standard alpha patterns
    for pattern in ALPHA_PATTERNS:
        match = pattern.fullmatch(name)
        if match:
            base_name = match.group(1)
            logger.debug(f"Matched alpha pattern for {name}, base name: {base_name}")
            return base_name
    
    # Special case for patterns like portraits#0a.png
    if name.endswith('a') and '#' in name:
        # Check if this is portrait#0a.png type file (where portrait#0.png is the RGB file)
        base_without_a = name[:-1]  # Remove the trailing 'a'
        dir_path = os.path.dirname(alpha_path)
        
        # Check if the base file exists
        rgb_path = os.path.join(dir_path, base_without_a + ext)
        if os.path.exists(rgb_path):
            logger.debug(f"Found match for {name}: {base_without_a}")
            return base_without_a
        
        # Also check with $0 suffix which is common in Arknights assets
        rgb_path_dollar = os.path.join(dir_path, base_without_a + "$0" + ext)
        if os.path.exists(rgb_path_dollar):
            logger.debug(f"Found $0 match for {name}: {base_without_a}")
            return base_without_a
    
    return None

def find_rgb_file(alpha_path):
    """Find the corresponding RGB file for an alpha file."""
    base_name = get_base_name(alpha_path)
    if not base_name:
        return None
    
    dir_path = os.path.dirname(alpha_path)
    ext = os.path.splitext(alpha_path)[1]
    alpha_filename = os.path.basename(alpha_path)
    
    logger.debug(f"Finding RGB file for alpha: {alpha_filename}, base: {base_name}")
    
    # Check for [alpha] files specifically to prioritize $0 version
    if "[alpha]" in alpha_filename:
        rgb_path_dollar = os.path.join(dir_path, base_name + "$0" + ext)
        if os.path.exists(rgb_path_dollar):
            logger.info(f"Found $0 match for [alpha] file: {rgb_path_dollar}")
            return rgb_path_dollar
    
    # First, prioritize check for the $0 version which is common in Arknights assets
    rgb_path_dollar = os.path.join(dir_path, base_name + "$0" + ext)
    if os.path.exists(rgb_path_dollar):
        logger.debug(f"Found $0 match: {rgb_path_dollar}")
        return rgb_path_dollar
    
    # Then check for exact match with base name
    rgb_path = os.path.join(dir_path, base_name + ext)
    if os.path.exists(rgb_path):
        logger.debug(f"Found exact match: {rgb_path}")
        return rgb_path
    
    # If no exact match, look for files starting with base_name and possibly having $ suffix
    candidates = []
    for filename in os.listdir(dir_path):
        if is_image_file(filename) and filename.startswith(base_name):
            # Avoid other alpha files
            if not any(p.fullmatch(os.path.splitext(filename)[0]) for p in ALPHA_PATTERNS):
                candidates.append(os.path.join(dir_path, filename))
    
    if not candidates:
        logger.warning(f"No RGB candidates found for {alpha_path}")
        return None
    elif len(candidates) == 1:
        logger.debug(f"Found single candidate: {candidates[0]}")
        return candidates[0]
    else:
        # Multiple candidates - prioritize ones with $ suffix
        for candidate in candidates:
            if '$' in os.path.basename(candidate):
                logger.debug(f"Selected $ candidate from multiple: {candidate}")
                return candidate
        # If no $ suffix found, take the first one
        logger.debug(f"Selected first candidate from multiple: {candidates[0]}")
        return candidates[0]

def combine_images(rgb_path, alpha_path, output_path=None, overwrite=False, delete_alpha=False):
    """Combine RGB and alpha images into a single RGBA image."""
    if not output_path:
        output_path = rgb_path  # Overwrite original
    
    if os.path.exists(output_path) and not overwrite:
        logger.info(f"Skipping {output_path} (already exists)")
        return False

    try:
        # Open images
        rgb_img = Image.open(rgb_path).convert("RGBA")
        alpha_img = Image.open(alpha_path).convert("L")  # Grayscale for alpha channel
        
        # Resize alpha to match RGB if needed
        if alpha_img.size != rgb_img.size:
            alpha_img = alpha_img.resize(rgb_img.size, Image.LANCZOS)
        
        # Apply alpha channel
        rgb_img.putalpha(alpha_img)
        
        # Remove color bleeding (set RGB to black where alpha is 0)
        data = rgb_img.getdata()
        new_data = []
        for item in data:
            # If alpha is 0, set RGB to black
            if item[3] == 0:
                new_data.append((0, 0, 0, 0))
            else:
                new_data.append(item)
        rgb_img.putdata(new_data)
        
        # Create directory if needed
        os.makedirs(os.path.dirname(output_path), exist_ok=True)
        
        # Save combined image
        rgb_img.save(output_path)
        logger.info(f"Combined {os.path.basename(rgb_path)} with {os.path.basename(alpha_path)} -> {os.path.basename(output_path)}")
        
        # Delete alpha file if requested
        if delete_alpha and os.path.exists(alpha_path):
            try:
                os.remove(alpha_path)
                logger.info(f"Deleted alpha file: {alpha_path}")
            except Exception as e:
                logger.error(f"Failed to delete alpha file {alpha_path}: {e}")
        
        return True
    
    except Exception as e:
        logger.error(f"Error combining {rgb_path} with {alpha_path}: {e}")
        return False

def process_alpha_file(alpha_path, output_dir=None, overwrite=False, delete_alpha=False):
    """Process a single alpha file."""
    logger.info(f"Processing alpha file: {alpha_path}")
    rgb_path = find_rgb_file(alpha_path)
    if not rgb_path:
        logger.warning(f"Could not find RGB file for {alpha_path}")
        return False
    
    logger.info(f"Found matching RGB file: {rgb_path}")
    
    if output_dir:
        # Create relative path in output directory
        rel_path = os.path.relpath(rgb_path, start=args.input_dir)
        output_path = os.path.join(output_dir, rel_path)
    else:
        output_path = rgb_path
    
    return combine_images(rgb_path, alpha_path, output_path, overwrite, delete_alpha)

def find_alpha_files(directory):
    """Find all alpha files in a directory recursively."""
    alpha_files = []
    potential_rgb_files = []
    
    # First pass: Find files matching alpha patterns and potential RGB files
    for root, _, files in os.walk(directory):
        for file in files:
            if is_image_file(file):
                file_path = os.path.join(root, file)
                basename = os.path.basename(file_path)
                name, _ = os.path.splitext(basename)
                
                # Check if it's a standard alpha file
                base_name = get_base_name(file_path)
                if base_name:  # It's an alpha file
                    alpha_files.append(file_path)
                # Check if it's a potential RGB file with '#' that might have an alpha counterpart
                elif '#' in name:
                    potential_rgb_files.append(file_path)
    
    # Second pass: Check potential RGB files for alpha counterparts
    for rgb_path in potential_rgb_files:
        basename = os.path.basename(rgb_path)
        name, ext = os.path.splitext(basename)
        
        # Check for alpha file with 'a' suffix
        dir_path = os.path.dirname(rgb_path)
        alpha_name = name + 'a' + ext
        alpha_path = os.path.join(dir_path, alpha_name)
        
        if os.path.exists(alpha_path) and alpha_path not in alpha_files:
            alpha_files.append(alpha_path)
    
    return alpha_files

# Add specific pattern to detect if a file is an RGB file with alpha counterpart
def has_alpha_counterpart(filepath):
    """Check if there might be an alpha counterpart for this file."""
    basename = os.path.basename(filepath)
    name, ext = os.path.splitext(basename)
    
    # Check for patterns like portraits#0.png that could have portraits#0a.png as alpha
    if '#' in name:
        dir_path = os.path.dirname(filepath)
        alpha_name = name + 'a' + ext
        alpha_path = os.path.join(dir_path, alpha_name)
        return os.path.exists(alpha_path)
    
    return False

def main():
    parser = argparse.ArgumentParser(description='Combine alpha channel images with their RGB counterparts')
    parser.add_argument('--input-dir', '-i', type=str, default='Ark-Unpacker/Unpacked',
                        help='Directory containing the unpacked files (default: Ark-Unpacker/Unpacked)')
    parser.add_argument('--output-dir', '-o', type=str, default=None,
                        help='Output directory for combined images (default: overwrite input)')
    parser.add_argument('--overwrite', '-f', action='store_true', default=True,
                        help='Overwrite existing files (default: True)')
    parser.add_argument('--workers', '-w', type=int, default=4,
                        help='Number of worker threads (default: 4)')
    parser.add_argument('--debug', action='store_true',
                        help='Run in debug mode with specific example file')
    parser.add_argument('--verbose', '-v', action='store_true',
                        help='Enable verbose logging')
    parser.add_argument('--delete-alpha', '-d', action='store_true',
                        help='Delete alpha files after successful combination')
    
    global args
    args = parser.parse_args()
    
    # Set logging level based on verbosity
    if args.verbose:
        logger.setLevel(logging.DEBUG)
    
    # Debug mode - process specific example file
    if args.debug:
        example_alpha = '/Users/eltik/Documents/Coding/Ark-Assets/Ark-Unpacker/Unpacked/skinpack/char_108_silent_sweep#1[alpha].png'
        if os.path.exists(example_alpha):
            logger.info(f"Debug mode: Processing example file: {example_alpha}")
            success = process_alpha_file(example_alpha, args.output_dir, args.overwrite, args.delete_alpha)
            if success:
                logger.info("Successfully processed example file!")
            else:
                logger.error("Failed to process example file!")
            return 0 if success else 1
        else:
            logger.error(f"Example file not found: {example_alpha}")
            return 1
    
    # Validate input directory
    if not os.path.isdir(args.input_dir):
        logger.error(f"Input directory does not exist: {args.input_dir}")
        return 1
    
    # Find all alpha files
    logger.info(f"Searching for alpha files in {args.input_dir}...")
    alpha_files = find_alpha_files(args.input_dir)
    logger.info(f"Found {len(alpha_files)} alpha files")
    
    if not alpha_files:
        logger.info("No alpha files found. Nothing to do.")
        return 0
    
    # Process files
    success_count = 0
    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        futures = []
        for alpha_path in alpha_files:
            futures.append(
                executor.submit(
                    process_alpha_file, alpha_path, args.output_dir, args.overwrite, args.delete_alpha
                )
            )
        
        for future in futures:
            if future.result():
                success_count += 1
    
    logger.info(f"Combined {success_count} out of {len(alpha_files)} alpha files")
    if args.delete_alpha:
        logger.info("Alpha files were deleted after successful combination")
    return 0

if __name__ == "__main__":
    exit(main()) 