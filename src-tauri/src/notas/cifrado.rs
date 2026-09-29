//! EL CIFRADO DEL ARCHIVO DE UNA REUNIÓN (ADR 015 §3 y §4).
//!
//! Puro: bytes en claro y una llave entran, bytes sellados salen, y al revés. Quién guarda la llave
//! (el Llavero) y quién escribe los bytes (`carpeta.rs`) viven fuera, del lado que sí toca disco.
//!
//! ```text
//!  6 B  «AGHOST»            ─┐
//!  1 B  versión = 1          ├─ cabecera EN CLARO, autenticada (datos asociados)
//!  8 B  vence (i64 BE, s)   ─┘  0 = «siempre»
//! 24 B  nonce al azar
//!  N B  XChaCha20-Poly1305(contenido) + 16 B de etiqueta
//! ```
//!
//! **El vencimiento va en claro** para que borrar lo vencido no necesite la llave, y **autenticado**
//! para que alargarle la vida a un archivo por fuera lo deje sin abrir.

use chacha20poly1305::aead::{Aead, Generate, KeyInit, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};

/// Los seis bytes con que empieza todo archivo de notas.
pub const MAGIA: &[u8; 6] = b"AGHOST";
/// La única versión que existe. Otra se rechaza con su nombre, no se intenta.
pub const VERSION: u8 = 1;
/// Magia + versión + vencimiento: lo que va en claro y se autentica.
pub const CABECERA: usize = 6 + 1 + 8;
const NONCE: usize = 24;
const ETIQUETA: usize = 16;
/// Bytes de la llave.
pub const LARGO_DE_LA_LLAVE: usize = 32;

/// La llave de tus notas. Se pone a cero al soltarla.
pub struct Llave([u8; LARGO_DE_LA_LLAVE]);

impl Llave {
    /// Una llave nueva, del generador del sistema.
    pub fn nueva() -> Llave {
        let mut k = Key::generate();
        let mut bytes = [0u8; LARGO_DE_LA_LLAVE];
        bytes.copy_from_slice(k.as_slice());
        // La copia del generador se pisa antes de soltarse (B26).
        k.as_mut_slice().fill(0);
        Llave(bytes)
    }

    /// Como se guarda en el Llavero, que guarda cadenas: 64 cifras hexadecimales.
    pub fn a_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// La vuelta de [`Llave::a_hex`]. Cualquier otra cosa —otro largo, otra letra— no es una llave.
    pub fn de_hex(texto: &str) -> Option<Llave> {
        let texto = texto.trim();
        if texto.len() != LARGO_DE_LA_LLAVE * 2 || !texto.is_ascii() {
            return None;
        }
        let mut bytes = [0u8; LARGO_DE_LA_LLAVE];
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = u8::from_str_radix(&texto[i * 2..i * 2 + 2], 16).ok()?;
        }
        Some(Llave(bytes))
    }

    /// ¿Es la misma llave? **En tiempo constante**, como el token de la puerta, y sin volverla texto.
    pub fn igual(&self, otra: &Llave) -> bool {
        self.0.iter().zip(otra.0.iter()).fold(0u8, |d, (x, y)| d | (x ^ y)) == 0
    }

    fn cifrador(&self) -> XChaCha20Poly1305 {
        // Desde la llave misma, sin una `Key` intermedia que nadie pisaría (auditoría del S3, B26).
        XChaCha20Poly1305::new_from_slice(&self.0).expect("la llave mide lo que pide el cifrado")
    }
}

impl Drop for Llave {
    fn drop(&mut self) {
        self.0.fill(0);
    }
}

/// Por qué un archivo no se abre. Cada caso con su nombre, para decírselo al usuario tal cual.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoSeAbre {
    /// No empieza por «AGHOST»: no es un archivo de notas de esta app.
    NoEsDeNotas,
    /// Es de notas, de una versión que esta app no conoce.
    Version(u8),
    /// Le faltan bytes: se cortó al copiarlo o al escribirlo.
    Truncado,
    /// La etiqueta no cuadra: lo cambió alguien por fuera, o se cifró con otra llave (otro Mac, un
    /// Llavero borrado). Desde aquí no se puede distinguir, y no se finge que sí.
    Manipulado,
}

