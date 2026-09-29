//! **(c) El proveedor externo** — opt-in, con la clave del usuario en su Llavero. ADR 011.
//!
//! Lo que hace, en orden, y nada más:
//! 1. **Tapa** en el Mac los nombres conocidos, los correos, los teléfonos, los números largos y
//!    las parejas de nombres propios ([`super::anonimo::Boveda`]).
//! 2. **Cuenta** los bytes que van a salir, por el único camino de entrada del contador de red, en
//!    el instante antes de enviarlos.
//! 3. **Envía** por el puente de Swift —la única puerta de la app a la red, efímera y https—.
//! 4. **Destapa** la respuesta en el Mac y se la da a `fundar`, que la valida como a cualquier otra.
//!
//! Tres proveedores intercambiables, dos formatos: el de Anthropic y el compatible con OpenAI (que
//! Gemini y Groq también hablan). El precio de cada uno está escrito aquí, con su fecha: el costo
//! que la app enseña es tokens medidos × ese precio.

use super::anonimo::Boveda;
use super::{PorQueNoRedacta, Proveedor, Quien, Respuesta};

/// Los proveedores externos que la app sabe usar. Cerrado: IA los ofrece por su nombre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Externo {
    Claude,
    Gemini,
    Groq,
}

/// Precios de lista por millón de tokens (entrada, salida), en dólares, **a 2026-09-26**. Si el
/// proveedor los cambia, el costo que se enseña se desvía hasta que se actualicen aquí.
impl Externo {
    pub const TODOS: [Externo; 3] = [Externo::Claude, Externo::Gemini, Externo::Groq];

    pub fn nombre(self) -> &'static str {
        match self {
            Externo::Claude => "Claude Haiku",
            Externo::Gemini => "Gemini Flash",
            Externo::Groq => "Groq · Llama",
        }
    }

    fn modelo(self) -> &'static str {
        match self {
            Externo::Claude => "claude-haiku-4-5",
            Externo::Gemini => "gemini-2.5-flash",
            Externo::Groq => "llama-3.3-70b-versatile",
        }
    }

    fn url(self) -> &'static str {
        match self {
            Externo::Claude => "https://api.anthropic.com/v1/messages",
            Externo::Gemini => "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
            Externo::Groq => "https://api.groq.com/openai/v1/chat/completions",
        }
    }

    fn precio(self) -> (f64, f64) {
        match self {
            Externo::Claude => (1.00, 5.00),
            Externo::Gemini => (0.30, 2.50),
            Externo::Groq => (0.59, 0.79),
        }
    }

    /// La cuenta del Llavero de este proveedor.
    fn cuenta(self) -> &'static str {
        match self {
            Externo::Claude => "claude",
            Externo::Gemini => "gemini",
            Externo::Groq => "groq",
        }
    }

    /// Dólares de una petición, con los tokens que el proveedor dice haber usado.
    pub fn costo(self, entrada: u64, salida: u64) -> f64 {
        let (e, s) = self.precio();
        (entrada as f64 * e + salida as f64 * s) / 1_000_000.0
    }

    /// Cabeceras y cuerpo de la petición.
    fn peticion(self, clave: &str, instrucciones: &str, texto: &str) -> (String, String) {
        let cuerpo = match self {
            Externo::Claude => serde_json::json!({
                "model": self.modelo(),
                "max_tokens": 200,
                "system": instrucciones,
                "messages": [{ "role": "user", "content": texto }],
            }),
            Externo::Gemini | Externo::Groq => serde_json::json!({
                "model": self.modelo(),
                "max_tokens": 200,
                "messages": [
                    { "role": "system", "content": instrucciones },
                    { "role": "user", "content": texto },
                ],
            }),
        };
        let cabeceras = match self {
            Externo::Claude => format!(
                "content-type: application/json\nx-api-key: {clave}\nanthropic-version: 2023-06-01"
            ),
            Externo::Gemini | Externo::Groq => {
                format!("content-type: application/json\nauthorization: Bearer {clave}")
            }
        };
        (cabeceras, cuerpo.to_string())
    }

    /// El texto y los tokens de la respuesta.
    fn leer(self, json: &serde_json::Value) -> Option<(String, u64, u64)> {
        match self {
            Externo::Claude => Some((
                json["content"][0]["text"].as_str()?.to_string(),
                json["usage"]["input_tokens"].as_u64().unwrap_or(0),
                json["usage"]["output_tokens"].as_u64().unwrap_or(0),
            )),
            Externo::Gemini | Externo::Groq => Some((
                json["choices"][0]["message"]["content"].as_str()?.to_string(),
                json["usage"]["prompt_tokens"].as_u64().unwrap_or(0),
                json["usage"]["completion_tokens"].as_u64().unwrap_or(0),
            )),
        }
    }
}

