// LA ÚNICA PUERTA DE LA APP A LA RED — el proveedor externo de la síntesis (C7), opt-in, con la
// clave del usuario. ADR 011 «proveedores del modelo y minimización».
//
// Lo que pasa por aquí ya viene **anonimizado en el Mac** (`sintesis::anonimo`) y minimizado: el
// último turno del cliente y tres fichas cortas. Nunca audio, pantalla ni documentos: ese camino no
// existe. Y la sesión es **efímera** —sin caché, sin cookies, sin credenciales guardadas—: una
// sesión por defecto del sistema escribe las respuestas en la caché del disco, y aquí la
// respuesta lleva texto del cliente.
//
// Rust cuenta los bytes (`red::registrar_salida`) y cierra el techo de tiempo; esto solo envía.

import Foundation

private final class CajaDeRed: @unchecked Sendable {
  var datos: Data?
  var estado: Int32 = 0
}

/// POST de un JSON. `cabeceras` va como líneas «Nombre: valor». Devuelve los bytes de la respuesta
/// escritos en `salida` (y el estado HTTP en `estado`), o un negativo: -1 fallo de red, -2 tarde,
/// -3 no cabe, -4 URL inválida (solo https).
@_cdecl("ag_red_post")
public func agRedPost(
  _ url: UnsafePointer<CChar>,
  _ cabeceras: UnsafePointer<CChar>,
  _ cuerpo: UnsafePointer<UInt8>,
  _ largo: Int32,
  _ segundos: Double,
  _ salida: UnsafeMutablePointer<UInt8>,
  _ capacidad: Int32,
  _ estado: UnsafeMutablePointer<Int32>
) -> Int32 {
  guard let destino = URL(string: String(cString: url)), destino.scheme == "https" else { return -4 }
  var peticion = URLRequest(url: destino, timeoutInterval: segundos)
  peticion.httpMethod = "POST"
  for linea in String(cString: cabeceras).split(separator: "\n") {
    if let dos = linea.firstIndex(of: ":") {
      let nombre = linea[..<dos].trimmingCharacters(in: .whitespaces)
      let valor = linea[linea.index(after: dos)...].trimmingCharacters(in: .whitespaces)
      peticion.setValue(valor, forHTTPHeaderField: nombre)
    }
  }
  peticion.httpBody = Data(bytes: cuerpo, count: Int(largo))
  let sesion = URLSession(configuration: .ephemeral) // verify-ephemeral:allow — ADR 011 «proveedores del modelo y minimización»: la única puerta, opt-in, efímera
  let caja = CajaDeRed()
  let semaforo = DispatchSemaphore(value: 0)
  sesion.dataTask(with: peticion) { datos, respuesta, _ in
    caja.datos = datos
    caja.estado = Int32((respuesta as? HTTPURLResponse)?.statusCode ?? 0)
    semaforo.signal()
  }.resume()
  defer { sesion.invalidateAndCancel() }
  guard semaforo.wait(timeout: .now() + segundos) == .success else { return -2 }
  estado.pointee = caja.estado
  guard let datos = caja.datos else { return -1 }
  guard datos.count <= Int(capacidad) else { return -3 }
  datos.copyBytes(to: salida, count: datos.count)
  return Int32(datos.count)
}
