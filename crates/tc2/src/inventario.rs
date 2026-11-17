//! Inventario de skills y agentes instalados (F3-9, RF-14; ADR F1-15
//! decision 2): la unica fuente de lo que existe en esta maquina y en este
//! proyecto. Lo usan `tc2 doctor --recursos` y, con el manifiesto de F1-14
//! (punto 8), `init`, `start` y `seleccionar`.
//!
//! Fuentes, todas de solo lectura:
//!
//! - los plugins de `<claude>/plugins/installed_plugins.json` (version 2:
//!   `plugins.<nombre@marketplace>` con `installPath`, `version` y `scope`)
//!   que esten habilitados: `enabledPlugins` de `settings.json` del usuario,
//!   `.claude/settings.json` y `.claude/settings.local.json` del proyecto, el
//!   ultimo que lo dice gana (D2a: `false` no cuenta como instalado); sin
//!   entrada, `defaultEnabled` de su `.claude-plugin/plugin.json` (por defecto
//!   `true`). Un `scope` de proyecto o local solo cuenta si su `projectPath`
//!   es este proyecto. Sus skills (`skills/<n>/SKILL.md`) y agentes
//!   (`agents/<n>.md`) llevan el prefijo del plugin (`plugin:id`);
//! - `<claude>/skills` y `<claude>/agents` (usuario);
//! - `.claude/skills` y `.claude/agents` del proyecto;
//! - los agentes integrados de Claude Code ([`INTEGRADOS`]).
//!
//! El id es el `name:` del frontmatter y, si falta, el nombre de la carpeta o
//! del fichero, con aviso si no coinciden (D5a). Solo se lee la cabecera de
//! cada fichero ([`MAX_CABECERA`]). Lo que no se puede leer no se da por
//! instalado ni por ausente: queda en [`Lectura::ilegibles`] y quien decide
//! lo dice como `no-comprobado` (C8).
//!
//! El resultado no depende del orden del sistema de ficheros: se ordena por
//! (tipo, id) y no se repite (C10).

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::contrato::cadena_json;

/// Agentes integrados de Claude Code que siempre estan.
pub const INTEGRADOS: [&str; 1] = ["general-purpose"];

/// Bytes que se leen como mucho de cada `SKILL.md` o agente: el frontmatter
/// va al principio y un fichero enorme no se lee entero.
pub const MAX_CABECERA: u64 = 16 * 1024;

/// Fichero de plugins instalados de Claude Code, relativo a su carpeta.
const INSTALADOS: [&str; 2] = ["plugins", "installed_plugins.json"];

/// Que es una entrada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tipo {
    /// Una skill (`SKILL.md`).
    Skill,
    /// Un agente (`<n>.md` en `agents/`).
    Agente,
}

impl Tipo {
    /// Valor en la salida: `skill` o `agente`.
    pub fn texto(self) -> &'static str {
        match self {
            Self::Skill => "skill",
            Self::Agente => "agente",
        }
    }
}

/// De donde viene una entrada.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Origen {
    /// `<claude>/skills` o `<claude>/agents`.
    Usuario,
    /// `.claude/skills` o `.claude/agents` del proyecto.
    Proyecto,
    /// Un plugin habilitado.
    Plugin {
        /// Nombre del plugin (el prefijo de sus ids).
        nombre: String,
        /// Version instalada, si se sabe.
        version: Option<String>,
    },
    /// Integrado en Claude Code.
    Integrado,
}

impl Origen {
    /// Valor en la salida: `usuario`, `proyecto`, `plugin` o `integrado`.
    pub fn texto(&self) -> &'static str {
        match self {
            Self::Usuario => "usuario",
            Self::Proyecto => "proyecto",
            Self::Plugin { .. } => "plugin",
            Self::Integrado => "integrado",
        }
    }
}

/// Una skill o un agente instalado.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entrada {
    /// Tipo (primero, para ordenar por tipo y luego por id).
    pub tipo: Tipo,
    /// Id con el que se nombra (`plugin:n` en un plugin).
    pub id: String,
    /// De donde viene.
    pub origen: Origen,
}

