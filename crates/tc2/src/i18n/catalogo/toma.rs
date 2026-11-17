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
    ASIGNADO_HEREDADO {
        es: "la tarea {id} tiene un asignado_a heredado de la 0.16 ({texto}): hay que reasignarla a una persona de la lista antes de empezarla",
        en: "task {id} has an assignee inherited from 0.16 ({texto}): it has to be reassigned to a person on the list before starting it",
    }
    TERMINADA {
        es: "la tarea {id} ya esta terminada: no se puede empezar",
        en: "task {id} is already finished: it cannot be started",
    }
    DESCARTADA {
        es: "la tarea {id} esta descartada: no se puede empezar",
        en: "task {id} is discarded: it cannot be started",
    }
    ESTADO_NO_VALIDO {
        es: "la tarea {id} esta {estado}: start solo empieza tareas planificadas o en diseno",
        en: "task {id} is {estado}: start only starts planned or in-design tasks",
    }
    DEPENDENCIAS_PENDIENTES {
        es: "la tarea {id} depende de tareas sin terminar ({ids}): terminalas antes o elige otra con tc2 next",
        en: "task {id} depends on unfinished tasks ({ids}): finish them first or pick another one with tc2 next",
    }
}
