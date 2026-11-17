//! Mensajes de `tc2 init` (F3-1).

crate::i18n::textos! {
    USO_INIT {
        es: "uso: init [--simular] [--json]",
        en: "usage: init [--simular] [--json]",
    }
    PROYECTO_016 {
        es: "{ruta} es de TaskCode 0.16: init no lo pisa. No se ha tocado nada; para pasar el proyecto a la 2.0: {arreglo}",
        en: "{ruta} belongs to TaskCode 0.16: init does not overwrite it. Nothing was touched; to move the project to 2.0: {arreglo}",
    }
    CONFIG_NO_VALE {
        es: "{ruta} existe y no vale ({motivo}): init no la pisa. No se ha tocado nada; arreglala (tc2 config la valida) y repite",
        en: "{ruta} exists and is not valid ({motivo}): init does not overwrite it. Nothing was touched; fix it (tc2 config validates it) and try again",
    }
    PERSONAS_NO_VALE {
        es: "{ruta} existe y no vale: init no la pisa. No se ha tocado nada; arreglala y repite",
        en: "{ruta} exists and is not valid: init does not overwrite it. Nothing was touched; fix it and try again",
    }
    BLOQUEADA {
        es: "no se ha tocado nada: {ruta} ({motivo}). Resuelvelo y repite tc2 init",
        en: "nothing was touched: {ruta} ({motivo}). Resolve it and run tc2 init again",
    }
    SIN_RED {
        es: "init necesita hablar con origin y no pudo: {error}. No se ha tocado nada: repite con red",
        en: "init needs to talk to origin and could not: {error}. Nothing was touched: try again with a network",
    }
    DENEGADA {
        es: "origin no deja leer ({motivo}): {error}. No se ha tocado nada: revisa la URL y las credenciales y repite",
        en: "origin does not allow reading ({motivo}): {error}. Nothing was touched: check the URL and the credentials and try again",
    }
    FALLO_AL_APLICAR {
        es: "no se pudo crear {ruta}: {error}. Lo de las lineas anteriores ya esta hecho: arregla la causa y repite tc2 init",
        en: "could not create {ruta}: {error}. What the previous lines list is already done: fix the cause and run tc2 init again",
    }
    NO_SE_LEE {
        es: "no se puede leer {ruta}: {error}",
        en: "cannot read {ruta}: {error}",
    }
    CAMBIO_DESDE_EL_PLAN {
        es: "{ruta} cambio despues de hacer el plan",
        en: "{ruta} changed after the plan was made",
    }
    AVISO_PERSONA {
        es: "{n} acciones las haces tu (decision=persona): cada linea trae la orden",
        en: "{n} actions are yours to do (decision=persona): each line has the command",
    }
}
