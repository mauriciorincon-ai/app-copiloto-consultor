//! **Todo el `unsafe` de la captura vive aquí**, igual que `acople/ax.rs` concentra el del acople.
//!
//! Abre dos grifos de audio de macOS y los deja cayendo dentro de un [`Anillo`]:
//!
//! - **el micrófono** — el dispositivo de entrada por defecto, que es el consultor;
//! - **el audio del sistema** — un *process tap* de Core Audio (macOS 14.2+), que es todo lo que
//!   suena en el Mac menos nosotros mismos, y por tanto la contraparte de la videollamada.
//!
//! **Por qué no hay `cpal`.** El plan del sprint nombraba esa biblioteca para el micrófono, y para
//! el micrófono solo habría estado bien. Pero el audio del sistema no está en ninguna biblioteca:
//! el tap hay que escribirlo contra Core Audio a mano pase lo que pase. Una vez escrito el camino
//! difícil, usar otro distinto para el fácil añade una dependencia, un segundo modelo mental y un
//! segundo sitio donde puede fallar la conversión a mono — sin ahorrar nada. Los dos grifos son
//! el mismo código con un dispositivo distinto. Queda anotado como decisión técnica en la bitácora.
//!
//! **Y lo que el spike descubrió corriendo, que manda sobre todo lo demás:** cuando no suena nada,
//! el callback del tap **no se llama**. No llegan ceros, no llega nada. Así que un contador de
//! muestras en cero no distingue «el cliente está callado» de «el tap se rompió». La diferencia
//! que sí se puede afirmar es otra —si el grifo se abrió o no—, y es la que la app enseña.

use super::anillo::Anillo;
use super::remuestreo::Remuestreador;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};

/// Un identificador de objeto del sistema de audio: un dispositivo, un proceso, un tap.
pub type ObjetoDeAudio = u32;
type Estado = i32;

const OK: Estado = 0;

/// Los selectores de Core Audio son cuatro letras empaquetadas en 32 bits. Se escriben aquí tal y
/// como aparecen en las cabeceras del sistema para que se puedan buscar.
const fn cuatro(s: &[u8; 4]) -> u32 {
    ((s[0] as u32) << 24) | ((s[1] as u32) << 16) | ((s[2] as u32) << 8) | (s[3] as u32)
}

const OBJETO_SISTEMA: ObjetoDeAudio = 1;
const AMBITO_GLOBAL: u32 = cuatro(b"glob");
const AMBITO_ENTRADA: u32 = cuatro(b"inpt");
const ELEMENTO_PRINCIPAL: u32 = 0;

const ENTRADA_POR_DEFECTO: u32 = cuatro(b"dIn ");
const SALIDA_POR_DEFECTO: u32 = cuatro(b"dOut");
const PID_A_PROCESO: u32 = cuatro(b"id2p");
const UID_DEL_DISPOSITIVO: u32 = cuatro(b"uid ");
/// `kAudioDevicePropertyStreamFormat`. La cabecera lo marca como obsoleto en favor de preguntar al
/// *stream*, pero sigue siendo la vía directa para saber a qué frecuencia entrega un dispositivo,
/// y es una lectura, no una escritura.
const FORMATO_DEL_DISPOSITIVO: u32 = cuatro(b"sfmt");
const FORMATO_DEL_TAP: u32 = cuatro(b"tfmt");
/// `kAudioFormatLinearPCM` y las dos banderas que hacen legible un búfer como `f32`:
/// `kAudioFormatFlagIsFloat` (1 << 0) y `kAudioFormatFlagIsPacked` (1 << 3). Se comprueban al
/// ABRIR, no en el callback: el callback corre en un hilo de tiempo real y ahí ya es tarde para
/// negociar nada — lo único que puede hacer es devolver sin tocar el búfer.
const PCM_LINEAL: u32 = cuatro(b"lpcm");
const ES_FLOTANTE: u32 = 1 << 0;
const ES_EMPAQUETADO: u32 = 1 << 3;
const TIPO_DE_TRANSPORTE: u32 = cuatro(b"tran");
const FUENTE_DE_DATOS: u32 = cuatro(b"ssrc");
const AMBITO_SALIDA: u32 = cuatro(b"outp");
const TRANSPORTE_INTERNO: u32 = cuatro(b"bltn");
/// El altavoz interno del Mac. No está en ninguna cabecera pública; se comprobó midiendo en este
/// Mac («bltn» + «ispk» + «MacBook Air Speakers») y es el valor que macOS lleva usando años.
const ALTAVOZ_INTERNO: u32 = cuatro(b"ispk");

