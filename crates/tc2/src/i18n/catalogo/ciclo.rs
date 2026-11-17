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
}
