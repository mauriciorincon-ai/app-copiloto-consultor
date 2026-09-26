// Pantalla.swift — el puente hacia ScreenCaptureKit y Vision: **leer la pantalla que se comparte**.
//
// **MÓDULO PROTEGIDO.** Lo que pasa por aquí es la ventana de una reunión: la diapositiva del
// cliente, los nombres de los participantes, su chat. La regla 1 de la casa lo pone del lado que
// muere siempre. Apple ofrece varias maneras de que un cuadro sobreviva a la memoria —hacerlo PNG,
// grabar la ventana a un vídeo, escribirlo en un archivo— y este archivo no usa ninguna;
// `pnpm verify:ephemeral` las prohíbe por nombre (la lista vive en el script, no aquí, porque
// nombrarlas aquí sería tropezar con el propio gate). Un cuadro de la reunión en un archivo sería
// exactamente una grabación de la pantalla del cliente.
//
// **Las tres decisiones de forma de este archivo.**
//
// **Una: el cuadro se escribe en memoria de Rust, no en memoria de Swift.** `ag_pantalla_mirar`
// recibe un puntero al búfer que Rust reservó una vez, y pinta ahí la ventana en grises. Swift no
// se queda con ninguna copia: el `CGImage` que devuelve ScreenCaptureKit vive lo que dura la
// función. Así el kill-switch tiene UN sitio que pisar, y lo pisa Rust.
//
// **Dos: se pregunta el permiso ANTES de tocar ScreenCaptureKit.** `SCShareableContent` provoca el
// diálogo de macOS si el permiso no se ha decidido, y esta app no pide nada por sorpresa (lo decidió
// la maqueta: «tú los concedes en el sistema, no aquí»). `CGPreflightScreenCaptureAccess` pregunta
// sin preguntar.
//
// **Tres: una ventana, no la pantalla.** Se le pide a ScreenCaptureKit la ventana más grande de la
// app de la videollamada —y, si es un navegador, la que dice «Meet» en el título—. El correo, los
// documentos del consultor y la propia banda quedan fuera por construcción.

import CoreGraphics
import Foundation
import ScreenCaptureKit
import Vision

/// Códigos que ve Rust. Negativos = no se pudo. Espejo de las constantes de `pantalla/apple.rs`.
private enum CodigoDePantalla: Int32 {
  case sinPermiso = -1
  case sinVentana = -2
  case fallo = -3
  case cabeMal = -4
  case sinSoporte = -5
}

/// Lo que sale de pedirle un cuadro a ScreenCaptureKit.
private enum Captura: @unchecked Sendable {
  case imagen(CGImage)
  case sinVentana
  case fallo
}

private final class Caja<T>: @unchecked Sendable {
  var valor: T?
}

/// Espera a una tarea asíncrona desde un hilo de Rust. **Con techo**: ScreenCaptureKit no promete
/// contestar, y un hilo de la app colgado para siempre por una captura sería peor que un cuadro
/// perdido. El que llama es el vigía, que vuelve a intentarlo medio segundo después.
private func esperar<T>(segundos: Double, _ tarea: @escaping () async -> T?) -> T? {
  let semaforo = DispatchSemaphore(value: 0)
  let caja = Caja<T>()
  Task.detached {
    caja.valor = await tarea()
    semaforo.signal()
  }
  guard semaforo.wait(timeout: .now() + segundos) == .success else { return nil }
  return caja.valor
}

