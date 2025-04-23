# -*- coding: utf-8 -*-
# Copyright (c) 2022-2025, Harry Huang
# @ BSD 3-Clause License
import time
import argparse
import os
import os.path as osp
import sys

from src.utils import ArgParser
from src.utils.Config import Config
from src.utils.Logger import Logger
from src.utils.GlobalMethods import color, input, print, clear, title, stacktrace, rmdir
from src import ResolveAB as AU_Rs
from src import DecodeTextAsset as AU_Fb
from src import CombineRGBwithA as AU_Cb
from src import CollectModels as AU_Cm
from src import CollectVoice as AU_Cv
from src import ModelsDataDist as AU_Mdd
from src import VoiceDataDist as AU_Vdd

ARKUNPACKER_VERSION = "v4.0"
ARKUNPACKER_LOCAL = "zh-CN"


def prt_homepage():
    Logger.info("CI: In Homepage.")
    clear()
    os.chdir(".")
    print(f"Welcome to ArkUnpacker {ARKUNPACKER_VERSION}", s=1)
    print("=" * 20)
    print(
        """Mode Selection:
1: Quick Start
2: Custom Resource Unpacking
3: Custom Image Combining
4: Custom TextAsset Decoding
5: ArkModels Extraction & Sorting Tool
6: ArkVoice Extraction & Sorting Tool
0: Exit""",
        c=6,
    )
    print(
        "Enter the number and press Enter. \nIf you are not familiar with these functions, it is strongly recommended to read the manual (README) first:\nhttps://github.com/isHarryh/Ark-Unpacker "
    )


def prt_subtitle(msg: str):
    clear()
    os.chdir(".")
    print("=" * 10, s=1)
    print(msg, s=1)
    print("=" * 10, s=1)


def prt_continue():
    UserInput.request("\n> Press Enter to continue...")


def run_quickaccess():
    Logger.info("CI: Run quick access.")
    title("ArkUnpacker - Processing")
    destdir = f"Unpacked_{int(time.time())}"
    ###
    prt_subtitle("Step 1|Resource Unpacking")
    time.sleep(1)
    AU_Rs.main(".", destdir)
    ###
    prt_subtitle("Step 2|Merging Images")
    time.sleep(1)
    AU_Cb.main(destdir, f"Combined_{int(time.time())}")


