//! EL TROCEADO POR SECCIÓN — lo que decide de qué se acordará la ficha.
//!
//! La orden pide *«chunking por sección»*, y la razón está en la ficha: su tercera línea es la
//! **fuente exacta (documento y sección)**. Trocear cada 500 palabras daría fuentes como
//! «Propuesta, trozo 7», que no le sirve de nada a quien tiene que abrir el documento en mitad
//! de una reunión.
//!
//! El reparto de responsabilidades: **cada lector sabe reconocer sus propios títulos** —el `#`
//! del Markdown, el `pStyle` del .docx, la forma de la línea en un PDF— y entrega líneas
//! marcadas. Aquí se convierten en secciones, una sola vez y de una sola manera.
//!
//! **Lo que este módulo NO puede arreglar:** un PDF no trae títulos, trae líneas. Lo que el
//! lector de PDF marca es una conjetura, y `Documento::secciones_conjeturadas` lo dice para que
//! la pantalla pueda decirlo también. Un documento del que no se pudo sacar ninguna sección se
//! indexa **entero, como una sola sección sin título**, y su ficha cita el documento sin
//! sección: es menos preciso, pero no es falso.

/// Techo de una sección antes de partirla. Una sección de treinta páginas hace ganar a BM25 por
/// tamaño y deja una fuente inútil («Anexo», en un anexo de treinta páginas).
///
/// Al partir, las piezas conservan el título de su sección: son la misma sección, contada en dos
/// tramos, y la fuente que ve el usuario no cambia.
const TECHO: usize = 1_200;

/// Debajo de esto, un fragmento no se indexa por su cuenta: se pega al anterior. Evita las
/// secciones de una línea —un pie de página, un número de página suelto— compitiendo en BM25
/// con el cuerpo real del documento.
const MINIMO: usize = 80;

/// Una línea tal y como la entrega un lector, ya con el veredicto sobre si es título.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Linea {
    pub texto: String,
    pub titulo: bool,
}

impl Linea {
    pub fn cuerpo(texto: impl Into<String>) -> Self {
        Linea { texto: texto.into(), titulo: false }
    }
    pub fn titulo(texto: impl Into<String>) -> Self {
        Linea { texto: texto.into(), titulo: true }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seccion {
    /// `None` cuando el documento no tenía títulos o el texto viene antes del primero.
    pub titulo: Option<String>,
    pub texto: String,
}

/// Convierte las líneas de un lector en secciones indexables.
pub fn trocear(lineas: &[Linea]) -> Vec<Seccion> {
    let mut crudas: Vec<Seccion> = Vec::new();
    let mut titulo: Option<String> = None;
    let mut cuerpo = String::new();

    let cerrar = |crudas: &mut Vec<Seccion>, titulo: &Option<String>, cuerpo: &mut String| {
        let t = cuerpo.trim();
        if !t.is_empty() {
            crudas.push(Seccion { titulo: titulo.clone(), texto: t.to_string() });
        }
        cuerpo.clear();
    };

    for l in lineas {
        let texto = l.texto.trim();
        if texto.is_empty() {
            continue;
        }
        if l.titulo {
            cerrar(&mut crudas, &titulo, &mut cuerpo);
            titulo = Some(texto.to_string());
        } else {
            if !cuerpo.is_empty() {
                cuerpo.push('\n');
            }
            cuerpo.push_str(texto);
        }
    }
    cerrar(&mut crudas, &titulo, &mut cuerpo);

    // Un título sin una sola línea de cuerpo debajo sigue siendo una sección: es el caso de una
    // portada, o de un índice. Se indexa con su propio título como texto para que se pueda
    // encontrar por él.
    if crudas.is_empty() {
        if let Some(t) = titulo {
            crudas.push(Seccion { texto: t.clone(), titulo: Some(t) });
        }
    }

    pegar_las_migajas(partir_las_largas(crudas))
}

/// Parte las secciones que pasan del techo, por frases, conservando el título.
fn partir_las_largas(secciones: Vec<Seccion>) -> Vec<Seccion> {
    let mut salida = Vec::new();
    for s in secciones {
        if s.texto.chars().count() <= TECHO {
            salida.push(s);
            continue;
        }
        let mut trozo = String::new();
        for frase in frases(&s.texto) {
            if !trozo.is_empty() && trozo.chars().count() + frase.chars().count() > TECHO {
                salida.push(Seccion { titulo: s.titulo.clone(), texto: trozo.trim().to_string() });
                trozo = String::new();
            }
            trozo.push_str(&frase);
        }
        if !trozo.trim().is_empty() {
            salida.push(Seccion { titulo: s.titulo.clone(), texto: trozo.trim().to_string() });
        }
    }
    salida
}

/// Pega al anterior lo que es demasiado corto para competir solo. Nunca pega a través de un
/// cambio de título: perdería la fuente, que es justamente lo que esto existe para conservar.
fn pegar_las_migajas(secciones: Vec<Seccion>) -> Vec<Seccion> {
    let mut salida: Vec<Seccion> = Vec::new();
    for s in secciones {
        let corta = s.texto.chars().count() < MINIMO;
        match salida.last_mut() {
            Some(previa) if corta && previa.titulo == s.titulo => {
                previa.texto.push('\n');
                previa.texto.push_str(&s.texto);
            }
            _ => salida.push(s),
        }
    }
    salida
}

/// Corta por final de frase conservando el separador, para que al volver a pegar los trozos el
/// texto siga leyéndose. Un punto entre dígitos («ISO 27.001», «1.500») no termina una frase.
fn frases(texto: &str) -> Vec<String> {
    let cs: Vec<char> = texto.chars().collect();
    let mut salida = Vec::new();
    let mut actual = String::new();
    for (i, &c) in cs.iter().enumerate() {
        actual.push(c);
        let cierra = matches!(c, '.' | '!' | '?' | '\n');
        let entre_digitos = c == '.'
            && i > 0
            && cs[i - 1].is_ascii_digit()
            && cs.get(i + 1).is_some_and(|s| s.is_ascii_digit());
        if cierra && !entre_digitos {
            salida.push(std::mem::take(&mut actual));
        }
    }
    if !actual.is_empty() {
        salida.push(actual);
    }
    salida
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn largo(n: usize) -> String {
        // Frases reales, para que el partidor tenga por dónde cortar.
        std::iter::repeat_n("Medimos la rentabilidad de cada canal. ", n).collect()
    }

    #[test]
    fn cada_titulo_abre_una_seccion_y_se_queda_con_su_cuerpo() {
        let s = trocear(&[
            Linea::titulo("Alcance"),
            Linea::cuerpo("Tres canales y una línea base de doce meses."),
            Linea::titulo("Precio"),
            Linea::cuerpo("Cuatro semanas, tarifa cerrada."),
        ]);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].titulo.as_deref(), Some("Alcance"));
        assert!(s[0].texto.contains("doce meses"));
        assert_eq!(s[1].titulo.as_deref(), Some("Precio"));
        assert!(!s[1].texto.contains("doce meses"), "una sección se llevó el cuerpo de la otra");
    }

