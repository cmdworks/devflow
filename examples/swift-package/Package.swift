// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "SampleSwiftCli",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(name: "SampleSwiftCli", targets: ["SampleSwiftCli"])
    ],
    targets: [
        .executableTarget(
            name: "SampleSwiftCli",
            path: "Sources"
        )
    ]
)
