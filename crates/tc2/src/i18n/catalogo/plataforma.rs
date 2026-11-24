//! Mensajes de `src/plataforma/`.
//!
//! `{cita}` es la cita del mensaje de la API (` (GitHub: «...»)`), vacia si
//! la API no dio mensaje: la compone `plataforma::errores` con el nombre de
//! la plataforma y el texto saneado, sin nada que traducir.

crate::i18n::textos! {
    // --- Respuestas HTTP que no son 2xx (errores.rs) ---
    TOKEN_NO_VALE {
        es: "el token de {rol} no vale o ha caducado: define o renueva {variable}{cita}",
        en: "the {rol} token is not valid or has expired: set or renew {variable}{cita}",
    }
    LIMITE_AGOTADO {
        es: "limite de peticiones de la API agotado: espera {segundos} s y relanza{cita}",
        en: "API rate limit exhausted: wait {segundos} s and run again{cita}",
    }
    PLAN_GITHUB {
        es: "el plan de GitHub no da esta funcion en un repo privado: haz el repo publico o usa GitHub Pro, Team o Enterprise{cita}",
        en: "the GitHub plan does not offer this feature on a private repo: make the repo public or use GitHub Pro, Team or Enterprise{cita}",
    }
    SIN_PERMISO {
        es: "el token de {rol} no tiene permiso: necesita {permiso}{cita}",
        en: "the {rol} token lacks permission: it needs {permiso}{cita}",
    }
    NO_EXISTE {
        es: "no existe o el token de {rol} no lo ve: revisa {objeto} y el alcance del token ({plataforma} da 404 en un repo privado sin acceso){cita}",
        en: "it does not exist or the {rol} token cannot see it: check {objeto} and the token scope ({plataforma} returns 404 on a private repo without access){cita}",
    }
    OBJETO_GITHUB {
        es: "el repo, el PR o el commit",
        en: "the repo, the PR or the commit",
    }
    OBJETO_GITLAB {
        es: "el proyecto, el MR o el commit",
        en: "the project, the MR or the commit",
    }
    NO_MERGEABLE {
        es: "{plataforma} no deja mergear: faltan puertas o hay conflictos; lanza estado-puertas{cita}",
        en: "{plataforma} does not allow the merge: gates are missing or there are conflicts; run estado-puertas{cita}",
    }
    HEAD_CAMBIADO {
        es: "la rama ha cambiado desde que se evaluaron las puertas: vuelve a lanzar estado-puertas{cita}",
        en: "the branch has changed since the gates were evaluated: run estado-puertas again{cita}",
    }
    SIN_COMMITS {
        es: "la rama no tiene commits nuevos respecto a la base: no hay nada que revisar{cita}",
        en: "the branch has no new commits relative to the base: there is nothing to review{cita}",
    }
    RESPUESTA_INESPERADA {
        es: "respuesta inesperada de {plataforma} ({estado}): reintenta mas tarde{cita}",
        en: "unexpected response from {plataforma} ({estado}): try again later{cita}",
    }
    VALIDACION {
        es: "{plataforma} rechazo los datos de la peticion{cita}",
        en: "{plataforma} rejected the request data{cita}",
    }

    // --- Fallos sin respuesta (errores.rs) ---
    LLAMADA_RECHAZADA {
        es: "llamada rechazada antes de salir a la red: {motivo}",
        en: "call rejected before reaching the network: {motivo}",
    }
    SIN_RED {
        es: "no se pudo hablar con la API: comprueba la red y reintenta ({detalle})",
        en: "could not reach the API: check the network and try again ({detalle})",
    }
    TLS {
        es: "fallo de TLS: si la red usa un proxy con su propia CA, instalala en el almacen de certificados del sistema ({detalle})",
        en: "TLS failure: if the network uses a proxy with its own CA, install it in the system certificate store ({detalle})",
    }
    RESPUESTA_ILEGIBLE {
        es: "respuesta de la API ilegible o demasiado grande ({detalle})",
        en: "API response unreadable or too large ({detalle})",
    }
    DECLARA_MAS_DE {
        es: "la respuesta declara mas de {limite} bytes",
        en: "the response declares more than {limite} bytes",
    }
    REDIRECCION {
        es: "la API respondio con una redireccion ({estado}) y no se siguen: revisa la URL de la API o el nombre del repo (renombrado o movido)",
        en: "the API answered with a redirect ({estado}) and redirects are not followed: check the API URL or the repo name (renamed or moved)",
    }
    CLIENTE_HTTP {
        es: "fallo del cliente HTTP ({detalle})",
        en: "HTTP client failure ({detalle})",
    }
    FORMA_INESPERADA {
        es: "la respuesta de {plataforma} no tiene la forma esperada ({que})",
        en: "the {plataforma} response does not have the expected shape ({que})",
    }
    RAMA_BASE_O_SHA {
        es: "rama base o sha del head",
        en: "base branch or head sha",
    }
    RAMA_DESTINO_O_SHA {
        es: "rama destino o sha del MR",
        en: "target branch or MR sha",
    }

    // --- Transporte comun (conexion.rs) ---
    FALTA_IDENTIDAD {
        es: "esta operacion corre con la identidad de {rol}: define {variable}{extra}",
        en: "this operation runs with the {rol} identity: set {variable}{extra}",
    }
    APP_SIN_TOKEN {
        es: "la GitHub App de {rol} no consiguio su token de instalacion ({causa}: {detalle}): revisa {variables}, que la App este instalada en el repo y que tenga los permisos del rol",
        en: "the {rol} GitHub App did not get its installation token ({causa}: {detalle}): check {variables}, that the App is installed on the repo and that it has the role permissions",
    }
    APP_SOLO_GITHUB {
        es: "en GitLab la GitHub App de {rol} no sirve ({variables} solo valen en GitHub): define {variable} con un token de proyecto o de cuenta de servicio de {rol}",
        en: "on GitLab the {rol} GitHub App does not apply ({variables} only work on GitHub): set {variable} to a project or service account token for {rol}",
    }
    APP_TLS {
        es: "la GitHub App de {rol} no consiguio su token por un fallo de TLS ({detalle}): si la red usa un proxy con su propia CA, instalala en el almacen de certificados del sistema; revisa tambien la hora del equipo",
        en: "the {rol} GitHub App did not get its token because of a TLS failure ({detalle}): if the network uses a proxy with its own CA, install it in the system certificate store; also check the machine clock",
    }
    SOLO_TERMINAL {
        es: " (el de persona solo se usa desde un terminal interactivo)",
        en: " (the persona one is only used from an interactive terminal)",
    }
    DEMASIADAS_PAGINAS {
        es: "la lista tiene mas de {n} elementos: no se evalua a medias",
        en: "the list has more than {n} elements: it is not evaluated halfway",
    }
    PERSONA_FUERA_DE_LISTA {
        es: "el token de persona es de «{login}», que no esta en la lista de personas del proyecto",
        en: "the persona token belongs to «{login}», who is not in the project's list of personas",
    }
    ROL_NO_VALIDO {
        es: "{op} corre con la identidad de agente o de persona",
        en: "{op} runs with the agente or persona identity",
    }

    // --- Adaptadores (github.rs y gitlab.rs) ---
    REPO_GITHUB {
        es: "el repo de GitHub tiene que ser dueno/repo",
        en: "the GitHub repo must be owner/repo",
    }
    PROYECTO_GITLAB {
        es: "el proyecto de GitLab tiene que ser su id numerico o grupo/proyecto",
        en: "the GitLab project must be its numeric id or group/project",
    }
    RAMA_NO_VALIDA {
        es: "nombre de rama no valido",
        en: "invalid branch name",
    }
    TITULO_NO_VALIDO {
        es: "el titulo no puede estar vacio ni llevar controles",
        en: "the title cannot be empty or contain control characters",
    }
    SHA_ESPERADO_COMPLETO {
        es: "el sha esperado tiene que ser completo (40 hex)",
        en: "the expected sha must be complete (40 hex)",
    }
    SHA_COMPLETO {
        es: "el sha tiene que ser completo (40 hex)",
        en: "the sha must be complete (40 hex)",
    }
    PR_CERRADO {
        es: "el PR esta cerrado sin mergear",
        en: "the PR is closed without being merged",
    }
    MR_CERRADO {
        es: "el MR esta cerrado sin mergear",
        en: "the MR is closed without being merged",
    }
    CI_PUEDE_APROBAR {
        es: "el token de Actions puede aprobar PR: desactiva «Allow GitHub Actions to create and approve pull requests» en Settings > Actions > General (G16)",
        en: "the Actions token can approve PRs: turn off «Allow GitHub Actions to create and approve pull requests» in Settings > Actions > General (G16)",
    }
    PERMISO_NINGUNO {
        es: "(ninguno)",
        en: "(none)",
    }
    PERMISO_PR_Y_CHECKS {
        es: "pull_requests:read y checks:read",
        en: "pull_requests:read and checks:read",
    }
    PERMISO_LECTURA_GITLAB {
        es: "el scope api (o read_api) y el rol Developer en el proyecto",
        en: "the api scope (or read_api) and the Developer role in the project",
    }
    PERMISO_ESCRITURA_GITLAB {
        es: "el scope api y el rol Developer en el proyecto",
        en: "the api scope and the Developer role in the project",
    }
    PERMISO_MERGE_GITLAB {
        es: "el scope api y el rol Maintainer en el proyecto",
        en: "the api scope and the Maintainer role in the project",
    }
    PR_EXISTENTE {
        es: "GitHub dice que ya hay un PR para la rama pero no se encuentra abierto: reintenta (GitHub: «{api}»)",
        en: "GitHub says there is already a PR for the branch but no open one is found: try again (GitHub: «{api}»)",
    }
    MR_EXISTENTE {
        es: "GitLab dice que ya hay un MR para la rama pero no se encuentra abierto: reintenta (GitLab: «{api}»)",
        en: "GitLab says there is already an MR for the branch but no open one is found: try again (GitLab: «{api}»)",
    }
    MERGE_NO_PERMITIDO_GITLAB {
        es: "GitLab no deja mergear a esta cuenta: necesita {permiso} (GitLab: «{api}»)",
        en: "GitLab does not let this account merge: it needs {permiso} (GitLab: «{api}»)",
    }
    ARREGLO_APROBACIONES_GITLAB {
        es: "GitLab no impone aprobaciones en este proyecto: crea una regla de aprobacion con al menos una aprobacion (requiere Premium o Ultimate) o quita la garantia de la politica",
        en: "GitLab does not enforce approvals in this project: create an approval rule with at least one approval (requires Premium or Ultimate) or remove the guarantee from the policy",
    }
    ARREGLO_RAMA_PROTEGIDA_GITLAB {
        es: "la rama destino no esta protegida con merge solo para Maintainers y sin force-push: protegela (taskctl setup lo propone)",
        en: "the target branch is not protected with merge only for Maintainers and no force-push: protect it (taskctl setup proposes it)",
    }

    // --- Puertas (puertas.rs) ---
    CHECKS_EN_ROJO {
        es: "hay checks obligatorios en rojo: corrige y vuelve a empujar",
        en: "required checks are red: fix and push again",
    }
    FALTAN_PUERTAS {
        es: "faltan puertas: {falta}; espera o atiende lo que falta (al-dia o sin-conflictos: lanza actualizar-rama)",
        en: "missing gates: {falta}; wait or address what is missing (al-dia or sin-conflictos: run actualizar-rama)",
    }
    PR_CERRADO_REABRE {
        es: "el PR esta cerrado sin mergear: reabrelo o abre otro",
        en: "the PR is closed without being merged: reopen it or open another one",
    }
    RECHAZADA_POR_PLAN {
        es: "la politica del proyecto exige {garantia} impuesta y el plan de la plataforma no la da: cambia de plan o quita {garantia} de la politica",
        en: "the project policy requires {garantia} to be enforced and the platform plan does not offer it: change plan or remove {garantia} from the policy",
    }
    NO_SE_CUMPLE {
        es: "{garantia} no se cumple: {arreglo}",
        en: "{garantia} is not met: {arreglo}",
    }

    // --- Deteccion (deteccion.rs) ---
    ORIGIN_NO_ENTENDIDO {
        es: "no se entiende la URL de origin: tiene que ser https, ssh o scp con dueno y repo",
        en: "the origin URL cannot be understood: it must be https, ssh or scp with owner and repo",
    }
    CLAVE_PLATAFORMA {
        es: "la clave plataforma vale github o gitlab",
        en: "the plataforma key must be github or gitlab",
    }
    URL_API {
        es: "URL de la API: {motivo}",
        en: "API URL: {motivo}",
    }
    PLATAFORMA_CONTRADICE {
        es: "la clave plataforma dice {plataforma} y origin es de {host}",
        en: "the plataforma key says {plataforma} and origin is on {host}",
    }
    API_NO_CAMBIABLE {
        es: "origin es de {host} y su API es {api}: la clave de la configuracion no puede cambiarla",
        en: "origin is on {host} and its API is {api}: the configuration key cannot change it",
    }
    DECLARA_PLATAFORMA {
        es: "origin es de {host}: declara plataforma (github o gitlab) y la URL de su API en .taskcode/config.yml",
        en: "origin is on {host}: declare plataforma (github or gitlab) and its API URL in .taskcode/config.yml",
    }
    ORIGIN_GITHUB {
        es: "en GitHub origin tiene que ser dueno/repo",
        en: "on GitHub, origin must be owner/repo",
    }

    // --- Envoltorio HTTP (http.rs) ---
    HTTP_SOLO_LOCAL {
        es: "http solo vale para 127.0.0.1 con un puerto y, si acaso, un prefijo valido",
        en: "http is only valid for 127.0.0.1 with a port and, optionally, a valid prefix",
    }
    URL_HTTPS {
        es: "la URL de la API tiene que ser https",
        en: "the API URL must be https",
    }
    HOST_NO_VALIDO {
        es: "host de la API no valido (sin usuario ni contrasena)",
        en: "invalid API host (no user or password)",
    }
    PREFIJO_CON_CONSULTA {
        es: "el prefijo de la API no lleva consulta ni fragmento",
        en: "the API prefix cannot have a query or fragment",
    }
    PREFIJO_NO_VALIDO {
        es: "prefijo de la API no valido",
        en: "invalid API prefix",
    }
    PAGINACION_OTRO_DESTINO {
        es: "paginacion hacia otro destino",
        en: "pagination to another destination",
    }
    RUTA_NO_RELATIVA {
        es: "la ruta tiene que ser relativa a la API",
        en: "the path must be relative to the API",
    }
    RUTA_CODIFICADA {
        es: "ruta con barra invertida o puntos codificados",
        en: "path with a backslash or encoded dots",
    }
    RUTA_SEGMENTOS {
        es: "ruta con segmentos vacios, . o ..",
        en: "path with empty, . or .. segments",
    }
    CARACTER_EN_RUTA {
        es: "caracter no permitido en la ruta",
        en: "character not allowed in the path",
    }
    CARACTER_EN_CONSULTA {
        es: "caracter no permitido en la consulta",
        en: "character not allowed in the query",
    }

    // --- entorno-push (F6-2) ---
    REMOTO_NO_HTTPS {
        es: "el remoto origin no es una URL https://: el agente solo empuja con su token por https (por ssh empujaria tu identidad y por http el token iria en claro). Cambialo con git remote set-url origin https://... y repite",
        en: "the origin remote is not an https:// URL: the agent only pushes with its token over https (over ssh it would push your identity and over http the token would travel in clear). Change it with git remote set-url origin https://... and try again",
    }
    ENTORNO_PUSH_NO_SOPORTADO {
        es: "esta plataforma no da el entorno para que el agente empuje con su identidad",
        en: "this platform does not provide the environment for the agent to push with its identity",
    }

    // --- aplicar-puertas (TASK-118) ---
    LEER_REVISION_NO_SOPORTADA {
        es: "esta plataforma no publica el check revision-ia (la revision IA en CI solo existe en GitHub): no hay informe que leer",
        en: "this platform does not publish the revision-ia check (AI review in CI only exists on GitHub): there is no report to read",
    }
    REVISION_SIN_CHECK {
        es: "el PR {pr} aun no tiene terminado el check revision-ia de la App {origen} en {sha}: espera a que acabe el workflow revision-ia y vuelve a lanzar tc2 review",
        en: "PR {pr} does not yet have a finished revision-ia check from App {origen} on {sha}: wait for the revision-ia workflow to finish and run tc2 review again",
    }
    RAW_DETAILS_REVISION {
        es: "el raw_details de una anotacion del check revision-ia, que no es un hallazgo de tc2 hallazgos veredicto",
        en: "the raw_details of a revision-ia check annotation, which is not a finding from tc2 hallazgos veredicto",
    }
    APLICAR_NO_SOPORTADA {
        es: "esta plataforma no admite aplicar las puertas desde tc2: aplicalas a mano con la propuesta de tc2 init --json",
        en: "this platform does not support applying the gates from tc2: apply them by hand with the proposal from tc2 init --json",
    }
    CHECKS_SIN_ORIGEN_VISTO {
        es: "no se han visto publicados por su origen en la cabeza de develop los checks {faltan}: exigirlos ahora bloquearia develop para todos. No se ha escrito nada en la plataforma; mergea en develop un PR en el que la CI y el revisor publiquen esos checks y repite tc2 init --aplicar-puertas",
        en: "the checks {faltan} have not been seen published by their origin on the head of develop: requiring them now would block develop for everyone. Nothing was written on the platform; merge into develop a PR where the CI and the reviewer publish those checks and run tc2 init --aplicar-puertas again",
    }
}
