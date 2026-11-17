//! `tc2 doctor` (F3-8): que le falta al entorno, a la rama de estado, a la
//! instalacion y a la plataforma para que TaskCodeTwo funcione, con el
//! arreglo de cada fallo (RF-64).
//!
//! Solo lee: ni `fetch` (`board`, desde F3-5b, si trae) ni escrituras en el repo, en
//! `<git-common-dir>/taskcodetwo/` ni en la plataforma (solo `GET`). Cada
//! comprobacion deja un [`Hallazgo`] (`ok`, `aviso`, `falla` o
//! `no-comprobado`); el agregador ([`informe`]) es puro.
//!
//! Salida (F1-4 decision 4, F1-13 seccion C): como mucho 5 lineas en
//! stdout. La primera, `doctor <ok|rechazada> fallos=N avisos=N
//! no-comprobado=N plataforma=... [plan=... impuestas=N sustituidas=N
//! no-disponibles=N [no-comprobadas=N]] [commit=...] [sincronizado=...]`
//! (la hora de la marca de F3-5b, sin `fetch`); detras, los
//! hallazgos que no son `ok`,
//! los peores primero, y si no caben, `y N mas: tc2 doctor --json`. Con
//! `--json`, una linea con todos los hallazgos (tambien los `ok`). Codigo 3
//! con alguna `falla` (D5a); 0 con avisos o no-comprobados.
//!
//! `doctor --recursos` (F3-9, [`recursos`]): el inventario de skills y
//! agentes y el manifiesto de recursos, con la primera linea `doctor
//! <ok|rechazada> fallos=N avisos=N no-comprobado=N skills=N agentes=N
//! inventario=<12hex>` y `y N mas: tc2 doctor --recursos --json`; con
//! `--json`, ademas de `hallazgos`, la lista `inventario`. El `doctor`
//! normal lleva el hallazgo agregado `recursos no-instalados`.

pub mod capacidades;
mod entorno;
mod estado;
pub mod plataforma;
pub mod recursos;

use std::path::Path;

use crate::Salida;
use crate::contrato::{Informe, Resultado, cadena_json, escapar_valor};
use crate::git::Git;
use crate::i18n::catalogo::doctor as txt;
use crate::i18n::{Idioma, Mensaje};
use crate::identidad::Conectar;

pub use entorno::{hallazgo_git, version_de_git};

/// Nombre de la orden.
pub const NOMBRE: &str = "doctor";

/// Lineas como mucho en stdout (F1-4, decision 4).
pub const MAX_LINEAS: usize = 5;

/// Arreglo de lo que deja `tc2 init`.
pub const ARREGLO_INIT: &str = "tc2-init";

/// Estado de una comprobacion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Estado {
    /// Falla: el codigo es 3.
    Falla,
    /// Funciona, pero algo esta degradado o falta.
    Aviso,
    /// No se pudo comprobar (sin red, sin token...): nunca se da por buena.
    NoComprobado,
    /// Bien.
    Ok,
}

impl Estado {
    /// Valor en la linea.
    pub fn texto(self) -> &'static str {
        match self {
            Self::Falla => "falla",
            Self::Aviso => "aviso",
            Self::NoComprobado => "no-comprobado",
            Self::Ok => "ok",
        }
    }
}

/// Lo que se encontro en una comprobacion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hallazgo {
    /// Zona: `entorno`, `estado`, `instalacion`, `proteccion`, `permisos`,
    /// `check`, `garantia`...
    pub area: &'static str,
    /// Que se comprobo dentro de la zona (`git`, `develop`, `G03`...).
    pub id: String,
    /// Como salio.
    pub estado: Estado,
    /// Por que (sin espacios).
    pub motivo: Option<String>,
    /// Claves de mas (`permiso=`, `ids=`...), en orden.
    pub extra: Vec<(&'static str, String)>,
    /// Que hacer (sin espacios).
    pub arreglo: Option<String>,
}

impl Hallazgo {
    /// Hallazgo sin motivo ni arreglo.
    pub fn nuevo(area: &'static str, id: impl Into<String>, estado: Estado) -> Self {
        Self {
            area,
            id: id.into(),
            estado,
            motivo: None,
            extra: Vec::new(),
            arreglo: None,
        }
    }

