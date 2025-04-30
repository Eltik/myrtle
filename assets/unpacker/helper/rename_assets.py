import os
import argparse

def update_atlas_content(atlas_path):
    """Reads an atlas file, replaces '#' with '_' in the PNG filename reference,
       and writes the modified content back.
    """
    try:
        with open(atlas_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()

        modified = False
        for i, line in enumerate(lines):
            stripped_line = line.strip()
            # Find the first non-empty line ending with .png (likely the texture reference)
            if stripped_line and stripped_line.endswith('.png'):
                if '#' in stripped_line:
                    original_png_ref = stripped_line
                    new_png_ref = original_png_ref.replace('#', '_')
                    lines[i] = line.replace(original_png_ref, new_png_ref) # Replace in the original line
                    print(f"  - Updated atlas content: '{original_png_ref}' -> '{new_png_ref}'")
                    modified = True
                break # Only process the first PNG reference found

        if modified:
            with open(atlas_path, 'w', encoding='utf-8') as f:
                f.writelines(lines)
        return modified

    except Exception as e:
        print(f"  - Error updating atlas content for '{atlas_path}': {e}")
        return False

def rename_assets_with_hash(root_dir, force_update_all_atlas=False):
    """
    Recursively renames files and directories within root_dir that contain
    a '#' character, replacing it with '_'. Also updates .atlas file content.
    Renames deeper items first.

    Args:
        root_dir (str): The root directory to scan.
        force_update_all_atlas (bool): If True, attempts to update content of all
                                      .atlas files found, not just those that were
                                      renamed or in renamed directories.
    """
    renamed_count = 0
    updated_atlas_count = 0
    if not os.path.isdir(root_dir):
        print(f"Error: Directory '{root_dir}' not found or is not a valid directory.")
        return

    print(f"Scanning directory: {root_dir}")

    # Use topdown=False to process deeper files/directories first
    for dirpath, dirnames, filenames in os.walk(root_dir, topdown=False):
        # Rename files first
        for filename in filenames:
            file_lower = filename.lower()
            current_path = os.path.join(dirpath, filename)
            needs_atlas_update_check = False

            if '#' in filename:
                old_path = os.path.join(dirpath, filename)
                new_filename = filename.replace('#', '_')
                new_path = os.path.join(dirpath, new_filename)
                try:
                    os.rename(old_path, new_path)
                    print(f"Renamed file: '{old_path}' -> '{new_path}'")
                    renamed_count += 1
                    current_path = new_path # Update path for atlas check
                    if file_lower.endswith('.atlas'):
                        needs_atlas_update_check = True
                except OSError as e:
                    print(f"Error renaming file '{old_path}' to '{new_path}': {e}")
            
            # Check atlas files
            if file_lower.endswith('.atlas'):
                if force_update_all_atlas:
                    needs_atlas_update_check = True
                    if '#' not in filename: # Log if forced check
                         print(f"Checking atlas due to --force-update: '{current_path}'")
                elif any('#' in part for part in dirpath.split(os.sep)):
                     # Check if containing dir had # (only if not forcing all)
                     needs_atlas_update_check = True
                     if '#' not in filename:
                          print(f"Checking atlas in potentially renamed dir: '{current_path}'")
                
                if needs_atlas_update_check:
                    if update_atlas_content(current_path):
                        updated_atlas_count += 1

        # Then rename directories
        for dirname in dirnames:
            if '#' in dirname:
                old_path = os.path.join(dirpath, dirname)
                new_dirname = dirname.replace('#', '_')
                new_path = os.path.join(dirpath, new_dirname)
                try:
                    # Important: Check if the new path already exists (could happen if run twice)
                    if not os.path.exists(new_path):
                         os.rename(old_path, new_path)
                         print(f"Renamed directory: '{old_path}' -> '{new_path}'")
                         renamed_count += 1
                    else:
                         print(f"Skipping rename, directory already exists: '{new_path}'")

                except OSError as e:
                    print(f"Error renaming directory '{old_path}' to '{new_path}': {e}")

    if renamed_count == 0 and updated_atlas_count == 0 and not force_update_all_atlas:
        print("No files or directories containing '#' were found to rename or update.")
    else:
        print(f"Finished scan.")
        if renamed_count > 0:
            print(f"  - Total items renamed: {renamed_count}")
        if updated_atlas_count > 0:
            print(f"  - Updated content in {updated_atlas_count} .atlas files.")
        if force_update_all_atlas:
             print(f"  - Forced update scan enabled for all .atlas files.")


if __name__ == "__main__":
    # Set up argument parser
    parser = argparse.ArgumentParser(description="Recursively rename files/dirs containing '#' to '_' and update .atlas references.")
    parser.add_argument("target_directory", 
                        help="The root directory to scan.")
    parser.add_argument("--force-update-all-atlas", action="store_true", 
                        help="Force checking and potentially updating the content of ALL .atlas files, not just renamed ones.")
    
    # Parse arguments
    args = parser.parse_args()
    
    # Use the provided directory and flag
    target_dir = args.target_directory
    force_update = args.force_update_all_atlas
    
    # Call the renaming function
    rename_assets_with_hash(target_dir, force_update_all_atlas=force_update) 