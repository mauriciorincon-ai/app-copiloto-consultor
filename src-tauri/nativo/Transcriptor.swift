// Transcriptor.swift — el puente hacia `SpeechAnalyzer` de macOS 26.
//
// **MÓDULO PROTEGIDO.** Vive bajo la misma regla que `src/stt/`: memoria y nada más.
// `pnpm verify:ephemeral` barre este archivo igual que los de Rust, y con la misma lista de API
// prohibida traducida a Swift. La única línea que roza la red está marcada y justificada abajo.
//
// **Por qué existe este archivo y no una llamada directa desde Rust.** La Accessibility API del
// acople se maneja desde Rust con externs declarados a mano, porque es C. `SpeechAnalyzer` no lo
// es: es un `actor` de Swift con secuencias asíncronas y tipos que solo existen en el runtime de
// Swift. No hay forma de mandarle un mensaje de Objective-C. O hay puente, o no hay motor — y el
// motor, medido, transcribe seis segundos de audio en ochenta y cuatro milisegundos.
//
// **Por qué el puente transcribe TURNOS COMPLETOS y no un flujo continuo.** Porque el fin de turno
// ya lo decide `voz::turno` con aritmética que se puede leer y probar. Dejar que el motor decida
// dónde acaba una frase sería devolverle a una caja cerrada la decisión más delicada del sprint, y
// además la más barata de hacer bien: a 70× tiempo real, transcribir el turno entero cuando ya se
// sabe que terminó cuesta menos que mantener un analizador vivo toda la reunión.

import AVFoundation
import Foundation
import Speech

/// Códigos que ve Rust. Negativos = no se pudo; el llamante decide qué contarle al usuario.
private enum Codigo: Int32, Error {
  case sinSoporte = -1        // este macOS no trae SpeechAnalyzer
  case idiomaDesconocido = -2 // el motor no conoce ese idioma
  case sinModelo = -3         // el idioma se soporta pero su modelo no está instalado
  case falloDelMotor = -4
  case audioInvalido = -5
  case cabeMal = -6           // la transcripción no cabe en el buffer que dio Rust
}

/// Estado de un idioma. Lo consulta la pantalla de Idioma para no prometer lo que no hay.
private enum Estado: Int32 {
  case noSoportado = 0
  case soportadoSinInstalar = 1
  case instalado = 2
}

/// Corre una tarea asíncrona y espera. El puente es síncrono porque el lado de Rust llama desde un
/// hilo suyo que ya está fuera del camino del audio: bloquear ahí no le quita muestras a nadie.
private func esperar<T>(_ tarea: @escaping () async -> T) -> T {
  let semaforo = DispatchSemaphore(value: 0)
  let caja = Caja<T>()
  Task.detached {
    caja.valor = await tarea()
    semaforo.signal()
  }
  semaforo.wait()
  return caja.valor!
}

private final class Caja<T>: @unchecked Sendable {
  var valor: T?
}

/// Reserva el idioma en el inventario de macOS.
///
/// **Esto no estaba en ningún plan y se descubrió midiendo.** `AssetInventory.status` devolvía
/// «soportado, sin instalar» para un español que `installedLocales` listaba como instalado — el
/// modelo estaba en el Mac, pero no *para nosotros*. macOS reparte los modelos de reconocimiento
/// por reserva: cada app declara con cuáles va a trabajar y, hasta que lo hace, el activo
/// instalado no le cuenta. El techo es de cinco idiomas por app (`maximumReservedLocales`), y ese
/// cinco es un hecho de producto, no un detalle.
///
/// **Y por qué reservar NO ocurre al preguntar, que es donde estaba al principio.** La primera
/// versión reservaba dentro de `ag_stt_estado`, y arrancar la app dejaba este rastro en el log:
///
///     [stt] motor «apple-speechanalyzer» · 30 idiomas soportados · techo 5
///     [stt] modelos instalados: en-AU, es-ES
///
/// Dos, de treinta, y elegidos por el orden en que la pantalla preguntó. Es decir: **enumerar los
/// idiomas para pintar una lista se estaba gastando los cinco cupos del usuario**, en silencio y
/// sin que nadie lo pidiera. Preguntar no puede cambiar el sistema. Ahora se reserva solo al
/// instalar y al transcribir, que son las dos veces que hace falta de verdad.
@available(macOS 26, *)
private func reservar(_ t: SpeechTranscriber) async {
  guard let loc = t.selectedLocales.first else { return }
  let yaEstan = await AssetInventory.reservedLocales
  if yaEstan.contains(where: { $0.identifier == loc.identifier }) { return }
  try? await AssetInventory.reserve(locale: loc)
}

