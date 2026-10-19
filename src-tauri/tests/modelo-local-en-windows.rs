//! **LA MEDIDA DEL MODELO LOCAL EN WINDOWS, DENTRO DEL PROCESO** (sprint 014, fase 1; ADR 023 enmienda 2).
//!
//! El S13 midió Foundry Local y lo dejó fuera: cabía en el tiempo, pero abría una conexión a la telemetría de Microsoft
//! (anexo del ADR 023). Este arnés es aquel, adaptado a `llama.cpp` (`llama-cpp-2`), que corre **dentro de este
//! proceso** y no trae red: se mide la descarga por la puerta de la app, la carga, la latencia sobre el kit de
//! sugerencias por el camino entero (`sugerir` y `fundar`), la canaria del turno buscada en todo lo que se escribió y las
//! conexiones del proceso fase por fase. Se construye solo si cumple las cuatro: 0 conexiones TCP del proceso en todas
//! las fases · canaria en 0 archivos · mediana < 6 s · ≥ 9 de 17 sugerencias pasan `fundar`.
//!
//! Una prueba corre en `cargo test` a secas y en la CI: que `llama.cpp` y `whisper.cpp` enlazan en el mismo binario con
//! un solo `ggml` (los dos crates enlazan estáticas con el mismo nombre; ver `Cargo.toml`). Lo demás es `en_vivo_` e
//! `#[ignore]`: ni la CI (`--skip en_vivo_`) ni `cargo test` a secas lo corren (regla 25).
//!
//! Uso, en este orden y **en dos procesos** (la conexión de la descarga no ensucia la medida de la red):
//!   1. con el «sí» a la fila M1: `cargo test -j 4 --test modelo-local-en-windows en_vivo_baja -- --ignored --nocapture`
//!   2. `cargo test -j 4 --test modelo-local-en-windows en_vivo_mide -- --ignored --nocapture`
//!      `AG_LLAMA_GRAMATICA=1` ata la salida al esquema JSON (gramática de llama.cpp) · `AG_LLAMA_HILOS=<n>` fija los
//!      hilos · `AG_LLAMA_MODELO=<ruta de un .gguf>` mide otro archivo ya bajado.
#![cfg(windows)]

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::CStr;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};

use app_copiloto_consultor_lib::corpus::evaluar::Kit;
use app_copiloto_consultor_lib::corpus::Corpus;
use app_copiloto_consultor_lib::ficha::{armar, Respuesta as Hallazgo};
use app_copiloto_consultor_lib::modelo_whisper::{self, Modelo};
use app_copiloto_consultor_lib::sintesis::{self, PorQueNoRedacta, Proveedor, Quien, Respuesta};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::token::LlamaToken;

/// **El candidato** (leído el 2026-10-19 de la API de Hugging Face, revisión fija): Qwen2.5-1.5B-Instruct, cuantizado
/// Q4_K_M, licencia apache-2.0. Baja por `modelo_whisper::bajar`, la única puerta a la red escrita en Rust, con su
/// tamaño exacto y su huella; ninguna línea de red nueva.
const QWEN: Modelo = Modelo {
    nombre: "qwen2.5-1.5b-instruct-q4_k_m",
    archivo: "qwen2.5-1.5b-instruct-q4_k_m.gguf",
    bytes: 1_117_320_736,
    sha256: "6a1a2eb6d15622bf3c96857206351ba97e1af16c30d7a74ee38970e434e9407e",
    origen: "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/91cad51170dc346986eccefdc2dd33a9da36ead9/qwen2.5-1.5b-instruct-q4_k_m.gguf",
};

/// El techo de la orden es [`sintesis::TECHO`]; la medida se toma con uno holgado para ver la latencia de verdad,
/// también la que no cabe.
const TECHO_DE_LA_MEDIDA: Duration = Duration::from_secs(120);

