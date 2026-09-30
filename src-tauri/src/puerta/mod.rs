//! LA PUERTA LOCAL PARA TU AGENTE (C16, sprint 003, fase 4, ADR 018). **MÓDULO PROTEGIDO**:
//! `pnpm verify:ephemeral` le prohíbe disco y red, salvo las líneas del socket, que llevan su marca y
//! el ADR en la misma línea.
//!
//! Claude Code, en la sesión del usuario y en este mismo Mac, opera la app por `ghost`
//! (`src/bin/ghost.rs`), que habla con un **socket Unix** en la carpeta privada de la app. La puerta:
//!
//! - **nace cerrada** y no se recuerda: se abre a mano, en IA, y se cierra al salir;
//! - **se cierra sola en reunión**, y en reunión lo deniega todo: un agente no toca jamás lo que está
//!   vivo en memoria;
//! - **pide una llave**: un token por apertura, que la app deja en el Llavero y `ghost` lee de ahí —la
//!   primera vez, macOS le pregunta al usuario—, comparado en tiempo constante;
//! - **tiene una lista cerrada de órdenes** ([`Orden`]): lo que no está se deniega;
//! - **registra lo que pasó** ([`Entrada`]) sin el contenido: qué orden, si se hizo o se denegó y por
//!   qué, a qué hora.
//!
//! Este archivo es la política, pura: qué se pide, qué se deja, qué se registra. El socket vive en
//! [`socket`] y la línea de órdenes de `ghost` en [`cli`].

pub mod cli;
pub mod socket;

use std::collections::VecDeque;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::corpus::evaluar::Kit;

/// El nombre del socket, dentro de la carpeta privada de la app.
pub const SOCKET: &str = "puerta.sock";

/// La cuenta del token dentro del servicio «Angel Ghost · puerta» del Llavero.
pub const CUENTA: &str = "token";

/// La carpeta de la app, igual que la calcula Tauri (`app_data_dir`): `ghost` no tiene Tauri y la
/// tiene que encontrar sola. Lo vigila un test contra `tauri.conf.json`.
pub const IDENTIFICADOR: &str = "com.aiapps.copiloto-consultor";

/// Lo más larga que puede ser una línea: el kit de preguntas cabe de sobra; una línea sin fin, no.
pub const TOPE_DE_LINEA: usize = 1 << 20;

/// Cuántas órdenes recuerda el registro. Es de esta sesión de la app y vive en memoria.
pub const TOPE_DEL_REGISTRO: usize = 50;

/// Cada cuánto mira el vigía si empezó una reunión, mientras la puerta está abierta.
pub const VIGIA: Duration = Duration::from_secs(2);

/// Lo que espera la puerta a que llegue la línea entera. Una orden a medias no la deja colgada.
pub const ESPERA: Duration = Duration::from_secs(5);

/// **Las órdenes que la puerta entiende. Lista cerrada**: lo que no está aquí no se interpreta, se
/// deniega como [`Motivo::OrdenDesconocida`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "que", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Orden {
    /// Las fichas de tu corpus, como las arma la banda.
    Buscar { texto: String },
    /// Vuelve a leer la carpeta del corpus y reintenta los ilegibles.
    Reindexar {},
    /// El kit de evaluación con tus preguntas. `ghost` lee el archivo y manda las preguntas: la app no
    /// abre rutas que le dicte el agente.
    Kit { kit: Kit },
    LeerPrefs {},
    /// Una preferencia de la lista blanca ([`Clave::delegable`]).
    CambiarPref { clave: String, valor: String },
    ListarNotas {},
    /// Una reunión guardada. Pide el desbloqueo en el Mac (ADR 015 §5).
    AbrirNota { archivo: String },
    /// **Existe para denegarse**: gastar tu dinero y sacar texto del Mac son decisiones tuyas.
    EncenderApi {},
}

