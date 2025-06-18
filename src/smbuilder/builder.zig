const std = @import("std");
pub const types = @import("types.zig");

const panic = std.debug.panic;

pub const Builder = struct {
    alloc: std.mem.Allocator,
    spec: types.Spec,
    base_dir: []const u8,

    const Self = Builder;

    const BuildStep = enum {
        clone_repo,
        convert_rom,
        copy_rom,
        setup_build_script,
        setup_post_build_scripts,
        build,
        run_post_build_scripts,
    };

    /// Creates a new `Builder` object.
    /// This will duplicate the base_dir string with the allocator.
    pub fn init(alloc: std.mem.Allocator, spec: types.Spec, base_dir: []const u8) Self {
        const a_base_dir = alloc.dupe(base_dir) catch |err| panic("could not dupe string: {any}", .{err});
        return Self{
            .alloc = alloc,
            .spec = spec,
            .base_dir = a_base_dir,
        };
    }

    pub fn deinit(self: *const Self) void {
        self.alloc.free(self.base_dir);
    }

    fn getNeededBuildSteps(self: *const Self) BuildStep {
        _ = self;
    }
};
