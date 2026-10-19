import type { Idioma } from "./index";
import type { Diccionario } from "./es";

/**
 * **LO QUE CAMBIA DE `es.ts` Y `en.ts` EN WINDOWS** (sprint 007, ADR 022 enmienda 1).
 *
 * `es.ts` y `en.ts` están atados a la maqueta (gate `i18n-fiel-a-la-maqueta`) y hablan del Mac: «Verificada solo en
 * Meet · macOS 26.6.2», «Verificado en tu Mac». En un PC con Windows eso es una promesa de invisibilidad que nadie
 * verificó (regla dura 6) o un sistema que no es el suyo. Aquí viven, por la misma ruta que el diccionario, las
 * frases que Windows dice distinto; `useT()` las pone encima del diccionario solo en Windows y en macOS no cambia nada.
 *
 * Las teclas no se escriben aquí: el diccionario pinta `⌃⌥A` y `useT()` la cambia por la de Windows
 * (`conTeclasDeEsteSistema`). Lo que sigue es solo lo que dice «Mac», «macOS» o una verificación que no se hizo.
 *
 * **Estado: maquetados, no vistos.** Van al gate humano del hito, con su línea en el cierre.
 */
type Parcial<T> = {
  [K in keyof T]?: T[K] extends string ? string : Parcial<T[K]>;
};

/** Una parte del diccionario: una clave que no existe en `es.ts` no compila. */
export type Cambios = Parcial<Diccionario>;

/**
 * La sugerencia redactada en Windows (ADR 023, enmiendas 1 y 2): sin API, y con el modelo local que se instala en IA
 * (sprint 014). Los motivos del modelo del sistema del Mac no se dan en Windows; si alguno llegara, dice la verdad de aquí.
 */
const SIN_SUGERENCIA_ES =
  "En Windows la sugerencia la redacta el modelo local, dentro de tu PC: instálalo en esta pantalla. El API de pago salió de Windows. Sin el modelo, la ficha sigue siendo la respuesta.";
const SIN_SUGERENCIA_EN =
  "On Windows the suggestion is written by the local model, inside your PC: install it on this screen. The paid API is gone from Windows. Without the model, the card is still the answer.";

