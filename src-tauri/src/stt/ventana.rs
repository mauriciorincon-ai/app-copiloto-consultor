//! La ventana deslizante de turnos: **lo único que queda de lo que se dijo, y dura poco**.
//!
//! La banda enseña los últimos turnos y el disparador de la fase 4 leerá el último del cliente.
//! Ninguno de los dos necesita la reunión entera, así que la reunión entera no existe: caben
//! [`TURNOS`] y el más viejo se cae solo. Es la misma idea del anillo de audio aplicada al texto —
//! la capacidad hace el trabajo que una promesa no puede hacer.
//!
//! **Y por qué [`Ventana::vaciar`] sobrescribe las letras antes de soltarlas.** Soltar un `String`
//! devuelve la memoria al asignador, que no la limpia: los bytes de lo que dijo el cliente siguen
//! ahí hasta que otra cosa los pise, y «hasta que otra cosa los pise» no es un plazo que se pueda
//! decir en voz alta delante de nadie. El kill-switch de esta app se pulsa en mitad de una
//! reunión; lo que promete es que después no queda nada, no que quedará poco tiempo.

use super::Turno;
use std::collections::VecDeque;

/// Cuántos turnos caben. Doce son más de los que la banda enseña (tres) y más de los que el
/// disparador necesita (uno), con margen para que la fase 4 pueda mirar atrás sin pedir permiso.
pub const TURNOS: usize = 12;

#[derive(Default)]
pub struct Ventana {
    turnos: VecDeque<Turno>,
}

impl Ventana {
    pub fn nueva() -> Self {
        Self { turnos: VecDeque::with_capacity(TURNOS) }
    }

    /// Mete un turno. Si ya no caben, **el más viejo se olvida de verdad**.
    pub fn empujar(&mut self, turno: Turno) {
        while self.turnos.len() >= TURNOS {
            if let Some(mut viejo) = self.turnos.pop_front() {
                olvidar(&mut viejo.texto);
            }
        }
        self.turnos.push_back(turno);
    }

    /// Los últimos `n` turnos, del más viejo al más nuevo.
    pub fn ultimos(&self, n: usize) -> Vec<&Turno> {
        self.turnos.iter().skip(self.turnos.len().saturating_sub(n)).collect()
    }

    /// El último turno de una pista concreta. Es lo que la fase 4 preguntará: «¿qué acaba de
    /// decir el cliente?».
    pub fn ultimo_de(&self, pista: crate::capture::Pista) -> Option<&Turno> {
        self.turnos.iter().rev().find(|t| t.pista == pista)
    }

    pub fn cuantos(&self) -> usize {
        self.turnos.len()
    }

    pub fn esta_vacia(&self) -> bool {
        self.turnos.is_empty()
    }

    /// Los bytes de texto vivos ahora mismo. Es lo que enseña la pantalla de Honestidad, contado y
    /// no estimado.
    pub fn bytes(&self) -> usize {
        self.turnos.iter().map(|t| t.texto.len()).sum()
    }

    /// El kill-switch. Sobrescribe cada turno antes de soltarlo.
    pub fn vaciar(&mut self) {
        for mut t in self.turnos.drain(..) {
            olvidar(&mut t.texto);
        }
    }
}

