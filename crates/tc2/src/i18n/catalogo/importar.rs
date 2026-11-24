//! Mensajes de `tc2 import` y del enunciado de `tc2 new` (F3-2).

crate::i18n::textos! {
    USO_IMPORT {
        es: "uso: import [--json] [--simular] [--sprint <N>] <fichero>",
        en: "usage: import [--json] [--simular] [--sprint <N>] <file>",
    }
    NO_SE_LEE {
        es: "no se puede leer {ruta}: {error}. No se ha importado nada",
        en: "cannot read {ruta}: {error}. Nothing was imported",
    }
    NO_UTF8 {
        es: "{ruta} no es texto UTF-8. No se ha importado nada",
        en: "{ruta} is not UTF-8 text. Nothing was imported",
    }
    VACIO {
        es: "{ruta} esta vacio. No se ha importado nada",
        en: "{ruta} is empty. Nothing was imported",
    }
    FICHERO_GRANDE {
        es: "el fichero pasa de {limite} bytes: partelo en varios lotes. No se ha importado nada",
        en: "the file is larger than {limite} bytes: split it into several batches. Nothing was imported",
    }
    DEMASIADAS_ENTRADAS {
        es: "el fichero tiene mas de {limite} entradas: partelo en varios lotes. No se ha importado nada",
        en: "the file has more than {limite} entries: split it into several batches. Nothing was imported",
    }
    SIN_ENTRADAS {
        es: "el fichero no tiene ninguna entrada «### <titulo>». No se ha importado nada",
        en: "the file has no «### <title>» entry. Nothing was imported",
    }
    CABECERA_SIN_CERRAR {
        es: "la cabecera que abre «---» en la linea 1 no se cierra con otra linea «---». No se ha importado nada",
        en: "the header opened by «---» on line 1 is not closed by another «---» line. Nothing was imported",
    }
    CABECERA_INVALIDA {
        es: "linea {linea} de la cabecera: solo valen «lote: <nombre>» y «sprint: <numero>», una vez cada una. No se ha importado nada",
        en: "header line {linea}: only «lote: <name>» and «sprint: <number>» are allowed, once each. Nothing was imported",
    }
    LOTE_INVALIDO {
        es: "el lote «{lote}» no vale: letras y cifras ASCII, «.», «_» y «-», hasta 64. No se ha importado nada",
        en: "the batch name «{lote}» is not valid: ASCII letters and digits, «.», «_» and «-», up to 64. Nothing was imported",
    }
    SPRINT_EN_CONFLICTO {
        es: "la cabecera dice sprint {cabecera} y --sprint dice {opcion}: deja solo uno. No se ha importado nada",
        en: "the header says sprint {cabecera} and --sprint says {opcion}: keep only one. Nothing was imported",
    }
    LINEA_SUELTA {
        es: "linea {linea}: no es un criterio («- ...»), el objetivo («> ...»), «Dependencias: ...» ni una linea en blanco. No se ha importado nada",
        en: "line {linea}: it is not a criterion («- ...»), the goal («> ...»), «Dependencias: ...» or a blank line. Nothing was imported",
    }
    TITULO_INVALIDO {
        es: "linea {linea}: el «###» no tiene titulo o lleva caracteres de control. No se ha importado nada",
        en: "line {linea}: the «###» has no title or contains control characters. Nothing was imported",
    }
    NO_LISTO_EN_LINEA {
        es: "linea {linea}: «{titulo}» no esta listo: {causa}. No se ha importado nada",
        en: "line {linea}: «{titulo}» is not ready: {causa}. Nothing was imported",
    }
    NO_LISTO {
        es: "«{titulo}» no esta listo: {causa}. No se ha creado nada",
        en: "«{titulo}» is not ready: {causa}. Nothing was created",
    }
    CAUSA_SIN_CRITERIOS {
        es: "no tiene ningun criterio de aceptacion",
        en: "it has no acceptance criteria",
    }
    CAUSA_CRITERIO_VACIO {
        es: "tiene un criterio vacio",
        en: "it has an empty criterion",
    }
    CAUSA_PLANTILLA {
        es: "conserva un texto de la plantilla («<...>»)",
        en: "it keeps a template placeholder («<...>»)",
    }
    CAUSA_REPETIDO {
        es: "repite un criterio",
        en: "it repeats a criterion",
    }
    CAUSA_CONTROL {
        es: "lleva caracteres de control",
        en: "it contains control characters",
    }
    CAUSA_SIN_OBJETIVO {
        es: "tiene criterios pero no objetivo",
        en: "it has criteria but no goal",
    }
    CAUSA_DEMASIADOS_CRITERIOS {
        es: "tiene mas de {max} criterios de aceptacion: partela en varias tareas",
        en: "it has more than {max} acceptance criteria: split it into several tasks",
    }
    CLAVE_DUPLICADA {
        es: "linea {linea}: la clave «{clave}» ya la usa otra entrada; empieza el «###» por una clave distinta (p. ej. «### F3-2 ...»). No se ha importado nada",
        en: "line {linea}: the key «{clave}» is already used by another entry; start the «###» with a different key (e.g. «### F3-2 ...»). Nothing was imported",
    }
    REFERENCIA_INEXISTENTE {
        es: "linea {linea}: la dependencia «{referencia}» no es una clave de este fichero ni un TASK-NNN. No se ha importado nada",
        en: "line {linea}: the dependency «{referencia}» is neither a key of this file nor a TASK-NNN. Nothing was imported",
    }
    EXTERNA_INEXISTENTE {
        es: "la dependencia {referencia} de «{clave}» no existe en la rama de estado. No se ha importado nada",
        en: "the dependency {referencia} of «{clave}» does not exist in the state branch. Nothing was imported",
    }
    DEPENDENCIA_INEXISTENTE {
        es: "la dependencia {referencia} no existe en la rama de estado. No se ha creado nada",
        en: "the dependency {referencia} does not exist in the state branch. Nothing was created",
    }
    CICLO {
        es: "las dependencias forman un ciclo: {ciclo}. No se ha importado nada",
        en: "the dependencies form a cycle: {ciclo}. Nothing was imported",
    }
    VALOR_INVALIDO {
        es: "{opcion} no vale «{valor}»: {permitidos}",
        en: "{opcion} does not accept «{valor}»: {permitidos}",
    }
    ETIQUETA_INVALIDA {
        es: "--etiqueta no vale «{valor}»: una palabra sin espacios ni caracteres de control",
        en: "--etiqueta does not accept «{valor}»: one word without spaces or control characters",
    }
    USO_NEW_CAMPOS {
        es: "uso: new [--json] \"<titulo>\" [--tipo <tipo>] [--objetivo \"<texto>\"] [--criterio \"<texto>\"]... [--etiqueta <e>]... [--complejidad <c>] [--sprint <N>] [--dependencia TASK-NNN]...",
        en: "usage: new [--json] \"<title>\" [--tipo <type>] [--objetivo \"<text>\"] [--criterio \"<text>\"]... [--etiqueta <tag>]... [--complejidad <c>] [--sprint <N>] [--dependencia TASK-NNN]...",
    }
}
