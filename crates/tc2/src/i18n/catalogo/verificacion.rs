//! Mensajes de `tc2 confiar` y `tc2 verificar` (F5-6) y de la confianza
//! local (F1-8 decision 5).

crate::i18n::textos! {
    USO_CONFIAR {
        es: "uso: confiar [--json] <clave> (se pueden confiar: {claves})",
        en: "usage: confiar [--json] <key> (keys that can be trusted: {claves})",
    }
    CLAVE_DESCONOCIDA {
        es: "tc2 no sabe confiar la clave «{clave}»: las que se pueden confiar son {claves}",
        en: "tc2 cannot trust the key «{clave}»: the keys that can be trusted are {claves}",
    }
    SIN_ORDEN {
        es: "{ruta} no declara {clave}: declarala antes de confiar en ella",
        en: "{ruta} does not declare {clave}: declare it before trusting it",
    }
    SIN_ALMACEN {
        es: "no se sabe donde guardar la confianza: define LOCALAPPDATA (Windows) o XDG_CACHE_HOME o HOME (resto)",
        en: "no place to store the trust: set LOCALAPPDATA (Windows) or XDG_CACHE_HOME or HOME (others)",
    }
    ALMACEN_NO_ESCRITO {
        es: "no se pudo escribir la confianza en {ruta}: {error}",
        en: "could not write the trust to {ruta}: {error}",
    }
    CIERRE_NO_LEIDO {
        es: "no se pudo leer un fichero de lo que ejecuta la orden: {error}",
        en: "could not read a file of what the command runs: {error}",
    }
    GLOB_INVALIDO {
        es: "el glob «{glob}» de confianza_incluye no es valido ({error}) y no se confia ni se ejecuta nada: corrigelo en {ruta}",
        en: "the glob «{glob}» in confianza_incluye is not valid ({error}) and nothing is trusted or run: fix it in {ruta}",
    }
    GLOBS_SIN_FICHEROS {
        es: "aviso: estos globs de confianza_incluye no casan con ningun fichero versionado y no suman nada a la confianza: {globs}; revisalos en {ruta}",
        en: "warning: these confianza_incluye globs match no tracked file and add nothing to the trust: {globs}; check them in {ruta}",
    }
    USO_VERIFICAR {
        es: "uso: verificar [--json] --perfil <rapida|impacto|completa|pesada> [--hook]",
        en: "usage: verificar [--json] --perfil <rapida|impacto|completa|pesada> [--hook]",
    }
    SIN_VERIFICACION {
        es: "{ruta} no declara verificacion: declara verificacion.comando, verificacion.shell y sus perfiles",
        en: "{ruta} does not declare verificacion: declare verificacion.comando, verificacion.shell and its profiles",
    }
    SIN_PERFIL {
        es: "{ruta} no declara el perfil {perfil}: anadelo en verificacion.perfiles con su timeout",
        en: "{ruta} does not declare the profile {perfil}: add it under verificacion.perfiles with its timeout",
    }
    NUNCA_CONFIADA {
        es: "la orden de {clave} no esta confiada en este repo y no se ejecuta: revisa la orden y los scripts que lanza y, si te fias, ejecuta `tc2 confiar {clave}`",
        en: "the command of {clave} is not trusted in this repo and is not run: review the command and the scripts it launches and, if you trust them, run `tc2 confiar {clave}`",
    }
    CAMBIADA {
        es: "la orden de {clave} o un script que ejecuta cambio desde que confiaste en ella y no se ejecuta: revisa el cambio y, si te fias, ejecuta `tc2 confiar {clave}`",
        en: "the command of {clave} or a script it runs changed since you trusted it and is not run: review the change and, if you trust it, run `tc2 confiar {clave}`",
    }
    NO_SE_LANZA {
        es: "no se pudo lanzar la shell {shell}: {error}; comprueba que esta en el PATH o cambia verificacion.shell",
        en: "could not launch the shell {shell}: {error}; check that it is on the PATH or change verificacion.shell",
    }
    ROJO {
        es: "la verificacion {perfil} salio en rojo (codigo {codigo}): mira su salida y corrige lo que falle",
        en: "the {perfil} verification is red (exit code {codigo}): read its output and fix what fails",
    }
    TIMEOUT {
        es: "la verificacion {perfil} agoto su timeout de {timeout} s y cuenta como rojo: acelera la suite o sube verificacion.perfiles.{perfil}.timeout",
        en: "the {perfil} verification ran out of its {timeout} s timeout and counts as red: speed up the suite or raise verificacion.perfiles.{perfil}.timeout",
    }
    HOOK_NO_SE_EJECUTA {
        es: "tc2 verificar --hook: no se ejecuta la verificacion {perfil}: {motivo}",
        en: "tc2 verificar --hook: the {perfil} verification is not run: {motivo}",
    }
    HOOK_MAL_ESCRITO {
        es: "tc2 verificar --hook: la entrada del hook esta mal escrita y no se ejecuta nada: {motivo}; corrigela en settings.json (`tc2 verificar --perfil rapida --hook`)",
        en: "tc2 verificar --hook: the hook entry is malformed and nothing is run: {motivo}; fix it in settings.json (`tc2 verificar --perfil rapida --hook`)",
    }
    HOOK_ROJO {
        es: "tc2 verificar --hook: {motivo}. Ultimas lineas de su salida:",
        en: "tc2 verificar --hook: {motivo}. Last lines of its output:",
    }
}
