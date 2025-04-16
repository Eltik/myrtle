import os
from typing import Callable

def main(
    ab: uc.AssetBundle,
    destdir: str,
    on_queued: "Callable|None" = None,
    on_saved: "Callable|None" = None,
):
    """Resolves the given AssetBundle and saves all the objects as files.

    :param ab: The AssetBundle to resolve;
    :param destdir: Destination directory;
    :param on_queued: Callback `f()` invoked when the file was queued, `None` for ignore;
    :param on_saved: Callback `f(file_path_or_none_for_not_saved)`, `None` for ignore;
    :rtype: None;
    """
    # Normalize the destination directory path
    destdir = os.path.normpath(destdir)
    
    # Get the base name of the AssetBundle
    base_name = os.path.basename(ab.file_name)
    if base_name.endswith(".ab"):
        base_name = base_name[:-3]
    
    # Create a subdirectory for this AssetBundle
    bundle_dir = os.path.join(destdir, base_name)
    
    # Save all objects in the AssetBundle
    for obj in ab.objects:
        # Get the object's name and ensure it's a valid filename
        obj_name = getattr(obj, "m_Name", "Unknown")
        if not obj_name:
            obj_name = "Unknown"
            
        # Create a safe filename
        safe_name = "".join(c for c in obj_name if c.isalnum() or c in "._- ")
        
        # Save the object
        SafeSaver.save_object(obj, bundle_dir, safe_name, on_queued, on_saved) 