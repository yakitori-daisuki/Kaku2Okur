import AppKit

guard CommandLine.arguments.count == 3 else {
    fatalError("usage: swift script/verify_app_icon.swift app-path output.png")
}
let app = URL(fileURLWithPath: CommandLine.arguments[1]).standardizedFileURL
guard let bundle = Bundle(url: app) else { fatalError("Invalid app bundle") }
let source = NSWorkspace.shared.icon(forFile: app.path)
let image = NSImage(size: NSSize(width: 512, height: 512))
image.lockFocus()
source.draw(in: NSRect(x: 0, y: 0, width: 512, height: 512))
image.unlockFocus()
guard let tiff = image.tiffRepresentation,
      let bitmap = NSBitmapImageRep(data: tiff),
      let png = bitmap.representation(using: .png, properties: [:]) else {
    fatalError("Could not render system-resolved app icon")
}
try png.write(to: URL(fileURLWithPath: CommandLine.arguments[2]))
print("App: \(app.path)")
print("Icon: \(bundle.object(forInfoDictionaryKey: "CFBundleIconName") ?? "none")")
