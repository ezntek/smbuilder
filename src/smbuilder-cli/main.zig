const std = @import("std");
const smbuilder = @import("smbuilder");

pub fn main() !void {
    const alloc = std.heap.page_allocator;
    var builder = smbuilder.builder.types.Spec.builder(alloc);
    const spec = try builder
        .setRepo("https://github.com/sm64pc/sm64ex@nightly")
        .setRom(.us, "./baserom.us.z64")
        .addMakeopt("BETTERCAMERA", "1")
        .setJobs(8)
        .build();
    const json = try spec.dumpJson(alloc);

    std.debug.print("{s}", .{json});
}
