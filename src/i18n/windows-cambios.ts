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
type Parcial<T> = { [K in keyof T]?: T[K] extends string ? string : Parcial<T[K]> };

/** Una parte del diccionario: una clave que no existe en `es.ts` no compila. */
export type Cambios = Parcial<Diccionario>;

const ES: Cambios = {
  banda: {
    // La banda no promete invisibilidad donde nadie la verificó (regla dura 6).
    sinVerificarCuando: "Sin verificar en Teams, Zoom ni Meet · se verifica en tu PC",
    sugerenciaEnTuMac: "sugerencia · en tu PC",
    radarTeMira: "Te está mirando un programa de tu PC",
  },
  cuaderno: {
    sesionSub: "Nada se enciende hasta que tú lo digas. Esto es lo que la app ve de tu propio PC.",
    proteccionDetalle:
      "Tu panel le pide a Windows que no salga en las capturas, pero en Teams, Zoom y Meet está sin verificar: se verifica con una videollamada real en tu PC. Mientras tanto, comparte una ventana, no la pantalla.",
    modeloDelSistema: "Modelo del sistema",
    permSistemaPara: "Lo que suena en tu PC: la voz del cliente. Sin bot en la reunión.",
    permAcoplePara: "Para que la banda no tape la llamada: la reunión se encoge y las dos conviven.",
    presencial: {
      reunionEnEsteMac: "reunión en este PC",
      pcPregunta: "¿La videollamada está en este PC?",
      pcQueHacer:
        "Usa «reunión en este PC»: la app oye al cliente por el audio del sistema. Presencial es para una sala sin videollamada.",
    },
  },
  ensayo: {
    vozSinReunion:
      "Sin reunión, la voz sale por donde suene tu PC; con una videollamada abierta, el ensayo no empieza por los altavoces del PC. Mientras habla, el micrófono no escucha.",
  },
};

const EN: Cambios = {
  banda: {
    sinVerificarCuando: "Not verified in Teams, Zoom or Meet · you verify it on your PC",
    sugerenciaEnTuMac: "suggestion · on your PC",
    radarTeMira: "A program on your PC is watching you",
  },
  cuaderno: {
    sesionSub: "Nothing turns on until you say so. This is what the app sees of your own PC.",
    proteccionDetalle:
      "Your panel asks Windows to stay out of screen captures, but in Teams, Zoom and Meet it is unverified: you verify it with a real video call on your PC. Until then, share a window, not the screen.",
    modeloDelSistema: "System model",
    permSistemaPara: "What plays on your PC: the client’s voice. No bot in the meeting.",
    permAcoplePara: "So the band does not cover the call: the meeting shrinks and the two live together.",
    presencial: {
      reunionEnEsteMac: "call on this PC",
      pcPregunta: "Is the call on this PC?",
      pcQueHacer:
        "Use «call on this PC»: the app hears the client through the system audio. In person is for a room with no video call.",
    },
  },
  ensayo: {
    vozSinReunion:
      "With no meeting, the voice plays wherever your PC plays sound; with a video call open, the rehearsal won’t start on the PC’s speakers. While it speaks, the microphone does not listen.",
  },
};

export const CAMBIOS_DE_WINDOWS: Record<Idioma, Cambios> = { es: ES, en: EN };