impl Orden {
    /// La orden como se enseña en el registro: **sin su contenido**. Ni lo que se buscó, ni qué
    /// reunión se abrió, ni una clave que no sea de la lista.
    pub fn sin_contenido(&self) -> String {
        match self {
            Orden::Buscar { .. } => "ghost corpus buscar".into(),
            Orden::Reindexar {} => "ghost corpus reindexar".into(),
            Orden::Kit { .. } => "ghost kit".into(),
            Orden::LeerPrefs {} => "ghost prefs leer".into(),
            Orden::CambiarPref { clave, .. } => match Clave::de(clave) {
                Some(c) => format!("ghost prefs cambiar {}", c.nombre()),
                None => "ghost prefs cambiar ?".into(),
            },
            Orden::ListarNotas {} => "ghost notas listar".into(),
            Orden::AbrirNota { .. } => "ghost notas abrir".into(),
            Orden::EncenderApi {} => "ghost ia --encender-api".into(),
        }
    }
}

/// Las preferencias que la puerta conoce. Las que no son [`delegable`](Clave::delegable) se nombran
/// para poder denegarlas con su motivo, en vez de tratarlas como una clave que no existe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clave {
    IdiomaConsultor,
    IdiomaCliente,
    Retencion,
    VentanaDeLaBandeja,
    LecturaAutomatica,
    Redactar,
    Api,
    Proveedor,
    ConservarMisTurnos,
    Nda,
}

impl Clave {
    pub const TODAS: [Clave; 10] = [
        Clave::IdiomaConsultor,
        Clave::IdiomaCliente,
        Clave::Retencion,
        Clave::VentanaDeLaBandeja,
        Clave::LecturaAutomatica,
        Clave::Redactar,
        Clave::Api,
        Clave::Proveedor,
        Clave::ConservarMisTurnos,
        Clave::Nda,
    ];

    pub fn nombre(self) -> &'static str {
        match self {
            Clave::IdiomaConsultor => "idioma-consultor",
            Clave::IdiomaCliente => "idioma-cliente",
            Clave::Retencion => "retencion",
            Clave::VentanaDeLaBandeja => "ventana-de-la-bandeja",
            Clave::LecturaAutomatica => "lectura-automatica",
            Clave::Redactar => "redactar",
            Clave::Api => "api",
            Clave::Proveedor => "proveedor",
            Clave::ConservarMisTurnos => "conservar-mis-turnos",
            Clave::Nda => "nda",
        }
    }

    pub fn de(nombre: &str) -> Option<Clave> {
        Clave::TODAS.into_iter().find(|c| c.nombre() == nombre)
    }

    /// **Lo que tu agente puede cambiar.** Lo demás es decisión tuya (ADR 018 §5): lo que enciende un
    /// modelo o saca texto del Mac, lo que se guarda de la reunión y lo que respondiste de una NDA.
    pub fn delegable(self) -> bool {
        match self {
            Clave::IdiomaConsultor
            | Clave::IdiomaCliente
            | Clave::Retencion
            | Clave::VentanaDeLaBandeja
            | Clave::LecturaAutomatica => true,
            Clave::Redactar | Clave::Api | Clave::Proveedor | Clave::ConservarMisTurnos | Clave::Nda => false,
        }
    }
}

/// Por qué se denegó.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Motivo {
    /// La llave no es la de esta apertura.
    LlaveErrada,
    /// Hay reunión, o no se puede saber si la hay. La puerta se cierra.
    EnReunion,
    /// Encender el API externo.
    ElApiEsTuyo,
    /// Una preferencia que no es delegable.
    NoDelegable,
    /// Una orden que no está en la lista.
    OrdenDesconocida,
}

/// Lo que vuelve por el socket.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "que", rename_all = "kebab-case")]
pub enum Respuesta {
    Hecho { datos: serde_json::Value },
    Denegado { motivo: Motivo },
    /// Se intentó y no salió («no hay corpus», «esa reunión no existe»). No es una denegación.
    Fallo { error: String },
}

