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
    log(f"Downloading file from {url}")
    file = requests.get(url)
    with open("/tmp/temp.zip", 'w') as f:
        f.write(file)

    # set up the source folders
    log(f"creating directory at {source_path}")
    os.mkdir(f"{source_path}")

    log(f"unpacking archive from /tmp/temp.zip to {source_path}")
    shutil.unpack_archive("/tmp/temp.zip", source_path)

    # create and run
    cmd = f"""
    cd {source_path};
    meson _build;
    ninja -C _build install;
    """

    log("running build script...")
    subprocess.run(cmd, shell=True)

    # clean up

    log("cleaning up...")
    
    shutil.rmtree(source_path)
    os.remove("/tmp/temp.zip")
    

if __name__ == "__main__":
    main()