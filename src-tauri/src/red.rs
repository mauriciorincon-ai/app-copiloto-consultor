//! EL CONTADOR DE RED (B2) — cuántos bytes salieron de este equipo en esta reunión.
//!
//! En el sprint 001 marca **0 siempre**, y no porque nadie lo llame: porque **no hay código que
//! pueda llamarlo**. Esa es la diferencia entre un contador y un adorno, y es lo que el gate
//! `tests/unit/contador-de-red.test.ts` vigila — barre el crate entero buscando cualquier forma
//! de abrir un socket y se pone rojo si aparece una.
//!
//! **Por qué existe ya, vacío.** El contador es la prueba visible de la promesa «nada crudo sale
//! del equipo», y una promesa que se instrumenta cuando llega la primera conexión llega tarde: el
//! día que el adaptador de LLM se encienda (sprint 2, con su ADR), el contador tiene que estar
//! puesto desde antes, con su cero comprobado, o no hay contra qué comparar.
//!
//! **Un solo camino de entrada.** [`registrar_salida`] es el único sitio que suma. No se expone un
//! contador mutable ni se suma desde varios lugares: si mañana hay dos caminos para salir a la
//! red, el segundo tiene que pasar por aquí o el número miente.

use std::sync::atomic::{AtomicU64, Ordering};

static SALIDA: AtomicU64 = AtomicU64::new(0);

/// Suma bytes que **salieron** del equipo. El único camino de entrada al contador.
pub fn registrar_salida(bytes: u64) {
    SALIDA.fetch_add(bytes, Ordering::Relaxed);
}

/// Los bytes que han salido en esta sesión.
pub fn bytes() -> u64 {
    SALIDA.load(Ordering::Relaxed)
}

/// Vuelve a cero. Lo llaman el inicio de una reunión y el **kill-switch**: el contador es «en
/// esta reunión», no «desde que instalaste la app».
pub fn reiniciar() {
    SALIDA.store(0, Ordering::Relaxed);
}

/// El número tal y como lo escribe la maqueta: `0 B`, `1,2 KB`, `3,4 MB`.
///
/// Con **coma decimal**, que es lo que la maqueta usa en español y lo que las cifras de la banda
/// ya escriben. Y con el cero **exacto**: `0 B`, nunca `0,0 B` — un cero con decimales se lee como
/// «casi cero», y aquí la diferencia entre cero y casi cero es la promesa entera.
pub fn formatear(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    match bytes {
        0 => "0 B".to_string(),
        b if b < KB => format!("{b} B"),
        b if b < MB => format!("{} KB", coma(b as f64 / KB as f64)),
        b if b < GB => format!("{} MB", coma(b as f64 / MB as f64)),
        b => format!("{} GB", coma(b as f64 / GB as f64)),
    }
}

fn coma(v: f64) -> String {
    format!("{v:.1}").replace('.', ",")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// El cero se escribe `0 B` y nada más. Un `0,0 B` se lee como «casi cero».
    #[test]
    fn el_cero_es_exacto() {
        assert_eq!(formatear(0), "0 B");
    }

    #[test]
    fn las_cifras_llevan_coma_decimal_como_la_maqueta() {
        assert_eq!(formatear(1536), "1,5 KB");
        assert_eq!(formatear(3_670_016), "3,5 MB");
        assert_eq!(formatear(900), "900 B");
        assert!(!formatear(1536).contains('.'), "punto decimal: la maqueta usa coma");
    }

    /// El contador es un estático del proceso y `cargo test` corre los tests en PARALELO: dos
    /// que lo reinicien a la vez se pisan y el fallo aparece una vez de cada veinte. Un gate
    /// intermitente es peor que no tenerlo —se aprende a reintentar hasta que pasa—, así que los
    /// que tocan el contador se turnan.
    static TURNO: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// El contador es «en esta reunión». Sin esto sumaría desde que se instaló la app y el número
    /// dejaría de significar lo que la pantalla dice que significa.
    #[test]
    fn el_contador_suma_por_un_solo_camino_y_el_kill_switch_lo_vacia() {
        let _turno = TURNO.lock().unwrap_or_else(|e| e.into_inner());
        reiniciar();
        assert_eq!(bytes(), 0);
        registrar_salida(1024);
        registrar_salida(512);
        assert_eq!(bytes(), 1536);
        reiniciar();
        assert_eq!(bytes(), 0, "el kill-switch tiene que dejarlo en cero");
    }

    /// El cero del sprint 001 no es «nadie llamó»: es que **no hay a quién llamar**. Quien lo
    /// vigila de verdad es el gate del lado TypeScript, que barre el crate entero buscando
    /// sockets; esto solo fija el valor de partida.
    #[test]
    fn en_este_sprint_el_contador_arranca_en_cero() {
        let _turno = TURNO.lock().unwrap_or_else(|e| e.into_inner());
        reiniciar();
        assert_eq!(formatear(bytes()), "0 B");
    }
}
