//! Mensajes del borde del CLI (`main.rs` y `lib.rs`).

crate::i18n::textos! {
    ARGUMENTO_NO_UTF8 {
        es: "tc2: argumento que no es UTF-8",
        en: "tc2: argument is not valid UTF-8",
    }
    NO_SE_PUDO_ESCRIBIR {
        es: "tc2: no se pudo escribir la salida: {error}",
        en: "tc2: could not write the output: {error}",
    }
    USO {
        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <carril>] | tc2 plan guardar [--json] TASK-NNN <destino> <fichero> | tc2 approve [--json] TASK-NNN [--pedir-cambios <fichero>] | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <lane>] | tc2 plan guardar [--json] TASK-NNN <destination> <file> | tc2 approve [--json] TASK-NNN [--pedir-cambios <file>] | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
    }
    FALTA_LA_ORDEN {
        es: "tc2: falta la orden. {uso}",
        en: "tc2: missing command. {uso}",
    }
    ORDEN_DESCONOCIDA {
        es: "tc2: orden desconocida «{orden}». {uso}",
        en: "tc2: unknown command «{orden}». {uso}",
    }
    CARPETA_ACTUAL {
        es: "no se puede leer la carpeta actual: {error}",
        en: "cannot read the current folder: {error}",
    }
    USO_GUARDAR {
        es: "uso: guardar \"<mensaje>\" <rutas...>",
        en: "usage: guardar \"<message>\" <paths...>",
    }
    USO_ACTUALIZAR_RAMA {
        es: "uso: actualizar-rama [--json]",
        en: "usage: actualizar-rama [--json]",
    }
    TITULO_INVALIDO {
        es: "el titulo no puede estar vacio ni llevar saltos de linea, tabuladores ni otros caracteres de control",
        en: "the title cannot be empty or contain line breaks, tabs or other control characters",
    }
    USO_NEW {
        es: "uso: new [--json] \"<titulo>\"",
        en: "usage: new [--json] \"<title>\"",
    }
    TITULO_ENTRE_COMILLAS {
        es: "pon el titulo entre comillas: new [--json] \"<titulo>\"",
        en: "put the title in quotes: new [--json] \"<title>\"",
    }
}
