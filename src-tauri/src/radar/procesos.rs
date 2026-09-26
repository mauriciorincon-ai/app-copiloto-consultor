//! **La mitad coral**: los procesos de ESTE Mac contra el catálogo.
//!
//! La lista se le pide al núcleo con dos llamadas de `libproc` —la misma que usa el Monitor de
//! Actividad— y no sale de aquí: se coteja en memoria y se tira. Nada de otra máquina entra en la
//! ecuación; no hay socket, ni nombre que resolver, ni programa externo.

use super::catalogo::Fila;
use super::Programa;

/// Las rutas de los ejecutables de todos los procesos de este Mac que se dejan ver.
///
/// Algunos del sistema no dan su ruta a un proceso sin privilegios; se saltan, y no importa: el
/// catálogo es de programas que un usuario instala, no de demonios de Apple.
#[cfg(target_os = "macos")]
pub fn listar() -> Vec<String> {
    use libc::{c_int, c_void, pid_t, proc_listallpids, proc_pidpath, PROC_PIDPATHINFO_MAXSIZE};
    // SEGURIDAD: con un búfer nulo, `proc_listallpids` solo devuelve cuántos procesos hay.
    let cuantos = unsafe { proc_listallpids(std::ptr::null_mut(), 0) };
    if cuantos <= 0 {
        return Vec::new();
    }
    // Holgura: entre las dos llamadas pueden nacer procesos.
    let mut pids: Vec<pid_t> = vec![0; cuantos as usize + 64];
    let bytes = (pids.len() * std::mem::size_of::<pid_t>()) as c_int;
    // SEGURIDAD: el búfer tiene exactamente `bytes` bytes y el núcleo no escribe más.
    let n = unsafe { proc_listallpids(pids.as_mut_ptr() as *mut c_void, bytes) };
    if n <= 0 {
        return Vec::new();
    }
    pids.truncate(n as usize);
    let mut ruta = vec![0u8; PROC_PIDPATHINFO_MAXSIZE as usize];
    let mut rutas = Vec::with_capacity(pids.len());
    for pid in pids {
        // SEGURIDAD: `ruta` mide PROC_PIDPATHINFO_MAXSIZE, el tamaño que la API exige.
        let largo = unsafe {
            proc_pidpath(pid, ruta.as_mut_ptr() as *mut c_void, PROC_PIDPATHINFO_MAXSIZE as u32)
        };
        if largo > 0 {
            rutas.push(String::from_utf8_lossy(&ruta[..largo as usize]).into_owned());
        }
    }
    rutas
}

#[cfg(not(target_os = "macos"))]
pub fn listar() -> Vec<String> {
    Vec::new()
}

/// El nombre del ejecutable: el último tramo de la ruta. Es lo que el catálogo compara.
fn ejecutable(ruta: &str) -> &str {
    ruta.rsplit('/').next().unwrap_or(ruta)
}

/// **El cotejo.** Un programa del catálogo está si alguno de sus ejecutables corre, comparado
/// ENTERO y sin mayúsculas. Nunca por trozos: «Teams» no es «TeamViewer», «AnyConnect» no es
/// «AnyDesk» y «Screen Sharing» —la app con la que TÚ miras otra pantalla— no es `screensharingd`,
/// que es por donde otro mira la tuya. El kit de procesos sintéticos trae esos parecidos a propósito.
///
/// Cada programa sale una vez aunque tenga diez procesos, en el orden del catálogo.
pub fn cotejar(rutas: &[String], catalogo: &[Fila]) -> Vec<Programa> {
    let corriendo: std::collections::HashSet<String> =
        rutas.iter().map(|r| ejecutable(r).to_lowercase()).collect();
    catalogo
        .iter()
        .filter(|f| f.procesos.iter().any(|p| corriendo.contains(&p.to_lowercase())))
        .map(Fila::programa)
        .collect()
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::radar::catalogo;

    fn rutas(lista: &[&str]) -> Vec<String> {
        lista.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn se_compara_el_ejecutable_entero_y_no_un_trozo() {
        let una_fila = catalogo::programas()
            .iter()
            .find(|f| !f.procesos.is_empty())
            .expect("el catálogo no tiene filas por proceso");
        let exe = &una_fila.procesos[0];
        // Entero, en otra carpeta y con otras mayúsculas: sí.
        let si = cotejar(&rutas(&[&format!("/Applications/X.app/Contents/MacOS/{}", exe.to_uppercase())]), catalogo::programas());
        assert_eq!(si.len(), 1, "no reconoció «{exe}»");
        // Con un trozo de más o de menos: no.
        let no = cotejar(
            &rutas(&[&format!("/usr/local/bin/{exe}Helper"), &format!("/bin/{}", &exe[..exe.len() - 1])]),
            catalogo::programas(),
        );
        assert!(no.is_empty(), "un trozo de «{exe}» contó como «{exe}»: {no:?}");
    }

    #[test]
    fn un_programa_con_varios_procesos_sale_una_vez() {
        let fila = catalogo::programas()
            .iter()
            .find(|f| f.procesos.len() >= 2)
            .expect("ninguna fila tiene dos procesos");
        let r: Vec<String> = fila.procesos.iter().map(|p| format!("/opt/{p}")).collect();
        assert_eq!(cotejar(&r, catalogo::programas()).len(), 1);
    }

    fn del_kit(texto: &str) -> Vec<String> {
        texto
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(String::from)
            .collect()
    }

    /// **EL KIT DEL RADAR** (`docs/kit-de-prueba/radar/`): un Mac sintético limpio —con nombres
    /// parecidos a los del catálogo metidos a propósito— da **cero** programas; el mismo Mac con un
    /// programa de cada fila del catálogo da **todas** las filas, ni una más. Si alguien añade una
    /// fila al catálogo sin añadirla al kit, esto falla: el 100 % se mide contra el catálogo entero.
    #[test]
    fn el_kit_de_procesos_sinteticos() {
        let limpio = del_kit(include_str!("../../../docs/kit-de-prueba/radar/mac-limpio.txt"));
        assert!(limpio.len() > 60, "el Mac limpio del kit tiene {} procesos", limpio.len());
        let falsos = cotejar(&limpio, catalogo::programas());
        assert!(falsos.is_empty(), "falsos positivos en un Mac limpio: {falsos:?}");

        let mut vigilado = limpio.clone();
        vigilado.extend(del_kit(include_str!("../../../docs/kit-de-prueba/radar/mac-vigilado.txt")));
        let vistos: Vec<String> =
            cotejar(&vigilado, catalogo::programas()).into_iter().map(|p| p.nombre).collect();
        let todos: Vec<String> = catalogo::programas()
            .iter()
            .filter(|f| !f.inscripcion)
            .map(|f| f.nombre.clone())
            .collect();
        assert_eq!(vistos, todos, "el kit no encuentra todo el catálogo, o encuentra de más");
        println!("[kit del radar] {} filas del catálogo · 0 falsos positivos en {} procesos", todos.len(), limpio.len());
    }

    /// El núcleo de este Mac contesta y el radar sabe leerlo. En la CI de macOS también corre: la
    /// lista de procesos no pide permisos.
    #[cfg(target_os = "macos")]
    #[test]
    fn el_nucleo_de_este_mac_da_su_lista_de_procesos() {
        let r = listar();
        assert!(r.len() > 10, "solo {} procesos: la lista no se está leyendo", r.len());
        let yo = std::env::current_exe().unwrap();
        let yo = yo.file_name().unwrap().to_string_lossy();
        assert!(
            r.iter().any(|x| ejecutable(x) == yo),
            "el propio test («{yo}») no aparece entre los procesos"
        );
    }
}
