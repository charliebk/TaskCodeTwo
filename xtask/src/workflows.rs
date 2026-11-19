//! Lint de lista blanca de los workflows de verificacion (TASK-041, F5-1).
//!
//! ADR F1-8, decision 1: los workflows de TaskCode separan dos clases de jobs
//! (ejecuta: solo `contents: read` y ningun secreto; privilegio: escribe, y
//! lee el PR solo como datos) y el lint se formula como **lista blanca**:
//!
//! - eventos admitidos: `workflow_call`, `pull_request`, `push`,
//!   `merge_group` y `workflow_dispatch` (fuera `pull_request_target`,
//!   `repository_dispatch`, `workflow_run`, comentarios...);
//! - bloque `permissions` arriba y en cada job, sin `write-all` ni
//!   `read-all`; un job con algun `write` es de clase privilegio;
//! - acciones admitidas: solo `actions/checkout`, fijada por SHA y con
//!   `persist-credentials: false`; en clase privilegio, sin `ref:` ni
//!   `repository:`;
//! - ni `secrets: inherit`, ni secretos en clase ejecuta, ni
//!   `github.event.*`, `github.head_ref` o `toJSON(github...)` en `run:`
//!   (los datos del evento se pasan por `env:`).
//!
//! En la plantilla de GitLab se aplica lo que le corresponde: ningun
//! `secrets:` y ninguna variable que escribe el autor del MR o del commit en
//! `script:`. Ademas, las entradas de la plantilla son las del workflow.
//!
//! Los ficheros son YAML de un subconjunto pequeno (mapas y listas por
//! sangria, escalares, bloques `|` y mapas o listas de flujo sin anidar): el
//! lector de abajo basta para ellos sin anadir dependencias al xtask.
#![allow(clippy::panic)] // modulo solo de test: panic! es la forma de fallar

use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Nodo {
    Escalar(String),
    Mapa(Vec<(String, Nodo)>),
    Lista(Vec<Nodo>),
}

impl Nodo {
    fn get(&self, clave: &str) -> Option<&Nodo> {
        match self {
            Nodo::Mapa(pares) => pares.iter().find(|(k, _)| k == clave).map(|(_, v)| v),
            _ => None,
        }
    }

    fn texto(&self) -> Option<&str> {
        match self {
            Nodo::Escalar(s) => Some(s),
            _ => None,
        }
    }

    fn claves(&self) -> Vec<&str> {
        match self {
            Nodo::Mapa(pares) => pares.iter().map(|(k, _)| k.as_str()).collect(),
            _ => Vec::new(),
        }
    }

    /// Todos los escalares del arbol, con la clave que los contiene.
    fn escalares<'a>(&'a self, clave: &'a str, salida: &mut Vec<(&'a str, &'a str)>) {
        match self {
            Nodo::Escalar(s) => salida.push((clave, s)),
            Nodo::Mapa(pares) => pares.iter().for_each(|(k, v)| v.escalares(k, salida)),
            Nodo::Lista(items) => items.iter().for_each(|v| v.escalares(clave, salida)),
        }
    }
}

struct Linea {
    sangria: usize,
    texto: String,
    vacia: bool,
}

/// Lee un documento YAML del subconjunto descrito arriba. Falla en cerrado:
/// si queda alguna linea sin consumir (sangria inesperada, continuacion de un
/// escalar plano...) o una linea de mapa no es `clave:`/`clave: valor`, el
/// documento no se entiende y el lint no puede darlo por bueno.
fn leer_yaml(texto: &str) -> Result<Nodo, String> {
    let mut lineas: Vec<Linea> = texto
        .lines()
        .map(|l| {
            let sin_fin = l.trim_end();
            let contenido = sin_fin.trim_start();
            Linea {
                sangria: sin_fin.len() - contenido.len(),
                texto: contenido.to_owned(),
                vacia: contenido.is_empty() || contenido.starts_with('#'),
            }
        })
        .collect();
    let mut i = 0;
    saltar_vacias(&lineas, &mut i);
    let raiz = match lineas.get(i).map(|l| l.sangria) {
        Some(sangria) => bloque(&mut lineas, &mut i, sangria)?,
        None => Nodo::Mapa(Vec::new()),
    };
    saltar_vacias(&lineas, &mut i);
    match lineas.get(i) {
        Some(l) => Err(format!(
            "documento no entendido: linea {}: {}",
            i + 1,
            l.texto
        )),
        None => Ok(raiz),
    }
}

