//! Seleccion determinista de agentes, skills y modelo por rol (F6-3 y F4-6;
//! ADR F1-15 decision 1).
//!
//! [`seleccionar`] es una funcion pura: recibe el rol (diseno,
//! implementacion o revision), las rutas del cambio (los dos nombres de cada
//! renombrado), los dominios detectados ([`dominios`]), las etiquetas, el
//! numero de roles de diseno por carril y complejidad, el catalogo y el
//! inventario, el carril efectivo, la complejidad y los modelos permitidos,
//! y devuelve agentes, skills (y revisoras), modelo, filas aplicadas,
//! senales y avisos. No lee ficheros, no llama a la red ni a un modelo:
//! recorre las filas de su rol y de `todos` de `tabla-seleccion` en su orden
//! ([`tabla`]) y la columna de su rol de `tabla-modelo`. [`revision`] es la
//! envoltura de F6-3 (sin dominios). La ronda (F6-6) la da `--ronda` de
//! `tc2 revisoras` (en CI, la de `tc2 hallazgos ronda`); sin ella es 1.

pub mod catalogo;
pub mod dominios;
pub mod orden;
pub mod tabla;

use std::collections::BTreeMap;

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

use crate::carril::Carril;
use crate::inventario::{Inventario, Tipo};

pub use catalogo::{Catalogo, Revisora};
use tabla::{FilaSeleccion, RolFila};

/// Umbral de dominios si la generica no lo declara (el de la 0.16).
pub const UMBRAL_POR_DEFECTO: u32 = 3;

/// Lo que necesita la seleccion ademas de las rutas, el catalogo y el
/// inventario.
#[derive(Debug, Clone)]
pub struct Contexto<'a> {
    /// Carril efectivo (F1-10).
    pub carril: Carril,
    /// Complejidad: la mayor entre la declarada y la heuristica; `None` si
    /// no se sabe (se toma `media`, con aviso).
    pub complejidad: Option<&'a str>,
    /// `seleccion.modelos_permitidos` de la configuracion de la base.
    pub modelos_permitidos: &'a [String],
    /// Ronda de revision (1 hasta F6-6).
    pub ronda: u32,
}

/// Rol para el que se selecciona.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rol {
    /// Diseno (`plan`).
    Diseno,
    /// Implementacion (`plan`).
    Implementacion,
    /// Revision (`review` y el revisor en CI).
    Revision,
}

impl Rol {
    /// Nombre en la tabla y en `roles.<rol>`.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Diseno => "diseno",
            Self::Implementacion => "implementacion",
            Self::Revision => "revision",
        }
    }

    fn fila(self) -> RolFila {
        match self {
            Self::Diseno => RolFila::Diseno,
            Self::Implementacion => RolFila::Implementacion,
            Self::Revision => RolFila::Revision,
        }
    }
}

/// Lo que se sabe de la tarea ademas del carril y la complejidad.
#[derive(Debug, Clone, Copy)]
pub struct Entrada<'a> {
    /// Rutas del cambio (en `plan`, vacias).
    pub rutas: &'a [String],
    /// Dominios detectados ([`dominios::dominios`], con la union de lo
    /// congelado si lo hay).
    pub dominios: &'a dominios::Dominios,
    /// Etiquetas de la tarea (solape de S07).
    pub etiquetas: &'a [String],
    /// Roles de diseno por carril y complejidad
    /// ([`Carril::roles_diseno`]): el numero de S02.
    pub roles_base: u8,
}

/// La seleccion de un rol.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Seleccion {
    /// Revisoras (skills), por id (revision).
    pub revisoras: Vec<String>,
    /// Agentes: los que lanzan cada revisora (S12) o los de diseno (S02 a
    /// S05), sin los no instalados (S14).
    pub agentes: Vec<String>,
    /// Agentes de diseno elegidos por las filas, en su orden y aunque no
    /// esten instalados (la peticion se escribe igual, con el aviso).
    pub roles: Vec<String>,
    /// Skills de diseno (S06) o de ejecucion (S07).
    pub skills: Vec<String>,
    /// Modelo de la escala (`haiku`, `sonnet` u `opus`); vacio sin plan.
    pub modelo: String,
    /// Ids de las filas que se aplicaron (la de `tabla-modelo` primero).
    pub reglas: Vec<String>,
    /// Avisos (`no-instalado:<id>`, `complejidad-desconocida`...).
    pub avisos: Vec<String>,
    /// Senales de dominio (`<dominio>:<senal>:<valor>`).
    pub senales: Vec<String>,
    /// Rutas que recibe cada revisora (la de dominio, las que casan con sus
    /// patrones; la generica, las sueltas o todas).
    pub rutas: BTreeMap<String, Vec<String>>,
}

/// Id del modelo en la API de Anthropic para un modelo de la escala.
pub fn modelo_api(modelo: &str) -> &'static str {
    match modelo {
        "haiku" => "claude-haiku-5-5",
        "opus" => "claude-opus-5-5",
        _ => "claude-sonnet-5-5",
    }
}

/// Los patrones de una revisora como un conjunto de globs: sin distinguir
/// mayusculas (F1-15 decision 3) y con `*` sin cruzar `/`. Un patron que no
/// es un glob valido no casa con nada.
fn globs(patrones: &[String]) -> GlobSet {
    let mut b = GlobSetBuilder::new();
    for p in patrones {
        if let Ok(g) = GlobBuilder::new(p)
            .case_insensitive(true)
            .literal_separator(true)
            .build()
        {
            b.add(g);
        }
    }
    b.build().unwrap_or_else(|_| GlobSet::empty())
}

/// Ruta con `/` y sin `./` delante.
fn normalizar(ruta: &str) -> String {
    let r = ruta.trim().replace('\\', "/");
    r.strip_prefix("./").map_or(r.clone(), str::to_owned)
}

/// Lo que se calcula de las rutas antes de recorrer la tabla.
struct Casan {
    /// Revisora de dominio -> rutas que casan.
    por_revisora: BTreeMap<String, Vec<String>>,
    /// Rutas que no casan con ninguna.
    sueltas: Vec<String>,
}

fn casar(rutas: &[String], cat: &Catalogo) -> Casan {
    let mut por_revisora: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut sueltas = Vec::new();
    let conjuntos: Vec<(&str, GlobSet)> = cat
        .revisoras()
        .iter()
        .filter(|r| !r.fallback && !r.patrones.is_empty())
        .map(|r| (r.id.as_str(), globs(&r.patrones)))
        .collect();
    for ruta in rutas {
        let mut alguna = false;
        for (id, g) in &conjuntos {
            if g.is_match(ruta) {
                alguna = true;
                por_revisora
                    .entry((*id).to_owned())
                    .or_default()
                    .push(ruta.clone());
            }
        }
        if !alguna {
            sueltas.push(ruta.clone());
        }
    }
    Casan {
        por_revisora,
        sueltas,
    }
}

