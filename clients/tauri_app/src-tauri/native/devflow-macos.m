#import <Cocoa/Cocoa.h>

void devflow_set_macos_dock_icon(const void* bytes, size_t len) {
    if (!bytes || len == 0) return;
    @autoreleasepool {
        NSData *data = [NSData dataWithBytes:bytes length:len];
        if (!data) return;
        NSImage *image = [[NSImage alloc] initWithData:data];
        if (!image) return;
        [NSApplication sharedApplication];
        [NSApp setApplicationIconImage:image];
    }
}
