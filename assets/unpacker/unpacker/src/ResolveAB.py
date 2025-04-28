# -*- coding: utf-8 -*-
# Copyright (c) 2022-2025, Harry Huang
# @ BSD 3-Clause License
import os
import os.path as osp
from contextlib import ContextDecorator
from typing import Any, Callable, Optional, Sequence, List, Set
import json
import time

import UnityPy
import UnityPy.classes as uc
from UnityPy.enums.BundleFile import CompressionFlags
from UnityPy.files.File import File
from UnityPy.helpers import CompressionHelper
from UnityPy.streams.EndianBinaryReader import EndianBinaryReader
from PIL import Image

from .CombineRGBwithA import AlphaRGBCombiner
from .lz4ak.Block import decompress_lz4ak
from .utils.GlobalMethods import print, rmdir, get_filelist, is_ab_file, stacktrace
from .utils.Logger import Logger
from .utils.SaverUtils import SafeSaver
from .utils.TaskUtils import ThreadCtrl, UICtrl, TaskReporter, TaskReporterTracker
from .utils.AtlasFile import AtlasFile

# New compression algorithms introduced in Arknights
# LZ4AK (v2.5.04+) - Currently used but labeled as LZHAM
CompressionHelper.DECOMPRESSION_MAP[CompressionFlags.LZHAM] = decompress_lz4ak

# Real LZHAM support (for future versions)
try:
    from .lzham.Block import decompress_lzham
    # Define a custom LZHAM compression flag since it might not be in the enum yet
    # This will need to be updated when the actual enum value is known
    LZHAM_COMPRESSION_FLAG = 4  # This value may need to be adjusted
    CompressionHelper.DECOMPRESSION_MAP[LZHAM_COMPRESSION_FLAG] = decompress_lzham
    Logger.info("LZHAM decompression support enabled")
except ImportError as e:
    Logger.info(f"LZHAM decompression not available: {e}")
    Logger.info("Only LZ4AK decompression will be used (current Arknights version)")