#[cfg(all(target_os = "macos", puente_de_swift))]
mod puente {
    use std::os::raw::{c_char, c_double, c_int};

    #[link(name = "agstt", kind = "static")]
    extern "C" {
        pub fn ag_red_post(
            url: *const c_char,
            cabeceras: *const c_char,
            cuerpo: *const u8,
            largo: c_int,
            segundos: c_double,
            salida: *mut u8,
            capacidad: c_int,
            estado: *mut c_int,
        ) -> c_int;
    }
}

// ── El Llavero ──────────────────────────────────────────────────────────────────────────────────
//
// Las claves viven en el Llavero del usuario, servicio «Angel Ghost · API» (`crate::llavero`, que
// desde el sprint 003 comparten las notas y la puerta local).

use crate::llavero::{self, Servicio};

/// Guarda la clave del usuario en su Llavero. Nunca en un archivo.
pub fn guardar_clave(externo: Externo, clave: &str) -> Result<(), String> {
    let clave = clave.trim();
    if clave.is_empty() {
        return Err("la clave está vacía".into());
    }
    llavero::guardar(Servicio::Api, externo.cuenta(), clave)
}

/// Lee la clave, si hay. Vive lo que dura una petición.
fn leer_clave(externo: Externo) -> Option<String> {
    llavero::leer(Servicio::Api, externo.cuenta())
}

/// ¿Hay clave? Se pregunta por los atributos, **sin leer el secreto**: la pantalla IA lo pregunta
/// cada vez que se pinta, y la clave solo se lee en el instante de enviar (auditoría del S2, B1).
pub fn hay_clave(externo: Externo) -> bool {
    llavero::hay(Servicio::Api, externo.cuenta())
}

pub fn borrar_clave(externo: Externo) -> Result<(), String> {
    llavero::borrar(Servicio::Api, externo.cuenta())
}

// ── Lo que salió (auditoría del S2, B37) ────────────────────────────────────────────────────────
//
// ADR 011, punto 4: «el texto exacto que salió, visible en la pantalla IA y solo en memoria, para que
// el usuario lo pueda leer». Hasta el sprint 003 ese texto y la cuenta de lo tapado morían dentro de
// `redactar`, y la pantalla IA no enseñaba ninguno de los dos. Ahora cada petición deja aquí lo que
// salió, trozo a trozo, con **lo que se reemplazó en tu Mac** al lado de cada marcador —lo que la
// maqueta dibuja tachado—. Vive lo que dura la reunión: lo vacían el corte y el final de la sesión,
// lo pide solo la ventana principal por comando, y cada copia se pisa al soltarse.

/// Un trozo de lo que salió: texto enviado tal cual, o un dato que la bóveda tapó.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "que", rename_all = "kebab-case")]
pub enum Trozo {
    Texto { texto: String },
    /// `marcador` es lo que salió («[CLIENTE_1]»); `original`, lo que se quedó en el Mac.
    Tapado { marcador: String, original: String },
}

impl Trozo {
    fn pisar(&mut self) {
        let textos: Vec<&mut String> = match self {
            Trozo::Texto { texto } => vec![texto],
            Trozo::Tapado { marcador, original } => vec![marcador, original],
        };
        for t in textos {
            // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
            unsafe { t.as_mut_vec() }.fill(0);
        }
    }
}

/// Una petición al proveedor externo, como la enseña IA.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoQueSalio {
    #[serde(skip)]
    pub id: u64,
    pub hora: String,
    pub externo: Externo,
    /// El titular de la ficha que la provocó.
    pub sobre: String,
    pub trozos: Vec<Trozo>,
    /// Los caracteres del texto que salió, marcadores incluidos.
    pub caracteres: usize,
    /// Cuántos datos tapó la bóveda en esta petición.
    pub tapadas: usize,
    /// Lo que costó, cuando el proveedor contesta. `None` mientras tanto, o si no contestó.
    pub usd: Option<f64>,
}

