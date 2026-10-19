//! **LA MEDIDA DEL MODELO LOCAL EN WINDOWS** (sprint 013, fase 1; ADR 023 enmienda 1).
//!
//! Antes de construir el proveedor se mide en la Surface, y la tabla va al cierre: primero el catálogo de Foundry Local
//! (alias, variante, CPU o NPU, tamaño y licencia) **sin bajar nada**; después —con la fila M1 de la matriz y el «sí» del
//! usuario— la descarga, la carga, la latencia sobre el kit de sugerencias por el camino entero (`sugerir` y `fundar`),
//! la canaria del turno buscada en todo lo que el SDK escribió y las conexiones del proceso mientras redacta.
//!
//! Todo es `en_vivo_` e `#[ignore]`: ni la CI (`--skip en_vivo_`) ni `cargo test` a secas lo corren (regla 25).
//!
//! Uso, en este orden:
//!   1. `cargo test --test modelo-local-en-windows en_vivo_catalogo -- --ignored --nocapture`
//!   2. con el «sí» a la fila M1: `AG_FOUNDRY_DIR=<carpeta del modelo> AG_FOUNDRY_MODELO=<id de la variante>
//!      cargo test --test modelo-local-en-windows en_vivo_mide -- --ignored --nocapture`
#![cfg(windows)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use app_copiloto_consultor_lib::corpus::evaluar::Kit;
use app_copiloto_consultor_lib::corpus::Corpus;
use app_copiloto_consultor_lib::ficha::{armar, Respuesta as Hallazgo};
use app_copiloto_consultor_lib::sintesis::{self, PorQueNoRedacta, Proveedor, Quien, Respuesta};
use foundry_local_sdk::{
    FoundryLocalConfig, FoundryLocalManager, Item, LogLevel, MessageRole, Model, Request, RequestOptions,
    SearchOptions, Session,
};
use tauri::async_runtime::block_on;

/// El techo de la orden; la medida se toma con uno holgado para ver la latencia de verdad, también la que no cabe.
const TECHO_DE_LA_MEDIDA: Duration = Duration::from_secs(120);

/// Todo lo que el SDK escribe (datos, modelos y registro) va a una sola carpeta, con el registro en `Error`: nada en
/// `~/.angel-ghost`, que es lo que haría por defecto.
///
/// **La telemetría del runtime.** Foundry Local Core y ONNX Runtime GenAI suben eventos de uso a Microsoft (1DS) y los
/// encolan en `%LOCALAPPDATA%\Microsoft\DeveloperTools\.onnxruntime\`. Con `AG_FOUNDRY_SIN_TELEMETRIA=1` se encienden
/// los dos interruptores que traen sus binarios (`DisableNonessentialTelemetry` y las variables `ORT_*`), para medir
/// qué queda saliendo con ellos.
fn gestor(dir: &Path) -> Arc<FoundryLocalManager> {
    let mut config = FoundryLocalConfig::new("angel-ghost")
        .app_data_dir(dir.to_string_lossy())
        .model_cache_dir(dir.join("modelos").to_string_lossy())
        .logs_dir(dir.join("registro").to_string_lossy())
        .log_level(LogLevel::Error);
    if sin_telemetria() {
        std::env::set_var("ORT_TELEMETRY_DISABLED", "1");
        std::env::set_var("ORT_DISABLE_TELEMETRY", "1");
        config = config.additional_setting("DisableNonessentialTelemetry", "true");
    }
    FoundryLocalManager::create(config).expect("el SDK no arrancó")
}

fn sin_telemetria() -> bool {
    std::env::var("AG_FOUNDRY_SIN_TELEMETRIA").as_deref() == Ok("1")
}

