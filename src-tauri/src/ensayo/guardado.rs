//! LO QUE QUEDA DE UN ENSAYO — el contenido de su archivo (ADR 015, enmienda 4). **MÓDULO PROTEGIDO**:
//! puro, sin disco ni red. Lo arma esta carpeta y lo escribe `ensayos.rs`, que no ve un solo marco de audio:
//! el mismo reparto que `notas/` y `carpeta.rs`.
//!
//! Queda **lo tuyo**: cada pregunta a la que llegaste, tu respuesta en texto (la pista del micrófono, la
//! única que el ensayo abre) y sus cifras. **El audio no llega hasta aquí**: el oído lo pisa al
//! transcribir, y la sesión solo tiene texto. Las preguntas a las que no llegaste no se guardan.
//!
//! Aquí viven también las dos lecturas de lo guardado: **el progreso** (las mismas cuatro cifras, ensayo a
//! ensayo, y cómo cambiaron desde el primero; sin puntajes) y **el texto que se exporta**.

use serde::{Deserialize, Serialize};

use super::banco::{De, Idioma, Pregunta};
use super::sesion::{informe_de, Informe, Suerte};

/// La versión del contenido de dentro del `.ghost`.
pub const VERSION: u32 = 1;
/// Cuántos ensayos enseña la tabla del progreso: los más recientes. Los de antes se cuentan debajo.
pub const FILAS_DEL_PROGRESO: usize = 6;

/// Una pregunta a la que llegaste, y qué pasó con ella.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Llegada {
    pub pregunta: Pregunta,
    pub suerte: Suerte,
}

/// **Un ensayo guardado**, antes de cifrar.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Guardado {
    pub version: u32,
    pub cliente: String,
    /// El nombre de tu propuesta; vacío si las preguntas salieron solo de la ficha del cliente.
    pub propuesta: String,
    pub idioma: Idioma,
    /// «2026-10-04 09:12»: la hora del Mac al empezar.
    pub empezo: String,
    pub llegadas: Vec<Llegada>,
}

impl Guardado {
    pub fn nuevo(cliente: &str, propuesta: &str, idioma: Idioma, empezo: &str, llegadas: Vec<(Pregunta, Suerte)>) -> Self {
        Self {
            version: VERSION,
            cliente: cliente.to_string(),
            propuesta: propuesta.to_string(),
            idioma,
            empezo: empezo.to_string(),
            llegadas: llegadas.into_iter().map(|(pregunta, suerte)| Llegada { pregunta, suerte }).collect(),
        }
    }

    /// El mismo informe que la pantalla enseñó al terminar.
    pub fn informe(&self) -> Informe {
        informe_de(self.llegadas.iter().enumerate().map(|(i, l)| (i + 1, l.pregunta.texto.as_str(), &l.suerte)))
    }