/// `leer_yaml` para las pruebas que leen un documento que tiene que entenderse.
fn yaml(texto: &str) -> Nodo {
    leer_yaml(texto).unwrap_or_else(|e| panic!("{e}"))
}

fn saltar_vacias(lineas: &[Linea], i: &mut usize) {
    while lineas.get(*i).is_some_and(|l| l.vacia) {
        *i += 1;
    }
}

fn es_item(texto: &str) -> bool {
    texto == "-" || texto.starts_with("- ")
}

fn bloque(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Result<Nodo, String> {
    if lineas.get(*i).is_some_and(|l| es_item(&l.texto)) {
        lista(lineas, i, sangria)
    } else {
        mapa(lineas, i, sangria)
    }
}

fn mapa(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Result<Nodo, String> {
    let mut pares = Vec::new();
    loop {
        saltar_vacias(lineas, i);
        let Some(linea) = lineas.get(*i) else { break };
        if linea.sangria != sangria || es_item(&linea.texto) {
            break;
        }
        let texto = linea.texto.clone();
        *i += 1;
        let (clave, valor) = match texto.find(": ") {
            Some(p) => (texto[..p].to_owned(), quitar_comentario(&texto[p + 2..])),
            None => match quitar_comentario(&texto).strip_suffix(':') {
                Some(c) => (c.to_owned(), String::new()),
                None => return Err(format!("documento no entendido: linea {}: {texto}", *i)),
            },
        };
        let clave = sin_comillas(clave.trim()).to_owned();
        let nodo = if valor.starts_with('|') || valor.starts_with('>') {
            Nodo::Escalar(bloque_literal(lineas, i, sangria))
        } else if !valor.is_empty() {
            escalar_o_flujo(&valor)
        } else {
            saltar_vacias(lineas, i);
            match lineas.get(*i) {
                Some(sig) if sig.sangria > sangria => {
                    let s = sig.sangria;
                    bloque(lineas, i, s)?
                }
                Some(sig) if sig.sangria == sangria && es_item(&sig.texto) => {
                    lista(lineas, i, sangria)?
                }
                _ => Nodo::Escalar(String::new()),
            }
        };
        pares.push((clave, nodo));
    }
    Ok(Nodo::Mapa(pares))
}

fn lista(lineas: &mut [Linea], i: &mut usize, sangria: usize) -> Result<Nodo, String> {
    let mut items = Vec::new();
    loop {
        saltar_vacias(lineas, i);
        let Some(linea) = lineas.get(*i) else { break };
        if linea.sangria != sangria || !es_item(&linea.texto) {
            break;
        }
        let resto = linea.texto[1..].trim_start().to_owned();
        if resto.is_empty() {
            *i += 1;
            saltar_vacias(lineas, i);
            match lineas.get(*i) {
                Some(sig) if sig.sangria > sangria => {
                    let s = sig.sangria;
                    items.push(bloque(lineas, i, s)?);
                }
                _ => items.push(Nodo::Escalar(String::new())),
            }
        } else if parece_clave(&resto) {
            // `- clave: valor`: el item es un mapa cuya primera clave empieza
            // donde acaba el guion; el resto de sus claves van a esa sangria.
            let s = linea.sangria + (linea.texto.len() - resto.len());
            if let Some(l) = lineas.get_mut(*i) {
                l.sangria = s;
                l.texto = resto;
            }
            items.push(mapa(lineas, i, s)?);
        } else {
            *i += 1;
            items.push(escalar_o_flujo(&quitar_comentario(&resto)));
        }
    }
    Ok(Nodo::Lista(items))
}

fn parece_clave(texto: &str) -> bool {
    if texto.starts_with(['"', '\'', '$', '{', '[']) {
        return false;
    }
    let clave = match texto.find(": ") {
        Some(p) => &texto[..p],
        None if texto.ends_with(':') => &texto[..texto.len() - 1],
        None => return false,
    };
    !clave.is_empty()
        && clave
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Las lineas de un bloque `|` o `>`: todas las que van mas sangradas que la
/// clave, y las vacias entre ellas. Los `#` de dentro no son comentarios.
fn bloque_literal(lineas: &[Linea], i: &mut usize, sangria: usize) -> String {
    let mut texto = Vec::new();
    let base = lineas[*i..]
        .iter()
        .find(|x| !x.texto.is_empty())
        .map_or(0, |x| x.sangria);
    while let Some(l) = lineas.get(*i) {
        let en_blanco = l.texto.is_empty();
        if !en_blanco && l.sangria <= sangria {
            break;
        }
        if en_blanco {
            let sigue = lineas[*i..]
                .iter()
                .find(|x| !x.texto.is_empty())
                .is_some_and(|x| x.sangria > sangria);
            if !sigue {
                break;
            }
        }
        let relativa = if en_blanco {
            0
        } else {
            l.sangria.saturating_sub(base)
        };
        texto.push(format!("{}{}", " ".repeat(relativa), l.texto));
        *i += 1;
    }
    texto.join("\n")
}

fn quitar_comentario(valor: &str) -> String {
    let mut comillas: Option<char> = None;
    let mut anterior = ' ';
    for (p, c) in valor.char_indices() {
        match comillas {
            Some(q) if c == q => comillas = None,
            Some(_) => {}
            None if (c == '"' || c == '\'') && anterior == ' ' => comillas = Some(c),
            None if c == '#' && anterior == ' ' => return valor[..p].trim_end().to_owned(),
            None => {}
        }
        anterior = c;
    }
    valor.trim_end().to_owned()
}

fn sin_comillas(valor: &str) -> &str {
    let v = valor.trim();
    for q in ['"', '\''] {
        if v.len() >= 2 && v.starts_with(q) && v.ends_with(q) {
            return &v[1..v.len() - 1];
        }
    }
    v
}

fn escalar_o_flujo(valor: &str) -> Nodo {
    let v = valor.trim();
    let partes = |interior: &str| -> Vec<String> {
        interior
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()
    };
    if v.starts_with('{') && v.ends_with('}') && !v.starts_with("{{") {
        let pares = partes(&v[1..v.len() - 1])
            .into_iter()
            .map(|p| match p.split_once(':') {
                Some((k, x)) => (
                    sin_comillas(k).to_owned(),
                    Nodo::Escalar(sin_comillas(x).to_owned()),
                ),
                None => (sin_comillas(&p).to_owned(), Nodo::Escalar(String::new())),
            })
            .collect();
        return Nodo::Mapa(pares);
    }
    if v.starts_with('[') && v.ends_with(']') {
        let items = partes(&v[1..v.len() - 1])
            .into_iter()
            .map(|p| Nodo::Escalar(sin_comillas(&p).to_owned()))
            .collect();
        return Nodo::Lista(items);
    }
    Nodo::Escalar(sin_comillas(v).to_owned())
}

// --- Lint del workflow de GitHub ---------------------------------------------

const EVENTOS_ADMITIDOS: [&str; 5] = [
    "workflow_call",
    "pull_request",
    "push",
    "merge_group",
    "workflow_dispatch",
];

const ACCIONES_ADMITIDAS: [&str; 1] = ["actions/checkout"];

/// Datos del evento que el autor del PR controla: nunca dentro de `run:`.
/// Se buscan sobre el valor normalizado (ver `normalizar`).
const DATOS_DEL_EVENTO: [&str; 5] = [
    "github.event.",
    "github.event[",
    "github.head_ref",
    "tojson(github",
    "secrets.",
];

/// El valor en minusculas, sin espacios y con los corchetes de cadena como
/// puntos: `toJSON( github)` pasa a `tojson(github` y `github['event']` o
/// `secrets["X"]` a `github.event` y `secrets.x`, para que los patrones no
/// dependan de como se escriba la expresion.
fn normalizar(valor: &str) -> String {
    valor
        .to_ascii_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .replace("['", ".")
        .replace("[\"", ".")
        .replace("']", "")
        .replace("\"]", "")
}

fn usa_secretos(normalizado: &str) -> bool {
    normalizado.contains("secrets.") || normalizado.contains("secrets[")
}

fn eventos(on: &Nodo) -> Vec<String> {
    match on {
        Nodo::Escalar(s) => vec![s.clone()],
        Nodo::Lista(items) => items
            .iter()
            .filter_map(|n| n.texto().map(str::to_owned))
            .collect(),
        Nodo::Mapa(_) => on.claves().into_iter().map(str::to_owned).collect(),
    }
}

/// `Ok(true)` si los permisos tienen algun `write`; `Err` si no son un bloque
/// explicito valido.
fn escribe(permisos: Option<&Nodo>) -> Result<bool, String> {
    match permisos {
        None => Err("sin bloque permissions".to_owned()),
        Some(Nodo::Mapa(pares)) => {
            let mut escribe = false;
            for (alcance, valor) in pares {
                match valor.texto() {
                    Some("read" | "none") => {}
                    Some("write") => escribe = true,
                    otro => return Err(format!("permiso {alcance}: {otro:?}")),
                }
            }
            Ok(escribe)
        }
        Some(Nodo::Escalar(s)) if s == "{}" => Ok(false),
        Some(otro) => Err(format!("permissions no admitido: {otro:?}")),
    }
}

fn es_sha(texto: &str) -> bool {
    texto.len() == 40 && texto.chars().all(|c| c.is_ascii_hexdigit())
}

fn lint_github(texto: &str) -> Vec<String> {
    let raiz = match leer_yaml(texto) {
        Ok(r) => r,
        Err(e) => return vec![e],
    };
    let mut errores = Vec::new();
    match raiz.get("on") {
        None => errores.push("sin bloque on".to_owned()),
        Some(on) => {
            for e in eventos(on) {
                if !EVENTOS_ADMITIDOS.contains(&e.as_str()) {
                    errores.push(format!("evento no admitido: {e}"));
                }
            }
            if on
                .get("workflow_call")
                .and_then(|w| w.get("secrets"))
                .is_some()
            {
                errores.push("workflow_call declara secretos".to_owned());
            }
        }
    }
    if let Err(e) = escribe(raiz.get("permissions")) {
        errores.push(format!("raiz: {e}"));
    }
    // `env` y `defaults` de la raiz llegan a todos los jobs, tambien a los de
    // clase ejecuta: ningun secreto en ellos, y en `defaults` (que acaba en
    // la linea de comandos: shell, working-directory) tampoco datos del evento.
    for bloque_raiz in ["env", "defaults"] {
        let mut valores = Vec::new();
        if let Some(n) = raiz.get(bloque_raiz) {
            n.escalares(bloque_raiz, &mut valores);
        }
        for (_, valor) in valores {
            let normalizado = normalizar(valor);
            if usa_secretos(&normalizado) {
                errores.push(format!("raiz: secreto en {bloque_raiz}"));
            }
            if bloque_raiz == "defaults" {
                for dato in DATOS_DEL_EVENTO {
                    if normalizado.contains(dato) {
                        errores.push(format!("raiz: {dato} en defaults"));
                    }
                }
            }
        }
    }
    let Some(jobs @ Nodo::Mapa(_)) = raiz.get("jobs") else {
        errores.push("sin jobs".to_owned());
        return errores;
    };
    for nombre in jobs.claves() {
        let Some(job) = jobs.get(nombre) else {
            continue;
        };
        let privilegio = match escribe(job.get("permissions")) {
            Ok(p) => p,
            Err(e) => {
                errores.push(format!("job {nombre}: {e}"));
                false
            }
        };
        // Un job que llama a otro workflow reutilizable: fijado por SHA.
        if let Some(uses) = job.get("uses").and_then(Nodo::texto) {
            let version = uses.split_once('@').map_or("", |(_, v)| v);
            if !es_sha(version) {
                errores.push(format!(
                    "job {nombre}: workflow reutilizable sin fijar por SHA: {uses}"
                ));
            }
        }
        let mut valores = Vec::new();
        job.escalares("", &mut valores);
        for (clave, valor) in &valores {
            if *clave == "secrets" && *valor == "inherit" {
                errores.push(format!("job {nombre}: secrets: inherit"));
            }
            let normalizado = normalizar(valor);
            if !privilegio && usa_secretos(&normalizado) {
                errores.push(format!("job {nombre}: secreto en clase ejecuta"));
            }
            if *clave == "run" {
                for dato in DATOS_DEL_EVENTO {
                    if normalizado.contains(dato) {
                        errores.push(format!("job {nombre}: {dato} en run"));
                    }
                }
            }
        }
        let pasos = match job.get("steps") {
            Some(Nodo::Lista(p)) => p.as_slice(),
            _ => &[],
        };
        for paso in pasos {
            let Some(uses) = paso.get("uses").and_then(Nodo::texto) else {
                continue;
            };
            let (accion, version) = uses.split_once('@').unwrap_or((uses, ""));
            if !ACCIONES_ADMITIDAS.contains(&accion) {
                errores.push(format!("job {nombre}: accion no admitida: {accion}"));
                continue;
            }
            if !es_sha(version) {
                errores.push(format!("job {nombre}: {accion} sin fijar por SHA"));
            }
            let con = paso.get("with");
            let opcion = |k: &str| con.and_then(|w| w.get(k));
            if opcion("persist-credentials").and_then(Nodo::texto) != Some("false") {
                errores.push(format!(
                    "job {nombre}: checkout sin persist-credentials: false"
                ));
            }
            if privilegio && (opcion("ref").is_some() || opcion("repository").is_some()) {
                errores.push(format!(
                    "job {nombre}: checkout con ref o repository en clase privilegio"
                ));
            }
        }
    }
    errores
}

// --- Lint de la plantilla de GitLab ------------------------------------------

/// Variables predefinidas que escribe el autor del MR o del commit: nunca
/// dentro de `script:` (se pasan, si hace falta, por `variables:`).
const VARIABLES_DEL_AUTOR: [&str; 7] = [
    "CI_MERGE_REQUEST_TITLE",
    "CI_MERGE_REQUEST_DESCRIPTION",
    "CI_MERGE_REQUEST_SOURCE_BRANCH_NAME",
    "CI_COMMIT_MESSAGE",
    "CI_COMMIT_TITLE",
    "CI_COMMIT_DESCRIPTION",
    "CI_COMMIT_REF_NAME",
];

/// La cabecera `spec:` y el cuerpo de una plantilla con `spec:inputs`.
fn partir_plantilla(texto: &str) -> Result<(Nodo, Nodo), String> {
    match texto.split_once("\n---") {
        Some((cabecera, cuerpo)) => Ok((leer_yaml(cabecera)?, leer_yaml(cuerpo)?)),
        None => Ok((Nodo::Mapa(Vec::new()), leer_yaml(texto)?)),
    }
}

/// Claves de primer nivel que no son jobs.
const CLAVES_GLOBALES: [&str; 6] = [
    "variables",
    "default",
    "include",
    "stages",
    "workflow",
    "spec",
];

/// `$VAR` o `${VAR}` en la expresion, con el nombre completo (`$VAR_X` no es
/// `$VAR`).
fn lee_variable(expresion: &str, variable: &str) -> bool {
    if expresion.contains(&format!("${{{variable}}}")) {
        return true;
    }
    let buscado = format!("${variable}");
    expresion.match_indices(&buscado).any(|(p, _)| {
        !expresion[p + buscado.len()..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

fn lint_gitlab(texto: &str) -> Vec<String> {
    let (cabecera, cuerpo) = match partir_plantilla(texto) {
        Ok(p) => p,
        Err(e) => return vec![e],
    };
    let mut errores = Vec::new();
    if cabecera.get("spec").and_then(|s| s.get("inputs")).is_none() {
        errores.push("sin spec:inputs".to_owned());
    }
    let jobs: Vec<&str> = cuerpo
        .claves()
        .into_iter()
        .filter(|k| !CLAVES_GLOBALES.contains(k))
        .collect();
    // `rules:if` se evalua al crear el pipeline, sin las `variables:` de los
    // jobs (GitLab no las ve ahi): una regla que lea una variable de job no
    // casa nunca. Las reglas usan variables globales o predefinidas.
    let de_job: BTreeSet<&str> = jobs
        .iter()
        .filter_map(|j| cuerpo.get(j).and_then(|n| n.get("variables")))
        .flat_map(Nodo::claves)
        .collect();
    for nombre in &jobs {
        let mut condiciones = Vec::new();
        if let Some(reglas) = cuerpo.get(nombre).and_then(|n| n.get("rules")) {
            reglas.escalares("rules", &mut condiciones);
        }
        for (clave, condicion) in condiciones {
            for variable in &de_job {
                if clave == "if" && lee_variable(condicion, variable) {
                    errores.push(format!(
                        "job {nombre}: rules:if lee la variable de job {variable}"
                    ));
                }
            }
        }
    }
    for nombre in jobs {
        let Some(job) = cuerpo.get(nombre) else {
            continue;
        };
        if job.get("secrets").is_some() {
            errores.push(format!("job {nombre}: secrets"));
        }
        let mut lineas = Vec::new();
        for clave in ["script", "before_script", "after_script"] {
            if let Some(s) = job.get(clave) {
                s.escalares(clave, &mut lineas);
            }
        }
        for (clave, linea) in lineas {
            for variable in VARIABLES_DEL_AUTOR {
                if linea.contains(variable) {
                    errores.push(format!("job {nombre}: {variable} en {clave}"));
                }
            }
        }
    }
    errores
}

// --- Pruebas -------------------------------------------------------------------

fn leer(relativa: &str) -> String {
    let ruta = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(relativa);
    let texto =
        std::fs::read_to_string(&ruta).unwrap_or_else(|e| panic!("{}: {e}", ruta.display()));
    sin_crlf(&texto)
}

/// Fines de linea LF, con independencia de `core.autocrlf` en el clon.
fn sin_crlf(texto: &str) -> String {
    texto.replace("\r\n", "\n")
}

fn plantilla(texto: &str) -> (Nodo, Nodo) {
    partir_plantilla(texto).unwrap_or_else(|e| panic!("{e}"))
}

const WORKFLOW: &str = ".github/workflows/verificacion.yml";
const PLANTILLA: &str = "plantillas/gitlab/verificacion.gitlab-ci.yml";

#[test]
fn el_lector_entiende_el_subconjunto() {
    let doc = "\
a: 1 # comentario
b:
  - c: x
    d: 'y # no'
  - z
e: { k: v, j: w }
f: [p, q]
g: |
  linea # dentro
    sangrada

  tras vacia
h:
- i
";
    let raiz = yaml(doc);
    assert_eq!(raiz.get("a"), Some(&Nodo::Escalar("1".into())));
    let Some(Nodo::Lista(b)) = raiz.get("b") else {
        panic!("b no es lista: {raiz:?}");
    };
    assert_eq!(b[0].get("d"), Some(&Nodo::Escalar("y # no".into())));
    assert_eq!(b[1], Nodo::Escalar("z".into()));
    assert_eq!(
        raiz.get("e").and_then(|e| e.get("j")),
        Some(&Nodo::Escalar("w".into()))
    );
    assert_eq!(
        raiz.get("f"),
        Some(&Nodo::Lista(vec![
            Nodo::Escalar("p".into()),
            Nodo::Escalar("q".into())
        ]))
    );
    assert_eq!(
        raiz.get("g").and_then(Nodo::texto),
        Some("linea # dentro\n  sangrada\n\ntras vacia")
    );
    assert_eq!(
        raiz.get("h"),
        Some(&Nodo::Lista(vec![Nodo::Escalar("i".into())]))
    );
}

#[test]
fn el_workflow_reutilizable_pasa_el_lint() {
    let texto = leer(WORKFLOW);
    let raiz = yaml(&texto);
    // El lector ve lo que hay que ver: tres jobs con sus pasos.
    let jobs = raiz.get("jobs").map(Nodo::claves).unwrap_or_default();
    assert_eq!(jobs, ["buscar", "ejecutar", "verificacion"]);
    assert_eq!(lint_github(&texto), Vec::<String>::new());
}

#[test]
fn la_plantilla_de_gitlab_pasa_el_lint() {
    let texto = leer(PLANTILLA);
    let (_, cuerpo) = plantilla(&texto);
    assert!(cuerpo.claves().contains(&"verificacion-completa"));
    assert_eq!(lint_gitlab(&texto), Vec::<String>::new());
}

/// Las entradas comunes son las mismas en los dos: lo propio de cada
/// plataforma es el runner (`runner-*` y `shell` en GitHub, `tags-*` y
/// `stage` en GitLab).
#[test]
fn workflow_y_plantilla_tienen_las_mismas_entradas() {
    let raiz = yaml(&leer(WORKFLOW));
    let github: BTreeSet<&str> = raiz
        .get("on")
        .and_then(|o| o.get("workflow_call"))
        .and_then(|w| w.get("inputs"))
        .map(Nodo::claves)
        .unwrap_or_default()
        .into_iter()
        .filter(|k| !k.starts_with("runner-") && *k != "shell")
        .collect();
    let texto = leer(PLANTILLA);
    let (cabecera, _) = plantilla(&texto);
    let gitlab: BTreeSet<&str> = cabecera
        .get("spec")
        .and_then(|s| s.get("inputs"))
        .map(Nodo::claves)
        .unwrap_or_default()
        .into_iter()
        .filter(|k| !k.starts_with("tags-") && *k != "stage")
        .collect();
    assert_eq!(github.len(), 6, "{github:?}");
    assert_eq!(github, gitlab);
}

/// Un workflow minimo que cumple todo: el caso positivo de las mutaciones.
const BUENO: &str = "\
on:
  pull_request:
permissions: {}
jobs:
  ejecutar:
    permissions:
      contents: read
    env:
      BASE: ${{ github.base_ref }}
      TITULO: ${{ github.event.pull_request.title }}
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - run: echo \"$TITULO\"
  publicar:
    permissions:
      checks: write
    steps:
      - env:
          GH_TOKEN: ${{ github.token }}
        run: |
          # comentario con secrets en prosa no cuenta: es un bloque
          gh api repos/x/check-runs
";

#[test]
fn el_caso_positivo_pasa() {
    assert_eq!(lint_github(BUENO), Vec::<String>::new());
}

#[test]
fn cada_mutacion_prohibida_da_su_error() {
    let casos: [(&str, &str, &str); 11] = [
        (
            "  pull_request:\n",
            "  pull_request_target:\n",
            "evento no admitido: pull_request_target",
        ),
        (
            "  pull_request:\n",
            "  pull_request:\n  workflow_run:\n",
            "evento no admitido: workflow_run",
        ),
        ("permissions: {}\n", "permissions: write-all\n", "raiz"),
        (
            "    permissions:\n      contents: read\n",
            "    runs-on: x\n",
            "job ejecutar: sin bloque permissions",
        ),
        (
            "      - run: echo \"$TITULO\"",
            "      - run: echo \"${{ github.event.pull_request.title }}\"",
            "github.event. en run",
        ),
        (
            "      - run: echo \"$TITULO\"",
            "      - run: git push origin ${{ github.head_ref }}",
            "github.head_ref en run",
        ),
        (
            "      BASE: ${{ github.base_ref }}",
            "      BASE: ${{ secrets.TOKEN }}",
            "secreto en clase ejecuta",
        ),
        (
            "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1",
            "actions/checkout@v4",
            "sin fijar por SHA",
        ),
        (
            "          persist-credentials: false\n",
            "          fetch-depth: 0\n",
            "persist-credentials",
        ),
        (
            "      - env:\n          GH_TOKEN",
            "      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1\n        with:\n          persist-credentials: false\n          ref: ${{ github.event.pull_request.head.sha }}\n      - env:\n          GH_TOKEN",
            "ref o repository en clase privilegio",
        ),
        (
            "      - run: echo \"$TITULO\"",
            "      - uses: actions/github-script@60a0d83039c74a4aee543508d2ffcb1c3799cdea",
            "accion no admitida: actions/github-script",
        ),
    ];
    for (original, mutado, esperado) in casos {
        assert!(BUENO.contains(original), "el caso no casa: {original}");
        let texto = BUENO.replacen(original, mutado, 1);
        let errores = lint_github(&texto);
        assert!(
            errores.iter().any(|e| e.contains(esperado)),
            "«{esperado}» no salio; errores: {errores:?}"
        );
    }
    let llamador = "on:\n  push:\npermissions: {}\njobs:\n  v:\n    permissions:\n      contents: read\n    uses: o/r/.github/workflows/x.yml@0000000000000000000000000000000000000000\n    secrets: inherit\n";
    assert!(
        lint_github(llamador)
            .iter()
            .any(|e| e.contains("secrets: inherit")),
        "{:?}",
        lint_github(llamador)
    );
}

#[test]
fn la_plantilla_de_gitlab_rechaza_datos_del_autor_y_secretos() {
    let texto = leer(PLANTILLA);
    let original = "    - $[[ inputs.comando ]] $[[ inputs.argumentos-completa ]]\n";
    assert!(texto.contains(original));
    let mutado = texto.replacen(
        original,
        "    - echo \"$CI_COMMIT_MESSAGE\"\n    - $[[ inputs.comando ]]\n",
        1,
    );
    let errores = lint_gitlab(&mutado);
    assert!(
        errores
            .iter()
            .any(|e| e.contains("verificacion-completa: CI_COMMIT_MESSAGE en script")),
        "{errores:?}"
    );
    let con_secreto = texto.replacen(
        "verificacion-pesada:\n",
        "verificacion-pesada:\n  secrets:\n    CLAVE:\n      vault: x/y@z\n",
        1,
    );
    assert!(
        lint_gitlab(&con_secreto)
            .iter()
            .any(|e| e.contains("verificacion-pesada: secrets")),
        "{:?}",
        lint_gitlab(&con_secreto)
    );
    assert!(lint_gitlab("a:\n  script: [x]\n").contains(&"sin spec:inputs".to_owned()));
}

/// Un clon de Windows con `core.autocrlf=true` (si `.gitattributes` no
/// aplica) da los `.yml` con CRLF: `leer` los deja en LF y las pruebas que
/// buscan lineas literales siguen valiendo.
#[test]
fn leer_normaliza_crlf() {
    let con_crlf = leer(PLANTILLA).replace('\n', "\r\n");
    let texto = sin_crlf(&con_crlf);
    assert!(!texto.contains('\r'));
    assert!(texto.contains("    - $[[ inputs.comando ]] $[[ inputs.argumentos-completa ]]\n"));
    assert_eq!(lint_gitlab(&texto), Vec::<String>::new());
}

/// Lo que el lector no entiende no se descarta en silencio: el lint falla.
#[test]
fn el_lector_falla_en_cerrado() {
    let continuacion = BUENO.replacen(
        "      - run: echo \"$TITULO\"\n",
        "      - run: echo uno\n          dos\n  malo:\n    permissions: write-all\n    steps:\n      - run: echo ${{ github.event.issue.title }}\n",
        1,
    );
    let sangria_impar = BUENO.replacen(
        "      - run: echo \"$TITULO\"\n",
        "      - run: echo a\n         b\n  otro:\n    permissions: write-all\n",
        1,
    );
    let sin_dos_puntos = BUENO.replacen("permissions: {}\n", "permissions: {}\nsuelto\n", 1);
    for texto in [continuacion, sangria_impar, sin_dos_puntos] {
        assert_ne!(texto, BUENO);
        let errores = lint_github(&texto);
        assert!(
            errores.iter().any(|e| e.contains("documento no entendido")),
            "{errores:?}"
        );
    }
}

/// `env` y `defaults` de la raiz, corchetes y espacios en las expresiones.
#[test]
fn el_lint_ve_la_raiz_y_las_formas_alternativas() {
    let casos: [(&str, &str, &str); 5] = [
        (
            "permissions: {}\n",
            "permissions: {}\nenv:\n  CLAVE: ${{ secrets.TOKEN }}\n",
            "raiz: secreto en env",
        ),
        (
            "permissions: {}\n",
            "permissions: {}\ndefaults:\n  run:\n    working-directory: ${{ github.head_ref }}\n",
            "raiz: github.head_ref en defaults",
        ),
        (
            "      - run: echo \"$TITULO\"",
            "      - run: echo ${{ github['event']['pull_request']['title'] }}",
            "github.event. en run",
        ),
        (
            "      BASE: ${{ github.base_ref }}",
            "      BASE: ${{ secrets['TOKEN'] }}",
            "secreto en clase ejecuta",
        ),
        (
            "      - run: echo \"$TITULO\"",
            "      - run: echo '${{ toJSON( github) }}'",
            "tojson(github en run",
        ),
    ];
    for (original, mutado, esperado) in casos {
        assert!(BUENO.contains(original), "el caso no casa: {original}");
        let errores = lint_github(&BUENO.replacen(original, mutado, 1));
        assert!(
            errores.iter().any(|e| e.contains(esperado)),
            "«{esperado}» no salio; errores: {errores:?}"
        );
    }
}

/// Un job que llama a otro workflow reutilizable lo fija por SHA.
#[test]
fn uses_a_nivel_de_job_exige_sha() {
    let llamador = |version: &str| {
        format!(
            "on:\n  push:\npermissions: {{}}\njobs:\n  v:\n    permissions:\n      contents: read\n    uses: o/r/.github/workflows/x.yml@{version}\n"
        )
    };
    assert_eq!(
        lint_github(&llamador("0123456789abcdef0123456789abcdef01234567")),
        Vec::<String>::new()
    );
    let errores = lint_github(&llamador("v1"));
    assert!(
        errores
            .iter()
            .any(|e| e.contains("job v: workflow reutilizable sin fijar por SHA")),
        "{errores:?}"
    );
}

/// El grupo de concurrencia incluye la PR o la ref: solo se encolan las
/// ejecuciones del mismo PR, no las de otros.
#[test]
fn la_concurrencia_es_por_pr_o_ref() {
    let raiz = yaml(&leer(WORKFLOW));
    let grupo = raiz
        .get("jobs")
        .and_then(|j| j.get("ejecutar"))
        .and_then(|e| e.get("concurrency"))
        .and_then(|c| c.get("group"))
        .and_then(Nodo::texto)
        .unwrap_or_default();
    assert!(
        grupo.contains("github.event.pull_request.number || github.ref"),
        "{grupo}"
    );
}

/// `rules:if` no ve las `variables:` de los jobs: la plantilla no las usa en
/// las reglas, y el lint lo detecta si alguien lo vuelve a hacer.
#[test]
fn las_reglas_de_gitlab_no_leen_variables_de_job() {
    let texto = leer(PLANTILLA);
    let (_, cuerpo) = plantilla(&texto);
    assert!(cuerpo.claves().contains(&"variables"));
    let mutado = texto.replacen(
        "    TC2_PERFIL: rapida\n",
        "    TC2_PERFIL: rapida\n    TC2_SOLO: rapida\n",
        1,
    );
    let mutado = mutado.replacen(
        "    - if: $TC2_PERFILES =~ / rapida /\n",
        "    - if: $TC2_SOLO == \"rapida\"\n",
        1,
    );
    assert_ne!(mutado, texto);
    let errores = lint_gitlab(&mutado);
    assert!(
        errores
            .iter()
            .any(|e| e.contains("verificacion-rapida: rules:if lee la variable de job TC2_SOLO")),
        "{errores:?}"
    );
}
