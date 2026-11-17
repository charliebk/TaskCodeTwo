//! Mensajes de `tc2 board` y `tc2 status` (F3-5).

crate::i18n::textos! {
    USO_STATUS {
        es: "uso: status [--json]",
        en: "usage: status [--json]",
    }
    USO_BOARD {
        es: "uso: board [--sin-plataforma] [--json]",
        en: "usage: board [--sin-plataforma] [--json]",
    }
    AVISO_PLATAFORMA_NO_CONFIGURADA {
        es: "aviso: no se consultaron los PR: no se reconoce la plataforma de origin; configura la clave plataforma o usa --sin-plataforma",
        en: "warning: pull requests were not read: the platform of origin is not recognised; set the plataforma key or use --sin-plataforma",
    }
    AVISO_PLATAFORMA_SIN_CREDENCIALES {
        es: "aviso: no se consultaron los PR: falta un token que valga (TC2_TOKEN_AGENTE, o TC2_TOKEN_PERSONA en un terminal)",
        en: "warning: pull requests were not read: no valid token (TC2_TOKEN_AGENTE, or TC2_TOKEN_PERSONA in a terminal)",
    }
    AVISO_PLATAFORMA_SIN_RED {
        es: "aviso: no se consultaron los PR: sin red; el estado de las tareas es el que conoce este clon",
        en: "warning: pull requests were not read: no network; task state is the one this clone knows",
    }
    AVISO_PLATAFORMA_ERROR {
        es: "aviso: no se pudieron leer los PR de la plataforma; el estado de las tareas sale completo",
        en: "warning: the platform pull requests could not be read; task state is shown in full",
    }
    AVISO_REVISION_SIN_PR {
        es: "aviso: en revision sin PR abierto listo: {ids}",
        en: "warning: in review without an open ready pull request: {ids}",
    }
    AVISO_TAREAS_CON_CAMBIOS {
        es: "aviso: {carpeta}/ tiene cambios sin publicar y no esta en lo publicado; la siguiente escritura no podra adelantarla: publicalos o deshazlos",
        en: "warning: {carpeta}/ has unpublished changes and is behind what was published; the next write will not be able to fast-forward it: publish or undo them",
    }
    AVISO_TAREAS_DIVERGE {
        es: "aviso: la rama local {rama} tiene commits que no estan en lo publicado; {carpeta}/ no se adelanta: revisalos",
        en: "warning: the local branch {rama} has commits that are not in what was published; {carpeta}/ is not fast-forwarded: review them",
    }
    AVISO_PR_SIN_REVISION {
        es: "aviso: con PR abierto listo y sin pasar a revision: {ids}",
        en: "warning: with an open ready pull request but not in review: {ids}",
    }
}
