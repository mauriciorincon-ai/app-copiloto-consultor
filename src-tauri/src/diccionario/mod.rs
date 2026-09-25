//! EL DICCIONARIO TÉCNICO — lo que el transcriptor oye mal y el consultor dice todo el día.
//!
//! Un motor de voz generalista no conoce la jerga de nadie. «Power BI» sale «power by», «DAX» sale
//! «the ax», «Lakehouse» sale «lake house», y el cliente de esta reunión —que se llama como se
//! llama— sale como cualquier cosa. Cada una de esas es una ficha que no llega: el disparador no
//! reconoce el término, el índice no lo encuentra, y la banda se queda en blanco por un error de
//! una letra.
//!
//! Esto lo arregla **con código y sin modelo**: una corrección posterior, determinista, sobre el
//! texto ya transcrito. Es la regla 14 de la casa en su forma más literal — antes de preguntarle a
//! una IA si eso que oyó era «Power BI», se mira en una lista.
//!
//! ### MÓDULO PROTEGIDO, y eso es una decisión de diseño, no una etiqueta
//!
//! Este módulo tiene el transcript del cliente en las manos: recibe cada turno y devuelve el turno
//! corregido. Así que **no puede tocar disco**, y `pnpm verify:ephemeral` lo comprueba.
//!
//! Pero el diccionario **sí persiste** —es del consultor, como sus notas— y eso parecía obligar a
//! lo contrario. No: lo que persiste se serializa aquí **a un `String`** y lo escribe la capa de
//! arriba (`lib.rs`), que no ve un solo turno. El plan del sprint decía «`diccionario/` puede tocar
//! disco»; esto es más fuerte y cuesta lo mismo: el módulo que toca la voz del cliente no tiene
//! manera de escribirla, y no hace falta confiar en que nadie se equivoque.
//!
//! ### La regla que lo hace inocuo: el diccionario NO APRENDE DE LA REUNIÓN
//!
//! Un diccionario que se corrigiera a sí mismo con lo que oye sería un transcript persistido con
//! otro nombre. Las entradas salen de **dos sitios y de ninguno más**: la semilla y el archivo del
//! usuario —que él escribe— y los **nombres del corpus**, que son sus propios documentos. `corregir`
//! no muta nada; tiene `&self` y ese `&` es la regla escrita en el tipo.

use std::collections::HashSet;

/// Cuánto se puede equivocar el transcriptor y aún así entenderse, según lo largo que sea el
/// término.
///
/// **Con cuatro letras o menos, cero.** Es la decisión que evita que este módulo haga más daño que
/// bien: «DAX» está a una edición de «das», «dos», «día», «tax», «max» y de media docena más de
/// palabras que la gente dice de verdad. Para los términos cortos la única corrección que se
/// acepta es una **variante escrita a mano** — «the ax» → «DAX» porque alguien lo puso ahí, no
/// porque se parezca.
fn tolerancia(largo: usize) -> usize {
    match largo {
        0..=4 => 0,
        5..=8 => 1,
        _ => 2,
    }
}

/// Cuántas palabras seguidas puede ocupar un término. «Semantic Model» son dos y «Azure Data
/// Factory» tres; más allá de tres, la ventana empieza a tragarse frases enteras.
const VENTANA: usize = 3;

/// Un término, tal como el consultor quiere verlo escrito, y las formas en que se oye mal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Termino {
    /// Lo que se escribe. Se respeta tal cual: mayúsculas incluidas.
    pub canonico: String,
    /// Lo que el transcriptor devuelve cuando falla. Normalizadas al leerse.
    pub variantes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Diccionario {
    terminos: Vec<Termino>,
    /// De dónde salió cada término, para que la pantalla pueda decirlo y para que nadie tenga que
    /// adivinar por qué la app corrigió algo.
    del_corpus: HashSet<String>,
}

/// La semilla: la jerga que este consultor usa en cada reunión.
///
/// Sale del brief y no de una lista genérica de tecnología. Cada variante está aquí porque es lo
/// que un motor de voz castellano o inglés devuelve de verdad al oír el término — no son erratas
/// inventadas.
const SEMILLA: &[(&str, &[&str])] = &[
    ("Power BI", &["power bi", "powerbi", "power by", "poder bi", "power be", "power vi"]),
    ("DAX", &["dax", "the ax", "daks", "dacs", "dachs", "de ax", "dax's"]),
    ("Microsoft Fabric", &["microsoft fabric", "fabric", "fabrica", "fabrics", "favric"]),
    ("Semantic Model", &["semantic model", "semantico model", "semantic modelo", "modelo semantico"]),
    ("Lakehouse", &["lakehouse", "lake house", "leikhaus", "lago house", "lake haus"]),
];