def run_custom_resolve_ab():
    Logger.info("CI: Customized unpack mode.")
    prt_subtitle("Custom Resource Unpacking")
    ###
    print("\nPlease enter the directory or file path to unpack")
    src = UserInput.request_path()
    # Normalize the source path and ensure it's relative
    src = os.path.normpath(src)
    if os.path.isabs(src):
        # If absolute path is provided, make it relative to current directory
        src = os.path.relpath(src)
    print("Unpacking target path: ", c=2)
    print(f"  {src}", c=6)
    ###
    print("\nPlease enter the path to the export directory")
    print("   Support relative paths, leave blank to automatically create")
    destdir = input("> ", c=2)
    if not destdir:
        destdir = f"Unpacked_{int(time.time())}"
    # Normalize the destination path and ensure it's relative
    destdir = os.path.normpath(destdir)
    if os.path.isabs(destdir):
        # If absolute path is provided, make it relative to current directory
        destdir = os.path.relpath(destdir)
    print("Export directory path: ", c=2)
    print(f"  {destdir}", c=6)
    ###
    do_del = False
    if os.path.isdir(destdir):
        print("\nThis export directory already exists, do you want to delete all files in it?")
        print("   Please! Be careful! Choose: [y]Delete, [n]Keep (default)", c=3)
        do_del = UserInput.request_yes_or_no(False)
    
    # If the directory exists and we're not deleting it, ask if user wants to resume
    resume = False
    if os.path.isdir(destdir) and not do_del:
        print("\nDo you want to continue unpacking from the last interrupted position?")
        print("  [y]Yes, [n]No (default)", c=3)
        resume = UserInput.request_yes_or_no(False)
        
    # Ask about skipping problematic files
    skip_problematic = False
    problematic_files_path = os.path.join(destdir, "problematic_files.json")
    if os.path.exists(problematic_files_path):
        print("\nDo you want to skip files that caused problems previously?")
        print("  [y]Yes (default), [n]No", c=3)
        skip_problematic = UserInput.request_yes_or_no(True)
        
    # Ask about timeout
    timeout = 300  # Default 5 minutes
    print("\nSet timeout for processing each file (in seconds, default: 300)")
    print("  Files that take longer than this will be marked as problematic")
    timeout_input = input("> ", c=2)
    if timeout_input.strip() and timeout_input.isdigit():
        timeout = int(timeout_input)
    print(f"  Timeout set to {timeout} seconds")
    ###
    separate = True
    if not os.path.isfile(src):
        print("\nDo you want to group exported files by source?")
        print("  [y]Yes (default), [n]No", c=3)
        separate = UserInput.request_yes_or_no(True)
    ###
    print("\nPlease enter the resource type to export")
    print("  [i]Image, [t]Text, [a]Audio", c=3)
    print("  [s]Spine Animation Model", c=3)
    print('   Example input: "ita", "ia"')
    do_them = input("> ", c=2).lower()
    do_img = True if "i" in do_them else False
    do_txt = True if "t" in do_them else False
    do_aud = True if "a" in do_them else False
    do_spi = True if "s" in do_them else False
    print(
        f"  [{'√' if do_img else '×'}]Image, [{'√' if do_txt else '×'}]Text, [{'√' if do_aud else '×'}]Audio",
        c=6,
    )
    print(f"  [{'√' if do_spi else '×'}]Spine Animation Model", c=6)
    ###
    prt_continue()
    title("ArkUnpacker - Processing")
    AU_Rs.main(src, destdir, do_del, do_img, do_txt, do_aud, do_spi, separate, resume, skip_problematic, timeout, args.target_list)


def run_custom_combine_image():
    Logger.info("CI: Customized image combine mode.")
    prt_subtitle("Custom Image Combining")
    ###
    print("\nPlease enter the path to the source image directory")
    rootdir = UserInput.request_path()
    print("Source image directory path:")
    print(f"  {osp.abspath(rootdir)}", c=6)
    ###
    print("\nPlease enter the destination")
    print("   Support relative paths, leave blank to automatically create")
    destdir = input("> ", c=2)
    if not destdir:
        destdir = f"Combined_{int(time.time())}"
    print("You selected export directory:")
    print(f"  {osp.abspath(destdir)}", c=6)
    ###
    do_del = False
    if osp.isdir(destdir):
        print("\nThis export directory already exists, do you want to delete all files in it?")
        print("   Please! Be careful! Choose: [y]Delete, [n]Keep (default)", c=3)
        do_del = UserInput.request_yes_or_no(False)
    ###
    prt_continue()
    title("ArkUnpacker - Processing")
    AU_Cb.main(rootdir, destdir, do_del)


def run_custom_textasset_decode():
    Logger.info("CI: Customized textasset decoding mode.")
    prt_subtitle("Custom TextAsset Decoding")
    ###
    print(
        "Arknights game data files are mainly located in TextAsset, stored in FlatBuffers format or AES encrypted."
    )
    print("After unpacking the resources, you need to decode these files to get game data.")
    print("\nPlease enter the path to the source file directory")
    print("If you are not familiar with which files are TextAsset, please select the entire unpacked directory.")
    rootdir = UserInput.request_path()
    print("  Source file directory:")
    print(f"  {osp.abspath(rootdir)}", c=6)
    ###
    print("\nPlease enter the destination")
    print("   Support relative paths, leave blank to automatically create")
    destdir = input("> ", c=2)
    if not destdir:
        destdir = f"Decoded_{int(time.time())}"
    print("You selected export directory:")
    print(f"  {osp.abspath(destdir)}", c=6)
    ###
    do_del = False
    if osp.isdir(destdir):
        print("\nThis export directory already exists, do you want to delete all files in it?")
        print("   Please! Be careful! Choose: [y]Delete, [n]Keep (default)", c=3)
        do_del = UserInput.request_yes_or_no(False)
    ###
    prt_continue()
    title("ArkUnpacker - Processing")
    AU_Fb.main(rootdir, destdir, do_del)


