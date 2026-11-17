//! Mensajes de `tc2 start` (F3-6).

crate::i18n::textos! {
    NO_EXISTE {
        es: "la tarea {id} no existe en la rama de estado: comprueba el ID con tc2 status",
        en: "task {id} does not exist in the state branch: check the ID with tc2 status",
    }
    FORMATO_1 {
        es: "la tarea {id} esta en el formato de la 0.16: conviertela con tc2 actualizar antes de empezarla",
        en: "task {id} is in the 0.16 format: convert it with tc2 actualizar before starting it",
    }
    ILEGIBLE {
        es: "el tarea.md de {id} no se entiende ({error}): corrigelo antes de empezarla",
        en: "the tarea.md of {id} cannot be read ({error}): fix it before starting it",
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
