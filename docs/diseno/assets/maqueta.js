// Conmutadores de la sala de diseño: estado · tema · idioma. Sin frameworks.
(function () {
  const html = document.documentElement;
  const guarda = (k, v) => { try { localStorage.setItem("ghost-maqueta-" + k, v); } catch (_) {} };
  const lee = (k) => { try { return localStorage.getItem("ghost-maqueta-" + k); } catch (_) { return null; } };
  html.dataset.theme = html.dataset.theme || lee("theme") || "dark";
  html.dataset.lang = html.dataset.lang || lee("lang") || "es";
  html.lang = html.dataset.lang;
  const primero = document.querySelector(".mq-bar [data-estado]");
  if (!html.dataset.estado && primero) html.dataset.estado = primero.dataset.estado;

  function aplicar() {
    const e = html.dataset.estado;
    document.querySelectorAll("[data-en]").forEach((el) => { el.hidden = !el.dataset.en.split(" ").includes(e); });
    document.querySelectorAll(".mq-bar [data-estado]").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.estado === e)));
    document.querySelectorAll(".mq-bar [data-theme-set]").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.themeSet === html.dataset.theme)));
    document.querySelectorAll(".mq-bar [data-lang-set]").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.langSet === html.dataset.lang)));
    document.querySelectorAll(".mq-nota .n").forEach((n) => n.classList.toggle("activa", n.dataset.para === e));
    document.querySelectorAll("[data-transcript-en]").forEach((p) => { p.dataset.transcript = p.dataset.transcriptEn.split(" ").includes(e) ? "on" : "off"; });
    html.lang = html.dataset.lang;
  }
  window.__mqApply = aplicar;
  document.addEventListener("click", (ev) => {
    const b = ev.target.closest(".mq-bar button"); if (!b) return;
    if (b.dataset.estado) html.dataset.estado = b.dataset.estado;
    if (b.dataset.themeSet) { html.dataset.theme = b.dataset.themeSet; guarda("theme", b.dataset.themeSet); }
    if (b.dataset.langSet) { html.dataset.lang = b.dataset.langSet; guarda("lang", b.dataset.langSet); }
    aplicar();
  });
  aplicar();
})();