def run_arkmodels_unpacking(dirs, destdir):
    Logger.info("CI: ArkModels unpack mode.")
    prt_subtitle("ArkModels Model Extraction")
    ###
    for i in dirs:
        if not osp.exists(i):
            print(
                f"Cannot find {i} in working directory, please ensure the folder is directly under the working directory. Also, it may be because the program version is incompatible with your resource version, you can try to get other version program.",
                c=3,
            )
            return
    title("ArkUnpacker - Processing")
    ###
    print("Cleaning up...")
    rmdir(destdir)
    for i in dirs:
        AU_Rs.main(i, destdir, do_img=False, do_txt=False, do_aud=False, do_spine=True)


def run_arkmodels_anon_unpacking(dirs, destdir):
    Logger.info("CI: ArkModels unpack mode.")
    prt_subtitle("ArkModels Model Extraction")
    ###
    for i in dirs:
        if not osp.exists(i):
            print(
                f"Cannot find {i} in working directory, please ensure the folder is directly under the working directory. Also, it may be because the program version is incompatible with your resource version, you can try to get other version program.",
                c=3,
            )
            return
    title("ArkUnpacker - Processing")
    ###
    print("Cleaning up...")
    rmdir(destdir)
    for i in dirs:
        AU_Rs.main(i, destdir, do_img=False, do_txt=True, do_aud=False, do_spine=False)


def run_arkmodels_filtering(dirs, destdirs):
    Logger.info("CI: ArkModels file filter mode.")
    prt_subtitle("ArkModels File Sorting")
    ###
    dirs_ = []
    destdirs_ = []
    for i, j in zip(dirs, destdirs):
        if not osp.exists(i):
            print(
                f'Cannot find {i} in working directory, please ensure the folder is directly under the working directory. Also, it may be because you did not perform the "model extraction" step beforehand.',
                c=3,
            )
            UserInput.request('> Enter symbol "*" to cancel task, or press Enter to force continue')
        else:
            dirs_.append(i)
            destdirs_.append(j)
    ###
    AU_Cm.main(dirs_, destdirs_)


def run_arkmodels_data_dist():
    Logger.info("CI: ArkModels dataset mode.")
    prt_subtitle("ArkModels Generate Dataset")
    ###
    for i in ["models", "models_enemies", "models_illust"]:
        if not osp.exists(i):
            print(f'Cannot find {i} in working directory, please confirm that you have run the "model sorting" beforehand.', c=3)
            UserInput.request('> Enter symbol "*" to cancel task, or press Enter to force continue')
            return
    if not osp.exists(AU_Mdd.ModelsDist.TEMP_DIR):
        print(
            f'Cannot find {AU_Mdd.ModelsDist.TEMP_DIR} in working directory, please confirm that you have run the "anonymous data extraction" beforehand.',
            c=3,
        )
        UserInput.request('> Enter symbol "*" to cancel task, or press Enter to force continue')
        return
    AU_Mdd.main()


