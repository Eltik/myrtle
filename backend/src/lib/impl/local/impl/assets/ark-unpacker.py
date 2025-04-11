#!/usr/bin/env python3
"""
Simplified Arknights Asset Unpacker

This script unpacks Arknights AssetBundle (.ab) files using the UNITYPY_AK environment
variable to enable special handling for Arknights compressed files.

Usage:
    List available directories:
        python simplified_unpacker.py --list-dirs
    
    Extract assets from specific directories:
        python simplified_unpacker.py -d arts ui audio chararts
    
    Extract all assets:
        python simplified_unpacker.py --all
    
    Extract with custom input/output paths:
        python simplified_unpacker.py -i /path/to/ArkAssets -o /path/to/output -d arts
    
    Force reprocessing of already extracted files:
        python simplified_unpacker.py -d arts -f
    
    Enable debug mode:
        python simplified_unpacker.py -d arts --debug

Common directory types:
    - arts: Background images, loading screens
    - ui: User interface elements, buttons, HUD
    - audio: Sound effects, music, voice lines
    - chararts: Character illustrations
    - charpack: Character models and animations
    - skinpack: Character skins
    - battle: Battle-related assets
    - spritepack: Various sprite collections
    - building: Base building elements
    - avg: Story and cutscene assets
"""

import os
import sys
import time
import json
import UnityPy
from pathlib import Path
from tqdm import tqdm

# Critical: Set environment variable for Arknights-specific handling
os.environ['UNITYPY_AK'] = '1'
print(f"UNITYPY_AK environment variable set to: {os.environ.get('UNITYPY_AK')}")

# Enable debug mode
debug_mode = True
print("Debug mode is enabled. Will show detailed output.")