    /// Con motivo.
    #[must_use]
    pub fn motivo(mut self, m: impl Into<String>) -> Self {
        self.motivo = Some(m.into());
        self
    }

    /// Con una clave de mas.
    #[must_use]
    pub fn con(mut self, k: &'static str, v: impl Into<String>) -> Self {
        self.extra.push((k, v.into()));
        self
    }

    /// Con arreglo.
    #[must_use]
    pub fn arreglo(mut self, a: impl Into<String>) -> Self {
        self.arreglo = Some(a.into());
        self
    }

    /// Valor de una clave de mas.
    pub fn valor(&self, k: &str) -> Option<&str> {
        self.extra
            .iter()
            .find(|(c, _)| *c == k)
            .map(|(_, v)| v.as_str())
    }

    /// `<area> <id> <estado> [motivo=...] [claves...] [arreglo=...]`, con los
    /// valores escapados como el contrato (sin espacios).
    pub fn linea(&self) -> String {
        let mut s = format!(
            "{} {} {}",
            self.area,
            escapar_valor(&self.id, false),
            self.estado.texto()
        );
        let claves = self
            .motivo
            .iter()
            .map(|m| ("motivo", m.as_str()))
            .chain(self.extra.iter().map(|(k, v)| (*k, v.as_str())))
            .chain(self.arreglo.iter().map(|a| ("arreglo", a.as_str())));
        for (k, v) in claves {
            s.push_str(&format!(" {k}={}", escapar_valor(v, false)));
        }
        s
    }

    /// El hallazgo como objeto JSON.
    pub fn json(&self) -> String {
        let mut s = format!(
            "{{\"area\":{},\"id\":{},\"estado\":{}",
            cadena_json(self.area),
            cadena_json(&self.id),
            cadena_json(self.estado.texto())
        );
        if let Some(m) = &self.motivo {
            s.push_str(&format!(",\"motivo\":{}", cadena_json(m)));
        }
        for (k, v) in &self.extra {
            s.push_str(&format!(",{}:{}", cadena_json(k), cadena_json(v)));
        }
        if let Some(a) = &self.arreglo {
            s.push_str(&format!(",\"arreglo\":{}", cadena_json(a)));
        }
        s.push('}');
        s
    }
}

/// Lo que va en la primera linea ademas de las cuentas.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resumen {
    /// Valor de `plataforma=`: `ok`, `no-configurada`, `sin-credenciales`,
    /// `sin-red` o `error`.
    pub plataforma: String,
    /// Plan observado y cuentas de `capacidades` (F1-13 decision 3): `None`
    /// si no se llego a la plataforma.
    pub capacidades: Option<capacidades::Cuentas>,
    /// Commit de la rama de estado (12), si se leyo.
    pub commit: Option<String>,
    /// Hora del ultimo `fetch` con exito de la rama de estado o `nunca`
    /// (F3-5b, D4a): de la marca, sin `fetch`.
    pub sincronizado: Option<String>,
}

/// Cuantos hallazgos hay en `e`.
fn cuantos(hs: &[Hallazgo], e: Estado) -> usize {
    hs.iter().filter(|h| h.estado == e).count()
}

/// Donde se ve el detalle de `doctor`.
const DETALLE: &str = "tc2 doctor --json";

/// Donde se ve el detalle de `doctor --recursos`.
const DETALLE_RECURSOS: &str = "tc2 doctor --recursos --json";

/// El principio de la linea de contrato: `rechazada` con alguna `falla`
/// (D5a), `ok` si no, y las tres cuentas.
fn cuentas(hs: &[Hallazgo]) -> Informe {
    let fallos = cuantos(hs, Estado::Falla);
    let resultado = if fallos > 0 {
        Resultado::Rechazada
    } else {
        Resultado::Ok
    };
    Informe::nuevo(NOMBRE, resultado)
        .con("fallos", fallos.to_string())
        .con("avisos", cuantos(hs, Estado::Aviso).to_string())
        .con(
            "no-comprobado",
            cuantos(hs, Estado::NoComprobado).to_string(),
        )
}

