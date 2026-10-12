//! **La hora local del equipo**, en un solo sitio (ADR 022).
//!
//! Hasta el sprint 006 siete funciones la leían cada una con `libc::localtime_r`, y eso dejaba el crate sin compilar
//! fuera de Unix. Aquí vive la única lectura: en Unix con `localtime_r`, exactamente como antes; en Windows con
//! `chrono`, que pregunta lo mismo al sistema. Sin una biblioteca de fechas en macOS para escribir «14:02».

/// Las partes de un instante en la hora local: lo que necesitan las notas, la bandeja, el costo y el transcript.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Local {
    pub anio: i32,
    pub mes: u32,
    pub dia: u32,
    pub hora: u32,
    pub minuto: u32,
    pub segundo: u32,
    /// Los segundos que la zona local lleva sobre UTC en ese instante.
    pub desplazamiento: i64,
}

/// Ahora, en segundos desde la época Unix.
pub fn ahora() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// `instante` (segundos Unix) en la hora local del equipo.
#[cfg(unix)]
pub fn local(instante: i64) -> Local {
    // SEGURIDAD: `localtime_r` escribe en una estructura que vive en esta función.
    unsafe {
        let t = instante as libc::time_t;
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&t, &mut tm).is_null() {
            return en_utc(instante);
        }
        Local {
            anio: tm.tm_year + 1900,
            mes: (tm.tm_mon + 1) as u32,
            dia: tm.tm_mday as u32,
            hora: tm.tm_hour as u32,
            minuto: tm.tm_min as u32,
            segundo: tm.tm_sec as u32,
            desplazamiento: tm.tm_gmtoff as i64,
        }
    }
}

/// `instante` (segundos Unix) en la hora local del equipo.
#[cfg(windows)]
pub fn local(instante: i64) -> Local {
    use chrono::{Datelike, Offset, TimeZone, Timelike};
    match chrono::Local.timestamp_opt(instante, 0).single() {
        Some(t) => Local {
            anio: t.year(),
            mes: t.month(),
            dia: t.day(),
            hora: t.hour(),
            minuto: t.minute(),
            segundo: t.second(),
            desplazamiento: i64::from(t.offset().fix().local_minus_utc()),
        },
        None => en_utc(instante),
    }
}

/// Ni Unix ni Windows: la hora de UTC, dicha. La app no se distribuye ahí; el crate compila.
#[cfg(not(any(unix, windows)))]
pub fn local(instante: i64) -> Local {
    en_utc(instante)
}

/// UTC, sin zona: el respaldo cuando el sistema no contesta. Algoritmo civil de días a fecha (Howard Hinnant).
pub(crate) fn en_utc(instante: i64) -> Local {
    let dias = instante.div_euclid(86_400);
    let del_dia = instante.rem_euclid(86_400);
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let dia = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let mes = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let anio = (yoe + era * 400 + i64::from(mes <= 2)) as i32;
    Local {
        anio,
        mes,
        dia,
        hora: (del_dia / 3_600) as u32,
        minuto: ((del_dia % 3_600) / 60) as u32,
        segundo: (del_dia % 60) as u32,
        desplazamiento: 0,
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// El respaldo de UTC cuenta bien las fechas, también los años bisiestos y el cambio de siglo.
    #[test]
    fn utc_cuenta_bien_las_fechas() {
        assert_eq!(en_utc(0), Local { anio: 1970, mes: 1, dia: 1, hora: 0, minuto: 0, segundo: 0, desplazamiento: 0 });
        let l = en_utc(951_782_400); // 2000-02-29 00:00:00 UTC
        assert_eq!((l.anio, l.mes, l.dia), (2000, 2, 29));
        let l = en_utc(1_791_590_399); // 2026-10-09 23:59:59 UTC
        assert_eq!((l.anio, l.mes, l.dia, l.hora, l.minuto, l.segundo), (2026, 10, 9, 23, 59, 59));
    }

    /// La hora local y la de UTC difieren exactamente en el desplazamiento que el sistema declara.
    #[test]
    fn la_hora_local_es_utc_mas_el_desplazamiento() {
        let t = 1_791_590_399;
        let l = local(t);
        let u = en_utc(t + l.desplazamiento);
        assert_eq!((l.anio, l.mes, l.dia, l.hora, l.minuto, l.segundo), (u.anio, u.mes, u.dia, u.hora, u.minuto, u.segundo));
    }
}
