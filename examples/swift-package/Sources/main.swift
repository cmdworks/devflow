import Foundation

print("[SampleSwiftCli] Swift service started v1.0.0")
for i in 1...3 {
    print("[SampleSwiftCli] Heartbeat #\(i) on macOS Darwin")
    Thread.sleep(forTimeInterval: 0.3)
}
print("[SampleSwiftCli] Finished run.")
