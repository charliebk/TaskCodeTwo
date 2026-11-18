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
//! - los plugins sincronizados desde claude.ai (`<claude>/plugins/synced/`,
//!   id `<name>@synced`): los del `manifest.json` de cuenta mas reciente,
//!   salvo `"<name>@synced": false`, `defaultEnabled: false` o
//!   `syncClaudeAiPlugins: false`; ceden ante un plugin de otro origen con el
//!   mismo nombre y `plugins/.trash/` no cuenta;
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
    /// `formato-desconocido`, `sin-install-path`, `install-path-relativo`,
    /// `install-path-inexistente`, `sin-carpeta`, `nombre-invalido`,
    /// `nombre-no-utf8` o `nombre-distinto`.
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

/// El valor de una linea `clave: valor` de YAML: entre comillas (simples o
/// dobles), lo de dentro, con lo que siga detras ignorado; sin comillas,
/// hasta un comentario (`#` al principio o tras un espacio o tabulador).
fn valor_yaml(v: &str) -> &str {
    let v = v.trim();
    for q in ['"', '\''] {
        if let Some(resto) = v.strip_prefix(q)
            && let Some((dentro, _)) = resto.split_once(q)
        {
            return dentro.trim();
        }
    }
    if v.starts_with('#') {
        return "";
    }
    let corte = v
        .char_indices()
        .zip(v.chars().skip(1))
        .find(|((_, c), sig)| (*c == ' ' || *c == '\t') && *sig == '#')
        .map_or(v.len(), |((i, _), _)| i);
    v.get(..corte).unwrap_or(v).trim()
}

/// El `name:` del frontmatter (`---` ... `---`) de los primeros
/// [`MAX_CABECERA`] bytes; `None` si no hay. Las comillas y un comentario
/// final (`name: foo # nota`) se quitan.
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
            let v = valor_yaml(v);
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

