import AppKit
import CoreGraphics

let repoRoot = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
let iconSvgPath = repoRoot.appendingPathComponent("assets/image/logo/icon.svg")
let outputIconsDir = repoRoot.appendingPathComponent("clients/tauri_app/src-tauri/icons")
let tmpDir = FileManager.default.temporaryDirectory.appendingPathComponent("devflow_icon_gen_\(UUID().uuidString)")
let iconsetDir = tmpDir.appendingPathComponent("devflow.iconset")

try? FileManager.default.createDirectory(at: iconsetDir, withIntermediateDirectories: true)
try? FileManager.default.createDirectory(at: outputIconsDir, withIntermediateDirectories: true)

let size = CGSize(width: 1024, height: 1024)
let colorSpace = CGColorSpaceCreateDeviceRGB()
guard let context = CGContext(
    data: nil,
    width: Int(size.width),
    height: Int(size.height),
    bitsPerComponent: 8,
    bytesPerRow: 0,
    space: colorSpace,
    bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
) else {
    print("❌ Failed to create CGContext")
    exit(1)
}

// 1. Transparent base canvas (outside squircle is 100% transparent)
context.clear(CGRect(origin: .zero, size: size))

// 2. macOS standard squircle bounds: 824x824 centered in 1024x1024 (padding: 100)
let rect = CGRect(x: 100, y: 100, width: 824, height: 824)
let cornerRadius: CGFloat = 185.0
let path = CGPath(roundedRect: rect, cornerWidth: cornerRadius, cornerHeight: cornerRadius, transform: nil)

// 3. Subtle macOS Drop Shadow
context.saveGState()
context.setShadow(
    offset: CGSize(width: 0, height: -18),
    blur: 32,
    color: CGColor(red: 0, green: 0, blue: 0, alpha: 0.5)
)

// Fill rounded rect with base color to cast shadow
context.addPath(path)
context.setFillColor(CGColor(red: 0.05, green: 0.07, blue: 0.1, alpha: 1.0))
context.fillPath()
context.restoreGState()

// 4. Clip to squircle and draw Dark Gradient
context.saveGState()
context.addPath(path)
context.clip()

let gradientColors = [
    CGColor(red: 0.08, green: 0.10, blue: 0.15, alpha: 1.0),
    CGColor(red: 0.04, green: 0.05, blue: 0.08, alpha: 1.0),
    CGColor(red: 0.02, green: 0.03, blue: 0.05, alpha: 1.0)
] as CFArray

let locations: [CGFloat] = [0.0, 0.5, 1.0]
if let gradient = CGGradient(colorsSpace: colorSpace, colors: gradientColors, locations: locations) {
    context.drawLinearGradient(
        gradient,
        start: CGPoint(x: 512, y: 924),
        end: CGPoint(x: 512, y: 100),
        options: []
    )
}

// 5. Ambient Radial Glow behind the icon mark
let glowColors = [
    CGColor(red: 0.0, green: 0.9, blue: 1.0, alpha: 0.22),
    CGColor(red: 0.11, green: 0.31, blue: 0.85, alpha: 0.10),
    CGColor(red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0)
] as CFArray
if let radialGlow = CGGradient(colorsSpace: colorSpace, colors: glowColors, locations: [0.0, 0.45, 1.0]) {
    context.drawRadialGradient(
        radialGlow,
        startCenter: CGPoint(x: 512, y: 512),
        startRadius: 0,
        endCenter: CGPoint(x: 512, y: 512),
        endRadius: 360,
        options: []
    )
}

// 6. Draw SVG Mark inside
if let svgImage = NSImage(contentsOf: iconSvgPath) {
    let markWidth: CGFloat = 460
    let markHeight: CGFloat = markWidth * (206.0 / 180.0) // aspect ratio of SVG viewBox (180x206)
    let markRect = NSRect(
        x: (1024 - markWidth) / 2 + 10,
        y: (1024 - markHeight) / 2,
        width: markWidth,
        height: markHeight
    )

    let graphicsContext = NSGraphicsContext(cgContext: context, flipped: false)
    NSGraphicsContext.current = graphicsContext
    svgImage.draw(in: markRect, from: .zero, operation: .sourceOver, fraction: 1.0)
}

