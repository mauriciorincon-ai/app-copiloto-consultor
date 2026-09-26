// Habla.swift — el puente hacia `AVSpeechSynthesizer`: **la voz que SALE**.
//
// **MÓDULO PROTEGIDO.** Vive bajo la misma regla que `Transcriptor.swift`, y aquí la regla tiene
// un filo propio: `AVSpeechSynthesizer` trae `write(_:toBufferCallback:)`, que entrega la voz como
// búferes de audio — es decir, una manera de **convertir la ficha del usuario en un archivo**.
// Este archivo no la usa y `pnpm verify:ephemeral` barre esta carpeta entera. Un WAV con la
// evidencia del consultor leída en voz alta sería exactamente lo que la promesa de esta app niega.
//
// **Por qué un puente y no una FFI directa.** Por lo mismo que el transcriptor: `AVSpeechUtterance`
// y el delegado del sintetizador son tipos de Objective-C envueltos en Swift, y el estado que hace
// falta —«¿está hablando ahora mismo?»— hay que mantenerlo con un delegado. Escribir eso con
// `objc_msgSend` a mano sería más `unsafe` que un archivo de Swift de cien líneas.
//
// **Las tres decisiones de forma de este archivo.**
//
// **Una: el sintetizador es UNO y vive lo que vive la app.** Crear uno por frase es el error
// clásico de esta API: el objeto se libera antes de que el sistema acabe de hablar y la voz se
// corta a mitad. Vive en una variable global, y por eso mismo `callar()` siempre tiene a quién
// callar.
//
// **Dos: hablar NO bloquea.** `speak` encola y vuelve. El lado de Rust se llama desde un comando
// de Tauri, y bloquear ahí dejaría la interfaz parada los seis segundos que dura una ficha. Quien
// quiera saber si terminó pregunta por `ag_habla_hablando`.
//
// **Tres: el «está hablando» NO se le pregunta al sintetizador.** `isSpeaking` es una propiedad de
// un objeto de AppKit que se toca desde el hilo principal; la pregunta llega desde un hilo de Rust
// cualquiera, y varias veces por segundo. Se responde con una bandera atómica que el delegado
// mueve. Es la misma frontera que el grifo de audio: el hilo que mide no comparte candado con el
// hilo que trabaja.

import AVFoundation
import Foundation

/// Códigos que ve Rust. Negativos = no se pudo. Comparten la disciplina de `Transcriptor.swift`:
/// un número no se le puede enseñar a nadie, así que Rust los traduce a motivos en español.
private enum CodigoDeHabla: Int32 {
  case sinVozParaEseIdioma = -1
  case textoVacio = -2
  case idiomaInvalido = -3
}

/// **La bandera de «está hablando», y por qué es un `NSLock` y no un `Bool` a secas.**
///
/// La escriben el hilo principal (el delegado del sintetizador) y la leen los hilos de Rust. Un
/// `Bool` global compartido entre hilos es una carrera de datos, y en Swift 6 además no compila
/// sin mentirle al compilador. Un candado que se coge para leer un booleano cuesta nanosegundos y
/// no hace falta razonar sobre él.
private final class Bandera: @unchecked Sendable {
  private let candado = NSLock()
  private var valor = false

  var puesta: Bool {
    candado.lock()
    defer { candado.unlock() }
    return valor
  }

  func poner(_ nuevo: Bool) {
    candado.lock()
    valor = nuevo
    candado.unlock()
  }
}

private let hablando = Bandera()

/// El delegado que mueve la bandera. Las tres devoluciones que importan son las tres formas en que
/// una frase deja de sonar: terminó, la cancelaron, o nunca empezó porque el sistema la descartó.
private final class Delegado: NSObject, AVSpeechSynthesizerDelegate, @unchecked Sendable {
  func speechSynthesizer(_: AVSpeechSynthesizer, didStart _: AVSpeechUtterance) {
    hablando.poner(true)
  }

  func speechSynthesizer(_: AVSpeechSynthesizer, didFinish _: AVSpeechUtterance) {
    hablando.poner(false)
  }

  func speechSynthesizer(_: AVSpeechSynthesizer, didCancel _: AVSpeechUtterance) {
    hablando.poner(false)
  }
}

private let delegado = Delegado()

