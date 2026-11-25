//! Fechas UTC sin dependencias (A8-03): el instante `AAAA-MM-DDTHH:MM:SSZ`
//! de las transiciones, la marca de sincronizacion y la estimacion (F3-5b,
//! D6a), y la fecha civil del algoritmo de Howard Hinnant, una sola vez.
//!
//! `xtask` (`fecha_utc` de `xtask/src/main.rs`) tiene su propia copia: solo
//! depende de `distribucion`, no de `tc2`.

/// Instante UTC `AAAA-MM-DDTHH:MM:SSZ` de unos segundos desde 1970, con la
/// fecha de [`fecha_de_dias`].
pub fn instante(segundos: u64) -> String {
    let dias = i64::try_from(segundos / 86_400).unwrap_or(0);
    let resto = segundos % 86_400;
    format!(
        "{}T{:02}:{:02}:{:02}Z",
        fecha_de_dias(dias),
        resto / 3600,
        resto % 3600 / 60,
        resto % 60
    )
}

/// Fecha civil `AAAA-MM-DD` de un numero de dias desde 1970-01-01
/// (algoritmo de Howard Hinnant, `civil_from_days`).
pub(crate) fn fecha_de_dias(dias: i64) -> String {
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let a = yoe + era * 400 + i64::from(m <= 2);
    format!("{a:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Los asserts de fechas que estaban en `alta` y en `toma`, juntos.
    #[test]
    fn fechas_civiles() {
        assert_eq!(fecha_de_dias(0), "1970-01-01");
        assert_eq!(fecha_de_dias(11_016), "2000-02-29");
        assert_eq!(fecha_de_dias(20_772), "2026-11-15");
        assert_eq!(fecha_de_dias(-1), "1969-12-31");
    }

    #[test]
    fn instante_en_utc() {
        assert_eq!(instante(0), "1970-01-01T00:00:00Z");
        assert_eq!(instante(86_399), "1970-01-01T23:59:59Z");
        assert_eq!(instante(951_868_799), "2000-02-29T23:59:59Z");
        assert_eq!(instante(1_794_910_830), "2026-11-17T10:20:30Z");
        assert_eq!(instante(1_794_931_452), "2026-11-17T16:04:12Z");
    }
}