impl Diccionario {
    /// El diccionario con el que la app arranca la primera vez.
    pub fn semilla() -> Self {
        Self {
            terminos: SEMILLA
                .iter()
                .map(|(c, vs)| Termino {
                    canonico: (*c).to_string(),
                    variantes: vs.iter().map(|v| normalizar(v)).collect(),
                })
                .collect(),
            del_corpus: HashSet::new(),
        }
    }

    pub fn terminos(&self) -> &[Termino] {
        &self.terminos
    }

    /// Cuántos términos vienen del corpus del usuario y no de su archivo. Lo enseña la pantalla.
    pub fn cuantos_del_corpus(&self) -> usize {
        self.del_corpus.len()
    }

    /// **Los nombres propios del corpus, que son los del cliente de hoy.**
    ///
    /// Vienen de `disparo::vocabulario`, que ya los saca de los nombres de documento y los títulos
    /// de sección y ya deja fuera las palabras que dice todo el mundo. Aquí se añaden **sin
    /// variantes**: no se sabe cómo los va a oír mal el transcriptor, así que lo único que se puede
    /// hacer es la distancia acotada, y para eso basta el canónico.
    ///
    /// Se ignoran los cortos, por lo mismo que la tolerancia es cero por debajo de cinco letras: un
    /// nombre de cuatro letras corregiría media conversación.
    pub fn con_nombres_del_corpus(&mut self, nombres: &[String]) {
        for n in nombres {
            if n.chars().count() < 5 {
                continue;
            }
            let norma = normalizar(n);
            if self.terminos.iter().any(|t| normalizar(&t.canonico) == norma) {
                continue;
            }
            self.del_corpus.insert(n.clone());
            self.terminos.push(Termino { canonico: n.clone(), variantes: Vec::new() });
        }
    }

    /// **Lo que este módulo existe para hacer:** el turno tal como salió del motor, entra; el turno
    /// con la jerga escrita bien, sale.
    ///
    /// Se prueban las ventanas **de más palabras a menos**, porque «Semantic Model» tiene que ganarle
    /// a «Semantic» a secas; y dentro de cada ventana, primero la coincidencia exacta con una
    /// variante y solo después la distancia acotada. Lo que no encaja se copia **tal cual**: este
    /// módulo no normaliza el texto del cliente, solo sustituye lo que reconoce.
    pub fn corregir(&self, texto: &str) -> String {
        let palabras: Vec<&str> = texto.split_inclusive(char::is_whitespace).collect();
        let mut salida = String::with_capacity(texto.len());
        let mut i = 0;
        while i < palabras.len() {
            let mut saltado = false;
            for n in (1..=VENTANA.min(palabras.len() - i)).rev() {
                let trozo: String = palabras[i..i + n].concat();
                let (cabeza, nucleo, cola) = partir(&trozo);
                let Some(canonico) = self.buscar(nucleo) else { continue };
                salida.push_str(cabeza);
                salida.push_str(canonico);
                salida.push_str(cola);
                i += n;
                saltado = true;
                break;
            }
            if !saltado {
                salida.push_str(palabras[i]);
                i += 1;
            }
        }
        salida
    }

    /// ¿A qué término canónico corresponde este trozo, si a alguno?
    fn buscar(&self, trozo: &str) -> Option<&str> {
        let norma = normalizar(trozo);
        if norma.is_empty() {
            return None;
        }
        // Primero lo exacto, y de un tirón por todos los términos: una variante escrita a mano vale
        // más que cualquier parecido, y es lo único que se acepta para los términos cortos.
        for t in &self.terminos {
            if normalizar(&t.canonico) == norma || t.variantes.iter().any(|v| *v == norma) {
                // Ya estaba bien escrito: no se toca, para no cambiar mayúsculas que el usuario
                // pueda haber querido.
                if t.canonico == trozo {
                    return None;
                }
                return Some(&t.canonico);
            }
        }
        // Y solo después el parecido, con el techo que le toca a su largo.
        let mut mejor: Option<(usize, &str)> = None;
        for t in &self.terminos {
            let candidatos = std::iter::once(normalizar(&t.canonico)).chain(t.variantes.iter().cloned());
            for c in candidatos {
                let techo = tolerancia(c.chars().count());
                if techo == 0 {
                    continue;
                }
                // Una diferencia de largo mayor que el techo no puede salvarse: se descarta sin
                // calcular nada. Con ventanas de hasta tres palabras esto se llama mucho.
                if norma.chars().count().abs_diff(c.chars().count()) > techo {
                    continue;
                }
                let d = distancia(&norma, &c, techo);
                if d <= techo && mejor.is_none_or(|(m, _)| d < m) {
                    mejor = Some((d, &t.canonico));
                }
            }
        }
        mejor.map(|(_, c)| c)
    }

