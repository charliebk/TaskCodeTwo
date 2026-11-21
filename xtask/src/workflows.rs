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
//!   `read-all`; un job con algun `write` es de clase privilegio; solo se
//!   leen `contents`, `checks` y `actions` (estos dos, para `buscar` de F5-2);
//! - acciones admitidas: `actions/checkout`, fijada por SHA, con
//!   `persist-credentials: false`, sin `clean:` distinto de `true` (F5-5) y
//!   nunca en clase privilegio; y
//!   `actions/upload-artifact` (informe de F5-4), fijada por SHA y solo en
//!   clase ejecuta;
//! - ni `secrets: inherit` (salvo el llamador de `revision-ia.yml`, perfil
//!   [`LLAMADOR_REVISION`]), ni secretos en clase ejecuta, ni
//!   `github.event.*`, `github.head_ref` o `toJSON(github...)` en `run:`
//!   (los datos del evento se pasan por `env:`).
//!
//! El workflow del revisor IA (`revision-ia.yml`, F6-3) pasa el mismo lint
//! con su perfil ([`REVISION`]): solo `workflow_call`; un job con
//! `environment:` es de clase privilegio aunque su GITHUB_TOKEN no escriba
//! (sus secretos estan ahi), y el unico entorno admitido es `revisor`;
//! ningun job escribe con GITHUB_TOKEN (el check lo publica la App del
//! revisor); y nada saca el head del PR: cada checkout lleva `ref:` sin
//! `head`, y `run:` no lleva `git checkout`, `switch`, `worktree`, `restore`,
//! `reset` ni `inputs.`.
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
        } else if resto.starts_with('|') || resto.starts_with('>') {
            // `- |`: un item que es un bloque literal (after_script de F5-2).
            *i += 1;
            items.push(Nodo::Escalar(bloque_literal(lineas, i, sangria)));
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

/// Subida del informe JSON (F5-4): solo en clase ejecuta (sin escritura ni
/// secretos), fijada por SHA.
const SUBIDA: &str = "actions/upload-artifact";

const ACCIONES_ADMITIDAS: [&str; 2] = ["actions/checkout", SUBIDA];

/// Alcances que un job puede leer: el codigo, y (F5-2, `buscar`) los
/// check-runs y las ejecuciones donde vive la cache por arbol.
const LECTURAS_ADMITIDAS: [&str; 3] = ["contents", "checks", "actions"];

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

/// Lo que cambia entre los workflows de la lista blanca: el de verificacion
/// (F5-1) y el del revisor IA (F6-3).
struct Perfil {
    /// Eventos admitidos.
    eventos: &'static [&'static str],
    /// Entornos de despliegue admitidos (`environment:` de un job). Un job
    /// con entorno es de clase privilegio aunque su GITHUB_TOKEN no escriba:
    /// sus secretos estan ahi (F6-3).
    entornos: &'static [&'static str],
    /// Ningun job escribe con GITHUB_TOKEN (lo que se publica va con una
    /// App).
    sin_escritura: bool,
    /// Ningun checkout del head del PR (ni `ref:` con `head`, ni `git
    /// checkout`, `switch`, `worktree`, `restore` o `reset` en `run:`), y
    /// ninguna `inputs.` en `run:` (la rama del PR llega por una entrada).
    sin_head: bool,
    /// Unico workflow reutilizable al que un job puede llamar con `secrets:
    /// inherit` (prefijo de `uses:` hasta la `@`, fijado por SHA). GitHub
    /// solo da los secretos del entorno `revisor` al job de `revision-ia.yml`
    /// si el llamador hereda (E2E de TASK-049; adenda de F1-8).
    herencia_admitida: Option<&'static str>,
}

/// `verificacion.yml` (F5-1, F5-2).
const VERIFICACION: Perfil = Perfil {
    eventos: &EVENTOS_ADMITIDOS,
    entornos: &[],
    sin_escritura: false,
    sin_head: false,
    herencia_admitida: None,
};

/// `revision-ia.yml` (F6-3): solo reutilizable, secretos solo en el job del
/// entorno `revisor`, sin escritura del GITHUB_TOKEN y sin el head.
const REVISION: Perfil = Perfil {
    eventos: &["workflow_call"],
    entornos: &["revisor"],
    sin_escritura: true,
    sin_head: true,
    herencia_admitida: None,
};

/// El llamador de `revision-ia.yml` en el proyecto (F6-3): solo
/// `workflow_run`, sin escritura del GITHUB_TOKEN y con `secrets: inherit`
/// unicamente hacia el `revision-ia.yml` de TaskCodeTwo fijado por SHA.
const LLAMADOR_REVISION: Perfil = Perfil {
    eventos: &["workflow_run"],
    entornos: &[],
    sin_escritura: true,
    sin_head: false,
    herencia_admitida: Some("charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml"),
};

/// Ordenes de git que en `run:` cambiarian el arbol de trabajo a otro commit.
const CAMBIAN_EL_ARBOL: [&str; 5] = ["checkout", "switch", "worktree", "restore", "reset"];

fn lint_github(texto: &str) -> Vec<String> {
    lint_con(texto, &VERIFICACION)
}

fn lint_revision(texto: &str) -> Vec<String> {
    lint_con(texto, &REVISION)
}

#[cfg(test)]
fn lint_llamador_revision(texto: &str) -> Vec<String> {
    lint_con(texto, &LLAMADOR_REVISION)
}

/// `secrets: inherit` solo hacia el reutilizable que admite el perfil, y
/// fijado por SHA.
fn hereda_admitido(job: &Nodo, perfil: &Perfil) -> bool {
    let (Some(admitido), Some(uses)) = (
        perfil.herencia_admitida,
        job.get("uses").and_then(Nodo::texto),
    ) else {
        return false;
    };
    uses.split_once('@')
        .is_some_and(|(destino, version)| destino == admitido && es_sha(version))
}