/// Pisa las letras de un texto con ceros, dejándolo del mismo largo.
///
/// El `unsafe` es de los benignos y conviene decir por qué: `as_mut_vec` obliga a mantener el
/// invariante de UTF-8 válido, y llenar de bytes cero lo mantiene —`\0` es un carácter UTF-8
/// perfectamente legal—. No se puede usar `clear()` porque eso solo mueve la longitud a cero y
/// deja las letras intactas en el respaldo, que es exactamente el disimulo que este módulo existe
/// para no hacer.
fn olvidar(texto: &mut String) {
    unsafe { texto.as_mut_vec() }.fill(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::Pista;

    fn turno(pista: Pista, n: usize) -> Turno {
        Turno {
            pista,
            desde_ms: n * 1_000,
            hasta_ms: n * 1_000 + 800,
            texto: format!("turno número {n}"),
            hora: "14:02".into(),
            eco: false,
        }
    }

    #[test]
    fn la_ventana_no_crece_mas_alla_del_techo() {
        let mut v = Ventana::nueva();
        for n in 0..(TURNOS * 3) {
            v.empujar(turno(Pista::Sistema, n));
        }
        assert_eq!(v.cuantos(), TURNOS);
        let ultimos = v.ultimos(TURNOS);
        assert_eq!(ultimos.first().unwrap().texto, format!("turno número {}", TURNOS * 2));
        assert_eq!(ultimos.last().unwrap().texto, format!("turno número {}", TURNOS * 3 - 1));
    }

    #[test]
    fn se_puede_preguntar_por_lo_ultimo_que_dijo_cada_uno() {
        let mut v = Ventana::nueva();
        v.empujar(turno(Pista::Sistema, 1));
        v.empujar(turno(Pista::Microfono, 2));
        v.empujar(turno(Pista::Sistema, 3));
        assert_eq!(v.ultimo_de(Pista::Sistema).unwrap().desde_ms, 3_000);
        assert_eq!(v.ultimo_de(Pista::Microfono).unwrap().desde_ms, 2_000);
    }

    #[test]
    fn una_ventana_recien_nacida_no_tiene_ultimo_de_nadie() {
        let v = Ventana::nueva();
        assert!(v.esta_vacia());
        assert_eq!(v.bytes(), 0);
        assert!(v.ultimo_de(Pista::Sistema).is_none());
        assert!(v.ultimos(3).is_empty());
    }

    #[test]
    fn los_bytes_salen_de_contar_el_texto() {
        let mut v = Ventana::nueva();
        v.empujar(Turno { pista: Pista::Sistema, desde_ms: 0, hasta_ms: 1, texto: "hola".into(), hora: String::new(), eco: false });
        v.empujar(Turno { pista: Pista::Microfono, desde_ms: 0, hasta_ms: 1, texto: "sí".into(), hora: String::new(), eco: false });
        assert_eq!(v.bytes(), 4 + 3, "«sí» son tres bytes en UTF-8, no dos");
    }

    #[test]
    fn vaciar_deja_la_ventana_sin_turnos() {
        let mut v = Ventana::nueva();
        for n in 0..5 {
            v.empujar(turno(Pista::Sistema, n));
        }
        v.vaciar();
        assert!(v.esta_vacia());
        assert_eq!(v.bytes(), 0);
    }

    /// **El gate del olvido.** Es la mitad que un test sobre la ventana no puede ver: una vez
    /// soltado el `String`, nadie puede mirarlo. Así que se prueba la pieza que lo pisa, antes de
    /// soltarlo.
    ///
    /// Se ve en rojo cambiando el cuerpo de `olvidar` por `texto.clear()`: la longitud baja a
    /// cero, la ventana parece vacía, y este test enseña las letras que siguen en el respaldo.
    #[test]
    fn olvidar_pisa_las_letras_y_no_solo_la_longitud() {
        let mut texto = String::from("¿Ustedes tienen certificación ISO 27001?");
        let largo = texto.len();
        olvidar(&mut texto);
        assert_eq!(texto.len(), largo, "olvidar no debe reasignar: pisa lo que hay donde está");
        let superviviente = texto.bytes().position(|b| b != 0);
        assert!(
            superviviente.is_none(),
            "el byte {} de lo que dijo el cliente sigue en memoria después de olvidar",
            superviviente.unwrap()
        );
    }

    #[test]
    fn el_turno_que_se_cae_por_viejo_tambien_se_olvida() {
        // No se puede mirar la memoria liberada, pero sí que el camino pasa por `olvidar`: si
        // `empujar` soltara el turno viejo con un `pop_front()` a secas, este test seguiría verde
        // — por eso el gate de verdad es el de arriba, y este solo fija el comportamiento visible.
        let mut v = Ventana::nueva();
        for n in 0..(TURNOS + 1) {
            v.empujar(turno(Pista::Sistema, n));
        }
        assert_eq!(v.cuantos(), TURNOS);
        assert!(v.ultimos(TURNOS).iter().all(|t| t.texto != "turno número 0"));
    }
}
