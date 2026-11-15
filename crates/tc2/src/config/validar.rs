//! Recorrido del arbol YAML contra la tabla de [`super::claves`]: junta
//! todos los errores con la ruta de cada clave y, si no hay ninguno, aplica
//! los valores por defecto de la tabla.

use serde_json::Value;

use super::ErrorConfig;
use super::claves::{CLAVES_016, Campo, Nodo, Regla};
use crate::plataforma::http::Base;

/// Errores de `valor` frente a `nodo`, con `ruta` como prefijo.
pub(crate) fn validar(nodo: &Nodo, valor: &Value, ruta: &str, errores: &mut Vec<ErrorConfig>) {
    let mut err = |mensaje: String| errores.push(ErrorConfig::nuevo(ruta, mensaje));
    match nodo {
        Nodo::Entero { min, max } => {
            let ok = valor
                .as_u64()
                .is_some_and(|n| n >= *min && max.is_none_or(|m| n <= m));
            if !ok {
                err(match max {
                    Some(m) if m == min => format!("se esperaba {min}, no {}", corto(valor)),
                    Some(m) => format!("se esperaba un entero de {min} a {m}, no {}", corto(valor)),
                    None => format!("se esperaba un entero de {min} o mas, no {}", corto(valor)),
                });
            }
        }
        Nodo::Booleano { solo } => match (solo, valor.as_bool()) {
            (Some(b), Some(x)) if *b == x => {}
            (Some(b), _) => err(format!("solo admite {b}")),
            (None, Some(_)) => {}
            (None, None) => err(format!("se esperaba true o false, no {}", corto(valor))),
        },
        Nodo::Texto { valores, regla } => {
            let Some(t) = valor.as_str() else {
                err(format!("se esperaba un texto, no {}", corto(valor)));
                return;
            };
            if !valores.is_empty() && !valores.contains(&t) {
                err(format!(
                    "valor no admitido «{t}» (admite: {})",
                    valores.join(", ")
                ));
            } else if let Some(m) = comprobar(*regla, t) {
                err(m);
            }
        }
        Nodo::Lista { de, min, primero } => {
            let Some(l) = valor.as_array() else {
                err(format!("se esperaba una lista, no {}", corto(valor)));
                return;
            };
            if l.len() < *min {
                err(format!(
                    "se esperaba una lista con al menos {min} elemento(s)"
                ));
            }
            if let Some(p) = primero
                && l.first().and_then(Value::as_str) != Some(p)
            {
                err(format!("el primer elemento tiene que ser «{p}»"));
            }
            for (i, x) in l.iter().enumerate() {
                validar(de, x, &format!("{ruta}[{i}]"), errores);
            }
        }
        Nodo::Objeto(campos) => {
            let Some(m) = valor.as_object() else {
                err(format!("se esperaba un mapa, no {}", corto(valor)));
                return;
            };
            for (k, x) in m {
                let r = unir(ruta, k);
                match campos.iter().find(|c| c.nombre == k) {
                    Some(c) => validar(&c.nodo, x, &r, errores),
                    None => errores.push(ErrorConfig::nuevo(&r, no_admitida(ruta, k))),
                }
            }
            for c in campos
                .iter()
                .filter(|c| c.obligatorio && !m.contains_key(c.nombre))
            {
                errores.push(ErrorConfig::nuevo(
                    &unir(ruta, c.nombre),
                    "falta la clave obligatoria",
                ));
            }
        }
        Nodo::UnaDe(formas) => {
            let encaja = formas.iter().any(|f| {
                let mut e = Vec::new();
                validar(f, valor, ruta, &mut e);
                e.is_empty()
            });
            if !encaja {
                let resumen: Vec<String> = formas.iter().map(resumir).collect();
                err(format!(
                    "no encaja con ninguna forma admitida: {}",
                    resumen.join(" o ")
                ));
            }
        }
    }
}

/// Reglas que cruzan campos y que la tabla no expresa.
pub(crate) fn cruzadas(raiz: &Value, errores: &mut Vec<ErrorConfig>) {
    if let Some(ds) = raiz.get("derivados").and_then(Value::as_array) {
        for (i, d) in ds.iter().enumerate() {
            let escribe = d.get("escribe").is_some();
            match d.get("tipo").and_then(Value::as_str) {
                Some("genera") if !escribe => errores.push(ErrorConfig::nuevo(
                    &format!("derivados[{i}].escribe"),
                    "obligatoria en un derivado de tipo genera",
                )),
                Some("comprueba") if escribe => errores.push(ErrorConfig::nuevo(
                    &format!("derivados[{i}].escribe"),
                    "prohibida en un derivado de tipo comprueba",
                )),
                _ => {}
            }
        }
    }
    if let Some(rs) = raiz.get("recursos").and_then(Value::as_array) {
        let mut vistos = Vec::new();
        for (i, r) in rs.iter().enumerate() {
            if let Some(id) = r.get("id").and_then(Value::as_str) {
                if vistos.contains(&id) {
                    errores.push(ErrorConfig::nuevo(
                        &format!("recursos[{i}].id"),
                        format!("«{id}» repetido"),
                    ));
                }
                vistos.push(id);
            }
        }
    }
}

