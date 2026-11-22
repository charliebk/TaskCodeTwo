//! Catalogo de seleccion (F1-15 decisiones 2 y 8): los metadatos que cada
//! skill declara en su bloque ```` ```yaml ```` (`rol` ∈ {`revisor`,
//! `diseno`, `ejecucion`}, `etiquetas`, `dominios`, `prioridad`; en las
//! revisoras `patrones_archivo`, y en la generica `fallback` y
//! `umbral_dominios`) y, para las ajenas, los que declara `recursos:` de la
//! configuracion ([`de_config`], F4-6).
//!
//! Se apoya en [`crate::inventario`] sin cambiarlo: el inventario dice que
//! existe; el catalogo lee, de las mismas carpetas, que declara cada skill.
//! Solo se lee la cabecera de cada `SKILL.md` ([`MAX_CABECERA`]): un bloque
//! que cae mas alla no cuenta (la skill queda sin metadatos y no se elige).
//!
//! Fuentes:
//! - [`de_carpeta`]: `<dir>/<n>/SKILL.md` de un plugin dado por ruta (el
//!   revisor en CI, con el plugin a una version fijada) o de una carpeta de
//!   skills;
//! - [`de_instalados`]: los plugins habilitados del inventario, por su
//!   `installPath` de `installed_plugins.json`;
//! - [`de_base`]: `.claude/skills` del proyecto **en la base**
//!   (`origin/develop`), nunca del worktree del PR (decision 2).

use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::contrato::Informe;
use crate::git::{Git, Subcomando};
use crate::inventario::{self, Entrada, Inventario, MAX_CABECERA, Origen, Tipo};

/// Roles que puede declarar una entrada del catalogo.
pub const ROLES: [&str; 3] = ["revisor", "diseno", "ejecucion"];

/// Una entrada del catalogo (una revisora o una skill de diseno o de
/// ejecucion).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Revisora {
    /// Id del inventario (`plugin:n` en un plugin).
    pub id: String,
    /// `revisor`, `diseno` o `ejecucion`.
    pub rol: String,
    /// Etiquetas (solape de S07).
    pub etiquetas: Vec<String>,
    /// Dominios de F1-15 que cubre (S06, S07, S10).
    pub dominios: Vec<String>,
    /// Prioridad (desempate de S07; mayor primero).
    pub prioridad: u32,
    /// Globs de `patrones_archivo`.
    pub patrones: Vec<String>,
    /// `fallback: true` (la generica): no compite por patrones.
    pub fallback: bool,
    /// `umbral_dominios` de la generica.
    pub umbral_dominios: Option<u32>,
    /// De donde se leyo (`SKILL.md` en disco o `<base>:<ruta>`).
    pub ruta: String,
}

/// Las entradas del catalogo, ordenadas por id y sin repetir: las
/// revisoras y las demas.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Catalogo {
    revisoras: Vec<Revisora>,
    otros: Vec<Revisora>,
}

impl Catalogo {
    /// Catalogo de `entradas` en cualquier orden: el resultado no depende
    /// del orden (se queda la primera de cada id tras ordenar).
    pub fn nuevo(mut entradas: Vec<Revisora>) -> Self {
        entradas.sort();
        entradas.dedup_by(|b, a| a.id == b.id);
        let (revisoras, otros) = entradas.into_iter().partition(|r| r.rol == "revisor");
        Self { revisoras, otros }
    }

    /// Las revisoras, por id.
    pub fn revisoras(&self) -> &[Revisora] {
        &self.revisoras
    }

    /// Las skills de diseno y de ejecucion, por id.
    pub fn otros(&self) -> &[Revisora] {
        &self.otros
    }

    /// Huella del catalogo (12 hex del sha256 de su forma canonica, como
    /// [`Inventario::huella`]): lo que cada entrada declara, sin la `ruta`
    /// de la que se leyo (la misma skill en otra maquina da la misma huella).
    pub fn huella(&self) -> String {
        let canonica: Vec<_> = self
            .revisoras
            .iter()
            .chain(&self.otros)
            .map(|r| {
                (
                    &r.id,
                    &r.rol,
                    &r.etiquetas,
                    &r.dominios,
                    r.prioridad,
                    &r.patrones,
                    r.fallback,
                    r.umbral_dominios,
                )
            })
            .collect();
        let s = format!("{canonica:?}");
        crate::contrato::corto(&distribucion::sha256::hex(s.as_bytes()))
    }

