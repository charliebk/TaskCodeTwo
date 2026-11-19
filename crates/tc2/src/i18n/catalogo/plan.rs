//! Mensajes de `tc2 plan` y `tc2 plan guardar` (F4-1).

crate::i18n::textos! {
    USO_PLAN {
        es: "uso: plan [--json] TASK-NNN [--carril fix|funcionalidad|riesgo-alto] | plan guardar [--json] TASK-NNN <destino> <fichero>",
        en: "usage: plan [--json] TASK-NNN [--carril fix|funcionalidad|riesgo-alto] | plan guardar [--json] TASK-NNN <destination> <file>",
    }
    DESTINO_INVALIDO {
        es: "el destino {destino} no vale: solo plan-final.md o brainstorm/salida-<rol>-<n>.md, con <rol> uno de {roles}",
        en: "destination {destino} is not valid: only plan-final.md or brainstorm/salida-<rol>-<n>.md, with <rol> one of {roles}",
    }
    DE_OTRA_PERSONA {
        es: "la tarea {id} la lleva {login}: solo ella la planifica (o pide que te la reasigne)",
        en: "task {id} is taken by {login}: only they plan it (or ask to have it reassigned)",
    }
    PAUSADA {
        es: "la tarea {id} esta en pausa: reanudala con tc2 reanudar {id} antes de planificarla",
        en: "task {id} is paused: resume it with tc2 reanudar {id} before planning it",
    }
    OBJETIVO_VACIO {
        es: "la tarea {id} no tiene objetivo (el ## Objetivo esta vacio o es el de la plantilla): escribelo antes de planificarla",
        en: "task {id} has no goal (## Objetivo is empty or the template one): write it before planning it",
    }
    SIN_CRITERIOS {
        es: "la tarea {id} no tiene ningun criterio de aceptacion (- [ ] en ## Criterios de aceptacion): escribelos antes de planificarla",
        en: "task {id} has no acceptance criteria (- [ ] in ## Criterios de aceptacion): write them before planning it",
    }
    CRITERIO_VACIO {
        es: "el criterio {n} de la tarea {id} esta vacio o es el de la plantilla: escribelo o quitalo antes de planificarla",
        en: "criterion {n} of task {id} is empty or the template one: write it or remove it before planning it",
    }
    DEMASIADOS_CRITERIOS {
        es: "la tarea {id} tiene {n} criterios (mas de {max}): es demasiado grande para una tarea; partela en varias",
        en: "task {id} has {n} criteria (more than {max}): it is too big for one task; split it into several",
    }
    CARRIL_BAJA {
        es: "el carril {pedido} esta por debajo del de la tarea {id} ({minimo}): --carril solo sube (ADR F1-10, decision 3)",
        en: "lane {pedido} is below that of task {id} ({minimo}): --carril only raises it (ADR F1-10, decision 3)",
    }
    SIN_PETICION {
        es: "no hay peticion para {destino} en la tarea {id} (falta {peticion}): guarda la salida de un rol que tc2 plan haya lanzado",
        en: "there is no request for {destino} in task {id} ({peticion} is missing): save the output of a role launched by tc2 plan",
    }
    PLAN_APROBADO {
        es: "el plan de la tarea {id} ya esta aprobado: no se cambia su planificacion",
        en: "the plan of task {id} is already approved: its planning is not changed",
    }
    NO_SE_LEE {
        es: "no se pudo leer {ruta} ({error})",
        en: "could not read {ruta} ({error})",
    }
    NO_UTF8 {
        es: "{ruta} no es texto UTF-8",
        en: "{ruta} is not UTF-8 text",
    }
    VACIO {
        es: "{ruta} esta vacio: no se publica un fichero sin contenido",
        en: "{ruta} is empty: a file without content is not published",
    }
    GRANDE {
        es: "{ruta} pasa de {max} bytes: no se publica",
        en: "{ruta} is larger than {max} bytes: it is not published",
    }
}