class Resource:
    """The class representing a collection of the objects in an UnityPy Environment."""

    def __init__(self, env: UnityPy.Environment):
        """Initializes with the given UnityPy Environment instance.

        :param env: The Environment instance from `UnityPy.load()`;
        :rtype: None;
        """
        if isinstance(env.file, File):
            self.name: str = env.file.name
        elif isinstance(env.file, EndianBinaryReader):
            self.name: str = ""
        else:
            raise TypeError(
                f"Unknown type of UnityPy Environment file: {type(env.file).__name__}"
            )
        self.env: UnityPy.Environment = env
        self.length: int = len(env.objects)
        ###
        self.sprites: "list[uc.Sprite]" = []
        self.texture2ds: "list[uc.Texture2D]" = []
        self.textassets: "list[uc.TextAsset]" = []
        self.audioclips: "list[uc.AudioClip]" = []
        self.materials: "list[uc.Material]" = []
        self.monobehaviors: "list[uc.MonoBehaviour]" = []
        self.spines: "list[Resource.SpineAsset]" = []
        ###
        for i in [o.read() for o in env.objects]:
            if isinstance(i, uc.Sprite):
                self.sprites.append(i)
            elif isinstance(i, uc.Texture2D):
                self.texture2ds.append(i)
            elif isinstance(i, uc.TextAsset) and not isinstance(i, uc.MonoScript):
                self.textassets.append(i)
            elif isinstance(i, uc.AudioClip):
                self.audioclips.append(i)
            elif isinstance(i, uc.Material):
                self.materials.append(i)
            elif isinstance(i, uc.MonoBehaviour):
                self.monobehaviors.append(i)
            elif isinstance(i, uc.AssetBundle):
                if getattr(i, "m_Name", None):
                    if self.name != osp.basename(i.m_Name):
                        Logger.debug(
                            f'ResolveAB: Resource "{self.name}" internally named "{i.m_Name}"'
                        )
                        self.name = osp.basename(i.m_Name)

    def get_object_by_pathid(
        self, pathid: "int|dict", search_in: "Sequence[uc.Object]"
    ):
        """Gets the object with the given PathID.

        :param pathid: PathID in int or a dict containing `m_PathID` field;
        :param search_in: Searching range;
        :returns: The object, `None` for not found;
        """
        _key = "m_PathID"
        if isinstance(pathid, dict):
            if _key in pathid:
                _pathid = int(pathid[_key])
            else:
                return None
        else:
            _pathid = pathid
        for i in search_in:
            if i.object_reader is not None and i.object_reader.path_id == _pathid:
                return i
        return None

    def sort_skeletons(self):
        """Sorts the Spine assets.

        :rtype: None;
        """
        spines: "list[Resource.SpineAsset]" = []
        try:
            for mono in self.monobehaviors:
                # (i stands for a MonoBehavior)
                with Resource.TreeReader(mono) as tree:
                    # As asset:
                    if "skeletonDataAsset" not in tree.keys():
                        continue  # Skip non-skeleton asset
                    mono_sd = self.get_object_by_pathid(
                        tree["skeletonDataAsset"], self.monobehaviors
                    )
                    if mono_sd is None:
                        Logger.debug(f'ResolveAB: Skipping skeleton with None skeletonDataAsset in resource "{self.name}"')
                        continue
                    with Resource.TreeReader(mono_sd) as tree_sd:
                        # As skeleton data asset:
                        skel = self.get_object_by_pathid(
                            tree_sd["skeletonJSON"], self.textassets
                        )
                        if skel is None:
                            Logger.debug(f'ResolveAB: Skipping skeleton with None skeletonJSON in resource "{self.name}"')
                            continue
                            
                        mono_ad = self.get_object_by_pathid(
                            tree_sd["atlasAssets"][0], self.monobehaviors
                        )
                        if mono_ad is None:
                            Logger.debug(f'ResolveAB: Skipping skeleton with None atlasAssets in resource "{self.name}"')
                            continue
                            
                        with Resource.TreeReader(mono_ad) as tree_ad:
                            # As atlas data asset:
                            atlas = self.get_object_by_pathid(
                                tree_ad["atlasFile"], self.textassets
                            )
                            if atlas is None:
                                Logger.debug(f'ResolveAB: Skipping skeleton with None atlasFile in resource "{self.name}"')
                                continue
                                
                            list2mat = [
                                self.get_object_by_pathid(i, self.materials)
                                for i in tree_ad["materials"]
                            ]
                            list2tex = []
                            for mat in list2mat:
                                if mat is None:
                                    Logger.debug(f'ResolveAB: Skipping material with None value in resource "{self.name}"')
                                    continue
                                tex_rgb, tex_alpha = None, None
                                with Resource.TreeReader(mat) as tree_mat:
                                    # As material asset:
                                    tex_envs = tree_mat["m_SavedProperties"][
                                        "m_TexEnvs"
                                    ]
                                    for tex in tex_envs:
                                        if tex[0] == "_MainTex":
                                            tex_rgb = self.get_object_by_pathid(
                                                tex[1]["m_Texture"], self.texture2ds
                                            )
                                        elif tex[0] == "_AlphaTex":
                                            tex_alpha = self.get_object_by_pathid(
                                                tex[1]["m_Texture"], self.texture2ds
                                            )
                                list2tex.append((tex_rgb, tex_alpha))
                            # Pack into Spine asset instance
                            spine = Resource.SpineAsset(
                                skel, atlas, list2tex, tree.get("_animationName", None)
                            )
                            spine.add_prefix()
                            spines.append(spine)
        except Exception as arg:
            Logger.warn(
                f'ResolveAB: Failed to handle skeletons in resource "{self.name}": {stacktrace()}'
            )
        # Store the collected spines
        self.spines = spines

    class TreeReader(ContextDecorator):
        """Reader of the serialized type tree of Unity objects."""

        def __init__(self, obj: "uc.Object|None"):
            self.obj = obj.object_reader if isinstance(obj, uc.Object) else obj

        def __enter__(self):
            if self.obj is None:
                raise AttributeError("Given object or object reader is none")
            if self.obj.serialized_type and getattr(self.obj.serialized_type, "nodes"):
                tree = self.obj.read_typetree()
                if isinstance(tree, dict):
                    return tree
            raise AttributeError("Given object has no serialized type tree")

        def __exit__(self, exc_type, exc_val, exc_tb):
            return False  # Hand down the exception

    class SpineAsset:
        UNKNOWN = "Unknown"
        BUILDING = "Building"
        BATTLE_FRONT = "BattleFront"
        BATTLE_BACK = "BattleBack"
        DYN_ILLUST = "DynIllust"

        def __init__(
            self,
            skel: Any,
            atlas: Any,
            tex_list: "list[tuple[uc.Texture2D,uc.Texture2D]]",
            anim_list: "list[str]|None",
        ):
            # Validate arguments
            if not isinstance(skel, uc.TextAsset) or not isinstance(
                atlas, uc.TextAsset
            ):
                raise TypeError("Spine asset unavailable, bad skel or atlas")
            if not isinstance(tex_list, list) or len(tex_list) == 0:
                raise TypeError("Spine asset unavailable, bad textures")
            self.skel = skel
            self.atlas = atlas
            self.tex_list = tex_list
            self.type = Resource.SpineAsset.UNKNOWN
            # Determine the type
            if skel.m_Name.lower().startswith("dyn_"):
                self.type = Resource.SpineAsset.DYN_ILLUST
            elif (
                anim_list
                and "Relax" in anim_list
                or skel.m_Name.lower().startswith("build_")
            ):
                self.type = Resource.SpineAsset.BUILDING
            else:
                t = self.atlas.m_Script.lower()
                if t.count("\nf_") + t.count("\nc_") >= t.count("\nb_"):
                    self.type = Resource.SpineAsset.BATTLE_FRONT
                else:
                    self.type = Resource.SpineAsset.BATTLE_BACK
                    
            # Parse the atlas file to get dimensions
            self.atlas_data = None
            try:
                self.atlas_data = AtlasFile.loads(self.atlas.m_Script)
                Logger.debug(f'ResolveAB: Successfully parsed atlas file "{self.atlas.m_Name}"')
            except Exception as e:
                Logger.warn(f'ResolveAB: Failed to parse atlas file "{self.atlas.m_Name}": {e}')

        def add_prefix(self):
            """Renames the Spine assets which includes skel, atlas and png files.
            Since the Spine in Arknights have 4 or more forms (Building, BattleFront, BattleBack, DynIllust),
            it is necessary to rename them so that name collisions can be avoided.

            :rtype: None;
            """

            def _add_prefix(obj: "uc.TextAsset|uc.Texture2D", pre: str):
                if obj and not obj.m_Name.startswith(pre):
                    # Get just the basename without any path components
                    basename = osp.basename(obj.m_Name)
                    # Create the new name with just the prefix and basename
                    obj.m_Name = pre + basename

            # Get the prefix string - use only the type and atlas name without path components
            atlas_basename = osp.splitext(osp.basename(self.atlas.m_Name))[0]
            prefix = f"{self.type}/{atlas_basename}/"
            
            # Do add prefix to skel, atlas and textures
            _add_prefix(self.skel, prefix)
            _add_prefix(self.atlas, prefix)
            for i in self.tex_list:
                for j in i:
                    _add_prefix(j, prefix)

        def save_spine(
            self,
            destdir: str,
            on_queued: Optional[Callable],
            on_saved: Optional[Callable],
        ):
            # Ensure the destination directory exists
            os.makedirs(destdir, exist_ok=True)
            
            # Get atlas dimensions if available
            atlas_width = None
            atlas_height = None
            if self.atlas_data and "pages" in self.atlas_data and len(self.atlas_data["pages"]) > 0:
                page = self.atlas_data["pages"][0]
                if "size" in page:
                    atlas_width, atlas_height = page["size"]
                    Logger.debug(f'ResolveAB: Atlas dimensions: {atlas_width}x{atlas_height}')
            
            for i in self.tex_list:
                if i[0]:
                    rgb = i[0].image
                    if i[1]:
                        rgba = AlphaRGBCombiner(i[1].image).combine_with(rgb)
                    else:
                        Logger.debug(
                            f'ResolveAB: Spine asset "{i[0].m_Name}" found with no Alpha texture.'
                        )
                        rgba = AlphaRGBCombiner.apply_premultiplied_alpha(rgb)
                    
                    # Resize the image if atlas dimensions are available
                    if atlas_width is not None and atlas_height is not None:
                        # Only resize if the dimensions don't match
                        if rgba.width != atlas_width or rgba.height != atlas_height:
                            Logger.debug(f'ResolveAB: Resizing image from {rgba.width}x{rgba.height} to {atlas_width}x{atlas_height}')
                            rgba = rgba.resize((atlas_width, atlas_height), Image.LANCZOS)
                    
                    # Create the directory for this specific file if needed
                    file_dir = osp.dirname(osp.join(destdir, i[0].m_Name))
                    os.makedirs(file_dir, exist_ok=True)
                    
                    if SafeSaver.save_image(
                        rgba,
                        destdir,
                        i[0].m_Name,
                        on_queued=on_queued,
                        on_saved=on_saved,
                    ):
                        Logger.debug(f'ResolveAB: Spine asset "{i[0].m_Name}" found.')
                else:
                    Logger.warn("ResolveAB: Spine asset RGB texture missing.")
            
            for i in (self.atlas, self.skel):
                # Create the directory for this specific file if needed
                file_dir = osp.dirname(osp.join(destdir, i.m_Name))
                os.makedirs(file_dir, exist_ok=True)
                
                SafeSaver.save_object(i, destdir, i.m_Name, on_queued, on_saved)
                Logger.debug(f'ResolveAB: Spine asset "{i.m_Name}" found.')

        # EndClass

    # EndClass