impl LoQueSalio {
    pub fn de(tapado: &str, boveda: &Boveda, externo: Externo, sobre: &str) -> Self {
        Self {
            id: 0,
            hora: crate::escucha::la_hora(),
            externo,
            sobre: sobre.to_string(),
            trozos: trozos(tapado, boveda),
            caracteres: tapado.chars().count(),
            tapadas: boveda.tapadas(),
            usd: None,
        }
    }
}

impl Drop for LoQueSalio {
    fn drop(&mut self) {
        for t in &mut self.trozos {
            t.pisar();
        }
        // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
        unsafe { self.sobre.as_mut_vec() }.fill(0);
    }
}

/// Parte el texto que salió en trozos: cada marcador que la bóveda puso, con su original al lado.
/// Un «[» que no es un marcador de la bóveda es texto.
pub fn trozos(tapado: &str, boveda: &Boveda) -> Vec<Trozo> {
    let mut salida = Vec::new();
    let mut texto = String::new();
    let mut resto = tapado;
    while let Some(i) = resto.find('[') {
        let tras = &resto[i..];
        let marcador = tras.find(']').map(|j| &tras[..=j]);
        match marcador.and_then(|m| boveda.original_de(m).map(|o| (m, o))) {
            Some((m, o)) => {
                texto.push_str(&resto[..i]);
                if !texto.is_empty() {
                    salida.push(Trozo::Texto { texto: std::mem::take(&mut texto) });
                }
                salida.push(Trozo::Tapado { marcador: m.to_string(), original: o.to_string() });
                resto = &tras[m.len()..];
            }
            None => {
                texto.push_str(&resto[..=i]);
                resto = &resto[i + 1..];
            }
        }
    }
    texto.push_str(resto);
    if !texto.is_empty() {
        salida.push(Trozo::Texto { texto });
    }
    salida
}

/// Cuántas peticiones recuerda el registro. Cada una lleva un turno anonimizado del cliente y, en
/// claro, lo que se tapó: sin tope, con el API encendido, guardaba la reunión entera, mucho más allá
/// del anillo de 30 s (auditoría del S3, B15). Al pasarlo, la más vieja se suelta y se pisa.
pub const TOPE_DE_LO_QUE_SALIO: usize = 20;

/// **Las peticiones de la reunión.** Lo comparten la sesión —que lo enseña y lo vacía— y cada `Api`.
#[derive(Clone, Default)]
pub struct Registro {
    peticiones: std::sync::Arc<std::sync::Mutex<Vec<LoQueSalio>>>,
    siguiente: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl Registro {
    /// Anota una petición y devuelve con qué número, para ponerle su costo cuando vuelva.
    pub fn anotar(&self, mut salio: LoQueSalio) -> u64 {
        let id = self.siguiente.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        salio.id = id;
        if let Ok(mut p) = self.peticiones.lock() {
            p.push(salio);
            if p.len() > TOPE_DE_LO_QUE_SALIO {
                // `LoQueSalio` se pisa al soltarse (su `Drop`).
                p.remove(0);
            }
        }
        id
    }

    /// Le pone su costo a una petición que volvió. Si el registro ya se vació —el corte llegó
    /// antes—, no hay nada que anotar y no pasa nada.
    pub fn cobrar(&self, id: u64, usd: f64) {
        if let Ok(mut p) = self.peticiones.lock() {
            if let Some(s) = p.iter_mut().find(|s| s.id == id) {
                s.usd = Some(usd);
            }
        }
    }

    /// De la más nueva a la más vieja, que es como las lee la tabla.
    pub fn todas(&self) -> Vec<LoQueSalio> {
        self.peticiones.lock().map(|p| p.iter().rev().cloned().collect()).unwrap_or_default()
    }

    pub fn cuantas(&self) -> usize {
        self.peticiones.lock().map(|p| p.len()).unwrap_or(0)
    }

    /// Se olvida todo (cada `LoQueSalio` se pisa al soltarse). El corte y el final de la reunión.
    pub fn vaciar(&self) {
        if let Ok(mut p) = self.peticiones.lock() {
            p.clear();
        }
    }
}

// ── El proveedor ────────────────────────────────────────────────────────────────────────────────

/// El proveedor externo de UNA petición: con los nombres que hay que tapar.
pub struct Api {
    pub externo: Externo,
    pub conocidos: Vec<String>,
    /// Dónde queda anotado lo que salió, para que IA lo enseñe (B37).
    pub registro: Registro,
    /// De qué ficha va la petición: su titular, para la columna «por qué salió».
    pub sobre: String,
    /// La época del corte en que nació la petición. Si `⌥⎋` llega antes de enviar, no se envía.
    pub vigencia: Vigencia,
}

/// **El corte alcanza a la petición en camino** (auditoría del S2, M2). La sesión sube la época en
/// cada `⌥⎋`; una petición que nació en la época anterior no sale a la red, aunque ya estuviera
/// armada. Lo que ya salió, salió: esto cierra la ventana entre armarla y enviarla.
#[derive(Clone, Debug, Default)]
pub struct Vigencia {
    epoca: std::sync::Arc<std::sync::atomic::AtomicU64>,
    desde: u64,
}

impl Vigencia {
    pub fn desde_ahora(epoca: &std::sync::Arc<std::sync::atomic::AtomicU64>) -> Self {
        Self { epoca: epoca.clone(), desde: epoca.load(std::sync::atomic::Ordering::SeqCst) }
    }
    pub fn sigue(&self) -> bool {
        self.epoca.load(std::sync::atomic::Ordering::SeqCst) == self.desde
    }
}

/// Lo que devuelve un envío que el corte alcanzó antes de salir.
pub const CORTADA: &str = "cortada antes de salir";

/// Lo que se envía de verdad, para que un test pueda mirarlo sin red: el texto ya tapado.
pub fn lo_que_sale(conocidos: &[String], instrucciones: &str, texto: &str) -> (String, Boveda) {
    let mut b = Boveda::nueva(conocidos);
    let tapado = b.tapar(texto);
    let _ = instrucciones;
    (tapado, b)
}

impl Proveedor for Api {
    fn quien(&self) -> Quien {
        Quien::Api
    }

