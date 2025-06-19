const std = @import("std");
const types = @import("types.zig");

const panic = std.debug.panic;

/// Builds a spec.
pub const Builder = struct {
    alloc: std.mem.Allocator,
    spec: types.Spec,
    base_dir: []const u8,

    const Self = Builder;

    const BuildStep = enum {
        clone_repo,
        copy_rom,
        setup_build_script,
        //setup_post_build_scripts,
        build,
        //run_post_build_scripts,
    };

    fn fileExists(path: []const u8) bool {
        var tmp = std.fs.openFileAbsolute(path, .{}) catch |err| switch (err) {
            error.FileNotFound => return false,
            else => panic("failed to open file `{s}`: {any}", .{ path, err }),
        };
        tmp.close();
        return true;
    }

    fn dirExists(path: []const u8) bool {
        var tmp = std.fs.openDirAbsolute(path, .{}) catch |err| switch (err) {
            error.FileNotFound => return false,
            else => panic("failed to open file `{s}`: {any}", .{ path, err }),
        };
        tmp.close();
        return true;
    }

    fn joinPaths(self: *const Self, paths: []const []const u8) []const u8 {
        return std.fs.path.join(self.alloc, paths) catch |err| panic("allocation failed: {any}", .{err});
    }

    /// Creates a new `Builder` object.
    /// This will duplicate the base_dir string with the allocator.
    /// Passing in a file for the base dir results in a panic.
    /// This struct expects the same allocator for both the spec and itself.
    pub fn init(alloc: std.mem.Allocator, spec: types.Spec, base_dir: []const u8) !Self {
        const a_base_dir = if (std.fs.path.isAbsolute(base_dir))
            try alloc.dupe(u8, base_dir)
        else
            std.fs.realpathAlloc(alloc, base_dir) catch |err| panic("failed to resolve realpath for base dir `{s}`: {any}", .{ base_dir, err });

        _ = std.fs.openDirAbsolute(a_base_dir, .{}) catch |err| switch (err) {
            error.NotDir => panic("expected a directory for base_dir, got a file", .{}),
            else => return err,
        }.close();

        return Self{
            .alloc = alloc,
            .spec = spec,
            .base_dir = a_base_dir,
        };
    }

    pub fn deinit(self: *const Self) void {
        self.spec.deinit(self.alloc);
    }

    fn checkCloneRepo(self: *const Self) bool {
        const dir = self.joinPaths(&.{ self.base_dir, self.spec.repo.getName() });
        defer self.alloc.free(dir);
        return !dirExists(dir);
    }

    fn checkCopyRom(self: *const Self) bool {
        const rom_path = self.joinPaths(&.{ self.base_dir, self.spec.repo.getName(), "baserom.us.z64" });
        defer self.alloc.free(rom_path);
        return !fileExists(rom_path);
    }

    fn checkSetupBuildScript(self: *const Self) bool {
        const build_script_path = self.joinPaths(&.{ self.base_dir, "scripts", "build.sh" });
        defer self.alloc.free(build_script_path);
        return !fileExists(build_script_path);
    }

    fn checkBuild(self: *const Self) bool {
        const region_pc = std.fmt.allocPrint(self.alloc, "{s}_pc", .{@tagName(self.spec.rom.region)}) catch |err| panic("allocation failed: {any}", .{err});
        const exe_name = std.fmt.allocPrint(self.alloc, "sm64.{s}.f3dex2e", .{@tagName(self.spec.rom.region)}) catch |err| panic("allocation failed: {any}", .{err});
        defer self.alloc.free(region_pc);
        defer self.alloc.free(exe_name);
        const exe_path = self.joinPaths(&.{ self.base_dir, self.spec.repo.getName(), "build", region_pc, exe_name });
        defer self.alloc.free(exe_path);
        return !fileExists(exe_path);
    }

    fn getNeededBuildSteps(self: *const Self) ![]BuildStep {
        var res = std.ArrayListUnmanaged(BuildStep).empty;

        if (self.checkCloneRepo()) {
            try res.append(self.alloc, .clone_repo);
        }

        if (self.checkCopyRom()) {
            try res.append(self.alloc, .copy_rom);
        }

        if (self.checkSetupBuildScript()) {
            try res.append(self.alloc, .setup_build_script);
        }

        if (self.checkBuild()) {
            try res.append(self.alloc, .build);
        }

        return try res.toOwnedSlice(self.alloc);
    }

    pub fn build(self: *const Self) !void {
        const steps = try self.getNeededBuildSteps();
        defer self.alloc.free(steps);

        for (steps) |step| {
            std.debug.print("got step: {any}\n", .{step});
        }
    }
};
