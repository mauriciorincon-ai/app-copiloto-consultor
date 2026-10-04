// El desbloqueo de tus notas guardadas (ADR 015 §5): Touch ID o la contraseña del Mac, para ABRIR o
// EXPORTAR una reunión guardada. Guardar no lo pide. La app lo recuerda hasta que sales de ella
// (`desbloqueo.rs`); aquí solo se pregunta.
//
// Sin disco ni red: LocalAuthentication contesta sí o no, y nada más cruza.

import Foundation
import LocalAuthentication

private final class CajaDelDesbloqueo: @unchecked Sendable {
  var resultado: Int32 = 0
}

/// Pide Touch ID o la contraseña del Mac con `razon` (lo que el diálogo dice debajo del nombre de la
/// app). 1 = desbloqueado · 0 = cancelado o fallido · -1 = este Mac no tiene con qué pedirlo.
///
/// Bloquea el hilo que llama hasta que el usuario contesta: lo llama un comando de Tauri, que corre
/// fuera del hilo principal, y el diálogo lo pinta el sistema.
@_cdecl("ag_desbloquear")
public func agDesbloquear(_ razon: UnsafePointer<CChar>) -> Int32 {
  let contexto = LAContext()
  var error: NSError?
  guard contexto.canEvaluatePolicy(.deviceOwnerAuthentication, error: &error) else { return -1 }
  let semaforo = DispatchSemaphore(value: 0)
  let caja = CajaDelDesbloqueo()
  contexto.evaluatePolicy(.deviceOwnerAuthentication, localizedReason: String(cString: razon)) { ok, _ in
    caja.resultado = ok ? 1 : 0
    semaforo.signal()
  }
  semaforo.wait()
  return caja.resultado
}