    /// El caso del PDF sin estructura: se indexa entero, con la fuente menos precisa pero cierta.
    #[test]
    fn un_documento_sin_titulos_es_una_sola_seccion_sin_titulo() {
        let s = trocear(&[
            Linea::cuerpo("Medimos la rentabilidad de cada canal."),
            Linea::cuerpo("En cuatro semanas."),
        ]);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].titulo, None);
        assert!(s[0].texto.contains("cuatro semanas"));
    }

    /// El texto anterior al primer título no se pierde ni se le atribuye al título que viene
    /// después: sería citar una fuente equivocada.
    #[test]
    fn lo_que_va_antes_del_primer_titulo_no_se_le_atribuye_a_ese_titulo() {
        let s = trocear(&[
            Linea::cuerpo(largo(3)),
            Linea::titulo("Alcance"),
            Linea::cuerpo(largo(3)),
        ]);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].titulo, None);
        assert_eq!(s[1].titulo.as_deref(), Some("Alcance"));
    }

    #[test]
    fn una_seccion_larga_se_parte_pero_conserva_su_fuente() {
        let s = trocear(&[Linea::titulo("Anexo"), Linea::cuerpo(largo(120))]);
        assert!(s.len() > 1, "una sección de {} caracteres no se partió", largo(120).len());
        for t in &s {
            assert_eq!(t.titulo.as_deref(), Some("Anexo"), "un trozo perdió su sección");
            assert!(t.texto.chars().count() <= TECHO + 40);
        }
    }

    /// Un pie de página no puede competir en BM25 con el cuerpo del documento.
    #[test]
    fn las_migajas_se_pegan_a_su_seccion_en_vez_de_competir_solas() {
        let s = trocear(&[
            Linea::titulo("Alcance"),
            Linea::cuerpo(largo(4)),
            Linea::cuerpo("p. 3"),
        ]);
        assert_eq!(s.len(), 1);
        assert!(s[0].texto.ends_with("p. 3"));
    }

    /// Pero jamás a través de un cambio de título: eso sí perdería la fuente.
    #[test]
    fn una_migaja_no_se_pega_a_la_seccion_de_al_lado() {
        let s = trocear(&[
            Linea::titulo("Alcance"),
            Linea::cuerpo(largo(4)),
            Linea::titulo("Precio"),
            Linea::cuerpo("Cerrado."),
        ]);
        assert_eq!(s.len(), 2);
        assert_eq!(s[1].titulo.as_deref(), Some("Precio"));
        assert_eq!(s[1].texto, "Cerrado.");
    }

    /// El mismo defecto que ya mordió en la transcripción: el motor escribe «ISO 27.001» con
    /// separador de millares, y un partidor ingenuo lo corta en dos frases por ese punto.
    #[test]
    fn un_punto_entre_digitos_no_termina_una_frase() {
        let f = frases("Cumple ISO 27.001 desde 2019. Y la 9.001 también.");
        assert_eq!(f.len(), 2, "se partió por el separador de millares: {f:?}");
        assert!(f[0].contains("27.001"));
    }

    #[test]
    fn un_titulo_solo_sigue_siendo_indexable_por_su_titulo() {
        let s = trocear(&[Linea::titulo("Páramo Azul")]);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].titulo.as_deref(), Some("Páramo Azul"));
        assert_eq!(s[0].texto, "Páramo Azul");
    }
}
