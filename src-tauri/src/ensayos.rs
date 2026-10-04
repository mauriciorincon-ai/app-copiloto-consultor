//! TUS ENSAYOS — la capa que escribe (ADR 015, enmienda 4).
//!
//! Recibe un [`Guardado`] **ya armado** por `ensayo/guardado.rs` —tus respuestas en texto y sus cifras; el
//! audio nunca llegó hasta ahí— y hace lo que aquel no puede: sellarlo con la llave de tus notas, escribirlo,
//! contarlo, abrirlo para el progreso, borrarlo y barrer lo vencido. Usa la carpeta sellada de `carpeta.rs`
//! —el mismo formato `.ghost`, la misma llave, el mismo `almacen`— con su propio contenido, como la bandeja.
//!
//! Vive en `~/Library/Application Support/<app>/ensayos/`, junto a `notas/` y `bandeja/`: 700 la carpeta y
//! 600 cada archivo. El nombre es `<cliente>-<AAAA-MM-DD>.ghost` y va en claro: con él se cuentan los ensayos
//! de un cliente **sin abrir ninguno**.

use std::path::{Path, PathBuf};

use crate::carpeta::{Carpeta, Guardada, Llaves, Reunion};
use crate::ensayo::guardado::{self, Guardado, Progreso};

/// La carpeta de tus ensayos, dentro de la de la app.
pub const CARPETA: &str = "ensayos";

pub struct Ensayos {
    carpeta: Carpeta,
}

/// El cliente que nombra un archivo de ensayo: `paramo-azul-2026-10-04-2.ghost` → `paramo-azul`. `None` si el
/// nombre no tiene la forma de un ensayo.
pub fn cliente_del_nombre(archivo: &str) -> Option<&str> {
    if !crate::carpeta::nombre_valido(archivo) {
        return None;
    }
    let base = archivo.strip_suffix(&format!(".{}", crate::carpeta::EXTENSION))?;
    // Quita el «-2», «-3»… del final si lo hay, y después la fecha: lo que queda es el cliente.
    let partes: Vec<&str> = base.split('-').collect();
    let es_fecha = |p: &[&str]| {
        p.len() == 3
            && p[0].len() == 4
            && p[1].len() == 2
            && p[2].len() == 2
            && p.iter().all(|x| x.chars().all(|c| c.is_ascii_digit()))
    };
    let n = partes.len();
    let sin_cuenta = if n >= 5 && !es_fecha(&partes[n - 3..]) && es_fecha(&partes[n - 4..n - 1]) { n - 1 } else { n };
    if sin_cuenta < 4 || !es_fecha(&partes[sin_cuenta - 3..sin_cuenta]) {
        return None;
    }
    let largo: usize = partes[..sin_cuenta - 3].iter().map(|p| p.len() + 1).sum::<usize>() - 1;
    Some(&base[..largo])
}

impl Ensayos {
    pub fn en(raiz: PathBuf) -> Ensayos {
        Ensayos { carpeta: Carpeta::en(raiz) }
    }

    pub fn raiz(&self) -> &Path {
        self.carpeta.raiz()
    }

    /// Sella y escribe un ensayo terminado. `base` es el nombre sin extensión (`notas::nombre_del_archivo` con
    /// el cliente); si ya existe, `-2`, `-3`… **Jamás pisa** otro ensayo. `vence` = 0 es «siempre».
    pub fn guardar(&self, llaves: &dyn Llaves, g: &Guardado, base: &str, vence: i64) -> Result<Guardada, String> {
        let archivo = self.carpeta.nombre_para(base).ok_or("no queda un nombre libre para este ensayo")?;
        let mut claro = g.a_bytes();
        let r = self.carpeta.escribir_sellado(llaves, &archivo, &claro, vence);
        claro.fill(0);
        let bytes = r?;
        Ok(Guardada { archivo, bytes, vence })
    }

    /// Todos tus ensayos, sin abrirlos: nombre, tamaño y vencimiento.
    pub fn lista(&self) -> Vec<Reunion> {
        self.carpeta.lista()
    }

    /// Los ensayos de `cliente`, **por el nombre del archivo**: sin la llave.
    pub fn del_cliente(&self, cliente: &str) -> Vec<Reunion> {
        let buscado = crate::notas::slug(cliente);
        self.lista().into_iter().filter(|r| cliente_del_nombre(&r.archivo) == Some(buscado.as_str())).collect()
    }

