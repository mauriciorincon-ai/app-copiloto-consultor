//! LA HUELLA DE UN CUADRO — ¿cambió la pantalla, o solo se movió el cursor?
//!
//! Leer una pantalla con Vision cuesta cientos de milisegundos de CPU en un Mac que además está
//! transcribiendo dos pistas. Leerla dos veces por segundo sería quemar el portátil del consultor
//! para leer cien veces la misma diapositiva. La huella decide **si hay algo nuevo que leer**, y
//! cuesta un par de milisegundos.
//!
//! **Por qué pHash y no comparar píxeles.** Una videollamada nunca entrega dos cuadros idénticos:
//! la compresión de vídeo mueve el ruido de un cuadro a otro, el cursor se pasea y un contador de
//! tiempo cambia cada segundo. Comparar píxeles diría «cambió» siempre. El pHash reduce el cuadro a
//! 32×32 grises, se queda con las **frecuencias bajas** de su transformada del coseno —la
//! composición, no el detalle— y las resume en 64 bits. Un cursor no mueve las frecuencias bajas;
//! una diapositiva nueva, sí.
//!
//! Todo aquí es aritmética sobre un búfer que ya está en memoria: ni disco, ni red, ni reloj.

use super::Cuadro;

/// El lado de la miniatura sobre la que se calcula la transformada.
const LADO: usize = 32;

/// Cuántas frecuencias bajas se miran por eje: 8 × 8 = los 64 bits de la huella.
const BAJAS: usize = 8;

/// Cuántas zonas por lado tiene la cuadrícula de la huella: 4 × 4 = 16 zonas.
pub const ZONAS: usize = 4;

/// Cuántos niveles de gris tiene que moverse una celda de la miniatura para contar como cambiada.
/// Por debajo, es el grano del vídeo, que al promediar cien píxeles por celda se queda en dos o tres.
pub const UMBRAL_GRIS: u8 = 12;

/// **LA HUELLA POR ZONAS: una miniatura de 32 × 32 grises por cada zona de una cuadrícula de 4 × 4.**
///
/// Llegó a esta forma equivocándose dos veces, las dos medidas —y se deja escrito porque las dos
/// primeras eran lo que dice el libro—:
///
/// 1. **Un pHash de la ventana entera** (lo que pedía el plan). El kit de pantalla lo tumbó en su
///    primera corrida: dos diapositivas distintas dentro de la misma ventana de Meet **solo diferían
///    en 8 bits de 64**, por debajo del umbral de cambio. El pHash mira la composición, y la de una
///    videollamada es casi toda interfaz —la barra, los recuadros, el rectángulo blanco—; el texto
///    que cambia es detalle, justo lo que el pHash tira.
/// 2. **Un pHash por zona.** Veía el texto nuevo, y su propio test lo tumbó por el otro lado: un
///    cursor en una zona casi lisa cambiaba **34 bits de 64**. El pHash de una zona sin contenido
///    compara coeficientes que son casi cero contra una mediana que también lo es: cualquier cosa lo
///    voltea.
///
/// Lo que funciona es lo más simple: **contar cuántas celdas de la miniatura cambiaron de verdad**
/// ([`UMBRAL_GRIS`]). Un cursor mueve unas pocas; el grano del vídeo, ninguna; una línea de texto
/// nueva, decenas. Y la cuadrícula da lo que un solo número no puede: saber **dónde** cambió, que es
/// lo que le permite al vigía ignorar las zonas que no paran —el vídeo de los participantes— sin
/// ignorar la diapositiva.
///
/// Pesa 16 KB, y el vigía guarda dos: son **miniaturas de la pantalla del cliente** a 32 × 32, que
/// no se pueden leer pero son suyas, y por eso viven en memoria y se olvidan con el vigía.
#[derive(Clone, PartialEq, Eq)]
pub struct Huella(Box<[[u8; LADO * LADO]; ZONAS * ZONAS]>);