    pub fn a_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("un ensayo siempre se serializa")
    }

    pub fn de_bytes(bytes: &[u8]) -> Result<Guardado, String> {
        let g: Guardado = serde_json::from_slice(bytes).map_err(|e| format!("el ensayo no se lee: {e}"))?;
        if g.version != VERSION {
            return Err(format!("un ensayo de una versión que esta app no conoce ({})", g.version));
        }
        Ok(g)
    }

    /// **Lo que se exporta**: Markdown, en el idioma de la interfaz. Los rótulos se redactan en los dos
    /// idiomas, no se traducen (regla bilingüe); las cifras son las de la pantalla.
    pub fn a_texto(&self, idioma: &str) -> String {
        let en = idioma.starts_with("en");
        let r = if en { &EN } else { &ES };
        let i = self.informe();
        let mut t = format!("# {} · {} · {}\n\n", r.ensayo, self.cliente, self.empezo);
        if !self.propuesta.is_empty() {
            t.push_str(&format!("{}: {}\n\n", r.propuesta, self.propuesta));
        }
        t.push_str(&format!(
            "{}: {} · {}: {} · {}: {} · {}: {}\n",
            r.evidencia,
            if i.evidencia == 0 { "—".to_string() } else { format!("{} {} {}", i.citadas, r.de, i.evidencia) },
            r.ritmo,
            i.ppm_medio.map_or("—".to_string(), |p| format!("{p} {}", r.ppm)),
            r.muletillas,
            i.muletillas,
            r.tiempo,
            i.tiempo_medio_ms.map_or("—".to_string(), reloj),
        ));
        for (n, l) in self.llegadas.iter().enumerate() {
            t.push_str(&format!("\n## {}. {}\n\n", n + 1, l.pregunta.texto));
            let de = match l.pregunta.de {
                De::Propuesta => r.de_propuesta,
                De::Ficha => r.de_ficha,
                De::Objeciones => r.de_objecion,
                De::Modelo => r.de_modelo,
            };
            match l.pregunta.seccion.as_ref().or(l.pregunta.fuente.as_ref()) {
                Some(s) => t.push_str(&format!("{de} · {s}\n\n")),
                None => t.push_str(&format!("{de}\n\n")),
            }
            let Suerte::Respondida { respuesta, evaluacion: e } = &l.suerte else {
                t.push_str(&format!("{}\n", r.saltada));
                continue;
            };
            let respuesta = respuesta.trim();
            t.push_str(&format!("{}: {}\n\n", r.tu_respuesta, if respuesta.is_empty() { "—" } else { respuesta }));
            let ficha = |ev: &super::evaluacion::Evidencia| match &ev.fuente.seccion {
                Some(s) => format!("- {} — {} · {s}\n", ev.titular, ev.fuente.documento),
                None => format!("- {} — {}\n", ev.titular, ev.fuente.documento),
            };
            let (usadas, sin_usar): (Vec<_>, Vec<_>) = e.evidencia.iter().partition(|ev| ev.citada || ev.dicha_por_ti);
            if !usadas.is_empty() {
                t.push_str(&format!("{}:\n", r.usaste));
                usadas.iter().for_each(|ev| t.push_str(&ficha(ev)));
                t.push('\n');
            }
            if !sin_usar.is_empty() {
                t.push_str(&format!("{}:\n", r.tenias));
                sin_usar.iter().for_each(|ev| t.push_str(&ficha(ev)));
                t.push('\n');
            }
            let muletillas = if e.muletillas.is_empty() {
                "0".to_string()
            } else {
                e.muletillas
                    .iter()
                    .map(|m| if en { format!("“{}” ×{}", m.frase, m.veces) } else { format!("«{}» ×{}", m.frase, m.veces) })
                    .collect::<Vec<_>>()
                    .join(" · ")
            };
            t.push_str(&format!(
                "{} {} · {} {} · {}: {}\n",
                r.tiempo,
                reloj(e.tiempo_ms),
                r.ritmo,
                e.ppm.map_or("—".to_string(), |p| format!("{p} {}", r.ppm)),
                r.muletillas,
                muletillas,
            ));
        }
        t
    }
}

impl Drop for Guardado {
    /// Tus respuestas se pisan antes de soltarlas, como el resto de la casa.
    fn drop(&mut self) {
        for l in &mut self.llegadas {
            if let Suerte::Respondida { respuesta, .. } = &mut l.suerte {
                // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
                unsafe { respuesta.as_mut_vec() }.fill(0);
                respuesta.clear();
            }
        }
    }
}

/// «1:12», como la pantalla.
fn reloj(ms: u64) -> String {
    let s = ms / 1000;
    format!("{}:{:02}", s / 60, s % 60)
}

/// Los rótulos del texto exportado, redactados en cada idioma.
struct Rotulos {
    ensayo: &'static str,
    propuesta: &'static str,
    evidencia: &'static str,
    de: &'static str,
    ritmo: &'static str,
    ppm: &'static str,
    muletillas: &'static str,
    tiempo: &'static str,
    de_propuesta: &'static str,
    de_ficha: &'static str,
    de_objecion: &'static str,
    de_modelo: &'static str,
    saltada: &'static str,
    tu_respuesta: &'static str,
    usaste: &'static str,
    tenias: &'static str,
}