def run_arkmodels_workflow():
    def visual(fp: str, default_c: int = 6):
        return f"{color(2 if osp.exists(fp) else 3)}{fp}{color(default_c)}"

    Logger.info("CI: In ArkModels workflow.")

    def prt_arkmodels_menu():
        clear()
        os.chdir(".")
        print("ArkModels Extraction & Sorting Tool", s=1)
        print("=" * 20)
        print(
            """ArkModels is the Spine model warehouse established by the author (https://github.com/isHarryh/Ark-Models), the following functions are specially designed for the update of ArkModels warehouse.
Before running some functions, please ensure that the resource folder indicated in the parentheses is located in the directory where the program is located."""
        )
        print(
            f"""Function Selection:
1: Quick Start
2: Operator Construction Model Extraction ({visual('chararts')}, {visual('skinpack')})
3: Enemy Battle Model Extraction ({visual('battle')})
4: Dynamic Portrait Model Extraction ({visual('arts')})
5: Anonymous Data Extraction ({visual(AU_Mdd.ModelsDist.GAMEDATA_DIR)})
6: Model Sorting
7: Generate Dataset
0: Return""",
            c=6,
        )
        print(
            "Enter the number and press Enter. \nIf necessary, please read the manual (README): \nhttps://github.com/isHarryh/Ark-Unpacker"
        )

    temp_dir_1 = "temp/am_upk_operator"
    temp_dir_2 = "temp/am_upk_enemy"
    temp_dir_3 = "temp/am_upk_dynillust"
    temp_dir_4 = AU_Mdd.ModelsDist.TEMP_DIR
    while True:
        title("ArkUnpacker")
        prt_arkmodels_menu()
        order = input("> ", c=2)
        wildcard = False
        if order == "1":
            wildcard = True
        if order == "2" or wildcard:
            run_arkmodels_unpacking(["chararts", "skinpack"], temp_dir_1)
        if order == "3" or wildcard:
            run_arkmodels_unpacking(["battle/prefabs/enemies"], temp_dir_2)
        if order == "4" or wildcard:
            run_arkmodels_unpacking(["arts/dynchars"], temp_dir_3)
        if order == "5" or wildcard:
            run_arkmodels_anon_unpacking([AU_Mdd.ModelsDist.GAMEDATA_DIR], temp_dir_4)
        if order == "6" or wildcard:
            run_arkmodels_filtering(
                [temp_dir_1, temp_dir_2, temp_dir_3],
                ["models", "models_enemies", "models_illust"],
            )
        if order == "7" or wildcard:
            run_arkmodels_data_dist()
        if order in ["1", "2", "3", "4", "5", "6", "7"]:
            prt_continue()
        if order == "0":
            return


def run_arkvoice_unpacking(dir, destdir1, destdir2, wildcard=False):
    def visual(fp: str, default_c: int = 6):
        return f"{color(2 if osp.exists(fp) else 3)}{fp}{color(default_c)}"

    Logger.info("CI: ArkVoice unpack mode.")

    def prt_arkvoice_unpacking_menu(dir, destdir1):
        clear()
        os.chdir(".")
        print("ArkVoice Extraction & Sorting Tool", s=1)
        print("=" * 20)
        print(
            f"""Mode Selection:
1: Extract Only Wav Files ({visual(dir)})
2: Extract and Merge Wav Files to Ogg Files ({visual(destdir1)})
3: Extract and Merge
0: Cancel""",
            c=6,
        )

    while True:
        title("ArkUnpacker")
        prt_arkvoice_unpacking_menu(dir, destdir1)
        order = "0"
        if not wildcard:
            order = input("> ", c=2)
        if order == "3":
            wildcard = True
        if order == "1" or wildcard:
            if not osp.exists(dir):
                print(
                    f"Cannot find {dir} in working directory, please ensure the folder is directly under the working directory.", c=3
                )
                return
            print("Cleaning up...")
            rmdir(destdir1)
            title("ArkUnpacker - Processing")
            AU_Rs.main(
                dir, destdir1, do_img=False, do_txt=False, do_aud=True, do_spine=False
            )
        if order == "2" or wildcard:
            if not osp.exists(destdir1):
                print(f"Cannot find {destdir1} in working directory, please ensure you have performed the preceding steps.", c=3)
                return
            print("Cleaning up...")
            rmdir(destdir2)
            title("ArkUnpacker - Processing")
            AU_Cv.main(destdir1, destdir2, "custom" in destdir2)
        if order == "0":
            return


