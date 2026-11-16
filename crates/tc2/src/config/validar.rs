//! Recorrido del arbol YAML contra la tabla de [`super::claves`]: junta
//! todos los errores con la ruta de cada clave y, si no hay ninguno, aplica
//! los valores por defecto de la tabla.

use serde_json::Value;

use super::ErrorConfig;
use super::claves::{CLAVES_016, Campo, Nodo, Regla};
use crate::i18n::catalogo::config as t;
use crate::i18n::{Mensaje, Texto};
use crate::plataforma::http::Base;

/// Errores de `valor` frente a `nodo`, con `ruta` como prefijo.
pub(crate) fn validar(nodo: &Nodo, valor: &Value, ruta: &str, errores: &mut Vec<ErrorConfig>) {
    let mut err = |mensaje: Mensaje| errores.push(ErrorConfig::nuevo(ruta, mensaje));
    match nodo {
        Nodo::Entero { min, max } => {
            let ok = valor
                .as_u64()
                .is_some_and(|n| n >= *min && max.is_none_or(|m| n <= m));
            if !ok {
                let m = match max {
                    Some(m) if m == min => {
                        Mensaje::nuevo(&t::SE_ESPERABA).con("esperado", min.to_string())
                    }
                    Some(m) => Mensaje::nuevo(&t::ENTERO_EN_RANGO)
                        .con("min", min.to_string())
                        .con("max", m.to_string()),
                    None => Mensaje::nuevo(&t::ENTERO_DESDE).con("min", min.to_string()),
                };
                err(con_valor(m, valor));
            }
        }
        Nodo::Booleano { solo } => match (solo, valor.as_bool()) {
            (Some(b), Some(x)) if *b == x => {}
            (Some(b), _) => err(Mensaje::nuevo(&t::SOLO_ADMITE).con("valor", b.to_string())),
            (None, Some(_)) => {}
            (None, None) => err(con_valor(Mensaje::nuevo(&t::SE_ESPERABA_BOOLEANO), valor)),
        },
        Nodo::Texto { valores, regla } => {
            let Some(s) = valor.as_str() else {
                err(con_valor(Mensaje::nuevo(&t::SE_ESPERABA_TEXTO), valor));
                return;
            };
            if !valores.is_empty() && !valores.contains(&s) {
                err(Mensaje::nuevo(&t::VALOR_NO_ADMITIDO)
                    .con("valor", s)
                    .con("admite", valores.join(", ")));
            } else if let Some(m) = comprobar(*regla, s) {
                err(m);
            }
        }
        Nodo::Lista { de, min, primero } => {
            let Some(l) = valor.as_array() else {
                err(con_valor(Mensaje::nuevo(&t::SE_ESPERABA_LISTA), valor));
                return;
            };
            if l.len() < *min {
                err(Mensaje::nuevo(&t::LISTA_CORTA).con("min", min.to_string()));
            }
            if let Some(p) = primero
                && l.first().and_then(Value::as_str) != Some(p)
            {
                err(Mensaje::nuevo(&t::PRIMER_ELEMENTO).con("valor", *p));
            }
            let textos: Vec<&str> = l.iter().filter_map(Value::as_str).collect();
            if let Some(d) = textos
                .iter()
                .enumerate()
                .find_map(|(i, x)| textos.get(..i).is_some_and(|a| a.contains(x)).then_some(x))
            {
                err(Mensaje::nuevo(&t::REPETIDO).con("valor", *d));
            }
            for (i, x) in l.iter().enumerate() {
                validar(de, x, &format!("{ruta}[{i}]"), errores);
            }
        }
        Nodo::Objeto(campos) => {
            let Some(m) = valor.as_object() else {
                err(con_valor(Mensaje::nuevo(&t::SE_ESPERABA_MAPA), valor));
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
                errores.push(ErrorConfig::nuevo(&unir(ruta, c.nombre), &t::FALTA_CLAVE));
            }
        }
        Nodo::UnaDe(formas) => {
            let encaja = formas.iter().any(|f| {
                let mut e = Vec::new();
                validar(f, valor, ruta, &mut e);
                e.is_empty()
            });
            if !encaja {
                // «a o b o c»: cada « o » es una entrada del catalogo.
                let resumen = formas.iter().map(resumir).reduce(|a, b| {
                    Mensaje::nuevo(&t::FORMA_O_FORMA)
                        .con_mensaje("a", a)
                        .con_mensaje("b", b)
                });
                let m = Mensaje::nuevo(&t::NO_ENCAJA);
                err(match resumen {
                    Some(r) => m.con_mensaje("formas", r),
                    None => m.con("formas", ""),
                });
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
                    &t::OBLIGATORIA_EN_GENERA,
                )),
                Some("comprueba") if escribe => errores.push(ErrorConfig::nuevo(
                    &format!("derivados[{i}].escribe"),
                    &t::PROHIBIDA_EN_COMPRUEBA,
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
                        Mensaje::nuevo(&t::REPETIDO).con("valor", id),
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

fn no_admitida(ruta: &str, clave: &str) -> &'static Texto {
    if ruta.is_empty() && CLAVES_016.contains(&clave) {
        &t::CLAVE_NO_ADMITIDA_016
    } else {
        &t::CLAVE_NO_ADMITIDA
    }
}

fn unir(ruta: &str, clave: &str) -> String {
    if ruta.is_empty() {
        clave.to_owned()
    } else {
        format!("{ruta}.{clave}")
    }
}

/// Rellena el hueco `{valor}` de `m` con el valor, corto y sin saltos de
/// linea: un texto tal cual entre «», un numero o booleano tal cual, y lo
/// demas («nada», «una lista», «un mapa») del catalogo.
fn con_valor(m: Mensaje, v: &Value) -> Mensaje {
    let s = match v {
        Value::Null => return m.con_mensaje("valor", &t::NADA),
        Value::Array(_) => return m.con_mensaje("valor", &t::UNA_LISTA),
        Value::Object(_) => return m.con_mensaje("valor", &t::UN_MAPA),
        Value::String(s) => format!("«{s}»"),
        otro => otro.to_string(),
    };
    let s: String = s.chars().filter(|c| !c.is_control()).take(60).collect();
    m.con("valor", s)
}

fn resumir(n: &Nodo) -> Mensaje {
    match n {
        Nodo::Objeto(c) => Mensaje::nuevo(&t::UN_MAPA_CON).con(
            "claves",
            c.iter()
                .map(|x: &Campo| x.nombre)
                .collect::<Vec<_>>()
                .join(", "),
        ),
        Nodo::Lista {
            primero: Some(p), ..
        } => Mensaje::nuevo(&t::LISTA_QUE_EMPIEZA).con("primero", *p),
        Nodo::Lista { .. } => Mensaje::nuevo(&t::UNA_LISTA),
        _ => Mensaje::nuevo(&t::UN_VALOR),
    }
}

/// Mensaje de error si `s` no cumple `regla`.
pub(crate) fn comprobar(regla: Regla, s: &str) -> Option<Mensaje> {
    let control = s.chars().any(char::is_control);
    match regla {
        Regla::Valor => None,
        Regla::Nombre => {
            (s.is_empty() || control || s.trim() != s).then(|| Mensaje::nuevo(&t::TEXTO_NO_NOMBRE))
        }
        Regla::Orden => {
            if s.trim().is_empty() {
                Some(Mensaje::nuevo(&t::ORDEN_VACIA))
            } else if s.contains(['\n', '\r']) {
                Some(Mensaje::nuevo(&t::ORDEN_CON_SALTOS))
            } else if control {
                Some(Mensaje::nuevo(&t::ORDEN_CON_CONTROL))
            } else {
                None
            }
        }
        Regla::Glob | Regla::GlobAcotado => {
            let motivo = if let Some(m) = glob_malo(s) {
                Some(m)
            } else if regla == Regla::GlobAcotado && todo_el_arbol(s) {
                Some(&t::NUNCA_TODO_EL_ARBOL)
            } else {
                None
            };
            motivo.map(|m| {
                Mensaje::nuevo(&t::GLOB_NO_ADMITIDO)
                    .con("glob", s)
                    .con_mensaje("motivo", m)
            })
        }
        Regla::RutaTareas => ruta_tareas(s).err().map(|m| {
            Mensaje::nuevo(&t::RUTA_NO_ADMITIDA)
                .con("ruta", s)
                .con_mensaje("motivo", m)
        }),
        Regla::Cron => {
            (!cron_valido(s)).then(|| Mensaje::nuevo(&t::CRON_NO_ADMITIDO).con("cron", s))
        }
        Regla::Url => Base::nueva(s)
            .err()
            .map(|m| Mensaje::nuevo(&t::URL_NO_ADMITIDA).con_mensaje("motivo", m)),
    }
}

/// Si un glob de `confianza_incluye` abarca todo el arbol: quitado `./`,
/// ningun segmento tiene mas que `*` y `.` (`**`, `*.*`, `**/`, `.`...).
fn todo_el_arbol(g: &str) -> bool {
    let g = g.trim_start_matches("./");
    let partes: Vec<&str> = g.split('/').filter(|p| !p.is_empty()).collect();
    // `*` y `.` son todo el arbol (F1-11); `*.*` o `.*` solo la raiz.
    partes
        .iter()
        .all(|p| p.chars().all(|c| c == '*' || c == '.'))
        && (partes.iter().any(|p| p.contains("**"))
            || matches!(partes.as_slice(), [] | ["*"] | ["."]))
}

/// Expresion cron de 5 campos con sus rangos (`*`, `a`, `a-b`, listas con
/// `,` y paso `/n` con n >= 1).
fn cron_valido(t: &str) -> bool {
    const RANGOS: [(u32, u32); 5] = [(0, 59), (0, 23), (1, 31), (1, 12), (0, 7)];
    let campos: Vec<&str> = t.split_whitespace().collect();
    campos.len() == RANGOS.len()
        && campos.iter().zip(RANGOS).all(|(c, (min, max))| {
            c.split(',').all(|item| {
                let (base, paso) = match item.split_once('/') {
                    Some((b, p)) => (b, Some(p)),
                    None => (item, None),
                };
                let en_rango = |x: &str| {
                    !x.is_empty()
                        && x.bytes().all(|b| b.is_ascii_digit())
                        && x.parse::<u32>().is_ok_and(|n| n >= min && n <= max)
                };
                let base_ok = base == "*"
                    || en_rango(base)
                    || base.split_once('-').is_some_and(|(a, b)| {
                        en_rango(a) && en_rango(b) && a.parse::<u32>().ok() <= b.parse::<u32>().ok()
                    });
                let paso_ok = paso.is_none_or(|p| {
                    !p.is_empty()
                        && p.bytes().all(|b| b.is_ascii_digit())
                        && p.parse::<u32>().is_ok_and(|n| n >= 1)
                });
                base_ok && paso_ok
            })
        })
}

fn glob_malo(g: &str) -> Option<&'static Texto> {
    if g.is_empty() {
        Some(&t::VACIO)
    } else if g.chars().any(char::is_control) {
        Some(&t::CARACTER_DE_CONTROL)
    } else if g.starts_with('!') {
        Some(&t::SIN_NEGACIONES)
    } else if g.starts_with(['/', '\\']) || g.as_bytes().get(1) == Some(&b':') {
        Some(&t::SIN_RUTAS_ABSOLUTAS)
    } else if g.split(['/', '\\']).any(|p| p == "..") {
        Some(&t::SIN_PUNTO_PUNTO)
    } else {
        None
    }
}

/// Normaliza una ruta de `derivados[].escribe` (F1-11, decision 1): sin
/// segmentos `.` ni vacios y con `..` resuelto; bajo `tareas/`.
pub(crate) fn ruta_tareas(r: &str) -> Result<String, &'static Texto> {
    if r.contains(['\\', ':']) {
        return Err(&t::SIN_BARRA_NI_DOS_PUNTOS);
    }
    if r.contains(['*', '?', '[', ']', '{', '}']) {
        return Err(&t::SIN_COMODINES);
    }
    if r.chars().any(char::is_control) {
        return Err(&t::CARACTER_DE_CONTROL);
    }
    if r.starts_with('/') {
        return Err(&t::SIN_RUTAS_ABSOLUTAS);
    }
    let mut partes: Vec<&str> = Vec::new();
    for p in r.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                if partes.pop().is_none() {
                    return Err(&t::SALE_DE_LA_RAIZ);
                }
            }
            p if p.ends_with(['.', ' ']) || nombre_corto(p) => {
                return Err(&t::SEGMENTO_WINDOWS);
            }
            p => partes.push(p),
        }
    }
    if partes
        .iter()
        .any(|p| crate::git::estado::reservado_en_windows(p))
    {
        return Err(&t::NOMBRE_RESERVADO_WINDOWS);
    }
    if partes.iter().any(|p| p.eq_ignore_ascii_case(".git")) {
        return Err(&t::NUNCA_GIT);
    }
    match partes.as_slice() {
        ["tareas", resto @ ..] if !resto.is_empty() => {
            let segunda = resto.first().copied().unwrap_or("");
            if resto.len() == 1 && segunda.eq_ignore_ascii_case("ROADMAP.md") {
                Err(&t::ROADMAP_RESERVADA)
            } else if es_carpeta_de_tarea(segunda) {
                Err(&t::CARPETAS_DE_TAREA_RESERVADAS)
            } else {
                Ok(partes.join("/"))
            }
        }
        ["tareas"] => Err(&t::NUNCA_TAREAS_ENTERA),
        _ => Err(&t::BAJO_TAREAS),
    }
}

/// Nombre corto 8.3 de Windows (`ROADMA~1.MD`): `~` seguido de una cifra.
fn nombre_corto(p: &str) -> bool {
    p.as_bytes()
        .windows(2)
        .any(|w| w.first() == Some(&b'~') && w.get(1).is_some_and(u8::is_ascii_digit))
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
            comprobar(Regla::Orden, "scripts\u{b}erificar.bat").is_some_and(|m| m
                .texto(crate::i18n::Idioma::Es)
                .contains("comillas simples"))
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
            let e = ruta_tareas(r).err().map_or("ok", |m| m.es);
            assert!(e.contains(m), "{r}: {e}");
        }
        assert_eq!(
            ruta_tareas("tareas/TASK-mapa.md"),
            Ok("tareas/TASK-mapa.md".into())
        );
    }
}
