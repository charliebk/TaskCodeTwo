//! Mensajes de la rama de estado (`src/estado/`) y del alta (`src/alta.rs`).

crate::i18n::textos! {
    MONTA_PRIMERO {
        es: "monta la rama de estado en {carpeta}/ primero",
        en: "mount the state branch in {carpeta}/ first",
    }

    // Cerrojo local.
    CERROJO_OCUPADO_AL_MONTAR {
        es: "otro proceso de este clon esta montando o escribiendo la rama de estado: reintenta",
        en: "another process of this clone is mounting or writing the state branch: retry",
    }
    CERROJO_OCUPADO {
        es: "otro proceso de este clon esta escribiendo la rama de estado: reintenta",
        en: "another process of this clone is writing the state branch: retry",
    }
    CERROJO_ERROR {
        es: "cerrojo: {error}",
        en: "lock: {error}",
    }

    // Montaje (`montaje.rs`).
    MONTADA_EN_OTRA_RUTA {
        es: "{rama} ya esta montada en {ruta}: quita ese worktree",
        en: "{rama} is already mounted at {ruta}: remove that worktree",
    }
    CARPETA_OCUPADA {
        es: "{carpeta}/ ya existe y no es la rama de estado (¿un repo de la 0.16? migralo con actualizar)",
        en: "{carpeta}/ already exists and is not the state branch (a 0.16 repo? migrate it with actualizar)",
    }
    MONTAR_EXIGE_RED {
        es: "{error}; montar la rama de estado exige red la primera vez",
        en: "{error}; mounting the state branch needs network access the first time",
    }
    MONTAR_DENEGADA {
        es: "{error}; revisa credenciales, permisos o la URL de origin",
        en: "{error}; check credentials, permissions or the origin URL",
    }
    RAMA_SIGUE_SIN_EXISTIR {
        es: "{rama} sigue sin existir tras crearla",
        en: "{rama} still does not exist after creating it",
    }
    RUTA_TAREAS_NO_UTF8 {
        es: "la ruta de tareas/ no es UTF-8",
        en: "the tareas/ path is not valid UTF-8",
    }

    // Transaccion (`transaccion.rs`).
    AGOTADA {
        es: "la rama de estado cambio en cada intento: no se ha escrito nada; reintenta",
        en: "the state branch changed on every attempt: nothing was written; retry",
    }
    RECHAZADA_COMO_ERROR {
        es: "{motivo}: {diagnostico}",
        en: "rejected ({motivo}): {diagnostico}",
    }
    TAREAS_CON_CAMBIOS {
        es: "{carpeta}/ tiene cambios que chocan con la operacion: guardalos o deshazlos y reintenta",
        en: "{carpeta}/ has changes that clash with the operation: save or undo them and retry",
    }
    RUTA_DUPLICADA {
        es: "{nueva} solo difiere en mayusculas de {otra}: chocarian en Windows y macOS",
        en: "{nueva} differs from {otra} only in case: they would clash on Windows and macOS",
    }
    SIN_MENSAJE {
        es: "sin mensaje",
        en: "no message",
    }
    ESCRITURA_DENEGADA {
        es: "{detalle}; el remoto deniega la escritura: revisa credenciales, permisos o reglas, no se reintenta",
        en: "{detalle}; the remote denies the write: check credentials, permissions or rules, it is not retried",
    }
    EXIGE_RED {
        es: "{detalle}; esta operacion exige red: no se ha escrito nada, reintenta con red",
        en: "{detalle}; this operation needs network access: nothing was written, retry with network access",
    }
    QUEDA_PENDIENTE {
        es: "{detalle}; queda pendiente de publicar: se publicara en la siguiente operacion con red",
        en: "{detalle}; pending publication: it will be published in the next operation with network access",
    }
    NO_SE_PUDO_ADELANTAR {
        es: "no se pudo adelantar {rama}: {error}",
        en: "could not fast-forward {rama}: {error}",
    }

    // Alta (`alta.rs`).
    SIN_IDS {
        es: "no quedan IDs de tarea",
        en: "no task IDs left",
    }
    AVISO_PARECIDAS {
        es: "aviso: ya hay tareas abiertas con un titulo igual o parecido: {lista}",
        en: "warning: there are already open tasks with the same or a similar title: {lista}",
    }
    PARECIDA_DUPLICADO {
        es: "{id} \"{titulo}\" (duplicado)",
        en: "{id} \"{titulo}\" (duplicate)",
    }
    PARECIDA_CASI {
        es: "{id} \"{titulo}\" (parecido {unidad},{centesimas})",
        en: "{id} \"{titulo}\" (similarity {unidad}.{centesimas})",
    }
    RAMA_DERIVADA_INVALIDA {
        es: "la rama derivada de {id} no es valida ({motivo})",
        en: "the branch derived from {id} is not valid ({motivo})",
    }
}