    // ── El archivo del usuario. Se serializa a texto AQUÍ y lo escribe `lib.rs`: este módulo
    //    tiene el transcript en las manos y no puede tocar disco. ─────────────────────────────

    /// El archivo tal como el usuario lo va a ver y editar.
    ///
    /// Es **YAML válido** a propósito, aunque se lea con un parser propio: así el editor del usuario
    /// lo colorea y cualquier herramienta futura puede leerlo sin convertir nada. Lo que NO se hace
    /// es traer una librería de YAML entera para una lista de pares — `serde_yaml` además está
    /// archivada por su autor, y una dependencia sin mantenimiento en la ruta de un archivo que el
    /// usuario edita es exactamente donde no conviene tenerla.
    ///
    /// Los términos que vinieron del corpus **no se escriben**: se recalculan del corpus en cada
    /// arranque, y guardarlos convertiría el archivo del usuario en una copia del nombre de su
    /// cliente sin que él lo hubiera escrito.
    pub fn a_texto(&self) -> String {
        let mut s = String::from(CABECERA);
        for t in &self.terminos {
            if self.del_corpus.contains(&t.canonico) {
                continue;
            }
            s.push_str(&t.canonico);
            s.push(':');
            if !t.variantes.is_empty() {
                s.push_str(" [");
                s.push_str(&t.variantes.join(", "));
                s.push(']');
            }
            s.push('\n');
        }
        s
    }

    /// Lee el archivo del usuario. Una línea que no se entiende **se dice con su número**, en vez de
    /// ignorarse: un diccionario que se come la mitad de lo que el usuario escribió sin avisar es
    /// peor que uno que no existe.
    pub fn de_texto(texto: &str) -> Result<Self, String> {
        let mut terminos: Vec<Termino> = Vec::new();
        for (n, linea) in texto.lines().enumerate() {
            let l = linea.trim();
            if l.is_empty() || l.starts_with('#') {
                continue;
            }
            let Some(dos) = l.find(':') else {
                return Err(format!(
                    "línea {}: «{l}» no tiene dos puntos. Cada línea es «Término: [como se oye mal, …]»",
                    n + 1
                ));
            };
            let canonico = l[..dos].trim().to_string();
            if canonico.is_empty() {
                return Err(format!("línea {}: falta el término antes de los dos puntos", n + 1));
            }
            let resto = l[dos + 1..].trim();
            let variantes = if resto.is_empty() {
                Vec::new()
            } else {
                let dentro = resto
                    .strip_prefix('[')
                    .and_then(|r| r.strip_suffix(']'))
                    .ok_or_else(|| {
                        format!("línea {}: las formas mal oídas van entre corchetes: [una, otra]", n + 1)
                    })?;
                dentro
                    .split(',')
                    .map(|v| normalizar(v))
                    .filter(|v| !v.is_empty())
                    .collect()
            };
            if terminos.iter().any(|t| t.canonico == canonico) {
                return Err(format!("línea {}: «{canonico}» ya estaba más arriba", n + 1));
            }
            terminos.push(Termino { canonico, variantes });
        }
        Ok(Self { terminos, del_corpus: HashSet::new() })
    }
}

const CABECERA: &str = "\
# Tu diccionario técnico — Angel Ghost
#
# El motor de voz de macOS no conoce tu jerga: «Power BI» le sale «power by» y «DAX» le sale «the
# ax». Esta lista lo arregla después de transcribir, sin que nada salga de tu equipo.
#
# Cada línea es un término COMO QUIERES VERLO ESCRITO y, entre corchetes, las formas en que se oye
# mal. Puedes editarla a mano: la app la lee al arrancar.
#
#   Power BI: [power bi, powerbi, power by]
#   Lakehouse: [lake house]
#   Un término sin corchetes también vale: se corrige por parecido.
#
# Los nombres de TUS CLIENTES no se escriben aquí: salen de los documentos de tu corpus en cada
# arranque, y por eso este archivo no guarda el nombre de nadie.

";

