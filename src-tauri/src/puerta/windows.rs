//! LA PUERTA EN WINDOWS (sprint 012, ADR 018 y ADR 022 enmienda 2). Parte del módulo protegido: las únicas líneas que
//! crean, abren, leen o escriben el pipe llevan su marca y el ADR en la misma línea.
//!
//! **Un named pipe con la MISMA forma que el socket Unix** (`UnixListener`, `UnixStream`): `socket.rs` no cambia en el
//! Mac, y en Windows la política, el hilo que atiende y vigila, la lectura con tope y la respuesta son las mismas. Lo que
//! en el Mac hace la carpeta en 700 y el socket en 600, aquí lo hace el pipe al nacer:
//!
//! - **DACL solo de tu usuario** (`D:P(A;;GA;;;<tu SID>)`): ni otra cuenta de este PC ni un servicio lo abren;
//! - **`PIPE_REJECT_REMOTE_CLIENTS`**: nadie lo abre desde otra máquina (regla dura 9, `puerta-solo-local`);
//! - **`FILE_FLAG_FIRST_PIPE_INSTANCE` y una sola instancia, que se reutiliza**: si otro proceso ya tenía ese nombre la
//!   puerta no abre, y mientras está abierta nadie puede colarse con otra instancia a recoger la llave de `ghost`;
//! - **`ghost` se identifica sin delegar** (`SECURITY_IDENTIFICATION`): quien atienda el pipe no puede actuar como él.
//!
//! El pipe no vive en el disco: desaparece cuando se cierra su último mango, así que no hay archivo que borrar ni que
//! quede tras una caída. La llave sí: la limpia el arranque (`socket::limpiar_lo_que_quedo`).

use std::cell::Cell;
use std::ffi::c_void;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, LocalFree, ERROR_BROKEN_PIPE, ERROR_FILE_NOT_FOUND, ERROR_INSUFFICIENT_BUFFER,
    ERROR_IO_INCOMPLETE, ERROR_IO_PENDING, ERROR_NO_DATA, ERROR_PIPE_BUSY, ERROR_PIPE_CONNECTED,
    ERROR_PIPE_NOT_CONNECTED, ERROR_SEM_TIMEOUT, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
    WAIT_OBJECT_0,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertSecurityDescriptorToStringSecurityDescriptorW, ConvertSidToStringSidW,
    ConvertStringSecurityDescriptorToSecurityDescriptorW, GetSecurityInfo, SDDL_REVISION_1, SE_KERNEL_OBJECT,
};
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenUser, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_QUERY,
    TOKEN_USER,
};
use windows_sys::Win32::Storage::FileSystem as fs;
use windows_sys::Win32::System::Pipes as tubos;
use windows_sys::Win32::System::Threading::{
    CreateEventW, GetCurrentProcess, OpenProcessToken, WaitForSingleObject, INFINITE,
};
use windows_sys::Win32::System::IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED};

/// Byte a byte, esperando, y **sin clientes de otra máquina**.
pub const MODO: u32 = tubos::PIPE_TYPE_BYTE | tubos::PIPE_READMODE_BYTE | tubos::PIPE_WAIT | tubos::PIPE_REJECT_REMOTE_CLIENTS;

/// Lo que la puerta abre: lectura y escritura, sin bloquear el hilo, y **solo si nadie tenía ya ese nombre**.
pub const APERTURA: u32 = fs::PIPE_ACCESS_DUPLEX | fs::FILE_FLAG_OVERLAPPED | fs::FILE_FLAG_FIRST_PIPE_INSTANCE;

/// Lo que se guarda en el pipe mientras el otro lado lee. La respuesta más larga pasa por él a trozos.
const BUFER: u32 = 64 * 1024;

/// Lo que la puerta espera, tras contestar, a que `ghost` cierre su lado antes de soltarlo.
const AL_SOLTAR: Duration = Duration::from_secs(2);

/// Un mango de Windows que se puede mover de hilo: la puerta lo usa desde un solo hilo a la vez (el que atiende).
struct Mango(HANDLE);

