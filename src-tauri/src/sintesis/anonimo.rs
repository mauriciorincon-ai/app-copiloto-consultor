//! **LA BÓVEDA** — anonimizar en el Mac lo que va al API, y deshacerlo al volver (patrón Velo,
//! ADR 011). Vive en memoria el tiempo de UNA petición y pisa lo que guardó al morir.
//!
//! Qué se tapa, de más seguro a más ancho:
//! 1. **Los nombres conocidos**: los clientes de tu corpus. Enteros, sin mayúsculas.
//! 2. **Lo que delata a alguien sin estar en ninguna lista**: correos, teléfonos y números largos
//!    (cédulas, NIT, cuentas) —siete dígitos o más—. Las cifras cortas de la evidencia («23 %»,
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

    /// Cuántas cosas se taparon. Es lo que IA enseña en «anonimizados».
    pub fn tapadas(&self) -> usize {
        self.pares.len()
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
        // 2 y 3. Palabra a palabra.
        let palabras: Vec<&str> = t.split(' ').collect();
        let mut salida: Vec<String> = Vec::with_capacity(palabras.len());
        let mut i = 0;
        while i < palabras.len() {
            let p = palabras[i];
            let (nucleo, cola) = separar_puntuacion(p);
            if es_correo(nucleo) {
                let m = self.marcador("CORREO", nucleo);
                salida.push(format!("{m}{cola}"));
                i += 1;
                continue;
            }
            if es_numero_largo(nucleo) {
                let m = self.marcador("NUMERO", nucleo);
                salida.push(format!("{m}{cola}"));
                i += 1;
                continue;
            }
            // Una racha de Nombres Propios: se mira hasta dónde llega.
            let mut j = i;
            while j < palabras.len() && es_nombre_propio(separar_puntuacion(palabras[j]).0) {
                // La puntuación corta la racha: «Torres, y» termina en «Torres».
                if !separar_puntuacion(palabras[j]).1.is_empty() {
                    j += 1;
                    break;
                }
                j += 1;
            }
            if j - i >= 2 {
                let (ultimo, cola) = separar_puntuacion(palabras[j - 1]);
                let mut nombre: Vec<&str> = palabras[i..j - 1].to_vec();
                nombre.push(ultimo);
                let m = self.marcador("PERSONA", &nombre.join(" "));
                salida.push(format!("{m}{cola}"));
                i = j;
            } else {
                salida.push(p.to_string());
                i += 1;
            }
        }
        salida.join(" ")
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

fn separar_puntuacion(p: &str) -> (&str, &str) {
    let fin = p.trim_end_matches(|c: char| ",.;:!?)»\"”".contains(c)).len();
    (&p[..fin], &p[fin..])
}

fn es_correo(p: &str) -> bool {
    let Some((usuario, dominio)) = p.split_once('@') else { return false };
    !usuario.is_empty() && dominio.contains('.') && !dominio.starts_with('.') && !dominio.ends_with('.')
}

/// Siete dígitos o más, contando los que van separados por puntos, guiones o espacios dentro de la
/// palabra («300-555-1234», «900.123.456»).
fn es_numero_largo(p: &str) -> bool {
    let digitos = p.chars().filter(char::is_ascii_digit).count();
    digitos >= 7 && p.chars().all(|c| c.is_ascii_digit() || "+-.()".contains(c))
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
