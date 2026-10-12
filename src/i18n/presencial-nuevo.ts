import { useIdioma, type Idioma } from "./index";

/**
 * **Lo que Sesión en presencial dice y no estaba en la maqueta** (sprint 008, auditoría del S5, M7 y B13).
 *
 * - **El micrófono de la sala, nombrado** (M7): la app escucha la sala por el micrófono por defecto del equipo. Si es el de
 *   unos auriculares (USB o Bluetooth), el sistema escucha por ellos y la app oiría sobre todo tu voz. Sesión nombra el
 *   micrófono antes de empezar y, si es externo, lo dice con dónde cambiarlo.
 * - **Sin voz para tu idioma** (B13): pediste la ficha al oído y no hay voz; antes la sesión seguía con «la banda» marcada
 *   sin decir por qué.
 *
 * Como `windows.ts`, estos textos **no existen en ninguna maqueta** y por eso viven aparte: el diccionario de la maqueta
 * (`es.ts`, `en.ts`) no puede inventar copy. **Estado: maquetados, no vistos**; van al gate humano del hito.
 */
export type TextosDePresencial = {
  /** «Micrófono de la sala: AirPods Pro». */
  microfono: (nombre: string) => string;
  /** El aviso cuando es externo, con dónde cambiarlo según el sistema. */
  externa: (nombre: string, dondeCambiarlo: string) => string;
  enElMac: string;
  enWindows: string;
  /** Con la sesión en marcha y la ficha al oído pedida, si no hay voz para el idioma del consultor. */
  sinVoz: (idioma: string) => string;
};

const ES: TextosDePresencial = {
  microfono: (n) => `Micrófono de la sala: ${n}`,
  externa: (n, donde) =>
    `${n} es un dispositivo externo. Si son tus auriculares con micrófono, el sistema escucha por ellos y la app oirá sobre todo tu voz: elige el micrófono del equipo en ${donde}.`,
  enElMac: "System Settings → Sound → Input",
  enWindows: "Configuración → Sistema → Sonido → Entrada",
  sinVoz: (idioma) => `No hay voz para leer en ${idioma}: la ficha va a la banda.`,
};

const EN: TextosDePresencial = {
  microfono: (n) => `Room microphone: ${n}`,
  externa: (n, donde) =>
    `${n} is an external device. If it is your headphones with a microphone, the system listens through them and the app will hear mostly you: pick the computer's microphone in ${donde}.`,
  enElMac: "System Settings → Sound → Input",
  enWindows: "Settings → System → Sound → Input",
  sinVoz: (idioma) => `There is no voice to read in ${idioma}: the card goes to the band.`,
};

export const TEXTOS_DE_PRESENCIAL: Record<Idioma, TextosDePresencial> = { es: ES, en: EN };

export function useTextosDePresencial(): TextosDePresencial {
  return TEXTOS_DE_PRESENCIAL[useIdioma()];
}
