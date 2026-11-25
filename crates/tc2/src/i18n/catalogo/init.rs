//! Mensajes de `tc2 init` (F3-1).

crate::i18n::textos! {
    USO_INIT {
        es: "uso: init [--simular] [--aplicar-puertas] [--json]",
        en: "usage: init [--simular] [--aplicar-puertas] [--json]",
    }
    PUERTAS_SIN_PLATAFORMA {
        es: "--aplicar-puertas necesita hablar con la plataforma como persona y no puede ({motivo}). No se ha tocado nada: resuelvelo (remoto origin, TC2_TOKEN_PERSONA en un terminal, la clave plataforma) y repite tc2 init --aplicar-puertas",
        en: "--aplicar-puertas needs to talk to the platform as the person and cannot ({motivo}). Nothing was touched: fix it (origin remote, TC2_TOKEN_PERSONA in a terminal, the plataforma key) and run tc2 init --aplicar-puertas again",
    }
    PUERTAS_SIN_PERSONAS {
        es: "--aplicar-puertas exige que {ruta} ya exista y este versionado con tu cuenta. No se ha tocado nada: ejecuta tc2 init sin la opcion, versiona {ruta} y repite tc2 init --aplicar-puertas",
        en: "--aplicar-puertas requires {ruta} to exist already and be committed with your account. Nothing was touched: run tc2 init without the option, commit {ruta} and run tc2 init --aplicar-puertas again",
    }
    PUERTAS_RAMAS_SIN_REMOTO {
        es: "--aplicar-puertas exige que {ramas} ya esten en origin: con las puertas puestas ya no se podrian subir con un push. No se ha tocado nada: publicalas (git push origin develop main) y repite tc2 init --aplicar-puertas",
        en: "--aplicar-puertas requires {ramas} to be on origin already: once the gates are in place they could no longer be pushed. Nothing was touched: publish them (git push origin develop main) and run tc2 init --aplicar-puertas again",
    }
    PUERTAS_RAMAS_POR_DELANTE {
        es: "{ramas} tienen en este clon commits que origin no tiene (o falta traerlos): con las puertas puestas ese push se rechazaria. No se ha tocado nada: ejecuta git fetch origin, sube lo que falte y repite tc2 init --aplicar-puertas",
        en: "{ramas} have commits in this clone that origin does not have (or they still need fetching): once the gates are in place that push would be rejected. Nothing was touched: run git fetch origin, push what is missing and run tc2 init --aplicar-puertas again",
    }
    PUERTAS_SIN_REVISOR_ID {
        es: "los checks {faltan} no tienen origen: sin el, cualquiera con permiso de escribir checks los daria por buenos. No se ha tocado nada: anade plataforma.revisor_id en {ruta} (el app_id de la App del revisor en GitHub o su user.id en GitLab) y repite tc2 init --aplicar-puertas",
        en: "the checks {faltan} have no origin: without it, anyone allowed to write checks could pass them. Nothing was touched: add plataforma.revisor_id to {ruta} (the app_id of the reviewer App on GitHub or its user.id on GitLab) and run tc2 init --aplicar-puertas again",
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
    CAMBIO_DESDE_EL_PLAN {
        es: "{ruta} cambio despues de hacer el plan",
        en: "{ruta} changed after the plan was made",
    }
    AVISO_SIN_RELACION {
        es: "aviso: la historia de este clon no tiene nada en comun con la de origin; init no propone publicar ninguna rama de codigo (la rama de estado {rama}, huerfana, si se publica): decide tu si este clon es de ese remoto",
        en: "warning: this clone's history has nothing in common with origin's; init does not suggest pushing any code branch (the state branch {rama}, an orphan, is pushed): decide yourself whether this clone belongs to that remote",
    }
    AVISO_NO_COMPROBADO {
        es: "aviso: no se pudo comprobar si la historia de este clon tiene algo en comun con la de origin (faltan sus commits en el clon); init no propone publicar ninguna rama de codigo (la rama de estado {rama}, huerfana, si se publica): ejecuta git fetch origin y repite tc2 init",
        en: "warning: could not check whether this clone's history has anything in common with origin's (its commits are missing from the clone); init does not suggest pushing any code branch (the state branch {rama}, an orphan, is pushed): run git fetch origin and run tc2 init again",
    }
    AVISO_PERSONA {
        es: "{n} acciones las haces tu (decision=persona): cada linea trae la orden",
        en: "{n} actions are yours to do (decision=persona): each line has the command",
    }
}