const ES: Rotulos = Rotulos {
    ensayo: "Ensayo",
    propuesta: "Propuesta",
    evidencia: "Evidencia usada",
    de: "de",
    ritmo: "Ritmo",
    ppm: "ppm",
    muletillas: "Muletillas",
    tiempo: "Tiempo",
    de_propuesta: "De tu propuesta",
    de_ficha: "De su ficha de cliente",
    de_objecion: "Objeción típica",
    de_modelo: "Sugerida por el modelo",
    saltada: "Saltada.",
    tu_respuesta: "Tu respuesta",
    usaste: "Usaste",
    tenias: "Tenías y no usaste",
};

const EN: Rotulos = Rotulos {
    ensayo: "Rehearsal",
    propuesta: "Proposal",
    evidencia: "Evidence used",
    de: "of",
    ritmo: "Pace",
    ppm: "wpm",
    muletillas: "Fillers",
    tiempo: "Time",
    de_propuesta: "From your proposal",
    de_ficha: "From their client card",
    de_objecion: "Typical objection",
    de_modelo: "Suggested by the model",
    saltada: "Skipped.",
    tu_respuesta: "Your answer",
    usaste: "You used",
    tenias: "You had it and did not use it",
};

// ─── el progreso ─────────────────────────────────────────────────────────────────────────────────

/// Un ensayo en la tabla del progreso: las cuatro cifras del informe, y cuándo.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilaDelProgreso {
    pub empezo: String,
    pub citadas: usize,
    pub evidencia: usize,
    pub ppm_medio: Option<u32>,
    /// `None` si no respondiste ninguna: cero muletillas en nada no es una cifra.
    pub muletillas: Option<u32>,
    pub tiempo_medio_ms: Option<u64>,
}

/// De cuánto a cuánto. La pantalla pone la flecha.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cambio {
    pub desde: u64,
    pub hasta: u64,
}

/// «Desde el primero»: cada cifra, del primer ensayo que la tiene al último que la tiene. `None` si no hay
/// dos ensayos con ella.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesdeElPrimero {
    pub evidencia: Option<Cambio>,
    pub ritmo: Option<Cambio>,
    pub muletillas: Option<Cambio>,
    pub tiempo: Option<Cambio>,
}

/// **Tu progreso con un cliente**: solo cifras que mediste, ensayo a ensayo. Cruza al webview; tus
/// respuestas, no.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progreso {
    pub cliente: String,
    /// Los [`FILAS_DEL_PROGRESO`] más recientes, del más viejo al más nuevo.
    pub filas: Vec<FilaDelProgreso>,
    /// Los que no caben en la tabla, más viejos.
    pub antes: usize,
    pub desde_el_primero: DesdeElPrimero,
}

impl FilaDelProgreso {
    fn de(g: &Guardado) -> Self {
        let i = g.informe();
        Self {
            empezo: g.empezo.clone(),
            citadas: i.citadas,
            evidencia: i.evidencia,
            ppm_medio: i.ppm_medio,
            muletillas: (i.respondidas > 0).then_some(i.muletillas),
            tiempo_medio_ms: i.tiempo_medio_ms,
        }
    }
}

/// Del primer ensayo que tiene la cifra al último que la tiene, si son dos distintos.
fn cambio(filas: &[FilaDelProgreso], cifra: impl Fn(&FilaDelProgreso) -> Option<u64>) -> Option<Cambio> {
    let primero = filas.iter().position(|f| cifra(f).is_some())?;
    let ultimo = filas.iter().rposition(|f| cifra(f).is_some())?;
    (primero < ultimo).then(|| Cambio { desde: cifra(&filas[primero]).unwrap_or(0), hasta: cifra(&filas[ultimo]).unwrap_or(0) })
}

