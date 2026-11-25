//! Mensajes de `tc2 desinstalar` (F2-14).

crate::i18n::textos! {
    USO_DESINSTALAR {
        es: "uso: desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]; --config y --borrar-tareas solo con --proyecto",
        en: "usage: desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]; --config and --borrar-tareas only with --proyecto",
    }
    NOMBRE_NO_COINCIDE {
        es: "--borrar-tareas exige el nombre exacto de la rama de estado, {rama}; se recibio «{nombre}». No se ha tocado nada",
        en: "--borrar-tareas needs the exact name of the state branch, {rama}; got «{nombre}». Nothing was touched",
    }
    HAZ_TU {
        es: "{clase} {ruta}: no lo quita tc2; quitalo tu con: {orden}",
        en: "{clase} {ruta}: tc2 does not remove it; remove it yourself with: {orden}",
    }
    AVISO_HOOK {
        es: "aviso: {ruta} tiene un hook que nombra tc2; desinstalar nunca edita los hooks: quitalo tu si ya no lo quieres",
        en: "warning: {ruta} has a hook that names tc2; desinstalar never edits hooks: remove it yourself if you no longer want it",
    }
    AVISO_ADELANTADA {
        es: "aviso: la rama local {rama} tiene commits que el remoto no tiene; se borra porque lo pediste con su nombre exacto. Para recuperarlos: git branch {rama} {sha}",
        en: "warning: the local branch {rama} has commits the remote does not have; it is deleted because you asked for it by its exact name. To recover them: git branch {rama} {sha}",
    }
    BLOQUEADA {
        es: "no se ha tocado nada: {ruta} ({motivo}). Guarda y publica sus cambios (o quitalo tu) y repite",
        en: "nothing was touched: {ruta} ({motivo}). Save and push its changes (or remove it yourself) and try again",
    }
    CERROJO_OCUPADO {
        es: "otro tc2 esta escribiendo el estado de este repo; no se ha tocado nada: espera a que acabe y repite",
        en: "another tc2 is writing this repo's state; nothing was touched: wait for it to finish and try again",
    }
    EN_USO {
        es: "{ruta} esta en uso por otro proceso: no se ha tocado ni el ni su entrada del PATH; cierralo y repite tc2 desinstalar --maquina (lo demas ya se quito)",
        en: "{ruta} is in use by another process: neither it nor its PATH entry was touched; close it and run tc2 desinstalar --maquina again (the rest is already removed)",
    }
    FALLO_AL_APLICAR {
        es: "no se pudo quitar {ruta}: {error}. Lo de las lineas anteriores ya se quito: arregla la causa y repite tc2 desinstalar",
        en: "could not remove {ruta}: {error}. What the previous lines list is already removed: fix the cause and run tc2 desinstalar again",
    }
    SIN_ENTORNO {
        es: "no se puede saber donde esta este tc2: {error}",
        en: "cannot tell where this tc2 is: {error}",
    }
}