#[test]
#[ignore = "en vivo: lee el catálogo de Foundry Local por la red, sin bajar ningún modelo"]
fn en_vivo_catalogo_del_modelo_local() {
    let dir = std::env::var("AG_FOUNDRY_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("ag-foundry-catalogo"));
    let g = gestor(&dir);
    let reloj = Instant::now();
    let modelos = block_on(g.catalog().get_models()).expect("el catálogo no respondió");
    println!("\n╭─ catálogo de Foundry Local · {} alias · {} ms", modelos.len(), reloj.elapsed().as_millis());
    for m in &modelos {
        for v in m.variants() {
            let Ok(i) = v.info() else { continue };
            let (disp, ep) = i
                .runtime
                .as_ref()
                .map(|r| (format!("{:?}", r.device_type), r.execution_provider.clone()))
                .unwrap_or_default();
            println!(
                "│ {:<26} {:<58} {:<4} {:<28} {:>6} MB · {} · caché {}",
                m.alias(),
                v.id(),
                disp,
                ep,
                i.file_size_mb.unwrap_or(0),
                i.license.clone().unwrap_or_else(|| "¿?".into()),
                i.cached
            );
        }
    }
    println!("╰─ carpeta del SDK: {}", dir.display());
}

/// **El proveedor de la medida**: una llamada al modelo local con el prompt real de la app. Es el que `sintesis/local.rs`
/// construirá si la medida cabe en el techo; aquí vive solo para medir. **Una sesión por sugerencia**, que se suelta al
/// terminar: una `ChatSession` guardaría el historial, y con él los turnos del cliente de una sugerencia a la siguiente.
struct Medido {
    modelo: Arc<Model>,
}

impl Proveedor for Medido {
    fn quien(&self) -> Quien {
        Quien::Sistema
    }
    fn nombre(&self) -> String {
        self.modelo.id().to_string()
    }
    fn disponible(&self) -> Result<(), PorQueNoRedacta> {
        Ok(())
    }
    fn redactar(&self, instrucciones: &str, texto: &str) -> Result<Respuesta, String> {
        let sesion = block_on(Session::new(&self.modelo)).map_err(|e| e.to_string())?;
        let opciones = RequestOptions {
            search: SearchOptions { temperature: Some(0.0), max_output_tokens: Some(160), ..Default::default() },
            ..Default::default()
        };
        let pedido = Request::from_items(vec![
            Item::system_message(vec![Item::text(instrucciones)]),
            Item::message(MessageRole::User, vec![Item::text(texto)]),
        ])
        .with_options(opciones);
        let r = block_on(sesion.process_request(pedido)).map_err(|e| e.to_string())?;
        Ok(Respuesta {
            json: r.text(),
            tokens_entrada: r.usage.prompt_tokens as u64,
            tokens_salida: r.usage.completion_tokens as u64,
            ..Default::default()
        })
    }
}

/// **El vigía de la red**: mientras dura una fase, pregunta al sistema (`netstat -ano`, unas tres veces por segundo)
/// qué conexiones TCP y qué puertos UDP son de este proceso. Solo direcciones y estados, nunca contenido. El contador
/// de la app no ve lo que hace el código nativo del SDK; el sistema operativo sí. Una conexión que nace y muere entre
/// dos miradas se le escapa: por eso la fase de reposo dura lo que tarda una subida de eventos en salir.
struct Vigia {
    parar: Arc<AtomicBool>,
    visto: Arc<Mutex<(usize, usize, BTreeSet<String>)>>,
    hilo: Option<std::thread::JoinHandle<()>>,
}

impl Vigia {
    fn empezar() -> Self {
        let parar = Arc::new(AtomicBool::new(false));
        let visto = Arc::new(Mutex::new((0usize, 0usize, BTreeSet::new())));
        let (p, v) = (parar.clone(), visto.clone());
        let pid = std::process::id().to_string();
        let hilo = std::thread::spawn(move || {
            while !p.load(Ordering::Relaxed) {
                for (proto, es_tcp) in [("TCP", true), ("UDP", false)] {
                    let Ok(s) = std::process::Command::new("netstat").args(["-ano", "-p", proto]).output() else { continue };
                    let texto = String::from_utf8_lossy(&s.stdout).to_string();
                    // «TCP  local  remoto  estado  pid» · «UDP  local  *:*  pid»
                    let mias: Vec<Vec<&str>> = texto
                        .lines()
                        .map(|l| l.split_whitespace().collect::<Vec<_>>())
                        .filter(|c| c.first() == Some(&proto) && c.last() == Some(&pid.as_str()))
                        .collect();
                    let mut g = v.lock().unwrap();
                    if es_tcp {
                        g.0 = g.0.max(mias.len());
                        for c in &mias {
                            let remoto = c.get(2).copied().unwrap_or("");
                            if !remoto.starts_with("127.") && !remoto.starts_with("[::1]") && !remoto.starts_with("0.0.0.0") {
                                g.2.insert(format!("{remoto} {}", c.get(3).copied().unwrap_or("")));
                            }
                        }
                    } else {
                        g.1 = g.1.max(mias.len());
                    }
                }
                std::thread::sleep(Duration::from_millis(150));
            }
        });
        Self { parar, visto, hilo: Some(hilo) }
    }

    fn parar(mut self) -> (usize, usize, BTreeSet<String>) {
        self.parar.store(true, Ordering::Relaxed);
        if let Some(h) = self.hilo.take() {
            let _ = h.join();
        }
        self.visto.lock().unwrap().clone()
    }
}

/// Lo que ocupa el proceso en memoria, en MB (el conjunto de trabajo).
fn memoria_mb() -> u64 {
    let orden = format!("(Get-Process -Id {}).WorkingSet64", std::process::id());
    std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &orden])
        .output()
        .ok()
        .and_then(|s| String::from_utf8_lossy(&s.stdout).trim().parse::<u64>().ok())
        .map(|b| b / 1_048_576)
        .unwrap_or(0)
}

