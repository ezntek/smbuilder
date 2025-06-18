//! Types to be used for the builder.

const std = @import("std");
const panic = std.debug.panic;
const util = @import("util.zig");
const rc = @import("n64comconvert");

/// Represents the ROM region
pub const Region = enum {
    us,
    eu,
    jp,
    shindou,
};

pub const RomType = rc.RomType;

/// Represents a ROM file
pub const Rom = struct {
    region: Region,
    /// owned slice to the path
    path: []const u8,

    const Self = Rom;

    pub fn init(alloc: std.mem.Allocator, path: []const u8, region: Region) !Self {
        const a_path = try alloc.dupe(path);
        return Self{
            .region = region,
            .path = a_path,
        };
    }

    /// deinits the struct. Calling this if the memory is not allocated by `alloc` is UB.
    pub fn deinit(self: *const Self, alloc: std.mem.Allocator) void {
        alloc.free(self.path);
    }
};

/// Represents a git repo with the source code of a port.
pub const Repo = struct {
    /// owned slice to the URL, in the form of <url>@<branch>
    url: []const u8,

    const Self = Repo;

    /// Create a new repo
    pub fn init(alloc: std.mem.Allocator, url: []const u8) !Self {
        const a_url = try alloc.dupe(u8, url);
        return Self{
            .url = a_url,
        };
    }

    /// returns just the repo part of the URL as a borrowed slice
    pub fn getURL(self: *const Self) []const u8 {
        const at_pos = std.mem.indexOfScalar(u8, self.url, '@');

        if (at_pos) |pos| {
            return self.url[0..pos];
        } else {
            return self.url;
        }
    }

    /// returns the branch part of the URL as a borrowed slice
    pub fn getBranch(self: *const Self) ?[]const u8 {
        const at_pos = std.mem.indexOfScalar(u8, self.url, '@');
        if (at_pos) |pos| {
            if (pos == self.url.len - 1) {
                return null;
            }

            return self.url[pos + 1 ..];
        } else {
            return null;
        }
    }

    /// deinits the struct. Calling this if the memory is not allocated by `alloc` is UB.
    pub fn deinit(self: *const Self, alloc: std.mem.Allocator) void {
        alloc.free(self.url);
    }
};

pub const TexturePack = struct {
    /// path of the texture pack
    path: []const u8,
    /// name of the texture pack, used as a name in launchers
    name: []const u8,

    const Self = TexturePack;

    pub fn init(alloc: std.mem.Allocator, path: []const u8, name: []const u8) !Self {
        const a_path = try alloc.dupe(path);
        const a_name = try alloc.dupe(name);
        return Self{
            .path = a_path,
            .name = a_name,
        };
    }

    pub fn deinit(self: *const Self, alloc: std.mem.Allocator) void {
        alloc.free(self.path);
        alloc.free(self.name);
    }
};

pub const DynosPack = struct {
    /// path of the datapack
    path: []const u8,
    /// name of the datapack, used as a name in launchers
    name: []const u8,

    const Self = DynosPack;

    pub fn init(alloc: std.mem.Allocator, path: []const u8, name: []const u8) !Self {
        const a_path = try alloc.dupe(path);
        const a_name = try alloc.dupe(name);
        return Self{
            .path = a_path,
            .name = a_name,
        };
    }

    pub fn deinit(self: *const Self, alloc: std.mem.Allocator) void {
        alloc.free(self.path);
        alloc.free(self.name);
    }
};

pub const Makeopt = struct {
    opt: []const u8,

    const Self = Makeopt;

    pub fn init(alloc: std.mem.Allocator, opt: []const u8, val: []const u8) !Self {
        const alist: std.ArrayListUnmanaged(u8) = .empty;
        try alist.appendSlice(alloc, opt);
        try alist.append(alloc, '=');
        try alist.appendSlice(alloc, val);
        return Self{
            .opt = try alist.toOwnedSlice(alloc),
        };
    }

    pub fn getOpt(self: *const Self) void {
        const idx = std.mem.indexOfScalar(u8, self.opt, '=');
        return self.opt[0..idx];
    }

    pub fn getValue(self: *const Self) void {
        const idx = std.mem.indexOfScalar(u8, self.opt, '=');
        return self.opt[idx + 1 ..];
    }

    pub fn deinit(self: *const Self, alloc: std.mem.Allocator) void {
        alloc.free(self.opt);
    }
};

