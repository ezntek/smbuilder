const std = @import("std");
const smbuilder = @import("smbuilder");

pub fn main() !void {
    const alloc = std.heap.page_allocator;
    var b = smbuilder.types.Spec.builder(alloc);
    const spec = try b
        .setRepo("https://github.com/sm64pc/sm64ex@nightly")
        .setRom(.us, "/home/ezntek/baserom.us.v64")
        .addMakeopt("DISCORDRPC", "1")
        .setJobs(8)
        .build();

    const dir = "./build";

    const builder = try smbuilder.builder.Builder.init(alloc, spec, dir);
    builder.build() catch |err| std.debug.panic("a fatal error occured: {any}", .{err});
}
