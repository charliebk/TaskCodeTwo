//! Mensajes de `tc2 review` (F6-2).

crate::i18n::textos! {
    USO_REVIEW {
        es: "uso: review [--json] TASK-NNN",
        en: "usage: review [--json] TASK-NNN",
    }
    DE_OTRA_PERSONA {
        es: "la tarea {id} la lleva {login}: solo quien la lleva la pasa a revision",
        en: "task {id} is taken by {login}: only whoever has it sends it to review",
    }
    SIN_RAMA {
        es: "la tarea {id} no tiene rama: empiezala con tc2 start {id}",
        en: "task {id} has no branch: start it with tc2 start {id}",
    }
    CARRIL_NO_SOPORTADO {
        es: "la rama {rama} de la tarea {id} no va contra develop: review solo abre PR contra develop por ahora (los hotfix llegan con F7)",
        en: "branch {rama} of task {id} does not target develop: review only opens PRs against develop for now (hotfixes come with F7)",
    }
    SIN_WORKTREE {
        es: "la tarea {id} no tiene su worktree en {ruta} con la rama {rama}: lanza tc2 start {id} para recrearlo y repite",
        en: "task {id} has no worktree at {ruta} on branch {rama}: run tc2 start {id} to recreate it and try again",
    }
    SIN_ORIGIN {
        es: "el repo no tiene remoto origin: review empuja la rama y abre el PR en la plataforma de origin",
        en: "the repo has no origin remote: review pushes the branch and opens the PR on the origin platform",
    }
}