    /// Abre uno. Quien llama ya pidió el desbloqueo (ADR 015 §5).
    pub fn abrir(&self, llaves: &dyn Llaves, archivo: &str) -> Result<Guardado, String> {
        let (mut claro, _) = self.carpeta.abrir_en_claro(llaves, archivo)?;
        let g = Guardado::de_bytes(&claro);
        claro.fill(0);
        g
    }

    /// **El progreso con `cliente`**: abre sus ensayos y devuelve solo las cifras. Uno que no se deja abrir
    /// (otra llave, manipulado) no tumba a los demás: se salta y se cuenta en el log. Quien llama ya pidió el
    /// desbloqueo.
    pub fn progreso(&self, llaves: &dyn Llaves, cliente: &str) -> Progreso {
        let mut abiertos = Vec::new();
        let mut ilegibles = 0;
        for r in self.del_cliente(cliente) {
            match self.abrir(llaves, &r.archivo) {
                Ok(g) => abiertos.push(g),
                Err(_) => ilegibles += 1,
            }
        }
        // Metadata, jamás contenido: cuántos, nunca de quién.
        println!("[ensayo] progreso: {} ensayo(s) abiertos · {ilegibles} no se dejaron abrir", abiertos.len());
        guardado::progreso(cliente, &abiertos)
    }

    /// «Borrar los ensayos de este cliente»: al momento, sin abrirlos. Devuelve cuántos.
    pub fn borrar_del_cliente(&self, cliente: &str) -> Result<usize, String> {
        let mut borrados = 0;
        for r in self.del_cliente(cliente) {
            self.carpeta.borrar(&r.archivo)?;
            borrados += 1;
        }
        Ok(borrados)
    }

    /// Lo vencido a `ahora`, borrado. Devuelve cuántos.
    pub fn barrer(&self, ahora: i64) -> usize {
        self.carpeta.barrer(ahora)
    }

    /// Lo que vence, para la tarea de launchd. «Siempre» no entra.
    pub fn pendientes(&self) -> Vec<crate::vencimiento::Pendiente> {
        self.carpeta.pendientes()
    }
}

/// **Exportar como texto** el ensayo que acabas de terminar: Markdown, en el idioma de la interfaz, donde
/// elegiste. Nace 600 y **la carpeta que elegiste no se toca** (auditoría del S3, M5). Quien llama ya pidió
/// el desbloqueo.
pub fn exportar(g: &Guardado, destino: &Path, idioma: &str) -> Result<(), String> {
    let mut texto = g.a_texto(idioma);
    let r = crate::almacen::escribir_en_carpeta_ajena(destino, texto.as_bytes());
    // SEGURIDAD: ceros son UTF-8 válido.
    unsafe { texto.as_mut_vec() }.fill(0);
    r
}

