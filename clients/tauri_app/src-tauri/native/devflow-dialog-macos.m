#import <Cocoa/Cocoa.h>

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        [NSApplication sharedApplication];
        [NSApp setActivationPolicy:NSApplicationActivationPolicyAccessory];
        [NSApp activateIgnoringOtherApps:YES];

        NSOpenPanel *panel = [NSOpenPanel openPanel];
        [panel setCanChooseFiles:NO];
        [panel setCanChooseDirectories:YES];
        [panel setAllowsMultipleSelection:NO];
        [panel setPrompt:@"Select"];
        [panel setMessage:@"Select DevFlow Workspace Directory"];
        [panel setTitle:@"Select Workspace Directory"];

        if (argc > 1) {
            NSString *initPath = [NSString stringWithUTF8String:argv[1]];
            if (initPath && [[NSFileManager defaultManager] fileExistsAtPath:initPath]) {
                [panel setDirectoryURL:[NSURL fileURLWithPath:initPath]];
            }
        }

        if ([panel runModal] == NSModalResponseOK) {
            NSURL *url = [[panel URLs] firstObject];
            if (url) {
                printf("%s\n", [[url path] UTF8String]);
                return 0;
            }
        }
        return 1;
    }
}