/// Lo que devuelve la app cuando hace una orden: los datos para el agente y, si la hay, una cuenta
/// para el registro («28» documentos, «3» fichas).
#[derive(Clone, Debug, PartialEq)]
pub struct Hecho {
    pub datos: serde_json::Value,
    pub cuenta: Option<u32>,
}

/// Lo que la puerta necesita de la app. La app lo implementa con su estado; las pruebas, con un doble.
pub trait Operaciones: Send + Sync {
    /// ¿Hay reunión? Escuchando, en solo notas, una videollamada detectada **o no se puede saber**.
    fn en_reunion(&self) -> bool;
    /// Hace una orden que la política ya dejó pasar.
    fn hacer(&self, orden: &Orden) -> Result<Hecho, String>;
    /// «11:04», la hora del Mac.
    fn hora(&self) -> String;
    /// La puerta cambió (se abrió, se cerró, atendió una orden): la pantalla se pone al día.
    fn avisar(&self);
}

/// Dónde vive el token. En la app, el Llavero ([`DelLlavero`]); en las pruebas, un doble: **ninguna
/// prueba toca el Llavero de verdad** (regla 22).
pub trait Llave: Send + Sync {
    fn guardar(&self, token: &str) -> Result<(), String>;
    fn borrar(&self);
}

/// El Llavero de macOS, servicio «Angel Ghost · puerta».
pub struct DelLlavero;

impl Llave for DelLlavero {
    fn guardar(&self, token: &str) -> Result<(), String> {
        crate::llavero::guardar(crate::llavero::Servicio::Puerta, CUENTA, token)
    }
    fn borrar(&self) {
        if let Err(e) = crate::llavero::borrar(crate::llavero::Servicio::Puerta, CUENTA) {
            println!("[puerta] el token no se pudo borrar del Llavero: {e}");
        }
    }
}

/// **La política**, pura: con la llave ya comprobada, ¿pasa esta orden? El orden de las preguntas
/// importa: primero la reunión, que lo deniega todo.
pub fn decidir(orden: &Orden, en_reunion: bool) -> Result<(), Motivo> {
    if en_reunion {
        return Err(Motivo::EnReunion);
    }
    match orden {
        Orden::EncenderApi {} => Err(Motivo::ElApiEsTuyo),
        Orden::CambiarPref { clave, .. } => match Clave::de(clave) {
            Some(c) if !c.delegable() => Err(Motivo::NoDelegable),
            // Una clave que no existe no es una denegación: la app contesta con las que hay.
            _ => Ok(()),
        },
        _ => Ok(()),
    }
}

/// El token de una apertura: 32 bytes del generador del sistema, en hexadecimal. Se pisa con ceros
/// al soltarse.
pub struct Token(String);

impl Token {
    pub fn nuevo() -> Token {
        use chacha20poly1305::aead::Generate;
        let mut bytes: [u8; 32] = chacha20poly1305::Key::generate().into();
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect();
        bytes.fill(0);
        Token(hex)
    }

    #[cfg(test)]
    pub fn de(texto: &str) -> Token {
        Token(texto.to_string())
    }

    pub fn como_texto(&self) -> &str {
        &self.0
    }

    /// **En tiempo constante**: cuánto tarda en decir «no» no delata cuántos caracteres acertaste. La
    /// longitud no es secreta —son siempre 64—, así que una distinta se rechaza en el acto.
    pub fn es(&self, otro: &str) -> bool {
        let (a, b) = (self.0.as_bytes(), otro.as_bytes());
        if a.len() != b.len() {
            return false;
        }
        a.iter().zip(b).fold(0u8, |d, (x, y)| d | (x ^ y)) == 0
    }
}

impl Drop for Token {
    fn drop(&mut self) {
        // SEGURIDAD: ceros sobre ASCII siguen siendo UTF-8 válido.
        unsafe { self.0.as_mut_vec() }.fill(0);
    }
}