    /// La revisora de id `id`.
    pub fn revisora(&self, id: &str) -> Option<&Revisora> {
        self.revisoras.iter().find(|r| r.id == id)
    }
}

/// El bloque yaml de una skill.
#[derive(Debug, Deserialize)]
struct Bloque {
    rol: Option<String>,
    #[serde(default)]
    patrones_archivo: Vec<String>,
    #[serde(default)]
    fallback: bool,
    umbral_dominios: Option<u32>,
    #[serde(default)]
    etiquetas: Vec<String>,
    #[serde(default)]
    dominios: Vec<String>,
    #[serde(default)]
    prioridad: u32,
}

/// El primer bloque ```` ```yaml ```` de `cabecera` (el texto entre la
/// valla de apertura y la de cierre), o `None` si no hay o no se cierra.
pub fn bloque_yaml(cabecera: &str) -> Option<&str> {
    let mut desde = None;
    let mut pos = 0;
    for l in cabecera.split_inclusive('\n') {
        let t = l.trim();
        match desde {
            None if t == "```yaml" => desde = Some(pos + l.len()),
            Some(d) if t == "```" => return cabecera.get(d..pos),
            _ => {}
        }
        pos += l.len();
    }
    None
}

/// La revisora que declara el texto de una skill: `None` si no tiene bloque
/// yaml, no es `rol: revisor` o el bloque no se entiende (lo instalado sin
/// metadatos no se elige, decision 2).
pub fn revisora_de(id: &str, texto: &str, ruta: &str) -> Option<Revisora> {
    recurso_de(id, texto, ruta).filter(|r| r.rol == "revisor")
}

/// La entrada que declara el texto de una skill, con cualquiera de los
/// [`ROLES`]: `None` si no tiene bloque yaml, su `rol` no es de la lista o
/// el bloque no se entiende.
pub fn recurso_de(id: &str, texto: &str, ruta: &str) -> Option<Revisora> {
    let mut corte = texto
        .len()
        .min(usize::try_from(MAX_CABECERA).unwrap_or(usize::MAX));
    while !texto.is_char_boundary(corte) {
        corte -= 1;
    }
    let cabecera = texto.get(..corte).unwrap_or_default();
    let b: Bloque =
        serde_saphyr::from_str_with_options(bloque_yaml(cabecera)?, crate::yaml::opciones())
            .ok()?;
    let rol = b.rol.filter(|r| ROLES.contains(&r.as_str()))?;
    Some(Revisora {
        id: id.to_owned(),
        rol,
        etiquetas: b.etiquetas,
        dominios: b.dominios,
        prioridad: b.prioridad,
        patrones: b.patrones_archivo,
        fallback: b.fallback,
        umbral_dominios: b.umbral_dominios,
        ruta: ruta.to_owned(),
    })
}

/// Lo leido de una fuente: las entradas que existen y las que declaran
/// metadatos de seleccion.
#[derive(Debug, Clone, Default)]
pub struct Lectura {
    /// Skills encontradas (para el inventario).
    pub entradas: Vec<Entrada>,
    /// Entradas del catalogo (revisoras y skills de diseno o ejecucion).
    pub revisoras: Vec<Revisora>,
}

/// Las entradas que declara `recursos:` de la configuracion (F1-14, F4-6):
/// solo las que dicen un `rol` de [`ROLES`]; las demas no se eligen.
pub fn de_config(recursos: &[crate::config::Recurso]) -> Vec<Revisora> {
    recursos
        .iter()
        .filter_map(|r| {
            let rol = r.rol.clone().filter(|x| ROLES.contains(&x.as_str()))?;
            Some(Revisora {
                id: r.id.clone(),
                rol,
                etiquetas: r.etiquetas.clone(),
                dominios: r.dominios.clone(),
                prioridad: r.prioridad.unwrap_or(0),
                patrones: Vec::new(),
                fallback: false,
                umbral_dominios: None,
                ruta: ".taskcode/config.yml".to_owned(),
            })
        })
        .collect()
}