/// **El progreso** con `cliente`, de sus ensayos guardados en cualquier orden.
pub fn progreso(cliente: &str, guardados: &[Guardado]) -> Progreso {
    let mut todas: Vec<FilaDelProgreso> = guardados.iter().map(FilaDelProgreso::de).collect();
    todas.sort_by(|a, b| a.empezo.cmp(&b.empezo));
    let desde_el_primero = DesdeElPrimero {
        // Con cero fichas no hay evidencia que contar: «0 de 0» no es un punto de partida.
        evidencia: cambio(&todas, |f| (f.evidencia > 0).then_some(f.citadas as u64)),
        ritmo: cambio(&todas, |f| f.ppm_medio.map(u64::from)),
        muletillas: cambio(&todas, |f| f.muletillas.map(u64::from)),
        tiempo: cambio(&todas, |f| f.tiempo_medio_ms),
    };
    let antes = todas.len().saturating_sub(FILAS_DEL_PROGRESO);
    Progreso { cliente: cliente.to_string(), filas: todas.split_off(antes), antes, desde_el_primero }
}

#[cfg(test)]
pub(crate) mod pruebas {
    use super::*;
    use crate::corpus::unidad::Unidad;
    use crate::ensayo::banco::Regla;
    use crate::ensayo::evaluacion::{Evaluacion, Evidencia, Muletilla};
    use crate::ficha::Fuente;

    pub fn pregunta(texto: &str) -> Pregunta {
        Pregunta {
            texto: texto.into(),
            regla: Regla::Seccion,
            de: De::Propuesta,
            seccion: Some("Supuestos".into()),
            clave: "supuestos".into(),
            fuente: None,
        }
    }

    pub fn respondida(respuesta: &str, citadas: usize, de: usize, tiempo_ms: u64, ppm: Option<u32>, muletillas: u32) -> Suerte {
        let fuente = Fuente { documento: "Propuesta Páramo Azul".into(), seccion: Some("Plazo".into()), unidad: Some(Unidad::Propuesta), conjeturada: false };
        Suerte::Respondida {
            respuesta: respuesta.into(),
            evaluacion: Evaluacion {
                evidencia: (0..de)
                    .map(|i| Evidencia { titular: format!("Ficha {i}"), fuente: fuente.clone(), citada: i < citadas, dicha_por_ti: false })
                    .collect(),
                tiempo_ms,
                ppm,
                muletillas: if muletillas == 0 { Vec::new() } else { vec![Muletilla { frase: "o sea".into(), veces: muletillas }] },
                // No se guarda (no es una cifra de la pantalla): un ensayo leído de vuelta la trae a cero.
                palabras: 0,
            },
        }
    }

    pub fn guardado(empezo: &str, llegadas: Vec<(Pregunta, Suerte)>) -> Guardado {
        Guardado::nuevo("Páramo Azul", "Rentabilidad por canal", Idioma::Es, empezo, llegadas)
    }

    /// **Lo guardado se lee igual que se escribió** —tu respuesta, sus fichas y sus cifras— y **dice lo mismo
    /// que la pantalla al terminar**: el informe sale de la misma cuenta (`informe_de`). Demostrado en rojo:
    /// con `tiempo_medio_ms` calculado sobre todas las filas (las saltadas como cero) en vez de las
    /// respondidas, el informe de lo guardado no cuadraba con el de la sesión.
    #[test]
    fn lo_guardado_vuelve_entero_y_su_informe_es_el_de_la_pantalla() {
        let g = guardado(
            "2026-10-04 09:12",
            vec![
                (pregunta("¿De dónde sale el 12 %?"), respondida("Sale del cierre de septiembre, o sea del ERP.", 2, 3, 44_000, Some(140), 1)),
                (pregunta("¿Quién lo va a usar?"), Suerte::Saltada),
                (pregunta("¿Y el plazo?"), respondida("Corre desde la entrega de datos.", 3, 3, 72_000, Some(142), 0)),
            ],
        );
        let vuelta = Guardado::de_bytes(&g.a_bytes()).unwrap();
        assert_eq!(vuelta, g, "lo guardado no volvió igual");
        let i = vuelta.informe();
        assert_eq!((i.respondidas, i.saltadas, i.citadas, i.evidencia), (2, 1, 5, 6));
        assert_eq!((i.ppm_medio, i.muletillas, i.tiempo_medio_ms), (Some(141), 1, Some(58_000)));
        assert_eq!(i.filas.iter().map(|f| f.numero).collect::<Vec<_>>(), [1, 2, 3]);
    }

