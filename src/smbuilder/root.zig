const std = @import("std");

pub fn sayHello(name: []const u8) void {
    const stdout = std.io.getStdOut().writer();
    var bw = std.io.bufferedWriter(stdout);
    defer bw.flush() catch @panic("could not flush buffer");
    const writer = bw.writer();
    writer.print("good morning, {s}\n", .{name}) catch @panic("could not write to stdout");
}
