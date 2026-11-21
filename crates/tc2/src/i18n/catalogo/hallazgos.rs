//! Mensajes de `tc2 hallazgos` (F6-4).

crate::i18n::textos! {
    USO_HALLAZGOS {
        es: "uso: hallazgos [--json] validar --respuesta <fichero> --diff <fichero> --revisora <id> [--informe <informe.json>] [--salida <salida.json>] [--resumen <salida.md>] | hallazgos [--json] veredicto --salida <carpeta> [--hallazgos <salida-de-validar.json>]... [--extra <lineas.txt>] | hallazgos [--json] instrucciones",
        en: "usage: hallazgos [--json] validar --respuesta <file> --diff <file> --revisora <id> [--informe <report.json>] [--salida <output.json>] [--resumen <output.md>] | hallazgos [--json] veredicto --salida <folder> [--hallazgos <validar-output.json>]... [--extra <lines.txt>] | hallazgos [--json] instrucciones",
    }
    NO_SE_LEE {
        es: "no se puede leer ni escribir {ruta}: {error}; comprueba la ruta y los permisos y vuelve a lanzar la orden",
        en: "cannot read or write {ruta}: {error}; check the path and permissions and run the command again",
    }
    DEMASIADO_GRANDE {
        es: "{ruta} pasa de {tope} bytes; recorta la entrada (el workflow corta la respuesta a 64 KiB y el diff a tope-diff)",
        en: "{ruta} is larger than {tope} bytes; trim the input (the workflow cuts the response to 64 KiB and the diff to tope-diff)",
    }
}
