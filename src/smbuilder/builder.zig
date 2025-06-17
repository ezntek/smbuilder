//! Types to be used for the builder.

const std = @import("std");

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
    format: ?RomType, // we can detect this

    const Self = Rom;

    pub fn init(alloc: std.mem.Allocator, path: []const u8, region: Region) !Self {
        const a_path = try alloc.dupe(path);
        return initUnmanaged(a_path, region);
    }

    pub fn initUnmanaged(path: []const u8, region: Region) Self {
        return Self{ .path = path, .region = region };
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
        return initUnmanaged(a_url);
    }

    /// Creates a new repo, assuming that the data is already allocated.
    pub fn initUnmanaged(url: []const u8) Self {
        return Self{
            .url = url,
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

/// Represents a specification to build a game.
pub const Spec = struct {
    /// Repository to build
    repo: Repo,
    /// Base ROM to use
    rom: Rom,
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
