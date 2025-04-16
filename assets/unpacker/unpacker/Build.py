# -*- coding: utf-8 -*-
# Copyright (c) 2022-2025, Harry Huang
# @ BSD 3-Clause License
import os
import os.path as osp
import toml
import platform
import site
import sys
import subprocess
import tempfile


def __normalize_path(path):
    """Normalize path separators based on the operating system."""
    if platform.system() == 'Windows':
        return path.replace('/', '\\')
    return path.replace('\\', '/')


def __get_site_packages_dir():
    """Get the site-packages directory for the current Python environment."""
    if hasattr(sys, 'real_prefix') or (hasattr(sys, 'base_prefix') and sys.base_prefix != sys.prefix):
        # We're in a virtual environment
        return site.getsitepackages()[0]
    else:
        # We're in the system Python
        return site.getsitepackages()[0]


def __get_venv_dir():
    import re, subprocess

    rst = subprocess.run(["poetry", "env", "info"], capture_output=True)

    if rst.returncode == 0:
        for l in rst.stdout.splitlines():
            match = re.search(r"Path:\s+(.+)", str(l, encoding="UTF-8"))
            if match:
                path = match.group(1).strip()
                if osp.isdir(path):
                    return path
        print("× Failed to parse poetry output to query venv dir.")
    else:
        print(
            f"× Failed to run poetry to query venv dir. Returned code: {rst.returncode}"
        )
        print(f"- StdErr: {rst.stderr}")
        print(f"- StdOut: {rst.stdout}")
    print("- Please check the compatibility of poetry version.")
    print("- Please check the poetry status and the venv info.")
    raise Exception("venv dir not found or poetry config failed")


def __get_proj_info():
    try:
        config = toml.load("pyproject.toml")
        config = config["project"]
        return {
            "name": config["name"],
            "version": config["version"],
            "description": config["description"],
            "author": config["authors"][0]["name"],
            "license": config["license"],
        }
    except KeyError as arg:
        print(f"x Required field missing, {arg}")
        raise arg
    except Exception as arg:
        print("× Failed to parse poetry project info.")
        raise arg


def __find_unitypy_files():
    """Find UnityPy files in the site-packages directory."""
    site_packages_dir = __get_site_packages_dir()
    unitypy_dir = osp.join(site_packages_dir, "UnityPy")
    
    if not osp.exists(unitypy_dir):
        print(f"Warning: UnityPy directory not found at {unitypy_dir}")
        return None, None
    
    lib_dir = osp.join(unitypy_dir, "lib")
    resources_dir = osp.join(unitypy_dir, "resources")
    
    if not osp.exists(lib_dir):
        print(f"Warning: UnityPy lib directory not found at {lib_dir}")
        lib_dir = None
    
    if not osp.exists(resources_dir):
        print(f"Warning: UnityPy resources directory not found at {resources_dir}")
        resources_dir = None
    
    return lib_dir, resources_dir


def __find_archspec_files():
    """Find archspec files in the site-packages directory."""
    site_packages_dir = __get_site_packages_dir()
    archspec_dir = osp.join(site_packages_dir, "archspec")
    
    if not osp.exists(archspec_dir):
        print(f"Warning: archspec directory not found at {archspec_dir}")
        return None, None
    
    # Look for the json directory in multiple possible locations
    json_dir = None
    possible_json_paths = [
        osp.join(archspec_dir, "json"),
        osp.join(archspec_dir, "cpu", "json"),
        osp.join(archspec_dir, "..", "archspec", "json")
    ]
    
    for path in possible_json_paths:
        if osp.exists(path):
            json_dir = path
            print(f"Found archspec json directory at: {json_dir}")
            break
    
    if not json_dir:
        print(f"Warning: archspec json directory not found in any of the expected locations")
        return None, None
    
    # Look for microarchitectures.json
    microarch_path = osp.join(json_dir, "cpu", "microarchitectures.json")
    if not osp.exists(microarch_path):
        # Try alternative locations
        alt_paths = [
            osp.join(json_dir, "microarchitectures.json"),
            osp.join(archspec_dir, "json", "cpu", "microarchitectures.json"),
            osp.join(archspec_dir, "cpu", "json", "microarchitectures.json")
        ]
        for path in alt_paths:
            if osp.exists(path):
                microarch_path = path
                break
    
    if not osp.exists(microarch_path):
        print(f"Warning: microarchitectures.json not found")
        return json_dir, None
    
    print(f"Found microarchitectures.json at: {microarch_path}")
    return json_dir, microarch_path


