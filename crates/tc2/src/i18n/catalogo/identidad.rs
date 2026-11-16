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
        es: "el `login` no puede estar vacio ni llevar espacios ni caracteres de control",
        en: "`login` cannot be empty or contain spaces or control characters",
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
}
