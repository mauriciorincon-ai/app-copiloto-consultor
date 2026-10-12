//! **LA PANTALLA Y LA VOZ DE WINDOWS, CONTRA FIXTURES** (sprint 008, auditoría del S7, M6).
//!
//! El plan del S7 decía «voz y OCR contra fixtures en `build-windows`» y solo se compilaban: `pantalla/windows.rs` y
//! `habla/windows.rs` no tenían ni un test que las ejerciera. Aquí, en el runner efímero de la CI:
//!
//! - **el OCR** lee una diapositiva del kit (`docs/kit-de-prueba/pantalla/tablero-margen.png`, sintética) por el mismo
//!   `Lector` que usa la app, y tiene que sacar palabras de ella;
//! - **la voz** sintetiza una frase a memoria (sin sonar: el runner no tiene altavoces) con la voz del sistema, y el
//!   resultado tiene que ser sonido de verdad: frecuencia razonable, duración razonable, no silencio.
//!
//! **Lo que no mide:** la calidad de lectura (la diapositiva del kit es en español y el runner trae el inglés, así que
//! se piden palabras sueltas y cifras, no frases) ni que la voz suene por los altavoces de nadie: eso es del PC del
//! usuario (gate del H-1). Si el runner no trae el reconocimiento de texto o la voz, **el test es rojo y lo dice**, con
//! los idiomas que sí hay: un test que se salta no es un test.
//!
//! La corre `build-windows` (`--include-ignored`), jamás tu PC: la voz crea un sintetizador y el OCR un motor de lectura.

#![cfg(windows)]

use app_copiloto_consultor_lib::habla;
use app_copiloto_consultor_lib::pantalla::{self, Cuadro};

/// Una imagen del kit, en grises, como la dejaría la captura en el búfer de Rust.
fn cuadro_de(ruta: &str) -> Cuadro {
    let archivo = std::fs::File::open(ruta).unwrap_or_else(|e| panic!("no se pudo abrir {ruta}: {e}"));
    let mut decodificador = png::Decoder::new(archivo);
    decodificador.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut lector = decodificador.read_info().expect("PNG ilegible");
    let mut bytes = vec![0; lector.output_buffer_size()];
    let info = lector.next_frame(&mut bytes).expect("PNG sin cuadro");
    let canales = info.color_type.samples();
    let (ancho, alto) = (info.width as usize, info.height as usize);
    let gris = bytes[..ancho * alto * canales]
        .chunks(canales)
        .map(|p| {
            if canales >= 3 {
                ((p[0] as u32 * 299 + p[1] as u32 * 587 + p[2] as u32 * 114) / 1000) as u8
            } else {
                p[0]
            }
        })
        .collect();
    Cuadro { ancho, alto, gris }
}

#[test]
#[ignore = "hardware: el reconocimiento de texto de Windows sobre una diapositiva del kit; la corre build-windows"]
fn el_ocr_de_windows_lee_la_diapositiva_del_kit() {
    let cuadro = cuadro_de(concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/kit-de-prueba/pantalla/tablero-margen.png"));
    let (_, lector) = pantalla::ojos();
    let lineas = lector.leer(&cuadro).unwrap_or_else(|e| {
        panic!(
            "el lector de Windows no pudo leer la diapositiva: {e}\n\
             (si dice que no hay reconocimiento de texto, el runner no trae el paquete del idioma: \
             Add-WindowsCapability Language.OCR~~~en-US~0.0.1.0)"
        )
    });
    let texto = lineas.iter().map(|l| l.texto.to_lowercase()).collect::<Vec<_>>().join(" ");
    println!("[ocr] {} línea(s) leídas de la diapositiva del kit", lineas.len());
    // La diapositiva trae «Margen por canal», tres canales con su porcentaje y una gráfica. Palabras sueltas y cifras:
    // el motor del runner es el inglés y la diapositiva está en español.
    let esperadas = ["margen", "canal", "mayorista", "vereda", "directa", "18", "23"];
    let halladas: Vec<&&str> = esperadas.iter().filter(|p| texto.contains(**p)).collect();
    assert!(
        halladas.len() >= 3,
        "el OCR de Windows leyó {} línea(s) y solo {} de las {} palabras o cifras de la diapositiva: {halladas:?}",
        lineas.len(),
        halladas.len(),
        esperadas.len()
    );
    // El alto de cada línea sale del rectángulo de sus palabras (Windows no da confianza): el título es la más alta.
    let mas_alta = lineas.iter().map(|l| l.alto).fold(0.0f32, f32::max);
    assert!(mas_alta > 0.0, "ninguna línea trae su alto: el lector no mide el rectángulo de las palabras");
}

#[test]
#[ignore = "hardware: la voz de Windows sintetizando a memoria, sin hacerla sonar; la corre build-windows"]
fn la_voz_de_windows_sintetiza_una_frase_a_memoria() {
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
    // COM multihilo en el hilo de la prueba, que no se deshace: un hilo que lo cierra con objetos de WinRT vivos tumbó el
    // proceso de pruebas en el S7 (A3).
    // SAFETY: iniciar COM en este hilo; si ya estaba en el mismo modo, solo dice «ya estaba».
    let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };

    let idiomas = habla::windows::idiomas_de_las_voces();
    println!("[voz] idiomas de las voces de este equipo: {idiomas:?}");
    let idioma = ["en-US", "es-ES"]
        .into_iter()
        .find(|i| habla::voz().hay_para(i))
        .unwrap_or_else(|| panic!("el runner no trae voz ni para en-US ni para es-ES; trae: {idiomas:?}"));
    let frase = habla::windows::sintetizar_a_memoria(idioma, "The scope includes data cleaning.")
        .unwrap_or_else(|e| panic!("la voz de Windows no sintetizó: {e}"));

    let segundos = frase.muestras.len() as f32 / frase.hz as f32;
    let pico = frase.muestras.iter().fold(0.0f32, |a, m| a.max(m.abs()));
    println!("[voz] {idioma}: {segundos:.2} s a {} Hz, pico {pico:.2}", frase.hz);
    assert!(frase.hz >= 8_000, "la frecuencia {} Hz no es la de una voz", frase.hz);
    assert!((0.5..=15.0).contains(&segundos), "una frase corta no dura {segundos:.2} s");
    assert!(pico > 0.05, "la voz sintetizada es casi silencio (pico {pico})");
}
