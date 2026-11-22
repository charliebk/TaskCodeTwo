//! Mensajes de `tc2 revisoras` (F6-3) y `tc2 seleccion` (F4-6).

crate::i18n::textos! {
    USO_REVISORAS {
        es: "uso: revisoras [--json] --rutas <fichero|-> [--nul] [--tarea TASK-NNN] [--carril <carril>] [--complejidad <nivel>] [--plugin <nombre>=<carpeta>]... [--informe <fichero> --seccion <salida> [--artefacto <nombre>]] [--ronda <n>] [--detalle <salida.json>]",
        en: "usage: revisoras [--json] --rutas <file|-> [--nul] [--tarea TASK-NNN] [--carril <lane>] [--complejidad <level>] [--plugin <name>=<folder>]... [--informe <file> --seccion <output> [--artefacto <name>]] [--ronda <n>] [--detalle <output.json>]",
    }
    USO_SELECCION {
        es: "uso: seleccion [--json] TASK-NNN | --tarea-fichero <tarea.md> [--rutas <fichero>]; da la tarea por su ID (rama de estado) o por fichero",
        en: "usage: seleccion [--json] TASK-NNN | --tarea-fichero <tarea.md> [--rutas <file>]; give the task by its ID (state branch) or by file",
    }
    NO_SE_LEE {
        es: "no se puede leer ni escribir {ruta}: {error}",
        en: "cannot read or write {ruta}: {error}",
    }
    TABLA_ILEGIBLE {
        es: "las tablas del ADR F1-15 compiladas en tc2 no se leen: {error}",
        en: "the ADR F1-15 tables compiled into tc2 cannot be read: {error}",
    }
}