def __get_build_def(proj_dir, venv_dir):
    try:
        config = toml.load("pyproject.toml")
        build_def = {}
        
        # Get the site-packages directory
        site_packages_dir = __get_site_packages_dir()
        
        # Find UnityPy and archspec files
        unitypy_lib, unitypy_resources = __find_unitypy_files()
        archspec_json, microarch_json = __find_archspec_files()
        
        for k, v in config["tool"]["build"].items():
            # Replace $venv$ with the actual site-packages directory
            if "$venv$" in v:
                v = v.replace("$venv$", site_packages_dir)
            
            # Normalize path separators based on OS
            normalized_path = __normalize_path(v)
            build_def[k] = normalized_path.replace("$project$", proj_dir)
        
        # Override add-binary and add-data with actual paths if found
        if unitypy_lib and unitypy_resources:
            build_def["add-binary"] = f"{unitypy_lib}:UnityPy/lib|{unitypy_resources}/uncompressed.tpk:UnityPy/resources"
        
        # Only add the CN file to add-data, archspec files will be handled separately
        build_def["add-data"] = f"{proj_dir}/src/fbs/CN:src/fbs/CN"
        
        return build_def
    except Exception as arg:
        print("× Failed to parse build definition fields.")
        raise arg


def __main():
    proj_dir = osp.dirname(osp.abspath(__file__))
    venv_dir = __get_venv_dir()
    proj_info = __get_proj_info()
    build_def = __get_build_def(proj_dir, venv_dir)
    print(
        f"Project: {proj_info['name']}|{proj_info['version']}|{proj_info['author']}|{proj_info['license']}"
    )
    print(f"Root: {proj_dir}")
    print(f"Venv: {venv_dir}")
    print("")
    __build(proj_info, proj_dir, build_def)
    exit(0)


def __exec(cmd):
    rst = os.system(cmd)
    if rst == 0:
        print(f"\n[Done] <- {cmd}")
    else:
        print(f"\n[Error] <- {cmd}")
        print(f"× Execution failed! Returned code: {rst}")
        exit(1)