/// Captura UNA vez la ventana de la reunión y la deja en grises en `salida`.
///
/// - `bundle`: la app de la videollamada (`us.zoom.xos`, `com.google.Chrome`).
/// - `senales`: si es un navegador, lo que tiene que decir el título («google meet»), separado por
///   comas y en minúsculas. Vacío = cualquier ventana de esa app.
/// - `ladoMaximo`: el cuadro se escala para que ni el ancho ni el alto pasen de esto.
///
/// Devuelve 1 y escribe `ancho`/`alto`, o un código negativo.
@_cdecl("ag_pantalla_mirar")
public func agPantallaMirar(
  _ bundle: UnsafePointer<CChar>,
  _ senales: UnsafePointer<CChar>,
  _ ladoMaximo: Int32,
  _ salida: UnsafeMutablePointer<UInt8>,
  _ capacidad: Int32,
  _ ancho: UnsafeMutablePointer<Int32>,
  _ alto: UnsafeMutablePointer<Int32>
) -> Int32 {
  guard #available(macOS 14, *) else { return CodigoDePantalla.sinSoporte.rawValue }
  guard CGPreflightScreenCaptureAccess() else { return CodigoDePantalla.sinPermiso.rawValue }

  let app = String(cString: bundle)
  let pistas = String(cString: senales).split(separator: ",").map { $0.lowercased() }

  let captura = esperar(segundos: 3) { () async -> Captura? in
    guard
      let contenido = try? await SCShareableContent.excludingDesktopWindows(
        true, onScreenWindowsOnly: true)
    else { return .fallo }
    let candidatas = contenido.windows.filter { w in
      guard w.owningApplication?.bundleIdentifier == app, w.isOnScreen,
        w.frame.width >= 320, w.frame.height >= 200
      else { return false }
      if pistas.isEmpty { return true }
      let titulo = (w.title ?? "").lowercased()
      return pistas.contains { titulo.contains($0) }
    }
    guard
      let ventana = candidatas.max(by: {
        $0.frame.width * $0.frame.height < $1.frame.width * $1.frame.height
      })
    else { return .sinVentana }

    let escala = min(1.0, Double(ladoMaximo) / Double(max(ventana.frame.width, ventana.frame.height)))
    let config = SCStreamConfiguration()
    // Se pide ya del tamaño que se va a leer: ni un píxel de más cruza a la memoria.
    config.width = max(1, Int(ventana.frame.width * escala))
    config.height = max(1, Int(ventana.frame.height * escala))
    config.showsCursor = false
    config.capturesAudio = false
    let filtro = SCContentFilter(desktopIndependentWindow: ventana)
    guard
      let imagen = try? await SCScreenshotManager.captureImage(
        contentFilter: filtro, configuration: config)
    else { return .fallo }
    return .imagen(imagen)
  }

  let cg: CGImage
  switch captura {
  case .none, .fallo: return CodigoDePantalla.fallo.rawValue  // .none = no contestó a tiempo
  case .sinVentana: return CodigoDePantalla.sinVentana.rawValue
  case .imagen(let i): cg = i
  }

  let w = cg.width
  let h = cg.height
  guard w * h <= Int(capacidad) else { return CodigoDePantalla.cabeMal.rawValue }
  // Se pinta DIRECTAMENTE en el búfer de Rust, en grises de 8 bits: ninguna copia intermedia.
  guard
    let lienzo = CGContext(
      data: salida, width: w, height: h, bitsPerComponent: 8, bytesPerRow: w,
      space: CGColorSpaceCreateDeviceGray(), bitmapInfo: CGImageAlphaInfo.none.rawValue)
  else { return CodigoDePantalla.fallo.rawValue }
  lienzo.draw(cg, in: CGRect(x: 0, y: 0, width: w, height: h))
  ancho.pointee = Int32(w)
  alto.pointee = Int32(h)
  return 1
}

/// Lee el texto de un cuadro en grises con Vision, en español e inglés.
///
/// Escribe en `salida` una línea por renglón: `confianza \t alto \t texto`, con el alto relativo al
/// del cuadro (0…1), que es lo que distingue un título. Devuelve cuántas líneas escribió, o un
/// código negativo. Si el texto no cabe, **no escribe a medias**: devuelve `cabeMal`.
@_cdecl("ag_pantalla_leer")
public func agPantallaLeer(
  _ gris: UnsafePointer<UInt8>,
  _ ancho: Int32,
  _ alto: Int32,
  _ salida: UnsafeMutablePointer<CChar>,
  _ capacidad: Int32
) -> Int32 {
  let w = Int(ancho)
  let h = Int(alto)
  guard w > 0, h > 0 else { return CodigoDePantalla.fallo.rawValue }
  // El `CGImage` se arma sobre los bytes de Rust sin copiarlos, y muere al salir de la función.
  guard
    let proveedor = CGDataProvider(
      dataInfo: nil, data: gris, size: w * h, releaseData: { _, _, _ in }),
    let imagen = CGImage(
      width: w, height: h, bitsPerComponent: 8, bitsPerPixel: 8, bytesPerRow: w,
      space: CGColorSpaceCreateDeviceGray(), bitmapInfo: CGBitmapInfo(rawValue: 0),
      provider: proveedor, decode: nil, shouldInterpolate: false, intent: .defaultIntent)
  else { return CodigoDePantalla.fallo.rawValue }

  let pedido = VNRecognizeTextRequest()
  pedido.recognitionLevel = .accurate
  pedido.recognitionLanguages = ["es-ES", "en-US"]
  pedido.usesLanguageCorrection = true
  do {
    try VNImageRequestHandler(cgImage: imagen, options: [:]).perform([pedido])
  } catch {
    return CodigoDePantalla.fallo.rawValue
  }

  var texto = ""
  var lineas: Int32 = 0
  for observacion in pedido.results ?? [] {
    guard let mejor = observacion.topCandidates(1).first else { continue }
    // Un salto de línea dentro de un renglón rompería el formato; se aplana.
    let limpio = mejor.string.replacingOccurrences(of: "\n", with: " ")
    texto += "\(mejor.confidence)\t\(observacion.boundingBox.height)\t\(limpio)\n"
    lineas += 1
  }
  let bytes = Array(texto.utf8)
  guard bytes.count < Int(capacidad) else { return CodigoDePantalla.cabeMal.rawValue }
  bytes.withUnsafeBufferPointer { origen in
    salida.withMemoryRebound(to: UInt8.self, capacity: bytes.count + 1) { destino in
      destino.update(from: origen.baseAddress!, count: bytes.count)
      destino[bytes.count] = 0
    }
  }
  return lineas
}
