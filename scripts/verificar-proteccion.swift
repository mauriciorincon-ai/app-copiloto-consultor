// Verifica la promesa central de Angel Ghost CON LA MISMA API QUE USA UNA VIDEOLLAMADA.
//
// Por qué existe: `screencapture` no sirve para esto. Medido en vivo — al fotografiar, el sistema
// MUEVE y REDIMENSIONA las ventanas de la app (reportadas en x=-1530, y=839, 1470x117 justo
// después de una captura, cuando en reposo están en x=0, y=868, 1470x88). Con la herramienta
// desplazando aquello mismo que se quiere medir, la franja salía «vacía» y eso se podía leer, con
// toda la buena fe del mundo, como «el relleno no funciona» — o peor, como «la protección
// funciona», cuando lo único que pasaba era que la ventana no estaba donde se miraba.
//
// ScreenCaptureKit es lo que usan Meet, Zoom y Teams. Lo que vea esto es lo que vería el cliente.
//
// Uso: swift scripts/verificar-proteccion.swift [alto-de-la-franja]
import AVFoundation
import CoreGraphics
import Foundation
import ImageIO
import ScreenCaptureKit
import UniformTypeIdentifiers

let altoFranja = CGFloat(Int(CommandLine.arguments.dropFirst().first ?? "88") ?? 88)
let sem = DispatchSemaphore(value: 0)
var codigo: Int32 = 0

Task {
    defer { sem.signal() }
    do {
        let contenido = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
        guard let pantalla = contenido.displays.first else {
            print("✗ no hay pantalla que compartir"); codigo = 1; return
        }

        // 1. ¿Qué ventanas de la app LISTA el sistema como compartibles?
        let mias = contenido.windows.filter {
            ($0.owningApplication?.applicationName ?? "").lowercased().contains("copiloto")
                || ($0.owningApplication?.applicationName ?? "").lowercased().contains("angel")
        }
        print("── lo que una videollamada puede compartir de esta app ──────")
        if mias.isEmpty {
            print("   ninguna ventana de la app aparece en el contenido compartible")
        }
        for w in mias {
            let r = w.frame
            print("   id=\(w.windowID) \(Int(r.width))x\(Int(r.height)) en (\(Int(r.minX)),\(Int(r.minY)))  capa=\(w.windowLayer)")
        }
        print("────────────────────────────────────────────────────────────\n")

        // 2. La pantalla entera, tal como la vería el cliente.
        let filtro = SCContentFilter(display: pantalla, excludingWindows: [])
        let cfg = SCStreamConfiguration()
        cfg.width = pantalla.width * 2
        cfg.height = pantalla.height * 2
        cfg.showsCursor = false
        let imagen = try await SCScreenshotManager.captureImage(contentFilter: filtro, configuration: cfg)

        // 3. SOLO la franja de abajo: ni un píxel más de la pantalla del usuario se inspecciona.
        let escala = CGFloat(imagen.height) / CGFloat(pantalla.height)
        let alto = Int(altoFranja * escala)
        let franjaRect = CGRect(x: 0, y: imagen.height - alto, width: imagen.width, height: alto)
        guard let franja = imagen.cropping(to: franjaRect) else {
            print("✗ no se pudo recortar la franja"); codigo = 1; return
        }

        // 4. Color medio y porcentaje de negro puro, nada más.
        let ancho = franja.width, altoPx = franja.height
        var datos = [UInt8](repeating: 0, count: ancho * altoPx * 4)
        let espacio = CGColorSpaceCreateDeviceRGB()
        guard let ctx = CGContext(data: &datos, width: ancho, height: altoPx, bitsPerComponent: 8,
                                  bytesPerRow: ancho * 4, space: espacio,
                                  bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue) else {
            print("✗ no se pudo leer la franja"); codigo = 1; return
        }
        ctx.draw(franja, in: CGRect(x: 0, y: 0, width: ancho, height: altoPx))
        var r = 0, g = 0, b = 0, negros = 0
        let n = ancho * altoPx
        for i in stride(from: 0, to: datos.count, by: 4) {
            r += Int(datos[i]); g += Int(datos[i+1]); b += Int(datos[i+2])
            if datos[i] < 12 && datos[i+1] < 12 && datos[i+2] < 12 { negros += 1 }
        }
        let pctNegro = Double(negros) / Double(n) * 100
        print("── la franja donde vive la banda, vista por la videollamada ─")
        print("   tamaño: \(ancho)x\(altoPx) px")
        print("   color medio: rgb(\(r/n), \(g/n), \(b/n))")
        print(String(format: "   negro puro: %.1f %%", pctNegro))
        if pctNegro > 99 {
            print("   ✓ la franja es NEGRA: el relleno la cubre y no se filtra nada")
        } else {
            print("   ⚠ la franja NO es uniforme: algo distinto del relleno se está viendo ahí")
            codigo = 2
        }
        print("────────────────────────────────────────────────────────────")

        if let salida = ProcessInfo.processInfo.environment["FRANJA_PNG"] {
            let url = URL(fileURLWithPath: salida)
            if let dst = CGImageDestinationCreateWithURL(url as CFURL, UTType.png.identifier as CFString, 1, nil) {
                CGImageDestinationAddImage(dst, franja, nil)
                CGImageDestinationFinalize(dst)
                print("   (franja guardada en \(salida))")
            }
        }
    } catch {
        print("✗ ScreenCaptureKit: \(error)")
        codigo = 1
    }
}
sem.wait()
exit(codigo)