/// El diagnostico que nombra el primer fallo, si lo hay.
fn con_diagnostico(i: Informe, hs: &[Hallazgo], detalle: &str) -> Informe {
    match ordenados(hs)
        .into_iter()
        .find(|h| h.estado == Estado::Falla)
    {
        Some(primero) => i.diagnostico(
            Mensaje::nuevo(&txt::CON_FALLOS)
                .con("n", cuantos(hs, Estado::Falla).to_string())
                .con("primero", primero.linea())
                .con("detalle", detalle),
        ),
        None => i,
    }
}

/// La linea de contrato de `doctor --recursos` (funcion pura): las cuentas,
/// `skills=`, `agentes=` e `inventario=` (la huella), sin `plataforma=`.
pub fn informe_recursos(hs: &[Hallazgo], inv: &crate::inventario::Inventario) -> Informe {
    use crate::inventario::Tipo;
    let i = cuentas(hs)
        .con("skills", inv.cuantas(Tipo::Skill).to_string())
        .con("agentes", inv.cuantas(Tipo::Agente).to_string())
        .con("inventario", inv.huella());
    con_diagnostico(i, hs, DETALLE_RECURSOS)
}

/// La linea de contrato de `doctor` (funcion pura): `rechazada` con alguna
/// `falla` (D5a), `ok` si no. El diagnostico nombra el primer fallo.
pub fn informe(hs: &[Hallazgo], r: &Resumen) -> Informe {
    let mut i = cuentas(hs).con("plataforma", r.plataforma.clone());
    if let Some(c) = &r.capacidades {
        i = i.con("plan", c.plan.clone());
        if let Some((imp, sus, nod)) = c.cuentas {
            i = i
                .con("impuestas", imp.to_string())
                .con("sustituidas", sus.to_string())
                .con("no-disponibles", nod.to_string());
            if c.no_comprobadas > 0 {
                i = i.con("no-comprobadas", c.no_comprobadas.to_string());
            }
        }
    }
    if let Some(c) = &r.commit {
        i = i.con("commit", c.clone());
    }
    if let Some(s) = &r.sincronizado {
        i = i.con("sincronizado", s.clone());
    }
    con_diagnostico(i, hs, DETALLE)
}

/// Los hallazgos que no son `ok`, los peores primero (estable: dentro de
/// cada estado, en el orden en que se comprobaron).
pub fn ordenados(hs: &[Hallazgo]) -> Vec<&Hallazgo> {
    let mut v: Vec<&Hallazgo> = hs.iter().filter(|h| h.estado != Estado::Ok).collect();
    v.sort_by_key(|h| h.estado);
    v
}

/// Las lineas de stdout sin `--json` (funcion pura): la primera y, detras,
/// los hallazgos que caben en [`MAX_LINEAS`]; si no caben todos, los
/// primeros y `y N mas: tc2 doctor --json`.
pub fn lineas(hs: &[Hallazgo], r: &Resumen) -> Vec<String> {
    lineas_de(informe(hs, r).linea(), hs, DETALLE)
}

/// Las lineas de stdout de `doctor --recursos` sin `--json` (funcion pura),
/// con `y N mas: tc2 doctor --recursos --json` si no caben.
pub fn lineas_recursos(hs: &[Hallazgo], inv: &crate::inventario::Inventario) -> Vec<String> {
    lineas_de(informe_recursos(hs, inv).linea(), hs, DETALLE_RECURSOS)
}

fn lineas_de(primera: String, hs: &[Hallazgo], detalle: &str) -> Vec<String> {
    let mut v = vec![primera];
    let malos = ordenados(hs);
    let hueco = MAX_LINEAS - 1;
    if malos.len() <= hueco {
        v.extend(malos.iter().map(|h| h.linea()));
    } else {
        v.extend(malos.iter().take(hueco - 1).map(|h| h.linea()));
        v.push(format!("y {} mas: {detalle}", malos.len() - (hueco - 1)));
    }
    v
}

/// La salida de un informe: texto (`lineas`) o JSON (una linea con todos
/// los hallazgos y `extra`, ya con su clave), diagnostico en stderr solo con
/// alguna `falla`.
fn salida_de(
    i: &Informe,
    hs: &[Hallazgo],
    json: bool,
    idioma: Idioma,
    lineas: Vec<String>,
    extra: &str,
) -> Salida {
    let mut s = i.salida_en(json, idioma);
    if json {
        let todos: Vec<String> = hs.iter().map(Hallazgo::json).collect();
        let linea = s.stdout.trim_end();
        if let Some(sin_cierre) = linea.strip_suffix('}') {
            s.stdout = format!(
                "{sin_cierre},\"hallazgos\":[{}]{extra}}}\n",
                todos.join(",")
            );
        }
    } else {
        s.stdout = lineas.join("\n") + "\n";
    }
    s
}