impl std::fmt::Display for NoSeAbre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoSeAbre::NoEsDeNotas => write!(f, "no es un archivo de notas de Angel Ghost"),
            NoSeAbre::Version(v) => write!(f, "es de una versión que esta app no conoce ({v})"),
            NoSeAbre::Truncado => write!(f, "está incompleto"),
            NoSeAbre::Manipulado => write!(f, "no se puede abrir: se cambió por fuera o es de otra llave"),
        }
    }
}

fn cabecera(vence: i64) -> [u8; CABECERA] {
    let mut c = [0u8; CABECERA];
    c[..6].copy_from_slice(MAGIA);
    c[6] = VERSION;
    c[7..].copy_from_slice(&vence.to_be_bytes());
    c
}

/// Sella el contenido. `vence` son segundos Unix; 0 es «siempre».
pub fn sellar(llave: &Llave, vence: i64, claro: &[u8]) -> Vec<u8> {
    let cab = cabecera(vence);
    let nonce = XNonce::generate();
    let cifrado = llave
        .cifrador()
        .encrypt(&nonce, Payload { msg: claro, aad: &cab })
        // Con una llave del largo justo y un mensaje en memoria, el único error posible es un
        // mensaje de más de 256 GiB. Una reunión no llega.
        .expect("XChaCha20-Poly1305 no pudo sellar un mensaje en memoria");
    let mut salida = Vec::with_capacity(CABECERA + NONCE + cifrado.len());
    salida.extend_from_slice(&cab);
    salida.extend_from_slice(nonce.as_slice());
    salida.extend_from_slice(&cifrado);
    salida
}

/// Lee la cabecera **sin la llave**: cuándo vence. Es lo que usa el barrido.
pub fn vence_de(sellado: &[u8]) -> Result<i64, NoSeAbre> {
    if sellado.len() < MAGIA.len() || &sellado[..MAGIA.len()] != MAGIA {
        return Err(NoSeAbre::NoEsDeNotas);
    }
    if sellado.len() < CABECERA {
        return Err(NoSeAbre::Truncado);
    }
    if sellado[6] != VERSION {
        return Err(NoSeAbre::Version(sellado[6]));
    }
    let mut v = [0u8; 8];
    v.copy_from_slice(&sellado[7..CABECERA]);
    Ok(i64::from_be_bytes(v))
}

