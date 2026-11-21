//! Mensajes de `src/config/`.

crate::i18n::textos! {
    // --- Diagnostico de `config rechazada` (mod.rs) ---
    DIAGNOSTICO {
        es: "{ruta}: {mensaje}{extra}. Arreglo: {arreglo}",
        en: "{ruta}: {mensaje}{extra}. Fix: {arreglo}",
    }
    DIAGNOSTICO_CON_CLAVE {
        es: "{ruta}: {clave}: {mensaje}{extra}. Arreglo: {arreglo}",
        en: "{ruta}: {clave}: {mensaje}{extra}. Fix: {arreglo}",
    }
    MAS_ERRORES {
        es: " (y {n} error(es) mas)",
        en: " (and {n} more error(s))",
    }
    ARREGLO_SIN_CONFIG {
        es: "crea la configuracion con tc2 init",
        en: "create the configuration with tc2 init",
    }
    ARREGLO_FORMATO {
        es: "es un proyecto de la 0.16: migralo con tc2 actualizar",
        en: "it is a 0.16 project: migrate it with tc2 actualizar",
    }
    ARREGLO_FORMATO_NUEVO {
        es: "el proyecto es de un tc2 mas nuevo: actualiza tc2",
        en: "the project is from a newer tc2: update tc2",
    }
    ARREGLO_YAML {
        es: "corrige el YAML de {ruta}",
        en: "fix the YAML in {ruta}",
    }
    ARREGLO_CLAVES_016 {
        es: "quedan claves de la 0.16: tc2 actualizar las traduce",
        en: "0.16 keys remain: tc2 actualizar translates them",
    }
    ARREGLO_ESQUEMA {
        es: "corrige {ruta} (esquema: crates/tc2/esquemas/config.schema.json)",
        en: "fix {ruta} (schema: crates/tc2/esquemas/config.schema.json)",
    }

    // --- Lectura del documento (mod.rs) ---
    PRIMERA_LINEA_SIN_BOM {
        es: "la primera linea tiene que ser --- (sin BOM)",
        en: "the first line must be --- (without BOM)",
    }
    PRIMERA_LINEA {
        es: "la primera linea tiene que ser ---",
        en: "the first line must be ---",
    }
    YAML_NO_VALIDO {
        es: "YAML no valido: {error}",
        en: "invalid YAML: {error}",
    }
    RAIZ_NO_MAPA {
        es: "la raiz tiene que ser un mapa",
        en: "the root must be a map",
    }
    FORMATO_NUEVO {
        es: "formato {formato}: este tc2 solo lee el formato 2",
        en: "formato {formato}: this tc2 only reads format 2",
    }
    SE_ESPERABA {
        es: "se esperaba {esperado}, no {valor}",
        en: "expected {esperado}, not {valor}",
    }
    FALTA_CLAVE {
        es: "falta la clave obligatoria",
        en: "the required key is missing",
    }
    ERROR_INTERNO {
        es: "error interno al construir la configuracion validada: {error}",
        en: "internal error building the validated configuration: {error}",
    }
    SIN_BASE {
        es: "no existe {base}: haz fetch o crea la rama develop en el remoto",
        en: "{base} does not exist: fetch or create the develop branch on the remote",
    }
    NO_HAY_CONFIG {
        es: "no hay {ruta} en {base}",
        en: "there is no {ruta} in {base}",
    }
    NO_HAY_CONFIG_EN_ARBOL {
        es: "no hay {ruta} en el arbol de trabajo",
        en: "there is no {ruta} in the working tree",
    }

    // --- Validacion contra la tabla (validar.rs) ---
    ENTERO_EN_RANGO {
        es: "se esperaba un entero de {min} a {max}, no {valor}",
        en: "expected an integer from {min} to {max}, not {valor}",
    }
    ENTERO_DESDE {
        es: "se esperaba un entero de {min} o mas, no {valor}",
        en: "expected an integer of {min} or more, not {valor}",
    }
    SOLO_ADMITE {
        es: "solo admite {valor}",
        en: "only accepts {valor}",
    }
    SE_ESPERABA_BOOLEANO {
        es: "se esperaba true o false, no {valor}",
        en: "expected true or false, not {valor}",
    }
    SE_ESPERABA_TEXTO {
        es: "se esperaba un texto, no {valor}",
        en: "expected a text, not {valor}",
    }
    VALOR_NO_ADMITIDO {
        es: "valor no admitido «{valor}» (admite: {admite})",
        en: "value not accepted «{valor}» (accepts: {admite})",
    }
    SE_ESPERABA_LISTA {
        es: "se esperaba una lista, no {valor}",
        en: "expected a list, not {valor}",
    }
    LISTA_CORTA {
        es: "se esperaba una lista con al menos {min} elemento(s)",
        en: "expected a list with at least {min} element(s)",
    }
    PRIMER_ELEMENTO {
        es: "el primer elemento tiene que ser «{valor}»",
        en: "the first element must be «{valor}»",
    }
    REPETIDO {
        es: "«{valor}» repetido",
        en: "«{valor}» repeated",
    }
    SE_ESPERABA_MAPA {
        es: "se esperaba un mapa, no {valor}",
        en: "expected a map, not {valor}",
    }
    NO_ENCAJA {
        es: "no encaja con ninguna forma admitida: {formas}",
        en: "does not match any accepted form: {formas}",
    }
    FORMA_O_FORMA {
        es: "{a} o {b}",
        en: "{a} or {b}",
    }
    UN_MAPA_CON {
        es: "un mapa con {claves}",
        en: "a map with {claves}",
    }
    LISTA_QUE_EMPIEZA {
        es: "una lista que empieza por {primero}",
        en: "a list starting with {primero}",
    }
    UNA_LISTA {
        es: "una lista",
        en: "a list",
    }
    UN_MAPA {
        es: "un mapa",
        en: "a map",
    }
    UN_VALOR {
        es: "un valor",
        en: "a value",
    }
    NADA {
        es: "nada",
        en: "nothing",
    }
    OBLIGATORIA_EN_GENERA {
        es: "obligatoria en un derivado de tipo genera",
        en: "required in a derivado of tipo genera",
    }
    PROHIBIDA_EN_COMPRUEBA {
        es: "prohibida en un derivado de tipo comprueba",
        en: "forbidden in a derivado of tipo comprueba",
    }
    CLAVE_NO_ADMITIDA_016 {
        es: "clave no admitida: es de la configuracion de la 0.16 (tc2 actualizar la traduce)",
        en: "key not accepted: it belongs to the 0.16 configuration (tc2 actualizar translates it)",
    }
    CLAVE_NO_ADMITIDA {
        es: "clave no admitida",
        en: "key not accepted",
    }

    // --- Reglas de texto (validar.rs) ---
    TEXTO_NO_NOMBRE {
        es: "texto vacio, con espacios en los extremos o con caracteres de control",
        en: "empty text, with spaces at the ends or with control characters",
    }
    ORDEN_VACIA {
        es: "orden vacia",
        en: "empty command",
    }
    ORDEN_CON_SALTOS {
        es: "la orden tiene saltos de linea: escribela en una sola linea",
        en: "the command has line breaks: write it on a single line",
    }
    ORDEN_CON_CONTROL {
        es: "la orden tiene un caracter de control (¿un \\v de una ruta de Windows entre comillas dobles? escribela entre comillas simples o con /)",
        en: "the command has a control character (a \\v from a Windows path in double quotes? write it in single quotes or with /)",
    }
    GLOB_NO_ADMITIDO {
        es: "glob no admitido «{glob}»: {motivo}",
        en: "glob not accepted «{glob}»: {motivo}",
    }
    NUNCA_TODO_EL_ARBOL {
        es: "nunca todo el arbol",
        en: "never the whole tree",
    }
    VACIO {
        es: "vacio",
        en: "empty",
    }
    CARACTER_DE_CONTROL {
        es: "caracter de control",
        en: "control character",
    }
    SIN_NEGACIONES {
        es: "sin negaciones",
        en: "no negations",
    }
    SIN_RUTAS_ABSOLUTAS {
        es: "sin rutas absolutas",
        en: "no absolute paths",
    }
    SIN_PUNTO_PUNTO {
        es: "sin ..",
        en: "no ..",
    }
    RUTA_NO_ADMITIDA {
        es: "ruta no admitida «{ruta}»: {motivo}",
        en: "path not accepted «{ruta}»: {motivo}",
    }
    SIN_BARRA_NI_DOS_PUNTOS {
        es: "sin \\ ni :",
        en: "no \\ or :",
    }
    SIN_COMODINES {
        es: "sin comodines",
        en: "no wildcards",
    }
    SALE_DE_LA_RAIZ {
        es: "sale de la raiz",
        en: "goes outside the root",
    }
    SEGMENTO_WINDOWS {
        es: "sin punto ni espacio al final de un segmento ni nombres cortos con ~ (Windows los ignora o los acorta)",
        en: "no dot or space at the end of a segment and no short names with ~ (Windows ignores or shortens them)",
    }
    NOMBRE_RESERVADO_WINDOWS {
        es: "nombre reservado de Windows",
        en: "reserved Windows name",
    }
    NUNCA_GIT {
        es: "nunca .git",
        en: "never .git",
    }
    ROADMAP_RESERVADA {
        es: "tareas/ROADMAP.md es reservada",
        en: "tareas/ROADMAP.md is reserved",
    }
    CARPETAS_DE_TAREA_RESERVADAS {
        es: "las carpetas tareas/TASK-NNN/ son reservadas",
        en: "the tareas/TASK-NNN/ folders are reserved",
    }
    NUNCA_TAREAS_ENTERA {
        es: "nunca tareas/ entera",
        en: "never the whole tareas/",
    }
    BAJO_TAREAS {
        es: "tiene que ir bajo tareas/",
        en: "it must be under tareas/",
    }
    CRON_NO_ADMITIDO {
        es: "cron no admitido «{cron}»: se esperan 5 campos (minuto 0-59, hora 0-23, dia 1-31, mes 1-12, dia de la semana 0-7) con *, a, a-b, ',' y /n",
        en: "cron not accepted «{cron}»: 5 fields are expected (minute 0-59, hour 0-23, day 1-31, month 1-12, day of the week 0-7) with *, a, a-b, ',' and /n",
    }
    URL_NO_ADMITIDA {
        es: "URL no admitida: {motivo}",
        en: "URL not accepted: {motivo}",
    }

    // --- Lo que exige cada regla (claves.rs). Solo va al esquema JSON
    // publicado, que queda en espanol; estan aqui para que no haya textos
    // de regla fuera del catalogo. ---
    REGLA_NOMBRE {
        es: "texto no vacio, sin caracteres de control ni espacios en los extremos",
        en: "non-empty text, without control characters or spaces at the ends",
    }
    REGLA_ORDEN {
        es: "orden no vacia y sin caracteres de control (rutas de Windows entre comillas simples o con /)",
        en: "non-empty command without control characters (Windows paths in single quotes or with /)",
    }
    REGLA_GLOB {
        es: "glob relativo: sin negacion (!), rutas absolutas ni ..",
        en: "relative glob: no negation (!), absolute paths or ..",
    }
    REGLA_GLOB_ACOTADO {
        es: "glob relativo: sin negacion (!), rutas absolutas ni .., y nunca todo el arbol (ningun segmento solo de * y .)",
        en: "relative glob: no negation (!), absolute paths or .., and never the whole tree (no segment made only of * and .)",
    }
    REGLA_RUTA_TAREAS {
        es: "ruta normalizada bajo tareas/, sin \\, :, comodines, caracteres de control, nombres reservados de Windows, .git, ~ ni punto o espacio al final de un segmento; nunca tareas/ entera, tareas/ROADMAP.md ni tareas/TASK-NNN/ (sin distinguir mayusculas)",
        en: "normalized path under tareas/, without \\, :, wildcards, control characters, reserved Windows names, .git, ~ or a dot or space at the end of a segment; never the whole tareas/, tareas/ROADMAP.md or tareas/TASK-NNN/ (case-insensitive)",
    }
    REGLA_CRON {
        es: "expresion cron de 5 campos con sus rangos (minuto, hora, dia, mes, dia de la semana)",
        en: "5-field cron expression with its ranges (minute, hour, day, month, day of the week)",
    }
    REGLA_URL {
        es: "URL https de la API (o http://127.0.0.1:<puerto> en pruebas)",
        en: "https URL of the API (or http://127.0.0.1:<puerto> in tests)",
    }
}