/// Lee como mucho [`MAX_CABECERA`] bytes de `p`.
fn cabecera(p: &Path) -> Option<String> {
    let mut b = Vec::new();
    std::fs::File::open(p)
        .ok()?
        .take(MAX_CABECERA)
        .read_to_end(&mut b)
        .ok()?;
    Some(String::from_utf8_lossy(&b).into_owned())
}

/// Las skills de `skills` (`<n>/SKILL.md`) con el `prefijo` de ids
/// (`plugin:` o vacio) y su `origen`.
pub fn de_carpeta(skills: &Path, prefijo: &str, origen: &Origen) -> Lectura {
    let mut l = Lectura::default();
    let mut hijos: Vec<PathBuf> = std::fs::read_dir(skills)
        .map(|d| d.filter_map(Result::ok).map(|e| e.path()).collect())
        .unwrap_or_default();
    hijos.sort();
    for d in hijos {
        let f = d.join("SKILL.md");
        let (Some(carpeta), Some(texto)) = (
            d.file_name().and_then(|n| n.to_str()).map(str::to_owned),
            f.is_file().then(|| cabecera(&f)).flatten(),
        ) else {
            continue;
        };
        let nombre = inventario::nombre_del_frontmatter(&texto).unwrap_or(carpeta);
        let id = format!("{prefijo}{nombre}");
        l.entradas.push(Entrada {
            tipo: Tipo::Skill,
            id: id.clone(),
            origen: origen.clone(),
        });
        if let Some(r) = recurso_de(&id, &texto, &f.display().to_string()) {
            l.revisoras.push(r);
        }
    }
    l
}

/// Las revisoras de los plugins habilitados de `inv`, leidas de su
/// `installPath` en `installed_plugins.json` de `dir_claude` (la instalacion
/// con la version del inventario o, si no la dice, la primera con ruta
/// absoluta que existe). Los sincronizados de claude.ai no se leen (no traen
/// revisoras con metadatos).
pub fn de_instalados(dir_claude: &Path, inv: &Inventario) -> Vec<Revisora> {
    let Ok(Some(Ok(v))) = inventario::leer_instalados(dir_claude) else {
        return Vec::new();
    };
    let mut salida = Vec::new();
    for p in inv.plugins() {
        let Some(lista) = v
            .get("plugins")
            .and_then(|m| m.get(&p.clave))
            .and_then(Value::as_array)
        else {
            continue;
        };
        let ruta = |x: &Value| {
            x.get("installPath")
                .and_then(Value::as_str)
                .map(PathBuf::from)
                .filter(|d| d.is_absolute() && d.is_dir())
        };
        let misma = lista.iter().find(|x| {
            p.version.is_some() && x.get("version").and_then(Value::as_str) == p.version.as_deref()
        });
        let Some(dir) = misma.and_then(ruta).or_else(|| lista.iter().find_map(ruta)) else {
            continue;
        };
        let origen = Origen::Plugin {
            nombre: p.nombre.clone(),
            version: p.version.clone(),
        };
        salida
            .extend(de_carpeta(&dir.join("skills"), &format!("{}:", p.nombre), &origen).revisoras);
    }
    salida
}

