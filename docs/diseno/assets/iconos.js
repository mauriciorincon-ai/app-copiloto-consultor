// Iconografía del design system (D9): SVG de trazo 16 px, monocromo, currentColor.
// Se inyecta como sprite al cargar; uso: <svg class="ic"><use href="#i-mic"/></svg>
(function () {
  const S = `<svg style="display:none" aria-hidden="true">
  <symbol id="i-check-circle" viewBox="0 0 16 16"><circle cx="8" cy="8" r="7" fill="currentColor" stroke="none"/><path d="M5 8.2l2 2 4-4.4" stroke="var(--bg)" stroke-width="1.9" fill="none"/></symbol>
  <symbol id="i-alert" viewBox="0 0 16 16"><path d="M8 2.2 14.3 13H1.7z" fill="currentColor" stroke="none"/><path d="M8 6v3.4M8 11.6v.2" stroke="var(--bg)" stroke-width="1.8" fill="none"/></symbol>
  <symbol id="i-x-circle" viewBox="0 0 16 16"><circle cx="8" cy="8" r="7" fill="currentColor" stroke="none"/><path d="M5.6 5.6l4.8 4.8M10.4 5.6l-4.8 4.8" stroke="var(--bg)" stroke-width="1.8" fill="none"/></symbol>
  <symbol id="i-dot" viewBox="0 0 16 16"><circle cx="8" cy="8" r="4" fill="currentColor" stroke="none"/></symbol>
  <symbol id="i-ring" viewBox="0 0 16 16"><circle cx="8" cy="8" r="4"/></symbol>
  <symbol id="i-pendiente" viewBox="0 0 16 16"><circle cx="8" cy="8" r="5.2" stroke-dasharray="2.1 2.3"/></symbol>
  <symbol id="i-half" viewBox="0 0 16 16"><circle cx="8" cy="8" r="5.5"/><path d="M8 2.5v11A5.5 5.5 0 0 0 8 2.5z" fill="currentColor" stroke="none"/></symbol>
  <symbol id="i-mic" viewBox="0 0 16 16"><rect x="5.5" y="1.5" width="5" height="8" rx="2.5"/><path d="M3 7.5a5 5 0 0 0 10 0M8 12.5v2M5.5 14.5h5"/></symbol>
  <symbol id="i-mic-off" viewBox="0 0 16 16"><rect x="5.5" y="1.5" width="5" height="8" rx="2.5"/><path d="M3 7.5a5 5 0 0 0 10 0M8 12.5v2M5.5 14.5h5M2 2l12 12"/></symbol>
  <symbol id="i-sistema" viewBox="0 0 16 16"><path d="M2.5 6h2.5l3.5-3v10L5 10H2.5zM11 5.5a3.5 3.5 0 0 1 0 5M12.8 3.2a6 6 0 0 1 0 9.6"/></symbol>
  <symbol id="i-sistema-off" viewBox="0 0 16 16"><path d="M2.5 6h2.5l3.5-3v10L5 10H2.5zM11 6l3 4M14 6l-3 4"/></symbol>
  <symbol id="i-pantalla" viewBox="0 0 16 16"><rect x="1.5" y="2.5" width="13" height="9" rx="1.5"/><path d="M5.5 14h5M8 11.5V14"/></symbol>
  <symbol id="i-pantalla-off" viewBox="0 0 16 16"><rect x="1.5" y="2.5" width="13" height="9" rx="1.5"/><path d="M5.5 14h5M8 11.5V14M2 2l12 12"/></symbol>
  <symbol id="i-ojo" viewBox="0 0 16 16"><path d="M1.5 8s2.5-4.5 6.5-4.5S14.5 8 14.5 8 12 12.5 8 12.5 1.5 8 1.5 8z"/><circle cx="8" cy="8" r="2"/></symbol>
  <symbol id="i-ojo-off" viewBox="0 0 16 16"><path d="M1.5 8s2.5-4.5 6.5-4.5S14.5 8 14.5 8 12 12.5 8 12.5 1.5 8 1.5 8zM2 2l12 12"/></symbol>
  <symbol id="i-candado" viewBox="0 0 16 16"><rect x="3" y="7" width="10" height="7.5" rx="1.5"/><path d="M5 7V5a3 3 0 0 1 6 0v2"/></symbol>
  <symbol id="i-rayo" viewBox="0 0 16 16"><path d="M9 1.5 3.5 9H8l-1 5.5L12.5 7H8z" fill="currentColor" stroke="none"/></symbol>
  <symbol id="i-globo" viewBox="0 0 16 16"><circle cx="8" cy="8" r="6.5"/><path d="M1.5 8h13M8 1.5c2 2 2 11 0 13M8 1.5c-2 2-2 11 0 13"/></symbol>
  <symbol id="i-doc" viewBox="0 0 16 16"><path d="M4 1.5h5l3.5 3.5v9.5H4z"/><path d="M9 1.5V5h3.5M6 8.5h4M6 11h4"/></symbol>
  <symbol id="i-pin" viewBox="0 0 16 16"><path d="M9.5 1.5 14.5 6.5l-2 .5-2.5 2.5.5 3-1 1-3-3-4 4-.5-.5 4-4-3-3 1-1 3 .5L9.5 3.5z"/></symbol>
  <symbol id="i-pin-lleno" viewBox="0 0 16 16"><path d="M9.5 1.5 14.5 6.5l-2 .5-2.5 2.5.5 3-1 1-3-3-4 4-.5-.5 4-4-3-3 1-1 3 .5L9.5 3.5z" fill="currentColor" stroke="none"/></symbol>
  <symbol id="i-subir" viewBox="0 0 16 16"><path d="M8 13.5v-11M3.5 7 8 2.5 12.5 7"/></symbol>
  <symbol id="i-buscar" viewBox="0 0 16 16"><circle cx="7" cy="7" r="4.5"/><path d="M10.5 10.5 14 14"/></symbol>
  <symbol id="i-chispa" viewBox="0 0 16 16"><path d="M8 1.5 9.6 6.4 14.5 8l-4.9 1.6L8 14.5 6.4 9.6 1.5 8l4.9-1.6z" fill="currentColor" stroke="none"/></symbol>
  <symbol id="i-video" viewBox="0 0 16 16"><rect x="1.5" y="4" width="9" height="8" rx="1.5"/><path d="M10.5 7l4-2.5v7l-4-2.5"/></symbol>
  <symbol id="i-rec" viewBox="0 0 16 16"><circle cx="8" cy="8" r="6.5"/><circle cx="8" cy="8" r="3" fill="currentColor" stroke="none"/></symbol>
  <symbol id="i-bot" viewBox="0 0 16 16"><rect x="2.5" y="5" width="11" height="8.5" rx="2"/><path d="M8 5V2.5M6 1.5h4"/><circle cx="6" cy="9" r="1" fill="currentColor" stroke="none"/><circle cx="10" cy="9" r="1" fill="currentColor" stroke="none"/></symbol>
  <symbol id="i-radar" viewBox="0 0 16 16"><circle cx="8" cy="8" r="6.5"/><circle cx="8" cy="8" r="3"/><path d="M8 8l4.5-4.5"/></symbol>
  <symbol id="i-chevron" viewBox="0 0 16 16"><path d="M4 6l4 4 4-4"/></symbol>
  <symbol id="i-mac" viewBox="0 0 16 16"><path d="M11.2 8.6c0-1.6 1.3-2.3 1.4-2.4-.8-1.1-2-1.3-2.4-1.3-1-.1-2 .6-2.5.6s-1.3-.6-2.2-.6C4.4 4.9 3.3 5.6 2.7 6.6c-1.2 2-.3 5 .8 6.6.6.8 1.2 1.7 2.1 1.7.8 0 1.2-.5 2.2-.5s1.3.5 2.2.5c.9 0 1.5-.8 2-1.6.6-.9.9-1.8.9-1.9 0 0-1.7-.7-1.7-2.8zM9.6 3.9c.4-.5.7-1.3.7-2-.6 0-1.4.4-1.8.9-.4.5-.8 1.2-.7 2 .7 0 1.4-.4 1.8-.9z" fill="currentColor" stroke="none"/></symbol>
  <symbol id="i-nube-off" viewBox="0 0 16 16"><path d="M4.5 12.5h7a3 3 0 0 0 .4-6A4 4 0 0 0 4.3 7.6 2.5 2.5 0 0 0 4.5 12.5zM2 2l12 12"/></symbol>
  <symbol id="i-nube" viewBox="0 0 16 16"><path d="M4.5 12.5h7a3 3 0 0 0 .4-6A4 4 0 0 0 4.3 7.6 2.5 2.5 0 0 0 4.5 12.5z"/></symbol>
  <symbol id="i-auriculares" viewBox="0 0 16 16"><path d="M2.5 10V8a5.5 5.5 0 0 1 11 0v2"/><rect x="1.5" y="9" width="3" height="4.5" rx="1"/><rect x="11.5" y="9" width="3" height="4.5" rx="1"/></symbol>
  <symbol id="i-nota" viewBox="0 0 16 16"><path d="M2.5 3.5h11v9h-11zM5 6.5h6M5 9h4"/></symbol>
  <symbol id="i-ram" viewBox="0 0 16 16"><rect x="1.5" y="4.5" width="13" height="7" rx="1"/><path d="M4 11.5v2M7 11.5v2M10 11.5v2M13 11.5v2M4 6.5h2M8 6.5h2"/></symbol>
  <symbol id="i-reloj" viewBox="0 0 16 16"><circle cx="8" cy="8" r="6.5"/><path d="M8 4.5V8l2.5 1.5"/></symbol>
  <symbol id="i-basura" viewBox="0 0 16 16"><path d="M3 4.5h10M6 4.5V3h4v1.5M4.5 4.5l.7 9h5.6l.7-9"/></symbol>
  <symbol id="i-llave" viewBox="0 0 16 16"><circle cx="5.5" cy="10.5" r="3"/><path d="M7.7 8.3 14 2M11.5 4.5l2 2M9.5 6.5l2 2"/></symbol>
  <symbol id="i-voz" viewBox="0 0 16 16"><path d="M2 7v2M5 4.5v7M8 2.5v11M11 5.5v5M14 7v2"/></symbol>
  <symbol id="i-auriculares-off" viewBox="0 0 16 16"><path d="M2.5 10V8a5.5 5.5 0 0 1 11 0v2"/><rect x="1.5" y="9" width="3" height="4.5" rx="1"/><rect x="11.5" y="9" width="3" height="4.5" rx="1"/><path d="M1.5 1.5l13 13"/></symbol>
  <symbol id="i-flecha" viewBox="0 0 16 16"><path d="M2.5 8h11M9.5 4l4 4-4 4"/></symbol>
</svg>`;
  function inyectar() { if (!document.getElementById("ghost-iconos")) { const d = document.createElement("div"); d.id = "ghost-iconos"; d.innerHTML = S; document.body.prepend(d); } }
  if (document.body) inyectar(); else document.addEventListener("DOMContentLoaded", inyectar);
})();