const ES: Cambios = {
  // La puerta local en Windows (sprint 012, ADR 022 enmienda 4): un named pipe y la llave en el Administrador de
  // credenciales, no un socket y el Llavero.
  puerta: {
    subResto:
      "para que Claude Code opere la app desde este mismo PC, con una llave en el Administrador de credenciales.",
    daselo:
      "Dáselo a Claude Code. Cada vez que abres la puerta, la app deja una llave nueva en el Administrador de credenciales y ghost la lee de ahí; al cerrarla, la borra. Mientras está abierta, cualquier programa de tu usuario podría leerla.",
    noAbre: {
      llavero: "El Administrador de credenciales no guardó la llave.",
      socket:
        "Windows no dejó crear la puerta: otro programa ya usa ese nombre.",
    },
    redPor: "solo escucha a este PC; el contador sigue igual",
  },
  banda: {
    // La banda no promete invisibilidad donde nadie la verificó (regla dura 6).
    sinVerificarCuando:
      "Sin verificar en Teams, Zoom ni Meet · se verifica en tu PC",
    sugerenciaEnTuMac: "sugerencia · en tu PC",
    radarTeMira: "Te está mirando un programa de tu PC",
    radarEnTuEquipo: "en tu PC, no en el del cliente",
  },
  cuaderno: {
    sesionSub:
      "Nada se enciende hasta que tú lo digas. Esto es lo que la app ve de tu propio PC.",
    // La promesa es graduada por plataforma (regla dura 6): en Windows se dice lo que se pidió, no que esté verificado.
    panelProtegido: "Panel con la protección pedida a Windows",
    proteccionDetalle:
      "Tu panel le pide a Windows que no salga en las capturas, pero en Teams, Zoom y Meet está sin verificar: se verifica con una videollamada real en tu PC. Mientras tanto, comparte una ventana, no la pantalla.",
    modeloDelSistema: "Modelo local",
    // Sprint 013: lo que Windows no tiene se dice «no por ahora», no «no disponible». Desde el sprint 014 solo le toca a
    // «Enriquecer el banco»: la sugerencia la redacta el modelo local.
    noDisponible: "no por ahora",
    iaSub:
      "En Windows todo corre en tu PC. Con el modelo local instalado, la sugerencia se redacta aquí mismo, sin red; sin él, la ficha es la respuesta.",
    siFallaQuedan: "La ficha es la respuesta",
    permisosTitulo: "Permisos de Windows",
    // La tarea de borrado es del Programador de tareas, no de «Ítems de inicio» de macOS (auditoría del S10, A1).
    noCorrio:
      "La tarea de borrado no corrió con la app cerrada: revisa el Programador de tareas (Task Scheduler)",
    // Las rutas de error: con el audio del sistema y la pantalla en Windows (S7), estos porqués SÍ se alcanzan allí
    // (auditoría del S7, A1). Ninguno manda a Permisos por algo que Windows no pide.
    funcionaEscucha: "Escucha las dos pistas y las transcribe en tu PC",
    escuchaAMedias: "Escucha y transcribe en tu PC —",
    porQueNoAbrio: {
      "sin-permiso-del-microfono":
        "Windows no dio permiso al micrófono. Concédelo en Ajustes → Privacidad y seguridad → Micrófono y vuelve a «Iniciar sesión».",
      "sin-permiso-del-audio":
        "Windows no dejó abrir el audio del sistema. Vuelve a «Iniciar sesión»; si se repite, revisa que el PC tenga una salida de audio por defecto.",
      "dispositivo-ocupado":
        "Otra app tiene el dispositivo de audio en modo exclusivo. Ciérrala y vuelve a «Iniciar sesión».",
      "no-dejo":
        "Windows no dejó abrir esta pista. Vuelve a «Iniciar sesión»; si se repite, comprueba la salida de audio por defecto o reinicia el PC.",
    },
    pantallaSinPermisoPor:
      "Windows no entregó la ventana de la reunión. La app lo vuelve a intentar sola.",
    pantallaNoPudoPor:
      "Windows no entregó la ventana de la reunión, o no tiene el reconocimiento de texto de tu idioma (Configuración → Hora e idioma → Idioma y región). La app lo vuelve a intentar sola.",
    porQueNoSeSabe: {
      "sin-salida": "Este PC no tiene una salida de audio por defecto.",
    },
    enTuLlavero:
      "se guarda en el Administrador de credenciales, nunca en un archivo",
    guardarEnLlavero: "Guardar en el Administrador de credenciales",
    permSistemaPara:
      "Lo que suena en tu PC: la voz del cliente. Sin bot en la reunión.",
    permAcoplePara:
      "Para que la banda no tape la llamada: la reunión se encoge y las dos conviven.",
    // En Windows, Teams y Zoom cuentan cuando están en una llamada, no por estar abiertos (sprint 008, S7 M1).
    sinReunionVoz:
      "Entra en tu llamada de Zoom, Teams o Meet y aparecerá aquí. Mientras tanto, prepara la reunión.",
    // IA en Windows (sprint 013, ADR 023 enmienda 1): sin API ni modelo local; cualquier motivo de macOS dice esa verdad.
    enTuMac: "en tu PC",
    // Sprint 010: el resto del diccionario del Mac que nombraba macOS, el Mac o un permiso de Apple, dicho para Windows.
    // Varios los pinta el componente con el texto de `windows.ts` (`modoDetalle`, `transcribeTuPc…`, `idiomasListos`); aquí
    // queda igual el diccionario entero, para que ninguna cadena del Mac sobreviva en él (gate `windows-sin-mac`) y para que
    // `windows-honestidad-y-manual` ate las copias a su fuente.
    concederEnMacos: "Conceder en Windows",
    modoDetalle:
      "La app abre una conexión solo cuando tú lo pides, una vez por modelo: al instalar el modelo de voz, 1,6 GB (0,5 GB en un PC ARM64), y al instalar el modelo local de la sugerencia, 1,1 GB; los dos se comprueban antes de usarse. En Windows no hay API: todo lo demás se queda en tu PC.",
    cincoIdiomas:
      "Dos idiomas listos a la vez, como mucho: los que la app ofrece.",
    idiomasListos:
      "idiomas listos a la vez, como mucho: los que la app ofrece.",
    transcribeTuMac: "Transcribe tu PC, no un servicio",
    transcribeTuMacDetalle:
      "El modelo de voz (whisper), dentro de tu equipo. Ningún audio sale para convertirse en texto. La única vez que toca la red es cuando pides instalar el modelo: lo baja Angel Ghost desde su origen y comprueba su huella antes de usarlo.",
    altavozExterno:
      "Por HDMI o DisplayPort el sonido sale por un altavoz: el micrófono va a oír al cliente y el modo solo audio no habla.",
    porQueNoSeVe: {
      "sin-accesibilidad":
        "La app no pudo ver las ventanas de tu navegador en este PC, y sin eso no sabe si tienes Meet abierto.",
    },
    dosPermisos:
      "En Windows, el audio del sistema y la pantalla no piden ningún permiso: solo el micrófono tiene el suyo, en Ajustes.",
    antesDeQueMacos: "El micrófono en Windows",
    fraseDeMacos:
      "La app no pide el permiso por su cuenta: lo concedes o lo quitas tú en Ajustes → Privacidad y seguridad → Micrófono.",
    porQueNoHayMotor: {
      "sin-transcriptor":
        "Este PC no tiene el modelo de voz. Instálalo en Idioma.",
      "no-contesta":
        "El modelo de voz no contestó. Vuelve a abrir la app; si se repite, reinicia el PC.",
    },
    porQueNoRedacta: {
      "apple-intelligence-apagado": SIN_SUGERENCIA_ES,
      "mac-no-compatible": SIN_SUGERENCIA_ES,
      "no-disponible": SIN_SUGERENCIA_ES,
      "modelo-descargandose": SIN_SUGERENCIA_ES,
      "sin-puente": SIN_SUGERENCIA_ES,
    },
    caracteresTachado:
      "caracteres. Tachado, lo que se reemplazó en tu PC antes de enviarlo.",
    // El radar en Sesión (sprint 008): mira este PC.
    vigilanciaTitulo: "Software invasivo corriendo en tu PC",
    vigilanciaSoloTuMac: "solo tu PC",
    presencial: {
      reunionEnEsteMac: "reunión en este PC",
      pcPregunta: "¿La videollamada está en este PC?",
      pcQueHacer:
        "Usa «reunión en este PC»: la app oye al cliente por el audio del sistema. Presencial es para una sala sin videollamada.",
    },
  },
  notas: {
    clienteNoLaVe: "con la protección pedida al compartir",
    // Time Machine es del Mac; en Windows no se promete qué copias de seguridad cubren la carpeta de la app.
    borrarDetalle:
      "Se borra ahora de la app y no vuelve. Si haces copias de seguridad de tu PC, esas copias la conservan hasta que caduquen.",
    lasTresPrimeras:
      "Las tres primeras murieron al cerrar; la bandeja, al vencer.",
    enTuLlavero: "en el Administrador de credenciales de tu usuario de Windows",
    mostrarEnFinder: "Mostrar en el Explorador de archivos",
    // Sprint 010: la bandeja con llave se abre con Windows Hello, no con Touch ID.
    paraLeerla:
      "Para leerla, Windows Hello (tu PIN, tu huella o tu cara), como para exportar una reunión. Una vez por sesión de la app.",
    abrirConTouchId: "Abrir con Windows Hello",
  },
  ensayo: {
    borrarDetalle:
      "Se borran ahora de la app y no vuelven. Si haces copias de seguridad de tu PC, esas copias los conservan hasta que caduquen.",
    microfonoSinPermiso:
      "Windows no dio permiso al micrófono. Concédelo en Ajustes → Privacidad y seguridad → Micrófono y vuelve a empezar el ensayo.",
    microfonoOtro:
      "Windows no dejó abrir el micrófono. Vuelve a empezar el ensayo; si se repite, reinicia el PC.",
    sinTranscripcion: "Este PC no sabe transcribir el idioma del ensayo",
    noSeSabeSiHayLlamadaQue:
      "La app no pudo ver si tu navegador está en una llamada. Ponte auriculares o cierra la llamada para ensayar.",
    vozSinReunion:
      "Sin reunión, la voz sale por donde suene tu PC; con una videollamada abierta, el ensayo no empieza por los altavoces del PC. Mientras habla, el micrófono no escucha.",
    // Sprint 013 (auditoría, A-2): en Windows nadie enriquece el banco, así que el ensayo no manda a encenderlo en IA. El
    // modelo local del sprint 014 redacta la sugerencia de la reunión, no el banco.
    enriquecerApagadoQue:
      "En Windows el banco de preguntas es el de las reglas; el acento de un modelo está «no por ahora».",
    porQueNoSumo: {
      "sin-proveedor": "en Windows el banco no se enriquece por ahora.",
    },
  },
  cliente: {
    panelProtegido: "El panel sigue con la protección pedida a Windows",
  },
};