    /// **En el archivo no hay audio ni lo que no llegaste a responder**: el contenido es texto con estos
    /// campos y no otros. Si un día alguien le añade un campo con muestras, este test lo nombra.
    #[test]
    fn lo_guardado_es_texto_y_solo_lo_que_llegaste() {
        let g = guardado("2026-10-04 09:12", vec![(pregunta("¿Uno?"), respondida("Mi respuesta.", 1, 3, 30_000, Some(120), 0))]);
        let json: serde_json::Value = serde_json::from_slice(&g.a_bytes()).unwrap();
        let mut claves: Vec<&str> = json.as_object().unwrap().keys().map(String::as_str).collect();
        claves.sort();
        assert_eq!(claves, ["cliente", "empezo", "idioma", "llegadas", "propuesta", "version"]);
        let respondida = &json["llegadas"][0]["suerte"];
        let mut claves: Vec<&str> = respondida.as_object().unwrap().keys().map(String::as_str).collect();
        claves.sort();
        assert_eq!(claves, ["evaluacion", "que", "respuesta"], "la respuesta guardada lleva algo más que texto y cifras");
        let mut cifras: Vec<&str> = respondida["evaluacion"].as_object().unwrap().keys().map(String::as_str).collect();
        cifras.sort();
        assert_eq!(cifras, ["evidencia", "muletillas", "ppm", "tiempoMs"]);
    }

    #[test]
    fn una_version_que_no_se_conoce_no_se_abre() {
        let mut json: serde_json::Value = serde_json::from_slice(&guardado("2026-10-04 09:12", Vec::new()).a_bytes()).unwrap();
        json["version"] = 2.into();
        assert!(Guardado::de_bytes(json.to_string().as_bytes()).is_err());
        assert!(Guardado::de_bytes(b"{}").is_err());
    }

    /// **El progreso compara el primero con el último** que tienen cada cifra, ordenados por fecha aunque
    /// lleguen desordenados, y enseña los seis más recientes. Demostrado en rojo: sin el `sort_by`, la lista
    /// en el orden del disco daba «desde el primero» al revés.
    #[test]
    fn el_progreso_va_del_primero_al_ultimo_y_cuenta_los_que_no_caben() {
        let ensayo = |dia: u32, citadas: usize, ppm: u32, muletillas: u32, ms: u64| {
            guardado(&format!("2026-09-{dia:02} 10:00"), vec![(pregunta("¿Una?"), respondida("…", citadas, 3, ms, Some(ppm), muletillas))])
        };
        let desordenados = vec![ensayo(27, 2, 150, 4, 69_000), ensayo(21, 1, 161, 7, 81_000), ensayo(30, 3, 138, 2, 58_000)];
        let p = progreso("Páramo Azul", &desordenados);
        assert_eq!(p.filas.iter().map(|f| f.empezo.as_str()).collect::<Vec<_>>(), ["2026-09-21 10:00", "2026-09-27 10:00", "2026-09-30 10:00"]);
        assert_eq!(p.desde_el_primero.evidencia, Some(Cambio { desde: 1, hasta: 3 }));
        assert_eq!(p.desde_el_primero.ritmo, Some(Cambio { desde: 161, hasta: 138 }));
        assert_eq!(p.desde_el_primero.muletillas, Some(Cambio { desde: 7, hasta: 2 }));
        assert_eq!(p.desde_el_primero.tiempo, Some(Cambio { desde: 81_000, hasta: 58_000 }));
        assert_eq!(p.antes, 0);

        let muchos: Vec<Guardado> = (1..=8).map(|d| ensayo(d, 1, 140, 3, 60_000)).collect();
        let p = progreso("Páramo Azul", &muchos);
        assert_eq!((p.filas.len(), p.antes), (FILAS_DEL_PROGRESO, 2));
        assert_eq!(p.filas[0].empezo, "2026-09-03 10:00", "la tabla no enseña los más recientes");
    }