/// Las skills de `.claude/skills` del proyecto en `base` (un commit o una
/// ref, en produccion `origin/develop`), leidas con git: lo que el PR anada
/// en su arbol no cuenta (decision 2).
pub fn de_base(git: &Git, dir: &Path, base: &str) -> Result<Lectura, Informe> {
    const OP: &str = "revisoras";
    let listado = crate::git::exigir(
        git,
        dir,
        OP,
        Subcomando::LsTree,
        &[
            "-r",
            "--full-tree",
            "--name-only",
            "--end-of-options",
            base,
            "--",
            ".claude/skills",
        ],
    )?;
    let rutas: Vec<&str> = listado
        .lines()
        .filter(|r| {
            r.strip_prefix(".claude/skills/")
                .and_then(|r| r.strip_suffix("/SKILL.md"))
                .is_some_and(|n| !n.is_empty() && !n.contains('/'))
        })
        .collect();
    let contenidos = crate::git::estado::leer_blobs(git, dir, OP, base, &rutas)?;
    let mut l = Lectura::default();
    for (ruta, texto) in rutas.iter().zip(contenidos) {
        let Some(texto) = texto else { continue };
        let carpeta = ruta
            .trim_start_matches(".claude/skills/")
            .trim_end_matches("/SKILL.md");
        let nombre =
            inventario::nombre_del_frontmatter(&texto).unwrap_or_else(|| carpeta.to_owned());
        l.entradas.push(Entrada {
            tipo: Tipo::Skill,
            id: nombre.clone(),
            origen: Origen::Proyecto,
        });
        if let Some(r) = recurso_de(&nombre, &texto, &format!("{base}:{ruta}")) {
            l.revisoras.push(r);
        }
    }
    Ok(l)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_bloque_yaml_se_lee_y_lo_demas_no_es_revisora() {
        let t = "---\nname: x\n---\n\ntexto\n\n```yaml\nrol: revisor\npatrones_archivo: [\"**/*.rs\"]   # nota\n```\n\nmas\n";
        let r = revisora_de("p:x", t, "x");
        assert_eq!(r.map(|r| r.patrones), Some(vec!["**/*.rs".to_owned()]));
        // Sin bloque, sin cierre, otro rol o yaml roto: no es revisora.
        assert!(revisora_de("p:x", "---\nname: x\n---\n", "x").is_none());
        assert!(revisora_de("p:x", "```yaml\nrol: revisor\n", "x").is_none());
        assert!(revisora_de("p:x", "```yaml\nrol: ejecucion\n```\n", "x").is_none());
        // F4-6: una skill de ejecucion entra en el catalogo con sus metadatos.
        let e = recurso_de(
            "p:x",
            "```yaml\nrol: ejecucion\netiquetas: [a, b]\ndominios: [rendimiento]\nprioridad: 10\n```\n",
            "x",
        );
        assert_eq!(
            e.map(|e| (e.rol, e.etiquetas, e.dominios, e.prioridad)),
            Some((
                "ejecucion".to_owned(),
                vec!["a".to_owned(), "b".to_owned()],
                vec!["rendimiento".to_owned()],
                10
            ))
        );
        assert!(recurso_de("p:x", "```yaml\nrol: otro\n```\n", "x").is_none());
        assert!(revisora_de("p:x", "```yaml\nrol: [\n```\n", "x").is_none());
        // Un bloque mas alla de la cabecera no cuenta.
        let lejos = format!(
            "{}\n```yaml\nrol: revisor\n```\n",
            "a".repeat(MAX_CABECERA as usize)
        );
        assert!(revisora_de("p:x", &lejos, "x").is_none());
    }

    #[test]
    fn el_catalogo_no_depende_del_orden() {
        let r = |id: &str| Revisora {
            id: id.to_owned(),
            rol: "revisor".to_owned(),
            etiquetas: Vec::new(),
            dominios: Vec::new(),
            prioridad: 0,
            patrones: Vec::new(),
            fallback: false,
            umbral_dominios: None,
            ruta: String::new(),
        };
        let a = Catalogo::nuevo(vec![r("b"), r("a"), r("b")]);
        let b = Catalogo::nuevo(vec![r("a"), r("b")]);
        assert_eq!(a, b);
        assert_eq!(a.revisoras().len(), 2);
        // La huella no depende de la ruta de lectura, si de lo declarado.
        let en = |ruta: &str| Revisora {
            ruta: ruta.to_owned(),
            ..r("x")
        };
        let huella = |r: Revisora| Catalogo::nuevo(vec![r]).huella();
        assert_eq!(
            huella(en("C:/u1/.claude/x/SKILL.md")),
            huella(en("/home/u2/.claude/x/SKILL.md"))
        );
        assert_ne!(
            huella(en("x")),
            huella(Revisora {
                prioridad: 1,
                ..en("x")
            })
        );
    }
}