def run_arkvoice_data_dist():
    Logger.info("CI: ArkVoice dataset mode.")
    prt_subtitle("ArkVoice Generate Dataset")
    ###
    AU_Vdd.main()


def run_arkvoice_workflow():
    Logger.info("CI: In ArkVoice workflow.")

    def prt_arkvoice_menu():
        clear()
        os.chdir(".")
        print("ArkVoice Extraction & Sorting Tool", s=1)
        print("=" * 20)
        print(
            "ArkVoice is the voice warehouse established by the author (https://github.com/isHarryh/Ark-Voice), the following functions are specially designed for the update of ArkVoice warehouse."
        )
        print(
            f"""Function Selection:
1: Quick Start
2: Extract and Sort Japanese Voice
3: Extract and Sort Chinese Voice
4: Extract and Sort English Voice
5: Extract and Sort Korean Voice
6: Extract and Sort Personal Voice
7: Generate Dataset
0: Return""",
            c=6,
        )
        print(
            "Enter the number and press Enter. \nIf necessary, please read the manual (README): \nhttps://github.com/isHarryh/Ark-Unpacker"
        )

    while True:
        title("ArkUnpacker")
        prt_arkvoice_menu()
        order = input("> ", c=2)
        wildcard = False
        if order == "1":
            wildcard = True
        if order == "2" or wildcard:
            run_arkvoice_unpacking(
                "audio/sound_beta_2/voice", "temp/av_upk", "voice", wildcard
            )
        if order == "3" or wildcard:
            run_arkvoice_unpacking(
                "audio/sound_beta_2/voice_cn", "temp/av_upk_cn", "voice_cn", wildcard
            )
        if order == "4" or wildcard:
            run_arkvoice_unpacking(
                "audio/sound_beta_2/voice_en", "temp/av_upk_en", "voice_en", wildcard
            )
        if order == "5" or wildcard:
            run_arkvoice_unpacking(
                "audio/sound_beta_2/voice_kr", "temp/av_upk_kr", "voice_kr", wildcard
            )
        if order == "6" or wildcard:
            run_arkvoice_unpacking(
                "audio/sound_beta_2/voice_custom",
                "temp/av_upk_custom",
                "voice_custom",
                wildcard,
            )
        if order == "7" or wildcard:
            run_arkvoice_data_dist()
        if order in ["1", "2", "3", "4", "5", "6", "7"]:
            prt_continue()
        if order == "0":
            return


def validate_input_output_arg(
    parser: argparse.ArgumentParser,
    args: argparse.Namespace,
    allow_file_input: bool = False,
):
    if not getattr(args, "input", None):
        parser.error("input should be defined in this mode")
    if not getattr(args, "output", None):
        parser.error("output should be defined in this mode")
    if not allow_file_input and os.path.isfile(args.input):
        parser.error("input should be a directory, not file")
    if not os.path.isdir(args.input) and not (
        allow_file_input and os.path.isfile(args.input)
    ):
        parser.error(
            f"input should be a {'file or ' if allow_file_input else ''}directory that exists"
        )


def validate_logging_level_arg(
    parer: argparse.ArgumentParser, args: argparse.Namespace
):
    if getattr(args, "logging_level", None) is None:
        return
    if args.logging_level not in range(5):
        parser.error("invalid logging level")
    Logger.set_level(args.logging_level)