// SEGURIDAD: un HANDLE es un número del núcleo; aquí lo usa un solo hilo a la vez y se cierra una sola vez (Drop).
unsafe impl Send for Mango {}
// SEGURIDAD: ídem; las operaciones del núcleo sobre un mismo mango son seguras entre hilos.
unsafe impl Sync for Mango {}

impl Drop for Mango {
    fn drop(&mut self) {
        if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
            // SEGURIDAD: un mango que esta pieza abrió y nadie más cierra.
            unsafe { CloseHandle(self.0) };
        }
    }
}

/// Un evento de reinicio manual, para esperar una operación con tope de tiempo.
fn evento() -> io::Result<Mango> {
    // SEGURIDAD: sin atributos ni nombre; Windows devuelve un mango nuevo o nulo.
    let e = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
    if e.is_null() {
        return Err(io::Error::last_os_error());
    }
    Ok(Mango(e))
}

/// La ruta del pipe en UTF-16 terminada en cero, como la pide Windows.
fn ancho(ruta: &Path) -> Vec<u16> {
    ruta.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
}

/// Un error de Windows por su número.
fn de(codigo: u32) -> io::Error {
    io::Error::from_raw_os_error(codigo as i32)
}

/// Un texto UTF-16 que Windows dejó en memoria suya, copiado y devuelto a Windows.
///
/// SEGURIDAD: `texto` apunta a un UTF-16 terminado en cero que reservó Windows con `LocalAlloc`.
unsafe fn tomar(texto: *mut u16) -> String {
    let mut n = 0;
    while unsafe { *texto.add(n) } != 0 {
        n += 1;
    }
    let s = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(texto, n) });
    unsafe { LocalFree(texto as *mut c_void) };
    s
}

