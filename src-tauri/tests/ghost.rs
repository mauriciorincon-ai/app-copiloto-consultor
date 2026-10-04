//! `ghost`, el binario de verdad, desde fuera (ADR 018 §8).
//!
//! **Ninguna de estas pruebas toca el Llavero ni abre la puerta** (regla 22): corren con `HOME`
//! apuntando a una carpeta vacía, así que no hay socket y `ghost` tiene que decir «cerrada» antes de
//! preguntarle nada a macOS. Si alguna vez leyera el Llavero primero, aquí no se vería un diálogo —no
//! hay token que leer—, así que eso lo vigila la última prueba, sobre el código.

use std::process::{Command, Output};

fn ghost(args: &[&str], casa: &std::path::Path, lang: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ghost"))
        .args(args)
        .env("HOME", casa)
        .env("LANG", lang)
        .output()
        .expect("ghost no arrancó")
}

fn casa_vacia(nombre: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("ag-ghost-{nombre}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn version_y_ayuda_no_necesitan_la_puerta() {
    let casa = casa_vacia("version");
    let v = ghost(&["--version"], &casa, "es_CO.UTF-8");
    assert_eq!(v.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&v.stdout).starts_with("ghost "));

    let es = ghost(&["--help"], &casa, "es_CO.UTF-8");
    assert_eq!(es.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&es.stdout).contains("ghost corpus buscar"));
    let en = ghost(&["--help"], &casa, "en_US.UTF-8");
    assert!(String::from_utf8_lossy(&en.stdout).contains("ghost corpus search"));
}

#[test]
fn con_la_puerta_cerrada_lo_dice_y_sale_con_3() {
    let casa = casa_vacia("cerrada");
    for orden in [&["corpus", "reindexar"][..], &["prefs", "leer"], &["ia", "--encender-api"]] {
        let s = ghost(orden, &casa, "es_CO.UTF-8");
        assert_eq!(s.status.code(), Some(3), "{orden:?}");
        assert!(String::from_utf8_lossy(&s.stderr).contains("La puerta está cerrada"), "{orden:?}");
        assert!(s.stdout.is_empty());
    }
    let en = ghost(&["prefs", "get"], &casa, "en_US.UTF-8");
    assert!(String::from_utf8_lossy(&en.stderr).contains("The door is closed"));
}

#[test]
fn una_orden_que_no_existe_es_un_error_de_uso() {
    let casa = casa_vacia("uso");
    let s = ghost(&["borrar", "todo"], &casa, "es_CO.UTF-8");
    assert_eq!(s.status.code(), Some(64));
    assert!(String::from_utf8_lossy(&s.stderr).contains("ghost --help"));
}

#[test]
fn un_kit_que_no_existe_falla_sin_llegar_a_la_puerta() {
    let casa = casa_vacia("kit");
    let s = ghost(&["kit", "/no/existe.json"], &casa, "es_CO.UTF-8");
    assert_eq!(s.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&s.stderr).contains("no se pudo leer"));
}

/// **Primero la puerta, después el Llavero.** Con el orden al revés, cada `ghost` con la puerta
/// cerrada le preguntaría a macOS por una llave que no iba a usarse —y, si quedara un token viejo, el
/// usuario vería el diálogo del Llavero por nada—.
#[test]
fn puerta_antes_que_el_llavero() {
    let codigo = include_str!("../src/bin/ghost.rs");
    let puerta = codigo.find("if !ruta.exists()").expect("ghost ya no mira el socket");
    let llavero = codigo.find("llavero::leer(").expect("ghost ya no lee el Llavero");
    assert!(puerta < llavero, "ghost lee el Llavero antes de mirar si la puerta está abierta");
}
