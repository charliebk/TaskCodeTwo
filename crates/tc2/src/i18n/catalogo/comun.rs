//! Entradas que no son de ninguna zona.

crate::i18n::textos! {
    /// Texto que viene de fuera de `tc2` (git, el sistema, HTTP) y se
    /// muestra tal cual.
    EXTERNO { es: "{texto}", en: "{texto}" }
    /// Un fichero propio que existe y no se lee (`init`, `desinstalar`,
    /// la configuracion: `crate::lectura::no_se_lee`).
    NO_SE_LEE {
        es: "no se puede leer {ruta}: {error}",
        en: "cannot read {ruta}: {error}",
    }
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
    /// Rechazos comunes de las operaciones de una tarea; `{orden}` es la
    /// orden (`start`, `descartar`, `pausa`, `reanudar`).
    TAREA_ASIGNADO_HEREDADO {
        es: "la tarea {id} tiene un asignado_a heredado de la 0.16 ({texto}): hay que reasignarla a una persona de la lista antes de usar tc2 {orden} con ella",
        en: "task {id} has an assignee inherited from 0.16 ({texto}): it has to be reassigned to a person on the list before using tc2 {orden} on it",
    }
    TAREA_TERMINADA {
        es: "la tarea {id} ya esta terminada: tc2 {orden} no se aplica a una tarea terminada",
        en: "task {id} is already finished: tc2 {orden} does not apply to a finished task",
    }
    TAREA_DESCARTADA {
        es: "la tarea {id} esta descartada: tc2 {orden} no se aplica a una tarea descartada",
        en: "task {id} is discarded: tc2 {orden} does not apply to a discarded task",
    }
    TAREA_ESTADO_NO_VALIDO {
        es: "la tarea {id} esta {estado}: tc2 {orden} solo se aplica a tareas en {estados}",
        en: "task {id} is {estado}: tc2 {orden} only applies to tasks in {estados}",
    }
}