/// **El SID del usuario que corre este proceso**, en texto (`S-1-5-21-…`). Es el único que la DACL deja pasar.
pub fn sid_del_usuario() -> io::Result<String> {
    let mut token: HANDLE = std::ptr::null_mut();
    // SEGURIDAD: el token de este mismo proceso, solo para leerlo.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let token = Mango(token);
    let mut largo = 0u32;
    // SEGURIDAD: primero se pregunta cuánto mide; falla a propósito con «buffer corto».
    unsafe { GetTokenInformation(token.0, TokenUser, std::ptr::null_mut(), 0, &mut largo) };
    // SEGURIDAD: sin argumentos; lee el último error de este hilo.
    let e = unsafe { GetLastError() };
    if e != ERROR_INSUFFICIENT_BUFFER || largo == 0 {
        return Err(de(e));
    }
    // En `u64` para que `TOKEN_USER` quede alineado.
    let mut caja = vec![0u64; (largo as usize).div_ceil(8)];
    // SEGURIDAD: la caja mide al menos `largo` bytes y vive hasta el final de la función.
    if unsafe { GetTokenInformation(token.0, TokenUser, caja.as_mut_ptr() as *mut c_void, largo, &mut largo) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SEGURIDAD: Windows dejó un TOKEN_USER al principio de la caja, alineada a 8.
    let usuario = unsafe { &*(caja.as_ptr() as *const TOKEN_USER) };
    let mut texto: *mut u16 = std::ptr::null_mut();
    // SEGURIDAD: el SID vive dentro de la caja; Windows reserva el texto y lo devolvemos en `tomar`.
    if unsafe { ConvertSidToStringSidW(usuario.User.Sid, &mut texto) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SEGURIDAD: texto UTF-16 terminado en cero que reservó Windows.
    Ok(unsafe { tomar(texto) })
}

/// **La DACL de la puerta**, en SDDL: solo tu usuario, todos los permisos; nadie más, y sin heredar nada (`P`).
pub fn dacl_de_la_puerta_para(sid: &str) -> String {
    format!("D:P(A;;GA;;;{sid})")
}

/// Un descriptor de seguridad que reservó Windows y se le devuelve al soltarlo.
struct Descriptor(PSECURITY_DESCRIPTOR);

impl Drop for Descriptor {
    fn drop(&mut self) {
        // SEGURIDAD: lo reservó `ConvertStringSecurityDescriptorToSecurityDescriptorW` y solo se suelta aquí.
        unsafe { LocalFree(self.0) };
    }
}

fn descriptor_del_usuario() -> io::Result<Descriptor> {
    let sddl: Vec<u16> = dacl_de_la_puerta_para(&sid_del_usuario()?).encode_utf16().chain(std::iter::once(0)).collect();
    let mut sd: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
    // SEGURIDAD: un SDDL terminado en cero; Windows reserva el descriptor y lo suelta `Descriptor`.
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(sddl.as_ptr(), SDDL_REVISION_1, &mut sd, std::ptr::null_mut())
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(Descriptor(sd))
}

/// **La DACL que tiene de verdad el pipe abierto**, leída desde el lado del cliente, en SDDL. La usa la prueba que
/// comprueba que solo tu usuario entra: la DACL se lee del pipe, no se cree.
pub fn dacl_de(ruta: &Path) -> io::Result<String> {
    let cliente = UnixStream::connect(ruta)?;
    let mut sd: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
    // SEGURIDAD: el mango del cliente se abrió con lectura (incluye READ_CONTROL); Windows reserva el descriptor.
    let r = unsafe {
        GetSecurityInfo(
            cliente.mango.0 .0,
            SE_KERNEL_OBJECT,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut sd,
        )
    };
    if r != 0 {
        return Err(de(r));
    }
    let sd = Descriptor(sd);
    let mut texto: *mut u16 = std::ptr::null_mut();
    // SEGURIDAD: el descriptor vive en `sd`; Windows reserva el texto y lo devolvemos en `tomar`.
    if unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd.0,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut texto,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    // SEGURIDAD: texto UTF-16 terminado en cero que reservó Windows.
    Ok(unsafe { tomar(texto) })
}

/// **¿Hay puerta?** Sin conectarse: conectarse gastaría la única instancia. Si el pipe existe pero está atendiendo a
/// otro, también hay puerta.
pub fn existe(ruta: &Path) -> bool {
    let nombre = ancho(ruta);
    // SEGURIDAD: un nombre terminado en cero; esperar 1 ms no conecta nada.
    if unsafe { tubos::WaitNamedPipeW(nombre.as_ptr(), 1) } != 0 {
        return true;
    }
    // SEGURIDAD: sin argumentos; lee el último error de este hilo.
    matches!(unsafe { GetLastError() }, ERROR_SEM_TIMEOUT | ERROR_PIPE_BUSY)
}

/// La instancia del pipe: la que nace con la puerta y se reutiliza para cada conexión.
struct Instancia(Mango);

/// **El lado de la app**: la instancia del pipe y la espera de una conexión, sin bloquear.
pub struct UnixListener {
    instancia: Arc<Instancia>,
    evento: Mango,
    /// La espera de `ConnectNamedPipe` en curso: Windows escribe aquí hasta que llega alguien o se cancela.
    espera: Box<std::cell::UnsafeCell<OVERLAPPED>>,
    pendiente: Cell<bool>,
}

// SEGURIDAD: el `OVERLAPPED` y su evento solo los toca el hilo que atiende la puerta, que es dueño de esto.
unsafe impl Send for UnixListener {}

impl UnixListener {
    /// **Crea la puerta**: la única instancia del pipe, con la DACL de tu usuario y sin clientes remotos. Si el nombre ya
    /// existía —otro proceso se adelantó—, falla y la puerta no abre.
    pub fn bind(ruta: &Path) -> io::Result<UnixListener> {
        let nombre = ancho(ruta);
        let sd = descriptor_del_usuario()?;
        let atributos = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: sd.0,
            bInheritHandle: 0,
        };
        // SEGURIDAD: nombre terminado en cero y atributos vivos durante la llamada.
        let h = unsafe { tubos::CreateNamedPipeW(nombre.as_ptr(), APERTURA, MODO, 1, BUFER, BUFER, 0, &atributos) }; // verify-ephemeral:allow — ADR 018 §1 y ADR 022: la puerta en Windows es un named pipe con la DACL del usuario
        if h == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let instancia = Arc::new(Instancia(Mango(h)));
        let evento = evento()?;
        // SEGURIDAD: un OVERLAPPED en ceros es válido; solo se le pone el evento.
        let mut ov: OVERLAPPED = unsafe { std::mem::zeroed() };
        ov.hEvent = evento.0;
        Ok(UnixListener {
            instancia,
            evento,
            espera: Box::new(std::cell::UnsafeCell::new(ov)),
            pendiente: Cell::new(false),
        })
    }

    /// Siempre espera sin bloquear: `accept` contesta `WouldBlock` mientras no llegue nadie.
    pub fn set_nonblocking(&self, _no: bool) -> io::Result<()> {
        Ok(())
    }

    /// Una conexión, si la hay. Sin ella, `WouldBlock`: el hilo de la puerta vigila la reunión entre una y otra.
    pub fn accept(&self) -> io::Result<(UnixStream, ())> {
        let h = self.instancia.0 .0;
        let ov = self.espera.get();
        if !self.pendiente.get() {
            // SEGURIDAD: el OVERLAPPED vive en una caja del oyente mientras dure la espera.
            unsafe {
                (*ov).Internal = 0;
                (*ov).InternalHigh = 0;
                (*ov).Anonymous = std::mem::zeroed();
                (*ov).hEvent = self.evento.0;
            }
            // SEGURIDAD: mango vivo y OVERLAPPED estable (en la caja).
            if unsafe { tubos::ConnectNamedPipe(h, ov) } != 0 {
                return self.conexion();
            }
            // SEGURIDAD: sin argumentos; lee el último error de este hilo.
            match unsafe { GetLastError() } {
                ERROR_PIPE_CONNECTED => return self.conexion(),
                ERROR_IO_PENDING => self.pendiente.set(true),
                // Llegó y se fue antes de que mirásemos: se suelta y se vuelve a esperar.
                ERROR_NO_DATA => {
                    // SEGURIDAD: mango vivo de esta instancia.
                    unsafe { tubos::DisconnectNamedPipe(h) };
                    return Err(io::ErrorKind::WouldBlock.into());
                }
                e => return Err(de(e)),
            }
        }
        let mut n = 0u32;
        // SEGURIDAD: el mismo OVERLAPPED de la espera en curso; sin esperar.
        if unsafe { GetOverlappedResult(h, ov, &mut n, 0) } != 0 {
            self.pendiente.set(false);
            return self.conexion();
        }
        // SEGURIDAD: sin argumentos; lee el último error de este hilo.
        match unsafe { GetLastError() } {
            ERROR_IO_INCOMPLETE => Err(io::ErrorKind::WouldBlock.into()),
            e => {
                self.pendiente.set(false);
                Err(de(e))
            }
        }
    }

    fn conexion(&self) -> io::Result<(UnixStream, ())> {
        Ok((UnixStream::con(self.instancia.clone(), true)?, ()))
    }
}

impl Drop for UnixListener {
    fn drop(&mut self) {
        if self.pendiente.get() {
            let h = self.instancia.0 .0;
            let ov = self.espera.get();
            let mut n = 0u32;
            // SEGURIDAD: se cancela la espera en curso y se espera a que Windows suelte el OVERLAPPED.
            unsafe {
                CancelIoEx(h, ov);
                GetOverlappedResult(h, ov, &mut n, 1);
            }
        }
    }
}

/// **Una conexión**: del lado de la app (sobre la instancia, que al soltarla se desconecta y queda para la siguiente)
/// o del lado de `ghost` (su propio mango).
pub struct UnixStream {
    mango: Arc<Instancia>,
    de_la_app: bool,
    evento: Mango,
    lectura: Cell<Option<Duration>>,
    escritura: Cell<Option<Duration>>,
}

// SEGURIDAD: una conexión la usa un solo hilo; el evento y los topes son suyos.
unsafe impl Send for UnixStream {}

impl UnixStream {
    fn con(mango: Arc<Instancia>, de_la_app: bool) -> io::Result<UnixStream> {
        Ok(UnixStream { mango, de_la_app, evento: evento()?, lectura: Cell::new(None), escritura: Cell::new(None) })
    }

    /// **El lado de `ghost`**: abre el pipe, sin dejar que quien lo atiende actúe en su nombre. Cerrada = `NotFound`.
    pub fn connect(ruta: &Path) -> io::Result<UnixStream> {
        let nombre = ancho(ruta);
        for intento in 0..2 {
            // SEGURIDAD: nombre terminado en cero; sin atributos ni plantilla.
            let h = unsafe {
                fs::CreateFileW( // verify-ephemeral:allow — ADR 018 §3 y ADR 022: ghost abre el pipe de la puerta, solo identificándose
                    nombre.as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    0,
                    std::ptr::null(),
                    fs::OPEN_EXISTING,
                    fs::FILE_FLAG_OVERLAPPED | fs::SECURITY_SQOS_PRESENT | fs::SECURITY_IDENTIFICATION,
                    std::ptr::null_mut(),
                )
            };
            if h != INVALID_HANDLE_VALUE {
                return UnixStream::con(Arc::new(Instancia(Mango(h))), false);
            }
            // SEGURIDAD: sin argumentos; lee el último error de este hilo.
            match unsafe { GetLastError() } {
                ERROR_FILE_NOT_FOUND => {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "la puerta está cerrada"));
                }
                ERROR_PIPE_BUSY if intento == 0 => {
                    // SEGURIDAD: nombre terminado en cero; espera a que la instancia quede libre.
                    unsafe { tubos::WaitNamedPipeW(nombre.as_ptr(), 2000) };
                }
                e => return Err(de(e)),
            }
        }
        Err(io::Error::other("la puerta está atendiendo otra orden: vuelve a intentarlo"))
    }

    pub fn set_nonblocking(&self, _no: bool) -> io::Result<()> {
        Ok(())
    }

    pub fn set_read_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.lectura.set(t);
        Ok(())
    }

    pub fn set_write_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.escritura.set(t);
        Ok(())
    }

    /// Una lectura o una escritura con su tope de tiempo. Sin tope, espera lo que haga falta.
    fn operar(&self, buf: *mut u8, largo: usize, leer: bool, tope: Option<Duration>) -> io::Result<usize> {
        let h = self.mango.0 .0;
        let largo = largo.min(u32::MAX as usize) as u32;
        // SEGURIDAD: un OVERLAPPED en ceros con el evento de esta conexión; vive hasta que la operación acaba.
        let mut ov: OVERLAPPED = unsafe { std::mem::zeroed() };
        ov.hEvent = self.evento.0;
        let mut n = 0u32;
        // SEGURIDAD: `buf` mide `largo` bytes y vive hasta que la operación acaba (se espera o se cancela abajo).
        let hecho = unsafe {
            if leer {
                fs::ReadFile(h, buf, largo, std::ptr::null_mut(), &mut ov) // verify-ephemeral:allow — ADR 018 §3 y ADR 022: la orden llega por el pipe
            } else {
                fs::WriteFile(h, buf, largo, std::ptr::null_mut(), &mut ov) // verify-ephemeral:allow — ADR 018 §3 y ADR 022: la respuesta vuelve por el pipe
            }
        };
        if hecho == 0 {
            // SEGURIDAD: sin argumentos; lee el último error de este hilo.
            let e = unsafe { GetLastError() };
            if e != ERROR_IO_PENDING {
                return fin(e, leer);
            }
            let ms = tope.map(|t| t.as_millis().min(u128::from(INFINITE - 1)) as u32).unwrap_or(INFINITE);
            // SEGURIDAD: el evento de esta conexión, vivo.
            if unsafe { WaitForSingleObject(self.evento.0, ms) } != WAIT_OBJECT_0 {
                // SEGURIDAD: se cancela la operación y se espera a que Windows suelte `ov` y `buf`.
                unsafe {
                    CancelIoEx(h, &ov);
                    GetOverlappedResult(h, &ov, &mut n, 1);
                }
                return Err(io::ErrorKind::TimedOut.into());
            }
        }
        // SEGURIDAD: la operación ya acabó; solo se lee cuántos bytes movió.
        if unsafe { GetOverlappedResult(h, &ov, &mut n, 1) } == 0 {
            // SEGURIDAD: sin argumentos; lee el último error de este hilo.
            return fin(unsafe { GetLastError() }, leer);
        }
        Ok(n as usize)
    }
}