/// Modelo de `rol` en `tabla-modelo` para el carril y la complejidad
/// (vacio en diseno de `errata`, «—»).
fn fila_modelo(
    filas: &[tabla::FilaModelo],
    rol: Rol,
    carril: Carril,
    cx: &str,
) -> Option<(String, String)> {
    filas
        .iter()
        .find(|f| {
            f.carril == carril.nombre()
                && (f.complejidades.is_empty() || f.complejidades.iter().any(|x| x == cx))
        })
        .map(|f| {
            let m = match rol {
                Rol::Diseno => f.diseno.clone().unwrap_or_default(),
                Rol::Implementacion => f.implementacion.clone(),
                Rol::Revision => f.revision.clone(),
            };
            (f.id.clone(), m)
        })
}

/// El permitido mas cercano por debajo de `modelo`; si no hay, por encima.
fn restringir(modelo: &str, permitidos: &[String]) -> Option<String> {
    let i = tabla::ESCALA.iter().position(|m| *m == modelo)?;
    let ok = |j: usize| {
        tabla::ESCALA
            .get(j)
            .filter(|m| permitidos.iter().any(|p| p == *m))
            .map(|m| (*m).to_owned())
    };
    (0..i)
        .rev()
        .find_map(ok)
        .or_else(|| (i + 1..tabla::ESCALA.len()).find_map(ok))
}

/// Estado mientras se recorren las filas.
struct Estado<'a> {
    sel: Seleccion,
    casan: Casan,
    todas: Vec<String>,
    umbral: u32,
    generica: Option<String>,
    inv: &'a Inventario,
    cat: &'a Catalogo,
    ctx: &'a Contexto<'a>,
    entrada: &'a Entrada<'a>,
    /// Dominio que se evalua en una fila por dominio (S10, S11).
    dom: Option<String>,
    /// Diseno: roles de la escalera (S02, S03), ids de la escalera (los de
    /// S02), roles anadidos (S04, S05) y si hay plan (S01).
    n: usize,
    escalera: Vec<String>,
    extra: Vec<String>,
    plan: bool,
}

