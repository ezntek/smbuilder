#!/usr/bin/env python3
#
# script to install dependencies that cargo/your system package manager cant install
#
#
# blueprint-compiler

import sys
import subprocess
import os
import shutil
import requests

def log(text: str):
    print(f"\033[1mInfo: \033[0m{text}")

def main():
    # set up some vars for convenience
    source_path = "/tmp/blueprint-compiler"
    url = "https://gitlab.gnome.org/jwestman/blueprint-compiler/-/archive/main/blueprint-compiler-main.zip"

    # get the latest zip from the repo
    try:
        log(f"Downloading file from {url}")
        file = requests.get(url, allow_redirects=True)
        with open("/tmp/temp.zip", 'wb') as f:
            f.write(file.content)

        # set up the source folders
        log(f"creating directory at {source_path}")
        os.mkdir(f"{source_path}")

        log(f"unpacking archive from /tmp/temp.zip to {source_path}")
        shutil.unpack_archive("/tmp/temp.zip", source_path)

        # create and run
        cmd = f"""
        cd {source_path}/blueprint-compiler-main;
        meson setup _build;
        ninja -C _build install;
        """

        log("running build script...")
        subprocess.run(cmd, shell=True)
    finally:
        return
        # clean up
        log("cleaning up...")
        shutil.rmtree(source_path)
        os.remove("/tmp/temp.zip")
    

if __name__ == "__main__":
    main()