/// Un error al leer o escribir. El otro lado cerró: para quien lee es el final; para quien escribe, un error.
fn fin(e: u32, leer: bool) -> io::Result<usize> {
    match e {
        ERROR_BROKEN_PIPE | ERROR_PIPE_NOT_CONNECTED | ERROR_NO_DATA if leer => Ok(0),
        ERROR_BROKEN_PIPE | ERROR_PIPE_NOT_CONNECTED | ERROR_NO_DATA => Err(io::ErrorKind::BrokenPipe.into()),
        e => Err(de(e)),
    }
}

impl io::Read for &UnixStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.operar(buf.as_mut_ptr(), buf.len(), true, self.lectura.get())
    }
}

impl io::Write for UnixStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Escribir no toca `buf`: el puntero mutable es solo el tipo que pide la función compartida de leer y escribir.
        self.operar(buf.as_ptr() as *mut u8, buf.len(), false, self.escritura.get())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for UnixStream {
    /// Del lado de la app, **la instancia no se cierra: se desconecta** y queda para la siguiente orden, así nadie puede
    /// crear otra con el mismo nombre. Antes se espera a que `ghost` lea la respuesta y cierre su lado (con tope): soltar
    /// antes la tiraría sin leer.
    fn drop(&mut self) {
        if !self.de_la_app {
            return;
        }
        let mut resto = [0u8; 64];
        while let Ok(n) = self.operar(resto.as_mut_ptr(), resto.len(), true, Some(AL_SOLTAR)) {
            if n == 0 {
                break;
            }
        }
        // SEGURIDAD: la instancia de la puerta, viva mientras la tenga el oyente.
        unsafe { tubos::DisconnectNamedPipe(self.mango.0 .0) };
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// **La puerta no se abre desde otra máquina, ni la ocupa otro proceso primero**: las dos banderas están en la
    /// creación del pipe. ¿Puede fallar? Sí: quitar cualquiera de las dos de `MODO` o de `APERTURA` lo pone rojo.
    #[test]
    fn el_pipe_nace_solo_local_y_sin_que_nadie_se_adelante() {
        assert_ne!(MODO & tubos::PIPE_REJECT_REMOTE_CLIENTS, 0, "el pipe acepta clientes de otra máquina");
        assert_ne!(APERTURA & fs::FILE_FLAG_FIRST_PIPE_INSTANCE, 0, "otro proceso podría crear el pipe antes");
        assert_eq!(MODO & tubos::PIPE_TYPE_MESSAGE, 0, "la puerta habla en bytes, como el socket");
    }

    /// La DACL de la puerta: solo tu SID, todos los permisos, sin herencia. ¿Puede fallar? Sí: un `WD` (Everyone) o un
    /// `AU` (usuarios autenticados) en el SDDL, o quitar la `P`, es rojo.
    #[test]
    fn la_dacl_es_solo_del_usuario() {
        let d = dacl_de_la_puerta_para("S-1-5-21-1-2-3-1001");
        assert_eq!(d, "D:P(A;;GA;;;S-1-5-21-1-2-3-1001)");
        for ajeno in [";;;WD)", ";;;AU)", ";;;BU)", ";;;AN)", ";;;NU)"] {
            assert!(!d.contains(ajeno), "la DACL deja pasar a {ajeno}");
        }
    }
}
