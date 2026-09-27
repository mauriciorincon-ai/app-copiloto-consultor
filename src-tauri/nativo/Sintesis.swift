// La síntesis (C7) con el MODELO DEL SISTEMA — Apple Foundation Models, dentro del Mac.
//
// ADR 010 «síntesis, código primero» y ADR 011 «proveedores del modelo y minimización». Este
// archivo es el proveedor (a): no abre red, no escribe nada, y lo que recibe —el turno del cliente
// y tres fichas— lo suelta al terminar. La salida va con **generación guiada** (un esquema
// dinámico): el modelo no puede devolver otra forma que la del esquema, y aun así Rust la vuelve a
// validar (`sintesis::fundar`), porque el grounding —que la fuente sea una de las fichas dadas— es
// de la app, no del modelo.

import Foundation
import FoundationModels

private enum CodigoDeSintesis: Int32 {
  case appleIntelligenceApagado = -1
  case macNoCompatible = -2
  case modeloDescargandose = -3
  case noDisponible = -4
  case fallo = -5
  case cabeMal = -6
  case tarde = -7
}

/// El esquema de la salida, **sin macros**: la macro de Foundation Models necesita el compilador de
/// Xcode y este puente se compila con `swiftc` a secas (el mismo que la CI). El esquema dinámico
/// hace lo mismo: el modelo no puede devolver otra forma, y `fuente` y `confianza` solo pueden ser
/// uno de sus valores.
private func esquemaDeLaSugerencia() throws -> GenerationSchema {
  typealias E = DynamicGenerationSchema
  return try GenerationSchema(
    root: E(
      name: "Sugerencia",
      properties: [
        E.Property(name: "titular", description: "Up to 8 words.", schema: E(type: String.self)),
        E.Property(
          name: "linea", description: "One sentence, up to 30 words, using only what the cards say.",
          schema: E(type: String.self)),
        E.Property(
          name: "fuente", description: "The id of the card you used.",
          schema: E(name: "Fuente", anyOf: ["F1", "F2", "F3"])),
        E.Property(
          name: "confianza", description: "baja when no card really answers the question.",
          schema: E(name: "Confianza", anyOf: ["alta", "media", "baja"])),
      ]),
    dependencies: [])
}

private final class CajaDeSintesis: @unchecked Sendable {
  var texto: String?
  var fallo = false
}

/// ¿Puede redactar el modelo del sistema? 0 = sí; si no, el motivo.
@_cdecl("ag_sintesis_disponible")
public func agSintesisDisponible() -> Int32 {
  switch SystemLanguageModel.default.availability {
  case .available:
    return 0
  case .unavailable(.appleIntelligenceNotEnabled):
    return CodigoDeSintesis.appleIntelligenceApagado.rawValue
  case .unavailable(.deviceNotEligible):
    return CodigoDeSintesis.macNoCompatible.rawValue
  case .unavailable(.modelNotReady):
    return CodigoDeSintesis.modeloDescargandose.rawValue
  default:
    return CodigoDeSintesis.noDisponible.rawValue
  }
}

/// Redacta UNA sugerencia y la escribe como JSON en `salida`. Devuelve los bytes escritos o un
/// código negativo. `segundos` es el techo: pasado, se devuelve `tarde` y lo que llegue se tira.
@_cdecl("ag_sintesis_redactar")
public func agSintesisRedactar(
  _ instrucciones: UnsafePointer<CChar>,
  _ texto: UnsafePointer<CChar>,
  _ segundos: Double,
  _ salida: UnsafeMutablePointer<CChar>,
  _ capacidad: Int32
) -> Int32 {
  guard case .available = SystemLanguageModel.default.availability else {
    return agSintesisDisponible()
  }
  let (i, t) = (String(cString: instrucciones), String(cString: texto))
  let caja = CajaDeSintesis()
  let semaforo = DispatchSemaphore(value: 0)
  Task.detached {
    do {
      let sesion = LanguageModelSession(instructions: i)
      let r = try await sesion.respond(to: t, schema: try esquemaDeLaSugerencia())
      var campos: [String: String] = [:]
      for k in ["titular", "linea", "fuente", "confianza"] {
        campos[k] = try r.content.value(String.self, forProperty: k)
      }
      let datos = try JSONSerialization.data(withJSONObject: campos)
      caja.texto = String(data: datos, encoding: .utf8)
    } catch {
      caja.fallo = true
    }
    semaforo.signal()
  }
  guard semaforo.wait(timeout: .now() + segundos) == .success else {
    return CodigoDeSintesis.tarde.rawValue
  }
  guard !caja.fallo, let json = caja.texto else { return CodigoDeSintesis.fallo.rawValue }
  let bytes = Array(json.utf8)
  guard bytes.count < Int(capacidad) else { return CodigoDeSintesis.cabeMal.rawValue }
  bytes.withUnsafeBufferPointer { b in
    b.baseAddress!.withMemoryRebound(to: CChar.self, capacity: bytes.count) {
      salida.update(from: $0, count: bytes.count)
    }
  }
  salida[bytes.count] = 0
  return Int32(bytes.count)
}