    /// **Sin dos ensayos con la cifra, no hay flecha.** Uno solo, o el primero sin ritmo (no transcribió),
    /// o uno sin fichas: la pantalla no inventa un punto de partida.
    #[test]
    fn sin_dos_ensayos_con_la_cifra_no_hay_cambio() {
        let solo = vec![guardado("2026-09-21 10:00", vec![(pregunta("¿Una?"), respondida("…", 1, 3, 60_000, Some(150), 2))])];
        assert_eq!(progreso("Páramo Azul", &solo).desde_el_primero, DesdeElPrimero::default());

        let sin_ritmo_ni_fichas = guardado("2026-09-21 10:00", vec![(pregunta("¿Una?"), respondida("", 0, 0, 30_000, None, 0))]);
        let despues = guardado("2026-09-27 10:00", vec![(pregunta("¿Una?"), respondida("…", 2, 3, 50_000, Some(140), 1))]);
        let tercero = guardado("2026-09-30 10:00", vec![(pregunta("¿Una?"), respondida("…", 3, 3, 40_000, Some(130), 0))]);
        let p = progreso("Páramo Azul", &[sin_ritmo_ni_fichas, despues, tercero]);
        assert_eq!(p.desde_el_primero.evidencia, Some(Cambio { desde: 2, hasta: 3 }), "contó «0 de 0» como punto de partida");
        assert_eq!(p.desde_el_primero.ritmo, Some(Cambio { desde: 140, hasta: 130 }));
        assert_eq!(p.desde_el_primero.tiempo, Some(Cambio { desde: 30_000, hasta: 40_000 }));

        let todo_saltado = guardado("2026-10-01 10:00", vec![(pregunta("¿Una?"), Suerte::Saltada)]);
        assert_eq!(FilaDelProgreso::de(&todo_saltado).muletillas, None, "cero muletillas en nada se contó como cifra");
    }

    /// **Lo exportado se lee en el idioma de la interfaz**, con tu respuesta, lo que usaste, lo que tenías y
    /// sus cifras; la saltada lo dice. Redactado en los dos idiomas.
    #[test]
    fn el_texto_exportado_es_bilingue_y_lleva_tus_respuestas() {
        let g = guardado(
            "2026-10-04 09:12",
            vec![
                (pregunta("¿De dónde sale el 12 %?"), respondida("Sale del cierre, o sea del ERP.", 1, 2, 44_000, Some(140), 1)),
                (pregunta("¿Quién lo va a usar?"), Suerte::Saltada),
            ],
        );
        let es = g.a_texto("es");
        assert!(es.starts_with("# Ensayo · Páramo Azul · 2026-10-04 09:12"));
        for frase in ["Evidencia usada: 1 de 2", "Tu respuesta: Sale del cierre, o sea del ERP.", "Usaste:\n- Ficha 0", "Tenías y no usaste:\n- Ficha 1", "«o sea» ×1", "Tiempo 0:44", "Saltada."] {
            assert!(es.contains(frase), "falta «{frase}» en:\n{es}");
        }
        let en = g.a_texto("en");
        for frase in ["# Rehearsal · Páramo Azul", "Evidence used: 1 of 2", "Your answer:", "You had it and did not use it:", "“o sea” ×1", "140 wpm", "Skipped."] {
            assert!(en.contains(frase), "falta «{frase}» en:\n{en}");
        }
    }
}
