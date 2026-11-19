// Builds the OMNI Launcher and OMNI Client icons from the brand sheet's symbol.
// swift make_icons.swift <out dir>   (sheet: omni-launcher-brand-sheet.png next to this file)
import Foundation
import CoreGraphics
import ImageIO
import UniformTypeIdentifiers

let here = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let out = URL(fileURLWithPath: CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "/tmp/omni-icons")
try? FileManager.default.createDirectory(at: out, withIntermediateDirectories: true)

let sheet = CGImageSourceCreateWithURL(here.appendingPathComponent("omni-launcher-brand-sheet.png") as CFURL, nil)
    .flatMap { CGImageSourceCreateImageAtIndex($0, 0, nil) }!
let crop = sheet.cropping(to: CGRect(x: 1250, y: 448, width: 245, height: 172))!
let W = crop.width, H = crop.height
let space = CGColorSpaceCreateDeviceRGB()
func bitmap(_ w: Int, _ h: Int) -> CGContext {
    CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: 0, space: space,
              bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
}
// Read the crop's pixels once.
let src = bitmap(W, H)
src.draw(crop, in: CGRect(x: 0, y: 0, width: W, height: H))
let px = src.data!.assumingMemoryBound(to: UInt8.self)
let rb = src.bytesPerRow

/// The symbol with the sheet's black keyed out; `invert` makes the negative for the cream tile.
func symbol(invert: Bool) -> CGImage {
    let c = bitmap(W, H)
    let o = c.data!.assumingMemoryBound(to: UInt8.self)
    let charcoal: [Double] = [26, 26, 29], cream: [Double] = [232, 220, 200]
    for y in 0..<H { for x in 0..<W {
        let i = y * rb + x * 4
        let r = Double(px[i]), g = Double(px[i + 1]), b = Double(px[i + 2])
        let m = max(r, g, b)
        let red = r - b > 70
        var col = [r, g, b]
        var a = min(1, max(0, (m - 28) / 60))
        if invert && !red { col = charcoal }
        o[i] = UInt8(col[0] * a); o[i + 1] = UInt8(col[1] * a); o[i + 2] = UInt8(col[2] * a); o[i + 3] = UInt8(a * 255)
    } }
    return c.makeImage()!
}

func icon(size: Int, client: Bool) -> CGImage {
    let c = bitmap(size, size)
    let s = CGFloat(size)
    let tile = s * 824 / 1024, inset = (s - tile) / 2
    let rect = CGRect(x: inset, y: inset, width: tile, height: tile)
    let path = CGPath(roundedRect: rect, cornerWidth: tile * 0.2237, cornerHeight: tile * 0.2237, transform: nil)
    c.addPath(path); c.clip()
    let fr: CGFloat = client ? 0.9098 : 0.1020
    let fg: CGFloat = client ? 0.8627 : 0.1020
    let fb: CGFloat = client ? 0.7843 : 0.1137
    c.setFillColor(red: fr, green: fg, blue: fb, alpha: 1); c.fill(rect)
    if !client {   // a quiet hairline so the tile reads on a dark dock
        c.setStrokeColor(red: 1, green: 1, blue: 1, alpha: 0.08); c.setLineWidth(max(1, s / 256))
        c.addPath(path); c.strokePath()
    }
    let img = symbol(invert: client)
    let w = tile * 0.8, h = w * CGFloat(H) / CGFloat(W)
    c.interpolationQuality = .high
    c.draw(img, in: CGRect(x: s / 2 - w / 2, y: s / 2 - h / 2 - tile * 0.01, width: w, height: h))
    return c.makeImage()!
}
func save(_ img: CGImage, _ url: URL) {
    let d = CGImageDestinationCreateWithURL(url as CFURL, UTType.png.identifier as CFString, 1, nil)!
    CGImageDestinationAddImage(d, img, nil); _ = CGImageDestinationFinalize(d)
}
for (name, client) in [("launcher", false), ("client", true)] {
    let set = out.appendingPathComponent("\(name).iconset")
    try? FileManager.default.createDirectory(at: set, withIntermediateDirectories: true)
    for base in [16, 32, 128, 256, 512] {
        save(icon(size: base, client: client), set.appendingPathComponent("icon_\(base)x\(base).png"))
        save(icon(size: base * 2, client: client), set.appendingPathComponent("icon_\(base)x\(base)@2x.png"))
    }
    for n in [24, 48, 64] { save(icon(size: n, client: client), out.appendingPathComponent("\(name)-\(n).png")) }
    save(icon(size: 1024, client: client), out.appendingPathComponent("\(name)-1024.png"))
}
// the symbol alone, transparent, for the in-app marks
save(symbol(invert: false), out.appendingPathComponent("symbol.png"))
print("done")
