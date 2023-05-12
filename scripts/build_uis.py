#!/usr/bin/env python3

import sys, os, subprocess

__bold__ = "\033[1m"
__end__ = "\033[0m"

if __name__ == "__main__":
    try:
        dirname, target_dir_name = sys.argv[1], sys.argv[2]
    except IndexError:
        print(f"{__bold__}Usage:{__end__} /path/to/script source_dir dest_dir")
        exit(1)
    
    if not os.path.exists("ui_xml"):
        os.mkdir("ui_xml")

    for file in os.listdir(dirname):
        name, ext = os.path.splitext(file)
        if ext in [".blueprint", ".blp"]:
            print(f"{__bold__}Building:{__end__} {os.path.join(dirname, f'{name}{ext}')} {__bold__}->{__end__} {os.path.join(target_dir_name, f'{name}.ui')}")
            subprocess.run(
                ["blueprint-compiler", "compile", "--output", os.path.join(target_dir_name, f"{name}.ui"), os.path.join(dirname, f"{name}{ext}")]
            )