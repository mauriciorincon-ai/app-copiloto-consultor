//! LA GEOMETRÍA DE LA FRANJA — dónde va la banda, arriba o abajo (sprint 004, ADR 004 enmienda 1).
//!
//! **Pura**: entra un monitor (su marco entero y su área útil) y sale el rectángulo de la franja. Es
//! la única fuente de esa geometría: la banda, su relleno y el acople la piden aquí, y un relleno que
//! calculara la suya por su cuenta sería la franja descubierta por la puerta de al lado.
//!
//! **Arriba** (de fábrica desde el sprint 004, decisión del usuario del 2026-09-27): la franja nace en
//! el borde superior del **área útil**, justo debajo de la barra de menús y del notch. Nunca con una
//! constante: la barra mide distinto con notch, sin él y en una pantalla externa, y el área útil es lo
//! que macOS dice que queda libre (`visibleFrame`, que Tauri da como `work_area`).
//!
//! **Abajo** sigue exactamente como en el H1: el borde inferior del marco entero del monitor. Es la
//! regresión que el test de abajo vigila con la fórmula de entonces.

use serde::{Deserialize, Serialize};

use crate::acople::Marco;

/// Dónde vive la banda. Viaja en las preferencias (`posicionDeLaBanda`, ADR 002 enmienda 8) y a la
/// interfaz, que la dibuja con `data-borde`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Borde {
    /// Bajo la barra de menús, junto a la cámara. La de fábrica.
    #[default]
    Arriba,
    /// Pegada al borde inferior, como en el H1.
    Abajo,
}

impl Borde {
    /// El otro borde: lo que hace `⌃⌥B`.
    pub fn otro(self) -> Self {
        match self {
            Borde::Arriba => Borde::Abajo,
            Borde::Abajo => Borde::Arriba,
        }
    }
}

/// Un monitor, en puntos y con origen arriba-izquierda (el sistema de la Accessibility API).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Monitor {
    /// El marco entero.
    pub marco: Marco,
    /// Lo que macOS deja libre: sin la barra de menús, el notch ni el Dock.
    pub util: Marco,
}

/// La franja de la banda —y de su relleno, que es la misma— para un borde y un alto.
///
/// Ocupa el ancho entero del monitor en los dos bordes. Arriba, el Dock nunca llega (vive abajo o en
/// un lado, centrado), así que el área útil solo aporta su borde superior.
pub fn franja(borde: Borde, monitor: &Monitor, alto: u32) -> Marco {
    let alto = f64::from(alto);
    let m = monitor.marco;
    match borde {
        Borde::Arriba => Marco::nuevo(m.x, monitor.util.y, m.ancho, alto),
        Borde::Abajo => Marco::nuevo(m.x, m.y + m.alto - alto, m.ancho, alto),
    }
}

/// Lo que la banda deja libre debajo de sí, como poco, cuando la bajas (sprint 011): el alto de la banda de voz.
/// Una banda pegada al borde inferior del área útil taparía la barra de tareas y la mitad de abajo de cualquier ventana.
pub const HUECO_BAJO_LA_BANDA: f64 = 44.0;

/// **Cuánto se puede bajar la banda arriba** en este monitor y con este alto (sprint 011, ADR 004 enmienda 3): que
/// quede entera dentro del área útil y con [`HUECO_BAJO_LA_BANDA`] libre debajo. Nunca negativo: en una pantalla donde
/// ni eso cabe, cero, la banda pegada arriba como siempre.
pub fn desplazamiento_maximo(monitor: &Monitor, alto: u32) -> u32 {
    let libre = monitor.util.alto - f64::from(alto) - HUECO_BAJO_LA_BANDA;
    libre.max(0.0).floor() as u32
}

/// El desplazamiento que de verdad se aplica: el pedido, acotado a lo que cabe con ese alto. Se acota **al calcular**,
/// no solo al guardar: agrandar con el asa una banda que bajaste la sacaría del área útil, y así no sale.
pub fn desplazamiento_acotado(monitor: &Monitor, alto: u32, desplazamiento: u32) -> u32 {
    desplazamiento.min(desplazamiento_maximo(monitor, alto))
}

/// **La franja a la altura que elegiste** (sprint 011): la de [`franja`], bajada `desplazamiento` puntos desde el borde
/// superior del área útil. **Solo arriba**: abajo la banda vive pegada al borde inferior y el desplazamiento no se mira.
/// Con 0 es exactamente la de [`franja`] —los tests del sprint 004 la siguen llamando a ella, sin tocarlos—, así que
/// para quien no baja la banda, en el Mac y en Windows, no cambia nada.
pub fn franja_desplazada(borde: Borde, monitor: &Monitor, alto: u32, desplazamiento: u32) -> Marco {
    let f = franja(borde, monitor, alto);
    match borde {
        Borde::Arriba => {
            let d = f64::from(desplazamiento_acotado(monitor, alto, desplazamiento));
            Marco::nuevo(f.x, f.y + d, f.ancho, f.alto)
        }
        Borde::Abajo => f,
    }
}

