# smbuilder -- API documentation

smbuilder is a small rust crate that provides an interface for the compilation of various ports within the family of ports of Super Mario 64 to the PC.

The crate:
 * Provides strong types and models that wraps around the resources needed and other "moving parts" (texture packs, etc) for a given port
 * Allows for the types to be serialized/deserialized into/from yaml for reproducible build specifications (specs)
 * Provides classes and traits to provide an interface to build specs.

## Notes

* This crate is not quite production quality yet. breaking API changes may come sooner or later.
* The bindings of these resources/moving parts for a build cannot be fully complete without actually modifying the port. However, more bindings may be added later.
* The choice of repositories and the makeopts supported by those ports should be handled by the app that uses this crate. However, makeopts may be implemented as enums/structs in a later version or in another crate. 

## Usage

WIP