@available(macOS 26, *)
private func transcriptorDe(_ identificador: String) async -> SpeechTranscriber? {
  guard let loc = await SpeechTranscriber.supportedLocale(equivalentTo: Locale(identifier: identificador))
  else { return nil }
  return SpeechTranscriber(locale: loc, preset: .transcription)
}

// ---------------------------------------------------------------------------------------------
// Lo que Rust puede llamar
// ---------------------------------------------------------------------------------------------

/// ¿Existe el motor en este Mac? 1 sí, 0 no.
@_cdecl("ag_stt_disponible")
public func agSttDisponible() -> Int32 {
  if #available(macOS 26, *) { return SpeechTranscriber.isAvailable ? 1 : 0 }
  return 0
}

/// Cuántos idiomas puede tener listos esta app a la vez. macOS lo impone; la pantalla de Idioma lo
/// enseña en vez de dejar que el sexto falle sin explicación.
@_cdecl("ag_stt_techo_de_idiomas")
public func agSttTechoDeIdiomas() -> Int32 {
  guard #available(macOS 26, *) else { return 0 }
  return Int32(AssetInventory.maximumReservedLocales)
}

/// Los idiomas que este Mac sabe transcribir, separados por comas.
///
/// Se pregunta en vez de llevar una lista escrita: la lista de macOS cambia entre versiones, y una
/// lista nuestra desfasada haría que la pantalla de Idioma ofreciera un idioma que no existe o
/// escondiera uno que sí. Lo que el Mac sabe hacer lo dice el Mac.
@_cdecl("ag_stt_idiomas")
public func agSttIdiomas(_ salida: UnsafeMutablePointer<CChar>, _ capacidad: Int32) -> Int32 {
  guard #available(macOS 26, *) else { return Codigo.sinSoporte.rawValue }
  let lista = esperar { await SpeechTranscriber.supportedLocales.map { $0.identifier } }
  let bytes = Array(lista.sorted().joined(separator: ",").utf8)
  guard bytes.count < Int(capacidad) else { return Codigo.cabeMal.rawValue }
  bytes.withUnsafeBufferPointer { origen in
    salida.withMemoryRebound(to: UInt8.self, capacity: Int(capacidad)) { destino in
      destino.update(from: origen.baseAddress!, count: bytes.count)
      destino[bytes.count] = 0
    }
  }
  return Int32(bytes.count)
}

/// ¿En qué estado está un idioma? Ver [`Estado`]; negativo si no hay motor.
@_cdecl("ag_stt_estado")
public func agSttEstado(_ idioma: UnsafePointer<CChar>) -> Int32 {
  guard #available(macOS 26, *) else { return Codigo.sinSoporte.rawValue }
  let nombre = String(cString: idioma)
  return esperar {
    guard let t = await transcriptorDe(nombre) else { return Estado.noSoportado.rawValue }
    // Se pregunta a `installedLocales`, que dice qué hay en el MAC, y no a `status`, que dice qué
    // hay **para esta app** y solo contesta que sí después de reservar. Para la pantalla de Idioma
    // la verdad útil es la primera: el modelo está descargado o no lo está.
    let instalados = await SpeechTranscriber.installedLocales
    let mio = t.selectedLocales.first?.identifier
    let esta = instalados.contains { $0.identifier == mio }
    return esta ? Estado.instalado.rawValue : Estado.soportadoSinInstalar.rawValue
  }
}

/// Instala el modelo de un idioma. **Bloquea y usa la red**: macOS descarga el activo.
///
/// Lo llama la pantalla de Idioma cuando el usuario lo pide, jamás la app por su cuenta. Es la
/// única puerta a la red que tiene un módulo protegido de esta app, y por eso está aquí sola, con
/// nombre propio y con la marca que el barrido exige.
@_cdecl("ag_stt_instalar")
public func agSttInstalar(_ idioma: UnsafePointer<CChar>) -> Int32 {
  guard #available(macOS 26, *) else { return Codigo.sinSoporte.rawValue }
  let nombre = String(cString: idioma)
  return esperar {
    guard let t = await transcriptorDe(nombre) else { return Codigo.idiomaDesconocido.rawValue }
    do {
      await reservar(t)
      // Las dos líneas de abajo son la ÚNICA puerta a la red de un módulo protegido de esta app,
      // y llevan su marca porque el barrido las prohíbe por defecto: es macOS quien descarga su
      // propio modelo de reconocimiento, a petición explícita del usuario desde la pantalla de
      // Idioma. No sale audio ni texto — solo entra el modelo.
      if let peticion = try await AssetInventory.assetInstallationRequest(supporting: [t]) { // verify-ephemeral:allow — ADR 006 «el STT local y su modelo»
        try await peticion.downloadAndInstall() // verify-ephemeral:allow — ADR 006 «el STT local y su modelo»
      }
      return Estado.instalado.rawValue
    } catch {
      return Codigo.falloDelMotor.rawValue
    }
  }
}

