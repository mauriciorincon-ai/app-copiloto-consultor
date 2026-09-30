//! **LA BÓVEDA** — anonimizar en el Mac lo que va al API, y deshacerlo al volver (patrón Velo,
//! ADR 011). Vive en memoria el tiempo de UNA petición y pisa lo que guardó al morir.
//!
//! Qué se tapa, de más seguro a más ancho:
//! 1. **Los nombres conocidos**: los clientes de tu corpus. Enteros, sin mayúsculas.
//! 2. **Lo que delata a alguien sin estar en ninguna lista**: correos, teléfonos y números largos
//!    (cédulas, NIT, cuentas) —siete dígitos o más, aunque vengan partidos por espacios, puntos o
//!    guiones: «300 555 1234», «900.123.456»—. Las cifras cortas de la evidencia («23 %»,
//!    «9 semanas», «2026») se quedan: son lo que la sugerencia necesita.
//! 3. **Parejas de Nombres Propios** («Andrea Villalba», «Ana Torres»): dos o más palabras seguidas
//!    que empiezan en mayúscula y siguen en minúscula. Tapa también cosas que no son personas
//!    («Google Meet»), y es a propósito: **en la duda, se tapa de más**. Tapar de más empeora un
//!    poco la sugerencia; tapar de menos saca un nombre del Mac.
//!
//! Lo que NO se promete, dicho en el manual: un nombre suelto que no esté en el corpus («Andrea»)
//! no se reconoce.

/// La bóveda de una petición.
#[derive(Debug, Default)]
pub struct Boveda {
    /// Nombres conocidos, del más largo al más corto: «Páramo Azul S.A.S.» antes que «Páramo Azul».
    conocidos: Vec<String>,
    /// Lo tapado: (marcador, original).
    pares: Vec<(String, String)>,
}

impl Boveda {
    pub fn nueva(conocidos: &[String]) -> Self {
        let mut c: Vec<String> = conocidos
            .iter()
            .map(|n| n.trim().to_string())
            .filter(|n| n.chars().count() >= 3)
            .collect();
        c.sort_by_key(|n| std::cmp::Reverse(n.chars().count()));
        c.dedup();
        Self { conocidos: c, pares: Vec::new() }
    }

    /// Cuántas cosas se taparon. Desde el sprint 003 viaja con cada petición en
    /// `api::LoQueSalio::tapadas`, que es lo que la pantalla IA pide (auditoría del S2, B37).
    pub fn tapadas(&self) -> usize {
        self.pares.len()
    }

    /// Lo que un marcador tapó, para enseñarlo **en el Mac** al lado de lo que salió.
    pub fn original_de(&self, marcador: &str) -> Option<&str> {
        self.pares.iter().find(|(m, _)| m == marcador).map(|(_, o)| o.as_str())
    }

    fn marcador(&mut self, clase: &str, original: &str) -> String {
        if let Some((m, _)) = self.pares.iter().find(|(_, o)| o.eq_ignore_ascii_case(original)) {
            return m.clone();
        }
        let n = self.pares.iter().filter(|(m, _)| m.starts_with(&format!("[{clase}_"))).count() + 1;
        let m = format!("[{clase}_{n}]");
        self.pares.push((m.clone(), original.to_string()));
        m
    }