def ab_resolve(
    abfile: str,
    destdir: str,
    do_img: bool,
    do_txt: bool,
    do_aud: bool,
    do_spine: bool,
    on_processed: Optional[Callable] = None,
    on_file_queued: Optional[Callable] = None,
    on_file_saved: Optional[Callable] = None,
):
    """Extracts an AB file.

    :param abfile: Path to the AB file;
    :param destdir: Destination directory;
    :param do_img: Whether to extract images;
    :param do_txt: Whether to extract text scripts;
    :param do_aud: Whether to extract audios;
    :param do_spine: Whether to extract Spine assets, note that the Spine assets may have some identical file with the images/scripts;
    :param on_processed: Callback `f()` for finished, `None` for ignore;
    :param on_file_queued: Callback `f()` invoked when a file was queued, `None` for ignore;
    :param on_file_saved: Callback `f(file_path_or_none_for_not_saved)`, `None` for ignore;
    :rtype: None;
    """
    if not osp.isfile(abfile):
        if on_processed:
            on_processed()
        return
    try:
        res = Resource(UnityPy.load(abfile))
        Logger.debug(f'ResolveAB: "{res.name}" has {res.length} objects.')
        if res.length >= 10000:
            Logger.info(
                f'ResolveAB: Too many objects in file "{res.name}", unpacking it may take a long time.'
            )
        elif res.length == 0:
            Logger.info(f'ResolveAB: No object in file "{res.name}".')
        # Preprocess
        res.sort_skeletons()
        if do_spine:
            for i in res.spines:
                i.save_spine(destdir, on_file_queued, on_file_saved)
        if do_img:
            SafeSaver.save_objects(res.sprites, destdir, on_file_queued, on_file_saved)
            SafeSaver.save_objects(
                res.texture2ds, destdir, on_file_queued, on_file_saved
            )
        if do_txt:
            SafeSaver.save_objects(
                res.textassets, destdir, on_file_queued, on_file_saved
            )
        if do_aud:
            SafeSaver.save_objects(
                res.audioclips, destdir, on_file_queued, on_file_saved
            )
    except BaseException as arg:
        # Error feedback
        Logger.error(
            f'ResolveAB: Error occurred while unpacking file "{abfile}": Exception{type(arg)} {arg}'
        )
    if on_processed:
        on_processed()