/// **El sintetizador, uno y vivo.** Se crea la primera vez que alguien lo pide y no se suelta: un
/// `AVSpeechSynthesizer` liberado a mitad de una frase deja la frase a medias.
private let sintetizador: AVSpeechSynthesizer = {
  let s = AVSpeechSynthesizer()
  s.delegate = delegado
  return s
}()

/// La voz del sistema para un idioma, si hay alguna.
///
/// `AVSpeechSynthesisVoice(language:)` **se inventa un respaldo**: pídele «xx-ZZ» y te puede
/// devolver la voz por defecto del Mac, que hablaría la ficha en inglés creyendo que es español.
/// Por eso no se usa su resultado a ciegas: se busca en `speechVoices()` una voz cuyo idioma
/// empiece por el que se pidió —«es» acepta «es-ES» y «es-MX», que es lo que el usuario espera— y
/// si no hay ninguna, no hay voz. Preferir la del idioma exacto y caer en la de la misma lengua es
/// lo que hace que un Mac con Paulina (es-MX) lea español en vez de callarse.
private func vozPara(_ idioma: String) -> AVSpeechSynthesisVoice? {
  let pedido = idioma.replacingOccurrences(of: "_", with: "-").lowercased()
  let lengua = String(pedido.prefix(2))
  let todas = AVSpeechSynthesisVoice.speechVoices()
  if let exacta = todas.first(where: { $0.language.lowercased() == pedido }) { return exacta }
  return todas.first(where: { $0.language.lowercased().hasPrefix(lengua + "-") })
}

/// ¿Hay voz en este Mac para este idioma? `1` sí, `0` no.
@_cdecl("ag_habla_voz")
public func agHablaVoz(_ idioma: UnsafePointer<CChar>) -> Int32 {
  let nombre = String(cString: idioma)
  guard !nombre.isEmpty else { return CodigoDeHabla.idiomaInvalido.rawValue }
  return vozPara(nombre) == nil ? 0 : 1
}

/// **Di esto.** Encola la frase y vuelve enseguida; el sonido sigue después.
///
/// Devuelve el número de caracteres que se pusieron a hablar, o un código negativo. Que devuelva
/// un número positivo **no** significa que el usuario lo haya oído: significa que el sistema lo
/// aceptó. Lo primero no se puede saber desde aquí y no se finge.
@_cdecl("ag_habla_decir")
public func agHablaDecir(_ idioma: UnsafePointer<CChar>, _ texto: UnsafePointer<CChar>) -> Int32 {
  let nombre = String(cString: idioma)
  let frase = String(cString: texto).trimmingCharacters(in: .whitespacesAndNewlines)
  guard !frase.isEmpty else { return CodigoDeHabla.textoVacio.rawValue }
  guard let voz = vozPara(nombre) else { return CodigoDeHabla.sinVozParaEseIdioma.rawValue }

  let frasePara = AVSpeechUtterance(string: frase)
  frasePara.voice = voz
  // El sintetizador vive en el hilo principal porque es un objeto de AppKit, y esta llamada llega
  // desde un hilo de Rust. `async` y no `sync` a propósito: un `sync` desde el hilo principal —que
  // puede pasar, si algún día esto se llama desde un menú— se bloquearía consigo mismo.
  //
  // La bandera se pone AQUÍ y no solo en el delegado: entre encolar y que el sistema avise hay
  // unos milisegundos, y en ese hueco `hablando` diría «no» justo después de que alguien pidiera
  // hablar. El disparador automático mira esa bandera para no interrumpirse, así que ese hueco es
  // exactamente el que produciría dos voces a la vez.
  hablando.poner(true)
  DispatchQueue.main.async {
    sintetizador.speak(frasePara)
  }
  return Int32(frase.count)
}

/// **Cállate.** Corta lo que esté diciendo y tira lo que quede en la cola. `⎋` acaba aquí.
///
/// `.immediate` y no `.word`: la tecla existe para cuando el cliente vuelve a hablar, y esperar a
/// que termine la palabra es medio segundo de la app hablando encima de él.
@_cdecl("ag_habla_callar")
public func agHablaCallar() -> Int32 {
  hablando.poner(false)
  DispatchQueue.main.async {
    sintetizador.stopSpeaking(at: .immediate)
  }
  return 0
}

/// ¿Está diciendo algo ahora mismo? `1` sí, `0` no.
@_cdecl("ag_habla_hablando")
public func agHablaHablando() -> Int32 {
  hablando.puesta ? 1 : 0
}
