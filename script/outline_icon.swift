import CoreGraphics
import Foundation

// Expand the original ink stroke so Icon Composer can recolor its filled silhouette.
let line = CGMutablePath()
line.move(to: CGPoint(x: 390, y: 620))
line.addCurve(to: CGPoint(x: 268, y: 691),
              control1: CGPoint(x: 326, y: 601), control2: CGPoint(x: 257, y: 637))
line.addCurve(to: CGPoint(x: 463, y: 720),
              control1: CGPoint(x: 281, y: 752), control2: CGPoint(x: 385, y: 758))
line.addCurve(to: CGPoint(x: 640, y: 647),
              control1: CGPoint(x: 528, y: 688), control2: CGPoint(x: 569, y: 650))
let outline = line.copy(strokingWithWidth: 38, lineCap: .round, lineJoin: .round, miterLimit: 10)
func coordinates(_ point: CGPoint) -> String {
    String(format: "%.3f %.3f", Double(point.x), Double(point.y))
}
var commands: [String] = []
outline.applyWithBlock { pointer in
    let element = pointer.pointee
    switch element.type {
    case .moveToPoint: commands.append("M" + coordinates(element.points[0]))
    case .addLineToPoint: commands.append("L" + coordinates(element.points[0]))
    case .addQuadCurveToPoint:
        commands.append("Q" + coordinates(element.points[0]) + " " + coordinates(element.points[1]))
    case .addCurveToPoint:
        commands.append("C" + coordinates(element.points[0]) + " " + coordinates(element.points[1]) + " " + coordinates(element.points[2]))
    case .closeSubpath: commands.append("Z")
    @unknown default: fatalError("Unsupported path element")
    }
}
let svg = """
<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024">
  <path d="\(commands.joined(separator: " "))" fill="#263746"/>
</svg>

"""
guard CommandLine.arguments.count == 2 else {
    fatalError("Usage: swift script/outline_icon.swift output.svg")
}
try svg.write(toFile: CommandLine.arguments[1], atomically: true, encoding: .utf8)
