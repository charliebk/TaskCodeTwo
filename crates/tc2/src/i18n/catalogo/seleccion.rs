//! Mensajes de `tc2 revisoras` (F6-3).

crate::i18n::textos! {
    USO_REVISORAS {
        es: "uso: revisoras [--json] --rutas <fichero|-> [--tarea TASK-NNN] [--carril <carril>] [--complejidad <nivel>] [--plugin <nombre>=<carpeta>]... [--informe <fichero> --seccion <salida> [--artefacto <nombre>]] [--detalle <salida.json>]",
        en: "usage: revisoras [--json] --rutas <file|-> [--tarea TASK-NNN] [--carril <lane>] [--complejidad <level>] [--plugin <name>=<folder>]... [--informe <file> --seccion <output> [--artefacto <name>]] [--detalle <output.json>]",
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
