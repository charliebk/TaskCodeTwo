//! Mensajes de `tc2 descartar`, `tc2 pausa` y `tc2 reanudar` (F3-7).

crate::i18n::textos! {
    USO_DESCARTAR {
        es: "uso: descartar [--json] TASK-NNN \"<motivo>\"",
        en: "usage: descartar [--json] TASK-NNN \"<reason>\"",
    }
    USO_PAUSA {
        es: "uso: pausa [--json] TASK-NNN",
        en: "usage: pausa [--json] TASK-NNN",
    }
    USO_REANUDAR {
        es: "uso: reanudar [--json] TASK-NNN",
        en: "usage: reanudar [--json] TASK-NNN",
    }
    MOTIVO_INVALIDO {
        es: "el motivo del descarte no puede estar vacio ni llevar saltos de linea o caracteres de control, y tiene como mucho {max} caracteres",
        en: "the reason for discarding cannot be empty or contain line breaks or control characters, and has at most {max} characters",
    }
    DE_OTRA_PERSONA {
        es: "la tarea {id} la lleva {login}: solo quien la lleva (o cualquiera, si no la lleva nadie) puede descartarla, pausarla o reanudarla",
        en: "task {id} is taken by {login}: only whoever has it (or anyone, if nobody has it) can discard, pause or resume it",
    }
    ASIGNADO_HEREDADO {
        es: "la tarea {id} tiene un asignado_a heredado de la 0.16 ({texto}): hay que reasignarla a una persona de la lista antes de cambiarla",
        en: "task {id} has an assignee inherited from 0.16 ({texto}): it has to be reassigned to a person on the list before changing it",
    }
    TERMINADA {
        es: "la tarea {id} ya esta terminada: no se puede descartar, pausar ni reanudar",
        en: "task {id} is already finished: it cannot be discarded, paused or resumed",
    }
    DESCARTADA {
        es: "la tarea {id} esta descartada: no se puede pausar ni reanudar",
        en: "task {id} is discarded: it cannot be paused or resumed",
    }
    ESTADO_NO_VALIDO {
        es: "la tarea {id} esta {estado}: solo se descartan, pausan o reanudan tareas planificadas, en diseno, en curso o en revision",
        en: "task {id} is {estado}: only planned, in-design, in-progress or in-review tasks can be discarded, paused or resumed",
    }
}
