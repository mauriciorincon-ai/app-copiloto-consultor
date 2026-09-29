// Los secretos de la app, en el Llavero de macOS del usuario — jamás en un archivo (regla de la
// casa, ADR 011 y ADR 015). Tres servicios, uno por dueño: «Angel Ghost · API» (las claves de los
// proveedores externos), «Angel Ghost · notas» (la llave de tus notas cifradas) y «Angel Ghost ·
// puerta» (el token de la puerta local). **Hoy van al llavero de inicio de sesión**, el de archivo
// (`login.keychain-db`): se abre con tu sesión y no se sincroniza con iCloud, pero viaja con tus
// copias de Time Machine y con el Asistente de migración, protegido por tu contraseña (ADR 015,
// enmienda 2; auditoría del S3, A2). Para saber SI hay secreto se piden los atributos, nunca el
// secreto (`ag_llavero_hay`; auditoría del S2, B1).

import Foundation
import Security

private func consulta(_ servicio: String, _ cuenta: String) -> [String: Any] {
  [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: servicio,
    kSecAttrAccount as String: cuenta,
  ]
}

/// Guarda (o reemplaza) un secreto. 0 = bien; si no, el `OSStatus`.
@_cdecl("ag_llavero_guardar")
public func agLlaveroGuardar(
  _ servicio: UnsafePointer<CChar>, _ cuenta: UnsafePointer<CChar>, _ clave: UnsafePointer<CChar>
) -> Int32 {
  let s = String(cString: servicio)
  let c = String(cString: cuenta)
  SecItemDelete(consulta(s, c) as CFDictionary)
  var q = consulta(s, c)
  q[kSecValueData as String] = Data(String(cString: clave).utf8)
  // Solo opera en el llavero de protección de datos, que pide la app firmada con su grupo de
  // llaveros (G-Release, H2). En el de archivo, donde va hoy, macOS lo ignora (TN3137): se deja
  // escrito para el día de la firma, cuando se active el otro llavero (ADR 015, enmienda 2).
  q[kSecAttrAccessible as String] = kSecAttrAccessibleWhenUnlockedThisDeviceOnly
  return SecItemAdd(q as CFDictionary, nil)
}

/// Lee un secreto: los bytes escritos en `salida`, 0 si no hay, o un negativo.
@_cdecl("ag_llavero_leer")
public func agLlaveroLeer(
  _ servicio: UnsafePointer<CChar>, _ cuenta: UnsafePointer<CChar>,
  _ salida: UnsafeMutablePointer<UInt8>, _ capacidad: Int32
) -> Int32 {
  var q = consulta(String(cString: servicio), String(cString: cuenta))
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

/// ¿Hay secreto? 1 = sí, 0 = no, negativo = el Llavero no contestó. Pide solo los atributos: el
/// secreto no sale del Llavero para contestar esto.
@_cdecl("ag_llavero_hay")
public func agLlaveroHay(_ servicio: UnsafePointer<CChar>, _ cuenta: UnsafePointer<CChar>) -> Int32 {
  var q = consulta(String(cString: servicio), String(cString: cuenta))
  q[kSecReturnAttributes as String] = true
  q[kSecMatchLimit as String] = kSecMatchLimitOne
  var resultado: AnyObject?
  let estado = SecItemCopyMatching(q as CFDictionary, &resultado)
  if estado == errSecItemNotFound { return 0 }
  return estado == errSecSuccess ? 1 : -1
}

/// Borra un secreto. 0 = bien (también si no había).
@_cdecl("ag_llavero_borrar")
public func agLlaveroBorrar(_ servicio: UnsafePointer<CChar>, _ cuenta: UnsafePointer<CChar>) -> Int32 {
  let estado = SecItemDelete(consulta(String(cString: servicio), String(cString: cuenta)) as CFDictionary)
  return estado == errSecItemNotFound ? 0 : estado
}