/// Transcribe un turno. Devuelve cuántos bytes UTF-8 escribió en `salida`, o un [`Codigo`].
///
/// `muestras` son f32 mono a `hz`. Nada de esto toca el disco: el audio llega por puntero desde el
/// anillo de Rust, se convierte en memoria al formato que pide el motor, y el texto vuelve por el
/// buffer que Rust reservó.
@_cdecl("ag_stt_transcribir")
public func agSttTranscribir(
  _ idioma: UnsafePointer<CChar>,
  _ muestras: UnsafePointer<Float>,
  _ n: Int32,
  _ hz: Double,
  _ salida: UnsafeMutablePointer<CChar>,
  _ capacidad: Int32
) -> Int32 {
  guard #available(macOS 26, *) else { return Codigo.sinSoporte.rawValue }
  guard n > 0, hz > 0 else { return Codigo.audioInvalido.rawValue }
  let nombre = String(cString: idioma)
  let copia = Array(UnsafeBufferPointer(start: muestras, count: Int(n)))

  let resultado: Result<String, Codigo> = esperar {
    guard let t = await transcriptorDe(nombre) else { return .failure(.idiomaDesconocido) }
    await reservar(t)
    guard await AssetInventory.status(forModules: [t]) == .installed else { return .failure(.sinModelo) }
    guard let entrada = AVAudioFormat(commonFormat: .pcmFormatFloat32, sampleRate: hz, channels: 1, interleaved: false),
          let bufer = AVAudioPCMBuffer(pcmFormat: entrada, frameCapacity: AVAudioFrameCount(copia.count))
    else { return .failure(.audioInvalido) }
    bufer.frameLength = AVAudioFrameCount(copia.count)
    copia.withUnsafeBufferPointer { origen in
      bufer.floatChannelData!.pointee.update(from: origen.baseAddress!, count: copia.count)
    }

    // El motor elige el formato que le conviene; nosotros nos adaptamos. Convertir aquí, una vez,
    // es más barato que convertir cada bloque de 20 ms durante toda la reunión.
    let destinoDeseado = await SpeechAnalyzer.bestAvailableAudioFormat(compatibleWith: [t])
    let aEnviar: AVAudioPCMBuffer
    if let destino = destinoDeseado, destino != entrada {
      guard let conversor = AVAudioConverter(from: entrada, to: destino) else { return .failure(.audioInvalido) }
      let marcos = AVAudioFrameCount(Double(copia.count) * destino.sampleRate / entrada.sampleRate) + 1024
      guard let convertido = AVAudioPCMBuffer(pcmFormat: destino, frameCapacity: marcos) else {
        return .failure(.audioInvalido)
      }
      var entregado = false
      var fallo: NSError?
      conversor.convert(to: convertido, error: &fallo) { _, estado in
        if entregado { estado.pointee = .endOfStream; return nil }
        entregado = true
        estado.pointee = .haveData
        return bufer
      }
      if fallo != nil { return .failure(.audioInvalido) }
      aEnviar = convertido
    } else {
      aEnviar = bufer
    }

    let (flujo, grifo) = AsyncStream<AnalyzerInput>.makeStream()
    let analizador = SpeechAnalyzer(modules: [t])
    let recolector = Task { () -> String in
      var texto = ""
      for try await r in t.results where r.isFinal {
        texto += String(r.text.characters)
      }
      return texto
    }
    do {
      try await analizador.start(inputSequence: flujo)
      grifo.yield(AnalyzerInput(buffer: aEnviar))
      grifo.finish()
      try await analizador.finalizeAndFinishThroughEndOfInput()
      let texto = (try? await recolector.value) ?? ""
      return .success(texto.trimmingCharacters(in: .whitespacesAndNewlines))
    } catch {
      await analizador.cancelAndFinishNow()
      return .failure(.falloDelMotor)
    }
  }

  switch resultado {
  case .failure(let c):
    return c.rawValue
  case .success(let texto):
    let bytes = Array(texto.utf8)
    guard bytes.count < Int(capacidad) else { return Codigo.cabeMal.rawValue }
    bytes.withUnsafeBufferPointer { origen in
      salida.withMemoryRebound(to: UInt8.self, capacity: Int(capacidad)) { destino in
        destino.update(from: origen.baseAddress!, count: bytes.count)
        destino[bytes.count] = 0
      }
    }
    return Int32(bytes.count)
  }
}