    /// **Tapar.** Devuelve el texto que puede salir.
    pub fn tapar(&mut self, texto: &str) -> String {
        let mut t = texto.to_string();
        // 1. Los conocidos, enteros y sin mayúsculas.
        for nombre in self.conocidos.clone() {
            while let Some(i) = buscar_sin_mayusculas(&t, &nombre) {
                let original = t[i..i + nombre.len()].to_string();
                let m = self.marcador("CLIENTE", &original);
                t.replace_range(i..i + nombre.len(), &m);
            }
        }
        // 2 y 3. Palabra a palabra, partiendo por CUALQUIER blanco —el texto que sale lleva saltos
        // de línea entre el turno y cada ficha— y guardando cada blanco tal cual, para que el texto
        // se rearme idéntico y ninguna etiqueta «F1» quede pegada a lo que se tapa.
        let (prefijo, piezas) = piezas(&t);
        let mut salida = String::with_capacity(t.len());
        salida.push_str(prefijo);
        let mut i = 0;
        while i < piezas.len() {
            // Un número partido por espacios («300 555 1234») es UN número: se juntan las piezas
            // que solo tienen cifras y signos de teléfono, en la misma línea.
            let mut j = i;
            let mut digitos = 0;
            while let Some((abre, cifras, cierra)) = piezas.get(j).and_then(numero) {
                // Solo la primera puede traer algo delante, y la última algo detrás.
                if j > i && !abre.is_empty() {
                    break;
                }
                digitos += cifras.chars().filter(char::is_ascii_digit).count();
                j += 1;
                if !cierra.is_empty() || !piezas[j - 1].blanco.chars().all(|c| c == ' ') {
                    break;
                }
            }
            if j > i && digitos >= 7 {
                let (abre, _, _) = numero(&piezas[i]).unwrap_or_default();
                let (_, _, cierra) = numero(&piezas[j - 1]).unwrap_or_default();
                let todo = unir(&piezas[i..j]);
                let original = &todo[abre.len()..todo.len() - cierra.len()];
                let m = self.marcador("NUMERO", original);
                salida.push_str(&format!("{abre}{m}{cierra}{}", piezas[j - 1].blanco));
                i = j;
                continue;
            }
            let p = &piezas[i];
            if es_correo(p.nucleo) {
                let m = self.marcador("CORREO", p.nucleo);
                salida.push_str(&format!("{}{m}{}{}", p.cabeza, p.cola, p.blanco));
                i += 1;
                continue;
            }
            // Una racha de Nombres Propios: sigue mientras no haya puntuación entre dos palabras y
            // las separe un espacio de la misma línea. «(Juan Pérez)» y «¿Andrea Villalba?» son
            // rachas; «Torres, y» termina en «Torres»; un salto de línea termina cualquier racha.
            let mut j = i;
            while j < piezas.len() && es_nombre_propio(piezas[j].nucleo) {
                j += 1;
                let (esta, sigue) = (&piezas[j - 1], piezas.get(j));
                let continua = esta.cola.is_empty()
                    && esta.blanco.chars().all(|c| c == ' ')
                    && sigue.is_some_and(|s| s.cabeza.is_empty());
                if !continua {
                    break;
                }
            }
            if j - i >= 2 {
                let (primera, ultima) = (&piezas[i], &piezas[j - 1]);
                let mut nombre = String::new();
                for (k, q) in piezas[i..j].iter().enumerate() {
                    nombre.push_str(q.nucleo);
                    if k + 1 < j - i {
                        nombre.push_str(q.blanco);
                    }
                }
                let m = self.marcador("PERSONA", &nombre);
                salida.push_str(&format!("{}{m}{}{}", primera.cabeza, ultima.cola, ultima.blanco));
                i = j;
            } else {
                salida.push_str(p.palabra);
                salida.push_str(p.blanco);
                i += 1;
            }
        }
        salida
    }

    /// **Destapar** lo que volvió: cada marcador por su original, en el Mac.
    pub fn destapar(&self, texto: &str) -> String {
        let mut t = texto.to_string();
        for (m, o) in &self.pares {
            t = t.replace(m.as_str(), o);
        }
        t
    }
}

impl Drop for Boveda {
    fn drop(&mut self) {
        for (_, o) in self.pares.iter_mut() {
            // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
            unsafe { o.as_mut_vec() }.fill(0);
        }
    }
}

fn buscar_sin_mayusculas(texto: &str, nombre: &str) -> Option<usize> {
    let (t, n) = (texto.to_lowercase(), nombre.to_lowercase());
    // Solo si las minúsculas no cambian los largos (casi siempre): si cambian, se busca tal cual.
    if t.len() != texto.len() || n.len() != nombre.len() {
        return texto.find(nombre);
    }
    let mut desde = 0;
    while let Some(k) = t[desde..].find(&n) {
        let i = desde + k;
        let antes = t[..i].chars().next_back();
        let despues = t[i + n.len()..].chars().next();
        if !antes.is_some_and(char::is_alphanumeric) && !despues.is_some_and(char::is_alphanumeric) {
            return Some(i);
        }
        desde = i + n.len();
    }
    None
}

/// Una palabra del texto, con la puntuación que la abre y la cierra separada, y el blanco que la
/// sigue tal cual (espacio, salto de línea…): con eso el texto se rearma idéntico.
struct Pieza<'a> {
    palabra: &'a str,
    cabeza: &'a str,
    nucleo: &'a str,
    cola: &'a str,
    blanco: &'a str,
}

const ABRE: &str = "¿¡(«\"“'[";
const CIERRA: &str = ",.;:!?)»\"”'…]";

fn piezas(t: &str) -> (&str, Vec<Pieza<'_>>) {
    let (prefijo, mut resto) = t.split_at(t.len() - t.trim_start().len());
    let mut v = Vec::new();
    while !resto.is_empty() {
        let (palabra, tras) = resto.split_at(resto.find(char::is_whitespace).unwrap_or(resto.len()));
        let (blanco, sigue) = tras.split_at(tras.len() - tras.trim_start().len());
        let sin_cabeza = palabra.trim_start_matches(|c: char| ABRE.contains(c));
        let nucleo = sin_cabeza.trim_end_matches(|c: char| CIERRA.contains(c));
        v.push(Pieza {
            palabra,
            cabeza: &palabra[..palabra.len() - sin_cabeza.len()],
            nucleo,
            cola: &sin_cabeza[nucleo.len()..],
            blanco,
        });
        resto = sigue;
    }
    (prefijo, v)
}

