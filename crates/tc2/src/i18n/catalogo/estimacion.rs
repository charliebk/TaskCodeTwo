//! Mensajes de `tc2 estimar` (F3-13).

crate::i18n::textos! {
    USO_ESTIMAR {
        es: "uso: estimar [--json] [TASK-NNN | --error TASK-NNN]",
        en: "usage: estimar [--json] [TASK-NNN | --error TASK-NNN]",
    }
    SIN_ESTIMACION {
        es: "sin estimacion guardada en {id}: la pone tc2 estimar (o start); sin ella no hay error que medir",
        en: "no saved estimate in {id}: tc2 estimar (or start) sets it; without it there is no error to measure",
    }
    SIN_CIFRAS {
        es: "la estimacion guardada de {id} es sin-datos (no habia tareas parecidas): no hay cifras que comparar con el real",
        en: "the saved estimate of {id} is sin-datos (there were no similar tasks): there are no figures to compare with the actual time",
    }
    ESTIMACION_ILEGIBLE {
        es: "la estimacion guardada de {id} no tiene la forma de F1-17: {error}",
        en: "the saved estimate of {id} does not have the F1-17 shape: {error}",
    }
    SIN_REAL {
        es: "la tarea {id} no tiene real que medir ({motivo}): hacen falta start, review y finish en orden en sus transiciones",
        en: "task {id} has no actual time to measure ({motivo}): it needs start, review and finish in order in its transitions",
    }
}
