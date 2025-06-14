## smbuilder

Remember how cool smlinux was? time to bring it back. This is a launcher for Super Mario 64 PC port for Linux, written in zig and qt.

### Project Structure

There will be:
* A central library that actually handles serializing/deserializing build specs, and carrying the build process out
* a CLI that takes a spec in a builds it
* a GUI written in qt and `libqt6zig` which acts as a launcher
