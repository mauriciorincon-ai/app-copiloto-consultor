//! El búfer circular: **dónde vive el audio, y por cuánto tiempo**.
//!
//! Es la pieza que hace cierta la primera regla dura de la app. No porque prometa nada, sino por
//! su forma: tiene un tamaño fijo que se reserva al nacer y **nunca crece**. Grabar una reunión
//! de una hora en un anillo de treinta segundos es imposible, no está prohibido — a los veintiún
//! segundos las muestras del principio ya se pisaron con las nuevas. La diferencia importa: una
//! prohibición se puede saltar con un `if` mal puesto; una capacidad no.
//!
//! **Y por qué [`Anillo::vaciar`] escribe ceros en lugar de mover un índice.** Poner el cursor a
//! cero es lo barato y lo que haría cualquiera: el búfer «queda vacío» porque nadie va a leer más
//! allá del cursor. Pero las muestras siguen ahí, en RAM, íntegras, y cualquiera que mire la
//! memoria del proceso —un depurador, un volcado, el swap si el sistema decide paginar— las
//! encuentra enteras. El kill-switch de esta app se pulsa **delante del cliente**; si después de
//! pulsarlo su voz sigue en la memoria del proceso, la tecla es un adorno. Así que se sobrescribe.

/// Cuánto audio cabe, y con qué se llena. Un anillo de `hz × segundos` muestras de 32 bits.
///
/// Mono siempre: las dos pistas son dos anillos distintos porque el hablante se sabe **por
/// origen** (regla de cero huellas de voz), no por analizar la señal. Mezclarlas en estéreo
/// ahorraría memoria y nos dejaría sin la única atribución honesta que tenemos.
pub struct Anillo {
    muestras: Box<[f32]>,
    /// Dónde se escribe la siguiente muestra.
    cursor: usize,
    /// Cuántas muestras válidas hay (deja de crecer al llenarse; a partir de ahí es la capacidad).
    escritas: usize,
    /// Cuántas muestras han pasado por aquí **desde siempre**, se hayan pisado o no.
    ///
    /// Es el reloj que permite pedir «el trozo entre el segundo 12,4 y el 14,1 de esta pista» sin
    /// copiar el audio a ningún otro sitio. Sin él, quien quisiera el audio de un turno tendría
    /// que ir guardándoselo aparte mientras el turno ocurre — y entonces la voz del cliente
    /// viviría en dos sitios a la vez, con dos sitios que vaciar y uno que alguien olvidará.
    totales: u64,
    hz: u32,
}

/// Cuánto audio guarda cada pista. Treinta segundos son ~1,9 MB por anillo a 16 kHz: de sobra
/// para el turno más largo que un humano encadena sin respirar, y lo bastante poco para que la
/// app no se note en la memoria de un Mac en mitad de una videollamada.
pub const SEGUNDOS: f32 = 30.0;

/// La frecuencia a la que vive todo el audio dentro de la app.
///
/// No es la que entrega macOS (el tap da 48 kHz y el micrófono lo que diga el dispositivo): es a
/// la que se **remuestrea al entrar**, y por dos razones que apuntan al mismo sitio. Es la que
/// quieren tanto el detector de voz como el transcriptor, así que convertir una vez al entrar
/// evita convertir dos veces después; y a 16 kHz el anillo ocupa un tercio, que es un tercio de
/// voz del cliente viva en memoria.
pub const HZ: u32 = 16_000;

impl Anillo {
    pub fn nuevo(hz: u32, segundos: f32) -> Self {
        let capacidad = ((hz as f32) * segundos).ceil().max(1.0) as usize;
        Self {
            muestras: vec![0.0; capacidad].into_boxed_slice(),
            cursor: 0,
            escritas: 0,
            totales: 0,
            hz,
        }
    }

    /// Un anillo con la configuración de la app.
    pub fn de_la_app() -> Self {
        Self::nuevo(HZ, SEGUNDOS)
    }

    pub fn capacidad(&self) -> usize {
        self.muestras.len()
    }

    pub fn hz(&self) -> u32 {
        self.hz
    }

    /// Cuántas muestras válidas hay dentro ahora mismo.
    pub fn ocupadas(&self) -> usize {
        self.escritas
    }

    /// Cuántas muestras han entrado desde que se abrió el grifo, incluidas las que ya se pisaron.
    /// Es el índice global con el que se piden trozos concretos.
    pub fn totales(&self) -> u64 {
        self.totales
    }

    /// Los bytes que esta pista tiene vivos en memoria. Es lo que la pantalla de Honestidad
    /// enseña, y sale de contar, no de estimar.
    pub fn bytes(&self) -> usize {
        self.escritas * std::mem::size_of::<f32>()
    }

    /// Cuántos segundos de audio hay dentro.
    pub fn segundos(&self) -> f32 {
        self.escritas as f32 / self.hz as f32
    }

