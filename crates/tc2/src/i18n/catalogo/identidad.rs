//! Mensajes de la identidad de la persona (F2-13): la lista
//! `.taskcode/personas.yml` (`plataforma/personas.rs`), quien ejecuta
//! (`identidad.rs`) y la orden `tc2 identidad`.

crate::i18n::textos! {
    // --- Lista de personas (plataforma/personas.rs) ---
    PERSONAS_YAML {
        es: "{ruta} no es YAML valido o no tiene la forma esperada (`personas:` con `id` y `login` por entrada): {error}",
        en: "{ruta} is not valid YAML or does not have the expected shape (`personas:` with `id` and `login` per entry): {error}",
    }
    PERSONAS_ENTRADA {
        es: "{ruta}: la entrada {n} no vale: {que}",
        en: "{ruta}: entry {n} is not valid: {que}",
    }
    ID_CERO {
        es: "el `id` tiene que ser el numero de la cuenta (mayor que 0)",
        en: "`id` must be the account number (greater than 0)",
    }
    LOGIN_NO_VALIDO {
        es: "el `login` no puede estar vacio, pasar de 255 caracteres ni llevar espacios ni caracteres de control",
        en: "`login` cannot be empty, be longer than 255 characters or contain spaces or control characters",
    }
    NOMBRE_NO_VALIDO {
        es: "el `nombre` no puede estar vacio ni llevar caracteres de control",
        en: "`nombre` cannot be empty or contain control characters",
    }
    CORREO_NO_VALIDO {
        es: "cada `correos` tiene que ser una direccion sin espacios ni caracteres de control",
        en: "each `correos` entry must be an address without spaces or control characters",
    }
    ID_REPETIDO {
        es: "{ruta}: las entradas {a} y {b} repiten el id {id}; cada cuenta va una sola vez",
        en: "{ruta}: entries {a} and {b} repeat id {id}; each account goes only once",
    }
    LOGIN_REPETIDO {
        es: "{ruta}: las entradas {a} y {b} repiten el login (sin distinguir mayusculas); cada cuenta va una sola vez",
        en: "{ruta}: entries {a} and {b} repeat the login (case-insensitive); each account goes only once",
    }
    CORREO_REPETIDO {
        es: "{ruta}: las entradas {a} y {b} repiten el correo {correo}; un correo de la 0.16 solo puede ser de una persona",
        en: "{ruta}: entries {a} and {b} repeat the address {correo}; a 0.16 address can belong to only one person",
    }

    // --- Quien ejecuta (identidad.rs) ---
    CUENTA_BOT {
        es: "la cuenta del token de persona ({login}) es un bot: TC2_TOKEN_PERSONA tiene que ser el token de tu usuario",
        en: "the account of the person token ({login}) is a bot: TC2_TOKEN_PERSONA must be your user's token",
    }
    CUENTA_NO_PERSONA {
        es: "la cuenta del token de persona ({login}) es de tipo {tipo}, no de una persona: TC2_TOKEN_PERSONA tiene que ser el token de tu usuario",
        en: "the account of the person token ({login}) is of type {tipo}, not a person: TC2_TOKEN_PERSONA must be your user's token",
    }
    CUENTA_WEB_FLOW {
        es: "la cuenta del token de persona es web-flow, la de la web de GitHub: TC2_TOKEN_PERSONA tiene que ser el token de tu usuario",
        en: "the account of the person token is web-flow, GitHub's web account: TC2_TOKEN_PERSONA must be your user's token",
    }
    CUENTA_FUERA_DE_LISTA {
        es: "la cuenta {login} (id {id}) no esta en {ruta}: pide que la anadan con un PR, o usa el token de una persona de la lista; si ya la anadieron, ejecuta `tc2 identidad` (actualiza la lista)",
        en: "account {login} (id {id}) is not in {ruta}: ask for it to be added with a PR, or use the token of a person on the list; if it was already added, run `tc2 identidad` (it updates the list)",
    }
    LOGIN_DE_LA_API {
        es: "la plataforma devolvio un login que no vale para {login} (id {id}): revisa la cuenta del token",
        en: "the platform returned an invalid login for {login} (id {id}): check the token's account",
    }
    SIN_IDENTIDAD_GUARDADA {
        es: "sin terminal, el agente actua en nombre de la persona guardada y {estado}: lanza `tc2 identidad` en un terminal con TC2_TOKEN_PERSONA",
        en: "without a terminal, the agent acts on behalf of the stored person and {estado}: run `tc2 identidad` in a terminal with TC2_TOKEN_PERSONA",
    }
    CACHE_FALTA {
        es: "no hay ninguna guardada",
        en: "there is none stored",
    }
    CACHE_CADUCADA {
        es: "la guardada tiene mas de 7 dias",
        en: "the stored one is more than 7 days old",
    }
    CACHE_OTRA_PLATAFORMA {
        es: "la guardada es de otra plataforma",
        en: "the stored one is from another platform",
    }
    CACHE_FUERA_DE_LISTA {
        es: "la guardada ({login}, id {id}) no esta en la lista de personas de esta copia; si ya la anadieron, ejecuta `tc2 identidad` (actualiza la lista)",
        en: "the stored one ({login}, id {id}) is not on this copy's list of people; if it was already added, run `tc2 identidad` (it updates the list)",
    }
    NO_SE_ESCRIBE_CACHE {
        es: "no se pudo guardar la identidad en {ruta}: {error}",
        en: "could not store the identity in {ruta}: {error}",
    }

    // --- Orden `tc2 identidad` ---
    USO_IDENTIDAD {
        es: "uso: identidad [--json]",
        en: "usage: identidad [--json]",
    }
    NECESITA_TERMINAL {
        es: "`tc2 identidad` la lanza la persona en un terminal con TC2_TOKEN_PERSONA: sin terminal no se verifica ni se guarda nada",
        en: "`tc2 identidad` is run by the person in a terminal with TC2_TOKEN_PERSONA: without a terminal nothing is verified or stored",
    }
    SIN_ORIGIN {
        es: "el repo no tiene `origin`: la identidad sale de la plataforma de `origin` (GitHub o GitLab) y no hay modo sin plataforma",
        en: "the repo has no `origin`: the identity comes from the `origin` platform (GitHub or GitLab) and there is no mode without a platform",
    }
    SIN_PERSONAS {
        es: "no hay {ruta} en {base}: anade la lista de personas (id y login de cada cuenta) con un PR",
        en: "there is no {ruta} in {base}: add the list of people (id and login of each account) with a PR",
    }
}