class UserInput:
    CANCEL_CMD = "*"

    @staticmethod
    def request(prompt: str = "> "):
        uin = input(prompt, c=2)
        if uin == UserInput.CANCEL_CMD:
            print("   Task cancelled", c=3)
            raise InterruptedError("User cancelled")
        return uin

    @staticmethod
    def request_options(options: list):
        print(f'   Enter symbol "{UserInput.CANCEL_CMD}" to cancel task')
        uin = UserInput.request()
        while uin not in options:
            print("   Invalid option input", c=3)
            uin = UserInput.request()
        return uin

    @staticmethod
    def request_path():
        print(f'   Enter symbol "{UserInput.CANCEL_CMD}" to cancel task, support input relative path')
        uin = osp.normpath(UserInput.request())
        while not osp.exists(uin):
            print("   Input path does not exist", c=3)
            uin = osp.normpath(UserInput.request())
        return uin

    @staticmethod
    def request_yes_or_no(default: bool):
        print(f'   Enter symbol "{UserInput.CANCEL_CMD}" to cancel task')
        uin = UserInput.request().strip().lower()
        if default:
            return False if uin == "n" else True
        else:
            return True if uin == "y" else False


if __name__ == "__main__":
    parser = ArgParser.INSTANCE
    try:
        Logger.set_instance(Config.get("log_file"), Config.get("log_level"))
        Logger.info("CI: Initialized")
        print("")
        args = parser.parse_args()
        if getattr(args, "mode", None) is None:
            # No argument input -> ENTER -> Interactive CLI mode
            while True:
                try:
                    title("ArkUnpacker")
                    prt_homepage()
                    order = input("> ", c=2)
                    if order == "1":
                        run_quickaccess()
                        prt_continue()
                    elif order == "2":
                        run_custom_resolve_ab()
                        prt_continue()
                    elif order == "3":
                        run_custom_combine_image()
                        prt_continue()
                    elif order == "4":
                        run_custom_textasset_decode()
                        prt_continue()
                    elif order == "5":
                        run_arkmodels_workflow()
                    elif order == "6":
                        run_arkvoice_workflow()
                    elif order == "0":
                        print("\nUser exit")
                        break
                except InterruptedError as arg:
                    Logger.warn("CI: Program was slightly interrupted by user.")
                    print("\n[InterruptedError] User lightly interrupted", c=3)
        else:
            # Has arguments input -> GOTO -> The specified mode
            validate_logging_level_arg(parser, args)
            if args.mode == "ab":
                validate_input_output_arg(parser, args, allow_file_input=True)
                AU_Rs.main(
                    args.input,
                    args.output,
                    args.d,
                    args.image,
                    args.text,
                    args.audio,
                    args.spine,
                    args.group,
                    args.resume,
                    args.skip_problematic,
                    args.timeout,
                    args.target_list,
                )
            elif args.mode == "cb":
                validate_input_output_arg(parser, args)
                AU_Cb.main(args.input, args.output, args.d)
            elif args.mode == "fb":
                validate_input_output_arg(parser, args)
                AU_Fb.main(args.input, args.output, args.d)
    # Global error handlers
    except SystemExit as arg:
        Logger.info(f"CI: Program received explicit exit code {arg.code}")
        print("\n[SystemExit] Explicitly exit program", c=3)
        sys.exit(arg.code)
    except KeyboardInterrupt as arg:
        Logger.error("CI: Program was forcibly interrupted by user.")
        print("\n[KeyboardInterrupt] User forced interrupted", c=1, s=7)
        print(stacktrace(), c=3)
        sys.exit(1)
    except ArgParser.ArgParserFailure as arg:
        Logger.error(f"CI: Program failed ti parse input arguments, {arg}")
        print(parser.format_usage())
        print("[ArgParserFailure] Command line parameter parsing failed", c=1, s=7)
        print(f"{parser.prog} failed to parse arguments", c=1)
        print(arg, c=3)
        sys.exit(2)
    except BaseException as arg:
        Logger.error(f"CI: Oops! Unexpected error occurred: {stacktrace()}")
        print(f"\n[{type(arg).__name__}] Unexpected exception occurred", c=1, s=7)
        print(stacktrace(), c=3)
        input("> Press Enter to exit...", c=1)
        sys.exit(1)
    sys.exit(0)