/// Bytes de todos los archivos bajo `dir`.
fn bytes_bajo(dir: &Path) -> u64 {
    let Ok(entradas) = std::fs::read_dir(dir) else { return 0 };
    entradas
        .flatten()
        .map(|e| {
            let p = e.path();
            if p.is_dir() {
                bytes_bajo(&p)
            } else {
                e.metadata().map(|m| m.len()).unwrap_or(0)
            }
        })
        .sum()
}

/// Los archivos bajo `dir` que contienen la aguja, en UTF-8 o en UTF-16 (los registros de Windows suelen ir en UTF-16).
fn con_la_aguja(dir: &Path, aguja: &str) -> Vec<PathBuf> {
    let utf16: Vec<u8> = aguja.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let mut hallados = Vec::new();
    let Ok(entradas) = std::fs::read_dir(dir) else { return hallados };
    for e in entradas.flatten() {
        let p = e.path();
        if p.is_dir() {
            hallados.extend(con_la_aguja(&p, aguja));
        } else if let Ok(b) = std::fs::read(&p) {
            let esta = |n: &[u8]| b.windows(n.len()).any(|w| w == n);
            if esta(aguja.as_bytes()) || esta(&utf16) {
                hallados.push(p);
            }
        }
    }
    hallados
}

#[test]
#[ignore = "en vivo: baja el modelo (fila M1 de la matriz, con el «sí» del usuario) y lo mide"]
fn en_vivo_mide_el_modelo_local() {
    let dir = PathBuf::from(std::env::var("AG_FOUNDRY_DIR").expect("AG_FOUNDRY_DIR: la carpeta del modelo"));
    let cual = std::env::var("AG_FOUNDRY_MODELO").expect("AG_FOUNDRY_MODELO: el id de la variante elegida");
    let g = gestor(&dir);
    let modelo = block_on(g.catalog().get_model_variant(&cual))
        .or_else(|_| block_on(g.catalog().get_model(&cual)))
        .expect("esa variante no está en el catálogo");

    // 1. La descarga, con su tamaño en disco.
    let reloj = Instant::now();
    let ultimo = Arc::new(Mutex::new(-10.0f64));
    let u = ultimo.clone();
    block_on(modelo.download(Some(move |p: f64| {
        let mut l = u.lock().unwrap();
        if p - *l >= 10.0 {
            println!("[medida] bajando {p:.0} %");
            *l = p;
        }
    })))
    .expect("la descarga falló");
    let bajada = reloj.elapsed();
    let en_disco_mb = bytes_bajo(&dir.join("modelos")) / 1_048_576;

    // 2. La carga, y lo que ocupa en memoria.
    let memoria_antes = memoria_mb();
    let reloj = Instant::now();
    block_on(modelo.load()).expect("el modelo no cargó");
    let carga = reloj.elapsed();
    let memoria_cargado = memoria_mb();

    let proveedor: Arc<dyn Proveedor> = Arc::new(Medido { modelo: modelo.clone() });

    // 3. El kit de sugerencias por el camino entero, con el vigía de la red mirando.
    let raiz = concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/kit-de-prueba");
    let kit: Kit =
        serde_json::from_str(&std::fs::read_to_string(format!("{raiz}/preguntas.json")).expect("falta preguntas.json"))
            .unwrap();
    let mut corpus = Corpus::en_memoria().unwrap();
    corpus.indexar(Path::new(&format!("{raiz}/corpus")), &|_| {}).expect("no se indexó el kit");

    let vigia = Vigia::empezar();
    let (mut pedidas, mut fundadas, mut en_su_seccion, mut a_tiempo) = (0, 0, 0, 0);
    let mut descartes: BTreeMap<String, usize> = BTreeMap::new();
    let mut ms = Vec::new();
    let mut muestras = Vec::new();
    let mut un_respaldo = None;
    for c in &kit.preguntas {
        let hallazgos = corpus.buscar(&c.dice, 5).unwrap();
        let Hallazgo::Ficha(f) = armar(&c.dice, &hallazgos) else { continue };
        let Some(peticion) = sintesis::Peticion::nueva(&c.dice, &f.respaldo) else { continue };
        un_respaldo.get_or_insert_with(|| f.respaldo.clone());
        pedidas += 1;
        let r = sintesis::sugerir(proveedor.clone(), &peticion, TECHO_DE_LA_MEDIDA);
        ms.push(r.ms);
        match r.sugerencia {
            Ok(s) => {
                fundadas += 1;
                if r.ms <= sintesis::TECHO.as_millis() as u64 {
                    a_tiempo += 1;
                }
                let v = serde_json::to_value(&s).unwrap();
                if v["ficha"]["fuente"]["seccion"].as_str() == Some(c.espera.as_str()) {
                    en_su_seccion += 1;
                }
                if muestras.len() < 3 {
                    muestras.push(format!("«{}» → {} · {}", c.dice, v["titular"], v["linea"]));
                }
            }
            Err(d) => *descartes.entry(format!("{d:?}").split('(').next().unwrap().to_string()).or_default() += 1,
        }
    }

    // 4. La canaria: un turno marcado por el mismo camino; después se busca en todo lo que el SDK escribió.
    let canaria = format!("CANARIA-S13-{}", std::process::id());
    if let Some(respaldo) = &un_respaldo {
        let turno = format!("¿El {canaria} está dentro del alcance?");
        if let Some(p) = sintesis::Peticion::nueva(&turno, respaldo) {
            let _ = sintesis::sugerir(proveedor.clone(), &p, TECHO_DE_LA_MEDIDA);
        }
    }
    let (tcp, udp, remotos) = vigia.parar();
    block_on(modelo.unload()).expect("el modelo no se descargó de memoria");
    let con_canaria = con_la_aguja(&dir, &canaria);

    let primera = ms.first().copied().unwrap_or(0);
    ms.sort_unstable();
    let pct = |p: f64| ms.get(((ms.len().max(1) as f64 - 1.0) * p).round() as usize).copied().unwrap_or(0);
    println!("\n╭─ la medida del modelo local ──────────────────────────");
    println!("│ modelo           {}", modelo.id());
    println!("│ descarga         {} s · {en_disco_mb} MB en disco", bajada.as_secs());
    println!("│ carga            {} ms · memoria {memoria_antes} → {memoria_cargado} MB", carga.as_millis());
    println!("│ pedidas          {pedidas} de {} preguntas", kit.preguntas.len());
    println!("│ fundadas         {fundadas} de {pedidas} · a tiempo (≤ 6 s) {a_tiempo}");
    println!("│ en su sección    {en_su_seccion} de {pedidas}");
    println!("│ descartadas      {descartes:?}");
    println!("│ latencia         primera {} ms · mediana {} ms · p95 {} ms · máx {} ms (techo 6000)",
        primera, pct(0.5), pct(0.95), pct(1.0));
    println!("│ red del proceso  TCP máx {tcp} · UDP máx {udp} · remotos {remotos:?}");
    println!("│ canaria          {} archivo(s) del SDK la contienen", con_canaria.len());
    for m in &muestras {
        println!("│ muestra          {m}");
    }
    println!("╰───────────────────────────────────────────────────────");
    assert!(con_canaria.is_empty(), "la canaria del turno quedó en disco: {con_canaria:?}");
}