class Colors:
    RESET = '\033[0m'
    RED = '\033[91m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    MAGENTA = '\033[95m'
    CYAN = '\033[96m'
    BOLD = '\033[1m'

def print_color(text, color=Colors.RESET, end='\n'):
    """Print colored text to console"""
    print(f"{color}{text}{Colors.RESET}", end=end)

def debug_print(text, color=Colors.YELLOW):
    """Print debug information if debug mode is enabled"""
    if debug_mode:
        print_color(text, color)

# Define a custom JSON encoder to handle Unity object references and other special types
class UnityObjectEncoder(json.JSONEncoder):
    def default(self, obj):
        # Handle Unity PPtr references
        if hasattr(obj, '__dict__'):
            return str(obj)
        # Let the base class handle other types
        return json.JSONEncoder.default(self, obj)

def extract_unity_path(obj):
    """Extract a meaningful path from a Unity object"""
    path_components = []
    
    # Try to get container path
    if hasattr(obj, 'container') and hasattr(obj.container, 'path'):
        container_path = obj.container.path
        if container_path:
            return container_path
    
    # Try to get from parent objects if possible
    if hasattr(obj, 'parent') and obj.parent:
        try:
            parent_path = extract_unity_path(obj.parent)
            if parent_path:
                path_components.append(parent_path)
        except:
            pass
    
    # Add object name if available
    data = obj.read() if hasattr(obj, 'read') else None
    if data:
        if hasattr(data, 'name') and data.name and data.name != "unnamed":
            path_components.append(data.name)
        elif hasattr(data, 'm_Name') and data.m_Name and data.m_Name != "unnamed":
            path_components.append(data.m_Name)
    
    # Return constructed path
    if path_components:
        return '/'.join(path_components)
    return None

def process_arknights_filename(path, obj_name, obj_type):
    """Process filenames for Arknights-specific assets"""
    path_lower = path.lower()
    
    # Character assets (charpack, chararts)
    if any(x in path_lower for x in ['char_', 'charpack', 'chararts']):
        # Look for character ID patterns (like char_123_somename)
        parts = path_lower.split('/')
        char_id = None
        for part in parts:
            if part.startswith('char_'):
                char_id = part
                break
        
        if char_id:
            # Use character ID as filename prefix
            if obj_name.startswith(char_id):
                return obj_name  # Already has the correct prefix
            return f"{char_id}_{obj_name}"
    
    # Skin assets
    elif any(x in path_lower for x in ['skin_', 'skinpack']):
        # Look for skin ID patterns
        parts = path_lower.split('/')
        skin_id = None
        for part in parts:
            if part.startswith('char_') or part.startswith('skin_'):
                skin_id = part
                break
        
        if skin_id:
            if obj_name.startswith(skin_id):
                return obj_name
            return f"{skin_id}_{obj_name}"
    
    # UI elements
    elif 'ui' in path_lower or 'activity' in path_lower:
        if obj_type in ['Texture2D', 'Sprite']:
            # Extract module name from path if possible
            parts = path_lower.split('/')
            ui_module = None
            for part in parts:
                if part and part != 'ui' and not part.endswith('.ab'):
                    ui_module = part
                    break
            
            if ui_module:
                if obj_name.startswith(f"ui_{ui_module}_"):
                    return obj_name
                elif obj_name.startswith('ui_'):
                    return f"ui_{ui_module}_{obj_name[3:]}"
                return f"ui_{ui_module}_{obj_name}"
            else:
                # Generic UI asset without specific module
                if obj_name.startswith('ui_'):
                    return obj_name
                return f"ui_{obj_name}"
    
    # Battle assets
    elif 'battle' in path_lower:
        # Extract battle type if possible
        parts = path_lower.split('/')
        battle_type = None
        for part in parts:
            if part and part != 'battle' and not part.endswith('.ab'):
                battle_type = part
                break
        
        if battle_type:
            if obj_name.startswith(f"battle_{battle_type}_"):
                return obj_name
            elif obj_name.startswith('battle_'):
                return f"battle_{battle_type}_{obj_name[7:]}"
            return f"battle_{battle_type}_{obj_name}"
        else:
            if obj_name.startswith('battle_'):
                return obj_name
            return f"battle_{obj_name}"
    
    # Map assets
    elif any(x in path_lower for x in ['map_', 'scene']):
        parts = path_lower.split('/')
        map_id = None
        for part in parts:
            if part.startswith('map_'):
                map_id = part
                break
        
        if map_id:
            if obj_name.startswith(map_id):
                return obj_name
            return f"{map_id}_{obj_name}"
    
    # Enemy assets
    elif 'enemy' in path_lower:
        parts = path_lower.split('/')
        enemy_id = None
        for part in parts:
            if part.startswith('enemy_'):
                enemy_id = part
                break
        
        if enemy_id:
            if obj_name.startswith(enemy_id):
                return obj_name
            return f"{enemy_id}_{obj_name}"
    
    # Audio assets
    elif 'audio' in path_lower:
        parts = path_lower.split('/')
        audio_category = None
        for part in parts:
            if part and part != 'audio' and not part.endswith('.ab'):
                audio_category = part
                break
        
        if audio_category:
            if obj_name.startswith(f"audio_{audio_category}_"):
                return obj_name
            return f"audio_{audio_category}_{obj_name}"
        else:
            if obj_name.startswith('audio_'):
                return obj_name
            return f"audio_{obj_name}"
    
    # Items and effects
    elif 'item' in path_lower or 'effect' in path_lower:
        category = 'item' if 'item' in path_lower else 'effect'
        if obj_name.startswith(f"{category}_"):
            return obj_name
        return f"{category}_{obj_name}"
    
    # Building assets
    elif 'building' in path_lower:
        parts = path_lower.split('/')
        building_category = None
        for part in parts:
            if part and part != 'building' and not part.endswith('.ab'):
                building_category = part
                break
        
        if building_category:
            if obj_name.startswith(f"building_{building_category}_"):
                return obj_name
            return f"building_{building_category}_{obj_name}"
        else:
            if obj_name.startswith('building_'):
                return obj_name
            return f"building_{obj_name}"
    
    # Return the original name if no specific handling is needed
    return obj_name

def get_meaningful_name(obj, obj_type, path_id):
    """Get a meaningful name for a Unity object"""
    data = obj.read() if hasattr(obj, 'read') else None
    
    # Try various methods to get a meaningful name
    if data:
        # Method 1: Direct name attribute
        if hasattr(data, 'name') and data.name and data.name != "unnamed":
            obj_name = data.name
            # Apply Arknights-specific processing if we have path info
            if hasattr(obj, 'container') and hasattr(obj.container, 'path'):
                obj_name = process_arknights_filename(obj.container.path, obj_name, obj_type)
            return obj_name
        
        # Method 2: Unity's m_Name attribute
        if hasattr(data, 'm_Name') and data.m_Name and data.m_Name != "unnamed":
            obj_name = data.m_Name
            # Apply Arknights-specific processing if we have path info
            if hasattr(obj, 'container') and hasattr(obj.container, 'path'):
                obj_name = process_arknights_filename(obj.container.path, obj_name, obj_type)
            return obj_name
        
        # Method 3: For textures, check for specific IDs used by Arknights
        if obj_type in ['Texture2D', 'Sprite']:
            if hasattr(data, 'texture') and hasattr(data.texture, 'name') and data.texture.name:
                obj_name = data.texture.name
                # Apply Arknights-specific processing if we have path info
                if hasattr(obj, 'container') and hasattr(obj.container, 'path'):
                    obj_name = process_arknights_filename(obj.container.path, obj_name, obj_type)
                return obj_name
    
    # Try to get path info
    container_path = extract_unity_path(obj)
    if container_path:
        # Extract filename from path
        path_parts = container_path.split('/')
        if path_parts:
            filename = path_parts[-1]
            # Remove extension if present
            if '.' in filename:
                filename = filename.rsplit('.', 1)[0]
            
            obj_name = filename
            # Apply Arknights-specific processing
            if hasattr(obj, 'container') and hasattr(obj.container, 'path'):
                obj_name = process_arknights_filename(obj.container.path, obj_name, obj_type)
            return obj_name
    
    # For Arknights specific files, check if we can extract a character or skin ID
    if hasattr(obj, 'container') and hasattr(obj.container, 'path'):
        path = obj.container.path
        if 'chararts' in path or 'spritepack' in path or 'skinpack' in path:
            parts = path.split('/')
            for part in parts:
                # Arknights character IDs often follow patterns like 'char_123'
                if part.startswith('char_') or part.startswith('skin_'):
                    return f"{part}_{obj_type.lower()}"
    
    # Fallback - use type and ID but still try Arknights-specific processing
    obj_name = f"{obj_type.lower()}_{abs(path_id)}"
    if hasattr(obj, 'container') and hasattr(obj.container, 'path'):
        obj_name = process_arknights_filename(obj.container.path, obj_name, obj_type)
    return obj_name

def process_file(file_path, output_dir, input_base_path, force=False):
    """Process a single AssetBundle file"""
    try:
        debug_print(f"Processing file: {file_path}")
        
        # Create output directory matching the original directory structure
        # Get relative path from input base to the file's directory
        rel_path = file_path.relative_to(input_base_path).parent
        output_subdir = output_dir / rel_path
        
        # Always process the file, regardless of whether it's been processed before
        output_subdir.mkdir(parents=True, exist_ok=True)
        
        debug_print(f"Output directory: {output_subdir}")
        debug_print(f"Loading AssetBundle file...")
        
        # Track container paths for directory structure preservation
        container_paths = {}
        
        # Load the AssetBundle
        env = UnityPy.load(str(file_path))
        
        # First pass - collect container paths
        for obj in env.objects:
            path_id = obj.path_id
            
            # Store container paths if available
            if hasattr(obj, 'container') and hasattr(obj.container, 'path'):
                container_path = obj.container.path
                if container_path:
                    # Parse the container path to extract directory structure
                    container_dirs = os.path.dirname(container_path)
                    if container_dirs:
                        container_paths[path_id] = container_dirs
        
        debug_print(f"AssetBundle loaded successfully")
        
        # Count assets extracted
        extracted_count = 0
        # Track assets by type
        assets_by_type = {}
        
        # Count objects
        object_count = sum(1 for _ in env.objects)
        debug_print(f"Found {object_count} objects in bundle")
        
        # Dictionary to store images for alpha processing
        images = {}
        
        # First pass: collect all images
        for obj in env.objects:
            try:
                obj_type = obj.type.name
                path_id = obj.path_id
                
                debug_print(f"Processing object: Type={obj_type}, PathID={path_id}")
                
                # Handle images (Texture2D, Sprite)
                if obj_type in ['Texture2D', 'Sprite']:
                    # Read the asset
                    data = obj.read()
                    
                    try:
                        # Generate a name if the object doesn't have one
                        obj_name = get_meaningful_name(obj, obj_type, path_id)
                        
                        # Clean up filename to ensure it's valid
                        obj_name = obj_name.replace(':', '_').replace('/', '_').replace('\\', '_')
                        
                        # Check if this object has container path information
                        sub_path = ""
                        if path_id in container_paths:
                            # Create subdirectory based on container path
                            sub_path = container_paths[path_id].replace('/', os.sep)
                            # Create this subdirectory
                            asset_dir = output_subdir / sub_path
                            asset_dir.mkdir(parents=True, exist_ok=True)
                        else:
                            asset_dir = output_subdir
                        
                        # Store image for processing
                        image_key = f"({obj_type}){obj_name}"
                        
                        # Check if data has an image attribute before storing
                        if hasattr(data, 'image'):
                            images[image_key] = data
                            images[image_key + '_dir'] = sub_path  # Store the directory for later use
                            debug_print(f"Stored image with key: {image_key}")
                        else:
                            debug_print(f"{obj_type} object has no image attribute. PathID={path_id}", Colors.RED)
                    except Exception as e:
                        debug_print(f"Error processing {obj_type} object (PathID={path_id}): {str(e)}", Colors.RED)
            except Exception as e:
                print_color(f"Error collecting image in {file_path}: {str(e)}", color=Colors.RED)
        
        # Process and save images
        debug_print(f"Processing {len(images)} collected images")
        for name, data in images.items():
            try:
                # Skip the directory info entries
                if name.endswith('_dir'):
                    continue
                    
                # Skip alpha channel images (they will be merged with their main images)
                if name.endswith('[alpha]'):
                    continue
                    
                # Check if data has image attribute
                if not hasattr(data, 'image') or not data.image:
                    debug_print(f"Object has no valid image attribute: {name}", Colors.RED)
                    continue
                
                # Extract the object name without the type prefix
                obj_name = name
                if '(' in name and ')' in name:
                    obj_name = name.split(')', 1)[1]
                    
                # Get the subdirectory for this image if it exists
                sub_path = ""
                if name + '_dir' in images:
                    sub_path = images[name + '_dir']
                
                # Create the target directory 
                if sub_path:
                    image_dir = output_subdir / sub_path
                    image_dir.mkdir(parents=True, exist_ok=True)
                else:
                    image_dir = output_subdir
                    
                # Check if we have an alpha channel image
                alpha_key = name + '[alpha]'
                has_alpha = (alpha_key in images and 
                           hasattr(images[alpha_key], 'image') and 
                           images[alpha_key].image)
                    
                if has_alpha:
                    try:
                        debug_print(f"Found image with alpha channel: {name}")
                        # Get RGB channels from main image
                        r, g, b = data.image.split()[:3]
                        
                        # Get alpha channel from alpha image
                        a = images[alpha_key].image.split()[0]
                        
                        # Resize alpha if needed
                        if a.size != r.size:
                            a = a.resize(r.size)
                            
                        # Merge channels
                        from PIL import Image
                        merged_image = Image.merge('RGBA', (r, g, b, a))
                        
                        # Save the merged image - preserve original extension if exists
                        if '.' in obj_name and obj_name.split('.')[-1].lower() in ['png', 'jpg', 'jpeg', 'bmp', 'gif']:
                            image_path = image_dir / obj_name
                        else:
                            image_path = image_dir / f"{obj_name}.png"
                            
                        merged_image.save(str(image_path))
                        debug_print(f"Saved merged image to: {image_path}", Colors.GREEN)
                        extracted_count += 1
                        # Track asset type
                        if 'Texture2D' in assets_by_type:
                            assets_by_type['Texture2D'] += 1
                        else:
                            assets_by_type['Texture2D'] = 1
                    except Exception as e:
                        print_color(f"Error merging alpha for {name}: {str(e)}", Colors.RED)
                        # Fallback to saving without alpha
                        try:
                            if '.' in obj_name and obj_name.split('.')[-1].lower() in ['png', 'jpg', 'jpeg', 'bmp', 'gif']:
                                image_path = image_dir / obj_name
                            else:
                                image_path = image_dir / f"{obj_name}.png"
                                
                            data.image.save(str(image_path))
                            debug_print(f"Saved image without alpha to: {image_path}", Colors.GREEN)
                            extracted_count += 1
                            # Track asset type
                            if obj_type in assets_by_type:
                                assets_by_type[obj_type] += 1
                            else:
                                assets_by_type[obj_type] = 1
                        except Exception as inner_e:
                            print_color(f"Error saving fallback image for {name}: {str(inner_e)}", Colors.RED)
                else:
                    # Save the image directly
                    try:
                        if '.' in obj_name and obj_name.split('.')[-1].lower() in ['png', 'jpg', 'jpeg', 'bmp', 'gif']:
                            image_path = image_dir / obj_name
                        else:
                            image_path = image_dir / f"{obj_name}.png"
                            
                        data.image.save(str(image_path))
                        debug_print(f"Saved image to: {image_path}", Colors.GREEN)
                        extracted_count += 1
                        # Track asset type
                        if obj_type in assets_by_type:
                            assets_by_type[obj_type] += 1
                        else:
                            assets_by_type[obj_type] = 1
                    except Exception as e:
                        print_color(f"Error saving image {name}: {str(e)}", Colors.RED)
            except Exception as e:
                print_color(f"Error processing image {name}: {str(e)}", color=Colors.RED)
        
        # Second pass: extract other assets (TextAsset, AudioClip, etc.)
        for obj in env.objects:
            try:
                obj_type = obj.type.name
                path_id = obj.path_id
                
                # Skip images as we've already processed them
                if obj_type in ['Texture2D', 'Sprite']:
                    continue
                    
                # Read the asset
                data = obj.read()
                
                # Generate a name if the object doesn't have one
                obj_name = get_meaningful_name(obj, obj_type, path_id)
                
                # Clean up filename to ensure it's valid
                obj_name = obj_name.replace(':', '_').replace('/', '_').replace('\\', '_')
                
                # Handle TextAsset (text, JSON, binary data)
                if obj_type == 'TextAsset':
                    debug_print(f"Processing TextAsset: {obj_name}")
                    
                    try:
                        file_saved = False
                        
                        # Get the subdirectory for this asset if available
                        sub_path = ""
                        if path_id in container_paths:
                            sub_path = container_paths[path_id].replace('/', os.sep)
                            # Create this subdirectory
                            asset_dir = output_subdir / sub_path
                            asset_dir.mkdir(parents=True, exist_ok=True)
                        else:
                            asset_dir = output_subdir
                        
                        # Special handling for skeleton (.skel) and atlas files
                        if obj_name.endswith('.skel') or obj_name.endswith('.atlas'):
                            # For .skel files, always save as binary, they're Spine skeleton files
                            output_path = asset_dir / obj_name
                            output_path.parent.mkdir(parents=True, exist_ok=True)
                            
                            # Make sure we're using the raw bytes
                            if hasattr(data, 'script'):
                                with open(output_path, 'wb') as f:
                                    f.write(bytes(data.script))
                            elif hasattr(data, 'bytes'):
                                with open(output_path, 'wb') as f:
                                    f.write(data.bytes)
                            elif hasattr(data, 'm_Script'):
                                with open(output_path, 'wb') as f:
                                    f.write(bytes(data.m_Script))
                            else:
                                debug_print(f"Failed to extract data for {obj_name}", Colors.RED)
                                continue
                                
                            extracted_count += 1
                            file_saved = True
                            debug_print(f"Saved skeleton/atlas file: {output_path}", Colors.GREEN)
                            
                            # Track asset type
                            asset_type = 'SpineAsset'
                            if asset_type in assets_by_type:
                                assets_by_type[asset_type] += 1
                            else:
                                assets_by_type[asset_type] = 1
                            continue
                        
                        # Try to access the text content in different ways
                        text_content = None
                        
                        # Method 1: Use the script attribute if available
                        if hasattr(data, 'script'):
                            text_content = data.script
                            debug_print(f"Found text content via 'script' attribute for {obj_name}")
                        # Method 2: Use the text attribute if available
                        elif hasattr(data, 'text'):
                            text_content = data.text
                            debug_print(f"Found text content via 'text' attribute for {obj_name}")
                        # Method 3: Try the m_Script attribute (seen in some Unity versions)
                        elif hasattr(data, 'm_Script'):
                            text_content = data.m_Script
                            debug_print(f"Found text content via 'm_Script' attribute for {obj_name}")
                        # Method 4: Try just accessing data directly
                        elif hasattr(data, 'bytes'):
                            try:
                                text_content = data.bytes.decode('utf-8')
                                debug_print(f"Found text content via 'bytes' attribute for {obj_name}")
                            except UnicodeDecodeError:
                                # If not utf-8, it might be binary data or another encoding
                                debug_print(f"Bytes found but not valid UTF-8 text for {obj_name}, saving as binary")
                                
                                # If text extraction failed and we have binary data, save that
                                if hasattr(data, 'bytes'):
                                    # Check for known binary file extensions
                                    if obj_name.endswith('.bytes'):
                                        # Remove .bytes extension added by Unity
                                        base_name = obj_name[:-6]
                                        output_path = asset_dir / base_name
                                    else:
                                        # Just save with original name
                                        output_path = asset_dir / obj_name
                                        
                                    # Ensure directory exists
                                    output_path.parent.mkdir(parents=True, exist_ok=True)
                                    
                                    with open(output_path, 'wb') as f:
                                        f.write(data.bytes)
                                    extracted_count += 1
                                    file_saved = True
                                    debug_print(f"Saved binary data: {output_path}")
                                    # Track asset type
                                    if 'TextAsset_Binary' in assets_by_type:
                                        assets_by_type['TextAsset_Binary'] += 1
                                    else:
                                        assets_by_type['TextAsset_Binary'] = 1
                        
                        # Save text content if found
                        if text_content and not file_saved:
                            # Convert to string if it's not already
                            if not isinstance(text_content, str):
                                try:
                                    text_content = str(text_content)
                                except Exception as e:
                                    debug_print(f"Failed to convert text_content to string: {str(e)}", Colors.RED)
                                    continue
                            
                            # Determine file extension based on content
                            extension = '.txt'
                            if obj_name.lower().endswith('.json') or (text_content.strip().startswith('{') and text_content.strip().endswith('}')):
                                extension = '.json'
                            elif obj_name.lower().endswith('.xml') or (text_content.strip().startswith('<') and text_content.strip().endswith('>')):
                                extension = '.xml'
                            elif obj_name.lower().endswith('.csv'):
                                extension = '.csv'
                            
                            # Use original extension if present in the name
                            if '.' in obj_name and not obj_name.endswith('.'):
                                parts = obj_name.split('.')
                                potential_ext = parts[-1].lower()
                                if potential_ext in ['txt', 'json', 'xml', 'csv', 'lua', 'html', 'css', 'js']:
                                    output_path = asset_dir / obj_name
                                else:
                                    output_path = asset_dir / f"{obj_name}{extension}"
                            else:
                                output_path = asset_dir / f"{obj_name}{extension}"
                            
                            # Ensure the parent directory exists
                            output_path.parent.mkdir(parents=True, exist_ok=True)
                            
                            with open(output_path, 'w', encoding='utf-8') as f:
                                f.write(text_content)
                            extracted_count += 1
                            debug_print(f"Saved text: {output_path}")
                            # Track asset type
                            if output_path.suffix == '.json':
                                asset_type = 'TextAsset_JSON'
                            elif output_path.suffix == '.xml':
                                asset_type = 'TextAsset_XML'
                            elif output_path.suffix == '.csv':
                                asset_type = 'TextAsset_CSV'
                            else:
                                asset_type = 'TextAsset_Text'
                                
                            if asset_type in assets_by_type:
                                assets_by_type[asset_type] += 1
                            else:
                                assets_by_type[asset_type] = 1
                        elif not file_saved:
                            debug_print(f"Could not extract content from TextAsset {obj_name}", Colors.YELLOW)
                    except Exception as e:
                        debug_print(f"Error processing TextAsset {obj_name}: {str(e)}", Colors.RED)
                
                # Handle AudioClip (sound files)
                elif obj_type == 'AudioClip':
                    debug_print(f"Processing AudioClip: {obj_name}")
                    
                    try:
                        audio_extracted = False
                        
                        # Get the subdirectory for this asset if available
                        sub_path = ""
                        if path_id in container_paths:
                            sub_path = container_paths[path_id].replace('/', os.sep)
                            # Create this subdirectory
                            asset_dir = output_subdir / sub_path
                            asset_dir.mkdir(parents=True, exist_ok=True)
                        else:
                            asset_dir = output_subdir
                        
                        # Process audio samples
                        if hasattr(data, 'samples') and data.samples:
                            for audio_name, audio_data in data.samples.items():
                                # Preserve original audio file extension if present
                                if '.' in audio_name and audio_name.split('.')[-1].lower() in ['wav', 'mp3', 'ogg']:
                                    audio_path = asset_dir / audio_name
                                else:
                                    audio_path = asset_dir / f"{obj_name}_{audio_name}"
                                
                                # Ensure the parent directory exists
                                audio_path.parent.mkdir(parents=True, exist_ok=True)
                                
                                with open(audio_path, 'wb') as f:
                                    f.write(audio_data)
                                debug_print(f"Saved audio to: {audio_path}", Colors.GREEN)
                                extracted_count += 1
                                audio_extracted = True
                                # Track asset type
                                if 'AudioClip' in assets_by_type:
                                    assets_by_type['AudioClip'] += 1
                                else:
                                    assets_by_type['AudioClip'] = 1
                        
                        # Try alternative ways to get audio data
                        if not audio_extracted:
                            if hasattr(data, 'data') and data.data:
                                audio_path = output_subdir / f"{obj_name}.wav"
                                with open(audio_path, 'wb') as f:
                                    f.write(data.data)
                                debug_print(f"Saved audio data to: {audio_path}", Colors.GREEN)
                                extracted_count += 1
                                audio_extracted = True
                            elif hasattr(data, 'raw_data') and data.raw_data:
                                audio_path = output_subdir / f"{obj_name}.bin"
                                with open(audio_path, 'wb') as f:
                                    f.write(data.raw_data)
                                debug_print(f"Saved raw audio data to: {audio_path}", Colors.GREEN)
                                extracted_count += 1
                                audio_extracted = True
                        
                        if not audio_extracted:
                            debug_print(f"AudioClip has no extractable data: {obj_name}", Colors.RED)
                    except Exception as e:
                        debug_print(f"Error processing AudioClip {obj_name}: {str(e)}", Colors.RED)
                
                # Handle MonoBehaviour (custom scripts, might contain game data)
                elif obj_type == 'MonoBehaviour':
                    debug_print(f"Processing MonoBehaviour: {obj_name}")
                    
                    try:
                        mono_extracted = False
                        
                        # Get the subdirectory for this asset if available
                        sub_path = ""
                        if path_id in container_paths:
                            sub_path = container_paths[path_id].replace('/', os.sep)
                            # Create this subdirectory
                            asset_dir = output_subdir / sub_path
                            asset_dir.mkdir(parents=True, exist_ok=True)
                        else:
                            asset_dir = output_subdir
                        
                        # Try to convert to dict and save as JSON
                        if hasattr(data, 'to_dict'):
                            try:
                                json_data = data.to_dict()
                                
                                # Preserve original filename if it has .json extension
                                if obj_name.lower().endswith('.json'):
                                    json_path = asset_dir / obj_name
                                else:
                                    json_path = asset_dir / f"{obj_name}.json"
                                    
                                # Ensure parent directory exists
                                json_path.parent.mkdir(parents=True, exist_ok=True)
                                
                                with open(json_path, 'w', encoding='utf-8') as f:
                                    json.dump(json_data, f, ensure_ascii=False, indent=2, cls=UnityObjectEncoder)
                                debug_print(f"Saved MonoBehaviour as JSON: {json_path}", Colors.GREEN)
                                extracted_count += 1
                                mono_extracted = True
                                # Track asset type
                                if 'MonoBehaviour_JSON' in assets_by_type:
                                    assets_by_type['MonoBehaviour_JSON'] += 1
                                else:
                                    assets_by_type['MonoBehaviour_JSON'] = 1
                            except Exception as e:
                                debug_print(f"Error converting MonoBehaviour to JSON: {str(e)}", Colors.RED)
                        
                        # Try alternative approach - raw serialized content
                        if not mono_extracted and hasattr(data, 'raw_data'):
                            try:
                                # Preserve original filename if it has extension
                                if '.' in obj_name and not obj_name.endswith('.'):
                                    raw_path = asset_dir / obj_name
                                else:
                                    raw_path = asset_dir / f"{obj_name}_raw.bin"
                                    
                                # Ensure parent directory exists
                                raw_path.parent.mkdir(parents=True, exist_ok=True)
                                
                                with open(raw_path, 'wb') as f:
                                    f.write(data.raw_data)
                                debug_print(f"Saved MonoBehaviour raw data: {raw_path}", Colors.GREEN)
                                extracted_count += 1
                                mono_extracted = True
                                # Track asset type
                                if 'MonoBehaviour_Raw' in assets_by_type:
                                    assets_by_type['MonoBehaviour_Raw'] += 1
                                else:
                                    assets_by_type['MonoBehaviour_Raw'] = 1
                            except Exception as e:
                                debug_print(f"Error saving MonoBehaviour raw data: {str(e)}", Colors.RED)
                        
                        # Try to save any available properties as a simple JSON structure
                        if not mono_extracted:
                            try:
                                props = {}
                                for attr in dir(data):
                                    if not attr.startswith('_') and not callable(getattr(data, attr)):
                                        try:
                                            value = getattr(data, attr)
                                            # Only include simple types that can be JSON serialized
                                            if isinstance(value, (str, int, float, bool, list, dict)) or value is None:
                                                props[attr] = value
                                        except:
                                            pass
                                
                                if props:
                                    props_path = asset_dir / f"{obj_name}_props.json"
                                    with open(props_path, 'w', encoding='utf-8') as f:
                                        json.dump(props, f, ensure_ascii=False, indent=2, cls=UnityObjectEncoder)
                                    debug_print(f"Saved MonoBehaviour properties: {props_path}", Colors.GREEN)
                                    extracted_count += 1
                                    mono_extracted = True
                                    # Track asset type
                                    if 'MonoBehaviour_Props' in assets_by_type:
                                        assets_by_type['MonoBehaviour_Props'] += 1
                                    else:
                                        assets_by_type['MonoBehaviour_Props'] = 1
                            except Exception as e:
                                debug_print(f"Error saving MonoBehaviour properties: {str(e)}", Colors.RED)
                        
                        if not mono_extracted:
                            debug_print(f"Could not extract data from MonoBehaviour: {obj_name}", Colors.RED)
                            
                    except Exception as e:
                        debug_print(f"Failed to process MonoBehaviour {obj_name}: {str(e)}", Colors.RED)
            
            except Exception as e:
                print_color(f"Error processing object in {file_path}: {e}", color=Colors.RED)
        
        debug_print(f"Extracted {extracted_count} assets from {file_path}")
        return extracted_count, assets_by_type
    
    except Exception as e:
        print_color(f"Failed to process {file_path}: {e}", color=Colors.RED)
        return 0, {}

def main():
    """Main function"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Simplified Arknights Asset Unpacker')
    parser.add_argument('-i', '--input', default='./ArkAssets',
                        help='Input directory or file containing AssetBundle files')
    parser.add_argument('-o', '--output', default='./Unpacked',
                        help='Output directory for unpacked assets')
    parser.add_argument('-d', '--dirs', nargs='+', default=None,
                        help='Specific directories to process (space-separated, e.g., "charpack skinpack ui audio")')
    parser.add_argument('--all', action='store_true',
                        help='Process all directories in the input path')
    parser.add_argument('--debug', action='store_true',
                        help='Enable debug mode')
    parser.add_argument('-f', '--force', action='store_true',
                        help='Force reprocessing of files even if already decompressed')
    parser.add_argument('--list-dirs', action='store_true',
                        help='List available directories in the input path and exit')
    
    args = parser.parse_args()
    
    # Set debug mode
    global debug_mode
    debug_mode = args.debug or debug_mode
    
    # Create paths
    input_path = Path(args.input)
    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print_color(f"Input directory:  {input_path}", color=Colors.CYAN)
    print_color(f"Output directory: {output_dir}", color=Colors.CYAN)
    
    # List directories and exit if requested
    if args.list_dirs and input_path.is_dir():
        print_color("Available directories for extraction:", color=Colors.GREEN)
        dirs = [d.name for d in input_path.iterdir() if d.is_dir()]
        for d in sorted(dirs):
            print_color(f"  - {d}", color=Colors.YELLOW)
        return
    
    # Find all AB files
    ab_files = []
    
    # Check if input is a direct file
    if input_path.is_file() and input_path.suffix == '.ab':
        ab_files.append(input_path)
        # For single file input, use the parent directory as input_base_path
        input_base_path = input_path.parent
        print_color(f"Processing single file: {input_path}", color=Colors.GREEN)
    else:
        # Set the input base path for directory operations
        input_base_path = input_path
        
        # Handle directory input
        if args.all:
            # Process all directories
            print_color("Processing all directories in input path", color=Colors.GREEN)
            for root, _, files in os.walk(input_path):
                for file in files:
                    if file.endswith('.ab'):
                        ab_files.append(Path(root) / file)
        elif args.dirs:
            # Process specified directories
            print_color(f"Processing specified directories: {', '.join(args.dirs)}", color=Colors.GREEN)
            for target in args.dirs:
                target_path = input_path / target
                if target_path.exists():
                    if target_path.is_file() and target_path.suffix == '.ab':
                        ab_files.append(target_path)
                    else:
                        for root, _, files in os.walk(target_path):
                            for file in files:
                                if file.endswith('.ab'):
                                    ab_files.append(Path(root) / file)
                else:
                    print_color(f"Warning: Target {target_path} does not exist", color=Colors.YELLOW)
        else:
            # If no dirs or all flag specified, show help
            parser.print_help()
            print_color("\nNo directories specified. Use -d/--dirs to specify directories or --all to process all directories.", color=Colors.YELLOW)
            print_color("Use --list-dirs to see available directories in the input path.", color=Colors.YELLOW)
            return
    
    # Print info
    print_color(f"Found {len(ab_files)} AssetBundle files to process", color=Colors.GREEN)
    
    # Enable more detailed error reporting
    if debug_mode:
        import traceback
        def exception_handler(exc_type, exc_value, exc_traceback):
            print_color("".join(traceback.format_exception(exc_type, exc_value, exc_traceback)), Colors.RED)
        sys.excepthook = exception_handler
    
    # Process files
    total_extracted = 0
    processed_dirs = set()
    assets_by_type = {}  # Track assets by type
    start_time = time.time()
    
    with tqdm(total=len(ab_files), desc="Processing files", unit="files") as pbar:
        for file_path in ab_files:
            extracted, asset_types = process_file(file_path, output_dir, input_base_path, args.force)
            total_extracted += extracted
            
            # Track the processed directory
            rel_path = file_path.relative_to(input_base_path)
            dir_parts = str(rel_path).split(os.sep)
            if len(dir_parts) > 0:
                processed_dirs.add(dir_parts[0])
            
            # Track assets by type
            for asset_type, count in asset_types.items():
                if asset_type in assets_by_type:
                    assets_by_type[asset_type] += count
                else:
                    assets_by_type[asset_type] = count
            
            pbar.update(1)
    
    # Print stats
    elapsed = time.time() - start_time
    print_color("\nProcessing completed!", color=Colors.GREEN + Colors.BOLD)
    print_color("=" * 50, color=Colors.GREEN)
    print_color(f"Total time: {elapsed:.2f} seconds", color=Colors.CYAN)
    print_color(f"Files processed: {len(ab_files)}", color=Colors.CYAN)
    print_color(f"Total assets extracted: {total_extracted}", color=Colors.CYAN)
    
    # Print stats by directory
    if processed_dirs:
        print_color("\nProcessed directories:", color=Colors.MAGENTA)
        for dir_name in sorted(processed_dirs):
            print_color(f"  - {dir_name}", color=Colors.YELLOW)
    
    # Print stats by asset type
    if assets_by_type:
        print_color("\nAssets by type:", color=Colors.MAGENTA)
        for asset_type, count in sorted(assets_by_type.items(), key=lambda x: x[1], reverse=True):
            print_color(f"  - {asset_type}: {count}", color=Colors.YELLOW)
    
    print_color("=" * 50, color=Colors.GREEN)

if __name__ == "__main__":
    main() 