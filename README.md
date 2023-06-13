# smbuilder

Remember how cool smlinux was? ***It's now time to bring it back.***

## What is smbuilder?
In short, smbuilder (stylized as all lowercase) is a rust crate that provides an interface for the compilation of various ports within the family of ports of Super Mario 64 to the PC. smbuilder should allow for many features to be supported so the frontend does not have to perform any extra logic for building, as all edge cases should be handled by the backend, which is what this crate is. It aims to support/provide:

* An interface for compiling the PC ports (running the C compiler, configuring options, etc.)
* DynOS Datapacks (for ports that support it such as sm64ex-coop and Render96ex)
* A build spec in yaml for reproducible builds
* the hard-disabling of DynOS packs so that these packs are not visible to the game

## Background

smbuilder (along with its sister tool `smbuilder-configurator` which is still in development) hopes to be the spiritual successor to the now-dead `enigma9o7/smlinux` (one may interpret it as a linux version of the propietary, windows-only [sm64pcbuilder2](https://sm64pc.info/sm64pcbuilder2/)), which was (/is respectively) an insanely convenient build script/launcher for all the SM64 Ports to the PC, including sm64ex, Render96ex, sm64ex-coop, among others. smlinux died for whatever reason, and now there is no true build script for the PC Ports for Unix-like systems, such as Linux, macOS and the BSD family. I did try to make one before, [`ezntek/SM64LinuxLauncher-qt`](https://github.com/ezntek/SM64LinuxLauncher-qt), but that just faded into obscurity as this thing called School hit me. But now, I decided to create a spiritual successor to the project in Rust.

## A note on development

I'm a student, and I have this thing called school to worry about, so development may as well not move for weeks at a time. Until Late June, 2023, I cannot guarantee fast development.

**Oh, and this project may as well just be a project to help me sharpen my rust+pyo3 skills. I'll try to make this production-quality, but no promises.**

## Project quality

`[█████▌...] 6.5/10` looking good so far i guess, but untested 

## License

This project is licensed under the Apache 2.0 License. You can view the full license in the `LICENSE.md` file in the root of the project, but in summary:

You can:

 * Use the code for anything you want
 * View the source code withuot restrictions
 * Modify the source code to your liking
 * Distribute modified code
 * Patent the modified source code

But under these conditions:

 * One must **clearly** state the changes made to the original code, if they chose to modify it for their own project
 * If one decides to incorporate any part of this program in theirs without modification, they must license it under the Apache License too.
 * No warranty, this software is distributed AS-IS

Note that the summary may not be fully accurate, this is a breakdown for lazy readers, but only the full terms and conditions, not this summary applies to this project.

## What am I working on?

* Writing a GUI in gtk4+libadwaita+blueprint

## To-Do list

* Write a module for this in pyo3
* Write stubs for pyo3