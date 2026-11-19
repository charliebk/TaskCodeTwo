//! Mensajes de `src/git/`.

crate::i18n::textos! {
    // Puerto y llamadas a git (`mod.rs`, `puerto.rs`, `estado.rs`).
    ARGUMENTO_PROHIBIDO {
        es: "argumento prohibido para la capa: {argumento}",
        en: "argument not allowed for the layer: {argumento}",
    }
    SIN_ENTRADA {
        es: "argumento prohibido para la capa: {sub} con entrada por stdin (solo cat-file, hash-object -w --no-filters --stdin-paths y update-index --add -z --index-info con indice temporal)",
        en: "argument not allowed for the layer: {sub} with stdin input (only cat-file, hash-object -w --no-filters --stdin-paths and update-index --add -z --index-info with a temporary index)",
    }
    CAT_FILE_INCOMPLETO {
        es: "git cat-file --batch devolvio una salida que no se entiende",
        en: "git cat-file --batch returned output that cannot be parsed",
    }
    SIN_INDICE_TEMPORAL {
        es: "argumento prohibido para la capa: {sub} sin indice temporal",
        en: "argument not allowed for the layer: {sub} without a temporary index",
    }
    CON_INDICE_TEMPORAL {
        es: "argumento prohibido para la capa: {sub} con indice temporal",
        en: "argument not allowed for the layer: {sub} with a temporary index",
    }
    INDICE_RELATIVO {
        es: "argumento prohibido para la capa: {sub} con indice relativo",
        en: "argument not allowed for the layer: {sub} with a relative index",
    }
    NO_SE_PUDO_LANZAR_GIT {
        es: "no se pudo lanzar git: {error}",
        en: "could not run git: {error}",
    }
    GIT_FALLO {
        es: "git {subcomando} fallo: {error}",
        en: "git {subcomando} failed: {error}",
    }
    ERROR_DE_FICHERO {
        es: "{ruta}: {error}",
        en: "{ruta}: I/O error: {error}",
    }
    RUTA_TEMPORAL_NO_UTF8 {
        es: "ruta temporal no UTF-8",
        en: "temporary path is not valid UTF-8",
    }
    RUTA_INVALIDA_EN_ESTADO {
        es: "ruta invalida en la rama de estado: {ruta}",
        en: "invalid path in the state branch: {ruta}",
    }
    HASH_OBJECT_INCOMPLETO {
        es: "git hash-object no devolvio un hash por fichero",
        en: "git hash-object did not return one hash per file",
    }
    FIRMA_FALLO {
        es: "git no pudo firmar el commit ({error}): configura gpg.format y user.signingkey con tu clave de firma y vuelve a lanzarlo desde tu terminal",
        en: "git could not sign the commit ({error}): set gpg.format and user.signingkey to your signing key and run it again from your terminal",
    }

    // Donde se lanza.
    LANZALO_EN_EL_WORKTREE {
        es: "lanzalo en el worktree de la tarea",
        en: "run it in the task worktree",
    }
    LANZALO_FUERA_DE_PROTEGIDA {
        es: "lanzalo en el worktree de la tarea, no en develop ni en main",
        en: "run it in the task worktree, not on develop or main",
    }
    NO_ES_REPO {
        es: "lanza el comando dentro de un repositorio git",
        en: "run the command inside a git repository",
    }

    // Remoto (`remoto.rs`, `rama.rs`, `tag.rs`).
    SIN_RED_REMOTO {
        es: "{op}: no se pudo leer {remoto}; reintenta con red",
        en: "{op}: could not read {remoto}; retry with network access",
    }
    SIN_RAMA_BASE {
        es: "{remoto} no tiene la rama base {rama}: creala y publicala (o revisa que {remoto} es el repo del proyecto); reintentar no lo arregla",
        en: "{remoto} has no base branch {rama}: create and publish it (or check that {remoto} is the project's repo); retrying will not fix it",
    }
    DENEGADA_CREDENCIALES {
        es: "{op}: {detalle}; revisa tus credenciales o permisos en el remoto",
        en: "{op}: {detalle}; check your credentials or permissions on the remote",
    }
    DENEGADA_URL {
        es: "{op}: {detalle}; revisa la URL de origin",
        en: "{op}: {detalle}; check the origin URL",
    }
    DENEGADA_REGLAS {
        es: "{op}: {detalle}; revisa las reglas del remoto o avisa a la persona",
        en: "{op}: {detalle}; check the remote rules or tell the person",
    }
    ERROR_LOCAL {
        es: "{op}: {detalle}; fallo local de git: revisa los permisos de la carpeta del repo",
        en: "{op}: {detalle}; local git failure: check the permissions of the repo folder",
    }
    QUE_HACER_DENEGADA {
        es: "el remoto deniega el push: revisa sus reglas, tus permisos o avisa a la persona",
        en: "the remote denies the push: check its rules, your permissions or tell the person",
    }
    QUE_HACER_ERROR_LOCAL {
        es: "fallo local de git: revisa los permisos de la carpeta del repo",
        en: "local git failure: check the permissions of the repo folder",
    }
    QUE_HACER_SIN_RED {
        es: "reintenta con red",
        en: "retry with network access",
    }
    PUSH_FALLO {
        es: "push: {ultimo}; {que_hacer}",
        en: "push failed: {ultimo}; {que_hacer}",
    }
    PUSH_FALLO_ACTUALIZADA_EN_LOCAL {
        es: "push: {ultimo}; la rama esta actualizada en local; {que_hacer}",
        en: "push failed: {ultimo}; the branch is updated locally; {que_hacer}",
    }
    LANZA_ACTUALIZAR_RAMA {
        es: "lanza actualizar-rama",
        en: "run actualizar-rama",
    }
    LANZA_GUARDAR_CON_RESUELTOS {
        es: "lanza guardar con los ficheros resueltos",
        en: "run guardar with the resolved files",
    }
    RESUELVE_Y_LANZA_GUARDAR {
        es: "resuelve {ficheros} y lanza guardar",
        en: "resolve {ficheros} and run guardar",
    }
    GUARDA_ANTES_DE_ACTUALIZAR {
        es: "guarda los cambios con guardar antes de actualizar la rama",
        en: "save the changes with guardar before updating the branch",
    }
    BASE_NO_EXISTE_EN_REMOTO {
        es: "{base} no existe en el remoto",
        en: "{base} does not exist on the remote",
    }
    PUBLICARON_ENTRE_MEDIAS {
        es: "alguien publico en la rama entre medias: vuelve a lanzar actualizar-rama",
        en: "someone pushed to the branch in the meantime: run actualizar-rama again",
    }
    PRINCIPAL_EN_OTRA_RAMA {
        es: "el worktree principal no esta en {rama}",
        en: "the main worktree is not on {rama}",
    }
    PRINCIPAL_DIVERGENTE {
        es: "{rama} local tiene commits que no estan en {remota}",
        en: "local {rama} has commits that are not in {remota}",
    }

    // `guardar`.
    RESUELVE_Y_GUARDA_CON_EL_RESTO {
        es: "resuelve {ficheros} y guardalo con el resto",
        en: "resolve {ficheros} and save it with the rest",
    }
    QUITA_LAS_MARCAS {
        es: "quita las marcas de conflicto de {ficheros}",
        en: "remove the conflict markers from {ficheros}",
    }

    // `tag-anotado`.
    REFERENCIA_NO_EXISTE {
        es: "{referencia} no existe",
        en: "{referencia} does not exist",
    }
    TAG_EN_OTRO_COMMIT {
        es: "el tag {tag} ya existe sobre otro commit: elige otro nombre",
        en: "tag {tag} already exists on another commit: choose another name",
    }
    TAG_SIN_PUBLICAR {
        es: "no se pudo publicar el tag: {ultimo}; el tag queda en local, {que_hacer}",
        en: "could not publish the tag: {ultimo}; the tag stays local, {que_hacer}",
    }

    // `quitar-worktree`.
    WORKTREE_CON_CAMBIOS {
        es: "guarda o descarta los cambios del worktree antes de quitarlo",
        en: "save or discard the worktree changes before removing it",
    }
    WORKTREE_HEAD_SUELTO {
        es: "el worktree tiene HEAD suelto: vuelve a la rama de la tarea",
        en: "the worktree has a detached HEAD: go back to the task branch",
    }
    WORKTREE_SIN_PUBLICAR {
        es: "publica la rama antes de quitar el worktree",
        en: "push the branch before removing the worktree",
    }

    // Rama y worktree de tarea (`rama.rs`).
    BASE_NO_EXISTE {
        es: "la base {base} no existe: lanza fetch o revisa el nombre",
        en: "base {base} does not exist: run fetch or check the name",
    }
    ID_CON_RAMA {
        es: "{id} ya tiene rama: elige otro ID",
        en: "{id} already has a branch: choose another ID",
    }
    RUTA_WORKTREE_NO_UTF8 {
        es: "la ruta del worktree no es UTF-8",
        en: "the worktree path is not valid UTF-8",
    }
    RAMA_EN_OTRO_WORKTREE {
        es: "la rama ya esta abierta en otro worktree",
        en: "the branch is already open in another worktree",
    }
    RAMA_YA_EXISTE {
        es: "la rama ya existe: la recupera reanudar",
        en: "the branch already exists: reanudar recovers it",
    }
    RAMA_YA_PUBLICADA {
        es: "la rama ya esta publicada: la recupera reanudar",
        en: "the branch is already published: reanudar recovers it",
    }
    CARPETA_OCUPADA {
        es: "{ruta} ya existe y no es el worktree de la tarea",
        en: "{ruta} already exists and is not the task worktree",
    }
    TAREA_SIN_RAMA {
        es: "la tarea no tiene rama ni en local ni en el remoto",
        en: "the task has no branch, neither locally nor on the remote",
    }
}