impl Estado<'_> {
    fn instalado(&self, id: &str) -> bool {
        self.inv.contiene(Tipo::Skill, id) || self.inv.contiene(Tipo::Agente, id)
    }

    fn hay_dominio(&self, d: &str) -> bool {
        self.entrada.dominios.iter().any(|(x, _)| x == d)
    }

    /// Revisoras declaradas e instaladas para el dominio `d`, por id (S10 y
    /// S11: una declarada y no instalada cuenta como «sin revisora»).
    fn revisoras_de(&self, d: &str) -> Vec<String> {
        self.cat
            .revisoras()
            .iter()
            .filter(|r| r.dominios.iter().any(|x| x == d) && self.instalado(&r.id))
            .map(|r| r.id.clone())
            .collect()
    }

    /// Skills `rol: diseno` de algun dominio detectado, por id.
    fn skills_diseno(&self) -> Vec<String> {
        self.cat
            .otros()
            .iter()
            .filter(|r| r.rol == "diseno" && r.dominios.iter().any(|d| self.hay_dominio(d)))
            .map(|r| r.id.clone())
            .collect()
    }

    /// Candidatas de ejecucion con solape de etiquetas o dominios, en orden:
    /// mayor solape, mayor prioridad, menor id.
    fn ejecucion(&self) -> Vec<(usize, u32, String)> {
        let etq: Vec<String> = self
            .entrada
            .etiquetas
            .iter()
            .map(|e| crate::tarea::texto::plegar(e))
            .collect();
        let mut v: Vec<(usize, u32, String)> = self
            .cat
            .otros()
            .iter()
            .filter(|r| r.rol == "ejecucion")
            .filter_map(|r| {
                let s = r
                    .etiquetas
                    .iter()
                    .filter(|e| etq.contains(&crate::tarea::texto::plegar(e)))
                    .count()
                    + r.dominios.iter().filter(|d| self.hay_dominio(d)).count();
                (s > 0).then(|| (s, r.prioridad, r.id.clone()))
            })
            .collect();
        v.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
        v
    }

    /// Roles de diseno elegidos hasta ahora (escalera y anadidos).
    fn roles_diseno(&self) -> Vec<String> {
        if !self.plan {
            return Vec::new();
        }
        self.escalera
            .iter()
            .take(self.n)
            .chain(&self.extra)
            .cloned()
            .collect()
    }

    /// Diseno: los agentes de los roles elegidos pasan a la seleccion (antes
    /// de las filas de `todos`, que miran lo instalado).
    fn materializar(&mut self) {
        if self.sel.roles.is_empty() {
            self.sel.roles = self.roles_diseno();
            self.sel.agentes.clone_from(&self.sel.roles);
        }
    }

    fn atomo(&self, texto: &str) -> bool {
        let n = u32::try_from(self.casan.por_revisora.len()).unwrap_or(u32::MAX);
        let elegidos = || {
            self.sel
                .revisoras
                .iter()
                .chain(&self.sel.agentes)
                .chain(&self.sel.skills)
        };
        match texto {
            "siempre" => true,
            "rutas-casan-entre-1-y-umbral" => (1..=self.umbral).contains(&n),
            "rutas-no-casan-o-pasan-umbral" => n == 0 || n > self.umbral,
            "hay-no-instalado" => elegidos().any(|id| !self.instalado(id)),
            "modelo-no-permitido" => !self.ctx.modelos_permitidos.contains(&self.sel.modelo),
            "dominio-con-revisora" => self
                .dom
                .as_deref()
                .is_some_and(|d| !self.revisoras_de(d).is_empty()),
            "dominio-sin-revisora" => self
                .dom
                .as_deref()
                .is_some_and(|d| self.revisoras_de(d).is_empty()),
            "hay-skill-diseno-del-dominio" => !self.skills_diseno().is_empty(),
            "hay-skill-ejecucion-con-solape" => !self.ejecucion().is_empty(),
            t => {
                if let Some(c) = t.strip_prefix("carril!=") {
                    c != self.ctx.carril.nombre()
                } else if let Some(c) = t.strip_prefix("carril=") {
                    c == self.ctx.carril.nombre()
                } else if let Some(r) = t.strip_prefix("ronda>=") {
                    r.parse().is_ok_and(|r: u32| self.ctx.ronda >= r)
                } else if let Some(d) = t.strip_prefix("dominio=") {
                    self.hay_dominio(d)
                } else if let Some(r) = t.strip_prefix("roles>=") {
                    r.parse()
                        .is_ok_and(|r: usize| self.roles_diseno().len() >= r)
                } else {
                    false
                }
            }
        }
    }

    /// Aplica la accion de `f`; `true` si cambio algo.
    fn aplicar(&mut self, f: &FilaSeleccion) -> bool {
        let antes = (self.sel.clone(), self.n, self.extra.clone(), self.plan);
        match f.accion.as_str() {
            "sin-plan" => self.plan = false,
            "roles=max(roles_min,complejidad)" => {
                f.ids.clone_into(&mut self.escalera);
                self.n = usize::from(self.entrada.roles_base);
                // La fila fija el numero aunque coincida con el de partida.
                return true;
            }
            "skills-diseno-del-dominio" => {
                let v = self.skills_diseno();
                self.sel.skills.extend(v);
            }
            "skill-ejecucion=solape,prioridad,id" => {
                let c = self.ejecucion();
                if let Some((s, p, id)) = c.first() {
                    let empate: Vec<&str> = c
                        .iter()
                        .filter(|x| x.0 == *s && x.1 == *p)
                        .map(|x| x.2.as_str())
                        .collect();
                    if empate.len() > 1 {
                        self.sel
                            .avisos
                            .push(format!("empate-por-id:{}", empate.join(",")));
                    }
                    self.sel.skills.push(id.clone());
                }
            }
            "revisora-del-dominio" => {
                if let Some(id) = self
                    .dom
                    .as_deref()
                    .and_then(|d| self.revisoras_de(d).into_iter().next())
                {
                    self.sel.revisoras.push(id.clone());
                    self.sel
                        .rutas
                        .entry(id)
                        .or_insert_with(|| self.todas.clone());
                }
            }
            "aviso=dominio-sin-revisora" => {
                if let Some(d) = &self.dom {
                    self.sel.avisos.push(format!("dominio-sin-revisora:{d}"));
                }
            }
            "revisoras-de-rutas+generica-si-sueltas" => {
                for (id, rutas) in &self.casan.por_revisora {
                    self.sel.revisoras.push(id.clone());
                    self.sel.rutas.insert(id.clone(), rutas.clone());
                }
                if !self.casan.sueltas.is_empty()
                    && let Some(g) = &self.generica
                {
                    self.sel.revisoras.push(g.clone());
                    self.sel.rutas.insert(g.clone(), self.casan.sueltas.clone());
                }
            }
            "generica-del-plugin" => {
                if let Some(g) = &self.generica {
                    self.sel.revisoras = vec![g.clone()];
                    self.sel.rutas = BTreeMap::from([(g.clone(), self.todas.clone())]);
                }
            }
            "lanzador" => self.sel.agentes.extend(f.ids.iter().cloned()),
            "quitar-no-instalado" => {
                let fuera: Vec<String> = self
                    .sel
                    .revisoras
                    .iter()
                    .chain(&self.sel.agentes)
                    .chain(&self.sel.skills)
                    .filter(|id| !self.instalado(id))
                    .cloned()
                    .collect();
                for id in &fuera {
                    self.sel.avisos.push(format!("no-instalado:{id}"));
                    self.sel.rutas.remove(id);
                }
                self.sel.revisoras.retain(|id| !fuera.contains(id));
                self.sel.agentes.retain(|id| !fuera.contains(id));
                self.sel.skills.retain(|id| !fuera.contains(id));
            }
            "restringir-modelo" => {
                if let Some(m) = restringir(&self.sel.modelo, self.ctx.modelos_permitidos) {
                    self.sel
                        .avisos
                        .push(format!("modelo-no-permitido:{}->{m}", self.sel.modelo));
                    self.sel.modelo = m;
                }
            }
            a => {
                if let Some(n) = a
                    .strip_prefix("suelo=")
                    .and_then(|n| n.parse::<usize>().ok())
                {
                    self.n = self.n.max(n);
                } else if let Some(r) = a.strip_prefix("mas-rol=") {
                    // El agente es el de la fila (columna «Ids del inventario»).
                    let id = f.ids.first().cloned().unwrap_or_else(|| r.to_owned());
                    if !self.extra.contains(&id) {
                        self.extra.push(id);
                    }
                } else if let Some(n) = a
                    .strip_prefix("modelo-baja=")
                    .and_then(|n| n.parse::<usize>().ok())
                {
                    let i = tabla::ESCALA
                        .iter()
                        .position(|m| *m == self.sel.modelo)
                        .unwrap_or(0);
                    if let Some(m) = tabla::ESCALA.get(i.saturating_sub(n)) {
                        (*m).clone_into(&mut self.sel.modelo);
                    }
                }
            }
        }
        self.sel.revisoras.sort();
        self.sel.revisoras.dedup();
        // Los agentes de diseno van en el orden de sus roles.
        if f.rol == RolFila::Revision {
            self.sel.agentes.sort();
            self.sel.agentes.dedup();
        }
        self.sel.skills.dedup();
        self.sel.avisos.dedup();
        (self.sel.clone(), self.n, self.extra.clone(), self.plan) != antes
    }
}

/// La seleccion de revision para las `rutas` del diff, sin dominios (la de
/// F6-3). `Err` si las tablas del ADR no se leen (no pasa con el ADR
/// compilado: lo prueba `las_tablas_del_adr_se_leen`).
pub fn revision(
    rutas: &[String],
    cat: &Catalogo,
    inv: &Inventario,
    ctx: &Contexto<'_>,
) -> Result<Seleccion, String> {
    revision_con(tabla::ADR, rutas, cat, inv, ctx)
}

/// Como [`revision`], con las tablas de `adr` (para probar variaciones).
pub fn revision_con(
    adr: &str,
    rutas: &[String],
    cat: &Catalogo,
    inv: &Inventario,
    ctx: &Contexto<'_>,
) -> Result<Seleccion, String> {
    let sin = dominios::Dominios::new();
    let e = Entrada {
        rutas,
        dominios: &sin,
        etiquetas: &[],
        roles_base: 0,
    };
    seleccionar_con(adr, Rol::Revision, &e, cat, inv, ctx)
}

/// La seleccion de `rol` con las tablas del ADR compilado.
pub fn seleccionar(
    rol: Rol,
    e: &Entrada<'_>,
    cat: &Catalogo,
    inv: &Inventario,
    ctx: &Contexto<'_>,
) -> Result<Seleccion, String> {
    seleccionar_con(tabla::ADR, rol, e, cat, inv, ctx)
}