    fn nombre(&self) -> String {
        self.externo.nombre().into()
    }

    fn disponible(&self) -> Result<(), PorQueNoRedacta> {
        if hay_clave(self.externo) { Ok(()) } else { Err(PorQueNoRedacta::SinClave) }
    }

    #[cfg(all(target_os = "macos", puente_de_swift))]
    fn redactar(&self, instrucciones: &str, texto: &str) -> Result<Respuesta, String> {
        use std::ffi::CString;
        if !self.vigencia.sigue() {
            return Err(CORTADA.into());
        }
        let mut clave = leer_clave(self.externo).ok_or("sin clave en el Llavero")?;
        let (mut tapado, boveda) = lo_que_sale(&self.conocidos, instrucciones, texto);
        let (mut cabeceras, mut cuerpo) = self.externo.peticion(&clave, instrucciones, &tapado);
        // Lo que IA enseñará, armado ANTES de pisar el texto: el mismo `tapado` que va en el cuerpo.
        let salio = LoQueSalio::de(&tapado, &boveda, self.externo, &self.sobre);
        // SEGURIDAD (las tres): ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
        unsafe { clave.as_mut_vec() }.fill(0);
        unsafe { tapado.as_mut_vec() }.fill(0);
        let bytes_fuera = (cuerpo.len() + cabeceras.len()) as u64;
        // Otra vez, lo más cerca posible de la red: el corte pudo llegar mientras se armaba.
        if !self.vigencia.sigue() {
            unsafe { cabeceras.as_mut_vec() }.fill(0);
            unsafe { cuerpo.as_mut_vec() }.fill(0);
            return Err(CORTADA.into());
        }
        // Se cuenta ANTES de enviar: un envío que falla a medias también salió. Y se anota ya, por
        // la misma razón: lo que salió se enseña aunque el proveedor no conteste nunca.
        crate::red::registrar_salida(bytes_fuera);
        let id = self.registro.anotar(salio);
        let url = CString::new(self.externo.url()).map_err(|e| e.to_string())?;
        let cab = CString::new(cabeceras.clone()).map_err(|e| e.to_string())?;
        unsafe { cabeceras.as_mut_vec() }.fill(0);
        let mut salida = vec![0u8; 64 * 1024];
        let mut estado: i32 = 0;
        // SEGURIDAD: textos terminados en cero, un cuerpo de `largo` bytes y una salida de
        // `capacidad` bytes, todos vivos hasta que la llamada vuelve.
        let n = unsafe {
            puente::ag_red_post(
                url.as_ptr(),
                cab.as_ptr(),
                cuerpo.as_ptr(),
                cuerpo.len() as i32,
                super::TECHO.as_secs_f64() + 1.0,
                salida.as_mut_ptr(),
                salida.len() as i32,
                &mut estado,
            )
        };
        let mut cab = cab.into_bytes();
        cab.fill(0);
        unsafe { cuerpo.as_mut_vec() }.fill(0);
        if n < 0 {
            return Err(format!("el proveedor no contestó (código {n})"));
        }
        let json: serde_json::Value = serde_json::from_slice(&salida[..n as usize])
            .map_err(|_| format!("respuesta ilegible (HTTP {estado})"))?;
        salida.fill(0);
        if !(200..300).contains(&estado) {
            return Err(format!("HTTP {estado}"));
        }
        let (texto, entrada, salida_tokens) =
            self.externo.leer(&json).ok_or(format!("HTTP {estado} sin texto"))?;
        Ok(Respuesta {
            json: boveda.destapar(&texto),
            bytes_fuera,
            tokens_entrada: entrada,
            tokens_salida: salida_tokens,
            salida: Some(id),
        })
    }

