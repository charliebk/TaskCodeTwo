//! Lectura YAML estricta compartida (ADR F1-14, punto 1: el YAML vive detras
//! de un modulo propio). La usan el frontmatter de `tarea.md` (F2-5) y la
//! configuracion `.taskcode/config.yml` (F2-7): la politica contra alias y
//! claves duplicadas de F1-8 existe una sola vez.

use serde_saphyr::{DuplicateKeyPolicy, MergeKeyPolicy, Options};

/// Opciones de lectura: duplicadas, claves de fusion, anclas y alias como
/// error (ninguna tarea ni configuracion real los usa; un alias que crece es
/// un vector de denegacion de servicio, F1-8).
pub(crate) fn opciones() -> Options {
    let mut o = Options::default();
    // Booleanos y numeros solo con la grafia de YAML 1.2 (`yes`, `1_0` o `010`
    // no son bool ni entero, como para un parser estandar).
    o.strict_booleans = true;
    o.underscored_numbers_as_strings = true;
    o.leading_zero_numbers_as_strings = true;
    o.duplicate_keys = DuplicateKeyPolicy::Error;
    o.merge_keys = MergeKeyPolicy::Error;
    let mut presupuesto = o.budget.unwrap_or_default();
    presupuesto.max_aliases = 0;
    presupuesto.max_anchors = 0;
    o.budget = Some(presupuesto);
    o
}
