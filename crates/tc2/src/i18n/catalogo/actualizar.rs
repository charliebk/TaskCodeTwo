//! Mensajes de `tc2 version`, `tc2 compatible` y `tc2 actualizar --cli`
//! (F2-12, ADR F1-14 puntos 4 y 5).

crate::i18n::textos! {
    USO_VERSION {
        es: "uso: version [--json]",
        en: "usage: version [--json]",
    }
    NO_SE_LEE_EL_EJECUTABLE {
        es: "no se puede leer el ejecutable de tc2 ({ruta}): {error}",
        en: "cannot read the tc2 executable ({ruta}): {error}",
    }
    TARGET_FUERA_DE_P12 {
        es: "este sistema ({sistema}) no es uno de los 6 de las releases de tc2: no hay binario que bajar",
        en: "this system ({sistema}) is not one of the 6 in the tc2 releases: there is no binary to download",
    }
    USO_COMPATIBLE {
        es: "uso: compatible [--json] --protocolo <A-B> (A y B enteros, A <= B)",
        en: "usage: compatible [--json] --protocolo <A-B> (A and B integers, A <= B)",
    }
    PROTOCOLO_INCOMPATIBLE {
        es: "este tc2 habla el protocolo {protocolo} y se pide {rango}: motivo=protocolo-incompatible arreglo=\"tc2 actualizar --cli\"",
        en: "this tc2 speaks protocol {protocolo} and {rango} is required: motivo=protocolo-incompatible arreglo=\"tc2 actualizar --cli\"",
    }
    USO_ACTUALIZAR {
        es: "uso: actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] (actualizar un proyecto, sin --cli, llega con F9-2)",
        en: "usage: actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] (updating a project, without --cli, comes with F9-2)",
    }
    GESTIONADO_POR_PLUGIN {
        es: "este tc2 es el del plugin ({ruta}) y lo actualiza el plugin: motivo=gestionado-por-plugin arreglo=\"actualiza el plugin taskcodetwo con /plugin en Claude Code\"",
        en: "this tc2 belongs to the plugin ({ruta}) and the plugin updates it: motivo=gestionado-por-plugin arreglo=\"update the taskcodetwo plugin with /plugin in Claude Code\"",
    }
    GESTIONADO_POR_GESTOR {
        es: "este tc2 lo ha instalado {gestor} ({ruta}): actualizalo con {gestor}, no con tc2 actualizar --cli",
        en: "this tc2 was installed by {gestor} ({ruta}): update it with {gestor}, not with tc2 actualizar --cli",
    }
    BASE_URL_INVALIDA {
        es: "TC2_BASE_URL no vale ({url}): tiene que ser https, o http a 127.0.0.1:<puerto>, sin usuario. Quitala; solo sirve para ensayos",
        en: "TC2_BASE_URL is not valid ({url}): it must be https, or http to 127.0.0.1:<port>, without a user. Remove it; it is only for rehearsals",
    }
    SIN_RED {
        es: "no se pudo bajar {url}: {error}. Comprueba la conexion o el proxy y repite",
        en: "could not download {url}: {error}. Check the connection or the proxy and retry",
    }
    NO_PUBLICADA {
        es: "{url} no existe (HTTP {estado}): comprueba la version en https://github.com/charliebk/TaskCodeTwo/releases",
        en: "{url} does not exist (HTTP {estado}): check the version at https://github.com/charliebk/TaskCodeTwo/releases",
    }
    HTTP_ESTADO {
        es: "{url} respondio HTTP {estado}; repite mas tarde",
        en: "{url} answered HTTP {estado}; retry later",
    }
    REDIRECCION_NO_PERMITIDA {
        es: "{url} redirige a un destino que no es de las releases de GitHub ({destino}): no se sigue",
        en: "{url} redirects to a destination outside the GitHub releases ({destino}): not followed",
    }
    DESCARGA_GRANDE {
        es: "{url} pasa de {limite} bytes: no es un binario de tc2",
        en: "{url} is larger than {limite} bytes: it is not a tc2 binary",
    }
    SUMAS_INVALIDAS {
        es: "{url} no es un SHA256SUMS valido: {error}",
        en: "{url} is not a valid SHA256SUMS: {error}",
    }
    SIN_BINARIO {
        es: "el SHA256SUMS de {url} no trae el binario de {target}{version}",
        en: "the SHA256SUMS at {url} has no binary for {target}{version}",
    }
    VERSION_INVALIDA {
        es: "la version tiene que ser X.Y.Z o X.Y.Z-sufijo: «{version}»",
        en: "the version must be X.Y.Z or X.Y.Z-suffix: «{version}»",
    }
    SIN_PERMISO {
        es: "no se puede escribir en {ruta}: {error}. Da permiso de escritura o reinstala con la linea de instalacion",
        en: "cannot write to {ruta}: {error}. Grant write permission or reinstall with the install line",
    }
    SHA256_NO_CASA {
        es: "el sha256 de {fichero} ({obtenido}) no casa con SHA256SUMS ({esperado}): no se ha cambiado nada. Repite; si sigue, avisa en el repositorio",
        en: "the sha256 of {fichero} ({obtenido}) does not match SHA256SUMS ({esperado}): nothing was changed. Retry; if it persists, report it in the repository",
    }
    NO_ARRANCA {
        es: "el binario descargado no responde como tc2 {version} de {target}: {detalle}. No se ha cambiado nada",
        en: "the downloaded binary does not answer as tc2 {version} for {target}: {detalle}. Nothing was changed",
    }
    NO_SE_SUSTITUYE {
        es: "no se pudo sustituir {ruta}: {error}. Queda el binario anterior; cierra los tc2 abiertos y repite",
        en: "could not replace {ruta}: {error}. The previous binary is kept; close any running tc2 and retry",
    }
    ESCRITURA {
        es: "no se pudo escribir {ruta}: {error}. Comprueba el espacio libre en disco y repite; no se ha cambiado nada",
        en: "could not write {ruta}: {error}. Check the free disk space and retry; nothing was changed",
    }
    ESCRITURA_DESCARGA {
        es: "no se pudo escribir la descarga: {error}. Comprueba el espacio libre en disco y repite",
        en: "could not write the download: {error}. Check the free disk space and retry",
    }
    ATTESTATION_NO_VALIDA {
        es: "la attestation de procedencia de {fichero} no es valida ({detalle}): no se ha cambiado nada. Avisa en https://github.com/charliebk/TaskCodeTwo/issues",
        en: "the provenance attestation of {fichero} is not valid ({detalle}): nothing was changed. Report it at https://github.com/charliebk/TaskCodeTwo/issues",
    }
}
