import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

/**
 * AXE sobre cada pantalla, en los dos temas y con el cinturón de movimiento puesto.
 *
 * **El tema importa y no es decorativo.** El usuario de esta app tiene daltonismo leve y el
 * design system obliga a que todo estado sea símbolo + texto + color. El contraste se mide aquí,
 * en los dos temas, porque una paleta que cumple AA en oscuro puede no cumplirlo en claro.
 *
 * **Y con reduced motion**, porque es la combinación que el kit exige explícitamente: lo que se
 * esconde esperando una animación que no va a llegar no lo ve un axe en condiciones normales.
 */

const PANTALLAS = [
  { que: "sesión", url: "ventana=principal&pantalla=sesion" },
  { que: "permisos", url: "ventana=principal&pantalla=permisos" },
  { que: "corpus", url: "ventana=principal&pantalla=corpus" },
  { que: "honestidad", url: "ventana=principal&pantalla=honestidad" },
  { que: "idioma", url: "ventana=principal&pantalla=idioma" },
  { que: "banda · ficha", url: "ventana=banda&estado=ficha" },
  { que: "banda · sin resultado", url: "ventana=banda&estado=sin-resultado" },
  { que: "banda · sin verificar", url: "ventana=banda&estado=sin-verificar&verificado=0" },
  // El radar (sprint 002, fase 4): ámbar y coral son avisos con color propio, y el coral lleva
  // botones. En el navegador la ventana es alta, así que el coral sale ampliado —con sus botones—.
  { que: "banda · radar ámbar", url: "ventana=banda&estado=radar" },
  { que: "banda · radar coral", url: "ventana=banda&estado=radar-invasivo" },
  { que: "sesión · software invasivo en tu Mac", url: "ventana=principal&pantalla=sesion&radar=vigilancia" },
  // La síntesis (fase 5): la pantalla IA, con su formulario de clave, y la sugerencia en la banda.
  { que: "ia", url: "ventana=principal&pantalla=ia" },
  // Sprint 003, fase 1: lo que salió al API (B37) y las cuatro vistas de Notas (C9).
  { que: "ia · lo que salió", url: "ventana=principal&pantalla=ia&vista=salio" },
  { que: "ia · la puerta", url: "ventana=principal&pantalla=ia&vista=puerta" },
  { que: "ia · la puerta abierta", url: "ventana=principal&pantalla=ia&vista=puerta&puerta=abierta" },
  { que: "ia · la puerta se cerró sola", url: "ventana=principal&pantalla=ia&vista=puerta&puerta=en-reunion" },
  { que: "notas · durante", url: "ventana=principal&pantalla=notas&estado=durante" },
  { que: "notas · al cerrar", url: "ventana=principal&pantalla=notas&estado=al-cerrar" },
  { que: "notas · el archivo", url: "ventana=principal&pantalla=notas&estado=archivo" },
  { que: "notas · exportar", url: "ventana=principal&pantalla=notas&estado=exportar" },
  // Las propuestas y la bandeja (sprint 003, fase 2).
  { que: "notas · con propuestas", url: "ventana=principal&pantalla=notas&estado=propuestas" },
  { que: "notas · al cerrar, con bandeja", url: "ventana=principal&pantalla=notas&estado=al-cerrar-bandeja" },
  { que: "notas · al cerrar, ventana cero", url: "ventana=principal&pantalla=notas&estado=al-cerrar-cero" },
  { que: "notas · la bandeja", url: "ventana=principal&pantalla=notas&estado=bandeja" },
  { que: "notas · bandeja con llave", url: "ventana=principal&pantalla=notas&estado=bandeja-llave" },
  { que: "notas · bandeja vencida", url: "ventana=principal&pantalla=notas&estado=vencida" },
  { que: "honestidad · con bandeja", url: "ventana=principal&pantalla=honestidad&estado=bandeja" },
  { que: "honestidad · la tarea no corrió", url: "ventana=principal&pantalla=honestidad&estado=no-corrio" },
  { que: "banda · te propongo guardar", url: "ventana=banda&estado=ficha-propuesta" },
  { que: "banda · fijada", url: "ventana=banda&estado=ficha-fijada" },
  // El marco en la mano (sprint 003, fase 3, ADR 017): «Este cliente», la NDA, la cláusula y solo notas.
  { que: "sesión · en marcha", url: "ventana=principal&pantalla=sesion&estado=en-marcha" },
  { que: "sesión · la pregunta de la NDA", url: "ventana=principal&pantalla=sesion&estado=pregunta" },
  { que: "sesión · sin jurisdicción", url: "ventana=principal&pantalla=sesion&estado=sin-bandera" },
  { que: "sesión · la cláusula", url: "ventana=principal&pantalla=sesion&estado=clausula" },
  { que: "sesión · la NDA lo prohíbe", url: "ventana=principal&pantalla=sesion&estado=nda" },
  { que: "sesión · en solo notas", url: "ventana=principal&pantalla=sesion&estado=solo-notas" },
  { que: "banda · solo notas", url: "ventana=banda&estado=solo-notas" },
  // La banda arriba (sprint 004): la variante con el asa abajo, y el aviso de la primera vez en Sesión.
  { que: "banda · arriba", url: "ventana=banda&estado=ficha&borde=arriba" },
  { que: "banda · arriba · solo audio", url: "ventana=banda&estado=voz&borde=arriba" },
  { que: "sesión · la primera vez con la banda arriba", url: "ventana=principal&pantalla=sesion&estado=aviso" },
  { que: "banda · sugerencia", url: "ventana=banda&estado=sugerencia-local" },
  // El ensayo (sprint 004, fase 3, ADR 019): cada estado que la pantalla dibuja.
  { que: "ensayo · preparar", url: "ventana=principal&pantalla=ensayo&estado=preparar" },
  { que: "ensayo · no empezó", url: "ventana=principal&pantalla=ensayo&estado=no-empezo" },
  { que: "ensayo · preguntando", url: "ventana=principal&pantalla=ensayo&estado=preguntando" },
  { que: "ensayo · del modelo", url: "ventana=principal&pantalla=ensayo&estado=del-modelo" },
  { que: "ensayo · respondiendo", url: "ventana=principal&pantalla=ensayo&estado=respondiendo" },
  { que: "ensayo · evaluada", url: "ventana=principal&pantalla=ensayo&estado=evaluada" },
  { que: "ensayo · el informe", url: "ventana=principal&pantalla=ensayo&estado=cerrado" },
  { que: "ensayo · sin corpus", url: "ventana=principal&pantalla=ensayo&estado=sin-corpus" },
];

test.use({ reducedMotion: "reduce" });

for (const tema of ["dark", "light"] as const) {
  for (const p of PANTALLAS) {
    test(`«${p.que}» · tema ${tema} · sin hallazgos de axe`, async ({ page }) => {
      await page.goto(`/?${p.url}`);
      await page.evaluate((t) => {
        document.documentElement.dataset.theme = t;
      }, tema);
      await page.evaluate(() => document.fonts.ready);

      const { violations } = await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
        .analyze();

      expect(
        violations.map((v) => `${v.id}: ${v.nodes.length} · ${v.help}`),
        `axe encontró ${violations.length} incumplimiento(s) en «${p.que}» (${tema})`,
      ).toEqual([]);
    });
  }
}