########## Main-主程序 ##########
def main(
    src: str,
    destdir: str,
    do_del: bool = False,
    do_img: bool = True,
    do_txt: bool = True,
    do_aud: bool = True,
    do_spine: bool = False,
    separate: bool = True,
    resume: bool = False,
    skip_problematic: bool = False,
    timeout: int = 300,  # 5 minutes timeout per file
    target_list: Optional[str] = None,
):
    """Extract all the AB files from the given directory or extract a given AB file.

    :param src: Source directory or file;
    :param destdir: Destination directory;
    :param do_del: Whether to delete the existing files in the destination directory, `False` for default;
    :param do_img: Whether to extract images;
    :param do_txt: Whether to extract text scripts;
    :param do_aud: Whether to extract audios;
    :param do_spine: Whether to extract Spine assets, note that the Spine assets may have some identical file with the images/scripts;
    :param separate: Whether to sort the extracted files by their source AB file path.
    :param resume: Whether to resume from previous checkpoint.
    :param skip_problematic: Whether to skip files that are known to cause issues.
    :param timeout: Timeout in seconds for processing each file.
    :param target_list: Optional path to a JSON file containing a list of specific .ab files to process. If provided, checkpoint/resume logic is skipped.
    :rtype: None;
    """
    print("\nParsing paths...", s=1)
    Logger.info("ResolveAB: Retrieving file paths...")
    
    # Normalize paths and ensure they're relative
    src = os.path.normpath(src)
    destdir = os.path.normpath(destdir)
    
    using_target_list = False
    # Get the list of files to process
    if target_list and os.path.exists(target_list):
        using_target_list = True
        Logger.info(f"ResolveAB: Loading target file list from {target_list}")
        try:
            with open(target_list, 'r') as f:
                target_data = json.load(f)
                # Assuming the JSON contains a list under a key like "assets" or similar
                # Check common keys first
                if isinstance(target_data, dict):
                    possible_keys = ["lzham_missing_assets", "missing_assets", "lzham_assets", "assets", "files"]
                    flist = None
                    for key in possible_keys:
                        if key in target_data and isinstance(target_data[key], list):
                            flist = target_data[key]
                            Logger.info(f"ResolveAB: Loaded {len(flist)} files from key '{key}' in {target_list}")
                            break
                    if flist is None:
                         Logger.error(f"ResolveAB: Could not find a valid list of files in {target_list}. Looked for keys: {possible_keys}")
                         flist = []
                elif isinstance(target_data, list):
                    flist = target_data # Assume the JSON is just a list of paths
                    Logger.info(f"ResolveAB: Loaded {len(flist)} files directly from list in {target_list}")
                else:
                    Logger.error(f"ResolveAB: Invalid format in {target_list}. Expected a JSON list or dict with a list.")
                    flist = []
            # Validate paths in the list
            flist = [os.path.normpath(f) for f in flist if isinstance(f, str) and f.endswith('.ab')]
            valid_count = 0
            final_flist = []
            for f_path in flist:
                if os.path.exists(f_path):
                    final_flist.append(f_path)
                    valid_count += 1
                else:
                    # Try resolving relative to the source directory if it doesn't exist directly
                    potential_path = os.path.normpath(os.path.join(os.path.dirname(src) if os.path.isfile(src) else src, os.path.basename(f_path)))
                    if os.path.exists(potential_path):
                         final_flist.append(potential_path)
                         valid_count += 1
                    else:
                         Logger.warn(f"ResolveAB: File specified in target list not found: {f_path} (also checked {potential_path})")
            flist = final_flist
            Logger.info(f"ResolveAB: Processing {len(flist)} valid files from target list.")
        except Exception as e:
            Logger.error(f"ResolveAB: Failed to load or parse target list {target_list}: {e}")
            flist = []
    else:
        if target_list:
             Logger.warn(f"ResolveAB: Target list file not found: {target_list}. Falling back to scanning source directory.")
        flist = [src] if os.path.isfile(src) else get_filelist(src)
        flist = list(filter(is_ab_file, flist))

    # --- Checkpoint and Problematic File Handling (Skip if using target_list) ---
    processed_files = set()
    problematic_files = set()
    checkpoint_file = os.path.join(destdir, "extraction_checkpoint.json")
    problematic_files_path = os.path.join(destdir, "problematic_files.json")

    if not using_target_list:
        # Load checkpoint if resuming
        if resume and os.path.exists(checkpoint_file):
            try:
                with open(checkpoint_file, 'r') as f:
                    checkpoint_data = json.load(f)
                    processed_files = set(checkpoint_data.get('processed_files', []))
                    Logger.info(f"ResolveAB: Resuming from checkpoint with {len(processed_files)} previously processed files.")
            except Exception as e:
                Logger.warn(f"ResolveAB: Failed to load checkpoint: {e}")
        
        # Load problematic files list
        if os.path.exists(problematic_files_path):
            try:
                with open(problematic_files_path, 'r') as f:
                    problematic_data = json.load(f)
                    problematic_files = set(problematic_data.get('problematic_files', []))
                    Logger.info(f"ResolveAB: Found {len(problematic_files)} problematic files.")
            except Exception as e:
                Logger.warn(f"ResolveAB: Failed to load problematic files list: {e}")
        
        # Filter out already processed files and problematic files if needed
        original_count = len(flist)
        if resume:
            flist = [f for f in flist if f not in processed_files]
            Logger.info(f"ResolveAB: Skipping {original_count - len(flist)} already processed files based on checkpoint.")
        
        if skip_problematic and problematic_files:
            skipped_count = len(flist)
            flist = [f for f in flist if f not in problematic_files]
            skipped_count = skipped_count - len(flist)
            Logger.info(f"ResolveAB: Skipping {skipped_count} problematic files based on list.")
    else:
        if resume:
            Logger.warn("ResolveAB: --resume flag ignored when --target-list is used.")
        if skip_problematic:
             Logger.warn("ResolveAB: --skip-problematic flag ignored when --target-list is used.")
        Logger.info(f"ResolveAB: Processing {len(flist)} files specified in the target list.")
    # --- End Checkpoint Handling ---

    if do_del and not resume and not using_target_list: # Only delete if not resuming and not using target list
        print("\nCleaning...", s=1)
        rmdir(destdir)  # Danger zone
    SafeSaver.get_instance().reset_counter()
    thread_ctrl = ThreadCtrl()
    ui = UICtrl()
    tr_processed = TaskReporter(50, len(flist))
    tr_file_saving = TaskReporter(1)
    tracker = TaskReporterTracker(tr_processed, tr_file_saving)

    ui.reset()
    ui.loop_start()
    for i in flist:
        # (i stands for a file's path)
        ui.request(
            [
                "Batch unpacking...",
                tracker.to_progress_bar_str(),
                f"Current directory:\t{os.path.basename(os.path.dirname(i))}",
                f"Current file:\t{os.path.basename(i)}",
                f"Total unpacked:\t{tr_processed.to_progress_str()}",
                f"Total exported:\t{tr_file_saving.to_progress_str()}",
                f"Remaining time:\t{tracker.to_eta_str()}",
            ]
        )
        ###
        # Calculate the output directory path
        if os.path.isfile(src):
            # If source is a file, use the destination directory directly
            curdestdir = destdir
        else:
            # If source is a directory, calculate the relative path
            rel_path = os.path.relpath(i, src)
            # Get the directory part of the relative path
            rel_dir = os.path.dirname(rel_path)
            # Create the output directory path
            if separate:
                # If separate is True, create a subdirectory for each AB file
                ab_name = os.path.splitext(os.path.basename(i))[0]
                curdestdir = os.path.join(destdir, ab_name)
            else:
                # If separate is False, use the relative directory structure
                curdestdir = os.path.join(destdir, rel_dir)
        
        # Ensure the directory exists
        os.makedirs(curdestdir, exist_ok=True)
        
        # Save a checkpoint after each file is processed (ONLY if not using target_list)
        def on_file_processed():
            tr_processed.report()
            if not using_target_list:
                # Save to checkpoint
                processed_files.add(i)
                try:
                    with open(checkpoint_file, 'w') as f:
                        json.dump({'processed_files': list(processed_files)}, f)
                except Exception as e:
                    Logger.warn(f"ResolveAB: Failed to save checkpoint: {e}")
        
        # Create a thread with timeout
        thread = thread_ctrl.run_subthread(
            ab_resolve,
            (
                i,
                curdestdir,
                do_img,
                do_txt,
                do_aud,
                do_spine,
                on_file_processed,
                tr_file_saving.update_demand,
                tr_file_saving.report,
            ),
            name=f"RsThread:{id(i)}",
        )
        
        # Wait for the thread to complete with timeout
        start_time = time.time()
        while thread.is_alive() and time.time() - start_time < timeout:
            time.sleep(1)
            ui.refresh(post_delay=0.1)
        
        # If the thread is still running after timeout, mark it as problematic (ONLY if not using target_list)
        if thread.is_alive():
            Logger.warn(f"ResolveAB: File {i} timed out after {timeout} seconds.")
            if not using_target_list:
                Logger.info(f"ResolveAB: Marking {i} as problematic.")
                # Add to problematic files list
                problematic_files.add(i)
                try:
                    with open(problematic_files_path, 'w') as f:
                        json.dump({'problematic_files': list(problematic_files)}, f)
                except Exception as e:
                    Logger.warn(f"ResolveAB: Failed to save problematic files list: {e}")
            
            # Terminate the thread
            thread_ctrl.terminate_subthread(thread)
            
            # Report progress as if the file was processed
            tr_processed.report()
            
            # Save to checkpoint so we don't try this file again (ONLY if not using target_list)
            if not using_target_list:
                processed_files.add(i)
                try:
                    with open(checkpoint_file, 'w') as f:
                        json.dump({'processed_files': list(processed_files)}, f)
                except Exception as e:
                    Logger.warn(f"ResolveAB: Failed to save checkpoint: {e}")

    ui.reset()
    ui.loop_stop()
    while (
        thread_ctrl.count_subthread()
        or not SafeSaver.get_instance().completed()
        or tracker.get_progress() < 1
    ):
        ui.request(
            [
                "Batch unpacking...",
                tracker.to_progress_bar_str(),
                f"Total unpacked:\t{tr_processed.to_progress_str()}",
                f"Total exported:\t{tr_file_saving.to_progress_str()}",
                f"Remaining time:\t{tracker.to_eta_str()}",
            ]
        )
        ui.refresh(post_delay=0.1)

    ui.reset()
    print("\nBatch unpacking completed!", s=1)
    print(f"  Total unpacked {tr_processed.get_done()} files")
    print(f"  Total exported {tr_file_saving.get_done()} files")
    if problematic_files:
        print(f"  Skipped {len(problematic_files)} problematic files")
    print(f"  Time used {round(tracker.get_rt(), 1)} seconds")