/// Lo más que se deja escribir al modelo: el JSON del esquema cabe holgado en 160 piezas.
const PIEZAS_MAXIMAS: usize = 160;

/// **El esquema cerrado como gramática** (segundo intento sin bajar nada): el modelo no puede escribir otra cosa que
/// `{titular, linea, fuente, confianza}` con una fuente F1–F3 y una confianza de las tres. `fundar` decide igual.
const GRAMATICA: &str = r#"
root      ::= "{" ws "\"titular\"" ws ":" ws cadena ws "," ws "\"linea\"" ws ":" ws cadena ws "," ws "\"fuente\"" ws ":" ws fuente ws "," ws "\"confianza\"" ws ":" ws confianza ws "}"
cadena    ::= "\"" ( [^"\\\x7F\x00-\x1F] | "\\" ["\\/bfnrt] )* "\""
fuente    ::= "\"F" [1-3] "\""
confianza ::= "\"" ( "alta" | "media" | "baja" ) "\""
ws        ::= [ \t\n]?
"#;

/// El backend de llama.cpp se inicia una vez por proceso, con sus registros apagados: lo que imprime incluye trozos
/// del prompt (y el prompt lleva el turno del cliente).
fn backend() -> &'static LlamaBackend {
    static B: OnceLock<LlamaBackend> = OnceLock::new();
    B.get_or_init(|| {
        let mut b = LlamaBackend::init().expect("el backend de llama.cpp no arrancó");
        b.void_logs();
        b
    })
}

fn hilos() -> i32 {
    std::env::var("AG_LLAMA_HILOS")
        .ok()
        .and_then(|h| h.parse().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get() as i32).unwrap_or(4))
}

fn con_gramatica() -> bool {
    std::env::var("AG_LLAMA_GRAMATICA").as_deref() == Ok("1")
}

/// **llama.cpp y whisper.cpp en el mismo binario, con un solo ggml.** Corre en `cargo test` a secas y en la CI: si una
/// subida de cualquiera de los dos crates trae otro ggml, el enlace o esta prueba se rompen aquí y no en la Surface del
/// usuario. Solo pregunta por la CPU: no abre ningún dispositivo.
#[test]
fn llama_y_whisper_enlazan_con_un_solo_ggml() {
    let _ = backend();
    let whisper = whisper_rs::print_system_info();
    // SAFETY: las dos devuelven cadenas estáticas de la biblioteca, terminadas en cero.
    let (llama, version) = unsafe {
        (
            CStr::from_ptr(llama_cpp_sys_2::llama_print_system_info()).to_string_lossy().into_owned(),
            CStr::from_ptr(llama_cpp_sys_2::ggml_version()).to_string_lossy().into_owned(),
        )
    };
    println!("whisper: {whisper}\nllama:   {llama}\nggml:    {version}");
    assert!(whisper.contains("CPU"), "whisper no ve la CPU: {whisper}");
    assert!(llama.contains("CPU"), "llama no ve la CPU: {llama}");
    assert_eq!(version, "0.9.5", "el ggml del binario no es el de whisper-rs-sys 0.15 (ver Cargo.toml)");
}

/// **El proveedor de la medida**: una generación con el prompt real de la app. Es el que `sintesis/local.rs` construirá
/// si la medida cumple; aquí vive solo para medir. **Un contexto por sugerencia**, que se suelta al terminar: su caché
/// guardaría el turno del cliente de una sugerencia a la siguiente. Lo que lleva el turno se pisa con ceros al acabar.
struct Medido {
    modelo: Arc<LlamaModel>,
    nombre: String,
}