const EN: Cambios = {
  puerta: {
    subResto:
      "so Claude Code can operate the app from this same PC, with a key in Credential Manager.",
    daselo:
      "Give it to Claude Code. Each time you open the door, the app leaves a new key in Credential Manager and ghost reads it from there; closing the door deletes it. While it is open, any program of your user could read it.",
    noAbre: {
      llavero: "Credential Manager did not store the key.",
      socket:
        "Windows did not let the door be created: another program already uses that name.",
    },
    redPor: "it listens only to this PC; the counter stays",
  },
  banda: {
    sinVerificarCuando:
      "Not verified in Teams, Zoom or Meet · you verify it on your PC",
    sugerenciaEnTuMac: "suggestion · on your PC",
    radarTeMira: "A program on your PC is watching you",
    radarEnTuEquipo: "on your PC, not the client’s",
  },
  cuaderno: {
    panelProtegido: "Panel with the protection requested from Windows",
    sesionSub:
      "Nothing turns on until you say so. This is what the app sees of your own PC.",
    proteccionDetalle:
      "Your panel asks Windows to stay out of screen captures, but in Teams, Zoom and Meet it is unverified: you verify it with a real video call on your PC. Until then, share a window, not the screen.",
    modeloDelSistema: "Local model",
    noDisponible: "not for now",
    iaSub:
      "On Windows everything runs on your PC. With the local model installed, the suggestion is written right here, offline; without it, the card is the answer.",
    siFallaQuedan: "The card is the answer",
    permisosTitulo: "Windows permissions",
    noCorrio:
      "The deletion task did not run while the app was closed: check Task Scheduler",
    funcionaEscucha:
      "It listens to both tracks and transcribes them on your PC",
    escuchaAMedias: "Listens and transcribes on your PC —",
    porQueNoAbrio: {
      "sin-permiso-del-microfono":
        "Windows did not grant the microphone. Grant it in Settings → Privacy & security → Microphone and press “Start session” again.",
      "sin-permiso-del-audio":
        "Windows would not open the system audio. Press “Start session” again; if it repeats, check that the PC has a default audio output.",
      "dispositivo-ocupado":
        "Another app holds the audio device in exclusive mode. Close it and press “Start session” again.",
      "no-dejo":
        "Windows would not open this track. Press “Start session” again; if it repeats, check the default audio output or restart the PC.",
    },
    pantallaSinPermisoPor:
      "Windows did not hand over the meeting window. The app tries again on its own.",
    pantallaNoPudoPor:
      "Windows did not hand over the meeting window, or it lacks the text recognition for your language (Settings → Time & language → Language & region). The app tries again on its own.",
    porQueNoSeSabe: { "sin-salida": "This PC has no default audio output." },
    enTuLlavero: "stored in Credential Manager, never in a file",
    guardarEnLlavero: "Save to Credential Manager",
    permSistemaPara:
      "What plays on your PC: the client’s voice. No bot in the meeting.",
    permAcoplePara:
      "So the band does not cover the call: the meeting shrinks and the two live together.",
    sinReunionVoz:
      "Join your Zoom, Teams or Meet call and it will show up here. Meanwhile, get ready.",
    enTuMac: "on your PC",
    // Sprint 010: el resto del diccionario del Mac que nombraba macOS, el Mac o un permiso de Apple, dicho para Windows.
    // Varios los pinta el componente con el texto de `windows.ts` (`modoDetalle`, `transcribeTuPc…`, `idiomasListos`); aquí
    // queda igual el diccionario entero, para que ninguna cadena del Mac sobreviva en él (gate `windows-sin-mac`) y para que
    // `windows-honestidad-y-manual` ate las copias a su fuente.
    concederEnMacos: "Grant in Windows",
    modoDetalle:
      "The app opens a connection only when you ask, once per model: to install the speech model, 1.6 GB (0.5 GB on an ARM64 PC), and to install the suggestion’s local model, 1.1 GB; both are checked before use. There is no API on Windows: everything else stays on your PC.",
    cincoIdiomas:
      "Two languages ready at a time, at most: the ones the app offers.",
    idiomasListos: "languages ready at once, at most: the ones the app offers.",
    transcribeTuMac: "Transcribes on your PC, not a service",
    transcribeTuMacDetalle:
      "The speech model (whisper) lives inside your machine. No audio leaves to become text. The only time it touches the network is when you ask to install a model: Angel Ghost downloads it from its origin and checks its checksum before using it.",
    altavozExterno:
      "Over HDMI or DisplayPort the sound comes out of a speaker: the microphone will hear the client and audio-only mode won\u2019t speak.",
    porQueNoSeVe: {
      "sin-accesibilidad":
        "The app could not see your browser\u2019s windows on this PC, and without that it cannot tell whether you have Meet open.",
    },
    dosPermisos:
      "On Windows, system audio and Screen ask for no permission: only the microphone has its own, in Settings.",
    antesDeQueMacos: "The microphone on Windows",
    fraseDeMacos:
      "The app never asks for the permission on its own: you grant or revoke it yourself in Settings → Privacy & security → Microphone.",
    porQueNoHayMotor: {
      "sin-transcriptor":
        "This PC does not have the speech model. Install it in Language.",
      "no-contesta":
        "The speech model did not answer. Reopen the app; if it repeats, restart the PC.",
    },
    porQueNoRedacta: {
      "apple-intelligence-apagado": SIN_SUGERENCIA_EN,
      "mac-no-compatible": SIN_SUGERENCIA_EN,
      "no-disponible": SIN_SUGERENCIA_EN,
      "modelo-descargandose": SIN_SUGERENCIA_EN,
      "sin-puente": SIN_SUGERENCIA_EN,
    },
    caracteresTachado:
      "characters. Struck through, what was replaced on your PC before sending.",
    vigilanciaTitulo: "Invasive software running on your PC",
    vigilanciaSoloTuMac: "only your PC",
    presencial: {
      reunionEnEsteMac: "call on this PC",
      pcPregunta: "Is the call on this PC?",
      pcQueHacer:
        "Use «call on this PC»: the app hears the client through the system audio. In person is for a room with no video call.",
    },
  },
  notas: {
    clienteNoLaVe: "with protection requested when sharing",
    borrarDetalle:
      "It is deleted from the app now and does not come back. If you back up your PC, your backups keep it until they expire.",
    lasTresPrimeras: "The first three died on close; the tray, on expiry.",
    enTuLlavero: "in your Windows user’s Credential Manager",
    mostrarEnFinder: "Show in File Explorer",
    paraLeerla:
      "To read it, Windows Hello (your PIN, fingerprint or face), as for exporting a meeting. Once per app session.",
    abrirConTouchId: "Open with Windows Hello",
  },
  ensayo: {
    borrarDetalle:
      "They are deleted from the app now and do not come back. If you back up your PC, your backups keep them until they expire.",
    microfonoSinPermiso:
      "Windows did not grant the microphone. Grant it in Settings → Privacy & security → Microphone and start the rehearsal again.",
    microfonoOtro:
      "Windows would not open the microphone. Start the rehearsal again; if it repeats, restart the PC.",
    sinTranscripcion: "This PC cannot transcribe the rehearsal’s language",
    noSeSabeSiHayLlamadaQue:
      "The app could not see whether your browser is in a call. Put on headphones or close the call to rehearse.",
    vozSinReunion:
      "With no meeting, the voice plays wherever your PC plays sound; with a video call open, the rehearsal won’t start on the PC’s speakers. While it speaks, the microphone does not listen.",
    // Sprint 013 (audit, A-2): on Windows nobody enriches the bank, so the rehearsal does not send you to turn it on in AI.
    // The sprint 014 local model writes the meeting’s suggestion, not the bank.
    enriquecerApagadoQue:
      "On Windows the question bank is the rules-based one; a model’s extra touch is “not for now”.",
    porQueNoSumo: {
      "sin-proveedor": "on Windows the bank isn’t enriched for now.",
    },
  },
  cliente: {
    panelProtegido: "The panel keeps the protection requested from Windows",
  },
};

export const CAMBIOS_DE_WINDOWS: Record<Idioma, Cambios> = { es: ES, en: EN };
