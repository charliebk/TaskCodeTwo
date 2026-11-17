//! Entradas que no son de ninguna zona.

crate::i18n::textos! {
    /// Texto que viene de fuera de `tc2` (git, el sistema, HTTP) y se
    /// muestra tal cual.
    EXTERNO { es: "{texto}", en: "{texto}" }
    /// La tarea que se quiere cambiar no esta en la rama de estado
    /// (`start`, `descartar`, `pausa`, `reanudar`).
    TAREA_NO_EXISTE {
        es: "la tarea {id} no existe en la rama de estado: comprueba el ID con tc2 status",
        en: "task {id} does not exist in the state branch: check the ID with tc2 status",
    }
    TAREA_FORMATO_1 {
        es: "la tarea {id} esta en el formato de la 0.16: conviertela con tc2 actualizar antes de cambiarla",
        en: "task {id} is in the 0.16 format: convert it with tc2 actualizar before changing it",
    }
    TAREA_ILEGIBLE {
        es: "el tarea.md de {id} no se entiende ({error}): corrigelo antes de cambiarla",
        en: "the tarea.md of {id} cannot be read ({error}): fix it before changing it",
    }
}
