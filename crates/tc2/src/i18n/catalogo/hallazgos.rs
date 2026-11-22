//! Mensajes de `tc2 hallazgos` (F6-4, F6-5 y F6-6).

crate::i18n::textos! {
    USO_HALLAZGOS {
        es: "uso: hallazgos [--json] validar --respuesta <fichero> --diff <fichero> --revisora <id> [--informe <informe.json>] [--abiertos <abiertos.json>] [--salida <salida.json>] [--resumen <salida.md>] [--prosa <salida.md>] | hallazgos [--json] veredicto --salida <carpeta> [--hallazgos <salida-de-validar.json>]... [--extra <lineas.txt>] [--ronda <ronda.json>] | hallazgos [--json] anterior --check-runs <json>... [--origen <app-id>] | hallazgos [--json] ronda --check-runs <json>... [--anotaciones <json>]... --rutas <fichero-nul> [--tarea TASK-NNN] [--origen <app-id>] --salida <carpeta> | hallazgos [--json] abiertos --ronda <ronda.json> --detalle <detalle.json> --salida <carpeta> | hallazgos [--json] desde --anterior <sha> --head <sha> --base <ref> | hallazgos [--json] prompt --revisora <id> --contexto <fichero> [--skill <SKILL.md>] [--seccion <seccion.md>] [--abiertos <abiertos.md>] --diff <fichero> --bytes <n> --tope <n> --salida <prompt.md> | hallazgos [--json] instrucciones",
        en: "usage: hallazgos [--json] validar --respuesta <file> --diff <file> --revisora <id> [--informe <report.json>] [--abiertos <open.json>] [--salida <output.json>] [--resumen <output.md>] [--prosa <output.md>] | hallazgos [--json] veredicto --salida <folder> [--hallazgos <validar-output.json>]... [--extra <lines.txt>] [--ronda <ronda.json>] | hallazgos [--json] anterior --check-runs <json>... [--origen <app-id>] | hallazgos [--json] ronda --check-runs <json>... [--anotaciones <json>]... --rutas <nul-file> [--tarea TASK-NNN] [--origen <app-id>] --salida <folder> | hallazgos [--json] abiertos --ronda <ronda.json> --detalle <detail.json> --salida <folder> | hallazgos [--json] desde --anterior <sha> --head <sha> --base <ref> | hallazgos [--json] prompt --revisora <id> --contexto <file> [--skill <SKILL.md>] [--seccion <section.md>] [--abiertos <open.md>] --diff <file> --bytes <n> --tope <n> --salida <prompt.md> | hallazgos [--json] instrucciones",
    }
    NO_SE_LEE {
        es: "no se puede leer ni escribir {ruta}: {error}; comprueba la ruta y los permisos y vuelve a lanzar la orden",
        en: "cannot read or write {ruta}: {error}; check the path and permissions and run the command again",
    }
    DEMASIADO_GRANDE {
        es: "{ruta} pasa de {tope} bytes; recorta la entrada (el workflow corta la respuesta a 64 KiB y el diff a tope-diff)",
        en: "{ruta} is larger than {tope} bytes; trim the input (the workflow cuts the response to 64 KiB and the diff to tope-diff)",
    }
    NO_ES_DEL_CONTRATO {
        es: "{ruta} no es {que}: {error}; pasale el fichero tal como lo dejo el paso anterior del workflow (o la API de GitHub), sin editarlo, y vuelve a lanzar la orden",
        en: "{ruta} is not {que}: {error}; pass the file exactly as the previous workflow step (or the GitHub API) left it, unedited, and run the command again",
    }
    ANTERIOR_INCOHERENTE {
        es: "el check revision-ia {check} del commit {sha} no casa con sus anotaciones (titulo, conclusion y hallazgos abiertos): o no se leyo una pagina de anotaciones, o se publico a medias (fallo un lote de anotaciones), o esta alterado. La ronda no se calcula sin el (falla en cerrado). Primero vuelve a lanzar este workflow (si fue la lectura, pasa). Si sigue, vuelve a lanzar (Re-run) el workflow revision-ia del commit {sha}: publica en ese commit un check completo, que pasa delante del roto porque de cada commit cuenta el mas nuevo que no sea incompleta; despues, vuelve a lanzar este. Si el check esta alterado, revisalo en GitHub antes de seguir",
        en: "the revision-ia check {check} on commit {sha} does not match its annotations (title, conclusion and open findings): an annotations page was not read, it was published halfway (an annotations batch failed), or it was altered. The round is not computed without it (fail closed). First re-run this workflow (if it was the read, it passes). If it persists, re-run the revision-ia workflow of commit {sha}: it publishes a complete check on that commit, which takes precedence over the broken one because on each commit the newest one that is not incompleta counts; then re-run this one. If the check was altered, inspect it on GitHub before going on",
    }
}
