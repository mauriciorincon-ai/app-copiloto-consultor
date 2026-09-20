---
description: Checklist pre-merge a main para el PERFIL ESCRITORIO (Tauri). Sustituye a /deploy-check en apps que se distribuyen como binario, no como sitio.
---

# /release-check

Checklist exhaustivo antes de mergear a `main` en una app de **escritorio** (kit v1.27.0). Aquí
mergear NO publica nada: la app se **distribuye** como binario firmado/notarizado, y el usuario
lo instala. Por eso las secciones de Vercel y Lighthouse de `/deploy-check` no existen y aparecen
en su lugar las del binario, los permisos del sistema y la no-persistencia.

> **Regla dura (kit v1.15.0): cada casilla se verifica con EL comando del `ci-escritorio.yml`,
> no con uno parecido.** Y todo comando que sea gate viaja ENTRE BACKTICKS (kit v1.24.0).

### 1. Tests
- [ ] `pnpm test` verde (con su `--coverage`); cobertura ≥70 % en `src/**` sin errores de glob.
- [ ] `pnpm test:e2e` verde y con CERO flaky (la UI de la webview vía `pnpm preview`).
- [ ] `cargo test --locked` verde en `src-tauri/` (lo nativo: captura, ventana, permisos, buffers).

### 2. Type safety y lint
- [ ] `pnpm typecheck` sin errores · `pnpm lint` sin warnings nuevos · `cargo clippy -- -D warnings` limpio.
- [ ] Tokens de tinta vetados y reduced-motion como en `/deploy-check` §3 (la UI sigue siendo web).

### 3. Build del binario
- [ ] `pnpm build` (frontend) verde · `cargo check --locked` verde en CI (`build-escritorio`).
- [ ] `pnpm tauri build` corre en local y produce el bundle (`.app`/`.dmg` en macOS); tamaño
      anotado en la bitácora (regresiones de peso se ven aquí, no en Lighthouse).
- [ ] Firma y notarización: **antes de G-Release**, no por PR** — pero el PR que toque
      `tauri.conf.json`/`Info.plist` declara qué cambió en permisos y entitlements.

### 4. Permisos del sistema (TCC en macOS)
- [ ] Cada permiso que la app pide (micrófono · pantalla y audio del sistema · accesibilidad)
      tiene su `NS*UsageDescription` en español e inglés y su estado en la UI con símbolo + texto
      + color (daltonismo).
- [ ] La app arranca y es usable SIN permisos concedidos (estado explícito, no crash).

### 5. Ventana protegida (si aplica)
- [ ] `content_protected: true` en la ventana del panel y test unitario de la config.
- [ ] El gate ⭐ de la guía de prueba incluye la parada «la ventana NO aparece en la pantalla
      compartida» por cliente de videollamada — **ningún e2e puede verla**; se declara humano.

### 6. No persistencia — estándar 4-T (apps que captan a terceros)
- [ ] `pnpm verify:ephemeral` verde: tras una sesión completa, cero archivos nuevos fuera de la
      carpeta de notas; buffers vaciados al cerrar; sin temporales.
- [ ] Test de **fuga inyectada** en rojo demostrado (un `fs::write` plantado en `capture/` hace
      fallar el lint/test) y registrado en la bitácora del sprint.
- [ ] Término plantado en logs: un secreto conocido dicho en la sesión de prueba NO aparece en
      ningún log ni archivo.
- [ ] Contador de salida a red = 0 en modo local durante la sesión de prueba.

### 7. Security
- [ ] `pnpm audit --audit-level high` limpio · `cargo audit` limpio (o ADR con la excepción).
- [ ] Sin secretos (gitleaks) · CSP de la webview restrictiva · `allowlist`/capabilities de Tauri
      mínimas (solo los comandos que la UI usa).

### 8. Observabilidad
- [ ] Logs solo-metadatos (Pino en la UI / `tracing` en Rust): jamás contenido del usuario ni de
      terceros. Sentry, si existe, metadata-only e inerte sin DSN.

### 9. Accesibilidad y diseño
- [ ] axe en los e2e · teclado end-to-end · ambos temas · `design-system.md` y `design-sync/`
      actualizados si el sprint tocó UI (`/deploy-check` §7 aplica tal cual).

### 10. Documentación y cero enlaces
- [ ] `docs/MANUAL-DE-USO.md` y `docs/GUIA-DE-PRUEBA.html` (acumulativa) al día; summary EN el PR.
- [ ] Barrido de cero enlaces (regla 17) — aquí además: ningún enlace de DESCARGA del binario
      publicado (la app se muestra, no se entrega).

### 11. Los checks del PR — ¿EJECUTARON? (kit v1.16.0)
- [ ] `gh pr checks` muestra `quality`, `e2e` y `build-escritorio` en `success` propio.

### 12. El disco en runtime (kit v1.21.0)
- [ ] Inventario de lo que la app ESCRIBE al correr (índice del corpus, notas, config, logs) con
      permisos y ruta; nada fuera del inventario (`verify:ephemeral` lo demuestra).

## Output esperado
### ✅ Pasa (N/12) · ### ❌ Falla (N/12) — bloquea merge · ### ⚠️ Warnings
### Decisión: MERGE OK | NO MERGE
