//! Types to be used for the builder.

const std = @import("std");
const panic = std.debug.panic;
const rc = @import("n64comconvert");

/// Represents the ROM region
pub const Region = enum {
    us,
    eu,
    jp,
};

pub const RomType = rc.RomType;

/// Represents a ROM file
pub const Rom = struct {
    region: Region,
    /// owned slice to the path
    path: []const u8,

    const Self = Rom;

    pub fn init(alloc: std.mem.Allocator, region: Region, path: []const u8) !Self {
        const a_path = try alloc.dupe(u8, path);
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

    pub fn getName(self: *const Self) []const u8 {
        const url = self.getURL();
        const slash_pos = std.mem.lastIndexOfScalar(u8, url, '/') orelse panic("could not get base name of repo", .{});
        return url[slash_pos + 1 ..];
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

    pub fn jsonStringify(self: *const Self, out: anytype) !void {
        try out.print("\"{s}\"", .{self.url});
    }

    pub fn jsonParse(alloc: std.mem.Allocator, source: anytype, options: std.json.ParseOptions) !Self {
        _ = options; // im a barbaric madman
        return switch (try source.next()) {
            .string => |url| Self{
                .url = try alloc.dupe(u8, url),
            },
            else => return error.UnexpectedToken,
        };
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
        const a_path = try alloc.dupe(u8, path);
        const a_name = try alloc.dupe(u8, name);
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
        const a_path = try alloc.dupe(u8, path);
        const a_name = try alloc.dupe(u8, name);
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
        var alist: std.ArrayListUnmanaged(u8) = .empty;
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

    pub fn jsonStringify(self: *const Self, out: anytype) !void {
        try out.print("\"{s}\"", .{self.opt});
    }

    pub fn jsonParse(alloc: std.mem.Allocator, source: anytype, options: std.json.ParseOptions) !Makeopt {
        _ = options; // im a barbaric madman
        return switch (try source.next()) {
            .string => |opt| Makeopt{
                .opt = try alloc.dupe(u8, opt),
            },
            else => return error.UnexpectedToken,
        };
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

    pub fn deinit(self: *const Self, alloc: std.mem.Allocator) void {
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

    pub fn toJson(self: *const Self, alloc: std.mem.Allocator) ![]const u8 {
        const res = try std.json.stringifyAlloc(alloc, self.*, .{});
        return res;
    }

    pub fn fromJson(alloc: std.mem.Allocator, json: []const u8) !Self {
        const parsed = try std.json.parseFromSlice(Self, alloc, json, .{});
        const res = try alloc.dupe(Self, parsed.value);
        parsed.deinit();
        return res;
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
            .repo = null,
            .rom = null,
            .texture_pack = null,
            .jobs = null,
        };
    }

    pub fn setRepo(self: *Self, url: []const u8) *Self {
        self.repo = Repo.init(self.alloc, url) catch |err| panic("allocation for Repo object creation failed: {any}", .{err});
        return self;
    }

    pub fn setRom(self: *Self, region: Region, path: []const u8) *Self {
        self.rom = Rom.init(self.alloc, region, path) catch |err| panic("allocation for ROM object creation failed: {any}", .{err});
        return self;
    }

    pub fn setTexturePack(self: *Self, path: []const u8, name: []const u8) *Self {
        const pack = TexturePack.init(self.alloc, path, name);
        return self.setTexturePackStruct(pack);
    }

    pub fn setTexturePackStruct(self: *Self, pack: TexturePack) *Self {
        self.texture_pack = pack;
        return self;
    }

    pub fn addDynosPack(self: *Self, pack: DynosPack) *Self {
        self.dynos_packs.append(self.alloc, pack) catch |err| panic("allocation for new DynOS pack failed: {any}", .{err});
        return self;
    }

    pub fn addDynosPackStruct(self: *Self, path: []const u8, name: []const u8) *Self {
        const pack = DynosPack.init(self.alloc, path, name) catch |err| panic("allocation for DynOS pack failed: {any}", .{err});
        return self.addDynosPack(pack);
    }

    pub fn addMakeopt(self: *Self, opt: []const u8, val: []const u8) *Self {
        const makeopt = Makeopt.init(self.alloc, opt, val) catch |err| panic("allocation for new Makeopt failed: {any}", .{err});
        return self.addMakeoptStruct(makeopt);
    }

    pub fn addMakeoptStruct(self: *Self, makeopt: Makeopt) *Self {
        self.makeopts.append(self.alloc, makeopt) catch |err| panic("allocation for new Makeopt failed: {any}", .{err});
        return self;
    }

    pub fn setJobs(self: *Self, jobs: u16) *Self {
        self.jobs = jobs;
        return self;
    }

    pub fn build(self: *Self) !Spec {
        const repo = self.repo.?;
        const rom = self.rom.?;
        const texture_pack = self.texture_pack;
        const dynos_packs = try self.dynos_packs.toOwnedSlice(self.alloc);
        const makeopts = try self.makeopts.toOwnedSlice(self.alloc);
        const jobs = self.jobs.?;

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
    const alloc = std.heap.page_allocator;
    const repo = try Repo.init(alloc, "theURL@theBranch");

    try std.testing.expectEqualStrings("theURL", repo.getURL());
}

test "rom branch slicing" {
    const alloc = std.heap.page_allocator;
    const repo = try Repo.init(alloc, "theURL@theBranch");
    defer repo.deinit(alloc);

    try std.testing.expectEqualStrings("theBranch", repo.getBranch().?);
}

test "rom branch slicing (no at)" {
    const alloc = std.heap.page_allocator;
    const repo = try Repo.init(alloc, "theURL");
    defer repo.deinit(alloc);

    try std.testing.expectEqual(null, repo.getBranch());
}

test "rom branch slicing (nothing after at)" {
    const alloc = std.heap.page_allocator;
    const repo = try Repo.init(alloc, "theURL@");
    defer repo.deinit(alloc);

    try std.testing.expectEqual(null, repo.getBranch());
}

test "makeopt json dump" {
    const alloc = std.heap.page_allocator;
    var builder = Spec.builder(alloc);
    const spec = try builder
        .setRepo("https://github.com/sm64pc/sm64ex@nightly")
        .setRom(.us, "./baserom.us.z64")
        .addMakeopt("BETTERCAMERA", "1")
        .setJobs(8)
        .build();
    const json = try spec.toJson(alloc);
    defer alloc.free(json);

    std.debug.print("{s}", .{json});
}

test "makeopt json parse" {
    const alloc = std.heap.page_allocator;
    const s = "{ \"makeopts\": [\"goodmorning\"] }";
    const TestStruct = struct { makeopts: []Makeopt };
    const parsed = try std.json.parseFromSlice(TestStruct, alloc, s, .{});
    defer parsed.deinit();
    try std.testing.expectEqualStrings("goodmorning", parsed.value.makeopts[0].opt);
}
