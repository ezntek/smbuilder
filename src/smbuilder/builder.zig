const std = @import("std");
const types = @import("types.zig");
const rc = @import("n64comconvert");
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
        var a_base_dir: []u8 = undefined;
        if (std.fs.path.isAbsolute(base_dir)) {
            std.fs.makeDirAbsolute(base_dir) catch |err| switch (err) {
                error.PathAlreadyExists => {},
                else => return err,
            };

            a_base_dir = try alloc.dupe(u8, base_dir);
        } else {
            std.fs.cwd().makeDir(base_dir) catch |err| switch (err) {
                error.PathAlreadyExists => {},
                else => return err,
            };

            a_base_dir = std.fs.realpathAlloc(alloc, base_dir) catch |err|
                panic("failed to resolve realpath for base dir `{s}`: {any}", .{ base_dir, err });
        }

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

    fn needCloneRepo(self: *const Self) bool {
        const dir = self.joinPaths(&.{ self.base_dir, self.spec.repo.getName() });
        defer self.alloc.free(dir);
        return !dirExists(dir);
    }

    fn needCopyRom(self: *const Self) bool {
        const rom_path = self.joinPaths(&.{ self.base_dir, self.spec.repo.getName(), "baserom.us.z64" });
        defer self.alloc.free(rom_path);
        return !fileExists(rom_path);
    }

    fn needSetupBuildScript(self: *const Self) bool {
        const build_script_path = self.joinPaths(&.{ self.base_dir, "scripts", "build.sh" });
        defer self.alloc.free(build_script_path);
        return !fileExists(build_script_path);
    }

    fn needRunBuildScript(self: *const Self) bool {
        const region_pc = std.fmt.allocPrint(self.alloc, "{s}_pc", .{@tagName(self.spec.rom.region)}) catch |err| panic("allocation failed: {any}", .{err});
        const exe_name = std.fmt.allocPrint(self.alloc, "sm64.{s}.f3dex2e", .{@tagName(self.spec.rom.region)}) catch |err| panic("allocation failed: {any}", .{err});
        defer self.alloc.free(region_pc);
        defer self.alloc.free(exe_name);
        const exe_path = self.joinPaths(&.{ self.base_dir, self.spec.repo.getName(), "build", region_pc, exe_name });
        defer self.alloc.free(exe_path);
        return !fileExists(exe_path);
    }

    fn cloneRepo(self: *const Self) !void {
        // we need at least "git" "clone" <repo url> "--depth=1"
        var cmd = try std.ArrayListUnmanaged([]const u8).initCapacity(self.alloc, 4);

        try cmd.appendSlice(self.alloc, &.{ "git", "clone", self.spec.repo.getURL(), "--depth=1" });
        if (self.spec.repo.getBranch()) |branch| {
            try cmd.appendSlice(self.alloc, &.{ "--branch", branch });
        }

        const target_path = self.joinPaths(&.{ self.base_dir, self.spec.repo.getName() });
        try cmd.append(self.alloc, target_path);

        const argv = try cmd.toOwnedSlice(self.alloc);
        defer self.alloc.free(argv);

        var child = std.process.Child.init(argv, self.alloc);
        child.stdout_behavior = .Pipe;
        child.stderr_behavior = .Pipe;

        _ = try child.spawn();

        var poller = std.io.poll(self.alloc, enum { stdout, stderr }, .{
            .stdout = child.stdout.?,
            .stderr = child.stderr.?,
        });
        defer poller.deinit();

        const max_output_bytes = 16384;
        var stdout_buf: [max_output_bytes]u8 = undefined;
        var stderr_buf: [max_output_bytes]u8 = undefined;
        while (try poller.poll()) {
            if (poller.fifo(.stdout).count > max_output_bytes)
                @panic("");

            if (poller.fifo(.stderr).count > max_output_bytes)
                @panic("");

            _ = try poller.fifo(.stdout).reader().read(&stdout_buf);
            _ = try poller.fifo(.stderr).reader().read(&stderr_buf);

            // we now have the data in the buffer
        }

        _ = try child.wait();
    }

    fn copyRom(self: *const Self) !void {
        const target_path = self.joinPaths(&.{ self.base_dir, self.spec.repo.getName(), "baserom.us.z64" });
        defer self.alloc.free(target_path);

        const src_path = if (std.fs.path.isAbsolute(self.spec.rom.path))
            try self.alloc.dupe(u8, self.spec.rom.path)
        else
            try std.fs.realpathAlloc(self.alloc, self.spec.rom.path);

        defer self.alloc.free(src_path);

        const src_format = try rc.determineFormatFromPath(src_path);
        const target_format = .big_endian;

        if (src_format == target_format) {
            try std.fs.copyFileAbsolute(src_path, target_path, .{});
        } else {
            try rc.convertPaths(src_format, target_format, src_path, target_path);
        }
    }

    fn setupBuildScript(self: *const Self) !void {
        _ = self;
    }

    fn runBuildScript(self: *const Self) !void {
        _ = self;
    }

    pub fn build(self: *const Self) !void {
        if (self.needCloneRepo()) {
            try self.cloneRepo();
        }

        if (self.needCopyRom()) {
            try self.copyRom();
        }

        if (self.needSetupBuildScript()) {
            try self.setupBuildScript();
        }

        if (self.needRunBuildScript()) {
            try self.runBuildScript();
        }
    }
};