impl Entrada {
    /// La entrada como objeto JSON.
    pub fn json(&self) -> String {
        let mut s = format!(
            "{{\"id\":{},\"tipo\":{},\"origen\":{}",
            cadena_json(&self.id),
            cadena_json(self.tipo.texto()),
            cadena_json(self.origen.texto())
        );
        if let Origen::Plugin { nombre, version } = &self.origen {
            s.push_str(&format!(",\"plugin\":{}", cadena_json(nombre)));
            if let Some(v) = version {
                s.push_str(&format!(",\"version\":{}", cadena_json(v)));
            }
        }
        s.push('}');
        s
    }
}

/// Un plugin habilitado.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Plugin {
    /// Nombre (el prefijo de sus ids).
    pub nombre: String,
    /// Clave en `installed_plugins.json` (`nombre@marketplace`).
    pub clave: String,
    /// Version instalada, si se sabe.
    pub version: Option<String>,
}

/// Lo instalado y habilitado, ordenado y sin repetir.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Inventario {
    entradas: Vec<Entrada>,
    plugins: Vec<Plugin>,
}

impl Inventario {
    /// Inventario de `entradas` y `plugins` en cualquier orden: se ordenan
    /// por (tipo, id) y una entrada repetida (mismo tipo e id) se queda con
    /// el primer origen en el orden de [`Origen`].
    pub fn nuevo(mut entradas: Vec<Entrada>, mut plugins: Vec<Plugin>) -> Self {
        entradas.sort();
        entradas.dedup_by(|b, a| a.tipo == b.tipo && a.id == b.id);
        plugins.sort();
        plugins.dedup_by(|b, a| a.nombre == b.nombre);
        Self { entradas, plugins }
    }

    /// Las entradas, ordenadas por (tipo, id).
    pub fn entradas(&self) -> &[Entrada] {
        &self.entradas
    }

    /// Los plugins habilitados, por nombre.
    pub fn plugins(&self) -> &[Plugin] {
        &self.plugins
    }

    /// Si hay una entrada de `tipo` con `id`.
    pub fn contiene(&self, tipo: Tipo, id: &str) -> bool {
        self.entradas.iter().any(|e| e.tipo == tipo && e.id == id)
    }

    /// El plugin habilitado de nombre `nombre`.
    pub fn plugin(&self, nombre: &str) -> Option<&Plugin> {
        self.plugins.iter().find(|p| p.nombre == nombre)
    }

    /// Cuantas entradas hay de `tipo`.
    pub fn cuantas(&self, tipo: Tipo) -> usize {
        self.entradas.iter().filter(|e| e.tipo == tipo).count()
    }

    /// Huella del inventario (12 hex del sha256 de su forma canonica): la
    /// que se anota en `roles.<rol>.inventario` (F1-15 decision 10).
    pub fn huella(&self) -> String {
        let mut s = String::new();
        for e in &self.entradas {
            s.push_str(&format!(
                "{}\t{}\t{}",
                e.tipo.texto(),
                e.id,
                e.origen.texto()
            ));
            if let Origen::Plugin { nombre, version } = &e.origen {
                s.push_str(&format!("\t{nombre}\t{}", version.as_deref().unwrap_or("")));
            }
            s.push('\n');
        }
        for p in &self.plugins {
            s.push_str(&format!(
                "plugin\t{}\t{}\t{}\n",
                p.nombre,
                p.clave,
                p.version.as_deref().unwrap_or("")
            ));
        }
        crate::contrato::corto(&distribucion::sha256::hex(s.as_bytes()))
    }

    /// Las entradas como lista JSON.
    pub fn json(&self) -> String {
        let v: Vec<String> = self.entradas.iter().map(Entrada::json).collect();
        format!("[{}]", v.join(","))
    }
}

/// Algo que se encontro al leer: un fichero que no se pudo leer o un
/// nombre que no coincide con su carpeta.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Incidencia {
    /// A que se refiere: el id de la entrada o la clave del plugin.
    pub id: String,
    /// Por que (sin espacios): `no-se-lee`, `json-invalido`,
    /// `formato-desconocido`, `sin-install-path`, `install-path-inexistente`
    /// o `nombre-distinto`.
    pub motivo: &'static str,
    /// Fichero o carpeta.
    pub ruta: String,
}

/// Lo que da [`leer`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Lectura {
    /// Lo instalado y habilitado.
    pub inventario: Inventario,
    /// Fuentes que no se pudieron leer: el inventario puede estar
    /// incompleto.
    pub ilegibles: Vec<Incidencia>,
    /// Avisos que no impiden leer (`nombre-distinto`).
    pub avisos: Vec<Incidencia>,
    /// Nombres de servidores MCP configurados (deteccion aproximada para el
    /// nivel conveniente de F1-14): `.claude.json`, `.mcp.json` del proyecto y
    /// de cada plugin habilitado.
    pub mcp: BTreeSet<String>,
}