/// La salida del proceso: texto o JSON (una linea con todos los
/// hallazgos), diagnostico en stderr solo con alguna `falla`.
pub fn salida(hs: &[Hallazgo], r: &Resumen, json: bool, idioma: Idioma) -> Salida {
    salida_de(&informe(hs, r), hs, json, idioma, lineas(hs, r), "")
}

/// La salida de `doctor --recursos`: como [`salida`], y en JSON ademas la
/// lista `inventario` (D4a: la lista completa solo en JSON).
pub fn salida_recursos(
    hs: &[Hallazgo],
    inv: &crate::inventario::Inventario,
    json: bool,
    idioma: Idioma,
) -> Salida {
    salida_de(
        &informe_recursos(hs, inv),
        hs,
        json,
        idioma,
        lineas_recursos(hs, inv),
        &format!(",\"inventario\":{}", inv.json()),
    )
}

/// Lo que necesita `doctor` del entorno.
pub struct Entorno<'a> {
    /// Carpeta desde la que se lanza.
    pub dir: &'a Path,
    /// Puerto Git.
    pub git: &'a Git,
    /// Construye el adaptador de la plataforma.
    pub conectar: &'a Conectar<'a>,
    /// Carpeta de Claude Code (`CLAUDE_CONFIG_DIR` o `~/.claude`) para el
    /// inventario. `None`: no se sabe; el `doctor` normal no agrega los
    /// recursos y `--recursos` lo dice como `no-comprobado`.
    pub claude: Option<&'a Path>,
}

/// Todas las comprobaciones. `Err` solo si no se puede ni empezar (fuera de
/// un repo): `rechazada` con el diagnostico.
pub fn comprobar(e: &Entorno<'_>) -> Result<(Vec<Hallazgo>, Resumen), Informe> {
    let mut hs = vec![entorno::tc2()];
    let git = entorno::git(e.git, e.dir);
    let hay_git = !matches!(git.motivo.as_deref(), Some(entorno::SIN_GIT));
    hs.push(git);
    if !hay_git {
        // Sin git no se puede comprobar nada mas: se dice, sin suponer.
        for (area, id) in [
            ("entorno", "config"),
            ("estado", "rama"),
            ("instalacion", "plan"),
            ("plataforma", "conexion"),
        ] {
            hs.push(Hallazgo::nuevo(area, id, Estado::NoComprobado).motivo(entorno::SIN_GIT));
        }
        return Ok((
            hs,
            Resumen {
                plataforma: "no-configurada".to_owned(),
                ..Resumen::default()
            },
        ));
    }
    entorno::repo(e.git, e.dir)?;
    hs.push(entorno::config(e.git, e.dir));
    let (h, commit) = estado::rama(e.git, e.dir);
    hs.extend(h);
    hs.extend(estado::instalacion(e.git, e.dir));
    let (h, plataforma, capacidades) = plataforma::comprobar(e);
    hs.extend(h);
    if e.claude.is_some() {
        hs.push(recursos::agregado(e));
    }
    // La hora de la marca, sin `fetch` (D4a de F3-5b: doctor solo lee).
    let sincronizado =
        crate::estado::sincronizado_texto(crate::estado::sincronizado_en(e.git, e.dir));
    Ok((
        hs,
        Resumen {
            plataforma,
            capacidades,
            commit,
            sincronizado: Some(sincronizado),
        },
    ))
}

/// Lee las opciones (sin `--json`): nada o `--recursos` (devuelve si esta).
fn leer_opciones(args: &[String]) -> Result<bool, Informe> {
    match args {
        [] => Ok(false),
        [a] if a == "--recursos" => Ok(true),
        _ => Err(Informe::nuevo(NOMBRE, Resultado::Uso)
            .con("motivo", "argumentos")
            .diagnostico(&txt::USO_DOCTOR)),
    }
}

