#!/usr/bin/env python3
"""
Sync script to copy missing or different files from ArkAssets to upk.

ArkAssets is the updated version, upk is the target directory.
Files are copied if:
  1. They don't exist in the target
  2. They exist but have different sizes
  3. They exist with same size but different content (MD5 hash)
"""

import os
import shutil
import hashlib
import argparse
from pathlib import Path

SOURCE = "/Users/eltik/Documents/Coding/myrtle.moe/assets/Unpacked/upk/ArkAssets"
TARGET = "/Users/eltik/Documents/Coding/myrtle.moe/assets/Unpacked/upk"


def get_file_hash(filepath: str, block_size: int = 65536) -> str:
    """Compute MD5 hash of a file."""
    hasher = hashlib.md5()
    with open(filepath, 'rb') as f:
        for block in iter(lambda: f.read(block_size), b''):
            hasher.update(block)
    return hasher.hexdigest()


def files_are_identical(src: str, dst: str) -> bool:
    """
    Compare two files: size first (fast), then hash if sizes match.
    Returns True if files are identical.
    """
    src_size = os.path.getsize(src)
    dst_size = os.path.getsize(dst)

    if src_size != dst_size:
        return False

    return get_file_hash(src) == get_file_hash(dst)


def sync_files(dry_run: bool = False, verbose: bool = False) -> dict:
    """
    Main sync logic. Walks through ArkAssets and syncs to upk.

    Returns a dict with statistics.
    """
    stats = {
        'copied_missing': 0,
        'copied_different': 0,
        'skipped_identical': 0,
        'errors': 0,
        'dirs_created': 0,
    }

    source_path = Path(SOURCE)
    target_base = Path(TARGET)

    if not source_path.exists():
        print(f"Error: Source directory does not exist: {SOURCE}")
        return stats

    print(f"Source: {SOURCE}")
    print(f"Target: {TARGET}")
    print(f"Mode: {'DRY RUN' if dry_run else 'LIVE'}")
    print("-" * 60)

    file_count = 0
    for root, dirs, files in os.walk(SOURCE):
        # Skip .DS_Store files and hidden directories
        dirs[:] = [d for d in dirs if not d.startswith('.')]

        for filename in files:
            # Skip hidden files and .DS_Store
            if filename.startswith('.'):
                continue

            file_count += 1
            if file_count % 1000 == 0:
                print(f"Processed {file_count} files...")

            src_file = os.path.join(root, filename)

            # Compute relative path from ArkAssets
            rel_path = os.path.relpath(src_file, SOURCE)

            # Compute target path in upk
            dst_file = os.path.join(TARGET, rel_path)
            dst_dir = os.path.dirname(dst_file)

            try:
                # Check if target exists
                if not os.path.exists(dst_file):
                    # File is missing in target
                    if verbose or dry_run:
                        print(f"[MISSING] {rel_path}")

                    if not dry_run:
                        os.makedirs(dst_dir, exist_ok=True)
                        shutil.copy2(src_file, dst_file)

                    stats['copied_missing'] += 1
                else:
                    # File exists, compare
                    if files_are_identical(src_file, dst_file):
                        if verbose:
                            print(f"[IDENTICAL] {rel_path}")
                        stats['skipped_identical'] += 1
                    else:
                        if verbose or dry_run:
                            print(f"[DIFFERENT] {rel_path}")

                        if not dry_run:
                            shutil.copy2(src_file, dst_file)

                        stats['copied_different'] += 1

            except Exception as e:
                print(f"[ERROR] {rel_path}: {e}")
                stats['errors'] += 1

    return stats


def print_summary(stats: dict, dry_run: bool):
    """Print summary statistics."""
    print("-" * 60)
    print("SUMMARY")
    print("-" * 60)

    action = "Would copy" if dry_run else "Copied"

    total_copied = stats['copied_missing'] + stats['copied_different']

    print(f"{action} (missing):    {stats['copied_missing']:,}")
    print(f"{action} (different):  {stats['copied_different']:,}")
    print(f"Skipped (identical): {stats['skipped_identical']:,}")
    print(f"Errors:              {stats['errors']:,}")
    print("-" * 60)
    print(f"Total {action.lower()}: {total_copied:,}")
    print(f"Total processed:     {total_copied + stats['skipped_identical']:,}")


def main():
    parser = argparse.ArgumentParser(
        description="Sync files from ArkAssets to upk directory"
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help="Show what would be copied without actually copying"
    )
    parser.add_argument(
        '--verbose', '-v',
        action='store_true',
        help="Show all files being processed (including skipped)"
    )

    args = parser.parse_args()

    stats = sync_files(dry_run=args.dry_run, verbose=args.verbose)
    print_summary(stats, args.dry_run)


if __name__ == "__main__":
    main()
