## smbuilder

Remember how cool smlinux was? time to bring it back. This is a launcher for Super Mario 64 PC port for Linux, written in zig and qt.

### Project Structure

There will be:
* A central library that actually handles serializing/deserializing build specs, and carrying the build process out
* a CLI that takes a spec in a builds it
* a GUI written in qt and `libqt6zig` which acts as a launcher

### Notes

* This project is largely based upon the architecture of my previous attempt at writing a builder, `smbuilder-old` in Rust. You might see carbon-copies of structs and doc comments.
  * Hopefully there will be some more DOD (data-oriented design) patterns throughout the project
  * There will be less hippie code style stuff
* I am not responsible if you get into trouble with the Nintendo Police. Provide your own ROMs, please.