/// Las piezas seguidas, con sus blancos de por medio: lo que se tapó, tal como estaba.
fn unir(piezas: &[Pieza<'_>]) -> String {
    let mut s = String::new();
    for (k, p) in piezas.iter().enumerate() {
        s.push_str(p.palabra);
        if k + 1 < piezas.len() {
            s.push_str(p.blanco);
        }
    }
    s
}

/// Si la palabra es un trozo de teléfono, cédula o cuenta —cifras y `+ - . ( )`—, la separa de la
/// puntuación de frase que la rodea: (lo de delante, el número, lo de detrás).
fn numero<'a>(p: &Pieza<'a>) -> Option<(&'a str, &'a str, &'a str)> {
    let w = p.palabra;
    let sin_abre = w.trim_start_matches(|c: char| "¿¡«\"“'".contains(c));
    let cifras = sin_abre.trim_end_matches(|c: char| ",;:!?»\"”'…".contains(c));
    let es = cifras.chars().any(|c| c.is_ascii_digit())
        && cifras.chars().all(|c| c.is_ascii_digit() || "+-.()".contains(c));
    es.then(|| (&w[..w.len() - sin_abre.len()], cifras, &sin_abre[cifras.len()..]))
}

fn es_correo(p: &str) -> bool {
    let Some((usuario, dominio)) = p.split_once('@') else { return false };
    !usuario.is_empty() && dominio.contains('.') && !dominio.starts_with('.') && !dominio.ends_with('.')
}

fn es_nombre_propio(p: &str) -> bool {
    let mut cs = p.chars();
    let Some(primera) = cs.next() else { return false };
    let resto: Vec<char> = cs.collect();
    primera.is_uppercase() && resto.len() >= 2 && resto.iter().all(|c| c.is_lowercase())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn boveda() -> Boveda {
        Boveda::nueva(&["Páramo Azul".into(), "Cooperativa Sur del Valle".into()])
    }

    /// **Lo que sale no lleva ninguno de los nombres plantados**, y al volver se restauran todos.
    #[test]
    fn los_nombres_plantados_no_salen_y_vuelven_al_destapar() {
        let mut b = boveda();
        let texto = "Andrea Villalba, de páramo azul, pregunta si la Cooperativa Sur del Valle \
                     usó el Excel. Escríbele a andrea@paramo.co o al 300-555-1234; NIT 900.123.456. \
                     El margen es 23 % en 9 semanas.";
        let fuera = b.tapar(texto);
        for plantado in ["Andrea Villalba", "páramo azul", "Cooperativa Sur del Valle", "andrea@paramo.co", "300-555-1234", "900.123.456"] {
            assert!(!fuera.contains(plantado), "«{plantado}» salió: {fuera}");
        }
        // Las cifras de la evidencia se quedan.
        assert!(fuera.contains("23 %") && fuera.contains("9 semanas"), "{fuera}");
        assert_eq!(b.destapar(&fuera), texto);
        assert!(b.tapadas() >= 5);
    }

    /// El modelo repite un marcador en su respuesta: vuelve con el nombre original.
    #[test]
    fn la_respuesta_del_modelo_se_destapa() {
        let mut b = boveda();
        let fuera = b.tapar("¿Y Páramo Azul tiene el Excel?");
        assert!(fuera.contains("[CLIENTE_1]"));
        assert_eq!(b.destapar("Confirma a [CLIENTE_1] las tres fuentes."), "Confirma a Páramo Azul las tres fuentes.");
    }

    /// **Lo que de verdad sale**: el texto de `Peticion::texto`, con el turno en la primera línea
    /// y una ficha por línea detrás. Cada turno lleva un dato plantado en una posición que el
    /// tokenizador por espacios dejaba pasar (auditoría del S2, A1): tras «¿» o «(», entre
    /// comillas, pegado al salto de línea de la ficha, o un teléfono partido por espacios.
    #[test]
    fn los_datos_plantados_no_salen_en_ninguna_posicion_del_texto_real() {
        use super::super::{pruebas::respaldo, Peticion};
        for (turno, plantado) in [
            ("¿Andrea Villalba ya lo aprobó?", "Villalba"),
            ("Lo aprobó Andrea Villalba", "Villalba"),
            ("Mi celular es 300 555 1234", "555 1234"),
            ("llámame al 3005551234", "3005551234"),
            ("(Juan Pérez) dijo que sí", "Pérez"),
            ("«Ana Torres» lo firmó", "Torres"),
            ("escríbele a andrea@paramo.co", "andrea@"),
        ] {
            let texto = Peticion::nueva(turno, &respaldo()).unwrap().texto();
            let mut b = Boveda::default();
            let fuera = b.tapar(&texto);
            assert!(!fuera.contains(plantado), "«{plantado}» salió:\n{fuera}");
            for id in ["F1 · ", "F2 · ", "F3 · "] {
                assert!(fuera.contains(id), "se perdió «{id}» (el modelo ya no podría citarla):\n{fuera}");
            }
            assert_eq!(b.destapar(&fuera), texto, "el texto no vuelve igual");
        }
    }

    #[test]
    fn un_nombre_dentro_de_otra_palabra_no_se_tapa() {
        let mut b = Boveda::nueva(&["Azul".into()]);
        assert_eq!(b.tapar("azulejo y lapislázuli"), "azulejo y lapislázuli");
    }

    #[test]
    fn una_sola_palabra_con_mayuscula_no_es_una_persona() {
        let mut b = Boveda::default();
        assert_eq!(b.tapar("Confirma el alcance con Power BI."), "Confirma el alcance con Power BI.");
    }
}
