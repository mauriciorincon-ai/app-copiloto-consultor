//! **El barrido de Windows, con PowerShell de verdad** (sprint 008, ADR 016 enmienda 2): la misma prueba que el
//! barrido de `/bin/sh` en el Mac (`contra-el-mac-de-verdad.rs`), con el guion que la tarea del Programador corre.
//! No registra ninguna tarea ni toca el Programador: lanza `powershell.exe` con los mismos argumentos de la tarea
//! sobre una carpeta temporal con archivos inventados, y mira qué quedó. Corre en la CI de Windows a secas.
#![cfg(windows)]

use app_copiloto_consultor_lib::vencimiento;
use std::path::PathBuf;
use std::process::Command;

fn carpeta(nombre: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ag-vencimiento-{nombre}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn barrer(lista: &std::path::Path) -> bool {
    // Los argumentos de la tarea van en una sola línea; ninguno lleva espacios dentro (la ruta viaja codificada).
    let argumentos = vencimiento::argumentos_de_windows(&lista.display().to_string());
    Command::new("powershell.exe").args(argumentos.split(' ')).status().is_ok_and(|s| s.success())
}

#[test]
fn el_barrido_de_windows_borra_solo_lo_vencido_de_la_lista() {
    let d = carpeta("barrido");
    let ahora = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let vencida = d.join("vencida.ghost");
    let con_espacios = d.join("con espacios y tildes ñ O'Neil.ghost");
    let futura = d.join("futura.ghost");
    let texto = d.join("vencido.txt");
    let fuera = d.join("fuera-de-la-lista.ghost");
    for f in [&vencida, &con_espacios, &futura, &texto, &fuera] {
        std::fs::write(f, b"x").unwrap();
    }
    let lista = d.join("lista O'Neil").join("vencimientos");
    std::fs::create_dir_all(lista.parent().unwrap()).unwrap();
    let contenido = format!(
        "{}\t{}\n{}\t{}\n{}\t{}\n{}\t{}\nbasura sin tabulador\n",
        ahora - 60,
        vencida.display(),
        ahora - 60,
        con_espacios.display(),
        ahora + 3_600,
        futura.display(),
        ahora - 60,
        texto.display()
    );
    std::fs::write(&lista, &contenido).unwrap();

    assert!(barrer(&lista), "powershell no salió bien");
    assert!(!vencida.exists(), "lo vencido sigue ahí");
    assert!(!con_espacios.exists(), "una ruta con espacios, tildes y comilla no se borró");
    assert!(futura.exists(), "se borró algo que no había vencido");
    assert!(texto.exists(), "se borró un archivo que no es .ghost");
    assert!(fuera.exists(), "se borró algo que no estaba en la lista");
    assert_eq!(std::fs::read_to_string(&lista).unwrap(), contenido, "el barrido escribió la lista");

    // Sin lista, no hace nada y sale bien.
    std::fs::remove_file(&lista).unwrap();
    assert!(barrer(&lista));
    let _ = std::fs::remove_dir_all(&d);
}