/// Cuánto mide la barra de menús (con el notch, si lo hay): la distancia del borde superior del
/// monitor al del área útil. Es lo que el relleno sube el fondo de escritorio cuando la banda va arriba,
/// para que por la franja asome justo el trozo que habría debajo.
pub fn barra(monitor: &Monitor) -> f64 {
    (monitor.util.y - monitor.marco.y).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un MacBook Air de 13" sin notch: barra de 25 pt y el Dock abajo (70 pt).
    fn air() -> Monitor {
        Monitor {
            marco: Marco::nuevo(0.0, 0.0, 1440.0, 900.0),
            util: Marco::nuevo(0.0, 25.0, 1440.0, 805.0),
        }
    }

    /// Un MacBook Pro de 14" con notch: la barra mide 38 pt.
    fn con_notch() -> Monitor {
        Monitor {
            marco: Marco::nuevo(0.0, 0.0, 1512.0, 982.0),
            util: Marco::nuevo(0.0, 38.0, 1512.0, 874.0),
        }
    }

    /// Una pantalla externa como principal, desplazada respecto al origen global y con el Dock a la
    /// izquierda: el área útil empieza más a la derecha, y la franja **no** debe seguirla.
    fn externa() -> Monitor {
        Monitor {
            marco: Marco::nuevo(1440.0, -200.0, 1920.0, 1080.0),
            util: Marco::nuevo(1504.0, -175.0, 1856.0, 1055.0),
        }
    }

    #[test]
    fn arriba_nace_bajo_la_barra_de_menus_y_el_notch_nunca_con_una_constante() {
        for (monitor, y) in [(air(), 25.0), (con_notch(), 38.0), (externa(), -175.0)] {
            let f = franja(Borde::Arriba, &monitor, 88);
            assert_eq!(f.y, y, "arriba no cae bajo la barra de {monitor:?}");
            assert_eq!((f.x, f.ancho, f.alto), (monitor.marco.x, monitor.marco.ancho, 88.0));
        }
    }

    /// **La regresión del H1:** abajo es la fórmula de entonces —el borde inferior del marco entero,
    /// sin mirar el Dock—, en las tres pantallas y en los tres altos.
    #[test]
    fn abajo_es_exactamente_la_franja_del_h1() {
        for monitor in [air(), con_notch(), externa()] {
            for alto in [44, 88, 200] {
                let f = franja(Borde::Abajo, &monitor, alto);
                let m = monitor.marco;
                let h1 = Marco::nuevo(m.x, m.y + m.alto - f64::from(alto), m.ancho, f64::from(alto));
                assert_eq!(f, h1);
            }
        }
    }

    /// El asa: arriba se queda clavado el borde superior y crece hacia abajo; abajo, al revés. Si el
    /// borde fijo se moviera, la banda «saltaría» al arrastrarla.
    #[test]
    fn el_asa_deja_quieto_el_borde_que_toca_la_pantalla() {
        let m = con_notch();
        let arriba: Vec<f64> = [44, 88, 200].iter().map(|a| franja(Borde::Arriba, &m, *a).y).collect();
        assert!(arriba.iter().all(|y| *y == 38.0), "arriba se movió el borde de arriba: {arriba:?}");
        let fondos: Vec<f64> = [44, 88, 200]
            .iter()
            .map(|a| {
                let f = franja(Borde::Abajo, &m, *a);
                f.y + f.alto
            })
            .collect();
        assert!(fondos.iter().all(|f| *f == 982.0), "abajo se movió el borde de abajo: {fondos:?}");
    }

    #[test]
    fn la_barra_es_lo_que_el_relleno_sube_el_fondo() {
        assert_eq!(barra(&air()), 25.0);
        assert_eq!(barra(&con_notch()), 38.0);
        assert_eq!(barra(&externa()), 25.0);
        // Un monitor sin barra (menús ocultos): cero, nunca negativo.
        let sin = Monitor { marco: Marco::nuevo(0.0, 0.0, 800.0, 600.0), util: Marco::nuevo(0.0, 0.0, 800.0, 600.0) };
        assert_eq!(barra(&sin), 0.0);
    }

    #[test]
    fn el_borde_de_fabrica_es_arriba_y_b_lo_alterna() {
        assert_eq!(Borde::default(), Borde::Arriba);
        assert_eq!(Borde::Arriba.otro(), Borde::Abajo);
        assert_eq!(Borde::Abajo.otro().otro(), Borde::Abajo);
        assert_eq!(serde_json::to_string(&Borde::Arriba).unwrap(), r#""arriba""#);
        assert_eq!(serde_json::from_str::<Borde>(r#""abajo""#).unwrap(), Borde::Abajo);
    }

    /// Una Surface con Windows a escala 200 %: sin barra de menús (el área útil empieza en 0) y la barra de tareas abajo,
    /// 48 pt. Es la pantalla donde la banda tapaba las pestañas de Chrome (sprint 011).
    fn surface() -> Monitor {
        Monitor {
            marco: Marco::nuevo(0.0, 0.0, 1440.0, 960.0),
            util: Marco::nuevo(0.0, 0.0, 1440.0, 912.0),
        }
    }

    /// **Con desplazamiento 0, la franja de hoy, byte a byte** (sprint 011): en los dos bordes, las cuatro pantallas y
    /// los tres altos. ¿Puede fallar? Sí: sumar el desplazamiento antes de acotarlo, o acotarlo con un mínimo distinto
    /// de 0, movería la banda de quien nunca la tocó.
    #[test]
    fn con_desplazamiento_0_es_la_franja_de_hoy() {
        for monitor in [air(), con_notch(), externa(), surface()] {
            for borde in [Borde::Arriba, Borde::Abajo] {
                for alto in [44, 88, 200] {
                    assert_eq!(franja_desplazada(borde, &monitor, alto, 0), franja(borde, &monitor, alto), "{borde:?} {alto}");
                }
            }
        }
    }

    /// Arriba, la banda baja lo pedido desde el borde superior del área útil, y nada más cambia: ni el ancho, ni la x,
    /// ni el alto.
    #[test]
    fn arriba_desplazada_baja_lo_pedido_en_las_cuatro_pantallas() {
        for (monitor, util_y) in [(air(), 25.0), (con_notch(), 38.0), (externa(), -175.0), (surface(), 0.0)] {
            let f = franja_desplazada(Borde::Arriba, &monitor, 88, 120);
            assert_eq!(f.y, util_y + 120.0, "no bajó lo pedido en {monitor:?}");
            assert_eq!((f.x, f.ancho, f.alto), (monitor.marco.x, monitor.marco.ancho, 88.0));
        }
    }

    /// **Abajo no se desplaza**: la franja es la del H1 aunque las preferencias traigan un desplazamiento.
    #[test]
    fn abajo_el_desplazamiento_no_se_mira() {
        for monitor in [air(), con_notch(), externa(), surface()] {
            for d in [1, 120, 10_000] {
                assert_eq!(franja_desplazada(Borde::Abajo, &monitor, 88, d), franja(Borde::Abajo, &monitor, 88));
            }
        }
    }

    /// **Acotada:** por mucho que se pida, la banda queda entera en el área útil y deja libres 44 pt debajo; y en una
    /// pantalla donde ni eso cabe, se queda pegada arriba. ¿Puede fallar? Sí: sin la cota, un arrastre largo dejaba la
    /// banda bajo la barra de tareas.
    #[test]
    fn el_desplazamiento_se_acota_y_deja_hueco_debajo() {
        for monitor in [air(), con_notch(), externa(), surface()] {
            for alto in [44, 88, 200] {
                let f = franja_desplazada(Borde::Arriba, &monitor, alto, 10_000);
                let fondo_util = monitor.util.y + monitor.util.alto;
                assert!(f.fondo() <= fondo_util - HUECO_BAJO_LA_BANDA, "{monitor:?} {alto}: se salió del área útil");
                assert!(fondo_util - HUECO_BAJO_LA_BANDA - f.fondo() < 1.0, "{monitor:?} {alto}: se quedó corta");
            }
        }
        assert_eq!(desplazamiento_maximo(&surface(), 88), 912 - 88 - 44);
        let diminuta = Monitor { marco: Marco::nuevo(0.0, 0.0, 800.0, 120.0), util: Marco::nuevo(0.0, 0.0, 800.0, 120.0) };
        assert_eq!(desplazamiento_maximo(&diminuta, 88), 0);
        assert_eq!(franja_desplazada(Borde::Arriba, &diminuta, 88, 50), franja(Borde::Arriba, &diminuta, 88));
    }

    /// Agrandar con el asa una banda bajada no la saca del área útil: el mismo desplazamiento se acota con el alto nuevo.
    #[test]
    fn agrandar_una_banda_bajada_la_vuelve_a_acotar() {
        let m = surface();
        let pegada_abajo = desplazamiento_maximo(&m, 88);
        let ampliada = franja_desplazada(Borde::Arriba, &m, 200, pegada_abajo);
        assert!(ampliada.fondo() <= m.util.alto - HUECO_BAJO_LA_BANDA, "la ampliada se salió: {ampliada:?}");
        assert_eq!(desplazamiento_acotado(&m, 200, pegada_abajo), desplazamiento_maximo(&m, 200));
    }
}