fn lint_con(texto: &str, perfil: &Perfil) -> Vec<String> {
    let raiz = match leer_yaml(texto) {
        Ok(r) => r,
        Err(e) => return vec![e],
    };
    let mut errores = Vec::new();
    match raiz.get("on") {
        None => errores.push("sin bloque on".to_owned()),
        Some(on) => {
            for e in eventos(on) {
                if !perfil.eventos.contains(&e.as_str()) {
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
        let escritura = match escribe(job.get("permissions")) {
            Ok(p) => p,
            Err(e) => {
                errores.push(format!("job {nombre}: {e}"));
                false
            }
        };
        if escritura && perfil.sin_escritura {
            errores.push(format!("job {nombre}: escritura con GITHUB_TOKEN"));
        }
        let entorno = job.get("environment");
        if let Some(n) = entorno {
            let valor = n.texto().or_else(|| n.get("name").and_then(Nodo::texto));
            if !valor.is_some_and(|v| perfil.entornos.contains(&v)) {
                errores.push(format!("job {nombre}: entorno no admitido: {valor:?}"));
            }
        }
        let privilegio = escritura || entorno.is_some();
        if let Some(Nodo::Mapa(pares)) = job.get("permissions") {
            for (alcance, valor) in pares {
                if valor.texto() == Some("read") && !LECTURAS_ADMITIDAS.contains(&alcance.as_str())
                {
                    errores.push(format!("job {nombre}: lectura no admitida: {alcance}"));
                }
            }
        }
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
            if *clave == "secrets" && *valor == "inherit" && !hereda_admitido(job, perfil) {
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
                if perfil.sin_head {
                    if normalizado.contains("inputs.") {
                        errores.push(format!("job {nombre}: inputs. en run"));
                    }
                    // `git [-C dir] checkout ...`: la orden de git en
                    // cualquier posicion de una linea que llama a git.
                    for linea in valor.lines() {
                        let t: Vec<&str> = linea.split_whitespace().collect();
                        if !t.contains(&"git") {
                            continue;
                        }
                        for orden in CAMBIAN_EL_ARBOL {
                            if t.contains(&orden) {
                                errores.push(format!("job {nombre}: git {orden} en run"));
                            }
                        }
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
            if accion == SUBIDA {
                if privilegio {
                    errores.push(format!("job {nombre}: {SUBIDA} en clase privilegio"));
                }
                continue;
            }
            let con = paso.get("with");
            let opcion = |k: &str| con.and_then(|w| w.get(k));
            if opcion("persist-credentials").and_then(Nodo::texto) != Some("false") {
                errores.push(format!(
                    "job {nombre}: checkout sin persist-credentials: false"
                ));
            }
            // F1-8, decision 6 (D3b b), adenda F5-5: el `clean: true` por
            // defecto deja el directorio limpio en cada job del runner propio.
            if opcion("clean").is_some_and(|c| c.texto() != Some("true")) {
                errores.push(format!("job {nombre}: checkout sin clean: true"));
            }
            // F1-8: un job con privilegio nunca hace checkout (en un PR, sin
            // `ref:` saca el merge ref, que es codigo del PR).
            if privilegio {
                errores.push(format!("job {nombre}: checkout en clase privilegio"));
            }
            if privilegio && (opcion("ref").is_some() || opcion("repository").is_some()) {
                errores.push(format!(
                    "job {nombre}: checkout con ref o repository en clase privilegio"
                ));
            }
            // F6-3: la base y el plugin se sacan; el head del PR, nunca (sin
            // `ref:` en un evento de PR sale el merge ref, codigo del PR).
            if perfil.sin_head {
                match opcion("ref").and_then(Nodo::texto) {
                    Some(r) if normalizar(r).contains("head") => {
                        errores.push(format!("job {nombre}: checkout del head del PR"));
                    }
                    Some(_) => {}
                    None => errores.push(format!("job {nombre}: checkout sin ref fija")),
                }
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
j:
  - |
    if x; then
      y
    fi
";
    let raiz = yaml(doc);
    assert_eq!(
        raiz.get("j"),
        Some(&Nodo::Lista(vec![Nodo::Escalar(
            "if x; then\n  y\nfi".into()
        )]))
    );
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
    assert_eq!(github.len(), 9, "{github:?}");
    assert_eq!(github, gitlab);
}

// --- Cola de merge (F5-3) ---------------------------------------------------------

/// El `run:` del paso `nombre` del job `job` del workflow, tal cual.
fn run_del_paso(texto: &str, job: &str, nombre: &str) -> String {
    let raiz = yaml(texto);
    let Some(Nodo::Lista(pasos)) = raiz
        .get("jobs")
        .and_then(|j| j.get(job))
        .and_then(|j| j.get("steps"))
    else {
        panic!("job {job} sin pasos");
    };
    pasos
        .iter()
        .find(|p| p.get("name").and_then(Nodo::texto) == Some(nombre))
        .and_then(|p| p.get("run"))
        .and_then(Nodo::texto)
        .unwrap_or_else(|| panic!("job {job}: sin paso {nombre}"))
        .to_owned()
}

/// El `bash` de los runners de GitHub. En Windows, `CreateProcess` mira
/// `System32` antes que el `PATH` y daria el `bash.exe` de WSL: se busca el
/// de Git en el `PATH` y, si no esta, junto al `git` instalado.
fn ruta_bash() -> PathBuf {
    if !cfg!(windows) {
        return PathBuf::from("bash");
    }
    let sistema = std::env::var("SystemRoot")
        .unwrap_or_else(|_| r"C:\Windows".to_owned())
        .to_lowercase();
    let en_path = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|d| d.join("bash.exe"))
        .find(|c| {
            let texto = c.to_string_lossy().to_lowercase();
            c.is_file() && !texto.starts_with(&sistema) && !texto.contains("windowsapps")
        });
    if let Some(ruta) = en_path {
        return ruta;
    }
    // `git --exec-path` = <git>/mingw64/libexec/git-core; bash en <git>/bin.
    let salida = std::process::Command::new("git")
        .arg("--exec-path")
        .output()
        .unwrap_or_else(|e| panic!("sin bash en el PATH ni git: {e}"));
    let nucleo = PathBuf::from(String::from_utf8_lossy(&salida.stdout).trim());
    let ruta = nucleo
        .ancestors()
        .nth(3)
        .map(|g| g.join("bin").join("bash.exe"))
        .unwrap_or_default();
    assert!(ruta.is_file(), "sin bash de Git: {}", ruta.display());
    ruta
}

/// Ejecuta con `bash` el `run:` de «Perfiles del evento» extraido de `texto`,
/// con el entorno del paso simulado, y devuelve lo que escribe en
/// `GITHUB_OUTPUT` (o el error y el codigo de salida).
fn perfiles_del_evento(
    texto: &str,
    evento: &str,
    base: &str,
    origen: &str,
    cola: &str,
) -> Result<String, String> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let script = run_del_paso(texto, "buscar", "Perfiles del evento");
    let salida = std::env::temp_dir().join(format!(
        "tc2-xtask-perfiles-{}-{}.txt",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&salida, "").unwrap_or_else(|e| panic!("{}: {e}", salida.display()));
    let proceso = std::process::Command::new(ruta_bash())
        .arg("-c")
        .arg(&script)
        .env("EVENTO", evento)
        .env("BASE", base)
        .env("ORIGEN", origen)
        .env("FORZADOS", "")
        .env("COLA", cola)
        .env("GITHUB_OUTPUT", &salida)
        .output()
        .unwrap_or_else(|e| panic!("bash no arranca (hace falta bash en el PATH): {e}"));
    let escrito = std::fs::read_to_string(&salida).unwrap_or_default();
    let _ = std::fs::remove_file(&salida);
    if proceso.status.success() {
        Ok(escrito.trim().to_owned())
    } else {
        Err(format!(
            "{:?}: {}",
            proceso.status.code(),
            String::from_utf8_lossy(&proceso.stderr).trim()
        ))
    }
}

/// Adenda F5-3 de F1-7 (D1): con `cola-de-merge` el PR corre la impacto y el
/// `merge_group` la completa; la release sigue con completa y pesada, y con
/// `false` cada evento da lo mismo que antes de la entrada. Se ejecuta el
/// `case` real del workflow.
#[test]
fn la_cola_de_merge_elige_los_perfiles_del_evento() {
    let texto = leer(WORKFLOW);
    let casos: [(&str, &str, &str, &str, &str); 10] = [
        // (evento, base, origen, cola, perfiles)
        (
            "pull_request",
            "develop",
            "feature/x",
            "false",
            r#"["completa"]"#,
        ),
        (
            "pull_request",
            "main",
            "develop",
            "false",
            r#"["completa","pesada"]"#,
        ),
        (
            "pull_request",
            "main",
            "hotfix/x",
            "false",
            r#"["completa"]"#,
        ),
        ("merge_group", "", "", "false", r#"["completa"]"#),
        ("push", "", "", "false", "[]"),
        (
            "pull_request",
            "develop",
            "feature/x",
            "true",
            r#"["impacto"]"#,
        ),
        (
            "pull_request",
            "main",
            "develop",
            "true",
            r#"["completa","pesada"]"#,
        ),
        ("pull_request", "main", "hotfix/x", "true", r#"["impacto"]"#),
        ("merge_group", "", "", "true", r#"["completa"]"#),
        ("workflow_dispatch", "", "", "true", r#"["completa"]"#),
    ];
    for (evento, base, origen, cola, esperado) in casos {
        assert_eq!(
            perfiles_del_evento(&texto, evento, base, origen, cola),
            Ok(format!("perfiles={esperado}")),
            "{evento} {base}<-{origen} cola={cola}"
        );
    }
    assert!(
        perfiles_del_evento(&texto, "pull_request_target", "", "", "true").is_err(),
        "un evento no admitido falla"
    );
}

// --- Runner propio (F5-5) ----------------------------------------------------------

/// Ejecuta con `bash` el `run:` de «Runner propio» extraido de `texto`, con el
/// entorno del paso simulado; devuelve lo que imprime o el codigo de salida y
/// el error.
fn runner_propio(texto: &str, entorno: &[(&str, &str)]) -> Result<String, String> {
    let script = run_del_paso(texto, "buscar", "Runner propio");
    let proceso = std::process::Command::new(ruta_bash())
        .arg("-c")
        .arg(&script)
        .envs(entorno.iter().copied())
        .output()
        .unwrap_or_else(|e| panic!("bash no arranca (hace falta bash en el PATH): {e}"));
    if proceso.status.success() {
        Ok(String::from_utf8_lossy(&proceso.stdout).trim().to_owned())
    } else {
        Err(format!(
            "{:?}: {}",
            proceso.status.code(),
            String::from_utf8_lossy(&proceso.stderr).trim()
        ))
    }
}

/// Entradas del workflow con el runner de la pesada dado (JSON de runs-on) y
/// la completa en un runner de GitHub.
fn entradas_con_pesada(runner: &str) -> String {
    format!(
        r#"{{"runner-completa":"\"ubuntu-latest\"","runner-pesada":{}}}"#,
        cadena_json(runner)
    )
}

/// `texto` como cadena JSON (comillas y barras escapadas): basta para los
/// runs-on de estas pruebas.
fn cadena_json(texto: &str) -> String {
    format!("\"{}\"", texto.replace('\\', "\\\\").replace('"', "\\\""))
}

/// F1-8, decision 6 (D3b), y adenda F5-5: el runner propio solo en la release
/// de develop a main de un repo privado con el head en el mismo repo; en
/// cualquier otro caso `buscar` falla cerrado con exit 2 y su motivo. Un
/// perfil en un runner de GitHub no se ve afectado en ningun evento. Se
/// ejecuta el `run:` real del workflow.
#[test]
fn el_runner_propio_solo_en_la_release_de_un_privado() {
    let texto = leer(WORKFLOW);
    let release: [(&str, &str); 8] = [
        ("PERFILES", r#"["completa","pesada"]"#),
        ("ENTRADAS", ""),
        ("EVENTO", "pull_request"),
        ("PRIVADO", "true"),
        ("BASE", "main"),
        ("ORIGEN", "develop"),
        ("REPO", "o/r"),
        ("REPO_HEAD", "o/r"),
    ];
    fn cambiar<'a>(
        base: &[(&'a str, &'a str)],
        cambios: &[(&'a str, &'a str)],
    ) -> Vec<(&'a str, &'a str)> {
        let mut e = base.to_vec();
        for &(k, v) in cambios {
            if let Some(par) = e.iter_mut().find(|(x, _)| *x == k) {
                par.1 = v;
            }
        }
        e
    }
    let propio = entradas_con_pesada(r#"["self-hosted","windows","tc2-e2e"]"#);
    let github = entradas_con_pesada(r#""ubuntu-latest""#);
    let malos = [
        ("repo publico", vec![("PRIVADO", "false")], "repo publico"),
        (
            "PR a develop",
            vec![
                ("PERFILES", r#"["pesada"]"#),
                ("BASE", "develop"),
                ("ORIGEN", "feature/x"),
            ],
            "solo en la release",
        ),
        (
            "workflow_dispatch",
            vec![
                ("PERFILES", r#"["pesada"]"#),
                ("EVENTO", "workflow_dispatch"),
                ("BASE", ""),
                ("ORIGEN", ""),
                ("REPO_HEAD", ""),
            ],
            "solo en la release",
        ),
        (
            "merge_group",
            vec![
                ("PERFILES", r#"["pesada"]"#),
                ("EVENTO", "merge_group"),
                ("BASE", ""),
                ("ORIGEN", ""),
                ("REPO_HEAD", ""),
            ],
            "solo en la release",
        ),
        (
            "head de un fork",
            vec![("REPO_HEAD", "otro/r")],
            "viene de otro repo (otro/r)",
        ),
        (
            "hotfix",
            vec![("PERFILES", r#"["pesada"]"#), ("ORIGEN", "hotfix/x")],
            "solo en la release",
        ),
        (
            "develop a otra rama",
            vec![("PERFILES", r#"["pesada"]"#), ("BASE", "release")],
            "solo en la release",
        ),
        // Cada condicion por separado: con base y origen de la release, otro
        // evento (con `perfiles` forzados) tampoco pasa.
        (
            "otro evento con base y origen de la release",
            vec![("PERFILES", r#"["pesada"]"#), ("EVENTO", "merge_group")],
            "solo en la release",
        ),
    ];
    // La release de un privado del mismo repo pasa; la etiqueta no distingue
    // mayusculas, y un runs-on que no es cadena ni lista (grupo) cuenta como
    // runner propio.
    for runner in [
        r#"["self-hosted","windows","tc2-e2e"]"#,
        r#"["Self-Hosted","windows"]"#,
        r#""self-hosted""#,
    ] {
        let entradas = entradas_con_pesada(runner);
        let salida = runner_propio(&texto, &cambiar(&release, &[("ENTRADAS", &entradas)]));
        assert_eq!(
            salida,
            Ok("Runner propio en pesada: release de develop a main en o/r (privado)".to_owned()),
            "{runner}"
        );
    }
    for (caso, cambios, motivo) in &malos {
        let mut e = cambios.clone();
        e.push(("ENTRADAS", &propio));
        let salida = runner_propio(&texto, &cambiar(&release, &e));
        assert!(
            matches!(&salida, Err(m) if m.starts_with("Some(2): ") && m.contains(motivo) && m.contains("runner propio en pesada")),
            "{caso}: {salida:?}"
        );
        // Con la pesada en un runner de GitHub, el mismo evento sigue igual.
        let mut e = cambios.clone();
        e.push(("ENTRADAS", &github));
        assert_eq!(
            runner_propio(&texto, &cambiar(&release, &e)),
            Ok("Runner propio: ningun perfil de este evento lo usa".to_owned()),
            "{caso} con runner de GitHub"
        );
    }
    let grupo = entradas_con_pesada(r#"{"group":"propios"}"#);
    assert!(
        matches!(runner_propio(&texto, &cambiar(&release, &[("ENTRADAS", &grupo), ("PRIVADO", "false")])), Err(m) if m.contains("repo publico")),
        "un grupo de runners cuenta como runner propio"
    );
    let ilegible = entradas_con_pesada("no-es-json");
    assert!(
        matches!(runner_propio(&texto, &cambiar(&release, &[("ENTRADAS", &ilegible)])), Err(m) if m.starts_with("Some(2): ") && m.contains("runner-pesada ilegible")),
        "un runs-on ilegible falla cerrado"
    );
    // Sin perfiles (push) no hay nada que mirar.
    assert_eq!(
        runner_propio(
            &texto,
            &cambiar(
                &release,
                &[
                    ("PERFILES", "[]"),
                    ("EVENTO", "push"),
                    ("ENTRADAS", &propio)
                ]
            )
        ),
        Ok("Runner propio: ningun perfil de este evento lo usa".to_owned())
    );
}

/// Windows PowerShell sin perfil y sin la directiva de la maquina, solo para el
/// proceso del paso: con `AllSigned` (E2E de TASK-045) el `.ps1` sin firmar que
/// genera el runner no llega a ejecutarse y el aislamiento no comprueba nada.
const SHELL_AISLAMIENTO: &str =
    r#"powershell -NoProfile -ExecutionPolicy Bypass -Command ". '{0}'""#;

/// El paso de aislamiento es el primero de `ejecutar` (antes del checkout),
/// solo en un runner propio y en Windows PowerShell; `Err` con lo que falla.
fn aislamiento_antes_del_checkout(texto: &str) -> Result<(), String> {
    let raiz = yaml(texto);
    let Some(Nodo::Lista(pasos)) = raiz
        .get("jobs")
        .and_then(|j| j.get("ejecutar"))
        .and_then(|j| j.get("steps"))
    else {
        return Err("ejecutar sin pasos".to_owned());
    };
    let primero = pasos.first().ok_or("ejecutar sin pasos")?;
    let campo = |k: &str| primero.get(k).and_then(Nodo::texto);
    if campo("name") != Some("aislamiento del runner propio") {
        return Err(format!(
            "el primer paso de ejecutar no es el aislamiento: {:?}",
            campo("name").or(campo("uses"))
        ));
    }
    if campo("if") != Some("runner.environment == 'self-hosted'") {
        return Err(format!("aislamiento con if {:?}", campo("if")));
    }
    if campo("shell") != Some(SHELL_AISLAMIENTO) {
        return Err(format!("aislamiento con shell {:?}", campo("shell")));
    }
    let run = campo("run").unwrap_or_default();
    // whoami por ruta: el de Git (usr\bin) en el PATH no entiende /groups.
    for exigido in [
        "S-1-5-18",
        "S-1-5-32-544",
        r#"System32\whoami.exe" /groups"#,
        "RUNNER_OS",
        "Get-Command jq",
    ] {
        if !run.contains(exigido) {
            return Err(format!("aislamiento sin {exigido}"));
        }
    }
    Ok(())
}

/// F1-8, decision 6 (D3b a): en un runner propio, antes de sacar codigo, se
/// comprueba que es Windows y que la cuenta no es SYSTEM ni administradora.
/// Quitar el paso, ponerlo despues del checkout o cambiar su condicion o su
/// shell rompe la prueba.
#[test]
fn el_aislamiento_va_antes_del_checkout_en_el_runner_propio() {
    let texto = leer(WORKFLOW);
    assert_eq!(aislamiento_antes_del_checkout(&texto), Ok(()));
    let inicio = texto
        .find("      - name: aislamiento del runner propio\n")
        .unwrap_or_else(|| panic!("sin paso de aislamiento"));
    let checkout = "      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1\n        with:\n          persist-credentials: false\n";
    let fin = inicio
        + texto[inicio..]
            .find(checkout)
            .unwrap_or_else(|| panic!("sin checkout tras el aislamiento"));
    let paso = &texto[inicio..fin];
    let quitado = format!("{}{}", &texto[..inicio], &texto[fin..]);
    let movido = format!(
        "{}{checkout}{paso}{}",
        &texto[..inicio],
        &texto[fin + checkout.len()..]
    );
    let mutaciones = [
        ("quitado", quitado),
        ("despues del checkout", movido),
        (
            "en cualquier runner",
            texto.replacen(
                "if: runner.environment == 'self-hosted'",
                "if: runner.os == 'Windows'",
                1,
            ),
        ),
        (
            "en bash",
            texto.replacen(
                &format!("        shell: {SHELL_AISLAMIENTO}\n"),
                "        shell: bash\n",
                1,
            ),
        ),
        (
            "con la directiva de la maquina",
            texto.replacen(
                &format!("        shell: {SHELL_AISLAMIENTO}\n"),
                "        shell: powershell\n",
                1,
            ),
        ),
        (
            "sin comprobar jq",
            texto.replacen("Get-Command jq", "Get-Command bash", 1),
        ),
        (
            "sin el grupo Administradores",
            texto.replacen("$grupos -match 'S-1-5-32-544'", "$false", 1),
        ),
    ];
    for (caso, mutado) in &mutaciones {
        assert_ne!(mutado, &texto, "{caso}: la mutacion no casa");
        assert!(aislamiento_antes_del_checkout(mutado).is_err(), "{caso}");
    }
}

/// E2E de TASK-045: con una ruta de Windows como argumento, GNU `sha256sum`
/// escapa la barra invertida y antepone `\` a la linea, asi que la huella local
/// de la pesada en el runner propio no casaria nunca con la cache. El workflow
/// solo resume por la entrada estandar.
#[test]
fn sha256sum_solo_por_la_entrada_estandar() {
    let con_ruta = |texto: &str| {
        texto.lines().any(|l| {
            l.split("sha256sum")
                .skip(1)
                .any(|resto| resto.trim_start().starts_with('"'))
        })
    };
    let texto = leer(WORKFLOW);
    assert!(
        texto.contains("sha256sum < \"$HUELLA\""),
        "sin la huella local"
    );
    assert!(!con_ruta(&texto), "sha256sum con una ruta como argumento");
    let mutado = texto.replacen("sha256sum < \"$HUELLA\"", "sha256sum \"$HUELLA\"", 1);
    assert_ne!(mutado, texto, "la mutacion no casa");
    assert!(con_ruta(&mutado), "la mutacion no se detecta");
}

/// D3: la impacto nunca se cachea. `buscar` no le calcula clave (sin clave no
/// hay candidatos) y el publicador no le escribe linea `arbol=`; el resumen
/// lleva `cola-de-merge=`. La entrada es booleana y falsa por defecto en las
/// dos plataformas, y la plantilla la aplica con las reglas de `merge_train`.
#[test]
fn la_impacto_nunca_se_cachea_y_la_cola_es_explicita() {
    let texto = leer(WORKFLOW);
    let candidatos = run_del_paso(&texto, "buscar", "Candidatos de la cache por arbol");
    assert!(
        candidatos.contains(r#"jq -r '.[] | select(. != "impacto")'"#),
        "{candidatos}"
    );
    assert!(candidatos.contains("if [ \"$claves\" = '{}' ]; then"));
    let publicar = run_del_paso(&texto, "verificacion", "Publicar el check verificacion");
    assert!(
        publicar.contains("if [ \"$p\" = impacto ]; then continue; fi"),
        "{publicar}"
    );
    assert!(publicar.contains("\ncola-de-merge=$COLA\""), "{publicar}");
    let raiz = yaml(&texto);
    let entrada = raiz
        .get("on")
        .and_then(|o| o.get("workflow_call"))
        .and_then(|w| w.get("inputs"))
        .and_then(|i| i.get("cola-de-merge"));
    assert_eq!(
        entrada.and_then(|e| e.get("type")).and_then(Nodo::texto),
        Some("boolean")
    );
    assert_eq!(
        entrada.and_then(|e| e.get("default")).and_then(Nodo::texto),
        Some("false")
    );

    let texto_gl = leer(PLANTILLA);
    let (cabecera, cuerpo) = plantilla(&texto_gl);
    let entrada_gl = cabecera
        .get("spec")
        .and_then(|s| s.get("inputs"))
        .and_then(|i| i.get("cola-de-merge"));
    assert_eq!(
        entrada_gl.and_then(|e| e.get("type")).and_then(Nodo::texto),
        Some("boolean")
    );
    assert_eq!(
        entrada_gl
            .and_then(|e| e.get("default"))
            .and_then(Nodo::texto),
        Some("false")
    );
    let reglas = |job: &str| {
        let mut condiciones = Vec::new();
        if let Some(r) = cuerpo.get(job).and_then(|n| n.get("rules")) {
            r.escalares("rules", &mut condiciones);
        }
        condiciones
            .into_iter()
            .filter(|(k, _)| *k == "if")
            .map(|(_, c)| c.to_owned())
            .collect::<Vec<_>>()
    };
    let mr = "$CI_PIPELINE_SOURCE == \"merge_request_event\"";
    assert_eq!(
        reglas("verificacion-impacto"),
        [
            "$TC2_PERFILES =~ / impacto /".to_owned(),
            format!(
                "{mr} && $TC2_COLA_DE_MERGE =~ / true / && $CI_MERGE_REQUEST_EVENT_TYPE != \"merge_train\" && ($CI_MERGE_REQUEST_TARGET_BRANCH_NAME != \"main\" || $CI_MERGE_REQUEST_SOURCE_BRANCH_NAME != \"develop\")"
            ),
        ]
    );
    assert_eq!(
        reglas("verificacion-completa"),
        [
            "$TC2_PERFILES =~ / completa /".to_owned(),
            format!("{mr} && $TC2_COLA_DE_MERGE !~ / true /"),
            format!("{mr} && $CI_MERGE_REQUEST_EVENT_TYPE == \"merge_train\""),
            format!(
                "{mr} && $CI_MERGE_REQUEST_TARGET_BRANCH_NAME == \"main\" && $CI_MERGE_REQUEST_SOURCE_BRANCH_NAME == \"develop\""
            ),
        ]
    );
    let mut despues = Vec::new();
    if let Some(s) = cuerpo
        .get(".tc2-verificacion")
        .and_then(|b| b.get("after_script"))
    {
        s.escalares("after_script", &mut despues);
    }
    assert!(
        despues
            .iter()
            .any(|(_, l)| l.contains("[ \"$TC2_PERFIL\" != impacto ]")),
        "{despues:?}"
    );
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
    let casos: [(&str, &str, &str); 13] = [
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
            "          persist-credentials: false\n",
            "          persist-credentials: false\n          clean: false\n",
            "checkout sin clean: true",
        ),
        (
            "          persist-credentials: false\n",
            "          persist-credentials: false\n          clean: ${{ inputs.limpiar }}\n",
            "checkout sin clean: true",
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

/// F5-4: `ejecutar` sube el informe aunque el comando falle, el publicador
/// nombra los artefactos en el resumen, la plantilla los guarda siempre y la
/// subida no se admite en un job que escribe.
#[test]
fn el_informe_se_sube_siempre_y_solo_desde_ejecutar() {
    let raiz = yaml(&leer(WORKFLOW));
    let job = |n: &str| raiz.get("jobs").and_then(|j| j.get(n));
    let pasos = |n: &str| match job(n).and_then(|j| j.get("steps")) {
        Some(Nodo::Lista(p)) => p.clone(),
        _ => Vec::new(),
    };
    let subida = pasos("ejecutar")
        .into_iter()
        .find(|p| {
            p.get("uses")
                .and_then(Nodo::texto)
                .is_some_and(|u| u.starts_with("actions/upload-artifact@"))
        })
        .unwrap_or_else(|| panic!("ejecutar no sube el informe"));
    assert_eq!(subida.get("if").and_then(Nodo::texto), Some("always()"));
    let con = |k: &str| {
        subida
            .get("with")
            .and_then(|w| w.get(k))
            .and_then(Nodo::texto)
    };
    assert_eq!(
        con("name"),
        Some("informe-verificacion-${{ matrix.perfil }}")
    );
    assert_eq!(con("if-no-files-found"), Some("ignore"));
    let publicar = pasos("verificacion")
        .first()
        .and_then(|p| p.get("run"))
        .and_then(Nodo::texto)
        .map(str::to_owned)
        .unwrap_or_default();
    assert!(publicar.contains("informe=$informes"), "{publicar}");

    let texto = leer(PLANTILLA);
    let (_, cuerpo) = plantilla(&texto);
    let artefactos = cuerpo
        .get(".tc2-verificacion")
        .and_then(|b| b.get("artifacts"));
    assert_eq!(
        artefactos.and_then(|a| a.get("when")).and_then(Nodo::texto),
        Some("always")
    );
    assert!(
        cuerpo
            .get(".tc2-verificacion")
            .and_then(|b| b.get("variables"))
            .and_then(|v| v.get("TC2_INFORME"))
            .is_some()
    );

    let en_privilegio = BUENO.replacen(
        "      - env:\n          GH_TOKEN",
        "      - uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02\n      - env:\n          GH_TOKEN",
        1,
    );
    assert_ne!(en_privilegio, BUENO);
    let errores = lint_github(&en_privilegio);
    assert!(
        errores
            .iter()
            .any(|e| e.contains("job publicar: actions/upload-artifact en clase privilegio")),
        "{errores:?}"
    );
    let sin_sha = BUENO.replacen(
        "      - run: echo \"$TITULO\"",
        "      - uses: actions/upload-artifact@v4",
        1,
    );
    assert!(
        lint_github(&sin_sha)
            .iter()
            .any(|e| e.contains("actions/upload-artifact sin fijar por SHA")),
        "{:?}",
        lint_github(&sin_sha)
    );
}

/// F5-2: `buscar` lee check-runs y ejecuciones sin escribir; el comando se
/// salta con un acierto; el publicador escribe la linea `arbol=` por perfil;
/// la plantilla deja la linea en el artefacto. Mutaciones negativas: `buscar`
/// con escritura (hace checkout: clase privilegio con codigo del PR) y una
/// lectura fuera de la lista.
#[test]
fn la_cache_por_arbol_respeta_las_clases() {
    let texto = leer(WORKFLOW);
    let raiz = yaml(&texto);
    let job = |n: &str| raiz.get("jobs").and_then(|j| j.get(n));
    let permiso = |n: &str, alcance: &str| {
        job(n)
            .and_then(|j| j.get("permissions"))
            .and_then(|p| p.get(alcance))
            .and_then(Nodo::texto)
            .map(str::to_owned)
    };
    assert_eq!(permiso("buscar", "checks").as_deref(), Some("read"));
    assert_eq!(permiso("buscar", "actions").as_deref(), Some("read"));
    assert_eq!(permiso("ejecutar", "checks"), None);
    let pasos = |n: &str| match job(n).and_then(|j| j.get("steps")) {
        Some(Nodo::Lista(p)) => p.clone(),
        _ => Vec::new(),
    };
    let comando = pasos("ejecutar")
        .into_iter()
        .find(|p| {
            p.get("name")
                .and_then(Nodo::texto)
                .is_some_and(|n| n.starts_with("comando del proyecto"))
        })
        .unwrap_or_else(|| panic!("sin paso del comando"));
    assert_eq!(
        comando.get("if").and_then(Nodo::texto),
        Some("steps.cache.outputs.acierto != 'si'")
    );
    let publicar = pasos("verificacion")
        .first()
        .and_then(|p| p.get("run"))
        .and_then(Nodo::texto)
        .map(str::to_owned)
        .unwrap_or_default();
    assert!(
        publicar.contains(
            "arbol=$arbol perfil=$p runner=$r clave=$k local=$huella origen=$origen $estado"
        ),
        "{publicar}"
    );

    let original = "      contents: read\n      checks: read\n";
    assert!(texto.contains(original));
    let con_escritura = texto.replacen(original, "      contents: read\n      checks: write\n", 1);
    let errores = lint_github(&con_escritura);
    assert!(
        errores
            .iter()
            .any(|e| e.contains("job buscar: checkout en clase privilegio")),
        "{errores:?}"
    );
    let otra_lectura = texto.replacen(
        original,
        "      contents: read\n      pull-requests: read\n",
        1,
    );
    let errores = lint_github(&otra_lectura);
    assert!(
        errores
            .iter()
            .any(|e| e.contains("job buscar: lectura no admitida: pull-requests")),
        "{errores:?}"
    );

    let plantilla_texto = leer(PLANTILLA);
    let (_, cuerpo) = plantilla(&plantilla_texto);
    let base = cuerpo.get(".tc2-verificacion");
    let mut rutas = Vec::new();
    if let Some(p) = base
        .and_then(|b| b.get("artifacts"))
        .and_then(|a| a.get("paths"))
    {
        p.escalares("paths", &mut rutas);
    }
    assert!(
        rutas
            .iter()
            .any(|(_, r)| *r == ".tc2/cache-verificacion-*.txt"),
        "{rutas:?}"
    );
    let mut despues = Vec::new();
    if let Some(s) = base.and_then(|b| b.get("after_script")) {
        s.escalares("after_script", &mut despues);
    }
    assert!(
        despues
            .iter()
            .any(|(_, l)| l.contains("CI_JOB_STATUS")
                && l.contains("cache-verificacion-$TC2_PERFIL")),
        "{despues:?}"
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

// --- Revisor IA (F6-3) -----------------------------------------------------------

const REVISION_IA: &str = ".github/workflows/revision-ia.yml";

/// Los `with.name` de los pasos `actions/upload-artifact` de un job.
fn artefactos_subidos(raiz: &Nodo, job: &str) -> Vec<String> {
    let Some(Nodo::Lista(pasos)) = raiz
        .get("jobs")
        .and_then(|j| j.get(job))
        .and_then(|j| j.get("steps"))
    else {
        return Vec::new();
    };
    pasos
        .iter()
        .filter(|p| {
            p.get("uses")
                .and_then(Nodo::texto)
                .is_some_and(|u| u.starts_with("actions/upload-artifact@"))
        })
        .map(|p| {
            p.get("with")
                .and_then(|w| w.get("name"))
                .and_then(Nodo::texto)
                .unwrap_or_default()
                .to_owned()
        })
        .collect()
}

/// `validar` sube exactamente el artefacto `revision-ia-hallazgos` y
/// `publicar` lee ese mismo nombre; si no casan, el check sale siempre
/// `neutral` con `sin-hallazgos` y solo el E2E lo veria.
fn validar_sube_lo_que_lee_publicar(texto: &str) -> Result<(), String> {
    const NOMBRE: &str = "revision-ia-hallazgos";
    let raiz = leer_yaml(texto)?;
    let subidos = artefactos_subidos(&raiz, "validar");
    if subidos != [NOMBRE] {
        return Err(format!("validar sube {subidos:?}, no [\"{NOMBRE}\"]"));
    }
    let mut valores = Vec::new();
    if let Some(j) = raiz.get("jobs").and_then(|j| j.get("publicar")) {
        j.escalares("", &mut valores);
    }
    let lee = valores
        .iter()
        .any(|(k, v)| *k == "run" && v.contains(&format!("-f name={NOMBRE} ")));
    if !lee {
        return Err(format!("publicar no lee el artefacto {NOMBRE}"));
    }
    Ok(())
}

/// El workflow del revisor pasa el lint con el perfil revision-ia: cuatro
/// jobs (F6-4); `consultar` y `publicar` en el entorno `revisor`, sin
/// checkout, cada uno solo con su secreto; `preparar` y `validar` sin
/// secretos; `validar` corre `tc2 hallazgos validar` y `publicar` lee
/// `revision-ia-hallazgos`, no las respuestas del modelo en bruto.
#[test]
fn el_workflow_del_revisor_pasa_el_lint() {
    let texto = leer(REVISION_IA);
    let raiz = yaml(&texto);
    let jobs = raiz.get("jobs").map(Nodo::claves).unwrap_or_default();
    assert_eq!(jobs, ["preparar", "consultar", "validar", "publicar"]);
    assert_eq!(lint_revision(&texto), Vec::<String>::new());
    let job = |n: &str| raiz.get("jobs").and_then(|j| j.get(n));
    let valores = |n: &str| -> Vec<String> {
        let mut v = Vec::new();
        if let Some(j) = job(n) {
            j.escalares("", &mut v);
        }
        v.iter().map(|(_, x)| normalizar(x)).collect()
    };
    let usa = |n: &str, buscado: &str| valores(n).iter().any(|v| v.contains(buscado));
    let entorno = |n: &str| {
        job(n)
            .and_then(|j| j.get("environment"))
            .and_then(Nodo::texto)
    };
    assert_eq!(entorno("consultar"), Some("revisor"));
    assert_eq!(entorno("publicar"), Some("revisor"));
    assert_eq!(entorno("preparar"), None);
    assert_eq!(entorno("validar"), None);
    let secretos = [
        ("ANTHROPIC_API_KEY", "consultar"),
        ("TC2_APP_REVISOR_ID", "publicar"),
        ("TC2_APP_REVISOR_CLAVE", "publicar"),
    ];
    for (s, dueno) in secretos {
        let buscado = format!("secrets.{}", s.to_ascii_lowercase());
        for n in jobs.iter().copied() {
            assert_eq!(usa(n, &buscado), n == dueno, "{s} en {n}");
        }
    }
    for n in ["preparar", "validar"] {
        assert!(!usa(n, "secrets."), "secreto en {n}");
    }
    // validar valida con tc2 y sube el artefacto que lee publicar.
    let runs_de = |n: &str| -> Vec<String> {
        let mut v = Vec::new();
        if let Some(j) = job(n) {
            j.escalares("", &mut v);
        }
        v.iter()
            .filter(|(k, _)| *k == "run")
            .map(|(_, x)| (*x).to_owned())
            .collect()
    };
    assert!(
        runs_de("validar")
            .iter()
            .any(|r| r.contains("tc2 hallazgos validar"))
    );
    assert_eq!(
        artefactos_subidos(&raiz, "validar"),
        ["revision-ia-hallazgos"]
    );
    assert_eq!(validar_sube_lo_que_lee_publicar(&texto), Ok(()));
    assert!(
        runs_de("preparar")
            .iter()
            .any(|r| r.contains("tc2 hallazgos instrucciones"))
    );
    // publicar: solo el artefacto validado, nunca las respuestas en bruto.
    assert!(usa("publicar", "revision-ia-hallazgos"));
    assert!(!usa("publicar", "revision-ia-prompts"));
    assert!(!usa("publicar", "needs.consultar.outputs"));
    assert!(!usa("publicar", "api.anthropic.com"));
    // Lo que viene de otro job (las respuestas del modelo) entra por env,
    // nunca interpolado en run.
    for n in jobs.iter().copied() {
        for r in runs_de(n) {
            assert!(!normalizar(&r).contains("needs."), "needs. en run de {n}");
        }
    }
    // Con el perfil de verificacion, el entorno y el evento no valen.
    assert!(!lint_github(&texto).is_empty());
}

#[test]
fn cada_mutacion_del_revisor_da_su_error() {
    let texto = leer(REVISION_IA);
    let casos: [(&str, &str, &str); 11] = [
        (
            "  workflow_call:\n",
            "  pull_request:\n  workflow_call:\n",
            "evento no admitido: pull_request",
        ),
        (
            "          ref: ${{ inputs.base }}\n",
            "          ref: ${{ inputs.head-sha }}\n",
            "checkout del head del PR",
        ),
        (
            "          ref: ${{ inputs.base }}\n",
            "",
            "checkout sin ref fija",
        ),
        (
            "          git -C base fetch --no-tags -q origin \"$HEAD_SHA\"\n",
            "          git -C base fetch --no-tags -q origin \"$HEAD_SHA\"\n          git -C base checkout \"$HEAD_SHA\"\n",
            "git checkout en run",
        ),
        (
            "    environment: revisor\n",
            "",
            "job consultar: secreto en clase ejecuta",
        ),
        (
            "    timeout-minutes: 15\n    environment: revisor\n",
            "    timeout-minutes: 15\n",
            "job publicar: secreto en clase ejecuta",
        ),
        (
            "    environment: revisor\n",
            "    environment: produccion\n",
            "entorno no admitido",
        ),
        (
            "    permissions:\n      actions: read\n    steps:\n      - name: Publicar",
            "    permissions:\n      actions: read\n      checks: write\n    steps:\n      - name: Publicar",
            "job publicar: escritura con GITHUB_TOKEN",
        ),
        (
            "          CONSULTAR: ${{ needs.consultar.result }}\n          R0:",
            "          CONSULTAR: ${{ needs.consultar.result }}\n          CLAVE: ${{ secrets.ANTHROPIC_API_KEY }}\n          R0:",
            "job validar: secreto en clase ejecuta",
        ),
        (
            "          FORZADA: ${{ inputs.tc2-sha }}\n",
            "          FORZADA: ${{ secrets.ANTHROPIC_API_KEY }}\n",
            "job preparar: secreto en clase ejecuta",
        ),
        (
            "          set -eu\n          rustup toolchain install\n",
            "          set -eu\n          echo ${{ inputs.rama }}\n          rustup toolchain install\n",
            "inputs. en run",
        ),
    ];
    for (original, mutado, esperado) in casos {
        assert!(texto.contains(original), "el caso no casa: {original}");
        let errores = lint_revision(&texto.replacen(original, mutado, 1));
        assert!(
            errores.iter().any(|e| e.contains(esperado)),
            "«{esperado}» no salio; errores: {errores:?}"
        );
    }
    // Un checkout en el publicador: privilegio por el entorno.
    let con_checkout = texto.replacen(
        "    steps:\n      - name: Publicar",
        "    steps:\n      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1\n        with:\n          persist-credentials: false\n          ref: develop\n      - name: Publicar",
        1,
    );
    assert_ne!(con_checkout, texto);
    let errores = lint_revision(&con_checkout);
    assert!(
        errores
            .iter()
            .any(|e| e.contains("job publicar: checkout en clase privilegio")),
        "{errores:?}"
    );
    // consultar no sube artefactos: es de clase privilegio.
    let con_subida = texto.replacen(
        "          done < \"$dir/prompts/lista.tsv\"\n\n  validar:\n",
        "          done < \"$dir/prompts/lista.tsv\"\n      - uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02\n        with:\n          name: x\n          path: y\n\n  validar:\n",
        1,
    );
    assert_ne!(con_subida, texto);
    let errores = lint_revision(&con_subida);
    assert!(
        errores
            .iter()
            .any(|e| e.contains("job consultar: actions/upload-artifact en clase privilegio")),
        "{errores:?}"
    );
    // El artefacto que sube validar y el que lee publicar tienen que casar:
    // renombrar cualquiera de los dos da rojo.
    let artefactos: [(&str, &str, &str); 2] = [
        (
            "          name: revision-ia-hallazgos\n",
            "          name: otro-nombre\n",
            "validar sube [\"otro-nombre\"]",
        ),
        (
            "-f name=revision-ia-hallazgos ",
            "-f name=otro-nombre ",
            "publicar no lee el artefacto revision-ia-hallazgos",
        ),
    ];
    for (original, mutado, esperado) in artefactos {
        assert!(texto.contains(original), "el caso no casa: {original}");
        let r = validar_sube_lo_que_lee_publicar(&texto.replacen(original, mutado, 1));
        assert!(
            r.as_ref().is_err_and(|e| e.contains(esperado)),
            "«{esperado}» no salio: {r:?}"
        );
    }
}

/// El llamador de la guia de F6-3 pasa el lint de su perfil; `secrets:
/// inherit` hacia otro reutilizable, sin SHA, o con el perfil de
/// verificacion, no (adenda F6-3 de F1-8).
#[test]
fn llamador_de_revision_ia_hereda_solo_hacia_revision_ia() {
    let guia = include_str!("../../docs/ci/revision-ia.md").replace("\r\n", "\n");
    let Some(inicio) = guia.find("```yaml\nname: revision-ia\n").map(|i| i + 8) else {
        panic!("la guia no tiene el bloque del llamador");
    };
    let Some(largo) = guia[inicio..].find("```") else {
        panic!("el bloque del llamador no se cierra");
    };
    let fin = inicio + largo;
    let llamador = guia[inicio..fin].replace("@<sha>", "@0000000000000000000000000000000000000000");
    assert_eq!(lint_llamador_revision(&llamador), Vec::<String>::new());
    let malos = [
        llamador.replace(
            "charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml",
            "o/r/.github/workflows/revision-ia.yml",
        ),
        llamador.replace(
            "revision-ia.yml@0000000000000000000000000000000000000000",
            "revision-ia.yml@main",
        ),
    ];
    for malo in &malos {
        assert_ne!(malo, &llamador);
        let errores = lint_llamador_revision(malo);
        assert!(
            errores.iter().any(|e| e.contains("secrets: inherit")),
            "{errores:?}"
        );
    }
    assert!(
        lint_github(&llamador)
            .iter()
            .any(|e| e.contains("secrets: inherit"))
    );
    // El reutilizable (perfil revision-ia) sigue sin admitir la herencia,
    // tampoco hacia el destino admitido en el llamador.
    assert!(
        lint_revision(&llamador)
            .iter()
            .any(|e| e.contains("secrets: inherit"))
    );
    // El llamador no escribe con el GITHUB_TOKEN.
    let con_escritura = llamador.replace("      contents: read\n", "      contents: write\n");
    assert_ne!(con_escritura, llamador);
    assert!(
        lint_llamador_revision(&con_escritura)
            .iter()
            .any(|e| e.contains("escritura")),
        "{:?}",
        lint_llamador_revision(&con_escritura)
    );
}