/// Minúsculas, sin tildes y con los espacios colapsados. Es la forma en la que se comparan las
/// cosas, nunca la que se escribe.
fn normalizar(texto: &str) -> String {
    let mut salida = String::with_capacity(texto.len());
    let mut espacio = false;
    for c in texto.trim().chars().flat_map(|c| c.to_lowercase()) {
        let c = match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            c => c,
        };
        if c.is_whitespace() {
            espacio = !salida.is_empty();
            continue;
        }
        if espacio {
            salida.push(' ');
            espacio = false;
        }
        if c.is_alphanumeric() {
            salida.push(c);
        }
    }
    salida
}

/// Separa el trozo en **lo que va antes, el término y lo que va después**, para que «¿power by?»
/// encuentre lo suyo y salga «¿Power BI?» con sus signos en su sitio.
///
/// Sin esto, la primera versión se comía la apertura: devolvía «Power BI?» y perdía el «¿». Lo cazó
/// su test, que es donde tenía que cazarse.
fn partir(trozo: &str) -> (&str, &str, &str) {
    let Some(primera) = trozo.find(char::is_alphanumeric) else {
        return ("", "", trozo);
    };
    let ultima = trozo.rfind(char::is_alphanumeric).unwrap_or(primera);
    let fin = ultima + trozo[ultima..].chars().next().map(char::len_utf8).unwrap_or(1);
    (&trozo[..primera], &trozo[primera..fin], &trozo[fin..])
}