/// Lo que el runtime encola para subir, fuera de la carpeta que se le dio: nombre, bytes y última escritura.
fn almacen_de_eventos() -> Vec<String> {
    let Some(base) = std::env::var_os("LOCALAPPDATA") else { return Vec::new() };
    let dir = PathBuf::from(base).join("Microsoft").join("DeveloperTools").join(".onnxruntime");
    let Ok(entradas) = std::fs::read_dir(&dir) else { return vec!["(no existe)".into()] };
    entradas
        .flatten()
        .filter_map(|e| {
            let m = e.metadata().ok()?;
            let hace = m.modified().ok()?.elapsed().map(|d| d.as_secs()).unwrap_or(0);
            Some(format!("{} {} B (escrito hace {hace} s)", e.file_name().to_string_lossy(), m.len()))
        })
        .collect()
}

/// **La red del modelo local, fase por fase**, con el modelo ya bajado: crear el SDK y buscar el modelo entre los que
/// están en disco, cargarlo, redactar tres sugerencias del kit, dejarlo cargado un minuto en reposo (lo que tarda en
/// salir una subida de eventos) y cerrarlo. Con `AG_FOUNDRY_SIN_TELEMETRIA=1`, lo mismo con los interruptores puestos.
#[test]
#[ignore = "en vivo: carga el modelo ya bajado y mira qué conexiones abre el proceso en cada fase"]
fn en_vivo_red_del_modelo_local() {
    let dir = PathBuf::from(std::env::var("AG_FOUNDRY_DIR").expect("AG_FOUNDRY_DIR: la carpeta del modelo"));
    let cual = std::env::var("AG_FOUNDRY_MODELO").expect("AG_FOUNDRY_MODELO: el id de la variante ya bajada");
    let eventos_antes = almacen_de_eventos();
    let mut fases: Vec<(&str, (usize, usize, BTreeSet<String>))> = Vec::new();

    let v = Vigia::empezar();
    let g = gestor(&dir);
    let en_disco = block_on(g.catalog().get_cached_models()).expect("no se leyeron los modelos en disco");
    let modelo = en_disco.into_iter().find(|m| m.id() == cual).expect("ese modelo no está bajado");
    fases.push(("crear y buscar", v.parar()));

    let v = Vigia::empezar();
    block_on(modelo.load()).expect("el modelo no cargó");
    fases.push(("cargar", v.parar()));

    let raiz = concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/kit-de-prueba");
    let kit: Kit =
        serde_json::from_str(&std::fs::read_to_string(format!("{raiz}/preguntas.json")).expect("falta preguntas.json"))
            .unwrap();
    let mut corpus = Corpus::en_memoria().unwrap();
    corpus.indexar(Path::new(&format!("{raiz}/corpus")), &|_| {}).expect("no se indexó el kit");
    let proveedor: Arc<dyn Proveedor> = Arc::new(Medido { modelo: modelo.clone() });
    let v = Vigia::empezar();
    let mut hechas = 0;
    for c in &kit.preguntas {
        let hallazgos = corpus.buscar(&c.dice, 5).unwrap();
        let Hallazgo::Ficha(f) = armar(&c.dice, &hallazgos) else { continue };
        let Some(p) = sintesis::Peticion::nueva(&c.dice, &f.respaldo) else { continue };
        let _ = sintesis::sugerir(proveedor.clone(), &p, TECHO_DE_LA_MEDIDA);
        hechas += 1;
        if hechas == 3 {
            break;
        }
    }
    fases.push(("redactar 3", v.parar()));

    let v = Vigia::empezar();
    std::thread::sleep(Duration::from_secs(60));
    fases.push(("reposo 60 s", v.parar()));

    let v = Vigia::empezar();
    block_on(modelo.unload()).expect("el modelo no se descargó de memoria");
    let _ = g.shutdown();
    drop(proveedor);
    drop(modelo);
    drop(g);
    std::thread::sleep(Duration::from_secs(10));
    fases.push(("cerrar + 10 s", v.parar()));

    println!("\n╭─ la red del modelo local · telemetría {} ─────────", if sin_telemetria() { "con los interruptores" } else { "por defecto" });
    for (nombre, (tcp, udp, remotos)) in &fases {
        println!("│ {nombre:<16} TCP máx {tcp:>3} · UDP máx {udp:>2} · fuera {remotos:?}");
    }
    println!("│ eventos antes    {eventos_antes:?}");
    println!("│ eventos después  {:?}", almacen_de_eventos());
    println!("╰───────────────────────────────────────────────────────");
}