def __build(proj_info, proj_dir, build_def):
    import time, shutil
    import tempfile

    t1 = time.time()

    print(f"Removing build dir...")
    os.chdir(proj_dir)
    build_dir = build_def["build-dir"]
    if osp.exists(build_dir):
        shutil.rmtree(build_dir, ignore_errors=False)

    print(f"Creating build dir...")
    os.makedirs(build_dir, exist_ok=True)
    os.chdir(build_dir)

    print(f"Creating version file...")
    version_file = "version.txt"
    with open(version_file, "w", encoding="UTF-8") as f:
        # spell-checker: disable
        f.write(
            f"""# UTF-8
VSVersionInfo(
  ffi=FixedFileInfo(
filevers=({proj_info['version'].replace('.',',')},0),
prodvers=({proj_info['version'].replace('.',',')},0),
mask=0x3f,
flags=0x0,
OS=0x4,
fileType=0x1,
subtype=0x0,
date=(0,0)
),
  kids=[
StringFileInfo([
  StringTable(
    u'040904B0',
    [StringStruct(u'CompanyName', u'{proj_info['author']}'),
    StringStruct(u'FileDescription', u'{proj_info['description']}'),
    StringStruct(u'FileVersion', u'{proj_info['version']}'),
    StringStruct(u'LegalCopyright', u'©{proj_info['author']} @{proj_info['license']} License'),
    StringStruct(u'ProductName', u'{proj_info['name']}'),
    StringStruct(u'ProductVersion', u'{proj_info['version']}')])
  ])
])
"""
        )  # End f.write
        # spell-checker: enable

    print("Running pyinstaller...")
    cmd_pyinstaller = f"poetry run pyinstaller -F"
    cmd_pyinstaller += f" --name \"{proj_info['name']}-v{proj_info['version']}\""
    cmd_pyinstaller += f" --version-file {version_file}"
    cmd_pyinstaller += (
        f" --icon \"{build_def['icon']}\"" if "icon" in build_def.keys() else ""
    )
    
    # Add macOS-specific options
    if platform.system() == 'Darwin':
        arch = platform.machine()
        print(f"Building for macOS architecture: {arch}")
    
    # Check if the binary and data files exist before adding them
    if "add-binary" in build_def.keys():
        for i in build_def["add-binary"].split("|"):
            if i:
                src, dst = i.split(":")
                src = __normalize_path(src)
                dst = __normalize_path(dst)
                
                if osp.exists(src):
                    cmd_pyinstaller += f' --add-binary "{src}:{dst}"'
                else:
                    print(f"Warning: Binary file not found: {src}")
    
    if "add-data" in build_def.keys():
        for i in build_def["add-data"].split("|"):
            if i:
                src, dst = i.split(":")
                src = __normalize_path(src)
                dst = __normalize_path(dst)
                
                if osp.exists(src):
                    cmd_pyinstaller += f' --add-data "{src}:{dst}"'
                else:
                    print(f"Warning: Data file not found: {src}")
    
    # Add archspec data files explicitly
    archspec_json, microarch_json = __find_archspec_files()
    if microarch_json and osp.exists(microarch_json):
        # Create a temporary directory for the archspec files
        temp_dir = tempfile.mkdtemp(prefix="archspec_")
        print(f"Created temporary directory for archspec files: {temp_dir}")
        
        # Create the exact directory structure that the program is looking for
        # The error shows it's looking for: /var/folders/w8/c_z68kcn4w71gyw4zn51m5br0000gn/T/_MEIe8ip9F/archspec/cpu/../json/cpu/microarchitectures.json
        
        # Create the base archspec directory
        archspec_dir = osp.join(temp_dir, "archspec")
        os.makedirs(archspec_dir, exist_ok=True)
        
        # Create the cpu directory
        cpu_dir = osp.join(archspec_dir, "cpu")
        os.makedirs(cpu_dir, exist_ok=True)
        
        # Create the json directory at the same level as cpu
        json_dir = osp.join(archspec_dir, "json")
        os.makedirs(json_dir, exist_ok=True)
        
        # Create the cpu directory inside json
        json_cpu_dir = osp.join(json_dir, "cpu")
        os.makedirs(json_cpu_dir, exist_ok=True)
        
        # Copy the microarchitectures.json file to both locations
        microarch_dest1 = osp.join(cpu_dir, "microarchitectures.json")
        microarch_dest2 = osp.join(json_cpu_dir, "microarchitectures.json")
        
        shutil.copy2(microarch_json, microarch_dest1)
        shutil.copy2(microarch_json, microarch_dest2)
        
        print(f"Copied microarchitectures.json to {microarch_dest1}")
        print(f"Copied microarchitectures.json to {microarch_dest2}")
        
        # Add the entire archspec directory to PyInstaller
        cmd_pyinstaller += f' --add-data "{archspec_dir}:archspec"'
    
    if "hidden-import" in build_def.keys():
        for i in build_def["hidden-import"].split("|"):
            if i and i != "pkg_resources.py2_warn":  # Skip problematic import
                cmd_pyinstaller += f' --hidden-import "{i}"'
    
    # Add additional hidden imports for macOS
    if platform.system() == 'Darwin':
        cmd_pyinstaller += ' --hidden-import "pkg_resources._vendor.jaraco.functools"'
        cmd_pyinstaller += ' --hidden-import "pkg_resources._vendor.jaraco.context"'
        cmd_pyinstaller += ' --hidden-import "pkg_resources._vendor.jaraco.text"'
        cmd_pyinstaller += ' --hidden-import "archspec"'
        cmd_pyinstaller += ' --hidden-import "archspec.cpu"'
    
    cmd_pyinstaller += (
        f" --log-level {build_def['log-level']}"
        if "log-level" in build_def.keys()
        else ""
    )
    
    cmd_pyinstaller += f" \"{build_def['entry']}\""
    __exec(cmd_pyinstaller)
    
    # Clean up the temporary directory
    try:
        shutil.rmtree(temp_dir)
        print(f"Cleaned up temporary directory: {temp_dir}")
    except Exception as e:
        print(f"Warning: Failed to clean up temporary directory: {e}")

    print(f"√ Build finished in {round(time.time() - t1, 1)}s!")
    print(f"- Dist files see: {osp.join(build_dir, 'dist')}")


if __name__ == "__main__":
    __main()