    /// Mete muestras nuevas. Si no caben, las viejas se pierden — que es justo lo que queremos.
    pub fn escribir(&mut self, entrada: &[f32]) {
        let cap = self.muestras.len();
        // Un bloque más grande que el anillo entero solo puede dejar dentro su cola: lo anterior
        // se habría pisado a sí mismo en el camino. Copiarlo entero sería dar vueltas para nada.
        let utiles = if entrada.len() > cap { &entrada[entrada.len() - cap..] } else { entrada };
        for m in utiles {
            self.muestras[self.cursor] = *m;
            self.cursor = (self.cursor + 1) % cap;
        }
        self.escritas = (self.escritas + utiles.len()).min(cap);
        // Se cuentan las que ENTRARON, no las que cupieron: si el contador saltara los descartes,
        // dos pistas que reciben lo mismo acabarían con relojes distintos.
        self.totales += entrada.len() as u64;
    }

    /// El trozo `[desde, hasta)` en índices globales, o `None` si ya se pisó.
    ///
    /// Devolver `None` en vez de «lo que quede» es deliberado: un turno recortado por detrás se
    /// transcribe igual y produce una frase a medias que nadie sabría que está a medias. Mejor
    /// decir que ese audio ya no existe.
    pub fn rango(&self, desde: u64, hasta: u64) -> Option<Vec<f32>> {
        if hasta > self.totales || desde > hasta {
            return None;
        }
        let mas_viejo = self.totales - self.escritas as u64;
        if desde < mas_viejo {
            return None;
        }
        let cap = self.muestras.len();
        let cuantas = (hasta - desde) as usize;
        // Dónde está `desde` dentro del anillo: se cuenta hacia atrás desde el cursor de escritura.
        let atras = (self.totales - desde) as usize;
        let inicio = (self.cursor + cap - atras % cap.max(1)) % cap.max(1);
        Some((0..cuantas).map(|i| self.muestras[(inicio + i) % cap]).collect())
    }

    /// Las últimas `n` muestras, en orden cronológico. Si se piden más de las que hay, devuelve
    /// las que hay.
    pub fn ultimas(&self, n: usize) -> Vec<f32> {
        let n = n.min(self.escritas);
        let cap = self.muestras.len();
        let mut salida = Vec::with_capacity(n);
        // `cursor` apunta a la siguiente escritura, así que la última muestra escrita está justo
        // detrás. Se retrocede `n` desde ahí, dando la vuelta al anillo si hace falta.
        let inicio = (self.cursor + cap - n) % cap;
        for i in 0..n {
            salida.push(self.muestras[(inicio + i) % cap]);
        }
        salida
    }

    /// Las últimas `segundos` de audio.
    pub fn ultimos_segundos(&self, segundos: f32) -> Vec<f32> {
        self.ultimas(((self.hz as f32) * segundos).ceil().max(0.0) as usize)
    }