/// Rellena las claves que faltan con el valor por defecto de la tabla.
pub(crate) fn aplicar_defectos(nodo: &Nodo, valor: &mut Value) {
    match (nodo, valor) {
        (Nodo::Objeto(campos), Value::Object(m)) => {
            for c in campos {
                if !m.contains_key(c.nombre)
                    && let Some(d) = &c.defecto
                {
                    m.insert(c.nombre.to_owned(), d.clone());
                }
                if let Some(x) = m.get_mut(c.nombre) {
                    aplicar_defectos(&c.nodo, x);
                }
            }
        }
        (Nodo::Lista { de, .. }, Value::Array(l)) => {
            for x in l {
                aplicar_defectos(de, x);
            }
        }
        _ => {}
    }
}

fn no_admitida(ruta: &str, clave: &str) -> String {
    if ruta.is_empty() && CLAVES_016.contains(&clave) {
        "clave no admitida: es de la configuracion de la 0.16 (tc2 actualizar la traduce)"
            .to_owned()
    } else {
        "clave no admitida".to_owned()
    }
}

fn unir(ruta: &str, clave: &str) -> String {
    if ruta.is_empty() {
        clave.to_owned()
    } else {
        format!("{ruta}.{clave}")
    }
}

/// El valor en el mensaje, corto y sin saltos de linea.
fn corto(v: &Value) -> String {
    let s = match v {
        Value::Null => "nada".to_owned(),
        Value::String(t) => format!("«{t}»"),
        Value::Array(_) => "una lista".to_owned(),
        Value::Object(_) => "un mapa".to_owned(),
        otro => otro.to_string(),
    };
    let s: String = s.chars().filter(|c| !c.is_control()).take(60).collect();
    s
}

fn resumir(n: &Nodo) -> String {
    match n {
        Nodo::Objeto(c) => format!(
            "un mapa con {}",
            c.iter()
                .map(|x: &Campo| x.nombre)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Nodo::Lista {
            primero: Some(p), ..
        } => format!("una lista que empieza por {p}"),
        Nodo::Lista { .. } => "una lista".to_owned(),
        _ => "un valor".to_owned(),
    }
}

/// Mensaje de error si `t` no cumple `regla`.
pub(crate) fn comprobar(regla: Regla, t: &str) -> Option<String> {
    let control = t.chars().any(char::is_control);
    match regla {
        Regla::Valor => None,
        Regla::Nombre => (t.is_empty() || control || t.trim() != t).then(|| {
            "texto vacio, con espacios en los extremos o con caracteres de control".to_owned()
        }),
        Regla::Orden => {
            if t.trim().is_empty() {
                Some("orden vacia".to_owned())
            } else if control {
                Some("la orden tiene un caracter de control (¿un \\v de una ruta de Windows entre comillas dobles? escribela entre comillas simples o con /)".to_owned())
            } else {
                None
            }
        }
        Regla::Glob | Regla::GlobAcotado => {
            if let Some(m) = glob_malo(t) {
                Some(format!("glob no admitido «{t}»: {m}"))
            } else if regla == Regla::GlobAcotado
                && matches!(t, "*" | "**" | "**/*" | "**/**" | "./**" | "./*")
            {
                Some(format!("glob no admitido «{t}»: nunca todo el arbol"))
            } else {
                None
            }
        }
        Regla::RutaTareas => ruta_tareas(t)
            .err()
            .map(|m| format!("ruta no admitida «{t}»: {m}")),
        Regla::Cron => {
            let campos: Vec<&str> = t.split_whitespace().collect();
            let ok = campos.len() == 5
                && campos.iter().all(|c| {
                    c.bytes()
                        .all(|b| b.is_ascii_digit() || b"*,/-".contains(&b))
                });
            (!ok).then(|| {
                format!("cron no admitido «{t}»: se esperan 5 campos con cifras, *, ',', / o -")
            })
        }
        Regla::Url => Base::nueva(t)
            .err()
            .map(|m| format!("URL no admitida: {m}")),
    }
}

fn glob_malo(g: &str) -> Option<&'static str> {
    if g.is_empty() {
        Some("vacio")
    } else if g.chars().any(char::is_control) {
        Some("caracter de control")
    } else if g.starts_with('!') {
        Some("sin negaciones")
    } else if g.starts_with(['/', '\\']) || g.as_bytes().get(1) == Some(&b':') {
        Some("sin rutas absolutas")
    } else if g.split(['/', '\\']).any(|p| p == "..") {
        Some("sin ..")
    } else {
        None
    }
}

