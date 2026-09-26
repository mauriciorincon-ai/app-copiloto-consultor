//! **La mitad ámbar**: lo que la pantalla de la reunión dice de quién graba.
//!
//! Recibe las líneas que la lectura de pantalla ya sacó de la ventana de la reunión —el radar no
//! captura nada— y busca dos cosas del catálogo `data/radar/avisos.json`: **el aviso de grabación
//! o transcripción** del cliente de videollamada, y **un bot de notas** en la lista de
//! participantes. Es «sábelo»: el cliente tiene derecho a grabar su reunión y a traer su bot, y la
//! app no bloquea nada; solo te cuenta lo que estaba en la pantalla y no ibas a leer mientras
//! hablabas.
//!
//! **Las líneas son texto de un tercero.** Lo que sale de aquí son dos datos del CATÁLOGO —si hay
//! aviso, y el nombre del bot tal y como el catálogo lo escribe—, nunca el texto leído; y las copias
//! normalizadas se pisan antes de soltarse.

use super::catalogo;
use crate::disparo::normalizar;
use std::sync::OnceLock;

/// Lo que el radar ámbar encontró en una lectura de la pantalla.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Aviso {
    /// El cliente de videollamada muestra su aviso de grabación o de transcripción.
    pub grabando: bool,
    /// Los bots de notas presentes, con el nombre del catálogo, en su orden.
    pub bots: Vec<String>,
}

impl Aviso {
    pub fn vacio(&self) -> bool {
        !self.grabando && self.bots.is_empty()
    }
}

/// Los patrones del catálogo ya normalizados, y rodeados de espacios para comparar palabras
/// enteras: «otter ai» no puede encontrarse dentro de «spotter aid».
struct Patrones {
    grabacion: Vec<String>,
    bots: Vec<(String, Vec<String>)>,
}

fn patrones() -> &'static Patrones {
    static P: OnceLock<Patrones> = OnceLock::new();
    P.get_or_init(|| Patrones {
        grabacion: catalogo::grabacion()
            .iter()
            .flat_map(|g| g.frases.iter().map(|f| rodear(&normalizar(f))))
            .collect(),
        bots: catalogo::bots()
            .iter()
            .map(|b| (b.nombre.clone(), b.patrones.iter().map(|p| rodear(&normalizar(p))).collect()))
            .collect(),
    })
}

fn rodear(s: &str) -> String {
    format!(" {} ", s.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// **El cotejo ámbar** sobre las líneas de UNA lectura.
pub fn en_el_texto<'a>(lineas: impl IntoIterator<Item = &'a str>) -> Aviso {
    let p = patrones();
    let mut aviso = Aviso::default();
    for linea in lineas {
        let mut l = rodear(&normalizar(linea));
        if !aviso.grabando && p.grabacion.iter().any(|f| l.contains(f.as_str())) {
            aviso.grabando = true;
        }
        for (nombre, formas) in &p.bots {
            if !aviso.bots.contains(nombre) && formas.iter().any(|f| l.contains(f.as_str())) {
                aviso.bots.push(nombre.clone());
            }
        }
        // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
        unsafe { l.as_mut_vec() }.fill(0);
    }
    // En el orden del catálogo, no en el de la pantalla: dos lecturas de la misma reunión dan el
    // mismo aviso aunque Vision devuelva las líneas en otro orden, y así no se repite en la banda.
    let orden: Vec<&String> = p.bots.iter().map(|(n, _)| n).collect();
    aviso.bots.sort_by_key(|b| orden.iter().position(|n| *n == b));
    aviso
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn una_frase() -> String {
        catalogo::grabacion()[0].frases[0].clone()
    }

    fn un_bot() -> (String, String) {
        let b = &catalogo::bots()[0];
        (b.nombre.clone(), b.patrones[0].clone())
    }

    #[test]
    fn el_aviso_de_grabacion_se_ve_con_otras_mayusculas_y_sin_tildes() {
        let f = una_frase();
        let a = en_el_texto(["Páramo Azul · seguimiento", &f.to_uppercase(), "Margen por canal: 23 %"]);
        assert!(a.grabando, "no vio «{f}»");
        assert!(a.bots.is_empty());
    }

    #[test]
    fn el_bot_sale_con_el_nombre_del_catalogo_y_no_con_lo_leido() {
        let (nombre, patron) = un_bot();
        let a = en_el_texto(["Laura Gómez", &format!("{patron} (Guest)"), "Tú"]);
        assert_eq!(a.bots, vec![nombre]);
        assert!(!a.grabando);
    }

    #[test]
    fn un_trozo_de_palabra_no_es_un_bot_ni_un_aviso() {
        let (_, patron) = un_bot();
        let pegado = format!("x{}x", normalizar(&patron).replace(' ', ""));
        let a = en_el_texto([pegado.as_str(), "Diapositiva 4 de 12", "Recordatorio: agenda"]);
        assert!(a.vacio(), "una palabra que solo contiene el patrón contó: {a:?}");
    }

    #[test]
    fn dos_lecturas_con_las_lineas_en_otro_orden_dan_el_mismo_aviso() {
        let bots = catalogo::bots();
        if bots.len() < 2 {
            return;
        }
        let (a, b) = (&bots[0].patrones[0], &bots[1].patrones[0]);
        assert_eq!(en_el_texto([a.as_str(), b.as_str()]), en_el_texto([b.as_str(), a.as_str()]));
    }
}