/// ¿Ya se cerró el primer objeto JSON? Así no se espera al fin de turno del modelo para lo que ya está escrito.
fn objeto_cerrado(b: &[u8]) -> bool {
    let (mut hondo, mut en_cadena, mut escape, mut abierto) = (0i32, false, false, false);
    for &c in b {
        if en_cadena {
            match (escape, c) {
                (true, _) => escape = false,
                (false, b'\\') => escape = true,
                (false, b'"') => en_cadena = false,
                _ => {}
            }
            continue;
        }
        match c {
            b'"' if abierto => en_cadena = true,
            b'{' => {
                hondo += 1;
                abierto = true;
            }
            b'}' if abierto => {
                hondo -= 1;
                if hondo == 0 {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn pisar_bytes(b: &mut [u8]) {
    b.fill(0);
}

impl Medido {
    fn generar(&self, instrucciones: &str, texto: &str) -> Result<(Vec<u8>, u64, u64), String> {
        let plantilla = self.modelo.chat_template(None).map_err(|e| format!("sin plantilla de chat: {e}"))?;
        let mensajes = vec![
            LlamaChatMessage::new("system".into(), instrucciones.into()).map_err(|e| e.to_string())?,
            LlamaChatMessage::new("user".into(), texto.into()).map_err(|e| e.to_string())?,
        ];
        let mut prompt = self.modelo.apply_chat_template(&plantilla, &mensajes, true).map_err(|e| e.to_string())?;
        drop(mensajes);
        let tokens = self.modelo.str_to_token(&prompt, AddBos::Always).map_err(|e| e.to_string());
        // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
        unsafe { prompt.as_mut_vec() }.fill(0);
        let mut tokens = tokens?;
        let n = hilos();
        let params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(2048))
            .with_n_batch(2048)
            .with_n_threads(n)
            .with_n_threads_batch(n);
        let mut ctx = self.modelo.new_context(backend(), params).map_err(|e| e.to_string())?;
        let mut lote = LlamaBatch::new(tokens.len().max(1), 1);
        let ultimo = tokens.len().saturating_sub(1);
        for (i, t) in tokens.iter().enumerate() {
            lote.add(*t, i as i32, &[0], i == ultimo).map_err(|e| e.to_string())?;
        }
        let entrada = tokens.len() as u64;
        tokens.fill(LlamaToken(0));
        ctx.decode(&mut lote).map_err(|e| e.to_string())?;
        let mut muestreo = if con_gramatica() {
            let g = LlamaSampler::grammar(&self.modelo, GRAMATICA, "root").map_err(|e| format!("gramática: {e}"))?;
            LlamaSampler::chain_simple([g, LlamaSampler::greedy()])
        } else {
            LlamaSampler::greedy()
        };
        let mut salida = Vec::new();
        let mut pos = entrada as i32;
        let mut piezas = 0u64;
        while (piezas as usize) < PIEZAS_MAXIMAS {
            let t = muestreo.sample(&ctx, lote.n_tokens() - 1);
            if self.modelo.is_eog_token(t) {
                break;
            }
            let pieza = match self.modelo.token_to_piece_bytes(t, 32, false, None) {
                Err(llama_cpp_2::TokenToStringError::InsufficientBufferSpace(i)) => {
                    self.modelo.token_to_piece_bytes(t, (-i) as usize, false, None)
                }
                otra => otra,
            }
            .map_err(|e| e.to_string())?;
            salida.extend_from_slice(&pieza);
            piezas += 1;
            if objeto_cerrado(&salida) {
                break;
            }
            lote.clear();
            lote.add(t, pos, &[0], true).map_err(|e| e.to_string())?;
            pos += 1;
            ctx.decode(&mut lote).map_err(|e| e.to_string())?;
        }
        // El contexto (y su caché, que guarda el turno) se suelta aquí, al salir.
        drop(ctx);
        Ok((salida, entrada, piezas))
    }
}

impl Proveedor for Medido {
    /// En F1 no hay `Quien::Local` todavía (lo trae F2): para medir da igual quién firma.
    fn quien(&self) -> Quien {
        Quien::Sistema
    }
    fn nombre(&self) -> String {
        self.nombre.clone()
    }
    fn disponible(&self) -> Result<(), PorQueNoRedacta> {
        Ok(())
    }
    fn redactar(&self, instrucciones: &str, texto: &str) -> Result<Respuesta, String> {
        let (mut salida, entrada, piezas) = self.generar(instrucciones, texto)?;
        let json = String::from_utf8_lossy(&salida).into_owned();
        pisar_bytes(&mut salida);
        Ok(Respuesta { json, tokens_entrada: entrada, tokens_salida: piezas, ..Default::default() })
    }
}

/// **El vigía de la red**: mientras dura una fase, pregunta al sistema (`netstat -ano`, unas tres veces por segundo)
/// qué conexiones TCP y qué puertos UDP son de este proceso. Solo direcciones y estados, nunca contenido. Una conexión
/// que nace y muere entre dos miradas se le escapa: por eso la fase de reposo dura un minuto.
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
        let hilo = std::thread::spawn(move || loop {
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
                        g.2.insert(format!("{} → {} {}", c.get(1).copied().unwrap_or(""), c.get(2).copied().unwrap_or(""), c.get(3).copied().unwrap_or("")));
                    }
                } else {
                    g.1 = g.1.max(mias.len());
                }
            }
            // Una última mirada después de que se pide parar: lo que abrió el final de la fase también cuenta.
            if p.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(Duration::from_millis(150));
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

/// Lo que ocupa el proceso en memoria, en MB: el conjunto de trabajo y su pico.
fn memoria_mb() -> (u64, u64) {
    let orden = format!("$p = Get-Process -Id {}; \"$($p.WorkingSet64) $($p.PeakWorkingSet64)\"", std::process::id());
    let s = std::process::Command::new("powershell").args(["-NoProfile", "-Command", &orden]).output().ok();
    let t = s.map(|s| String::from_utf8_lossy(&s.stdout).trim().to_string()).unwrap_or_default();
    let mut n = t.split_whitespace().map(|x| x.parse::<u64>().unwrap_or(0) / 1_048_576);
    (n.next().unwrap_or(0), n.next().unwrap_or(0))
}

/// Los archivos bajo `dir` escritos desde `desde` que contienen la aguja, en UTF-8 o en UTF-16 (los registros de
/// Windows suelen ir en UTF-16). Se salta lo que no deja leer, el modelo mismo y las carpetas `target` (compilaciones).
fn con_la_aguja(dir: &Path, aguja: &str, desde: SystemTime, mirados: &mut u64, hallados: &mut Vec<PathBuf>) {
    let utf16: Vec<u8> = aguja.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let Ok(entradas) = std::fs::read_dir(dir) else { return };
    for e in entradas.flatten() {
        let p = e.path();
        let Ok(tipo) = e.file_type() else { continue };
        if tipo.is_symlink() {
            continue;
        }
        if tipo.is_dir() {
            if e.file_name() != "target" {
                con_la_aguja(&p, aguja, desde, mirados, hallados);
            }
            continue;
        }
        let Ok(m) = e.metadata() else { continue };
        let reciente = m.modified().is_ok_and(|t| t >= desde) || m.created().is_ok_and(|t| t >= desde);
        if !reciente || m.len() > 512 * 1_048_576 || p.extension().is_some_and(|x| x == "gguf") {
            continue;
        }
        *mirados += 1;
        if let Ok(b) = std::fs::read(&p) {
            let esta = |n: &[u8]| b.windows(n.len()).any(|w| w == n);
            if esta(aguja.as_bytes()) || esta(&utf16) {
                hallados.push(p);
            }
        }
    }
}

#[test]
#[ignore = "en vivo: baja el modelo por la puerta de la app (fila M1 de la matriz, con el «sí» del usuario)"]
fn en_vivo_baja_el_modelo_local() {
    let reloj = Instant::now();
    let r = modelo_whisper::bajar(&QWEN).expect("la descarga falló");
    let m = std::fs::metadata(&r).unwrap();
    println!("\n╭─ la descarga del modelo local");
    println!("│ archivo   {}", r.display());
    println!("│ bytes     {} (esperados {}) · tamaño y huella SHA-256 comprobados por la puerta", m.len(), QWEN.bytes);
    println!("│ tiempo    {} s", reloj.elapsed().as_secs());
    println!("╰─");
    assert_eq!(m.len(), QWEN.bytes);
}

#[test]
#[ignore = "en vivo: carga el modelo ya bajado y lo mide (latencia, fundar, canaria y red del proceso por fase)"]
fn en_vivo_mide_el_modelo_local() {
    let desde = SystemTime::now();
    let ruta = std::env::var_os("AG_LLAMA_MODELO")
        .map(PathBuf::from)
        .or_else(|| modelo_whisper::listo(&QWEN))
        .expect("el modelo no está bajado: corre antes en_vivo_baja (fila M1)");
    let en_disco = std::fs::metadata(&ruta).map(|m| m.len()).unwrap_or(0);
    let mut fases: Vec<(&str, (usize, usize, BTreeSet<String>))> = Vec::new();

    // 1. Arrancar el backend.
    let (mem_inicio, _) = memoria_mb();
    let v = Vigia::empezar();
    let reloj = Instant::now();
    let _ = backend();
    let arranque = reloj.elapsed();
    fases.push(("arrancar", v.parar()));

    // 2. Cargar el modelo.
    let v = Vigia::empezar();
    let reloj = Instant::now();
    let modelo = Arc::new(LlamaModel::load_from_file(backend(), &ruta, &LlamaModelParams::default()).expect("el modelo no cargó"));
    let carga = reloj.elapsed();
    fases.push(("cargar", v.parar()));
    let (mem_cargado, _) = memoria_mb();
    let nombre = ruta.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let proveedor: Arc<dyn Proveedor> = Arc::new(Medido { modelo: modelo.clone(), nombre: nombre.clone() });

    // 3. El kit de sugerencias por el camino entero, y la canaria al final.
    let raiz = concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/kit-de-prueba");
    let kit: Kit =
        serde_json::from_str(&std::fs::read_to_string(format!("{raiz}/preguntas.json")).expect("falta preguntas.json"))
            .unwrap();
    let mut corpus = Corpus::en_memoria().unwrap();
    corpus.indexar(Path::new(&format!("{raiz}/corpus")), &|_| {}).expect("no se indexó el kit");

    let v = Vigia::empezar();
    let (mut pedidas, mut fundadas, mut en_su_seccion, mut a_tiempo) = (0, 0, 0, 0);
    let mut descartes: BTreeMap<String, usize> = BTreeMap::new();
    let mut ms = Vec::new();
    let mut muestras = Vec::new();
    let mut un_respaldo = None;
    let mut piezas = Vec::new();
    for c in &kit.preguntas {
        let hallazgos = corpus.buscar(&c.dice, 5).unwrap();
        let Hallazgo::Ficha(f) = armar(&c.dice, &hallazgos) else { continue };
        let Some(peticion) = sintesis::Peticion::nueva(&c.dice, &f.respaldo) else { continue };
        un_respaldo.get_or_insert_with(|| f.respaldo.clone());
        pedidas += 1;
        let r = sintesis::sugerir(proveedor.clone(), &peticion, TECHO_DE_LA_MEDIDA);
        ms.push(r.ms);
        piezas.push(r.respuesta.tokens_salida);
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
    // La canaria: un turno marcado por el mismo camino. Nunca se imprime: la salida de esta prueba puede ir a un archivo.
    let canaria = format!("CANARIA-S14-{}", std::process::id());
    if let Some(respaldo) = &un_respaldo {
        let turno = format!("¿El {canaria} está dentro del alcance?");
        if let Some(p) = sintesis::Peticion::nueva(&turno, respaldo) {
            let _ = sintesis::sugerir(proveedor.clone(), &p, TECHO_DE_LA_MEDIDA);
        }
    }
    fases.push(("redactar 17 + 1", v.parar()));
    let (mem_tras, mem_pico) = memoria_mb();

    // 4. Reposar un minuto con el modelo cargado: lo que tardaría en salir una subida de eventos, si la hubiera.
    let v = Vigia::empezar();
    std::thread::sleep(Duration::from_secs(60));
    fases.push(("reposar 60 s", v.parar()));

    // 5. Soltar el modelo y esperar diez segundos.
    let v = Vigia::empezar();
    drop(proveedor);
    drop(modelo);
    std::thread::sleep(Duration::from_secs(10));
    fases.push(("soltar + 10 s", v.parar()));
    let (mem_suelto, _) = memoria_mb();

    // 6. La canaria buscada en todo lo escrito desde que empezó la prueba: la carpeta de la app (con el modelo), los
    //    datos locales y de roaming del usuario (temporales incluidos) y el repo.
    let mut donde: Vec<PathBuf> = ["LOCALAPPDATA", "APPDATA"].iter().filter_map(std::env::var_os).map(PathBuf::from).collect();
    donde.push(std::env::temp_dir());
    donde.push(PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")));
    let (mut mirados, mut con_canaria) = (0u64, Vec::new());
    let reloj = Instant::now();
    for d in &donde {
        con_la_aguja(d, &canaria, desde, &mut mirados, &mut con_canaria);
    }
    let busqueda = reloj.elapsed();

    let primera = ms.first().copied().unwrap_or(0);
    let mut orden = ms.clone();
    orden.sort_unstable();
    let pct = |p: f64| orden.get(((orden.len().max(1) as f64 - 1.0) * p).round() as usize).copied().unwrap_or(0);
    let tcp_total: usize = fases.iter().map(|(_, (t, _, _))| *t).sum();
    println!("\n╭─ la medida del modelo local (llama.cpp en el proceso) ─────────────");
    println!("│ modelo           {nombre} · {en_disco} bytes en disco");
    println!("│ variante         {} · {} hilos · {} piezas máx.", if con_gramatica() { "con gramática JSON" } else { "sin gramática" }, hilos(), PIEZAS_MAXIMAS);
    println!("│ arranque         {} ms", arranque.as_millis());
    println!("│ carga            {} ms", carga.as_millis());
    println!("│ memoria (MB)     inicio {mem_inicio} · cargado {mem_cargado} · tras redactar {mem_tras} · pico {mem_pico} · suelto {mem_suelto}");
    println!("│ pedidas          {pedidas} de {} preguntas", kit.preguntas.len());
    println!("│ fundadas         {fundadas} de {pedidas} · a tiempo (≤ 6 s) {a_tiempo}");
    println!("│ en su sección    {en_su_seccion} de {pedidas}");
    println!("│ descartadas      {descartes:?}");
    println!("│ latencia         primera {primera} ms · mediana {} ms · p95 {} ms · máx {} ms (techo 6000)", pct(0.5), pct(0.95), pct(1.0));
    println!("│ cada una (ms)    {ms:?}");
    println!("│ piezas escritas  {piezas:?}");
    for (nombre, (tcp, udp, conexiones)) in &fases {
        println!("│ red · {nombre:<16} TCP máx {tcp} · UDP máx {udp} · {conexiones:?}");
    }
    println!("│ canaria          {} archivo(s) la contienen · {mirados} archivos escritos durante la prueba mirados en {} s", con_canaria.len(), busqueda.as_secs());
    for m in &muestras {
        println!("│ muestra          {m}");
    }
    println!("╰────────────────────────────────────────────────────────────────────");
    assert!(con_canaria.is_empty(), "la canaria del turno quedó en disco: {con_canaria:?}");
    assert_eq!(tcp_total, 0, "el proceso abrió conexiones TCP: {fases:?}");
}