/// Represents a specification to build a game.
pub const Spec = struct {
    /// Repository to build
    repo: Repo,
    /// Base ROM to use
    rom: Rom,
    /// Texture Pack
    texture_pack: ?TexturePack,
    /// Datapacks (DynOS packs),
    dynos_packs: []DynosPack,
    /// Custom Makeopts
    makeopts: []Makeopt,
    /// Jobs
    jobs: u16,

    const Self = Spec;

    pub fn builder(alloc: std.mem.Allocator) SpecBuilder {
        return SpecBuilder.init(alloc);
    }

    pub fn deinit(self: *const Self, alloc: std.mem.Allocator) Self {
        self.repo.deinit(alloc);
        self.rom.deinit(alloc);
        if (self.texture_pack) |pack| {
            pack.deinit(alloc);
        }
        for (self.dynos_packs) |pack| {
            pack.deinit(alloc);
        }
        for (self.makeopts) |makeopt| {
            makeopt.deinit(alloc);
        }
    }
};

pub const SpecBuilder = struct {
    alloc: std.mem.Allocator,
    repo: ?Repo,
    rom: ?Rom,
    texture_pack: ?TexturePack,
    dynos_packs: std.ArrayListUnmanaged(DynosPack),
    makeopts: std.ArrayListUnmanaged(Makeopt),
    jobs: ?u16,

    const Self = SpecBuilder;

    pub fn init(alloc: std.mem.Allocator) Self {
        return Self{
            .alloc = alloc,
            .dynos_packs = .empty,
            .makeopts = .empty,
        };
    }

    pub fn setRepo(self: *Self, url: []const u8) Self {
        self.repo = Repo.init(self.alloc, url);
        return self;
    }

    pub fn setRom(self: *Self, region: Region, path: []const u8) Self {
        self.rom = Rom.init(self.alloc, region, path) catch |err| panic("allocation for ROM object creation failed: {any}", .{err});
        return self;
    }

    pub fn setTexturePack(self: *Self, path: []const u8, name: []const u8) Self {
        self.texture_pack = TexturePack.init(self.alloc, path, name);
        return self;
    }

    pub fn addDynosPack(self: *Self, pack: DynosPack) Self {
        self.dynos_packs.append(self.alloc, pack) catch |err| panic("allocation for new DynOS pack failed: {any}", .{err});
        return self;
    }

    pub fn addMakeopt(self: *Self, makeopt: Makeopt) Self {
        self.makeopts.append(self.alloc, makeopt) catch |err| panic("allocation for new Makeopt failed: {any}", .{err});
        return self;
    }

    pub fn setJobs(self: *Self, jobs: u16) Self {
        self.jobs = jobs;
        return self;
    }

    pub fn build(self: Self) !Spec {
        const repo = self._repo.?;
        const rom = self._rom.?;
        const texture_pack = self._texture_pack;
        const dynos_packs = try self._dynos_packs.toOwnedSlice(self.alloc);
        const makeopts = try self._makeopts.toOwnedSlice(self.alloc);
        const jobs = self._jobs.?;

        return Spec{
            .repo = repo,
            .rom = rom,
            .texture_pack = texture_pack,
            .dynos_packs = dynos_packs,
            .makeopts = makeopts,
            .jobs = jobs,
        };
    }
};

test "rom repo slicing" {
    const repo = Repo.initUnmanaged("theURL@theBranch");

    try std.testing.expect(std.mem.eql(u8, repo.getURL(), "theURL"));
}

test "rom branch slicing" {
    const alloc = std.heap.page_allocator;
    const repo = try Repo.init(alloc, "theURL@theBranch");
    defer repo.deinit(alloc);

    try std.testing.expect(std.mem.eql(u8, repo.getBranch().?, "theBranch"));
}

test "rom branch slicing (no at)" {
    const repo = Repo.initUnmanaged("theURL");

    try std.testing.expect(repo.getBranch() == null);
}

test "rom branch slicing (nothing after at)" {
    const repo = Repo.initUnmanaged("theURL@");

    try std.testing.expect(repo.getBranch() == null);
}
