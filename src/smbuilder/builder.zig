const std = @import("std");
const builtin = @import("builtin");
const types = @import("types.zig");
const rc = @import("n64comconvert");
const panic = std.debug.panic;

/// Builds a spec.
pub const Builder = struct {
    alloc: std.mem.Allocator,
    spec: types.Spec,
    base_dir: []const u8,
    repo_dir: []const u8,
    scripts_dir: []const u8,

    const Self = Builder;

    const BuildStep = enum {
        clone_repo,
        copy_rom,
        create_scripts_dir,
        setup_build_script,
        //setup_post_build_scripts,
        build,
        //run_post_build_scripts,
    };

    fn fileExists(path: []const u8) bool {
        var tmp = std.fs.cwd().openFile(path, .{}) catch |err| switch (err) {
            error.FileNotFound => return false,
            else => panic("failed to open file `{s}`: {any}", .{ path, err }),
        };
        tmp.close();
        return true;
    }

    fn dirExists(path: []const u8) bool {
        var tmp = std.fs.cwd().openDir(path, .{}) catch |err| switch (err) {
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
        std.fs.cwd().makeDir(base_dir) catch |err| switch (err) {
            error.PathAlreadyExists => {},
            else => return err,
        };

        _ = std.fs.cwd().openDir(base_dir, .{}) catch |err| switch (err) {
            error.NotDir => panic("expected a directory for base_dir, got a file", .{}),
            else => return err,
        }.close();

        const repo_dir = try std.fs.path.join(alloc, &.{ base_dir, spec.repo.getName() });
        const scripts_dir = try std.fs.path.join(alloc, &.{ base_dir, "scripts" });

        return Self{
            .alloc = alloc,
            .spec = spec,
            .base_dir = base_dir,
            .repo_dir = repo_dir,
            .scripts_dir = scripts_dir,
        };
    }

    pub fn deinit(self: *const Self) void {
        self.spec.deinit(self.alloc);
        self.alloc.free(self.repo_dir);
    }

    fn needCloneRepo(self: *const Self) bool {
        return !dirExists(self.repo_dir);
    }

    fn needCopyRom(self: *const Self) bool {
        const rom_path = self.joinPaths(&.{ self.repo_dir, "baserom.us.z64" });
        defer self.alloc.free(rom_path);
        return !fileExists(rom_path);
    }

    fn needCreateScriptsDir(self: *const Self) bool {
        return !dirExists(self.scripts_dir);
    }

    fn needSetupBuildScript(self: *const Self) bool {
        const build_script_path = self.joinPaths(&.{ self.scripts_dir, "build.sh" });
        defer self.alloc.free(build_script_path);
        return !fileExists(build_script_path);
    }

    fn getExePath(self: *const Self) []const u8 {
        const region_pc = std.fmt.allocPrint(self.alloc, "{s}_pc", .{@tagName(self.spec.rom.region)}) catch |err| panic("allocation failed: {any}", .{err});
        const exe_name = std.fmt.allocPrint(self.alloc, "sm64.{s}.f3dex2e", .{@tagName(self.spec.rom.region)}) catch |err| panic("allocation failed: {any}", .{err});
        defer self.alloc.free(region_pc);
        defer self.alloc.free(exe_name);
        return self.joinPaths(&.{ self.repo_dir, "build", region_pc, exe_name });
    }

    fn needRunBuildScript(self: *const Self) bool {
        const exe_path = self.getExePath();
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

        try cmd.append(self.alloc, self.repo_dir);

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
            @memset(&stdout_buf, 0);
            @memset(&stderr_buf, 0);
        }

        _ = try child.wait();
    }

    fn copyRom(self: *const Self) !void {
        const target_path = self.joinPaths(&.{ self.repo_dir, "baserom.us.z64" });
        defer self.alloc.free(target_path);

        const src_path = self.spec.rom.path;

        const src_format = try rc.determineFormatFromPath(src_path);
        const target_format = .big_endian;

        if (src_format == target_format) {
            try std.fs.cwd().copyFile(src_path, std.fs.cwd(), target_path, .{});
        } else {
            try rc.convertPaths(src_format, target_format, src_path, target_path);
        }
    }

    fn createScriptsDir(self: *const Self) !void {
        try std.fs.cwd().makeDir(self.scripts_dir);
    }

    fn generateBuildScript(self: *const Self) ![]const u8 {
        const header =
            \\#!/usr/bin/env sh
            \\
            \\# Script generated by smbuilder.
            \\# DO NOT EDIT THIS SCRIPT. YOUR CHANGES WILL BE DELETED UPON REBUILDING.
            \\
        ;

        // we do a lot of constant appends, so fix the capacity first
        var cmd = try std.ArrayListUnmanaged(u8).initCapacity(self.alloc, header.len + 32);
        defer cmd.deinit(self.alloc);
        try cmd.appendSlice(self.alloc, header);

        // setup build cmd
        if (builtin.target.os.tag.isDarwin() or builtin.target.os.tag.isBSD()) {
            try cmd.append(self.alloc, 'g');
        }
        var buf: [256]u8 = undefined;
        const res = try std.fmt.bufPrint(&buf, "make -j{} -C ", .{self.spec.jobs});
        const abs_repo_dir = try std.fs.realpathAlloc(self.alloc, self.repo_dir);
        defer self.alloc.free(abs_repo_dir);
        try cmd.appendSlice(self.alloc, res);
        try cmd.appendSlice(self.alloc, abs_repo_dir);
        try cmd.append(self.alloc, ' ');

        // set makeopts
        if (builtin.target.os.tag.isDarwin()) {
            const macos_makeopts = "OSX_BUILD=1 TARGET_BITS=64 TARGET_ARCH=";
            try cmd.appendSlice(self.alloc, macos_makeopts);

            if (builtin.target.cpu.arch.isX86()) {
                try cmd.appendSlice(self.alloc, "x86_64-apple-darwin ");
            } else if (builtin.target.cpu.arch.isArm()) {
                try cmd.appendSlice(self.alloc, "aarch64-apple-darwin ");
            }
        }

        const default_makeopts = "EXTERNAL_DATA=1 RENDER_API=GL WINDOW_API=SDL2 AUDIO_API=SDL2 CONTROLLER_API=SDL2 ";
        try cmd.appendSlice(self.alloc, default_makeopts);

        for (self.spec.makeopts) |makeopt| {
            try cmd.appendSlice(self.alloc, makeopt.opt);
            try cmd.append(self.alloc, ' ');
        }

        // set version
        try cmd.appendSlice(self.alloc, "VERSION=");
        try cmd.appendSlice(self.alloc, @tagName(self.spec.rom.region));

        return try cmd.toOwnedSlice(self.alloc);
    }

    fn setupBuildScript(self: *const Self) !void {
        const script = try self.generateBuildScript();
        defer self.alloc.free(script);

        const script_path = self.joinPaths(&.{ self.base_dir, "scripts", "build.sh" });
        defer self.alloc.free(script_path);

        // create script as executable
        var script_file = try std.fs.cwd().createFile(script_path, .{ .mode = 0o755 });
        defer script_file.close();

        try script_file.writeAll(script);
    }

    fn runBuildScript(self: *const Self) !void {
        const cmd = self.joinPaths(&.{ self.base_dir, "scripts", "build.sh" });
        defer self.alloc.free(cmd);

        var child = std.process.Child.init(&.{cmd}, self.alloc);
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
            // FIXME: proper logging/custom writer support
            std.debug.print("stdout: {s}", .{stdout_buf});
            std.debug.print("stderr: {s}", .{stderr_buf});

            @memset(&stdout_buf, 0);
            @memset(&stderr_buf, 0);
        }

        _ = try child.wait();
    }

    pub fn build(self: *const Self) !void {
        if (self.needCloneRepo()) {
            try self.cloneRepo();
        }

        if (self.needCopyRom()) {
            try self.copyRom();
        }

        if (self.needCreateScriptsDir()) {
            try self.createScriptsDir();
        }

        if (self.needSetupBuildScript()) {
            try self.setupBuildScript();
        }

        if (self.needRunBuildScript()) {
            try self.runBuildScript();
        }
    }

    pub fn run(self: *const Self) !void {
        const cmd = self.getExePath();
        defer self.alloc.free(cmd);
        var child = std.process.Child.init(&.{cmd}, self.alloc);
        _ = try child.spawnAndWait();
    }
};