    #[cfg(not(all(target_os = "macos", puente_de_swift)))]
    fn redactar(&self, _i: &str, _t: &str) -> Result<Respuesta, String> {
        if !self.vigencia.sigue() {
            return Err(CORTADA.into());
        }
        Err("esta compilación no trae el puente de red".into())
    }
}

#[cfg(test)]
mod pruebas {
    use super::super::{pruebas::respaldo, Peticion};
    use super::*;

    /// **Solo texto anonimizado al API** (acceptance del sprint): lo que se enviaría no lleva los
    /// nombres plantados en el turno ni en las fichas.
    /// **B37: IA enseña el texto exacto que salió, con lo reemplazado al lado.** Los trozos, unidos,
    /// son exactamente el texto que se envió; cada marcador lleva su original; y la cuenta de lo
    /// tapado es la de la bóveda. ¿Puede fallar? Sí: con `trozos` devolviendo el texto de un solo
    /// trozo, o con `tapadas: 0`, es rojo (bitácora del sprint 003).
    #[test]
    fn lo_que_salio_es_el_texto_exacto_con_lo_reemplazado_al_lado() {
        let p = Peticion::nueva("Andrea Villalba de Páramo Azul pregunta [algo] por el alcance", &respaldo()).unwrap();
        let conocidos = vec!["Páramo Azul".to_string()];
        let (fuera, b) = lo_que_sale(&conocidos, Peticion::instrucciones(), &p.texto());
        let salio = LoQueSalio::de(&fuera, &b, Externo::Claude, "Alcance");
        let unido: String = salio
            .trozos
            .iter()
            .map(|t| match t {
                Trozo::Texto { texto } => texto.clone(),
                Trozo::Tapado { marcador, .. } => marcador.clone(),
            })
            .collect();
        assert_eq!(unido, fuera, "lo que IA enseña no es lo que salió");
        let tapados: Vec<(&str, &str)> = salio
            .trozos
            .iter()
            .filter_map(|t| match t {
                Trozo::Tapado { marcador, original } => Some((marcador.as_str(), original.as_str())),
                _ => None,
            })
            .collect();
        assert!(tapados.contains(&("[CLIENTE_1]", "Páramo Azul")), "{tapados:?}");
        assert!(tapados.contains(&("[PERSONA_1]", "Andrea Villalba")), "{tapados:?}");
        assert_eq!(salio.tapadas, b.tapadas());
        assert!(salio.tapadas >= 2);
        assert_eq!(salio.caracteres, fuera.chars().count());
        // «[algo]» no es un marcador de la bóveda: se queda como texto.
        assert!(unido.contains("[algo]"));
    }

    /// El registro de la reunión: anota, le pone costo a la que vuelve, y se vacía entero.
    #[test]
    fn el_registro_anota_cobra_y_se_vacia() {
        let r = Registro::default();
        let b = Boveda::nueva(&[]);
        let uno = r.anotar(LoQueSalio::de("uno", &b, Externo::Claude, "A"));
        let dos = r.anotar(LoQueSalio::de("dos", &b, Externo::Groq, "B"));
        assert_ne!(uno, dos);
        r.cobrar(uno, 0.004);
        let todas = r.todas();
        assert_eq!(todas.iter().map(|s| s.sobre.as_str()).collect::<Vec<_>>(), ["B", "A"], "la más nueva primero");
        assert_eq!(todas[1].usd, Some(0.004));
        assert_eq!(todas[0].usd, None);
        r.vaciar();
        assert_eq!(r.cuantas(), 0);
        // Cobrar una que ya no está no hace nada.
        r.cobrar(dos, 1.0);
        assert_eq!(r.cuantas(), 0);
    }