/// Como [`seleccionar`], con las tablas de `adr`.
pub fn seleccionar_con(
    adr: &str,
    rol: Rol,
    entrada: &Entrada<'_>,
    cat: &Catalogo,
    inv: &Inventario,
    ctx: &Contexto<'_>,
) -> Result<Seleccion, String> {
    let filas = tabla::seleccion(adr)?;
    let modelos = tabla::modelo(adr)?;
    let rutas = entrada.rutas;
    let mut todas: Vec<String> = rutas
        .iter()
        .map(|r| normalizar(r))
        .filter(|r| !r.is_empty())
        .collect();
    todas.sort();
    todas.dedup();
    // La generica es la de la fila `generica-del-plugin` (decision 6: solo
    // la del plugin; una skill del proyecto con `fallback: true` no la
    // sustituye).
    let generica = filas
        .iter()
        .find(|f| f.accion == "generica-del-plugin")
        .and_then(|f| f.ids.first().cloned());
    let umbral = generica
        .as_deref()
        .and_then(|g| cat.revisora(g))
        .and_then(|r| r.umbral_dominios)
        .unwrap_or(UMBRAL_POR_DEFECTO);
    let mut sel = Seleccion::default();
    let cx = match ctx.complejidad {
        Some(c) if crate::tarea::COMPLEJIDADES.contains(&c) => c,
        _ => {
            sel.avisos.push("complejidad-desconocida".to_owned());
            "media"
        }
    };
    let (id_modelo, modelo) = fila_modelo(&modelos, rol, ctx.carril, cx)
        .ok_or_else(|| format!("tabla-modelo sin fila para {}/{cx}", ctx.carril.nombre()))?;
    sel.modelo = modelo;
    sel.reglas.push(id_modelo);
    sel.senales = dominios::anotar(entrada.dominios);
    let mut e = Estado {
        sel,
        casan: casar(&todas, cat),
        todas,
        umbral,
        generica,
        inv,
        cat,
        ctx,
        entrada,
        dom: None,
        n: 0,
        escalera: Vec::new(),
        extra: Vec::new(),
        plan: true,
    };
    for f in filas
        .iter()
        .filter(|f| f.rol == rol.fila() || f.rol == RolFila::Todos)
    {
        if rol == Rol::Diseno {
            if f.rol == RolFila::Todos {
                e.materializar();
            } else if !e.plan {
                // Tras `sin-plan` no se evalua ninguna fila mas de diseno.
                continue;
            }
        }
        // Las filas que miran «el dominio» se evaluan una vez por dominio.
        let por_dominio = f.condicion.iter().any(|a| {
            matches!(
                a.texto.as_str(),
                "dominio-con-revisora" | "dominio-sin-revisora"
            )
        });
        let doms: Vec<Option<String>> = if por_dominio {
            entrada
                .dominios
                .iter()
                .map(|(d, _)| Some(d.clone()))
                .collect()
        } else {
            vec![None]
        };
        let mut anotada = false;
        for d in doms {
            e.dom = d;
            let cumple = f.condicion.iter().all(|a| e.atomo(&a.texto) != a.negado);
            if cumple && e.aplicar(f) {
                anotada = true;
            }
        }
        e.dom = None;
        if anotada {
            e.sel.reglas.push(f.id.clone());
        }
    }
    if rol == Rol::Diseno {
        e.materializar();
    }
    Ok(e.sel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventario::{Entrada, Origen};

    /// El bloque yaml de las revisoras del plugin 0.16.2, copiado tal cual
    /// (fixtures de `tests/fixtures/revisoras/`).
    fn fixture(n: &str) -> String {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/revisoras/skills")
            .join(n)
            .join("SKILL.md");
        std::fs::read_to_string(p).unwrap_or_default()
    }

    const REVISORAS: [&str; 5] = [
        "angular-vue-reviewer",
        "code-quality-reviewer",
        "csharp-autocad-ifc-reviewer",
        "java-spring-reviewer",
        "rust-wasm-reviewer",
    ];

    fn catalogo(sin: &[&str]) -> (Catalogo, Inventario) {
        let mut rs = Vec::new();
        let mut es = vec![Entrada {
            tipo: Tipo::Agente,
            id: "general-purpose".to_owned(),
            origen: Origen::Integrado,
        }];
        for n in REVISORAS {
            let id = format!("taskcode-plugin:{n}");
            if let Some(r) = catalogo::revisora_de(&id, &fixture(n), n) {
                rs.push(r);
            }
            if !sin.contains(&n) {
                es.push(Entrada {
                    tipo: Tipo::Skill,
                    id,
                    origen: Origen::Plugin {
                        nombre: "taskcode-plugin".to_owned(),
                        version: Some("0.16.2".to_owned()),
                    },
                });
            }
        }
        (Catalogo::nuevo(rs), Inventario::nuevo(es, Vec::new()))
    }

    fn todos() -> Vec<String> {
        tabla::ESCALA.iter().map(|m| (*m).to_owned()).collect()
    }

    fn sel(rutas: &[&str], sin: &[&str], cx: Option<&str>) -> Seleccion {
        let (c, i) = catalogo(sin);
        let permitidos = todos();
        let ctx = Contexto {
            carril: Carril::Funcionalidad,
            complejidad: cx,
            modelos_permitidos: &permitidos,
            ronda: 1,
        };
        let rutas: Vec<String> = rutas.iter().map(|r| (*r).to_owned()).collect();
        revision(&rutas, &c, &i, &ctx).unwrap_or_default()
    }

    /// Las cinco revisoras de los fixtures tienen su bloque yaml dentro de
    /// la cabecera (suposicion del plan).
    #[test]
    fn las_cinco_fixtures_son_revisoras() {
        let (c, _) = catalogo(&[]);
        assert_eq!(c.revisoras().len(), 5);
        let g = c.revisora("taskcode-plugin:code-quality-reviewer");
        assert_eq!(
            g.map(|g| (g.fallback, g.umbral_dominios)),
            Some((true, Some(3)))
        );
    }

    /// C1: un diff Rust + Vue lanza las dos de dominio, sin generica, con la
    /// regla S08, el lanzador S12 y el modelo M04; sin red ni modelo.
    #[test]
    fn rust_y_vue_lanzan_las_dos_de_dominio() {
        let s = sel(&["src/a.rs", "web/App.vue"], &[], Some("simple"));
        assert_eq!(
            s.revisoras,
            [
                "taskcode-plugin:angular-vue-reviewer",
                "taskcode-plugin:rust-wasm-reviewer"
            ]
        );
        assert_eq!(s.agentes, ["general-purpose"]);
        assert_eq!(s.modelo, "sonnet");
        assert_eq!(s.reglas, ["M04", "S08", "S12"]);
        assert!(s.avisos.is_empty(), "{:?}", s.avisos);
        assert_eq!(
            s.rutas.get("taskcode-plugin:rust-wasm-reviewer"),
            Some(&vec!["src/a.rs".to_owned()])
        );
        // Sin distinguir mayusculas y con `\`.
        let s = sel(&["SRC\\B.RS", "web/x.VUE"], &[], Some("simple"));
        assert_eq!(s.revisoras.len(), 2, "{s:?}");
    }

    /// C2: un README suelto va a la generica (S09); con Rust y un README, la
    /// de Rust y la generica con las sueltas (S08); mas de tres dominios, la
    /// generica sola (S09).
    #[test]
    fn la_generica_por_descarte_y_por_exceso() {
        let s = sel(&["README.md"], &[], Some("simple"));
        assert_eq!(s.revisoras, ["taskcode-plugin:code-quality-reviewer"]);
        assert_eq!(s.reglas, ["M04", "S09", "S12"]);
        let s = sel(&["src/a.rs", "README.md"], &[], Some("simple"));
        assert_eq!(s.revisoras.len(), 2);
        assert_eq!(
            s.rutas.get("taskcode-plugin:code-quality-reviewer"),
            Some(&vec!["README.md".to_owned()])
        );
        let s = sel(&["a.rs", "b.vue", "C.java", "d.cs"], &[], Some("media"));
        assert_eq!(s.revisoras, ["taskcode-plugin:code-quality-reviewer"]);
        assert_eq!(s.reglas, ["M05", "S09", "S12"]);
        assert_eq!(s.modelo, "opus");
        assert_eq!(s.rutas.values().map(Vec::len).sum::<usize>(), 4);
    }

    /// C2: lo que la seleccion elige y no esta instalado se quita con aviso
    /// `no-instalado` y la fila S14.
    #[test]
    fn la_no_instalada_se_quita_con_aviso() {
        let s = sel(
            &["src/a.rs", "web/App.vue"],
            &["angular-vue-reviewer"],
            Some("simple"),
        );
        assert_eq!(s.revisoras, ["taskcode-plugin:rust-wasm-reviewer"]);
        assert_eq!(
            s.avisos,
            ["no-instalado:taskcode-plugin:angular-vue-reviewer"]
        );
        assert_eq!(s.reglas, ["M04", "S08", "S12", "S14"]);
        let s = sel(&["README.md"], &["code-quality-reviewer"], Some("simple"));
        assert!(s.revisoras.is_empty());
        assert_eq!(
            s.avisos,
            ["no-instalado:taskcode-plugin:code-quality-reviewer"]
        );
    }

    /// Modelo: complejidad desconocida (media con aviso), carril y
    /// `modelos_permitidos` (S15); el orden de entrada no cambia nada.
    #[test]
    fn modelo_por_carril_complejidad_y_permitidos() {
        let s = sel(&["a.rs"], &[], None);
        assert_eq!((s.modelo.as_str(), s.reglas[0].as_str()), ("opus", "M05"));
        assert_eq!(s.avisos, ["complejidad-desconocida"]);
        let (c, i) = catalogo(&[]);
        let permitidos = vec!["haiku".to_owned(), "sonnet".to_owned()];
        let ctx = Contexto {
            carril: Carril::RiesgoAlto,
            complejidad: Some("trivial"),
            modelos_permitidos: &permitidos,
            ronda: 1,
        };
        let r = |v: &[&str]| {
            let v: Vec<String> = v.iter().map(|x| (*x).to_owned()).collect();
            revision(&v, &c, &i, &ctx).unwrap_or_default()
        };
        let s = r(&["a.rs", "b.vue"]);
        assert_eq!(s.modelo, "sonnet");
        assert_eq!(s.reglas, ["M07", "S08", "S12", "S15"]);
        assert_eq!(s.avisos, ["modelo-no-permitido:opus->sonnet"]);
        assert_eq!(r(&["b.vue", "a.rs", "a.rs"]), s);
        assert_eq!(modelo_api("opus"), "claude-opus-5-5");
        assert_eq!(modelo_api("haiku"), "claude-haiku-5-5");
        assert_eq!(modelo_api("sonnet"), "claude-sonnet-5-5");
    }

    /// Las reglas salen de la tabla: cambiar la accion de S12 en una copia
    /// del ADR cambia el resultado (no estan copiadas en el codigo).
    #[test]
    fn las_reglas_se_leen_de_la_tabla() {
        let (c, i) = catalogo(&[]);
        let permitidos = todos();
        let ctx = Contexto {
            carril: Carril::Funcionalidad,
            complejidad: Some("simple"),
            modelos_permitidos: &permitidos,
            ronda: 1,
        };
        let rutas = vec!["a.rs".to_owned()];
        let adr = tabla::ADR.replace(
            "| S12 | `revision` | `siempre` | `lanzador` | `general-purpose` |",
            "| S12 | `revision` | `carril=fix` | `lanzador` | `general-purpose` |",
        );
        assert_ne!(adr, tabla::ADR);
        let s = revision_con(&adr, &rutas, &c, &i, &ctx).unwrap_or_default();
        assert!(s.agentes.is_empty(), "{s:?}");
        let roto = tabla::ADR.replace("`lanzador`", "`a-ojo`");
        assert!(revision_con(&roto, &rutas, &c, &i, &ctx).is_err());
    }

    // --- F4-6: diseno e implementacion ---

    /// Agentes de diseno del plugin 0.16 (ids de la `tabla-seleccion`).
    const DISENO: [&str; 6] = [
        "arquitectura",
        "riesgos",
        "testing",
        "dominio",
        "rendimiento",
        "unificador",
    ];

    /// Catalogo de la seccion I del ADR: las cinco revisoras, `task-workflow`
    /// declarada en `recursos:` (como la tomo la PoC de `catalogo-skills.yml`)
    /// y lo instalado: lo anterior, los seis agentes de brainstorm y el
    /// integrado; `sin` quita ids del inventario.
    fn catalogo_muestra(sin: &[&str]) -> (Catalogo, Inventario) {
        let (c, i) = catalogo(&[]);
        let mut rs: Vec<Revisora> = c.revisoras().to_vec();
        rs.extend(catalogo::de_config(&[crate::config::Recurso {
            id: "taskcode-plugin:task-workflow".to_owned(),
            tipo: "skill".to_owned(),
            rol: Some("ejecucion".to_owned()),
            dominios: Vec::new(),
            etiquetas: [
                "taskctl",
                "sprints",
                "gitflow",
                "revision-por-pares",
                "planificacion",
            ]
            .iter()
            .map(|x| (*x).to_owned())
            .collect(),
            prioridad: Some(10),
        }]));
        let mut es: Vec<Entrada> = i.entradas().to_vec();
        es.push(Entrada {
            tipo: Tipo::Skill,
            id: "taskcode-plugin:task-workflow".to_owned(),
            origen: Origen::Integrado,
        });
        for n in DISENO {
            es.push(Entrada {
                tipo: Tipo::Agente,
                id: format!("taskcode-plugin:brainstorm-{n}"),
                origen: Origen::Integrado,
            });
        }
        es.retain(|e| !sin.contains(&e.id.as_str()));
        (Catalogo::nuevo(rs), Inventario::nuevo(es, Vec::new()))
    }

    /// Las tres selecciones de una tarea.
    struct Tres {
        d: Seleccion,
        i: Seleccion,
        r: Seleccion,
    }

    #[allow(clippy::too_many_arguments)]
    fn tres(
        tipo: &str,
        carril: Carril,
        cx: &str,
        texto: &dominios::Texto<'_>,
        rutas: &[String],
        cat: &Catalogo,
        inv: &Inventario,
        permitidos: &[String],
        ronda: u32,
    ) -> Tres {
        let tabla = tabla::dominios(tabla::ADR).unwrap_or_default();
        let doms = dominios::dominios(texto, rutas, &tabla, &BTreeMap::new());
        let base = carril.roles_diseno(
            crate::plan::heuristica::por_nivel(cx),
            tipo,
            &crate::config::Config::suelo(),
        );
        let e = super::Entrada {
            rutas,
            dominios: &doms,
            etiquetas: texto.etiquetas,
            roles_base: base,
        };
        let ctx = Contexto {
            carril,
            complejidad: Some(cx),
            modelos_permitidos: permitidos,
            ronda,
        };
        let s = |rol| seleccionar(rol, &e, cat, inv, &ctx).unwrap_or_default();
        Tres {
            d: s(Rol::Diseno),
            i: s(Rol::Implementacion),
            r: s(Rol::Revision),
        }
    }

    fn corto(id: &str) -> String {
        id.rsplit(':')
            .next()
            .unwrap_or(id)
            .trim_start_matches("brainstorm-")
            .to_owned()
    }

    /// Las celdas de la `tabla-muestra` del ADR para una tarea (Dominios,
    /// Diseno, Implementacion, Revision, Modelo D / I / R y Reglas).
    fn celdas(t: &Tres) -> Vec<String> {
        // La PoC muestra la primera senal de cada dominio (ordenadas).
        let mut doms: Vec<String> = Vec::new();
        for x in &t.d.senales {
            let dom = x.split(':').next().unwrap_or_default();
            if !doms.iter().any(|d| d.starts_with(&format!("{dom} ("))) {
                doms.push(format!("{})", x.replacen(':', " (", 1)));
            }
        }
        let roles: Vec<String> =
            t.d.roles
                .iter()
                .map(|a| corto(a))
                .filter(|a| a != "unificador")
                .collect();
        let mut diseno = format!("{}: {}", roles.len(), roles.join(", "));
        if t.d.roles.iter().any(|a| corto(a) == "unificador") {
            diseno.push_str(" + unificador");
        }
        let lista = |v: &[String]| {
            if v.is_empty() {
                "—".to_owned()
            } else {
                v.iter().map(|x| corto(x)).collect::<Vec<_>>().join(", ")
            }
        };
        // Las de dominio por id y la generica al final (como la PoC).
        let mut revs: Vec<String> = t.r.revisoras.iter().map(|x| corto(x)).collect();
        revs.sort_by_key(|x| (x == "code-quality-reviewer", x.clone()));
        let mut revision = if revs.is_empty() {
            "—".to_owned()
        } else {
            revs.join(", ")
        };
        let avisos: Vec<&str> =
            t.r.avisos
                .iter()
                .filter_map(|a| a.strip_prefix("dominio-sin-revisora:"))
                .collect();
        if !avisos.is_empty() {
            revision.push_str(&format!("; aviso: {} sin revisora", avisos.join(", ")));
        }
        let m = |s: &Seleccion| {
            if s.modelo.is_empty() {
                "—".to_owned()
            } else {
                s.modelo.clone()
            }
        };
        vec![
            if doms.is_empty() {
                "—".to_owned()
            } else {
                doms.join(", ")
            },
            diseno,
            lista(&t.i.skills),
            revision,
            format!("{} / {} / {}", m(&t.d), m(&t.i), m(&t.r)),
            format!(
                "{} · {} · {}",
                t.d.reglas.join(" "),
                t.i.reglas.join(" "),
                t.r.reglas.join(" ")
            ),
        ]
    }

    #[derive(serde::Deserialize)]
    struct TareaMuestra {
        tarea: String,
        tipo: String,
        etiquetas: Vec<String>,
        objetivo: String,
        criterios: Vec<String>,
        carril: String,
        rutas: Vec<String>,
    }

    /// Criterio 1 (dentro de muestra): con lo que la PoC T uso de las 10
    /// tareas de ToolsaWeb y OpenDockTrading (fixture
    /// `tests/fixtures/muestra-f1-15/tareas.json`, leida en solo lectura de
    /// sus commits fijos) y el catalogo de la seccion I, [`seleccionar`] da
    /// exactamente las columnas de la `tabla-muestra` del ADR, la que Carlos
    /// marco «Coincide las 10». El carril efectivo es el de la fixture y la
    /// complejidad la de la tabla (entradas de la PoC).
    #[test]
    fn la_muestra_del_adr_se_reproduce() {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/muestra-f1-15/tareas.json");
        let tareas: Vec<TareaMuestra> =
            serde_json::from_str(&std::fs::read_to_string(p).unwrap_or_default())
                .unwrap_or_default();
        assert_eq!(tareas.len(), 10);
        let filas: Vec<Vec<String>> = tabla::filas(tabla::ADR, "tabla-muestra")
            .unwrap_or_default()
            .into_iter()
            .filter(|f| f.first().is_some_and(|c| c.contains('/')))
            .collect();
        assert_eq!(filas.len(), 10);
        let (cat, inv) = catalogo_muestra(&[]);
        let permitidos = todos();
        let mut hash = Vec::new();
        for t in &tareas {
            let fila = filas
                .iter()
                .find(|f| f.first() == Some(&t.tarea))
                .cloned()
                .unwrap_or_default();
            let efectivo = fila
                .get(2)
                .and_then(|c| c.split('→').nth(1))
                .map(str::trim)
                .unwrap_or_default();
            assert_eq!(efectivo, t.carril, "{}", t.tarea);
            let cx = fila
                .get(3)
                .and_then(|c| c.split(' ').next())
                .unwrap_or_default();
            let texto = dominios::Texto {
                objetivo: &t.objetivo,
                criterios: &t.criterios,
                etiquetas: &t.etiquetas,
            };
            let carril = Carril::desde(&t.carril).unwrap_or(Carril::Funcionalidad);
            let s = tres(
                &t.tipo,
                carril,
                cx,
                &texto,
                &t.rutas,
                &cat,
                &inv,
                &permitidos,
                1,
            );
            let esperado: Vec<String> = fila
                .get(4..10)
                .map(|x| x.iter().map(|c| c.replace('`', "")).collect())
                .unwrap_or_default();
            assert_eq!(celdas(&s), esperado, "{}", t.tarea);
            hash.push(celdas(&s));
        }
        // Determinista: otra vez, con el catalogo en otro orden, lo mismo.
        let mut rs: Vec<Revisora> = cat.revisoras().to_vec();
        rs.extend(cat.otros().iter().cloned());
        rs.reverse();
        let cat2 = Catalogo::nuevo(rs);
        assert_eq!(cat2, cat);
    }

    /// Criterio 2: el modelo cambia con el carril y la complejidad en los
    /// tres roles (M01 a M07, errata sin diseno); S13 baja en la ronda 2 sin
    /// dominio y no con `puertas-seguridad`; S15 restringe tambien diseno e
    /// implementacion.
    #[test]
    fn modelo_por_rol_carril_y_complejidad() {
        let (cat, inv) = catalogo_muestra(&[]);
        let permitidos = todos();
        let criterios = vec!["criterio".to_owned()];
        let neutro = dominios::Texto {
            objetivo: "Cambio",
            criterios: &criterios,
            etiquetas: &[],
        };
        let rutas = vec!["src/a.rs".to_owned()];
        let modelos = |carril, cx: &str| {
            let t = tres(
                "feature",
                carril,
                cx,
                &neutro,
                &rutas,
                &cat,
                &inv,
                &permitidos,
                1,
            );
            (
                t.d.modelo,
                t.i.modelo,
                t.r.modelo,
                t.d.reglas.first().cloned().unwrap_or_default(),
            )
        };
        let m = |d: &str, i: &str, r: &str, id: &str| {
            (d.to_owned(), i.to_owned(), r.to_owned(), id.to_owned())
        };
        assert_eq!(
            modelos(Carril::Errata, "media"),
            m("", "haiku", "sonnet", "M01")
        );
        assert_eq!(
            modelos(Carril::Fix, "simple"),
            m("sonnet", "sonnet", "sonnet", "M02")
        );
        assert_eq!(
            modelos(Carril::Fix, "alta"),
            m("opus", "sonnet", "opus", "M03")
        );
        assert_eq!(
            modelos(Carril::Funcionalidad, "trivial"),
            m("sonnet", "sonnet", "sonnet", "M04")
        );
        assert_eq!(
            modelos(Carril::Funcionalidad, "media"),
            m("opus", "sonnet", "opus", "M05")
        );
        assert_eq!(
            modelos(Carril::Funcionalidad, "critica"),
            m("opus", "opus", "opus", "M06")
        );
        assert_eq!(
            modelos(Carril::RiesgoAlto, "simple"),
            m("opus", "opus", "opus", "M07")
        );
        // Errata: sin plan (S01), ningun agente de diseno.
        let t = tres(
            "feature",
            Carril::Errata,
            "media",
            &neutro,
            &rutas,
            &cat,
            &inv,
            &permitidos,
            1,
        );
        assert_eq!(t.d.reglas, ["M01", "S01"]);
        assert!(t.d.roles.is_empty() && t.d.agentes.is_empty());
        // S13: ronda 2 baja sin dominio; con puertas-seguridad no.
        let r2 = |texto: &dominios::Texto<'_>| {
            tres(
                "feature",
                Carril::Funcionalidad,
                "media",
                texto,
                &rutas,
                &cat,
                &inv,
                &permitidos,
                2,
            )
            .r
        };
        let s = r2(&neutro);
        assert_eq!(s.modelo, "sonnet");
        assert!(s.reglas.contains(&"S13".to_owned()), "{s:?}");
        let puertas = dominios::Texto {
            objetivo: "El check obligatorio",
            criterios: &criterios,
            etiquetas: &[],
        };
        let s = r2(&puertas);
        assert_eq!(s.modelo, "opus");
        assert!(!s.reglas.contains(&"S13".to_owned()), "{s:?}");
        assert_eq!(s.avisos, ["dominio-sin-revisora:puertas-seguridad"]);
        assert!(s.reglas.contains(&"S11".to_owned()), "{s:?}");
        // S15 en diseno e implementacion.
        let solo = vec!["haiku".to_owned(), "sonnet".to_owned()];
        let t = tres(
            "feature",
            Carril::RiesgoAlto,
            "media",
            &neutro,
            &rutas,
            &cat,
            &inv,
            &solo,
            1,
        );
        for s in [&t.d, &t.i] {
            assert_eq!(s.modelo, "sonnet", "{s:?}");
            assert!(s.reglas.contains(&"S15".to_owned()), "{s:?}");
            assert_eq!(s.avisos, ["modelo-no-permitido:opus->sonnet"]);
        }
    }

    /// S06 y S07 (decision 8): las skills de diseno de un dominio detectado;
    /// la de ejecucion por mayor solape (etiquetas y dominios), despues mayor
    /// prioridad, despues menor id, con aviso `empate-por-id`.
    #[test]
    fn skills_de_diseno_y_ejecucion_por_dominio_solape_prioridad_e_id() {
        let (cat, inv) = catalogo_muestra(&[]);
        let extra: Vec<crate::config::Recurso> = [
            ("d-rend", "diseno", "", "rendimiento", 0),
            ("d-puertas", "diseno", "", "puertas-seguridad", 0),
            ("e-solape2", "ejecucion", "k1,k2", "", 0),
            ("e-prio1", "ejecucion", "k1", "", 1),
            ("e-prio5", "ejecucion", "k1", "", 5),
            ("e-m", "ejecucion", "k3", "", 2),
            ("e-n", "ejecucion", "k3", "", 2),
            ("e-dom", "ejecucion", "", "rendimiento", 0),
        ]
        .iter()
        .map(|(id, rol, etq, dom, prioridad)| {
            let lista = |s: &str| {
                s.split(',')
                    .filter(|x| !x.is_empty())
                    .map(str::to_owned)
                    .collect()
            };
            crate::config::Recurso {
                id: (*id).to_owned(),
                tipo: "skill".to_owned(),
                rol: Some((*rol).to_owned()),
                dominios: lista(dom),
                etiquetas: lista(etq),
                prioridad: Some(*prioridad),
            }
        })
        .collect();
        let mut rs: Vec<Revisora> = cat.revisoras().iter().chain(cat.otros()).cloned().collect();
        rs.extend(catalogo::de_config(&extra));
        let mut es: Vec<Entrada> = inv.entradas().to_vec();
        es.extend(extra.iter().map(|r| Entrada {
            tipo: Tipo::Skill,
            id: r.id.clone(),
            origen: Origen::Proyecto,
        }));
        let (cat, inv) = (Catalogo::nuevo(rs), Inventario::nuevo(es, Vec::new()));
        let criterios = vec!["criterio".to_owned()];
        let con = |etq: &[&str]| {
            let etq: Vec<String> = etq.iter().map(|e| (*e).to_owned()).collect();
            let texto = dominios::Texto {
                objetivo: "Cambio",
                criterios: &criterios,
                etiquetas: &etq,
            };
            let t = tres(
                "feature",
                Carril::Funcionalidad,
                "simple",
                &texto,
                &[],
                &cat,
                &inv,
                &todos(),
                1,
            );
            (t.d, t.i)
        };
        // S06: solo la de diseno del dominio detectado.
        let (d, i) = con(&["wasm"]);
        assert_eq!(d.skills, ["d-rend"]);
        assert!(d.reglas.contains(&"S06".to_owned()), "{d:?}");
        // S07 por dominio (solape 1 con rendimiento).
        assert_eq!(i.skills, ["e-dom"]);
        let (d, _) = con(&["k1"]);
        assert!(
            d.skills.is_empty() && !d.reglas.contains(&"S06".to_owned()),
            "{d:?}"
        );
        // Mayor solape gana a mayor prioridad.
        let (_, i) = con(&["k1", "k2"]);
        assert_eq!((i.skills, i.avisos), (vec!["e-solape2".to_owned()], vec![]));
        // Mismo solape: mayor prioridad, aunque su id sea mayor.
        let (_, i) = con(&["k1"]);
        assert!(i.reglas.contains(&"S07".to_owned()), "{i:?}");
        assert_eq!((i.skills, i.avisos), (vec!["e-prio5".to_owned()], vec![]));
        // Mismo solape y prioridad: menor id, con aviso.
        let (_, i) = con(&["k3"]);
        assert_eq!(i.skills, ["e-m"]);
        assert_eq!(i.avisos, ["empate-por-id:e-m,e-n"]);
    }

    /// A5-03 (TASK-143, F1-15 S10 y S11): una revisora de dominio declarada
    /// en `recursos:` y no instalada cuenta como «sin revisora»: S11 con el
    /// aviso `dominio-sin-revisora`, sin S10, S14 ni `no-instalado`.
    /// Instalada, S10 la anade sin aviso.
    #[test]
    fn revisora_de_dominio_no_instalada_es_dominio_sin_revisora() {
        let (cat, inv) = catalogo_muestra(&[]);
        let mut rs: Vec<Revisora> = cat.revisoras().iter().chain(cat.otros()).cloned().collect();
        rs.extend(catalogo::de_config(&[crate::config::Recurso {
            id: "security-review".to_owned(),
            tipo: "skill".to_owned(),
            rol: Some("revisor".to_owned()),
            dominios: vec!["puertas-seguridad".to_owned()],
            etiquetas: Vec::new(),
            prioridad: None,
        }]));
        let cat = Catalogo::nuevo(rs);
        let criterios = vec!["criterio".to_owned()];
        let puertas = dominios::Texto {
            objetivo: "El check obligatorio",
            criterios: &criterios,
            etiquetas: &[],
        };
        let rutas = vec!["src/a.rs".to_owned()];
        let r = |inv: &Inventario| {
            tres(
                "feature",
                Carril::Funcionalidad,
                "media",
                &puertas,
                &rutas,
                &cat,
                inv,
                &todos(),
                1,
            )
            .r
        };
        let s = r(&inv);
        assert_eq!(s.avisos, ["dominio-sin-revisora:puertas-seguridad"]);
        assert!(s.reglas.contains(&"S11".to_owned()), "{s:?}");
        assert!(!s.reglas.iter().any(|x| x == "S10" || x == "S14"), "{s:?}");
        assert!(
            !s.revisoras.contains(&"security-review".to_owned()),
            "{s:?}"
        );
        // Instalada: S10 sin aviso.
        let mut es = inv.entradas().to_vec();
        es.push(Entrada {
            tipo: Tipo::Skill,
            id: "security-review".to_owned(),
            origen: Origen::Proyecto,
        });
        let s = r(&Inventario::nuevo(es, Vec::new()));
        assert!(s.revisoras.contains(&"security-review".to_owned()), "{s:?}");
        assert!(
            s.reglas.contains(&"S10".to_owned()) && !s.reglas.contains(&"S11".to_owned()),
            "{s:?}"
        );
        assert!(s.avisos.is_empty(), "{s:?}");
    }

    /// Criterio 3 (RR-17): el nucleo de `seleccion` no nombra procesos, red,
    /// ficheros ni clientes de modelos fuera de los tests; y un agente de
    /// diseno quitado del inventario sale con `no-instalado` (S14), sin
    /// dejar de estar en los roles (la peticion se escribe igual).
    #[test]
    fn sin_red_ni_modelo() {
        for (n, fuente) in [
            ("mod.rs", include_str!("mod.rs")),
            ("dominios.rs", include_str!("dominios.rs")),
            ("tabla.rs", include_str!("tabla.rs")),
        ] {
            let nucleo = fuente.split("#[cfg(test)]").next().unwrap_or_default();
            for prohibido in [
                "std::process",
                "std::net",
                "std::fs",
                "ureq",
                "reqwest",
                "Command::new",
                "api.anthropic",
            ] {
                assert!(!nucleo.contains(prohibido), "{n} nombra {prohibido}");
            }
        }
        let (cat, inv) = catalogo_muestra(&["taskcode-plugin:brainstorm-riesgos"]);
        let criterios = vec!["criterio".to_owned()];
        let puertas = dominios::Texto {
            objetivo: "Un token nuevo",
            criterios: &criterios,
            etiquetas: &[],
        };
        let t = tres(
            "feature",
            Carril::Funcionalidad,
            "simple",
            &puertas,
            &[],
            &cat,
            &inv,
            &todos(),
            1,
        );
        assert_eq!(
            t.d.roles,
            [
                "taskcode-plugin:brainstorm-arquitectura",
                "taskcode-plugin:brainstorm-riesgos",
                "taskcode-plugin:brainstorm-unificador"
            ]
        );
        assert_eq!(
            t.d.agentes,
            [
                "taskcode-plugin:brainstorm-arquitectura",
                "taskcode-plugin:brainstorm-unificador"
            ]
        );
        assert_eq!(
            t.d.avisos,
            ["no-instalado:taskcode-plugin:brainstorm-riesgos"]
        );
        assert_eq!(t.d.reglas, ["M04", "S02", "S03", "S05", "S14"]);
        // Rendimiento por etiqueta: su rol y, con dos, el unificador.
        let etq = vec!["wasm".to_owned()];
        let rend = dominios::Texto {
            objetivo: "Cambio",
            criterios: &criterios,
            etiquetas: &etq,
        };
        let (cat, inv) = catalogo_muestra(&[]);
        let t = tres(
            "feature",
            Carril::Funcionalidad,
            "simple",
            &rend,
            &[],
            &cat,
            &inv,
            &todos(),
            1,
        );
        assert_eq!(t.d.reglas, ["M04", "S02", "S04", "S05"]);
        assert_eq!(t.d.roles.len(), 3, "{:?}", t.d.roles);
    }
}
