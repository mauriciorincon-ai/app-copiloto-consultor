// La clave del API del usuario, en SU Llavero de macOS — jamás en un archivo (regla de la casa y
// ADR 011). El Llavero la cifra con la cuenta del usuario; la app la lee solo en el instante de
// enviar una petición y no la guarda en ningún otro sitio.

import Foundation
import Security

private let servicio = "Angel Ghost · API"

private func consulta(_ cuenta: String) -> [String: Any] {
  [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: servicio,
    kSecAttrAccount as String: cuenta,
  ]
}

/// Guarda (o reemplaza) la clave de un proveedor. 0 = bien; si no, el `OSStatus`.
@_cdecl("ag_llavero_guardar")
public func agLlaveroGuardar(_ cuenta: UnsafePointer<CChar>, _ clave: UnsafePointer<CChar>) -> Int32 {
  let c = String(cString: cuenta)
  SecItemDelete(consulta(c) as CFDictionary)
  var q = consulta(c)
  q[kSecValueData as String] = Data(String(cString: clave).utf8)
  q[kSecAttrAccessible as String] = kSecAttrAccessibleWhenUnlockedThisDeviceOnly
  return SecItemAdd(q as CFDictionary, nil)
}

/// Lee la clave: los bytes escritos en `salida`, 0 si no hay, o un negativo.
@_cdecl("ag_llavero_leer")
public func agLlaveroLeer(
  _ cuenta: UnsafePointer<CChar>, _ salida: UnsafeMutablePointer<UInt8>, _ capacidad: Int32
) -> Int32 {
  var q = consulta(String(cString: cuenta))
  q[kSecReturnData as String] = true
  q[kSecMatchLimit as String] = kSecMatchLimitOne
  var resultado: AnyObject?
  let estado = SecItemCopyMatching(q as CFDictionary, &resultado)
  if estado == errSecItemNotFound { return 0 }
  guard estado == errSecSuccess, let datos = resultado as? Data else { return -1 }
  guard datos.count <= Int(capacidad) else { return -2 }
  datos.copyBytes(to: salida, count: datos.count)
  return Int32(datos.count)
}

/// Borra la clave de un proveedor. 0 = bien (también si no había).
@_cdecl("ag_llavero_borrar")
public func agLlaveroBorrar(_ cuenta: UnsafePointer<CChar>) -> Int32 {
  let estado = SecItemDelete(consulta(String(cString: cuenta)) as CFDictionary)
  return estado == errSecItemNotFound ? 0 : estado
}