    /// **El registro tiene tope** (auditoría del S3, B15): 25 peticiones dejan las 20 últimas y la
    /// primera ya no está. Demostrado en rojo sin el `remove(0)`: quedaban 25.
    #[test]
    fn el_registro_recuerda_solo_las_ultimas() {
        let r = Registro::default();
        let b = Boveda::nueva(&[]);
        for n in 1..=25 {
            r.anotar(LoQueSalio::de("x", &b, Externo::Claude, &format!("ficha {n}")));
        }
        assert_eq!(r.cuantas(), TOPE_DE_LO_QUE_SALIO);
        let sobres: Vec<String> = r.todas().into_iter().map(|s| s.sobre.clone()).collect();
        assert!(!sobres.contains(&"ficha 1".to_string()), "la más vieja sigue en memoria");
        assert_eq!(sobres.first().map(String::as_str), Some("ficha 25"));
    }

    #[test]
    fn lo_que_sale_al_api_no_lleva_los_nombres_plantados() {
        let p = Peticion::nueva(
            "Andrea Villalba de Páramo Azul pregunta por la Cooperativa Sur del Valle: escríbele a andrea@paramo.co",
            &respaldo(),
        )
        .unwrap();
        let conocidos = vec!["Páramo Azul".to_string(), "Cooperativa Sur del Valle".to_string(), "Sur del Valle".to_string()];
        let (fuera, b) = lo_que_sale(&conocidos, Peticion::instrucciones(), &p.texto());
        for plantado in ["Andrea Villalba", "Páramo Azul", "Sur del Valle", "andrea@paramo.co"] {
            assert!(!fuera.contains(plantado), "«{plantado}» saldría del Mac:\n{fuera}");
        }
        assert!(b.tapadas() >= 4);
        // Y las fichas siguen diciendo lo suyo: las cifras no se tocan.
        assert!(fuera.contains("9 semanas"), "{fuera}");
    }

    /// **El corte alcanza a la petición armada** (auditoría del S2, M2): con la época subida entre
    /// nacer y enviar, no sale nada y el contador de red no se mueve.
    #[test]
    fn una_peticion_de_antes_del_corte_no_sale() {
        let epoca = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(7));
        let vigencia = Vigencia::desde_ahora(&epoca);
        assert!(vigencia.sigue());
        epoca.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        assert!(!vigencia.sigue());
        // Sale antes de leer la clave o de contar un byte: el error es el del corte, no el de «sin
        // clave» ni el del puente.
        let api = Api {
            externo: Externo::Claude,
            conocidos: Vec::new(),
            registro: Registro::default(),
            sobre: String::new(),
            vigencia,
        };
        assert_eq!(api.redactar("i", "t").err().as_deref(), Some(CORTADA));
    }

    #[test]
    fn el_costo_es_tokens_por_precio() {
        // 1 000 de entrada y 100 de salida en Claude Haiku: 0,001 + 0,0005 dólares.
        assert!((Externo::Claude.costo(1_000, 100) - 0.0015).abs() < 1e-9);
        assert_eq!(Externo::Groq.costo(0, 0), 0.0);
    }

    #[test]
    fn cada_proveedor_arma_su_peticion_y_lee_su_respuesta() {
        let (cab, cuerpo) = Externo::Claude.peticion("k", "sys", "hola");
        assert!(cab.contains("x-api-key: k") && cuerpo.contains("\"system\":\"sys\""));
        let (cab, cuerpo) = Externo::Groq.peticion("k", "sys", "hola");
        assert!(cab.contains("Bearer k") && cuerpo.contains("\"role\":\"system\""));
        let claude = serde_json::json!({"content":[{"text":"{}"}],"usage":{"input_tokens":10,"output_tokens":2}});
        assert_eq!(Externo::Claude.leer(&claude), Some(("{}".into(), 10, 2)));
        let groq = serde_json::json!({"choices":[{"message":{"content":"{}"}}],"usage":{"prompt_tokens":7,"completion_tokens":3}});
        assert_eq!(Externo::Gemini.leer(&groq), Some(("{}".into(), 7, 3)));
        for e in Externo::TODOS {
            assert!(e.url().starts_with("https://"), "{e:?} sin https");
        }
    }
}