/// Abre un archivo sellado. O el contenido entero, o por qué no: jamás medio contenido.
pub fn abrir(llave: &Llave, sellado: &[u8]) -> Result<Vec<u8>, NoSeAbre> {
    vence_de(sellado)?;
    if sellado.len() < CABECERA + NONCE + ETIQUETA {
        return Err(NoSeAbre::Truncado);
    }
    let (cab, resto) = sellado.split_at(CABECERA);
    let (nonce, cifrado) = resto.split_at(NONCE);
    let nonce = XNonce::try_from(nonce).map_err(|_| NoSeAbre::Truncado)?;
    llave
        .cifrador()
        .decrypt(&nonce, Payload { msg: cifrado, aad: cab })
        .map_err(|_| NoSeAbre::Manipulado)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    const TEXTO: &[u8] = "Piden la cuarta fuente. Fecha real: 12 semanas.".as_bytes();

    #[test]
    fn lo_que_se_sella_se_abre_igual() {
        let llave = Llave::nueva();
        let sellado = sellar(&llave, 1_800_000_000, TEXTO);
        assert_eq!(abrir(&llave, &sellado).unwrap(), TEXTO);
        assert_eq!(vence_de(&sellado).unwrap(), 1_800_000_000);
    }

    /// **La llave no deja copias sin pisar** (auditoría del S3, B26). Se compara byte a byte en tiempo
    /// constante, sin volverla texto hexadecimal (dos `String` que nadie pisaba), y el cifrador se arma
    /// sin una `Key` intermedia. Demostrado en rojo con el código de antes: `la_llave` pasaba las dos
    /// llaves a hexadecimal para compararlas.
    #[test]
    fn la_llave_se_compara_sin_volverse_texto() {
        let a = Llave::nueva();
        let b = Llave::de_hex(&a.a_hex()).unwrap();
        assert!(a.igual(&b) && !a.igual(&Llave::nueva()));
        for (archivo, fuente) in [("carpeta.rs", include_str!("../carpeta.rs")), ("cifrado.rs", include_str!("cifrado.rs"))] {
            let aguja = concat!("a_hex()", " !=");
            assert!(!fuente.contains(aguja), "{archivo} compara llaves pasándolas a texto");
        }
        assert!(!include_str!("cifrado.rs").contains(concat!("Key::", "from(self.0)")), "el cifrador copia la llave en una Key intermedia");
    }

    #[test]
    fn el_texto_no_se_ve_en_el_archivo() {
        let sellado = sellar(&Llave::nueva(), 0, TEXTO);
        let crudo = String::from_utf8_lossy(&sellado);
        assert!(!crudo.contains("cuarta fuente"), "el contenido viaja en claro");
        assert!(sellado.starts_with(MAGIA));
    }

    #[test]
    fn dos_sellados_del_mismo_texto_no_se_parecen() {
        // El nonce es al azar: dos guardados de la misma nota no delatan que son la misma.
        let llave = Llave::nueva();
        assert_ne!(sellar(&llave, 0, TEXTO), sellar(&llave, 0, TEXTO));
    }

    #[test]
    fn un_byte_cambiado_no_se_abre() {
        let llave = Llave::nueva();
        let mut sellado = sellar(&llave, 0, TEXTO);
        let ultimo = sellado.len() - 1;
        sellado[ultimo] ^= 1;
        assert_eq!(abrir(&llave, &sellado), Err(NoSeAbre::Manipulado));
    }

    #[test]
    fn alargarle_la_vida_por_fuera_lo_deja_sin_abrir() {
        let llave = Llave::nueva();
        let mut sellado = sellar(&llave, 1_800_000_000, TEXTO);
        sellado[7..CABECERA].copy_from_slice(&0i64.to_be_bytes()); // «siempre»
        assert_eq!(vence_de(&sellado).unwrap(), 0, "la cabecera sí se deja leer");
        assert_eq!(abrir(&llave, &sellado), Err(NoSeAbre::Manipulado), "pero el archivo ya no abre");
    }

    #[test]
    fn otra_llave_no_abre() {
        let sellado = sellar(&Llave::nueva(), 0, TEXTO);
        assert_eq!(abrir(&Llave::nueva(), &sellado), Err(NoSeAbre::Manipulado));
    }

    #[test]
    fn lo_que_no_es_un_archivo_de_notas_se_dice() {
        let llave = Llave::nueva();
        assert_eq!(abrir(&llave, b"# mis notas en claro"), Err(NoSeAbre::NoEsDeNotas));
        let sellado = sellar(&llave, 0, TEXTO);
        assert_eq!(abrir(&llave, &sellado[..CABECERA + 10]), Err(NoSeAbre::Truncado));
        let mut otra = sellado.clone();
        otra[6] = 9;
        assert_eq!(abrir(&llave, &otra), Err(NoSeAbre::Version(9)));
    }

    #[test]
    fn la_llave_va_y_vuelve_por_el_llavero_como_texto() {
        let llave = Llave::nueva();
        let hex = llave.a_hex();
        assert_eq!(hex.len(), 64);
        let otra = Llave::de_hex(&hex).expect("la vuelta de a_hex");
        let sellado = sellar(&llave, 0, TEXTO);
        assert_eq!(abrir(&otra, &sellado).unwrap(), TEXTO);
        assert!(Llave::de_hex("corta").is_none());
        assert!(Llave::de_hex(&"zz".repeat(32)).is_none());
    }
}