    /// **Vaciar de verdad.** Sobrescribe todas las muestras con ceros y reinicia el cursor.
    ///
    /// Lo de sobrescribir no es celo: es la diferencia entre un kill-switch y un cartel. Ver la
    /// nota de cabecera del módulo.
    pub fn vaciar(&mut self) {
        self.muestras.fill(0.0);
        self.cursor = 0;
        self.escritas = 0;
        self.totales = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lo_que_no_cabe_se_pisa_y_el_anillo_no_crece() {
        let mut a = Anillo::nuevo(10, 1.0); // 10 muestras
        assert_eq!(a.capacidad(), 10);
        let entrada: Vec<f32> = (1..=25).map(|i| i as f32).collect();
        a.escribir(&entrada);
        assert_eq!(a.capacidad(), 10, "el anillo creció: la promesa del efímero era su tamaño");
        assert_eq!(a.ocupadas(), 10);
        assert_eq!(a.ultimas(10), (16..=25).map(|i| i as f32).collect::<Vec<_>>());
    }

    #[test]
    fn un_bloque_mas_grande_que_el_anillo_deja_solo_su_cola() {
        let mut a = Anillo::nuevo(10, 1.0);
        let entrada: Vec<f32> = (1..=1000).map(|i| i as f32).collect();
        a.escribir(&entrada);
        assert_eq!(a.ultimas(10), (991..=1000).map(|i| i as f32).collect::<Vec<_>>());
    }

    #[test]
    fn las_ultimas_salen_en_orden_aunque_haya_dado_la_vuelta() {
        let mut a = Anillo::nuevo(10, 1.0);
        a.escribir(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
        a.escribir(&[8.0, 9.0, 10.0, 11.0, 12.0]); // aquí da la vuelta
        assert_eq!(a.ultimas(3), vec![10.0, 11.0, 12.0]);
        assert_eq!(a.ultimas(100).len(), 10, "pedir de más devuelve lo que hay, no rellena");
    }

    #[test]
    fn al_principio_solo_sale_lo_escrito() {
        let a_vacio = Anillo::nuevo(10, 1.0);
        assert!(a_vacio.ultimas(5).is_empty());
        let mut a = Anillo::nuevo(10, 1.0);
        a.escribir(&[1.0, 2.0]);
        assert_eq!(a.ultimas(5), vec![1.0, 2.0]);
        assert_eq!(a.bytes(), 8);
    }

    /// **El gate del kill-switch.** No basta con que `ultimas()` devuelva vacío: eso lo cumple
    /// también mover un índice. Lo que se comprueba es que **la memoria ya no tiene la voz**.
    ///
    /// Se ve en rojo cambiando `vaciar()` por `self.cursor = 0; self.escritas = 0;` — el resto
    /// de los tests de este módulo siguen verdes, y este cae nombrando la muestra superviviente.
    #[test]
    fn vaciar_sobrescribe_la_memoria_no_solo_el_cursor() {
        let mut a = Anillo::nuevo(10, 1.0);
        a.escribir(&[0.9; 10]);
        a.vaciar();
        assert_eq!(a.ocupadas(), 0);
        assert_eq!(a.bytes(), 0);
        assert!(a.ultimas(10).is_empty());
        // La prueba de verdad: ni una sola muestra sobrevive en el respaldo.
        let superviviente = a.muestras.iter().position(|m| *m != 0.0);
        assert!(
            superviviente.is_none(),
            "la muestra {} sigue en memoria después de vaciar: el kill-switch no vacía, disimula",
            superviviente.unwrap()
        );
    }

    #[test]
    fn despues_de_vaciar_se_puede_seguir_grabando() {
        let mut a = Anillo::nuevo(10, 1.0);
        a.escribir(&[1.0; 10]);
        a.vaciar();
        a.escribir(&[2.0, 3.0]);
        assert_eq!(a.ultimas(5), vec![2.0, 3.0]);
    }

    #[test]
    fn se_puede_pedir_un_trozo_por_su_sitio_en_el_tiempo() {
        let mut a = Anillo::nuevo(10, 1.0);
        a.escribir(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(a.totales(), 5);
        assert_eq!(a.rango(1, 4), Some(vec![2.0, 3.0, 4.0]));
        assert_eq!(a.rango(0, 5), Some(vec![1.0, 2.0, 3.0, 4.0, 5.0]));
        assert_eq!(a.rango(3, 3), Some(vec![]), "un trozo vacío es válido, no un error");
    }

    #[test]
    fn un_trozo_que_pide_el_futuro_no_existe() {
        let mut a = Anillo::nuevo(10, 1.0);
        a.escribir(&[1.0, 2.0, 3.0]);
        assert_eq!(a.rango(2, 9), None);
        assert_eq!(a.rango(5, 2), None, "un rango del revés tampoco");
    }

    /// **El gate del rango.** Un turno cuyo audio ya se pisó tiene que devolver `None`, no un
    /// trozo recortado: una frase a medias se transcribe igual de bien y nadie sabe que falta el
    /// principio.
    ///
    /// Se ve en rojo quitando la comprobación `desde < mas_viejo`.
    #[test]
    fn un_trozo_que_ya_se_piso_se_declara_perdido() {
        let mut a = Anillo::nuevo(10, 1.0);
        a.escribir(&(1..=25).map(|i| i as f32).collect::<Vec<_>>());
        assert_eq!(a.totales(), 25);
        // Solo sobreviven las muestras 15..25.
        assert_eq!(a.rango(15, 20), Some(vec![16.0, 17.0, 18.0, 19.0, 20.0]));
        assert_eq!(a.rango(3, 8), None, "devolvió audio de un trozo que ya se había pisado");
        assert_eq!(a.rango(14, 20), None, "el borde también cuenta: 14 ya no está");
    }

    #[test]
    fn el_trozo_sobrevive_a_la_vuelta_del_anillo() {
        let mut a = Anillo::nuevo(10, 1.0);
        a.escribir(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
        a.escribir(&[8.0, 9.0, 10.0, 11.0, 12.0]); // da la vuelta
        assert_eq!(a.rango(6, 10), Some(vec![7.0, 8.0, 9.0, 10.0]));
    }

    #[test]
    fn vaciar_tambien_pone_el_reloj_a_cero() {
        let mut a = Anillo::nuevo(10, 1.0);
        a.escribir(&[1.0; 8]);
        a.vaciar();
        assert_eq!(a.totales(), 0);
        assert_eq!(a.rango(0, 1), None);
    }

    #[test]
    fn los_segundos_salen_de_la_frecuencia_declarada() {
        let mut a = Anillo::nuevo(16_000, 30.0);
        assert_eq!(a.capacidad(), 480_000);
        a.escribir(&vec![0.1; 8_000]);
        assert!((a.segundos() - 0.5).abs() < 1e-6);
        assert_eq!(a.ultimos_segundos(0.25).len(), 4_000);
    }
}