context.restoreGState()

// 7. Subtle Border Stroke
context.saveGState()
context.addPath(path)
context.setLineWidth(3.0)
context.setStrokeColor(CGColor(red: 0.22, green: 0.28, blue: 0.38, alpha: 0.7))
context.strokePath()
context.restoreGState()

guard let masterImage = context.makeImage() else {
    print("❌ Failed to create master CGImage")
    exit(1)
}

func savePNG(_ cgImage: CGImage, to url: URL) {
    let dest = CGImageDestinationCreateWithURL(url as CFURL, "public.png" as CFString, 1, nil)!
    CGImageDestinationAddImage(dest, cgImage, nil)
    CGImageDestinationFinalize(dest)
}

func resizeImage(_ image: CGImage, targetSize: CGSize) -> CGImage? {
    guard let ctx = CGContext(
        data: nil,
        width: Int(targetSize.width),
        height: Int(targetSize.height),
        bitsPerComponent: 8,
        bytesPerRow: 0,
        space: colorSpace,
        bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
    ) else { return nil }
    ctx.interpolationQuality = .high
    ctx.draw(image, in: CGRect(origin: .zero, size: targetSize))
    return ctx.makeImage()
}

// Generate iconset files
let sizes: [(String, CGFloat)] = [
    ("icon_16x16.png", 16),
    ("icon_16x16@2x.png", 32),
    ("icon_32x32.png", 32),
    ("icon_32x32@2x.png", 64),
    ("icon_128x128.png", 128),
    ("icon_128x128@2x.png", 256),
    ("icon_256x256.png", 256),
    ("icon_256x256@2x.png", 512),
    ("icon_512x512.png", 512),
    ("icon_512x512@2x.png", 1024)
]

for (name, s) in sizes {
    if let resized = resizeImage(masterImage, targetSize: CGSize(width: s, height: s)) {
        savePNG(resized, to: iconsetDir.appendingPathComponent(name))
    }
}

// Generate Tauri bundle PNGs
savePNG(masterImage, to: outputIconsDir.appendingPathComponent("icon.png"))
if let i32 = resizeImage(masterImage, targetSize: CGSize(width: 32, height: 32)) {
    savePNG(i32, to: outputIconsDir.appendingPathComponent("32x32.png"))
}
if let i64 = resizeImage(masterImage, targetSize: CGSize(width: 64, height: 64)) {
    savePNG(i64, to: outputIconsDir.appendingPathComponent("64x64.png"))
}
if let i128 = resizeImage(masterImage, targetSize: CGSize(width: 128, height: 128)) {
    savePNG(i128, to: outputIconsDir.appendingPathComponent("128x128.png"))
}
if let i256 = resizeImage(masterImage, targetSize: CGSize(width: 256, height: 256)) {
    savePNG(i256, to: outputIconsDir.appendingPathComponent("128x128@2x.png"))
}

// Square Windows/Store sizes
let squareSizes: [(String, CGFloat)] = [
    ("Square30x30Logo.png", 30),
    ("Square44x44Logo.png", 44),
    ("Square71x71Logo.png", 71),
    ("Square89x89Logo.png", 89),
    ("Square107x107Logo.png", 107),
    ("Square142x142Logo.png", 142),
    ("Square150x150Logo.png", 150),
    ("Square284x284Logo.png", 284),
    ("Square310x310Logo.png", 310),
    ("StoreLogo.png", 50)
]
for (name, s) in squareSizes {
    if let resized = resizeImage(masterImage, targetSize: CGSize(width: s, height: s)) {
        savePNG(resized, to: outputIconsDir.appendingPathComponent(name))
    }
}

// Run iconutil to create native macOS icon.icns
let process = Process()
process.executableURL = URL(fileURLWithPath: "/usr/bin/iconutil")
process.arguments = ["-c", "icns", iconsetDir.path, "-o", outputIconsDir.appendingPathComponent("icon.icns").path]
try? process.run()
process.waitUntilExit()

try? FileManager.default.removeItem(at: tmpDir)
print("✅ Native macOS dock icon (.icns) with transparent background generated successfully!")
