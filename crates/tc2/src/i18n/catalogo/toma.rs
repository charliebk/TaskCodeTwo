//! Mensajes de `tc2 start` (F3-6).

crate::i18n::textos! {
    USO_START {
        es: "uso: start [--json] TASK-NNN",
        en: "usage: start [--json] TASK-NNN",
    }
    RAMA_INVALIDA {
        es: "la rama {rama} de la tarea {id} no tiene la forma <tipo>/task-nnn-<slug>: corrigela en su tarea.md antes de empezarla",
        en: "branch {rama} of task {id} is not of the form <tipo>/task-nnn-<slug>: fix it in its tarea.md before starting it",
    }
    TIPO_RELEASE {
        es: "la tarea {id} es de tipo release: una release no se empieza con start, sale de develop con tc2 release (ADR F1-3)",
        en: "task {id} is of type release: a release is not started with start, it comes from develop with tc2 release (ADR F1-3)",
    }
    TOMADA {
        es: "la tarea {id} ya la lleva {login}: elige otra con tc2 next o pide que te la reasigne",
        en: "task {id} is already taken by {login}: pick another one with tc2 next or ask to have it reassigned",
    }
    PAUSADA {
        es: "la tarea {id} esta en pausa: reanudala con tc2 reanudar {id} antes de empezarla",
        en: "task {id} is paused: resume it with tc2 reanudar {id} before starting it",
    }
    DEPENDENCIAS_PENDIENTES {
        es: "la tarea {id} depende de tareas sin terminar ({ids}): terminalas antes o elige otra con tc2 next",
        en: "task {id} depends on unfinished tasks ({ids}): finish them first or pick another one with tc2 next",
    }
    AVISO_SOLAPE {
        es: "aviso: {id} ({quien}) toca {que}: hablad antes de seguir (solo es un aviso; start ya se hizo)",
        en: "warning: {id} ({quien}) touches {que}: talk before going on (only a warning; start is done)",
    }
    SOLAPE_FICHEROS {
        es: "los mismos ficheros ({ficheros})",
        en: "the same files ({ficheros})",
    }
    SOLAPE_MODULOS {
        es: "los mismos modulos ({modulos})",
        en: "the same modules ({modulos})",
    }
    SOLAPE_FICHEROS_Y_MODULOS {
        es: "los mismos ficheros ({ficheros}) y modulos ({modulos})",
        en: "the same files ({ficheros}) and modules ({modulos})",
    }
    NOTA_EN_CURSO {
        es: "nota: tu rama aun no cambia nada; en curso con rama publicada: {tareas}",
        en: "note: your branch changes nothing yet; in progress with a published branch: {tareas}",
    }
    AVISO_Y_MAS {
        es: "aviso: y {n} tareas mas tocan lo mismo (todas en la clave solape)",
        en: "warning: and {n} more tasks touch the same (all of them in the solape key)",
    }
    NOTA_Y_MAS {
        es: "{tareas}; y {n} mas",
        en: "{tareas}; and {n} more",
    }
}