/// Normaliza una ruta de `derivados[].escribe` (F1-11, decision 1): sin
/// segmentos `.` ni vacios y con `..` resuelto; bajo `tareas/`.
pub(crate) fn ruta_tareas(t: &str) -> Result<String, &'static str> {
    if t.contains(['\\', ':']) {
        return Err("sin \\ ni :");
    }
    if t.contains(['*', '?', '[', ']', '{', '}']) {
        return Err("sin comodines");
    }
    if t.chars().any(char::is_control) {
        return Err("caracter de control");
    }
    if t.starts_with('/') {
        return Err("sin rutas absolutas");
    }
    let mut partes: Vec<&str> = Vec::new();
    for p in t.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                if partes.pop().is_none() {
                    return Err("sale de la raiz");
                }
            }
            p => partes.push(p),
        }
    }
    if partes
        .iter()
        .any(|p| crate::git::estado::reservado_en_windows(p))
    {
        return Err("nombre reservado de Windows");
    }
    if partes.iter().any(|p| p.eq_ignore_ascii_case(".git")) {
        return Err("nunca .git");
    }
    match partes.as_slice() {
        ["tareas", resto @ ..] if !resto.is_empty() => {
            let segunda = resto.first().copied().unwrap_or("");
            if resto.len() == 1 && segunda.eq_ignore_ascii_case("ROADMAP.md") {
                Err("tareas/ROADMAP.md es reservada")
            } else if es_carpeta_de_tarea(segunda) {
                Err("las carpetas tareas/TASK-NNN/ son reservadas")
            } else {
                Ok(partes.join("/"))
            }
        }
        ["tareas"] => Err("nunca tareas/ entera"),
        _ => Err("tiene que ir bajo tareas/"),
    }
}

fn es_carpeta_de_tarea(p: &str) -> bool {
    p.get(..5).is_some_and(|x| x.eq_ignore_ascii_case("TASK-"))
        && p.get(5..)
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reglas_de_texto() {
        assert!(
            comprobar(Regla::Orden, "scripts\u{b}erificar.bat")
                .is_some_and(|m| m.contains("comillas simples"))
        );
        assert!(comprobar(Regla::Orden, "  ").is_some());
        assert!(comprobar(Regla::Orden, "cargo test").is_none());
        for g in ["!docs/**", "/abs", "C:/x", "a/../b", "..", ""] {
            assert!(comprobar(Regla::Glob, g).is_some(), "{g}");
        }
        assert!(comprobar(Regla::Glob, "src/auth/**").is_none());
        assert!(comprobar(Regla::Glob, "a..b/*").is_none());
        assert!(comprobar(Regla::GlobAcotado, "**").is_some());
        assert!(comprobar(Regla::GlobAcotado, "*").is_some());
        assert!(comprobar(Regla::GlobAcotado, "scripts/*.py").is_none());
        assert!(comprobar(Regla::Cron, "0 3 * * 1-5").is_none());
        assert!(comprobar(Regla::Cron, "0 3 * *").is_some());
        assert!(comprobar(Regla::Cron, "@daily x y z w").is_some());
        assert!(comprobar(Regla::Url, "https://git.empresa.com/api/v4").is_none());
        assert!(comprobar(Regla::Url, "http://git.empresa.com").is_some());
        assert!(comprobar(Regla::Nombre, " x").is_some());
        assert!(comprobar(Regla::Nombre, "security-review").is_none());
    }

    #[test]
    fn rutas_de_escribe() {
        assert_eq!(ruta_tareas("tareas/hitos.md"), Ok("tareas/hitos.md".into()));
        assert_eq!(
            ruta_tareas("./tareas//x/../hitos.md"),
            Ok("tareas/hitos.md".into())
        );
        for (r, m) in [
            ("tareas", "entera"),
            ("tareas/", "entera"),
            ("docs/x.md", "bajo tareas/"),
            ("tareas/../../x", "raiz"),
            ("tareas/../docs/x", "bajo tareas/"),
            ("../x", "raiz"),
            ("tareas/roadmap.MD", "ROADMAP"),
            ("tareas/task-012/tarea.md", "TASK-NNN"),
            ("tareas/.GIT/x", ".git"),
            ("tareas/aux.md", "Windows"),
            ("tareas\\x", "\\"),
            ("tareas/*.md", "comodines"),
            ("/tareas/x", "absolutas"),
        ] {
            let e = ruta_tareas(r).err().unwrap_or("ok");
            assert!(e.contains(m), "{r}: {e}");
        }
        assert_eq!(
            ruta_tareas("tareas/TASK-mapa.md"),
            Ok("tareas/TASK-mapa.md".into())
        );
    }
}