impl std::fmt::Debug for Huella {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Ni en un mensaje de error: son miniaturas de la pantalla de un tercero.
        write!(f, "Huella(16 zonas)")
    }
}

impl Default for Huella {
    fn default() -> Self {
        Huella(Box::new([[0u8; LADO * LADO]; ZONAS * ZONAS]))
    }
}

impl Huella {
    pub fn de(cuadro: &Cuadro) -> Self {
        let mut h = Huella::default();
        if cuadro.ancho < ZONAS
            || cuadro.alto < ZONAS
            || cuadro.gris.len() < cuadro.ancho * cuadro.alto
        {
            return h;
        }
        for zy in 0..ZONAS {
            for zx in 0..ZONAS {
                let (x0, x1) = (zx * cuadro.ancho / ZONAS, (zx + 1) * cuadro.ancho / ZONAS);
                let (y0, y1) = (zy * cuadro.alto / ZONAS, (zy + 1) * cuadro.alto / ZONAS);
                let mini = reducir(cuadro, x0, y0, x1 - x0, y1 - y0);
                for (d, o) in h.0[zy * ZONAS + zx].iter_mut().zip(mini) {
                    *d = o.round().clamp(0.0, 255.0) as u8;
                }
            }
        }
        h
    }

    /// Cuántas celdas cambiaron, zona por zona.
    pub fn distancias(&self, otra: &Huella) -> [u32; ZONAS * ZONAS] {
        let mut d = [0u32; ZONAS * ZONAS];
        for (i, di) in d.iter_mut().enumerate() {
            *di = self.0[i]
                .iter()
                .zip(otra.0[i].iter())
                .filter(|(a, b)| a.abs_diff(**b) > UMBRAL_GRIS)
                .count() as u32;
        }
        d
    }

    /// La zona que más cambió. Para los mensajes y el kit.
    pub fn mayor_distancia(&self, otra: &Huella) -> u32 {
        self.distancias(otra).into_iter().max().unwrap_or(0)
    }

    /// Pisa las miniaturas. Son de la pantalla del cliente.
    pub fn pisar(&mut self) {
        for z in self.0.iter_mut() {
            z.fill(0);
        }
    }
}

/// El pHash de 64 bits de un cuadro entero.
pub fn phash(cuadro: &Cuadro) -> u64 {
    phash_de(cuadro, 0, 0, cuadro.ancho, cuadro.alto)
}

/// El pHash de 64 bits de un rectángulo del cuadro. Uno vacío tiene la huella 0, y no es un error:
/// es «no hay nada que comparar», que el vigía trata como cualquier otro.
pub fn phash_de(cuadro: &Cuadro, x0: usize, y0: usize, ancho: usize, alto: usize) -> u64 {
    if ancho == 0
        || alto == 0
        || cuadro.gris.len() < cuadro.ancho * cuadro.alto
        || x0 + ancho > cuadro.ancho
        || y0 + alto > cuadro.alto
    {
        return 0;
    }
    let mini = reducir(cuadro, x0, y0, ancho, alto);

    // La transformada del coseno es separable: primero las filas, luego las columnas. Solo hacen
    // falta las ocho primeras frecuencias de cada eje, así que se calculan esas y ninguna más.
    let cosenos = tabla_de_cosenos();
    let mut filas = [[0f32; BAJAS]; LADO];
    for (y, fila) in filas.iter_mut().enumerate() {
        for (u, salida) in fila.iter_mut().enumerate() {
            *salida = (0..LADO).map(|x| cosenos[u][x] * mini[y * LADO + x]).sum();
        }
    }
    let mut bajas = [0f32; BAJAS * BAJAS];
    for v in 0..BAJAS {
        for u in 0..BAJAS {
            bajas[v * BAJAS + u] = (0..LADO).map(|y| cosenos[v][y] * filas[y][u]).sum();
        }
    }

    // La componente continua —el brillo medio— no dice nada de la composición y desplazaría la
    // mediana: se deja fuera del cálculo del umbral, como en el pHash clásico.
    let mut orden: Vec<f32> = bajas[1..].to_vec();
    orden.sort_by(|a, b| a.total_cmp(b));
    let mediana = orden[orden.len() / 2];

    bajas.iter().enumerate().fold(
        0u64,
        |h, (i, &c)| if c > mediana { h | (1 << i) } else { h },
    )
}

