# smbuilder

Remember how cool smlinux was? ***It's now time to bring it back.***

## What is smbuilder?
In short, smbuilder (stylized as all lowercase) is a rust crate that provides an interface for the compilation of various ports within the family of ports of Super Mario 64 to the PC. smbuilder should allow for many features to be supported so the frontend does not have to perform any extra logic for building, as all edge cases should be handled by the backend, which is what this crate is. It aims to support/provide:

* An interface for compiling the PC ports (running the C compiler, configuring options, etc.)
* DynOS Datapacks (for ports that support it such as sm64ex-coop and Render96ex)
* A build spec in yaml for reproducible builds
* the hard-disabling of DynOS packs so that these packs are not visible to the game

## License

This project is licensed under the Apache 2.0 License. You can view the full license in the `LICENSE.md` file in the root of the project.

## To-Dos

[TODO.md](/TODO.md)