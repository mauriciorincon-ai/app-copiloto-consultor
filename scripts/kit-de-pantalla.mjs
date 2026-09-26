#!/usr/bin/env node
// kit-de-pantalla — genera las imágenes sintéticas de «pantalla compartida» del kit de prueba.
//
// Cada imagen es una ventana de videollamada inventada —barra de arriba, participantes a la
// derecha, reloj y código de reunión abajo— con una diapositiva del CLIENTE en el centro. La
// interfaz está a propósito: es el ruido que la lectura de pantalla tiene que aprender a tirar.
//
// **Datos 100 % sintéticos** (regla 5 de la casa): Páramo Azul y la Cooperativa Sur del Valle son
// los clientes inventados del corpus del kit; los participantes no existen.
//
// Se regenera con `node scripts/kit-de-pantalla.mjs` y las imágenes se versionan: el test que las
// lee (`contra-el-mac-de-verdad.rs`) no puede depender de un navegador en la integración continua.
import { chromium } from "@playwright/test";
import { writeFileSync } from "node:fs";

const ANCHO = 1600;
const ALTO = 1000;

/** Las diapositivas. `titulo` y `cuerpo` son lo que el cliente comparte. */
const DIAPOSITIVAS = [
  {
    archivo: "tablero-margen.png",
    titulo: "Margen por canal",
    cuerpo: [
      "Mayorista · 18 %",
      "Tiendas de vereda · 9 %",
      "Venta directa · 23 %",
    ],
    barras: [18, 9, 23],
  },
  {
    archivo: "cronograma-erp.png",
    titulo: "Cronograma del proyecto",
    cuerpo: [
      "Semana 1 · extractos del ERP",
      "Semana 2 · perfilado",
      "Semana 4 · tablero y taller",
    ],
  },
  {
    archivo: "caso-cooperativa.png",
    titulo: "Cooperativa Sur del Valle",
    cuerpo: [
      "Resultados del piloto",
      "Margen recuperado en 1 de 5 puntos de venta",
      "Cierre: 2 semanas de retraso",
    ],
  },
  {
    archivo: "retencion-datos.png",
    titulo: "Manejo de datos",
    cuerpo: [
      "Retención: 30 días tras el cierre",
      "Los datos viven en su nube",
      "Borrado certificado",
    ],
  },
  {
    archivo: "agenda.png",
    titulo: "Agenda",
    cuerpo: ["Bienvenida", "Revisión de avances", "Preguntas"],
  },
  // El radar ámbar (C14, sprint 002, fase 4): la misma reunión, pero GRABADA y con un bot de notas
  // en la lista. El aviso y el nombre del bot salen del catálogo `data/radar/avisos.json`. Esta
  // imagen no está en `pantalla.json`: la lee su propio test, `el_radar_ambar_lee_la_reunion_grabada`.
  {
    archivo: "reunion-grabada.png",
    titulo: "Revisión de avances",
    cuerpo: ["Tablero de margen: listo", "Piloto: 1 de 5 tiendas", "Pendiente: datos del ERP"],
    aviso: "Esta reunión se está grabando",
    participantes: ["Laura Méndez", "Laura's Notetaker (Otter.ai)", "Tú"],
  },
];

const PARTICIPANTES = ["Laura Méndez", "Andrés Quintero", "Tú"];

function html(d) {
  const barras = d.barras
    ? `<div class="barras">${d.barras.map((b) => `<div style="height:${b * 8}px"></div>`).join("")}</div>`
    : "";
  return `<!doctype html><html><head><meta charset="utf-8"><style>
    body{margin:0;width:${ANCHO}px;height:${ALTO}px;background:#202124;font-family:-apple-system,Helvetica,Arial,sans-serif;color:#e8eaed;overflow:hidden}
    .arriba{height:44px;display:flex;align-items:center;padding:0 20px;font-size:14px;color:#bdc1c6}
    .centro{position:absolute;left:24px;top:56px;width:1230px;height:860px;background:#fff;color:#1f2937;border-radius:8px;padding:56px 64px;box-sizing:border-box}
    h1{font-size:54px;margin:0 0 36px;font-weight:700}
    li{font-size:30px;margin:14px 0;list-style:none}
    .barras{display:flex;gap:48px;align-items:flex-end;height:220px;margin-top:28px}
    .barras div{width:90px;background:#2563eb}
    .lado{position:absolute;right:24px;top:56px;width:300px}
    .tile{height:170px;background:#3c4043;border-radius:8px;margin-bottom:12px;position:relative}
    .tile span{position:absolute;left:10px;bottom:8px;font-size:13px}
    .abajo{position:absolute;bottom:0;left:0;right:0;height:72px;display:flex;align-items:center;gap:24px;padding:0 24px;font-size:14px;color:#bdc1c6}
    .boton{background:#3c4043;border-radius:20px;padding:8px 16px}
    .rec{color:#f28b82;margin-right:18px}
  </style></head><body>
    <div class="arriba">${d.aviso ? `<span class="rec">● ${d.aviso}</span>` : ""}Revisión trimestral · Meet</div>
    <div class="centro"><h1>${d.titulo}</h1><ul>${d.cuerpo.map((c) => `<li>${c}</li>`).join("")}</ul>${barras}</div>
    <div class="lado">${(d.participantes ?? PARTICIPANTES).map((p) => `<div class="tile"><span>${p}</span></div>`).join("")}</div>
    <div class="abajo"><span>14:03</span><span>abc-defg-hij</span><span class="boton">Presentar ahora</span><span class="boton">Salir de la llamada</span></div>
  </body></html>`;
}

const nav = await chromium.launch();
const pg = await nav.newPage({
  viewport: { width: ANCHO, height: ALTO },
  deviceScaleFactor: 1,
});
// `--solo=archivo.png` regenera una sola imagen: las demás ya están versionadas, y volver a
// fotografiarlas con otra versión del navegador las cambiaría sin que cambie nada de lo que miden.
const solo = process.argv.find((a) => a.startsWith("--solo="))?.slice(7);
for (const d of DIAPOSITIVAS.filter((x) => !solo || x.archivo === solo)) {
  await pg.setContent(html(d));
  writeFileSync(
    `docs/kit-de-prueba/pantalla/${d.archivo}`,
    await pg.screenshot({ type: "png" }),
  );
  console.log(`✓ ${d.archivo}`);
}
await nav.close();