/// Cuántos de los 64 bits difieren entre dos huellas.
pub fn distancia(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

/// Un rectángulo del cuadro, reducido a 32×32 grises **promediando por áreas**. Muestrear un píxel
/// de cada bloque sería más barato y mucho peor: un texto fino cae entre dos muestras y desaparece.
fn reducir(cuadro: &Cuadro, x0: usize, y0: usize, ancho: usize, alto: usize) -> [f32; LADO * LADO] {
    let mut mini = [0f32; LADO * LADO];
    for my in 0..LADO {
        let (ya, yb) = (
            my * alto / LADO,
            ((my + 1) * alto / LADO).max(my * alto / LADO + 1),
        );
        for mx in 0..LADO {
            let (xa, xb) = (
                mx * ancho / LADO,
                ((mx + 1) * ancho / LADO).max(mx * ancho / LADO + 1),
            );
            let mut suma = 0u64;
            let mut n = 0u64;
            for y in (y0 + ya)..(y0 + yb).min(y0 + alto) {
                let fila = &cuadro.gris[y * cuadro.ancho..(y + 1) * cuadro.ancho];
                for &p in &fila[(x0 + xa)..(x0 + xb).min(x0 + ancho)] {
                    suma += p as u64;
                    n += 1;
                }
            }
            mini[my * LADO + mx] = if n == 0 { 0.0 } else { suma as f32 / n as f32 };
        }
    }
    mini
}

fn tabla_de_cosenos() -> [[f32; LADO]; BAJAS] {
    let mut t = [[0f32; LADO]; BAJAS];
    for (u, fila) in t.iter_mut().enumerate() {
        for (x, c) in fila.iter_mut().enumerate() {
            *c = (((2 * x + 1) as f32 * u as f32 * std::f32::consts::PI) / (2 * LADO) as f32).cos();
        }
    }
    t
}

#[cfg(test)]
pub(crate) mod pruebas {
    use super::*;

    /// Una «diapositiva» sintética: fondo claro, una franja de título y unas barras. Sin texto de
    /// verdad —eso lo prueba el kit con imágenes—, pero con composición, que es lo que la huella mira.
    pub(crate) fn diapositiva(barras: &[usize], ancho: usize, alto: usize) -> Cuadro {
        let mut gris = vec![235u8; ancho * alto];
        // el título
        for y in alto / 10..alto / 10 + alto / 14 {
            for x in ancho / 10..ancho * 7 / 10 {
                gris[y * ancho + x] = 30;
            }
        }
        // las barras
        let base = alto * 9 / 10;
        for (i, &h) in barras.iter().enumerate() {
            let x0 = ancho / 8 + i * ancho / 8;
            for y in base.saturating_sub(h * alto / 100)..base {
                for x in x0..(x0 + ancho / 14).min(ancho) {
                    gris[y * ancho + x] = 60;
                }
            }
        }
        Cuadro { ancho, alto, gris }
    }

    /// El cursor: una flecha oscura de 12×18 px en cualquier sitio.
    pub(crate) fn con_cursor(mut c: Cuadro, x: usize, y: usize) -> Cuadro {
        for dy in 0..18 {
            for dx in 0..(dy * 12 / 18).max(1) {
                if let Some(p) = c.gris.get_mut((y + dy) * c.ancho + x + dx) {
                    *p = 0;
                }
            }
        }
        c
    }

    #[test]
    fn el_mismo_cuadro_da_la_misma_huella() {
        let a = diapositiva(&[40, 70, 55], 1280, 720);
        let b = diapositiva(&[40, 70, 55], 1280, 720);
        assert_eq!(Huella::de(&a), Huella::de(&b));
        assert_ne!(
            phash(&a),
            0,
            "una diapositiva con contenido no puede tener la huella vacía"
        );
    }

    #[test]
    fn un_cursor_no_cambia_la_pantalla() {
        let a = diapositiva(&[40, 70, 55], 1280, 720);
        let b = con_cursor(diapositiva(&[40, 70, 55], 1280, 720), 900, 400);
        let d = Huella::de(&a).mayor_distancia(&Huella::de(&b));
        assert!(
            d <= super::super::QUIETO,
            "un cursor movió {d} bits de 64 en su zona"
        );
    }

    #[test]
    fn otra_diapositiva_si_cambia_la_pantalla() {
        let a = diapositiva(&[40, 70, 55], 1280, 720);
        let b = diapositiva(&[85, 20, 60, 35, 90], 1280, 720);
        let d = Huella::de(&a).mayor_distancia(&Huella::de(&b));
        assert!(
            d > super::super::CAMBIO,
            "dos diapositivas distintas solo difieren en {d} bits"
        );
    }

    #[test]
    fn el_ruido_de_la_compresion_no_cambia_la_pantalla() {
        let a = diapositiva(&[40, 70, 55], 1280, 720);
        let mut b = diapositiva(&[40, 70, 55], 1280, 720);
        // ±6 niveles de gris sobre cada píxel, determinista: es el grano que deja un códec de vídeo.
        for (i, p) in b.gris.iter_mut().enumerate() {
            let r = ((i as u32).wrapping_mul(2_654_435_761) >> 28) as i16 - 8;
            *p = (*p as i16 + r.clamp(-6, 6)).clamp(0, 255) as u8;
        }
        let d = Huella::de(&a).mayor_distancia(&Huella::de(&b));
        assert!(
            d <= super::super::QUIETO,
            "el grano de vídeo movió {d} bits"
        );
    }

    #[test]
    #[ignore = "medición, no prueba: se corre a mano para fijar los umbrales"]
    fn medir_los_umbrales() {
        let a = diapositiva(&[40, 70, 55], 1600, 1000);
        for (x, y) in [(900, 400), (10, 10), (395, 245), (1200, 800)] {
            let c = con_cursor(diapositiva(&[40, 70, 55], 1600, 1000), x, y);
            println!(
                "cursor en ({x},{y}): {} celdas",
                Huella::de(&a).mayor_distancia(&Huella::de(&c))
            );
        }
        let mut b = diapositiva(&[40, 70, 55], 1600, 1000);
        for (i, p) in b.gris.iter_mut().enumerate() {
            let r = ((i as u32).wrapping_mul(2_654_435_761) >> 28) as i16 - 8;
            *p = (*p as i16 + r).clamp(0, 255) as u8;
        }
        println!(
            "grano ±8: {} celdas",
            Huella::de(&a).mayor_distancia(&Huella::de(&b))
        );
        let c = diapositiva(&[85, 20, 60, 35, 90], 1600, 1000);
        println!(
            "otra diapositiva: {} celdas",
            Huella::de(&a).mayor_distancia(&Huella::de(&c))
        );
    }

    #[test]
    fn un_cuadro_vacio_o_roto_no_revienta() {
        assert_eq!(
            phash(&Cuadro {
                ancho: 0,
                alto: 0,
                gris: vec![]
            }),
            0
        );
        assert_eq!(
            phash(&Cuadro {
                ancho: 100,
                alto: 100,
                gris: vec![0; 10]
            }),
            0
        );
        assert_eq!(
            Huella::de(&Cuadro {
                ancho: 3,
                alto: 3,
                gris: vec![0; 9]
            }),
            Huella::default()
        );
        // Más pequeño que la miniatura: se amplía por bloques y sigue dando una huella.
        let _ = Huella::de(&diapositiva(&[50], 20, 12));
    }
}