#[repr(C)]
#[derive(Clone, Copy)]
struct Direccion {
    selector: u32,
    ambito: u32,
    elemento: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Bufer {
    canales: u32,
    bytes: u32,
    datos: *mut c_void,
}

#[repr(C)]
struct ListaDeBuferes {
    cuantos: u32,
    primero: [Bufer; 1],
}

/// `AudioStreamBasicDescription`. Solo se leen tres campos, pero la estructura entera tiene que
/// estar declarada o el sistema escribiría fuera.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
struct Formato {
    hz: f64,
    id: u32,
    banderas: u32,
    bytes_por_paquete: u32,
    marcos_por_paquete: u32,
    bytes_por_marco: u32,
    canales: u32,
    bits: u32,
    reservado: u32,
}

type ProcDeAudio = unsafe extern "C" fn(
    dispositivo: ObjetoDeAudio,
    ahora: *const c_void,
    entrada: *const ListaDeBuferes,
    hora_de_entrada: *const c_void,
    salida: *mut ListaDeBuferes,
    hora_de_salida: *const c_void,
    cliente: *mut c_void,
) -> Estado;

#[link(name = "CoreAudio", kind = "framework")]
extern "C" {
    fn AudioObjectGetPropertyData(
        objeto: ObjetoDeAudio,
        direccion: *const Direccion,
        tam_entrada: u32,
        entrada: *const c_void,
        tam: *mut u32,
        datos: *mut c_void,
    ) -> Estado;
    fn AudioHardwareCreateProcessTap(descripcion: *mut c_void, tap: *mut ObjetoDeAudio) -> Estado;
    fn AudioHardwareDestroyProcessTap(tap: ObjetoDeAudio) -> Estado;
    fn AudioHardwareCreateAggregateDevice(
        descripcion: *const c_void,
        dispositivo: *mut ObjetoDeAudio,
    ) -> Estado;
    fn AudioHardwareDestroyAggregateDevice(dispositivo: ObjetoDeAudio) -> Estado;
    fn AudioDeviceCreateIOProcID(
        dispositivo: ObjetoDeAudio,
        proc_: ProcDeAudio,
        cliente: *mut c_void,
        id: *mut *mut c_void,
    ) -> Estado;
    fn AudioDeviceDestroyIOProcID(dispositivo: ObjetoDeAudio, id: *mut c_void) -> Estado;
    fn AudioDeviceStart(dispositivo: ObjetoDeAudio, id: *mut c_void) -> Estado;
    fn AudioDeviceStop(dispositivo: ObjetoDeAudio, id: *mut c_void) -> Estado;
}

// ---------------------------------------------------------------------------------------------
// La parte sin `unsafe`: mezclar a mono. Es pura y se prueba sola.
// ---------------------------------------------------------------------------------------------

/// Deja en mono lo que Core Audio entrega, que puede venir de dos formas distintas.
///
/// Un dispositivo estéreo puede dar **un bloque con las muestras entrelazadas** (izquierda,
/// derecha, izquierda, derecha…) o **un bloque por canal**. Las dos formas son normales y la
/// bandera que las distingue vive en el formato, pero la lista de búferes ya lo dice sola: si hay
/// más de un bloque, hay un bloque por canal.
///
/// Se promedian los canales en vez de quedarse con el izquierdo. Un consultor con auriculares USB
/// mal balanceados puede tener casi toda su voz en un canal; quedarse con el otro sería no oírle.
pub fn a_mono(bloques: &[&[f32]], canales: u32) -> Vec<f32> {
    match bloques {
        [] => Vec::new(),
        [uno] if canales <= 1 => uno.to_vec(),
        [uno] => uno
            .chunks(canales as usize)
            .map(|m| m.iter().sum::<f32>() / m.len() as f32)
            .collect(),
        varios => {
            let largo = varios.iter().map(|b| b.len()).min().unwrap_or(0);
            (0..largo)
                .map(|i| varios.iter().map(|b| b[i]).sum::<f32>() / varios.len() as f32)
                .collect()
        }
    }
}

/// Por dónde sale el sonido del Mac — y por tanto, **si el micrófono va a oír al cliente**.
///
/// Esto no estaba en el plan y salió de correr la prueba de punta a punta: sonó la pregunta del
/// cliente por los altavoces y el turno apareció en las DOS pistas, porque el micrófono del
/// portátil oye a sus propios altavoces. La app promete que el micrófono es el consultor y el
/// sistema es el cliente; con altavoces esa promesa es falsa, y callarlo sería atribuirle al
/// consultor palabras que no dijo.
///
/// La maqueta ya lo había previsto —la pantalla de Sesión tiene una fila «Auriculares
/// conectados»—; lo que faltaba era comprobarlo de verdad en vez de suponerlo.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case", tag = "salida")]
pub enum Salida {
    /// Altavoces internos: **el micrófono va a oír al cliente**.
    Altavoces,
    /// Auriculares por el conector del Mac: las dos pistas quedan limpias.
    Auriculares,
    /// Un dispositivo externo (USB, Bluetooth, una interfaz). Puede ser un casco o un altavoz de
    /// mesa, y desde aquí **no se puede distinguir**: se dice el nombre y decide el usuario.
    Otra { nombre: String },
    NoSeSabe { motivo: String },
}

impl Salida {
    /// ¿Hay que avisar de que las pistas pueden mezclarse? `None` cuando no se sabe.
    pub fn puede_haber_eco(&self) -> Option<bool> {
        match self {
            Salida::Altavoces => Some(true),
            Salida::Auriculares => Some(false),
            Salida::Otra { .. } | Salida::NoSeSabe { .. } => None,
        }
    }
}

/// Mira por dónde sale hoy el sonido.
pub fn salida_de_audio() -> Salida {
    let Some(dispositivo) = leer_objeto(OBJETO_SISTEMA, SALIDA_POR_DEFECTO, AMBITO_GLOBAL) else {
        return Salida::NoSeSabe { motivo: "este Mac no declara salida de audio por defecto".into() };
    };
    let nombre = leer_cadena(dispositivo, cuatro(b"lnam")).unwrap_or_else(|| "sin nombre".into());
    let Some(transporte) = leer_u32(dispositivo, TIPO_DE_TRANSPORTE, AMBITO_GLOBAL) else {
        return Salida::NoSeSabe { motivo: format!("«{nombre}» no dice cómo está conectado") };
    };
    if transporte != TRANSPORTE_INTERNO {
        return Salida::Otra { nombre };
    }
    match leer_u32(dispositivo, FUENTE_DE_DATOS, AMBITO_SALIDA) {
        Some(f) if f == ALTAVOZ_INTERNO => Salida::Altavoces,
        Some(_) => Salida::Auriculares,
        // Conectado por dentro pero sin decir a qué: lo honesto es no elegir por el usuario.
        None => Salida::NoSeSabe { motivo: format!("«{nombre}» no dice por dónde suena") },
    }
}

// ---------------------------------------------------------------------------------------------
// El grifo
// ---------------------------------------------------------------------------------------------

/// Lo que el callback del audio necesita tener a mano. Vive en un `Box` que es propiedad del
/// [`Grifo`]; el sistema solo tiene un puntero prestado, y el `Drop` del grifo para el callback
/// **antes** de soltarlo.
struct Destino {
    anillo: Arc<Mutex<Anillo>>,
    remuestreador: Remuestreador,
    /// Cuántas muestras han entrado. Es un número, no audio: sirve para el log y para la pantalla.
    entradas: Arc<std::sync::atomic::AtomicU64>,
}

/// Un grifo de audio abierto. Se cierra solo al soltarlo.
pub struct Grifo {
    dispositivo: ObjetoDeAudio,
    proc_id: *mut c_void,
    /// Se conserva para poder soltarlo DESPUÉS de parar el callback. El compilador no puede
    /// saberlo; el orden del `Drop` sí.
    _destino: Box<Destino>,
    /// Solo el grifo del sistema los tiene.
    tap: Option<ObjetoDeAudio>,
    agregado: Option<ObjetoDeAudio>,
    entradas: Arc<std::sync::atomic::AtomicU64>,
    pub hz_del_dispositivo: u32,
}

/// El puntero de `proc_id` es un identificador opaco del sistema, no una referencia a memoria
/// nuestra, y el `Box` del destino solo lo toca el hilo de audio mientras el grifo está vivo. El
/// grifo se mueve entre hilos (nace en el hilo que abre la sesión y muere en el que la cierra) y
/// eso es seguro; lo que no sería seguro es compartirlo, y por eso no se marca `Sync`.
unsafe impl Send for Grifo {}

impl Grifo {
    /// Cuántas muestras han entrado por este grifo desde que se abrió.
    ///
    /// **Cero no significa avería.** Si nadie habla, el sistema no llama al callback ni una vez.
    /// Lo que significa avería es que el grifo no se haya abierto, y eso se sabe porque
    /// [`Grifo::del_microfono`] o [`Grifo::del_sistema`] devuelven el error en ese momento.
    pub fn muestras_recibidas(&self) -> u64 {
        self.entradas.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Abre el micrófono: el dispositivo de entrada por defecto del Mac.
    pub fn del_microfono(anillo: Arc<Mutex<Anillo>>) -> Result<Self, String> {
        let dispositivo = leer_objeto(OBJETO_SISTEMA, ENTRADA_POR_DEFECTO, AMBITO_GLOBAL)
            .ok_or("este Mac no tiene un dispositivo de entrada por defecto")?;
        let formato = leer_formato(dispositivo, FORMATO_DEL_DISPOSITIVO, AMBITO_ENTRADA)
            .ok_or("no se pudo leer el formato del micrófono")?;
        Self::abrir(dispositivo, formato, anillo, None, None)
    }

    /// Abre el audio del sistema con un *process tap* global que **se excluye a sí mismo**.
    ///
    /// Excluirnos no es cortesía: en cuanto exista el modo solo audio (C15), la app hablará por los
    /// altavoces, y un tap que se oyera a sí mismo transcribiría su propia voz como si fuera el
    /// cliente.
    pub fn del_sistema(anillo: Arc<Mutex<Anillo>>) -> Result<Self, String> {
        let descripcion = describir_el_tap()?;
        let mut tap: ObjetoDeAudio = 0;
        let estado =
            unsafe { AudioHardwareCreateProcessTap(descripcion.0, &mut tap) };
        if estado != OK || tap == 0 {
            return Err(formato_de_error("no se pudo crear el tap del audio del sistema", estado));
        }
        let formato = leer_formato(tap, FORMATO_DEL_TAP, AMBITO_GLOBAL).ok_or_else(|| {
            unsafe { AudioHardwareDestroyProcessTap(tap) };
            "no se pudo leer el formato del tap".to_string()
        })?;

        let agregado = match crear_agregado(&descripcion.1) {
            Ok(a) => a,
            Err(e) => {
                unsafe { AudioHardwareDestroyProcessTap(tap) };
                return Err(e);
            }
        };

        match Self::abrir(agregado, formato, anillo, Some(tap), Some(agregado)) {
            Ok(g) => Ok(g),
            Err(e) => {
                unsafe {
                    AudioHardwareDestroyAggregateDevice(agregado);
                    AudioHardwareDestroyProcessTap(tap);
                }
                Err(e)
            }
        }
    }

    fn abrir(
        dispositivo: ObjetoDeAudio,
        formato: Formato,
        anillo: Arc<Mutex<Anillo>>,
        tap: Option<ObjetoDeAudio>,
        agregado: Option<ObjetoDeAudio>,
    ) -> Result<Self, String> {
        let hz = formato.hz.round() as u32;
        if hz == 0 {
            return Err("el dispositivo no declara frecuencia de muestreo".into());
        }
        if !es_float32_empaquetado(&formato) {
            // Nombrado, como todo error de este módulo: quien lea el log tiene que poder decidir
            // qué hacer sin abrir el código.
            return Err(format!(
                "el audio no llega como flotante de 32 bits empaquetado y no se puede leer: \
                 formato {} · {} bits · banderas {:#x}",
                cuatro_letras(formato.id),
                formato.bits,
                formato.banderas
            ));
        }
        let entradas = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let destino = Box::new(Destino {
            anillo,
            remuestreador: Remuestreador::nuevo(hz, super::anillo::HZ),
            entradas: entradas.clone(),
        });
        let puntero = Box::into_raw(destino);
        let mut proc_id: *mut c_void = std::ptr::null_mut();
        let estado = unsafe {
            AudioDeviceCreateIOProcID(dispositivo, recibir, puntero as *mut c_void, &mut proc_id)
        };
        // Recuperar la propiedad del `Box` pase lo que pase: si el sistema no aceptó el callback,
        // dejar el puntero suelto sería una fuga de memoria por cada intento fallido.
        let destino = unsafe { Box::from_raw(puntero) };
        if estado != OK || proc_id.is_null() {
            return Err(formato_de_error("el sistema no aceptó el lector de audio", estado));
        }
        let estado = unsafe { AudioDeviceStart(dispositivo, proc_id) };
        if estado != OK {
            unsafe { AudioDeviceDestroyIOProcID(dispositivo, proc_id) };
            return Err(formato_de_error("el sistema no dejó arrancar la captura", estado));
        }
        Ok(Self {
            dispositivo,
            proc_id,
            _destino: destino,
            tap,
            agregado,
            entradas,
            hz_del_dispositivo: hz,
        })
    }
}

impl Drop for Grifo {
    fn drop(&mut self) {
        unsafe {
            AudioDeviceStop(self.dispositivo, self.proc_id);
            AudioDeviceDestroyIOProcID(self.dispositivo, self.proc_id);
            if let Some(a) = self.agregado {
                AudioHardwareDestroyAggregateDevice(a);
            }
            if let Some(t) = self.tap {
                AudioHardwareDestroyProcessTap(t);
            }
        }
    }
}

/// El callback del audio. Corre en un hilo de tiempo real del sistema: **aquí no se asigna
/// memoria si se puede evitar, no se imprime, y no se entra en pánico**. Un `panic` cruzando una
/// frontera de C es comportamiento indefinido, así que todo lo que puede fallar se resuelve
/// devolviendo sin hacer nada.
unsafe extern "C" fn recibir(
    _dispositivo: ObjetoDeAudio,
    _ahora: *const c_void,
    entrada: *const ListaDeBuferes,
    _hora_de_entrada: *const c_void,
    _salida: *mut ListaDeBuferes,
    _hora_de_salida: *const c_void,
    cliente: *mut c_void,
) -> Estado {
    if entrada.is_null() || cliente.is_null() {
        return OK;
    }
    let destino = &mut *(cliente as *mut Destino);
    let lista = &*entrada;
    let cuantos = lista.cuantos as usize;
    if cuantos == 0 {
        return OK;
    }
    let buferes = std::slice::from_raw_parts(lista.primero.as_ptr(), cuantos);

    let mut bloques: Vec<&[f32]> = Vec::with_capacity(cuantos);
    let mut canales = 1;
    for b in buferes {
        if b.datos.is_null() {
            continue;
        }
        // El formato ya se validó al abrir; esto cubre lo que el formato no dice: que ESTE búfer
        // traiga un número entero de muestras y empiece donde un `f32` puede empezar. Un
        // `from_raw_parts` desalineado es comportamiento indefinido, no un número raro.
        let tam = std::mem::size_of::<f32>();
        if b.bytes as usize % tam != 0 || (b.datos as usize) % std::mem::align_of::<f32>() != 0 {
            continue;
        }
        canales = b.canales.max(1);
        let n = b.bytes as usize / tam;
        bloques.push(std::slice::from_raw_parts(b.datos as *const f32, n));
    }
    if bloques.is_empty() {
        return OK;
    }

    let mono = a_mono(&bloques, canales);
    let a_dieciseis = destino.remuestreador.convertir(&mono);
    if a_dieciseis.is_empty() {
        return OK;
    }
    destino
        .entradas
        .fetch_add(a_dieciseis.len() as u64, std::sync::atomic::Ordering::Relaxed);
    // `lock()` en el hilo de audio no es lo ideal, y se acepta a sabiendas: la sección crítica es
    // copiar unos cientos de flotantes, el otro lado del candado solo lee en trozos igual de
    // cortos, y la alternativa —una cola sin candados— es bastante más código del que este sprint
    // puede justificar. Si alguna vez se oye un chasquido, aquí está la primera sospecha.
    if let Ok(mut a) = destino.anillo.lock() {
        a.escribir(&a_dieciseis);
    }
    OK
}

// ---------------------------------------------------------------------------------------------
// Lecturas y construcción del tap
// ---------------------------------------------------------------------------------------------

fn direccion(selector: u32, ambito: u32) -> Direccion {
    Direccion { selector, ambito, elemento: ELEMENTO_PRINCIPAL }
}

fn leer_objeto(objeto: ObjetoDeAudio, selector: u32, ambito: u32) -> Option<ObjetoDeAudio> {
    let d = direccion(selector, ambito);
    let mut valor: ObjetoDeAudio = 0;
    let mut tam = std::mem::size_of::<ObjetoDeAudio>() as u32;
    let estado = unsafe {
        AudioObjectGetPropertyData(
            objeto,
            &d,
            0,
            std::ptr::null(),
            &mut tam,
            &mut valor as *mut _ as *mut c_void,
        )
    };
    (estado == OK && valor != 0).then_some(valor)
}

fn leer_u32(objeto: ObjetoDeAudio, selector: u32, ambito: u32) -> Option<u32> {
    let d = direccion(selector, ambito);
    let mut valor: u32 = 0;
    let mut tam = std::mem::size_of::<u32>() as u32;
    let estado = unsafe {
        AudioObjectGetPropertyData(
            objeto,
            &d,
            0,
            std::ptr::null(),
            &mut tam,
            &mut valor as *mut _ as *mut c_void,
        )
    };
    (estado == OK).then_some(valor)
}

/// **¿Se puede leer este búfer como `f32`?**
///
/// El callback reinterpreta los bytes del sistema como flotantes de 32 bits. Hasta el sprint 002 lo
/// hacía a ciegas: si Core Audio hubiera negociado entero de 16 bits —cosa que puede hacer, y que
/// depende del dispositivo— cada muestra se habría leído como un número flotante formado por los
/// bytes de dos muestras enteras. No es un fallo ruidoso: es ruido, y el detector de voz lo habría
/// tomado por sonido. Por eso esto se responde **al abrir el grifo**, con un error nombrado, y no
/// dentro del hilo de tiempo real.
fn es_float32_empaquetado(f: &Formato) -> bool {
    f.id == PCM_LINEAL
        && f.bits == 32
        && f.banderas & ES_FLOTANTE != 0
        && f.banderas & ES_EMPAQUETADO != 0
}

fn leer_formato(objeto: ObjetoDeAudio, selector: u32, ambito: u32) -> Option<Formato> {
    let d = direccion(selector, ambito);
    let mut valor = Formato::default();
    let mut tam = std::mem::size_of::<Formato>() as u32;
    let estado = unsafe {
        AudioObjectGetPropertyData(
            objeto,
            &d,
            0,
            std::ptr::null(),
            &mut tam,
            &mut valor as *mut _ as *mut c_void,
        )
    };
    (estado == OK).then_some(valor)
}

/// Core Audio empaqueta cuatro letras en un entero —tanto sus errores como sus identificadores de
/// formato— y enseñarlas ahorra media hora a quien lea el log («!obj», «nope», «lpcm»).
fn cuatro_letras(valor: u32) -> String {
    valor
        .to_be_bytes()
        .iter()
        .map(|b| if b.is_ascii_graphic() { *b as char } else { '·' })
        .collect()
}

fn formato_de_error(que: &str, estado: Estado) -> String {
    format!("{que} (estado {estado} «{}»)", cuatro_letras(estado as u32))
}

/// Un `CATapDescription` vivo y el UID con el que referirse a él desde el dispositivo agregado.
struct DescripcionDelTap(*mut c_void, String, #[allow(dead_code)] objc2::rc::Retained<objc2::runtime::AnyObject>);

fn describir_el_tap() -> Result<DescripcionDelTap, String> {
    use objc2::rc::Retained;
    use objc2::runtime::{AnyClass, AnyObject};
    use objc2::msg_send;
    use objc2_foundation::{NSArray, NSNumber, NSString, NSUUID};

    let clase = AnyClass::get(c"CATapDescription")
        .ok_or("este macOS no trae los taps de audio (hacen falta 14.2 o más)")?;

    // Nuestro propio proceso, para excluirnos del tap.
    let mut nuestro: ObjetoDeAudio = 0;
    let mut pid = std::process::id() as i32;
    let d = direccion(PID_A_PROCESO, AMBITO_GLOBAL);
    let mut tam = std::mem::size_of::<ObjetoDeAudio>() as u32;
    unsafe {
        AudioObjectGetPropertyData(
            OBJETO_SISTEMA,
            &d,
            std::mem::size_of::<i32>() as u32,
            &mut pid as *mut _ as *const c_void,
            &mut tam,
            &mut nuestro as *mut _ as *mut c_void,
        );
    }
    let excluidos: Retained<NSArray<NSNumber>> = if nuestro == 0 {
        NSArray::new()
    } else {
        NSArray::from_retained_slice(&[NSNumber::new_u32(nuestro)])
    };

    let descripcion: Retained<AnyObject> = unsafe {
        let vacia: objc2::rc::Allocated<AnyObject> = msg_send![clase, alloc];
        msg_send![vacia, initMonoGlobalTapButExcludeProcesses: &*excluidos]
    };
    unsafe {
        let nombre = NSString::from_str("Angel Ghost · audio de la reunión");
        let _: () = msg_send![&*descripcion, setName: &*nombre];
        // Privado: el tap solo lo ve quien lo creó. Sin esto aparecería en la lista de
        // dispositivos de audio de todo el sistema, que es exactamente lo contrario de discreto.
        let _: () = msg_send![&*descripcion, setPrivate: true];
        // Sin silenciar: el cliente tiene que seguir oyéndose por los altavoces del consultor.
        let _: () = msg_send![&*descripcion, setMuteBehavior: 0isize];
    }
    let uid: String = unsafe {
        let u: Retained<NSUUID> = msg_send![&*descripcion, UUID];
        u.UUIDString().to_string()
    };
    let puntero = Retained::as_ptr(&descripcion) as *mut c_void;
    Ok(DescripcionDelTap(puntero, uid, descripcion))
}

/// Crea el dispositivo agregado **privado** que hace legible el tap.
///
/// Un tap por sí solo no se puede leer: hay que envolverlo en un dispositivo agregado, que es la
/// pieza de Core Audio que junta varias fuentes en una. Se marca privado (no aparece en los
/// ajustes de sonido del usuario) y con arranque automático.
fn crear_agregado(uid_del_tap: &str) -> Result<ObjetoDeAudio, String> {
    use objc2::runtime::AnyObject;
    use objc2_foundation::{NSArray, NSDictionary, NSNumber, NSString};

    let salida = leer_objeto(OBJETO_SISTEMA, SALIDA_POR_DEFECTO, AMBITO_GLOBAL)
        .ok_or("este Mac no tiene salida de audio por defecto")?;
    let uid_salida = leer_cadena(salida, UID_DEL_DISPOSITIVO)
        .ok_or("no se pudo identificar la salida de audio por defecto")?;

    let nuestro_uid = format!("com.angelghost.captura.{}", uuid_sencillo());
    let sub: objc2::rc::Retained<NSDictionary<NSString, AnyObject>> = NSDictionary::from_slices(
        &[&*NSString::from_str("uid")],
        &[&*NSString::from_str(&uid_salida) as &AnyObject],
    );
    let sub_tap: objc2::rc::Retained<NSDictionary<NSString, AnyObject>> = NSDictionary::from_slices(
        &[&*NSString::from_str("uid"), &*NSString::from_str("drift")],
        &[
            &*NSString::from_str(uid_del_tap) as &AnyObject,
            &*NSNumber::new_u32(1) as &AnyObject,
        ],
    );
    let lista_sub = NSArray::from_retained_slice(&[sub]);
    let lista_taps = NSArray::from_retained_slice(&[sub_tap]);

    let descripcion: objc2::rc::Retained<NSDictionary<NSString, AnyObject>> =
        NSDictionary::from_slices(
            &[
                &*NSString::from_str("name"),
                &*NSString::from_str("uid"),
                &*NSString::from_str("master"),
                &*NSString::from_str("private"),
                &*NSString::from_str("stacked"),
                &*NSString::from_str("tapautostart"),
                &*NSString::from_str("subdevices"),
                &*NSString::from_str("taps"),
            ],
            &[
                &*NSString::from_str("Angel Ghost") as &AnyObject,
                &*NSString::from_str(&nuestro_uid) as &AnyObject,
                &*NSString::from_str(&uid_salida) as &AnyObject,
                &*NSNumber::new_u32(1) as &AnyObject,
                &*NSNumber::new_u32(0) as &AnyObject,
                &*NSNumber::new_u32(1) as &AnyObject,
                &*lista_sub as &AnyObject,
                &*lista_taps as &AnyObject,
            ],
        );

    let mut dispositivo: ObjetoDeAudio = 0;
    let estado = unsafe {
        AudioHardwareCreateAggregateDevice(
            objc2::rc::Retained::as_ptr(&descripcion) as *const c_void,
            &mut dispositivo,
        )
    };
    if estado != OK || dispositivo == 0 {
        return Err(formato_de_error("no se pudo crear el dispositivo del tap", estado));
    }
    Ok(dispositivo)
}

fn leer_cadena(objeto: ObjetoDeAudio, selector: u32) -> Option<String> {
    use core_foundation::base::TCFType;
    use core_foundation::string::{CFString, CFStringRef};

    let d = direccion(selector, AMBITO_GLOBAL);
    let mut valor: CFStringRef = std::ptr::null_mut();
    let mut tam = std::mem::size_of::<CFStringRef>() as u32;
    let estado = unsafe {
        AudioObjectGetPropertyData(
            objeto,
            &d,
            0,
            std::ptr::null(),
            &mut tam,
            &mut valor as *mut _ as *mut c_void,
        )
    };
    if estado != OK || valor.is_null() {
        return None;
    }
    // `GetPropertyData` devuelve la cadena ya retenida: la envolvemos con `wrap_under_create_rule`
    // para que se suelte al salir de aquí.
    let cadena = unsafe { CFString::wrap_under_create_rule(valor) };
    Some(cadena.to_string())
}

/// Un identificador único y corto, sin traerse una dependencia para generarlo: el reloj del
/// sistema más el identificador del proceso bastan para que dos agregados de la misma máquina no
/// choquen, que es todo lo que se les pide.
fn uuid_sencillo() -> String {
    let ahora = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{}-{ahora:x}", std::process::id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_solo_canal_pasa_tal_cual() {
        let datos = [0.1, 0.2, 0.3];
        assert_eq!(a_mono(&[&datos], 1), vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn el_estereo_entrelazado_se_promedia_por_pares() {
        let datos = [1.0, 0.0, 0.5, 0.5, -1.0, 1.0];
        assert_eq!(a_mono(&[&datos], 2), vec![0.5, 0.5, 0.0]);
    }

    #[test]
    fn un_bloque_por_canal_tambien_se_promedia() {
        let izq = [1.0, 1.0, 1.0];
        let der = [0.0, 0.0, 0.0];
        assert_eq!(a_mono(&[&izq, &der], 1), vec![0.5, 0.5, 0.5]);
    }

    /// **El gate de la mezcla.** Quedarse con el primer canal es lo que sale solo al escribir esto,
    /// y funciona hasta que alguien tiene los auriculares desbalanceados: su voz vive en el canal
    /// que se tiró y la app se queda sorda sin que nada falle.
    ///
    /// Se ve en rojo cambiando el promedio por `varios[0].to_vec()`.
    #[test]
    fn una_voz_que_solo_esta_en_un_canal_no_se_pierde() {
        let mudo = [0.0; 4];
        let hablando = [0.8; 4];
        let mezcla = a_mono(&[&mudo, &hablando], 1);
        assert!(
            mezcla.iter().all(|m| *m > 0.3),
            "la voz del canal derecho se perdió al mezclar: {mezcla:?}"
        );
        let entrelazado = [0.0, 0.8, 0.0, 0.8];
        assert!(a_mono(&[&entrelazado], 2).iter().all(|m| *m > 0.3));
    }

    #[test]
    fn sin_bloques_no_hay_muestras() {
        assert!(a_mono(&[], 2).is_empty());
    }

    #[test]
    fn bloques_de_distinto_largo_no_salen_del_mas_corto() {
        let largo = [1.0; 10];
        let corto = [1.0; 3];
        assert_eq!(a_mono(&[&largo, &corto], 1).len(), 3);
    }

    /// Lo que se puede afirmar sin saber qué Mac corre esto: que la respuesta es una de las
    /// cuatro, que nunca miente por omisión, y que cuando no sabe lo dice con una frase.
    #[test]
    fn la_salida_de_audio_siempre_contesta_algo_que_se_pueda_enseñar() {
        let s = salida_de_audio();
        println!("salida de audio de este Mac: {s:?}");
        match &s {
            Salida::NoSeSabe { motivo } | Salida::Otra { nombre: motivo } => {
                assert!(!motivo.is_empty(), "una salida sin nombre ni motivo no se puede enseñar")
            }
            _ => {}
        }
        // Y la pregunta que de verdad importa tiene tres respuestas, no dos.
        assert!(matches!(s.puede_haber_eco(), Some(true) | Some(false) | None));
    }

    #[test]
    fn solo_los_auriculares_descartan_el_eco() {
        assert_eq!(Salida::Altavoces.puede_haber_eco(), Some(true));
        assert_eq!(Salida::Auriculares.puede_haber_eco(), Some(false));
        assert_eq!(Salida::Otra { nombre: "Altavoz de mesa".into() }.puede_haber_eco(), None);
        assert_eq!(Salida::NoSeSabe { motivo: "x".into() }.puede_haber_eco(), None);
    }

    #[test]
    fn los_selectores_son_las_cuatro_letras_de_la_cabecera() {
        assert_eq!(ENTRADA_POR_DEFECTO, u32::from_be_bytes(*b"dIn "));
        assert_eq!(FORMATO_DEL_TAP, u32::from_be_bytes(*b"tfmt"));
        assert_eq!(AMBITO_ENTRADA, u32::from_be_bytes(*b"inpt"));
        assert_eq!(PCM_LINEAL, u32::from_be_bytes(*b"lpcm"));
    }

    fn formato(id: u32, bits: u32, banderas: u32) -> Formato {
        Formato { hz: 48_000.0, id, bits, banderas, canales: 2, ..Formato::default() }
    }

    /// **M10 del sprint 001.** El callback lee los bytes del sistema como `f32`. Si Core Audio
    /// negocia otra cosa —entero de 16 bits, por ejemplo— cada muestra sale de los bytes de dos
    /// muestras distintas: no es un fallo ruidoso, es ruido, y el detector de voz lo toma por
    /// sonido. Antes se reinterpretaba a ciegas; ahora el grifo no abre.
    #[test]
    fn solo_se_abre_el_grifo_si_el_audio_llega_como_flotante_de_32_bits() {
        let bueno = formato(PCM_LINEAL, 32, ES_FLOTANTE | ES_EMPAQUETADO);
        assert!(es_float32_empaquetado(&bueno), "el formato que el Mac negocia de verdad");

        for (nombre, malo) in [
            ("entero de 16 bits", formato(PCM_LINEAL, 16, ES_EMPAQUETADO)),
            ("flotante sin empaquetar", formato(PCM_LINEAL, 32, ES_FLOTANTE)),
            ("entero de 32 bits", formato(PCM_LINEAL, 32, ES_EMPAQUETADO)),
            ("comprimido (AAC)", formato(cuatro(b"aac "), 32, ES_FLOTANTE | ES_EMPAQUETADO)),
        ] {
            assert!(!es_float32_empaquetado(&malo), "{nombre} no se puede leer como f32");
        }
    }

    #[test]
    fn el_error_del_formato_enseña_las_cuatro_letras() {
        assert_eq!(cuatro_letras(PCM_LINEAL), "lpcm");
        // Un byte que no es imprimible no puede romper el mensaje del log.
        assert_eq!(cuatro_letras(0), "····");
    }
}
