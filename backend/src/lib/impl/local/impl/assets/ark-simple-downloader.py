#!/usr/bin/env python3
# Simple Arknights Asset Downloader
# Based on https://github.com/ChaomengOrion/ArkAssetsTool

import json
import os
import sys
import requests
import shutil
from pathlib import Path

def print_status(message):
    """Print status message with timestamp"""
    print(f"[INFO] {message}")

def download_arknights_assets(output_dir):
    """Download Arknights assets to the specified directory"""
    print_status(f"Starting download to: {output_dir}")
    
    # Create output directory if it doesn't exist
    os.makedirs(output_dir, exist_ok=True)
    
    # Get game version info
    print_status("Fetching game versions...")
    try:
        version_info = requests.get('https://ak-conf.hypergryph.com/config/prod/official/Android/version').json()
        res_version = version_info['resVersion']
        client_version = version_info['clientVersion']
        print_status(f"Game Version: {client_version}, Resource Version: {res_version}")
    except Exception as e:
        print(f"[ERROR] Failed to get version info: {e}")
        return False
    
    # Get hot update list
    print_status("Fetching resource list...")
    try:
        hot_update_url = f'https://ak.hycdn.cn/assetbundle/official/Android/assets/{res_version}/hot_update_list.json'
        hot_update_data = requests.get(hot_update_url).json()
        
        # Count total files and size
        total_files = len(hot_update_data['abInfos'])
        total_size = sum(item['totalSize'] for item in hot_update_data['abInfos'])
        print_status(f"Found {total_files} files, total size: {total_size / (1024*1024):.2f} MB")
    except Exception as e:
        print(f"[ERROR] Failed to get resource list: {e}")
        return False
    
    # Create persistent resource list file
    persistent_file = os.path.join(output_dir, 'persistent_res_list.json')
    if not os.path.exists(persistent_file):
        with open(persistent_file, 'w') as f:
            f.write('{}')
    
    try:
        with open(persistent_file, 'r') as f:
            persistent_data = json.load(f)
    except Exception:
        persistent_data = {}
    
    # Download each file
    print_status("Starting downloads...")
    current_file = 0
    session = requests.Session()
    
    for item in hot_update_data['abInfos']:
        current_file += 1
        file_name = item['name']
        file_md5 = item['md5']
        file_size = item['totalSize']
        
        # Skip if already downloaded
        if file_name in persistent_data and persistent_data[file_name] == file_md5:
            print_status(f"[{current_file}/{total_files}] Skipping {file_name} (already exists)")
            continue
        
        print_status(f"[{current_file}/{total_files}] Downloading {file_name} ({file_size / (1024*1024):.2f} MB)")
        
        try:
            # Format the URL according to Arknights CDN rules
            url = f'https://ak.hycdn.cn/assetbundle/official/Android/assets/{res_version}/{file_name.replace("/", "_").replace(".", ".dat")}'
            
            # Download the file
            response = session.get(url, stream=True)
            response.raise_for_status()
            
            # Save to disk
            output_path = os.path.join(output_dir, file_name)
            os.makedirs(os.path.dirname(output_path), exist_ok=True)
            
            with open(output_path, 'wb') as f:
                for chunk in response.iter_content(chunk_size=8192):
                    if chunk:
                        f.write(chunk)
            
            # Update persistent data
            persistent_data[file_name] = file_md5
            with open(persistent_file, 'w') as f:
                json.dump(persistent_data, f)
                
            print_status(f"✓ Completed {file_name}")
            
        except Exception as e:
            print(f"[ERROR] Failed to download {file_name}: {e}")
    
    print_status("Download completed!")
    return True

if __name__ == "__main__":
    # Get output directory from command line args
    output_dir = "./downloads"
    if len(sys.argv) > 1:
        output_dir = sys.argv[1]
    
    print("Simple Arknights Asset Downloader")
    print("=================================")
    
    success = download_arknights_assets(output_dir)
    
    if success:
        print("\nDownload completed successfully!")
    else:
        print("\nDownload failed!")
        sys.exit(1) 