/// Distancia de edición (Levenshtein) con **corte**: si pasa de `techo`, se deja de calcular.
///
/// El corte no es una optimización decorativa. Este se llama una vez por término y por ventana en
/// cada turno de la reunión, y lo que se compara casi nunca se parece: la respuesta útil es «no» lo
/// antes posible.
fn distancia(a: &str, b: &str, techo: usize) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut fila: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut anterior = fila[0];
        fila[0] = i + 1;
        let mut minimo = fila[0];
        for (j, cb) in b.iter().enumerate() {
            let costo = usize::from(ca != cb);
            let nuevo = (fila[j + 1] + 1).min(fila[j] + 1).min(anterior + costo);
            anterior = fila[j + 1];
            fila[j + 1] = nuevo;
            minimo = minimo.min(nuevo);
        }
        // Toda la fila pasó del techo: ninguna continuación puede bajar de aquí.
        if minimo > techo {
            return techo + 1;
        }
    }
    fila[b.len()]
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn lo_que_el_motor_oye_mal_sale_escrito_bien() {
        let d = Diccionario::semilla();
        assert_eq!(d.corregir("Usamos power by para los tableros"), "Usamos Power BI para los tableros");
        assert_eq!(d.corregir("eso se hace con lake house"), "eso se hace con Lakehouse");
    }

    /// «Semantic Model» son dos palabras y tiene que ganarle a cualquier ventana más corta: si la de
    /// una palabra se resolviera primero, «semantic» saldría solo y «model» detrás, sin corregir.
    #[test]
    fn la_ventana_larga_le_gana_a_la_corta() {
        let d = Diccionario::semilla();
        assert_eq!(d.corregir("el semantic model del cliente"), "el Semantic Model del cliente");
    }

    /// **La decisión que evita que este módulo haga más daño que bien.** Con tres letras, «DAX» está
    /// a una edición de «das», «dos», «tax» y «max». Así que los términos cortos se corrigen SOLO
    /// por variante escrita a mano.
    #[test]
    fn un_termino_corto_no_se_corrige_por_parecido() {
        let d = Diccionario::semilla();
        // «the ax» está en la lista: se corrige porque alguien lo escribió.
        assert_eq!(d.corregir("mide con the ax"), "mide con DAX");
        // «tax» y «max» NO están, y se parecen igual. No se tocan.
        assert_eq!(d.corregir("el tax del año pasado"), "el tax del año pasado");
        assert_eq!(d.corregir("pon el max de la columna"), "pon el max de la columna");
    }

    /// El caso que la primera versión rompía: se comía el signo de apertura.
    #[test]
    fn la_puntuacion_se_queda_donde_estaba() {
        let d = Diccionario::semilla();
        assert_eq!(d.corregir("¿power by?"), "¿Power BI?");
        assert_eq!(d.corregir("sí, powerbi."), "sí, Power BI.");
        assert_eq!(d.corregir("(lake house)"), "(Lakehouse)");
    }

    /// Lo que ya estaba bien escrito no se toca — ni sus mayúsculas.
    #[test]
    fn lo_que_ya_estaba_bien_no_se_reescribe() {
        let d = Diccionario::semilla();
        let dicho = "Power BI y DAX, con Lakehouse";
        assert_eq!(d.corregir(dicho), dicho);
    }

    /// **El filo que de verdad importa: una frase normal sale intacta.** Un corrector que mejora la
    /// jerga y estropea el resto de la reunión sale perdiendo, y eso no se ve en los ejemplos
    /// bonitos: se ve aquí.
    #[test]
    fn una_frase_cualquiera_sale_sin_una_letra_cambiada() {
        let d = Diccionario::semilla();
        for frase in [
            "Nosotros veníamos trabajando con el proveedor anterior y no quedamos contentos",
            "¿Ustedes tienen certificación ISO 27001? Y la limpieza de datos, ¿está en el alcance?",
            "Do you have any references from the retail sector, maybe a case study?",
            "El plazo de entrega son cuatro semanas desde la firma del contrato",
            "Power of the people, the facts are clear, and the data speaks",
        ] {
            assert_eq!(d.corregir(frase), frase, "el diccionario estropeó una frase normal");
        }
    }

    /// Los nombres del cliente salen del CORPUS del usuario, y solo los suficientemente largos.
    #[test]
    fn los_nombres_del_corpus_entran_y_los_cortos_no() {
        let mut d = Diccionario::semilla();
        d.con_nombres_del_corpus(&["Páramo".into(), "Azul".into(), "Cooperativa".into()]);
        assert_eq!(d.cuantos_del_corpus(), 2, "«Azul» tiene cuatro letras y no debería entrar");
        // Con seis letras la tolerancia es 1: una letra de más se corrige.
        assert_eq!(d.corregir("lo de paramos"), "lo de Páramo");
        // «Azul» no entró, así que «asul» se queda como vino.
        assert_eq!(d.corregir("el asul de la marca"), "el asul de la marca");
    }

    /// **El diccionario no aprende de la reunión.** Lo garantiza el tipo —`corregir` toma `&self`—
    /// y esto comprueba la otra mitad: lo que se guarda no lleva el nombre del cliente, porque los
    /// del corpus se recalculan en cada arranque.
    #[test]
    fn lo_que_se_guarda_no_lleva_el_nombre_de_ningun_cliente() {
        let mut d = Diccionario::semilla();
        d.con_nombres_del_corpus(&["Páramo".into(), "Cooperativa Sur del Valle".into()]);
        let guardado = d.a_texto();
        assert!(!guardado.contains("Páramo"), "el archivo del usuario guardó el nombre del cliente");
        assert!(!guardado.contains("Cooperativa"), "el archivo del usuario guardó el nombre del cliente");
        assert!(guardado.contains("Power BI"), "y sí tiene que guardar lo que el usuario escribió");
    }

    #[test]
    fn lo_que_se_escribe_se_vuelve_a_leer_igual() {
        let d = Diccionario::semilla();
        let leido = Diccionario::de_texto(&d.a_texto()).expect("no se pudo releer lo que se escribió");
        assert_eq!(leido.terminos(), d.terminos());
    }

    /// Una línea que no se entiende **se dice con su número**. Comerse en silencio la mitad de lo que
    /// el usuario escribió es peor que no tener diccionario: él cree que la app lo sabe.
    #[test]
    fn una_linea_torcida_se_denuncia_con_su_numero() {
        let e = Diccionario::de_texto("Power BI: [power by]\nesto no lleva dos puntos\n").unwrap_err();
        assert!(e.contains("línea 2"), "el error no dice qué línea: «{e}»");
        let e = Diccionario::de_texto("Power BI: power by\n").unwrap_err();
        assert!(e.contains("corchetes"), "«{e}»");
        let e = Diccionario::de_texto("DAX: [dax]\nDAX: [the ax]\n").unwrap_err();
        assert!(e.contains("ya estaba más arriba"), "«{e}»");
    }

    #[test]
    fn un_archivo_solo_de_comentarios_es_un_diccionario_vacio_y_no_un_error() {
        let d = Diccionario::de_texto("# nada\n\n   \n").expect("los comentarios no son un error");
        assert!(d.terminos().is_empty());
        assert_eq!(d.corregir("lo que sea"), "lo que sea");
    }

    #[test]
    fn la_distancia_corta_cuando_ya_no_hay_nada_que_hacer() {
        assert_eq!(distancia("power bi", "power bi", 2), 0);
        assert_eq!(distancia("powerbi", "power bi", 2), 1);
        // Cuando pasa del techo devuelve techo+1, sin prometer el valor exacto.
        assert!(distancia("alcance", "power bi", 2) > 2);
    }

    #[test]
    fn normalizar_quita_tildes_y_colapsa_espacios() {
        assert_eq!(normalizar("  Páramo   Azul  "), "paramo azul");
        assert_eq!(normalizar("¿Power BI?"), "power bi");
        assert_eq!(normalizar("MAÑANA"), "manana");
    }
}
