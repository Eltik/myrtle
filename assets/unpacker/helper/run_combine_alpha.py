#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# Simple wrapper for the combine_alpha.py script

import os
import sys
import argparse

# Check if combine_alpha.py exists
if not os.path.exists('combine_alpha.py'):
    print("Error: combine_alpha.py not found in the current directory.")
    print("Please make sure you have created the combine_alpha.py script.")
    sys.exit(1)

try:
    import combine_alpha
except ImportError as e:
    print(f"Error importing combine_alpha module: {e}")
    print("Make sure you have all required dependencies installed:")
    print("  pip install Pillow")
    sys.exit(1)

def main():
    parser = argparse.ArgumentParser(description='Combine alpha channel images in Arknights assets')
    parser.add_argument('--input-dir', '-i', type=str, 
                        default='/Users/eltik/Documents/Coding/Ark-Assets/Ark-Unpacker/Unpacked',
                        help='Directory containing the unpacked files (default: /Users/eltik/Documents/Coding/Ark-Assets/Ark-Unpacker/Unpacked)')
    parser.add_argument('--output-dir', '-o', type=str, default=None,
                        help='Output directory for combined images (default: overwrite input)')
    parser.add_argument('--overwrite', '-f', action='store_true', 
                        help='Overwrite existing files')
    parser.add_argument('--workers', '-w', type=int, default=4,
                        help='Number of worker threads (default: 4)')
    
    args = parser.parse_args()
    
    print(f"Alpha Image Combiner")
    print(f"--------------------")
    print(f"Input directory:  {args.input_dir}")
    print(f"Output directory: {args.output_dir or 'Same as input (overwrite)'}")
    print(f"Overwrite files:  {'Yes' if args.overwrite else 'No'}")
    print(f"Worker threads:   {args.workers}")
    print()
    
    # Pass args to the combine_alpha script
    combine_alpha.args = args
    result = combine_alpha.main()
    
    if result == 0:
        print("\nAlpha channel images combined successfully!")
    else:
        print("\nFailed to combine alpha channel images.")
    
    return result

if __name__ == "__main__":
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        print("\nOperation cancelled by user")
        sys.exit(1)
    except Exception as e:
        print(f"An error occurred: {e}")
        sys.exit(1) 