/// Qué pasó con una orden, para el registro. Un fallo no lleva su error: podría llevar contenido.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "que", rename_all = "kebab-case")]
pub enum Resultado {
    Hecho { cuenta: Option<u32> },
    Denegado { motivo: Motivo },
    Fallo,
}

/// Una línea de «Qué hizo tu agente».
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entrada {
    pub hora: String,
    /// La orden sin su contenido ([`Orden::sin_contenido`]).
    pub orden: String,
    pub resultado: Resultado,
}

/// Resuelve una línea recibida: comprueba la llave, interpreta la orden, aplica la política y, si
/// pasa, la hace. Devuelve lo que vuelve por el socket y lo que va al registro.
///
/// **La llave va primero**: sin ella no se dice ni si hay reunión, que ya sería contarle algo a quien
/// no debe saber nada.
pub fn resolver(token: &Token, linea: &str, ops: &dyn Operaciones) -> (Respuesta, Entrada) {
    let pedido: serde_json::Value = serde_json::from_str(linea).unwrap_or(serde_json::Value::Null);
    let orden = pedido.get("orden").cloned().map(serde_json::from_value::<Orden>);
    let etiqueta = match &orden {
        Some(Ok(o)) => o.sin_contenido(),
        _ => "ghost ?".into(),
    };
    let entrada = |resultado| Entrada { hora: ops.hora(), orden: etiqueta.clone(), resultado };
    let denegar = |motivo| (Respuesta::Denegado { motivo }, entrada(Resultado::Denegado { motivo }));

    let traida = pedido.get("token").and_then(|t| t.as_str()).unwrap_or("");
    if !token.es(traida) {
        return denegar(Motivo::LlaveErrada);
    }
    let Some(Ok(orden)) = orden else {
        return denegar(Motivo::OrdenDesconocida);
    };
    if let Err(motivo) = decidir(&orden, ops.en_reunion()) {
        return denegar(motivo);
    }
    match ops.hacer(&orden) {
        Ok(h) => (Respuesta::Hecho { datos: h.datos }, entrada(Resultado::Hecho { cuenta: h.cuenta })),
        Err(error) => (Respuesta::Fallo { error }, entrada(Resultado::Fallo)),
    }
}

/// Por qué la puerta está cerrada ahora, si se cerró en esta sesión de la app.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Cierre {
    /// La cerraste tú.
    ATuMano,
    /// Se cerró sola: hay reunión (el vigía, una orden en reunión o «Iniciar sesión»).
    EnReunion,
}

/// Por qué no se abrió la última vez que lo intentaste.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoAbre {
    /// Hay reunión, o no se puede saber.
    EnReunion,
    /// La ruta del socket pasa de lo que macOS admite (104 bytes).
    RutaLarga,
    /// El Llavero no guardó el token.
    Llavero,
    /// El sistema no dejó crear el socket.
    Socket,
}

/// Lo que IA enseña de la puerta.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VistaDeLaPuerta {
    pub abierta: bool,
    pub cerro: Option<Cierre>,
    pub no_abre: Option<NoAbre>,
    /// La ruta de `ghost`, si está compilado junto a la app. En H1 no se instala en el PATH.
    pub ghost: Option<String>,
    /// De la más reciente a la más antigua.
    pub registro: Vec<Entrada>,
}

/// El registro de esta sesión de la app: las últimas [`TOPE_DEL_REGISTRO`] órdenes.
#[derive(Default)]
pub struct Registro(VecDeque<Entrada>);