/// Hijos de `dir` ordenados (vacio si no existe). Si la carpeta o una de
/// sus entradas no se puede leer (p. ej. sin permiso), una incidencia
/// `no-se-lee` de `id` en `ilegibles`: nunca cuenta como vacia sin decirlo.
fn hijos(dir: &Path, id: &str, a: &mut Acumulado) -> Vec<PathBuf> {
    let mut no_se_lee = || {
        a.ilegibles.push(Incidencia {
            id: id.to_owned(),
            motivo: "no-se-lee",
            ruta: t(dir),
        });
    };
    let mut v = Vec::new();
    match std::fs::read_dir(dir) {
        Ok(d) => {
            for e in d {
                match e {
                    Ok(e) => v.push(e.path()),
                    Err(_) => no_se_lee(),
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => no_se_lee(),
    }
    v.sort();
    v
}

/// El nombre de `n` como texto o, si no es UTF-8, una incidencia
/// `nombre-no-utf8` (con el nombre aproximado como id) y `None`.
fn nombre_utf8(
    n: Option<&std::ffi::OsStr>,
    prefijo: &str,
    ruta: &Path,
    a: &mut Acumulado,
) -> Option<String> {
    let n = n?;
    if let Some(s) = n.to_str() {
        return Some(s.to_owned());
    }
    a.ilegibles.push(Incidencia {
        id: format!("{prefijo}{}", n.to_string_lossy()),
        motivo: "nombre-no-utf8",
        ruta: t(ruta),
    });
    None
}

/// Lee las skills de `dir` (`<n>/SKILL.md`) y los agentes de `agentes`
/// (`<n>.md`) con el `prefijo` y el `origen` dados.
fn leer_carpetas(skills: &Path, agentes: &Path, prefijo: &str, origen: &Origen, a: &mut Acumulado) {
    let mut ficheros: Vec<(Tipo, PathBuf, String)> = Vec::new();
    for d in hijos(skills, &format!("{prefijo}skills"), a) {
        let f = d.join("SKILL.md");
        if d.is_dir()
            && f.is_file()
            && let Some(n) = nombre_utf8(d.file_name(), prefijo, &d, a)
        {
            ficheros.push((Tipo::Skill, f, n));
        }
    }
    for f in hijos(agentes, &format!("{prefijo}agents"), a) {
        if f.is_file()
            && f.extension().is_some_and(|e| e.eq_ignore_ascii_case("md"))
            && let Some(n) = nombre_utf8(f.file_stem(), prefijo, &f, a)
        {
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

/// Lo que dicen los settings sobre los plugins.
struct Ajustes {
    /// `enabledPlugins` de los tres ficheros, el ultimo gana.
    habilitados: BTreeMap<String, bool>,
    /// `syncClaudeAiPlugins` del usuario (por defecto `true`).
    sincronizar: bool,
}

/// `enabledPlugins` de los tres ficheros de settings (el ultimo gana) y
/// `syncClaudeAiPlugins` de los del usuario.
fn ajustes(dir_claude: &Path, raiz: &Path, a: &mut Acumulado) -> Ajustes {
    let mut m = BTreeMap::new();
    let mut sincronizar = true;
    let claude = raiz.join(".claude");
    for (i, p) in [
        dir_claude.join("settings.json"),
        claude.join("settings.json"),
        claude.join("settings.local.json"),
    ]
    .iter()
    .enumerate()
    {
        let Some(v) = json_de(p, a, "settings") else {
            continue;
        };
        if let Some(e) = v.get("enabledPlugins").and_then(Value::as_object) {
            for (k, x) in e {
                if let Some(b) = x.as_bool() {
                    m.insert(k.clone(), b);
                }
            }
        }
        if i == 0 && v.get("syncClaudeAiPlugins").and_then(Value::as_bool) == Some(false) {
            sincronizar = false;
        }
    }
    Ajustes {
        habilitados: m,
        sincronizar,
    }
}

/// Carga el plugin de `dir` con clave `clave` (`nombre@origen`) si esta
/// habilitado: `enabledPlugins` o, sin entrada, `defaultEnabled` de su
/// manifiesto. Su nombre (prefijo de los ids) es el `name` del manifiesto o
/// `por_defecto`. `omitir` descarta un nombre ya cargado (los sincronizados
/// ceden ante cualquier otro origen). Devuelve si lo cargo.
fn cargar_plugin(
    dir: &Path,
    clave: &str,
    por_defecto: &str,
    version: Option<&str>,
    ajustes: &Ajustes,
    omitir: bool,
    a: &mut Acumulado,
) -> bool {
    let manifiesto = json_de(&dir.join(".claude-plugin").join("plugin.json"), a, clave);
    let por_defecto_on = manifiesto
        .as_ref()
        .and_then(|m| m.get("defaultEnabled"))
        .and_then(Value::as_bool)
        .unwrap_or(true);
    if !ajustes
        .habilitados
        .get(clave)
        .copied()
        .unwrap_or(por_defecto_on)
    {
        return false;
    }
    let nombre = manifiesto
        .as_ref()
        .and_then(|m| m.get("name"))
        .and_then(Value::as_str)
        .filter(|n| !n.is_empty())
        .unwrap_or(por_defecto)
        .to_owned();
    if omitir && a.plugins.iter().any(|p| p.nombre == nombre) {
        return false;
    }
    let version = version
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
        clave: clave.to_owned(),
        version,
    });
    true
}

/// Dias desde 1970-01-01 de una fecha civil (algoritmo de H. Hinnant).
fn dias_desde_epoca(anio: i64, mes: i64, dia: i64) -> i64 {
    let y = if mes <= 2 { anio - 1 } else { anio };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (mes + 9) % 12;
    let doy = (153 * mp + 2) / 5 + dia - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Milisegundos desde la epoca de un texto ISO 8601 / RFC 3339
/// (`AAAA-MM-DD`, opcionalmente `THH:MM[:SS[.fff]]` y `Z` o `+HH:MM`);
/// `None` si no tiene esa forma o algun campo esta fuera de rango.
pub(crate) fn ms_de_iso(s: &str) -> Option<i64> {
    let num = |x: &str| -> Option<i64> {
        (!x.is_empty() && x.bytes().all(|b| b.is_ascii_digit()))
            .then(|| x.parse().ok())
            .flatten()
    };
    let (fecha, resto) = s.split_at_checked(10)?;
    let mut f = fecha.split('-');
    let (anio, mes, dia) = (num(f.next()?)?, num(f.next()?)?, num(f.next()?)?);
    if f.next().is_some() || !(1..=12).contains(&mes) || !(1..=31).contains(&dia) {
        return None;
    }
    let mut ms = dias_desde_epoca(anio, mes, dia) * 86_400_000;
    if resto.is_empty() {
        return Some(ms);
    }
    let resto = resto.strip_prefix(['T', 't', ' '])?;
    // Zona: `Z`, `+HH:MM` o `-HH:MM` al final.
    let (hora, zona) = match resto.find(['Z', 'z', '+', '-']) {
        Some(i) => resto.split_at(i),
        None => (resto, ""),
    };
    let desfase = match zona {
        "" | "Z" | "z" => 0,
        z => {
            let signo = if z.starts_with('-') { -1 } else { 1 };
            let (h, m) = z.get(1..)?.split_once(':')?;
            let (h, m) = (num(h)?, num(m)?);
            if h > 23 || m > 59 {
                return None;
            }
            signo * (h * 60 + m) * 60_000
        }
    };
    let (hms, frac) = hora.split_once('.').unwrap_or((hora, ""));
    let mut p = hms.split(':');
    let h = num(p.next()?)?;
    let mi = num(p.next()?)?;
    let se = p.next().map_or(Some(0), num)?;
    if p.next().is_some() || h > 23 || mi > 59 || se > 60 {
        return None;
    }
    let fr = if frac.is_empty() {
        0
    } else {
        if !frac.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        // Los tres primeros digitos son milisegundos.
        let mut d: String = frac.chars().take(3).collect();
        while d.len() < 3 {
            d.push('0');
        }
        num(&d)?
    };
    ms += ((h * 60 + mi) * 60 + se) * 1000 + fr - desfase;
    Some(ms)
}

/// Clave de orden de `lastUpdated`: milisegundos desde la epoca, de un
/// numero (ms) o de un texto ISO 8601. Un valor ausente, negativo, no
/// finito, fuera de rango o que no es fecha da `None`, que nunca gana a una
/// fecha valida (H4 de la ronda 1: se compara por valor, no por texto).
fn orden_de_fecha(v: Option<&Value>) -> Option<i64> {
    match v? {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                return (i >= 0).then_some(i);
            }
            if let Some(u) = n.as_u64() {
                return i64::try_from(u).ok();
            }
            let f = n.as_f64()?;
            // Rango de `i64` en ms con margen; `as` sobre un finito en rango.
            #[allow(clippy::cast_possible_truncation)]
            let ms = (f.is_finite() && (0.0..9.0e15).contains(&f)).then(|| f.trunc() as i64);
            ms
        }
        Value::String(s) => ms_de_iso(s.trim()).filter(|ms| *ms >= 0),
        _ => None,
    }
}

/// Los plugins sincronizados desde claude.ai (`plugins/synced/<cuenta>/`,
/// id `<name>@synced`; doc de Claude Code «Plugin loading reference»). Cuenta
/// solo la carpeta de cuenta cuyo `manifest.json` tiene el `lastUpdated` mas
/// reciente (la de la sesion que sincroniza en cada arranque); de ella, cada
/// plugin del manifiesto con su carpeta `<name>~g<generation>` o `<name>`.
/// `plugins/.trash/` no cuenta. Con `syncClaudeAiPlugins: false` no se leen.
fn leer_sincronizados(dir_claude: &Path, ajustes: &Ajustes, a: &mut Acumulado) {
    if !ajustes.sincronizar {
        return;
    }
    let raiz = dir_claude.join("plugins").join("synced");
    let mut cuentas: Vec<(Option<i64>, PathBuf, Value)> = Vec::new();
    for c in hijos(&raiz, "synced", a).into_iter().filter(|c| c.is_dir()) {
        let id = format!(
            "synced/{}",
            c.file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default()
        );
        let ruta = c.join("manifest.json");
        if !ruta.is_file() {
            continue;
        }
        let Some(m) = json_de(&ruta, a, &id) else {
            continue;
        };
        if !m.get("plugins").is_some_and(Value::is_array) {
            a.ilegibles.push(Incidencia {
                id,
                motivo: "formato-desconocido",
                ruta: t(&ruta),
            });
            continue;
        }
        cuentas.push((orden_de_fecha(m.get("lastUpdated")), c, m));
    }
    // La mas reciente (sin fecha valida, nunca gana a una con fecha); a
    // igualdad, la ultima por nombre (determinista).
    let Some((_, cuenta, m)) = cuentas
        .into_iter()
        .max_by(|x, y| (&x.0, &x.1).cmp(&(&y.0, &y.1)))
    else {
        return;
    };
    let lista = m
        .get("plugins")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let manifiesto = cuenta.join("manifest.json");
    for p in lista {
        // El `name` se une a la ruta de la cuenta: vacio, con separadores o
        // con `..` podria leer fuera de `synced/` (H3 de la ronda 1). No se
        // une: queda como ilegible.
        let nombre = match p.get("name").and_then(Value::as_str) {
            Some(n)
                if !n.is_empty()
                    && n != "."
                    && !n.contains(['/', '\\', ':'])
                    && !n.contains("..") =>
            {
                n
            }
            n => {
                a.ilegibles.push(Incidencia {
                    id: format!("{}@synced", n.unwrap_or_default()),
                    motivo: "nombre-invalido",
                    ruta: t(&manifiesto),
                });
                continue;
            }
        };
        let generacion = p.get("generation").and_then(Value::as_u64).unwrap_or(1);
        let candidatas = [
            (generacion > 1).then(|| cuenta.join(format!("{nombre}~g{generacion}"))),
            Some(cuenta.join(nombre)),
        ];
        let Some(dir) = candidatas.into_iter().flatten().find(|d| d.is_dir()) else {
            a.ilegibles.push(Incidencia {
                id: format!("{nombre}@synced"),
                motivo: "sin-carpeta",
                ruta: t(&cuenta),
            });
            continue;
        };
        let version = p.get("version").and_then(Value::as_str);
        cargar_plugin(
            &dir,
            &format!("{nombre}@synced"),
            nombre,
            version,
            ajustes,
            true,
            a,
        );
    }
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
fn leer_plugins(dir_claude: &Path, raiz: &Path, ajustes: &Ajustes, a: &mut Acumulado) {
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
    // Solo se entiende la version 2 (`plugins.<clave>` es una lista de
    // instalaciones); otra version, o sin ella, no se interpreta (H1 de la
    // ronda 1).
    let Some(mapa) = v
        .get("plugins")
        .and_then(Value::as_object)
        .filter(|_| v.get("version").and_then(Value::as_u64) == Some(2))
    else {
        a.ilegibles.push(Incidencia {
            id: "plugins".to_owned(),
            motivo: "formato-desconocido",
            ruta: t(&ruta),
        });
        return;
    };
    for (clave, entradas) in mapa {
        if ajustes.habilitados.get(clave) == Some(&false) {
            continue;
        }
        // Una entrada que no es lista (objeto, `null`...) no desaparece en
        // silencio: queda como ilegible y lo que dependa de ella sale
        // `no-comprobado` (C8).
        let Some(lista) = entradas.as_array() else {
            a.ilegibles.push(Incidencia {
                id: clave.clone(),
                motivo: "formato-desconocido",
                ruta: t(&ruta),
            });
            continue;
        };
        let Some(x) = lista.iter().find(|x| aplica(x, raiz)) else {
            continue;
        };
        let Some(install) = x.get("installPath").and_then(Value::as_str) else {
            a.ilegibles.push(Incidencia {
                id: clave.clone(),
                motivo: "sin-install-path",
                ruta: t(&ruta),
            });
            continue;
        };
        let dir = PathBuf::from(install);
        // Claude Code escribe `installPath` absoluto. Uno relativo no se
        // resuelve contra el directorio del proceso (dependeria de donde se
        // lance) ni se adivina contra la carpeta de Claude Code: queda como
        // ilegible (H3 de la ronda 1).
        if dir.is_relative() {
            a.ilegibles.push(Incidencia {
                id: clave.clone(),
                motivo: "install-path-relativo",
                ruta: install.to_owned(),
            });
            continue;
        }
        if !dir.is_dir() {
            a.ilegibles.push(Incidencia {
                id: clave.clone(),
                motivo: "install-path-inexistente",
                ruta: install.to_owned(),
            });
            continue;
        }
        cargar_plugin(
            &dir,
            clave,
            clave.split('@').next().unwrap_or(clave),
            x.get("version").and_then(Value::as_str),
            ajustes,
            false,
            a,
        );
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
        let ajustes = ajustes(d, raiz, &mut a);
        leer_plugins(d, raiz, &ajustes, &mut a);
        // Despues de los instalados: un sincronizado cede ante otro origen
        // con el mismo nombre (doc «Plugin loading reference», Name
        // conflicts, punto 5).
        leer_sincronizados(d, &ajustes, &mut a);
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
        // MEN-2 (ronda 1): un comentario final no entra en el id, salvo
        // dentro de comillas; un `#` pegado es parte del valor.
        for (cabecera, nombre) in [
            ("---\nname: foo # comentario\n---\n", "foo"),
            ("---\nname: foo\t# comentario\n---\n", "foo"),
            ("---\nname: \"foo # no\" # si\n---\n", "foo # no"),
            ("---\nname: 'a # b'\n---\n", "a # b"),
            ("---\nname: a#b\n---\n", "a#b"),
        ] {
            assert_eq!(
                nombre_del_frontmatter(cabecera).as_deref(),
                Some(nombre),
                "{cabecera:?}"
            );
        }
        for sin in [
            "",
            "# titulo\nname: x\n",
            "---\ndescription: x\n---\nname: x\n",
            "---\nname:\n---\n",
            "---\nname: # solo comentario\n---\n",
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

    /// Plugins sincronizados desde claude.ai (`plugins/synced/<cuenta>/`):
    /// cuenta la cuenta del `manifest.json` mas reciente; la carpeta `~g2` se
    /// nombra por el `name` de su `plugin.json`; `<name>@synced: false` lo
    /// apaga; cede ante un instalado del mismo nombre; `.trash` no cuenta; un
    /// manifiesto corrupto o un plugin sin carpeta quedan como ilegibles; con
    /// `syncClaudeAiPlugins: false` no se lee nada.
    #[test]
    fn sincronizados_desde_claude_ai() {
        let m = Tmp::nueva("sincronizados");
        let claude = m.0.join("claude");
        let bueno = m.0.join("cache").join("bueno");
        m.escribir("cache/bueno/skills/rev/SKILL.md", &skill("rev"));
        m.escribir(
            "claude/plugins/installed_plugins.json",
            &serde_json::json!({"version": 2, "plugins": {
                "bueno@mk": [{"scope": "user", "installPath": bueno}]
            }})
            .to_string(),
        );
        let s = "claude/plugins/synced";
        let entrada = |n: &str| {
            serde_json::json!({"pluginId": format!("id-{n}"), "name": n, "version": "1.0.0",
                "marketplaceName": "mk-claude", "installationPreference": "available",
                "marketplaceKind": "anthropic_catalog", "generation": 2})
        };
        m.escribir(
            &format!("{s}/cuenta-vieja/manifest.json"),
            &serde_json::json!({"lastUpdated": "2026-01-01T00:00:00Z", "plugins": [entrada("viejo")]})
                .to_string(),
        );
        m.escribir(
            &format!("{s}/cuenta-vieja/viejo~g2/skills/v/SKILL.md"),
            &skill("v"),
        );
        m.escribir(
            &format!("{s}/cuenta-nueva/manifest.json"),
            &serde_json::json!({"lastUpdated": "2026-11-01T00:00:00Z", "plugins": [
                entrada("datos"), entrada("apagado-s"), entrada("dup"), entrada("falta")
            ]})
            .to_string(),
        );
        m.escribir(
            &format!("{s}/cuenta-nueva/datos~g2/.claude-plugin/plugin.json"),
            r#"{"name": "data"}"#,
        );
        m.escribir(&format!("{s}/cuenta-nueva/datos~g2.meta.json"), "{}");
        m.escribir(
            &format!("{s}/cuenta-nueva/datos~g2/skills/analizar/SKILL.md"),
            &skill("analizar"),
        );
        m.escribir(
            &format!("{s}/cuenta-nueva/datos~g2/agents/ag-datos.md"),
            "x",
        );
        m.escribir(
            &format!("{s}/cuenta-nueva/apagado-s/skills/x/SKILL.md"),
            &skill("x"),
        );
        m.escribir(
            &format!("{s}/cuenta-nueva/dup~g2/.claude-plugin/plugin.json"),
            r#"{"name": "bueno"}"#,
        );
        m.escribir(
            &format!("{s}/cuenta-nueva/dup~g2/skills/otra/SKILL.md"),
            &skill("otra"),
        );
        m.escribir(&format!("{s}/cuenta-rota/manifest.json"), "{ roto");
        m.escribir(
            "claude/plugins/.trash/basura/skills/z/SKILL.md",
            &skill("z"),
        );
        m.escribir(
            "claude/settings.json",
            r#"{"enabledPlugins": {"apagado-s@synced": false}}"#,
        );
        let l = leer(Some(&claude), &m.0.join("p"));
        assert_eq!(
            lineas(&l),
            [
                "skill bueno:rev plugin",
                "skill data:analizar plugin",
                "agente data:ag-datos plugin",
                "agente general-purpose integrado",
            ]
        );
        assert_eq!(
            l.inventario
                .plugins()
                .iter()
                .map(|p| format!("{} {} {:?}", p.nombre, p.clave, p.version))
                .collect::<Vec<_>>(),
            ["bueno bueno@mk None", "data datos@synced Some(\"1.0.0\")"]
        );
        assert_eq!(
            l.ilegibles
                .iter()
                .map(|i| format!("{} {}", i.id, i.motivo))
                .collect::<Vec<_>>(),
            [
                "falta@synced sin-carpeta",
                "synced/cuenta-rota json-invalido"
            ]
        );
        m.escribir("claude/settings.json", r#"{"syncClaudeAiPlugins": false}"#);
        let l = leer(Some(&claude), &m.0.join("p"));
        assert!(l.inventario.plugin("data").is_none());
        assert!(l.ilegibles.is_empty(), "{:?}", l.ilegibles);
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

    fn incidencias(v: &[Incidencia]) -> Vec<String> {
        v.iter().map(|i| format!("{} {}", i.id, i.motivo)).collect()
    }

    /// H1 (ronda 1): una entrada de plugin que no es lista (objeto, `null`,
    /// numero) queda como ilegible `formato-desconocido`, no desaparece; la
    /// de un plugin apagado no cuenta; un fichero sin `version: 2` no se
    /// interpreta.
    #[test]
    fn entrada_de_plugin_que_no_es_lista() {
        let m = Tmp::nueva("h1");
        let claude = m.0.join("claude");
        let bueno = m.0.join("cache").join("bueno");
        m.escribir("cache/bueno/skills/rev/SKILL.md", &skill("rev"));
        let plugins = serde_json::json!({
            "bueno@mk": [{"scope": "user", "installPath": bueno}],
            "objeto@mk": {"scope": "user", "installPath": bueno},
            "nulo@mk": null,
            "numero@mk": 3,
            "apagado@mk": {"installPath": bueno}
        });
        m.escribir(
            "claude/settings.json",
            r#"{"enabledPlugins": {"apagado@mk": false}}"#,
        );
        let escribir = |version: Option<u64>| {
            let mut v = serde_json::json!({"plugins": plugins});
            if let Some(n) = version {
                v["version"] = n.into();
            }
            m.escribir("claude/plugins/installed_plugins.json", &v.to_string());
        };
        escribir(Some(2));
        let l = leer(Some(&claude), &m.0.join("p"));
        assert_eq!(
            incidencias(&l.ilegibles),
            [
                "nulo@mk formato-desconocido",
                "numero@mk formato-desconocido",
                "objeto@mk formato-desconocido",
            ]
        );
        assert!(l.inventario.contiene(Tipo::Skill, "bueno:rev"));
        for version in [None, Some(1), Some(3)] {
            escribir(version);
            let l = leer(Some(&claude), &m.0.join("p"));
            assert_eq!(
                incidencias(&l.ilegibles),
                ["plugins formato-desconocido"],
                "{version:?}"
            );
            assert!(l.inventario.plugins().is_empty());
        }
    }

    /// H2 (ronda 1): una carpeta `skills` o `agents` que no se puede listar
    /// (aqui, un fichero en su lugar: `read_dir` falla sin ser `NotFound`,
    /// como sin permiso) es una incidencia `no-se-lee`, no una carpeta vacia.
    #[test]
    fn carpeta_que_no_se_lista_es_ilegible() {
        let m = Tmp::nueva("h2");
        let claude = m.0.join("claude");
        m.escribir("claude/skills", "no soy una carpeta");
        m.escribir("claude/agents/ag.md", "x");
        m.escribir("p/.claude/agents", "tampoco");
        let l = leer(Some(&claude), &m.0.join("p"));
        assert_eq!(
            l.ilegibles
                .iter()
                .map(|i| format!("{} {} {}", i.id, i.motivo, i.ruta))
                .collect::<Vec<_>>(),
            [
                format!(
                    "agents no-se-lee {}",
                    m.0.join("p").join(".claude").join("agents").display()
                ),
                format!("skills no-se-lee {}", claude.join("skills").display()),
            ]
        );
        assert!(l.inventario.contiene(Tipo::Agente, "ag"));
        // Una carpeta que no existe sigue sin ser incidencia.
        let l = leer(Some(&m.0.join("no-existe")), &m.0.join("no-existe"));
        assert_eq!(l.ilegibles, []);
    }

    /// H2 (ronda 1): un nombre de carpeta o fichero que no es UTF-8 es una
    /// incidencia `nombre-no-utf8`, no un descarte silencioso.
    #[test]
    fn nombre_no_utf8_es_incidencia() {
        let m = Tmp::nueva("h2-utf8");
        let claude = m.0.join("claude");
        #[cfg(windows)]
        let raro = {
            use std::os::windows::ffi::OsStringExt;
            // `a` y un sustituto suelto: valido en NTFS, no en UTF-8.
            std::ffi::OsString::from_wide(&[0x61, 0xD800])
        };
        #[cfg(unix)]
        let raro = {
            use std::os::unix::ffi::OsStringExt;
            std::ffi::OsString::from_vec(vec![b'a', 0xFF])
        };
        let skills = claude.join("skills");
        let d = skills.join(&raro);
        std::fs::create_dir_all(&d).unwrap_or_default();
        std::fs::write(d.join("SKILL.md"), skill("raro")).unwrap_or_default();
        let mut agente = raro.clone();
        agente.push(".md");
        std::fs::create_dir_all(claude.join("agents")).unwrap_or_default();
        std::fs::write(claude.join("agents").join(agente), "x").unwrap_or_default();
        let l = leer(Some(&claude), &m.0.join("p"));
        assert_eq!(
            l.ilegibles.iter().map(|i| i.motivo).collect::<Vec<_>>(),
            ["nombre-no-utf8", "nombre-no-utf8"],
            "{:?}",
            l.ilegibles
        );
        assert!(l.ilegibles.iter().all(|i| i.id.starts_with('a')));
    }

    /// H3 (ronda 1): un `name` del manifiesto sincronizado vacio, con
    /// separadores, `:` o `..` no se une a la ruta (no se lee fuera de
    /// `synced/`): queda como `nombre-invalido`; un `installPath` relativo
    /// queda como `install-path-relativo` y no se resuelve contra el cwd.
    #[test]
    fn nombres_y_rutas_que_saldrian_de_su_carpeta() {
        let m = Tmp::nueva("h3");
        let claude = m.0.join("claude");
        // Lo que se leeria si se uniera `../../fuera` a la cuenta.
        m.escribir("claude/plugins/fuera/skills/malo/SKILL.md", &skill("malo"));
        m.escribir(
            "claude/plugins/synced/c/bien/skills/s/SKILL.md",
            &skill("s"),
        );
        m.escribir(
            "claude/plugins/synced/c/manifest.json",
            &serde_json::json!({"lastUpdated": 1, "plugins": [
                {"name": "../../fuera"}, {"name": "a/b"}, {"name": "a\\b"},
                {"name": ".."}, {"name": ""}, {"name": "C:x"}, {"version": "1"},
                {"name": "bien"}
            ]})
            .to_string(),
        );
        m.escribir(
            "claude/plugins/installed_plugins.json",
            &serde_json::json!({"version": 2, "plugins": {
                "rel@mk": [{"scope": "user", "installPath": "relativa/rel"}]
            }})
            .to_string(),
        );
        let l = leer(Some(&claude), &m.0.join("p"));
        assert_eq!(
            lineas(&l),
            ["skill bien:s plugin", "agente general-purpose integrado"]
        );
        assert_eq!(
            incidencias(&l.ilegibles),
            [
                "../../fuera@synced nombre-invalido",
                "..@synced nombre-invalido",
                // Vacio y sin `name`: la misma incidencia, una vez.
                "@synced nombre-invalido",
                "C:x@synced nombre-invalido",
                "a/b@synced nombre-invalido",
                "a\\b@synced nombre-invalido",
                "rel@mk install-path-relativo",
            ]
        );
    }

    /// H4 (ronda 1): `lastUpdated` se compara por valor (ms desde la
    /// epoca), venga como numero o como texto ISO; un valor invalido no
    /// gana a uno valido.
    #[test]
    fn la_fecha_se_compara_por_valor() {
        use serde_json::json;
        let f = |v: Value| orden_de_fecha(Some(&v));
        assert_eq!(f(json!("1970-01-01T00:00:00Z")), Some(0));
        assert_eq!(f(json!("2026-01-01T00:00:00Z")), Some(1_767_225_600_000));
        assert_eq!(f(json!("2026-01-01")), Some(1_767_225_600_000));
        assert_eq!(
            f(json!("2026-01-01T01:00:00.250+01:00")),
            Some(1_767_225_600_250)
        );
        assert_eq!(f(json!(1_767_225_600_000_u64)), Some(1_767_225_600_000));
        assert_eq!(f(json!(1.5e12)), Some(1_500_000_000_000));
        for malo in [
            json!(-5),
            json!(-1.0),
            json!(1e300),
            json!("no es fecha"),
            json!("2026-13-01T00:00:00Z"),
            json!("2026-01-01T25:00:00Z"),
            json!("~~~~"),
            json!(null),
            json!(true),
        ] {
            assert_eq!(f(malo.clone()), None, "{malo}");
        }
        assert_eq!(orden_de_fecha(None), None);
        // Numero mas reciente contra texto ISO mas antiguo (y contra uno
        // invalido que, por texto, ordenaria despues): gana el numero.
        let m = Tmp::nueva("h4");
        let claude = m.0.join("claude");
        let s = "claude/plugins/synced";
        for (cuenta, fecha, plugin) in [
            ("a-numero", json!(1_790_000_000_000_u64), "nuevo"),
            ("b-iso", json!("2026-01-01T00:00:00Z"), "viejo"),
            ("c-invalida", json!("~ no es fecha"), "roto"),
            ("d-negativa", json!(-1), "negativo"),
        ] {
            m.escribir(
                &format!("{s}/{cuenta}/manifest.json"),
                &json!({"lastUpdated": fecha, "plugins": [{"name": plugin}]}).to_string(),
            );
            m.escribir(
                &format!("{s}/{cuenta}/{plugin}/skills/x/SKILL.md"),
                &skill("x"),
            );
        }
        let l = leer(Some(&claude), &m.0.join("p"));
        assert_eq!(
            l.inventario
                .plugins()
                .iter()
                .map(|p| p.nombre.as_str())
                .collect::<Vec<_>>(),
            ["nuevo"]
        );
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
