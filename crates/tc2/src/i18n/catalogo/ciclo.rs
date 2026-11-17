//! Mensajes de `tc2 descartar`, `tc2 pausa` y `tc2 reanudar` (F3-7) y de
//! `tc2 liberar` y `tc2 reasignar` (F3-10).

crate::i18n::textos! {
    USO_LIBERAR {
        es: "uso: liberar [--json] TASK-NNN",
        en: "usage: liberar [--json] TASK-NNN",
    }
    USO_REASIGNAR {
        es: "uso: reasignar [--json] TASK-NNN <login>",
        en: "usage: reasignar [--json] TASK-NNN <login>",
    }
    ACTIVA {
        es: "la tarea {id} la lleva {login} y tuvo actividad hace {dias} dias (filas de Transiciones o commits de su rama publicada): solo su dueno puede lanzar tc2 {orden} sobre ella antes de {umbral} dias sin actividad (clave dias_sin_actividad)",
        en: "task {id} is taken by {login} and had activity {dias} days ago (Transitions rows or commits on its published branch): only its owner can run tc2 {orden} on it before {umbral} days without activity (dias_sin_actividad key)",
    }
    REQUIERE_PERSONA {
        es: "la tarea {id} la lleva {quien}: quitarsela a otra persona lo decide una persona en un terminal (tc2 {orden} con TC2_TOKEN_PERSONA), no el agente",
        en: "task {id} is taken by {quien}: taking it from someone else is decided by a person at a terminal (tc2 {orden} with TC2_TOKEN_PERSONA), not by the agent",
    }
    FUERA_DE_LISTA {
        es: "{login} no es una persona de {ruta} en {base}: reasigna a un login de la lista (un bot o un login mal escrito no valen)",
        en: "{login} is not a person in {ruta} on {base}: reassign to a login on the list (a bot or a misspelt login are not valid)",
    }
    SIN_PUBLICAR {
        es: "el worktree de {id} tiene trabajo que se perderia ({que}): guardalo y publica la rama (tc2 guardar y tc2 actualizar-rama) antes de dejar la tarea",
        en: "the worktree of {id} has work that would be lost ({que}): save it and publish the branch (tc2 guardar and tc2 actualizar-rama) before leaving the task",
    }
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