impl Registro {
    pub fn apuntar(&mut self, e: Entrada) {
        if self.0.len() == TOPE_DEL_REGISTRO {
            self.0.pop_back();
        }
        self.0.push_front(e);
    }
    pub fn entradas(&self) -> Vec<Entrada> {
        self.0.iter().cloned().collect()
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Mutex;

    /// Un doble de la app: dice si hay reunión, cuenta lo que hace y guarda la última orden.
    #[derive(Default)]
    pub struct Doble {
        pub reunion: AtomicBool,
        pub hechas: AtomicUsize,
        pub ultima: Mutex<Option<Orden>>,
    }

    impl Operaciones for Doble {
        fn en_reunion(&self) -> bool {
            self.reunion.load(Ordering::SeqCst)
        }
        fn hacer(&self, orden: &Orden) -> Result<Hecho, String> {
            self.hechas.fetch_add(1, Ordering::SeqCst);
            *self.ultima.lock().unwrap() = Some(orden.clone());
            Ok(Hecho { datos: serde_json::json!({"ok": true}), cuenta: Some(3) })
        }
        fn hora(&self) -> String {
            "11:04".into()
        }
        fn avisar(&self) {}
    }

    const BUENO: &str = "ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12";

    fn linea(token: &str, orden: serde_json::Value) -> String {
        serde_json::json!({ "token": token, "orden": orden }).to_string()
    }

    #[test]
    fn con_la_llave_y_sin_reunion_la_orden_se_hace() {
        let doble = Doble::default();
        let (r, e) = resolver(&Token::de(BUENO), &linea(BUENO, serde_json::json!({"que": "reindexar"})), &doble);
        assert!(matches!(r, Respuesta::Hecho { .. }));
        assert_eq!(e.orden, "ghost corpus reindexar");
        assert_eq!(e.resultado, Resultado::Hecho { cuenta: Some(3) });
        assert_eq!(doble.hechas.load(Ordering::SeqCst), 1);
    }

    /// **La llave errada no pasa, y no dice nada más**: ni si hay reunión.
    #[test]
    fn una_llave_errada_se_deniega_antes_que_nada() {
        let doble = Doble::default();
        doble.reunion.store(true, Ordering::SeqCst);
        let malo = BUENO.replace('a', "b");
        for traida in [malo.as_str(), "", "ab12"] {
            let (r, e) = resolver(&Token::de(BUENO), &linea(traida, serde_json::json!({"que": "reindexar"})), &doble);
            assert_eq!(r, Respuesta::Denegado { motivo: Motivo::LlaveErrada }, "llave «{traida}»");
            assert_eq!(e.resultado, Resultado::Denegado { motivo: Motivo::LlaveErrada });
        }
        assert_eq!(doble.hechas.load(Ordering::SeqCst), 0);
    }

    /// **En reunión, nada.** Ni buscar, que es lo más inocente: lo vivo en memoria no se toca.
    #[test]
    fn en_reunion_todo_se_deniega() {
        let doble = Doble::default();
        doble.reunion.store(true, Ordering::SeqCst);
        for orden in [
            serde_json::json!({"que": "buscar", "texto": "limpieza"}),
            serde_json::json!({"que": "leer-prefs"}),
            serde_json::json!({"que": "listar-notas"}),
        ] {
            let (r, _) = resolver(&Token::de(BUENO), &linea(BUENO, orden), &doble);
            assert_eq!(r, Respuesta::Denegado { motivo: Motivo::EnReunion });
        }
        assert_eq!(doble.hechas.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn encender_el_api_se_deniega_siempre() {
        let doble = Doble::default();
        let (r, e) = resolver(&Token::de(BUENO), &linea(BUENO, serde_json::json!({"que": "encender-api"})), &doble);
        assert_eq!(r, Respuesta::Denegado { motivo: Motivo::ElApiEsTuyo });
        assert_eq!(e.orden, "ghost ia --encender-api");
        assert_eq!(doble.hechas.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn lo_que_decides_tu_no_se_delega() {
        let doble = Doble::default();
        for clave in ["redactar", "api", "proveedor", "conservar-mis-turnos", "nda"] {
            let o = serde_json::json!({"que": "cambiar-pref", "clave": clave, "valor": "true"});
            let (r, e) = resolver(&Token::de(BUENO), &linea(BUENO, o), &doble);
            assert_eq!(r, Respuesta::Denegado { motivo: Motivo::NoDelegable }, "«{clave}»");
            assert_eq!(e.orden, format!("ghost prefs cambiar {clave}"));
        }
        let o = serde_json::json!({"que": "cambiar-pref", "clave": "retencion", "valor": "30d"});
        assert!(matches!(resolver(&Token::de(BUENO), &linea(BUENO, o), &doble).0, Respuesta::Hecho { .. }));
    }

    #[test]
    fn una_orden_que_no_esta_en_la_lista_se_deniega() {
        let doble = Doble::default();
        for orden in [
            serde_json::json!({"que": "borrar-todo"}),
            serde_json::json!({"que": "reindexar", "carpeta": "/"}),
            serde_json::json!("reindexar"),
        ] {
            let (r, e) = resolver(&Token::de(BUENO), &linea(BUENO, orden.clone()), &doble);
            assert_eq!(r, Respuesta::Denegado { motivo: Motivo::OrdenDesconocida }, "{orden}");
            assert_eq!(e.orden, "ghost ?");
        }
        let (r, _) = resolver(&Token::de(BUENO), "esto no es json", &doble);
        assert_eq!(r, Respuesta::Denegado { motivo: Motivo::LlaveErrada });
        assert_eq!(doble.hechas.load(Ordering::SeqCst), 0);
    }

    /// **El registro no lleva contenido**: ni lo que se buscó, ni la reunión que se abrió, ni una clave
    /// inventada. Se prueba con una canaria en cada campo que el agente escribe.
    #[test]
    fn el_registro_no_lleva_contenido() {
        let doble = Doble::default();
        let canaria = "canaria-zafiro-77";
        let mut registro = Registro::default();
        for orden in [
            serde_json::json!({"que": "buscar", "texto": canaria}),
            serde_json::json!({"que": "abrir-nota", "archivo": format!("{canaria}.ghost")}),
            serde_json::json!({"que": "cambiar-pref", "clave": canaria, "valor": canaria}),
            serde_json::json!({"que": canaria}),
        ] {
            registro.apuntar(resolver(&Token::de(BUENO), &linea(BUENO, orden), &doble).1);
        }
        let escrito = serde_json::to_string(&registro.entradas()).unwrap();
        assert!(!escrito.contains(canaria), "el registro lleva contenido: {escrito}");
    }

    #[test]
    fn el_registro_guarda_las_ultimas_y_la_mas_reciente_primero() {
        let mut r = Registro::default();
        for i in 0..TOPE_DEL_REGISTRO + 5 {
            r.apuntar(Entrada { hora: format!("{i}"), orden: "ghost kit".into(), resultado: Resultado::Fallo });
        }
        let e = r.entradas();
        assert_eq!(e.len(), TOPE_DEL_REGISTRO);
        assert_eq!(e[0].hora, format!("{}", TOPE_DEL_REGISTRO + 4));
    }

    #[test]
    fn el_token_es_nuevo_cada_vez_y_de_64_caracteres() {
        let (a, b) = (Token::nuevo(), Token::nuevo());
        assert_eq!(a.como_texto().len(), 64);
        assert!(a.como_texto().chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a.como_texto(), b.como_texto());
        assert!(a.es(a.como_texto()));
        assert!(!a.es(b.como_texto()));
    }

    #[test]
    fn cada_clave_tiene_nombre_unico_y_se_encuentra() {
        for c in Clave::TODAS {
            assert_eq!(Clave::de(c.nombre()), Some(c));
        }
        assert_eq!(Clave::de("puerta"), None, "la puerta no es una preferencia: no se recuerda");
    }

    /// `ghost` calcula la carpeta de la app sin Tauri: tiene que ser la misma que Tauri le da.
    #[test]
    fn el_identificador_es_el_de_la_app() {
        let conf: serde_json::Value = serde_json::from_str(include_str!("../../tauri.conf.json")).unwrap();
        assert_eq!(conf["identifier"], IDENTIFICADOR);
        assert_eq!(IDENTIFICADOR, crate::vencimiento::APP);
    }
}