/// La línea del log al guardar: **cuánto, nunca qué** (ADR 015 §9 y enmienda 4). Ni el cliente, ni el nombre
/// del archivo, ni una palabra tuya.
pub fn linea_de_log(g: &Guardado, guardada: &Guardada) -> String {
    let i = g.informe();
    format!(
        "[ensayo] guardado y cifrado · {} respondidas · {} saltadas · {} bytes · {}",
        i.respondidas,
        i.saltadas,
        guardada.bytes,
        if guardada.vence == 0 { "sin vencimiento" } else { "con vencimiento" }
    )
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::carpeta::doble::EnMemoria;
    use crate::ensayo::guardado::pruebas::{guardado, pregunta, respondida};
    use crate::ensayo::sesion::Suerte;

    const AHORA: i64 = 1_791_000_000;

    fn carpeta(nombre: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ag-ensayos-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    fn uno(empezo: &str, respuesta: &str) -> Guardado {
        guardado(empezo, vec![(pregunta("¿De dónde sale el 12 %?"), respondida(respuesta, 2, 3, 44_000, Some(140), 1)), (pregunta("¿Y el plazo?"), Suerte::Saltada)])
    }

    #[test]
    fn el_nombre_dice_el_cliente_y_nada_mas() {
        assert_eq!(cliente_del_nombre("paramo-azul-2026-10-04.ghost"), Some("paramo-azul"));
        assert_eq!(cliente_del_nombre("paramo-azul-2026-10-04-2.ghost"), Some("paramo-azul"));
        assert_eq!(cliente_del_nombre("paramo-azul-2026-10-04-12.ghost"), Some("paramo-azul"));
        // Un cliente que termina en cifras sigue siendo él, no una cuenta.
        assert_eq!(cliente_del_nombre("grupo-2026-2026-10-04.ghost"), Some("grupo-2026"));
        assert_eq!(cliente_del_nombre("reunion-2026-10-04-0912.ghost"), Some("reunion"));
        for malo in ["2026-10-04.ghost", "paramo-azul.ghost", "paramo-azul-2026-10.ghost", "../paramo-azul-2026-10-04.ghost", "paramo-azul-2026-10-04.txt"] {
            assert_eq!(cliente_del_nombre(malo), None, "{malo}");
        }
    }

    /// **Un ensayo nace cerrado**: 700 la carpeta, 600 el archivo, cifrado (ni tu respuesta ni el cliente en
    /// claro dentro) y con su vencimiento en la cabecera. Y se vuelve a abrir con la misma llave.
    #[test]
    fn un_ensayo_nace_cifrado_cerrado_y_con_su_vencimiento() {
        use std::os::unix::fs::PermissionsExt;
        let raiz = carpeta("nace");
        let e = Ensayos::en(raiz.clone());
        let llaves = EnMemoria::default();
        let vence = AHORA + 90 * 86_400;
        let g = e.guardar(&llaves, &uno("2026-10-04 09:12", "Sale del cierre de septiembre."), "paramo-azul-2026-10-04", vence).unwrap();
        assert_eq!((g.archivo.as_str(), g.vence), ("paramo-azul-2026-10-04.ghost", vence));
        let ruta = raiz.join(&g.archivo);
        assert_eq!(std::fs::metadata(&ruta).unwrap().permissions().mode() & 0o777, 0o600);
        assert_eq!(std::fs::metadata(&raiz).unwrap().permissions().mode() & 0o777, 0o700);
        let bytes = std::fs::read(&ruta).unwrap();
        let en_claro = String::from_utf8_lossy(&bytes);
        assert!(!en_claro.contains("septiembre") && !en_claro.contains("Páramo"), "el ensayo está en claro");
        assert_eq!(e.lista()[0].vence, vence);
        let abierto = e.abrir(&llaves, &g.archivo).unwrap();
        assert_eq!(abierto.informe().respondidas, 1);
        let _ = std::fs::remove_dir_all(&raiz);
    }

    /// **Dos ensayos el mismo día no se pisan**, y se cuentan por cliente sin abrir ninguno: los de otro
    /// cliente no entran. Borrar los de uno no toca los del otro.
    #[test]
    fn se_cuentan_y_se_borran_por_cliente_sin_abrirlos() {
        let raiz = carpeta("cliente");
        let e = Ensayos::en(raiz.clone());
        let llaves = EnMemoria::default();
        e.guardar(&llaves, &uno("2026-10-04 09:12", "a"), "paramo-azul-2026-10-04", 0).unwrap();
        let segundo = e.guardar(&llaves, &uno("2026-10-04 17:40", "b"), "paramo-azul-2026-10-04", 0).unwrap();
        assert_eq!(segundo.archivo, "paramo-azul-2026-10-04-2.ghost", "el segundo pisó al primero");
        e.guardar(&llaves, &uno("2026-10-03 10:00", "c"), "sur-del-valle-2026-10-03", 0).unwrap();
        assert_eq!(e.del_cliente("Páramo Azul").len(), 2);
        assert_eq!(e.del_cliente("Sur del Valle").len(), 1);
        assert_eq!(e.del_cliente("Páramo").len(), 0, "«Páramo» contó los de «Páramo Azul»");
        assert_eq!(e.borrar_del_cliente("Páramo Azul").unwrap(), 2);
        assert_eq!(e.lista().len(), 1, "borrar los de un cliente tocó los de otro");
        let _ = std::fs::remove_dir_all(&raiz);
    }

    /// **El progreso abre los de ese cliente y salta el que no se deja abrir**: uno sellado con otra llave
    /// no tumba a los demás.
    #[test]
    fn el_progreso_salta_lo_que_no_se_abre() {
        let raiz = carpeta("progreso");
        let e = Ensayos::en(raiz.clone());
        let llaves = EnMemoria::default();
        e.guardar(&llaves, &uno("2026-09-21 10:00", "a"), "paramo-azul-2026-09-21", 0).unwrap();
        e.guardar(&llaves, &uno("2026-09-27 10:00", "b"), "paramo-azul-2026-09-27", 0).unwrap();
        e.guardar(&EnMemoria::default(), &uno("2026-09-30 10:00", "c"), "paramo-azul-2026-09-30", 0).unwrap();
        let p = e.progreso(&llaves, "Páramo Azul");
        assert_eq!(p.filas.len(), 2, "el de otra llave entró, o tumbó a los demás");
        assert_eq!(p.filas[0].empezo, "2026-09-21 10:00");
        let _ = std::fs::remove_dir_all(&raiz);
    }

    /// **La matriz de envejecimiento** (regla 24): lo vencido se barre **en el segundo en que vence** y no un
    /// segundo antes; «siempre» no se barre nunca (ni a +100 días) y no va a la lista de launchd.
    #[test]
    fn lo_vencido_se_barre_justo_al_vencer_y_siempre_no_vence() {
        let raiz = carpeta("barrer");
        let e = Ensayos::en(raiz.clone());
        let llaves = EnMemoria::default();
        let vence = AHORA + 7 * 86_400;
        e.guardar(&llaves, &uno("2026-10-04 09:12", "a"), "paramo-azul-2026-10-04", vence).unwrap();
        e.guardar(&llaves, &uno("2026-10-04 10:00", "b"), "sur-del-valle-2026-10-04", 0).unwrap();
        assert_eq!(e.pendientes().len(), 1, "«siempre» entró en la lista de launchd");
        for (ahora, barridos) in [(AHORA, 0), (vence - 1, 0), (vence, 1), (vence + 1, 0), (AHORA + 100 * 86_400, 0)] {
            assert_eq!(e.barrer(ahora), barridos, "a {} s del vencimiento", ahora - vence);
        }
        assert_eq!(e.lista().len(), 1, "«siempre» se barrió");
        let _ = std::fs::remove_dir_all(&raiz);
    }

    /// **Lo exportado nace 600, en Markdown y con tus respuestas**; la carpeta que elegiste no se toca.
    #[test]
    fn exportar_escribe_el_texto_donde_elegiste() {
        use std::os::unix::fs::PermissionsExt;
        let raiz = carpeta("exportar");
        std::fs::create_dir_all(&raiz).unwrap();
        std::fs::set_permissions(&raiz, std::fs::Permissions::from_mode(0o755)).unwrap();
        let destino = raiz.join("paramo-azul-2026-10-04.md");
        exportar(&uno("2026-10-04 09:12", "Sale del cierre de septiembre."), &destino, "es").unwrap();
        let texto = std::fs::read_to_string(&destino).unwrap();
        assert!(texto.starts_with("# Ensayo · Páramo Azul") && texto.contains("Sale del cierre de septiembre."));
        assert_eq!(std::fs::metadata(&destino).unwrap().permissions().mode() & 0o777, 0o600);
        assert_eq!(std::fs::metadata(&raiz).unwrap().permissions().mode() & 0o777, 0o755, "se tocó la carpeta que elegiste");
        let _ = std::fs::remove_dir_all(&raiz);
    }

    /// **El log dice cuánto, nunca qué**: ni el cliente, ni el nombre del archivo, ni una palabra tuya.
    #[test]
    fn el_log_de_guardar_no_nombra_nada() {
        let g = uno("2026-10-04 09:12", "Sale del cierre de septiembre.");
        let linea = linea_de_log(&g, &Guardada { archivo: "paramo-azul-2026-10-04.ghost".into(), bytes: 812, vence: 1 });
        for prohibido in ["paramo", "Páramo", "septiembre", "12 %", "Rentabilidad"] {
            assert!(!linea.contains(prohibido), "el log lleva «{prohibido}»: {linea}");
        }
        assert!(linea.contains("1 respondidas") && linea.contains("812 bytes"));
    }

    #[test]
    fn un_nombre_que_se_sale_de_la_carpeta_no_se_abre() {
        let e = Ensayos::en(carpeta("nombre"));
        assert!(e.abrir(&EnMemoria::default(), "../notas/x.ghost").is_err());
        assert!(e.abrir(&EnMemoria::default(), "/etc/passwd").is_err());
    }
}