/// `doctor [--recursos] [--json]` (sin `--json`, que quita `lib.rs`).
pub fn orden(args: &[String], json: bool, idioma: Idioma, e: &Entorno<'_>) -> Salida {
    let con_recursos = match leer_opciones(args) {
        Ok(r) => r,
        Err(i) => return i.salida_en(json, idioma),
    };
    if con_recursos {
        return match recursos::comprobar(e) {
            Ok((hs, inv)) => salida_recursos(&hs, &inv, json, idioma),
            Err(i) => i.salida_en(json, idioma),
        };
    }
    match comprobar(e) {
        Ok((hs, r)) => salida(&hs, &r, json, idioma),
        Err(i) => i.salida_en(json, idioma),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(area: &'static str, id: &str, e: Estado) -> Hallazgo {
        Hallazgo::nuevo(area, id, e).motivo("m")
    }

    fn resumen() -> Resumen {
        Resumen {
            plataforma: "ok".to_owned(),
            capacidades: None,
            commit: Some("0123456789ab".to_owned()),
            sincronizado: None,
        }
    }

    /// C8 (parte pura): `sincronizado=` va detras de `commit=` cuando se
    /// sabe.
    #[test]
    fn sincronizado_tras_el_commit() {
        let r = Resumen {
            sincronizado: Some("2026-11-17T16:04:12Z".to_owned()),
            ..resumen()
        };
        assert_eq!(
            informe(&[], &r).linea(),
            "doctor ok fallos=0 avisos=0 no-comprobado=0 plataforma=ok commit=0123456789ab sincronizado=2026-11-17T16:04:12Z"
        );
    }

    /// C7: la primera linea fija; detras, los peores primero; con mas de 4,
    /// 3 y `y N mas`; nunca mas de 5 lineas; JSON con todos y sin ANSI.
    #[test]
    fn cinco_lineas_como_mucho_y_los_peores_primero() {
        let hs = vec![
            h("entorno", "tc2", Estado::Ok),
            h("instalacion", "workflow", Estado::Aviso),
            h("plataforma", "conexion", Estado::NoComprobado),
            h("proteccion", "develop", Estado::Falla),
            h("check", "carril", Estado::Aviso),
            h("check", "verificacion", Estado::Falla),
        ];
        let l = lineas(&hs, &resumen());
        assert_eq!(
            l,
            [
                "doctor rechazada fallos=2 avisos=2 no-comprobado=1 plataforma=ok commit=0123456789ab",
                "proteccion develop falla motivo=m",
                "check verificacion falla motivo=m",
                "instalacion workflow aviso motivo=m",
                "y 2 mas: tc2 doctor --json",
            ]
        );
        // Con 4 que no son ok, caben todos.
        let l = lineas(&hs[..5], &resumen());
        assert_eq!(l.len(), 5);
        assert_eq!(l[4], "plataforma conexion no-comprobado motivo=m");
        // Sin nada malo, solo la primera.
        assert_eq!(lineas(&hs[..1], &resumen()).len(), 1);
        let s = salida(&hs, &resumen(), false, Idioma::Es);
        assert_eq!(s.codigo, 3);
        assert_eq!(s.stdout.lines().count(), MAX_LINEAS);
        assert_eq!(s.stderr.lines().count(), 1);
        assert!(
            s.stderr.contains("proteccion develop falla"),
            "{}",
            s.stderr
        );
        let j = salida(&hs, &resumen(), true, Idioma::En);
        assert_eq!(j.codigo, 3);
        assert_eq!(j.stdout.lines().count(), 1);
        assert!(!j.stdout.contains('\u{1b}'));
        let v: serde_json::Value = serde_json::from_str(&j.stdout).unwrap_or_default();
        assert_eq!(v["hallazgos"].as_array().map(Vec::len), Some(6));
        assert_eq!(v["claves"]["fallos"], "2");
        assert_eq!(v["hallazgos"][0]["estado"], "ok");
    }

    /// C8: 0 sin fallos aunque haya avisos y no-comprobados; 3 con uno.
    #[test]
    fn codigo_cero_sin_fallos_y_tres_con_uno() {
        let mut hs = vec![
            h("instalacion", "workflow", Estado::Aviso),
            h("plataforma", "conexion", Estado::NoComprobado),
        ];
        let s = salida(&hs, &resumen(), false, Idioma::Es);
        assert_eq!((s.codigo, s.stderr.as_str()), (0, ""));
        assert!(
            s.stdout
                .starts_with("doctor ok fallos=0 avisos=1 no-comprobado=1")
        );
        hs.push(h("estado", "rama", Estado::Falla));
        assert_eq!(salida(&hs, &resumen(), true, Idioma::Es).codigo, 3);
    }

    #[test]
    fn la_linea_escapa_los_valores() {
        let x = Hallazgo::nuevo("instalacion", "rama", Estado::Aviso)
            .motivo("sin-publicar")
            .con("ruta", "refs/heads/main")
            .arreglo("git push origin main");
        assert_eq!(
            x.linea(),
            "instalacion rama aviso motivo=sin-publicar ruta=refs/heads/main arreglo=git%20push%20origin%20main"
        );
    }

    /// C11: `--recursos` solo; cualquier otra cosa, `doctor uso
    /// motivo=argumentos`.
    #[test]
    fn argumentos_de_mas_son_uso() {
        assert_eq!(leer_opciones(&[]).ok(), Some(false));
        assert_eq!(leer_opciones(&["--recursos".to_owned()]).ok(), Some(true));
        for a in [
            vec!["x"],
            vec!["--recursos", "x"],
            vec!["--recursos", "--recursos"],
            vec!["x", "--recursos"],
        ] {
            let args: Vec<String> = a.iter().map(|s| (*s).to_owned()).collect();
            let i = leer_opciones(&args).err();
            assert_eq!(
                i.map(|i| i.linea()).as_deref(),
                Some("doctor uso motivo=argumentos"),
                "{a:?}"
            );
        }
    }

    /// C4: la primera linea de `--recursos` lleva `skills=`, `agentes=` e
    /// `inventario=` (12 hex) sin `plataforma=`; como mucho 5 lineas, con `y
    /// N mas: tc2 doctor --recursos --json`. C5: el JSON en una linea con
    /// `hallazgos` (tambien los ok) e `inventario`, sin ANSI.
    #[test]
    fn salida_de_recursos() {
        use crate::inventario::{Entrada, Inventario, Origen, Tipo};
        let inv = Inventario::nuevo(
            vec![
                Entrada {
                    tipo: Tipo::Agente,
                    id: "general-purpose".to_owned(),
                    origen: Origen::Integrado,
                },
                Entrada {
                    tipo: Tipo::Skill,
                    id: "alfa".to_owned(),
                    origen: Origen::Usuario,
                },
            ],
            Vec::new(),
        );
        let hs = vec![
            h("recursos", "taskcodetwo", Estado::Falla),
            h("recursos", "a", Estado::Aviso),
            h("recursos", "b", Estado::Aviso),
            h("recursos", "c", Estado::NoComprobado),
            h("recursos", "d", Estado::Aviso),
            h("recursos", "alfa", Estado::Ok),
        ];
        let l = lineas_recursos(&hs, &inv);
        assert_eq!(l.len(), MAX_LINEAS);
        assert_eq!(
            l[0],
            format!(
                "doctor rechazada fallos=1 avisos=3 no-comprobado=1 skills=1 agentes=1 inventario={}",
                inv.huella()
            )
        );
        assert!(!l[0].contains("plataforma="));
        assert_eq!(l[1], "recursos taskcodetwo falla motivo=m");
        assert_eq!(l[4], "y 2 mas: tc2 doctor --recursos --json");
        let s = salida_recursos(&hs, &inv, false, Idioma::Es);
        assert_eq!(s.codigo, 3);
        assert_eq!(s.stderr.lines().count(), 1);
        assert!(
            s.stderr.contains("tc2 doctor --recursos --json"),
            "{}",
            s.stderr
        );
        let j = salida_recursos(&hs, &inv, true, Idioma::Es);
        assert_eq!(j.stdout.lines().count(), 1);
        assert!(!j.stdout.contains('\u{1b}'));
        let v: serde_json::Value = serde_json::from_str(&j.stdout).unwrap_or_default();
        assert_eq!(v["hallazgos"].as_array().map(Vec::len), Some(6));
        assert_eq!(v["hallazgos"][5]["estado"], "ok");
        assert_eq!(v["claves"]["inventario"], inv.huella());
        assert_eq!(v["inventario"][0]["id"], "alfa");
        assert_eq!(v["inventario"][1]["origen"], "integrado");
    }
}
