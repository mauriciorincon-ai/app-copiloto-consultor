// Fuera de las copias de Time Machine (ADR 016, enmienda 2; auditoría del S3, M2). La bandeja promete
// morir en su ventana —3 h de fábrica, 24 h como mucho—, y una copia horaria de Time Machine la haría
// vivir semanas en el disco de copias. Aquí se marca su carpeta con el atributo estándar de macOS
// («excluido de las copias»). No pide permiso, ni contraseña, ni enseña ningún aviso.
//
// Sin red. Toca el disco en una sola línea —el atributo de la carpeta que le pasa Rust—, marcada.

import Foundation

/// Marca `ruta` fuera de las copias de Time Machine. 0 = hecho · -1 = no se pudo.
@_cdecl("ag_fuera_de_las_copias")
public func agFueraDeLasCopias(_ ruta: UnsafePointer<CChar>) -> Int32 {
  var url = URL(fileURLWithPath: String(cString: ruta), isDirectory: true)
  var valores = URLResourceValues()
  valores.isExcludedFromBackup = true
  do {
    try url.setResourceValues(valores) // verify-ephemeral:allow — ADR 016, enmienda 2: solo el atributo «fuera de las copias» de la carpeta de la bandeja
    return 0
  } catch {
    return -1
  }
}
