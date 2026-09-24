import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
// El design system EN CÓDIGO, tal cual: `docs/diseno/assets/ghost.css` es la fuente de verdad
// visual que declara `design-system.md` (`fuente_en_codigo`). El producto no lo copia — lo usa.
// Así la banda construida y la maqueta comparten literalmente el mismo CSS y el gate de
// FIDELIDAD compara dos cosas que no pueden derivar.
import "../docs/diseno/assets/ghost.css";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