/// Lo que se va acumulando al leer.
#[derive(Default)]
struct Acumulado {
    entradas: Vec<Entrada>,
    plugins: Vec<Plugin>,
    ilegibles: Vec<Incidencia>,
    avisos: Vec<Incidencia>,
    mcp: BTreeSet<String>,
}

fn t(p: &Path) -> String {
    p.display().to_string()
}

/// Lee un fichero; `Ok(None)` si no existe.
fn leer_fichero(p: &Path) -> std::io::Result<Option<Vec<u8>>> {
    match std::fs::read(p) {
        Ok(b) => Ok(Some(b)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// Ruta de `installed_plugins.json` en la carpeta de Claude Code.
pub fn ruta_instalados(dir_claude: &Path) -> PathBuf {
    INSTALADOS
        .iter()
        .fold(dir_claude.to_path_buf(), |p, s| p.join(s))
}

/// `installed_plugins.json` de Claude Code (solo lectura): `Ok(None)` si no
/// existe; `Ok(Some(Err))` si no es JSON. Lo usan el inventario y
/// `desinstalar --maquina`.
pub fn leer_instalados(
    dir_claude: &Path,
) -> std::io::Result<Option<Result<Value, serde_json::Error>>> {
    Ok(leer_fichero(&ruta_instalados(dir_claude))?.map(|b| serde_json::from_slice(&b)))
}

/// Un JSON de configuracion: `None` si no existe; incidencia si no se lee o
/// no es JSON.
fn json_de(p: &Path, a: &mut Acumulado, id: &str) -> Option<Value> {
    match leer_fichero(p) {
        Ok(None) => None,
        Ok(Some(b)) => match serde_json::from_slice(&b) {
            Ok(v) => Some(v),
            Err(_) => {
                a.ilegibles.push(Incidencia {
                    id: id.to_owned(),
                    motivo: "json-invalido",
                    ruta: t(p),
                });
                None
            }
        },
        Err(_) => {
            a.ilegibles.push(Incidencia {
                id: id.to_owned(),
                motivo: "no-se-lee",
                ruta: t(p),
            });
            None
        }
    }
}

/// El `name:` del frontmatter (`---` ... `---`) de los primeros
/// [`MAX_CABECERA`] bytes; `None` si no hay. Las comillas se quitan.
pub fn nombre_del_frontmatter(cabecera: &str) -> Option<String> {
    let texto = cabecera.strip_prefix('\u{feff}').unwrap_or(cabecera);
    let mut lineas = texto.lines();
    if lineas.next().map(str::trim_end) != Some("---") {
        return None;
    }
    for l in lineas {
        let l = l.trim_end();
        if l == "---" {
            break;
        }
        if let Some(v) = l.strip_prefix("name:") {
            let v = v.trim();
            let v = v
                .strip_prefix('"')
                .and_then(|x| x.strip_suffix('"'))
                .or_else(|| v.strip_prefix('\'').and_then(|x| x.strip_suffix('\'')))
                .unwrap_or(v)
                .trim();
            return (!v.is_empty()).then(|| v.to_owned());
        }
    }
    None
}

/// La cabecera de `p` (como mucho [`MAX_CABECERA`] bytes).
fn cabecera(p: &Path) -> std::io::Result<String> {
    let mut b = Vec::new();
    std::fs::File::open(p)?
        .take(MAX_CABECERA)
        .read_to_end(&mut b)?;
    Ok(String::from_utf8_lossy(&b).into_owned())
}

/// Hijos de `dir` ordenados (vacio si no existe).
fn hijos(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|d| d.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    v.sort();
    v
}

/// Lee las skills de `dir` (`<n>/SKILL.md`) y los agentes de `agentes`
/// (`<n>.md`) con el `prefijo` y el `origen` dados.
fn leer_carpetas(skills: &Path, agentes: &Path, prefijo: &str, origen: &Origen, a: &mut Acumulado) {
    let mut ficheros: Vec<(Tipo, PathBuf, String)> = Vec::new();
    for d in hijos(skills) {
        let f = d.join("SKILL.md");
        if d.is_dir()
            && f.is_file()
            && let Some(n) = d.file_name().and_then(|n| n.to_str())
        {
            ficheros.push((Tipo::Skill, f, n.to_owned()));
        }
    }
    for f in hijos(agentes) {
        if f.is_file()
            && f.extension().is_some_and(|e| e.eq_ignore_ascii_case("md"))
            && let Some(n) = f.file_stem().and_then(|n| n.to_str())
        {
            let n = n.to_owned();
            ficheros.push((Tipo::Agente, f, n));
        }
    }
    for (tipo, f, carpeta) in ficheros {
        let nombre = match cabecera(&f) {
            Ok(c) => nombre_del_frontmatter(&c),
            Err(_) => {
                a.ilegibles.push(Incidencia {
                    id: format!("{prefijo}{carpeta}"),
                    motivo: "no-se-lee",
                    ruta: t(&f),
                });
                continue;
            }
        };
        let id = match nombre {
            Some(n) => {
                if n != carpeta {
                    a.avisos.push(Incidencia {
                        id: format!("{prefijo}{n}"),
                        motivo: "nombre-distinto",
                        ruta: t(&f),
                    });
                }
                n
            }
            None => carpeta,
        };
        a.entradas.push(Entrada {
            tipo,
            id: format!("{prefijo}{id}"),
            origen: origen.clone(),
        });
    }
}

/// Las claves de `mcpServers` de un JSON.
fn servidores_mcp(v: &Value, a: &mut Acumulado) {
    if let Some(m) = v.get("mcpServers").and_then(Value::as_object) {
        a.mcp.extend(m.keys().cloned());
    }
}

/// `enabledPlugins` de los tres ficheros de settings, el ultimo gana.
fn habilitados(dir_claude: &Path, raiz: &Path, a: &mut Acumulado) -> BTreeMap<String, bool> {
    let mut m = BTreeMap::new();
    let claude = raiz.join(".claude");
    for p in [
        dir_claude.join("settings.json"),
        claude.join("settings.json"),
        claude.join("settings.local.json"),
    ] {
        if let Some(v) = json_de(&p, a, "settings")
            && let Some(e) = v.get("enabledPlugins").and_then(Value::as_object)
        {
            for (k, x) in e {
                if let Some(b) = x.as_bool() {
                    m.insert(k.clone(), b);
                }
            }
        }
    }
    m
}

/// Si una entrada de `installed_plugins.json` vale para este proyecto: de
/// usuario (o sin `scope`) siempre; de proyecto o local, si su
/// `projectPath` es `raiz`.
fn aplica(x: &Value, raiz: &Path) -> bool {
    match x.get("scope").and_then(Value::as_str) {
        None | Some("user") => true,
        Some(_) => x
            .get("projectPath")
            .and_then(Value::as_str)
            .is_some_and(|p| crate::git::misma_ruta(Path::new(p), raiz)),
    }
}

/// Los plugins habilitados con sus skills y agentes.
fn leer_plugins(dir_claude: &Path, raiz: &Path, a: &mut Acumulado) {
    let ruta = ruta_instalados(dir_claude);
    let v = match leer_instalados(dir_claude) {
        Ok(None) => return,
        Ok(Some(Ok(v))) => v,
        Ok(Some(Err(_))) => {
            a.ilegibles.push(Incidencia {
                id: "plugins".to_owned(),
                motivo: "json-invalido",
                ruta: t(&ruta),
            });
            return;
        }
        Err(_) => {
            a.ilegibles.push(Incidencia {
                id: "plugins".to_owned(),
                motivo: "no-se-lee",
                ruta: t(&ruta),
            });
            return;
        }
    };
    let Some(mapa) = v.get("plugins").and_then(Value::as_object) else {
        a.ilegibles.push(Incidencia {
            id: "plugins".to_owned(),
            motivo: "formato-desconocido",
            ruta: t(&ruta),
        });
        return;
    };
    let estado = habilitados(dir_claude, raiz, a);
    for (clave, entradas) in mapa {
        let Some(x) = entradas
            .as_array()
            .and_then(|l| l.iter().find(|x| aplica(x, raiz)))
        else {
            continue;
        };
        if estado.get(clave) == Some(&false) {
            continue;
        }
        let Some(install) = x.get("installPath").and_then(Value::as_str) else {
            a.ilegibles.push(Incidencia {
                id: clave.clone(),
                motivo: "sin-install-path",
                ruta: t(&ruta),
            });
            continue;
        };
        let dir = PathBuf::from(install);
        if !dir.is_dir() {
            a.ilegibles.push(Incidencia {
                id: clave.clone(),
                motivo: "install-path-inexistente",
                ruta: install.to_owned(),
            });
            continue;
        }
        let manifiesto = json_de(&dir.join(".claude-plugin").join("plugin.json"), a, clave);
        let por_defecto = manifiesto
            .as_ref()
            .and_then(|m| m.get("defaultEnabled"))
            .and_then(Value::as_bool)
            .unwrap_or(true);
        if !estado.get(clave).copied().unwrap_or(por_defecto) {
            continue;
        }
        let nombre = manifiesto
            .as_ref()
            .and_then(|m| m.get("name"))
            .and_then(Value::as_str)
            .filter(|n| !n.is_empty())
            .map_or_else(
                || clave.split('@').next().unwrap_or(clave).to_owned(),
                str::to_owned,
            );
        let version = x
            .get("version")
            .and_then(Value::as_str)
            .or_else(|| {
                manifiesto
                    .as_ref()
                    .and_then(|m| m.get("version"))
                    .and_then(Value::as_str)
            })
            .map(str::to_owned);
        if let Some(m) = json_de(&dir.join(".mcp.json"), a, clave) {
            servidores_mcp(&m, a);
        }
        let origen = Origen::Plugin {
            nombre: nombre.clone(),
            version: version.clone(),
        };
        leer_carpetas(
            &dir.join("skills"),
            &dir.join("agents"),
            &format!("{nombre}:"),
            &origen,
            a,
        );
        a.plugins.push(Plugin {
            nombre,
            clave: clave.clone(),
            version,
        });
    }
}

/// `.claude.json` de Claude Code: dentro de `CLAUDE_CONFIG_DIR` si se dio
/// una carpeta propia, o junto a `~/.claude` (`~/.claude.json`).
fn ruta_claude_json(dir_claude: &Path) -> PathBuf {
    match (dir_claude.file_name(), dir_claude.parent()) {
        (Some(n), Some(padre)) if n == ".claude" => padre.join(".claude.json"),
        _ => dir_claude.join(".claude.json"),
    }
}

/// El inventario de la maquina (`dir_claude`, la carpeta de Claude Code; sin
/// ella, solo el proyecto y lo integrado) y del proyecto en `raiz`. Solo
/// lee.
pub fn leer(dir_claude: Option<&Path>, raiz: &Path) -> Lectura {
    let mut a = Acumulado::default();
    if let Some(d) = dir_claude {
        leer_plugins(d, raiz, &mut a);
        leer_carpetas(
            &d.join("skills"),
            &d.join("agents"),
            "",
            &Origen::Usuario,
            &mut a,
        );
        if let Some(v) = json_de(&ruta_claude_json(d), &mut a, "mcp") {
            servidores_mcp(&v, &mut a);
        }
    }
    let proyecto = raiz.join(".claude");
    leer_carpetas(
        &proyecto.join("skills"),
        &proyecto.join("agents"),
        "",
        &Origen::Proyecto,
        &mut a,
    );
    if let Some(v) = json_de(&raiz.join(".mcp.json"), &mut a, "mcp") {
        servidores_mcp(&v, &mut a);
    }
    for id in INTEGRADOS {
        a.entradas.push(Entrada {
            tipo: Tipo::Agente,
            id: id.to_owned(),
            origen: Origen::Integrado,
        });
    }
    a.ilegibles.sort();
    a.ilegibles.dedup();
    a.avisos.sort();
    a.avisos.dedup();
    Lectura {
        inventario: Inventario::nuevo(a.entradas, a.plugins),
        ilegibles: a.ilegibles,
        avisos: a.avisos,
        mcp: a.mcp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Carpeta temporal propia que se borra al terminar (nunca el `.claude`
    /// real).
    struct Tmp(PathBuf);

    impl Tmp {
        fn nueva(nombre: &str) -> Self {
            let p = std::env::temp_dir()
                .join("tc2-tests")
                .join(format!("inventario-{nombre}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&p);
            std::fs::create_dir_all(&p).unwrap_or_default();
            Self(p)
        }

        fn escribir(&self, rel: &str, texto: &str) {
            let p = self.0.join(rel);
            if let Some(d) = p.parent() {
                std::fs::create_dir_all(d).unwrap_or_default();
            }
            std::fs::write(p, texto).unwrap_or_default();
        }
    }

    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn skill(nombre: &str) -> String {
        format!("---\nname: {nombre}\ndescription: x\n---\n# cuerpo\n")
    }

    fn lineas(l: &Lectura) -> Vec<String> {
        l.inventario
            .entradas()
            .iter()
            .map(|e| format!("{} {} {}", e.tipo.texto(), e.id, e.origen.texto()))
            .collect()
    }

    #[test]
    fn el_frontmatter_da_el_nombre() {
        assert_eq!(
            nombre_del_frontmatter("---\nname: alfa\n---\n").as_deref(),
            Some("alfa")
        );
        assert_eq!(
            nombre_del_frontmatter("\u{feff}---\r\ndescription: x\r\nname: \"beta\"\r\n---\r\n")
                .as_deref(),
            Some("beta")
        );
        assert_eq!(
            nombre_del_frontmatter("---\nname: 'g'\n---").as_deref(),
            Some("g")
        );
        for sin in [
            "",
            "# titulo\nname: x\n",
            "---\ndescription: x\n---\nname: x\n",
            "---\nname:\n---\n",
            "---\n  name: sangrado\n---\n",
        ] {
            assert_eq!(nombre_del_frontmatter(sin), None, "{sin:?}");
        }
    }

    /// C1 (parte de biblioteca) y D5a: lo de usuario, proyecto, el plugin
    /// habilitado con su prefijo y lo integrado; sin el plugin apagado ni el
    /// de otro proyecto; ordenado por (tipo, id); un `name:` distinto del de
    /// su carpeta da el id del frontmatter con aviso.
    #[test]
    fn lee_lo_instalado_y_habilitado() {
        let m = Tmp::nueva("c1");
        let claude = m.0.join("claude");
        let raiz = m.0.join("proyecto");
        let bueno = m.0.join("cache").join("bueno");
        let apagado = m.0.join("cache").join("apagado");
        let ajeno = m.0.join("cache").join("ajeno");
        m.escribir("claude/skills/zeta/SKILL.md", &skill("zeta"));
        m.escribir("claude/skills/carpeta/SKILL.md", &skill("otro-nombre"));
        m.escribir("claude/skills/sin-skill/LEEME.md", "x");
        m.escribir("claude/agents/revisor.md", "---\nname: revisor\n---\n");
        m.escribir("claude/agents/notas.txt", "x");
        m.escribir(
            "proyecto/.claude/skills/local/SKILL.md",
            "# sin frontmatter\n",
        );
        m.escribir("cache/bueno/skills/rev/SKILL.md", &skill("rev"));
        m.escribir("cache/bueno/agents/ag.md", "---\nname: ag\n---\n");
        m.escribir(
            "cache/bueno/.claude-plugin/plugin.json",
            r#"{"name": "bueno", "version": "9.9.9"}"#,
        );
        m.escribir("cache/bueno/.mcp.json", r#"{"mcpServers": {"engram": {}}}"#);
        m.escribir("cache/apagado/skills/x/SKILL.md", &skill("x"));
        m.escribir("cache/ajeno/skills/y/SKILL.md", &skill("y"));
        let instalados = serde_json::json!({
            "version": 2,
            "plugins": {
                "bueno@mk": [{"scope": "user", "installPath": bueno, "version": "1.0.0"}],
                "apagado@mk": [{"scope": "user", "installPath": apagado}],
                "ajeno@mk": [{"scope": "project", "projectPath": m.0.join("otro"), "installPath": ajeno}]
            }
        });
        m.escribir(
            "claude/plugins/installed_plugins.json",
            &instalados.to_string(),
        );
        m.escribir(
            "claude/settings.json",
            r#"{"enabledPlugins": {"bueno@mk": false, "apagado@mk": false}}"#,
        );
        // El proyecto vuelve a habilitar `bueno` (gana el mas cercano).
        m.escribir(
            "proyecto/.claude/settings.local.json",
            r#"{"enabledPlugins": {"bueno@mk": true}}"#,
        );
        let l = leer(Some(&claude), &raiz);
        assert_eq!(
            lineas(&l),
            [
                "skill bueno:rev plugin",
                "skill local proyecto",
                "skill otro-nombre usuario",
                "skill zeta usuario",
                "agente bueno:ag plugin",
                "agente general-purpose integrado",
                "agente revisor usuario",
            ]
        );
        assert_eq!(l.ilegibles, []);
        assert_eq!(
            l.avisos
                .iter()
                .map(|i| format!("{} {}", i.id, i.motivo))
                .collect::<Vec<_>>(),
            ["otro-nombre nombre-distinto"]
        );
        assert_eq!(
            l.inventario.plugins(),
            [Plugin {
                nombre: "bueno".to_owned(),
                clave: "bueno@mk".to_owned(),
                version: Some("1.0.0".to_owned()),
            }]
        );
        assert!(l.mcp.contains("engram"));
        assert!(l.inventario.contiene(Tipo::Skill, "bueno:rev"));
        assert!(!l.inventario.contiene(Tipo::Agente, "bueno:rev"));
        assert_eq!(l.inventario.huella().len(), 12);
        // Sin carpeta de Claude Code: solo el proyecto y lo integrado.
        let solo = leer(None, &raiz);
        assert_eq!(
            lineas(&solo),
            ["skill local proyecto", "agente general-purpose integrado"]
        );
    }

    /// C8: `installed_plugins.json` corrupto o un `installPath` que no existe
    /// quedan como ilegibles con su motivo; el plugin no cuenta.
    #[test]
    fn plugins_ilegibles_no_cuentan() {
        let m = Tmp::nueva("c8");
        let claude = m.0.join("claude");
        m.escribir("claude/plugins/installed_plugins.json", "{ no es json");
        let l = leer(Some(&claude), &m.0);
        assert_eq!(
            l.ilegibles
                .iter()
                .map(|i| format!("{} {}", i.id, i.motivo))
                .collect::<Vec<_>>(),
            ["plugins json-invalido"]
        );
        let instalados = serde_json::json!({"version": 2, "plugins": {
            "taskcodetwo@mk": [{"scope": "user", "installPath": m.0.join("no-existe")}],
            "sin@mk": [{"scope": "user"}]
        }});
        m.escribir(
            "claude/plugins/installed_plugins.json",
            &instalados.to_string(),
        );
        let l = leer(Some(&claude), &m.0);
        assert_eq!(
            l.ilegibles
                .iter()
                .map(|i| format!("{} {}", i.id, i.motivo))
                .collect::<Vec<_>>(),
            [
                "sin@mk sin-install-path",
                "taskcodetwo@mk install-path-inexistente"
            ]
        );
        assert!(l.inventario.plugin("taskcodetwo").is_none());
        m.escribir("claude/plugins/installed_plugins.json", "[1, 2]");
        let l = leer(Some(&claude), &m.0);
        assert_eq!(l.ilegibles[0].motivo, "formato-desconocido");
    }

    /// C10 (parte de biblioteca): el mismo contenido creado en otro orden da
    /// el mismo inventario y la misma huella; otro contenido, otra huella.
    #[test]
    fn determinista_con_otro_orden_de_creacion() {
        let nombres = ["c", "a", "b"];
        let mut huellas = Vec::new();
        for (i, orden) in [nombres, [nombres[2], nombres[1], nombres[0]]]
            .iter()
            .enumerate()
        {
            let m = Tmp::nueva(&format!("c10-{i}"));
            for n in orden {
                m.escribir(&format!("claude/skills/{n}/SKILL.md"), &skill(n));
                m.escribir(&format!("claude/agents/ag-{n}.md"), "x");
            }
            let l = leer(Some(&m.0.join("claude")), &m.0.join("p"));
            huellas.push((l.inventario.huella(), l.inventario.json()));
        }
        assert_eq!(huellas[0], huellas[1]);
        let otro = Inventario::nuevo(
            vec![Entrada {
                tipo: Tipo::Skill,
                id: "a".to_owned(),
                origen: Origen::Proyecto,
            }],
            Vec::new(),
        );
        assert_ne!(otro.huella(), huellas[0].0);
    }

    #[test]
    fn la_cabecera_de_un_fichero_enorme_se_corta() {
        let m = Tmp::nueva("enorme");
        let mut texto = skill("grande");
        texto.push_str(&"x".repeat(usize::try_from(MAX_CABECERA).unwrap_or(0) * 4));
        m.escribir("s/grande/SKILL.md", &texto);
        let c = cabecera(&m.0.join("s/grande/SKILL.md")).unwrap_or_default();
        assert_eq!(c.len(), usize::try_from(MAX_CABECERA).unwrap_or(0));
        assert_eq!(nombre_del_frontmatter(&c).as_deref(), Some("grande"));
    }
}
