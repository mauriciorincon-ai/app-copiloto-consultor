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

/** En Windows no hay modelo del sistema (ADR 023): la sugerencia, con el API; sin él, la ficha. */
const SIN_MODELO_ES =
  "En Windows no hay modelo del sistema: la sugerencia la redacta el API, si lo enciendes con tu clave.";
const SIN_MODELO_EN =
  "Windows has no system model: the API writes the suggestion, if you turn it on with your key.";

const ES: Cambios = {
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
    modeloDelSistema: "Modelo del sistema",
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
    // IA en Windows (sprint 008, ADR 023): no hay modelo del sistema; cualquier motivo de macOS dice esa verdad.
    enTuMac: "en tu PC",
    // Sprint 010: el resto del diccionario del Mac que nombraba macOS, el Mac o un permiso de Apple, dicho para Windows.
    // Varios los pinta el componente con el texto de `windows.ts` (`modoDetalle`, `transcribeTuPc…`, `idiomasListos`); aquí
    // queda igual el diccionario entero, para que ninguna cadena del Mac sobreviva en él (gate `windows-sin-mac`) y para que
    // `windows-honestidad-y-manual` ate las copias a su fuente.
    concederEnMacos: "Conceder en Windows",
    modoDetalle:
      "La app abre una conexión solo si enciendes el API en IA, con tu clave, y solo con texto anonimizado. La otra la abre la propia app, una vez, cuando pides instalar el modelo de voz: 1,6 GB (0,5 GB en un PC ARM64), comprobados antes de usarse.",
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
      "apple-intelligence-apagado": SIN_MODELO_ES,
      "mac-no-compatible": SIN_MODELO_ES,
      "no-disponible": SIN_MODELO_ES,
      "modelo-descargandose": SIN_MODELO_ES,
      "sin-puente": SIN_MODELO_ES,
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
  },
  cliente: {
    panelProtegido: "El panel sigue con la protección pedida a Windows",
  },
};

const EN: Cambios = {
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
    modeloDelSistema: "System model",
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
      "The app opens a connection only if you turn on the API in AI, with your key, and only with anonymized text. The other one the app opens itself, once, when you ask to install the speech model: 1.6 GB (0.5 GB on an ARM64 PC), checked before use.",
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
      "apple-intelligence-apagado": SIN_MODELO_EN,
      "mac-no-compatible": SIN_MODELO_EN,
      "no-disponible": SIN_MODELO_EN,
      "modelo-descargandose": SIN_MODELO_EN,
      "sin-puente": SIN_MODELO_EN,
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
  },
  cliente: {
    panelProtegido: "The panel keeps the protection requested from Windows",
  },
};

export const CAMBIOS_DE_WINDOWS: Record<Idioma, Cambios> = { es: ES, en: EN };
