//! Seleccion determinista de revisoras y modelo (F6-3; ADR F1-15 decision 1,
//! adenda F6-3).
//!
//! [`revision`] es una funcion pura: recibe las rutas del diff (los dos
//! nombres de cada renombrado), el catalogo y el inventario de la base, el
//! carril efectivo, la complejidad y los modelos permitidos, y devuelve las
//! revisoras, el lanzador, el modelo, las filas que se aplicaron y los
//! avisos. No lee ficheros, no llama a la red ni a un modelo: recorre las
//! filas de `revision` y `todos` de `tabla-seleccion` en su orden ([`tabla`])
//! y la columna «Revisión» de `tabla-modelo`.
//!
//! Lo que F6-3 no evalua todavia (adenda F6-3 de F1-15): sin el detector
//! `dominios()` de F4-6 la lista de dominios es vacia, asi que S10 y S11 no
//! se cumplen nunca; sin rondas de F6-6 la ronda es 1 y S13 no se cumple.
//! Las filas se leen igual y su vocabulario se valida.

pub mod catalogo;
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

/// La seleccion de revision.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Seleccion {
    /// Revisoras (skills), por id.
    pub revisoras: Vec<String>,
    /// Agentes que lanzan cada revisora (S12).
    pub agentes: Vec<String>,
    /// Modelo de la escala (`haiku`, `sonnet` u `opus`).
    pub modelo: String,
    /// Ids de las filas que se aplicaron (la de `tabla-modelo` primero).
    pub reglas: Vec<String>,
    /// Avisos (`no-instalado:<id>`, `complejidad-desconocida`...).
    pub avisos: Vec<String>,
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

/// Modelo de revision de `tabla-modelo` para el carril y la complejidad.
fn fila_modelo(filas: &[tabla::FilaModelo], carril: Carril, cx: &str) -> Option<(String, String)> {
    filas
        .iter()
        .find(|f| {
            f.carril == carril.nombre()
                && (f.complejidades.is_empty() || f.complejidades.iter().any(|x| x == cx))
        })
        .map(|f| (f.id.clone(), f.revision.clone()))
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
    ctx: &'a Contexto<'a>,
}

impl Estado<'_> {
    fn instalado(&self, id: &str) -> bool {
        self.inv.contiene(Tipo::Skill, id) || self.inv.contiene(Tipo::Agente, id)
    }

    fn atomo(&self, texto: &str) -> bool {
        let n = u32::try_from(self.casan.por_revisora.len()).unwrap_or(u32::MAX);
        let elegidos = || self.sel.revisoras.iter().chain(&self.sel.agentes);
        match texto {
            "siempre" => true,
            "rutas-casan-entre-1-y-umbral" => (1..=self.umbral).contains(&n),
            "rutas-no-casan-o-pasan-umbral" => n == 0 || n > self.umbral,
            "hay-no-instalado" => elegidos().any(|id| !self.instalado(id)),
            "modelo-no-permitido" => !self.ctx.modelos_permitidos.contains(&self.sel.modelo),
            // Sin detector de dominios (F4-6) ni skills de diseno o ejecucion
            // en la revision: no se cumplen.
            "dominio-con-revisora"
            | "dominio-sin-revisora"
            | "hay-skill-diseno-del-dominio"
            | "hay-skill-ejecucion-con-solape" => false,
            t => {
                if let Some(c) = t.strip_prefix("carril!=") {
                    c != self.ctx.carril.nombre()
                } else if let Some(c) = t.strip_prefix("carril=") {
                    c == self.ctx.carril.nombre()
                } else if let Some(r) = t.strip_prefix("ronda>=") {
                    r.parse().is_ok_and(|r: u32| self.ctx.ronda >= r)
                } else {
                    // `dominio=` (lista vacia) y `roles>=` (solo diseno).
                    false
                }
            }
        }
    }

    /// Aplica la accion de `f`; `true` si cambio algo.
    fn aplicar(&mut self, f: &FilaSeleccion) -> bool {
        let antes = self.sel.clone();
        match f.accion.as_str() {
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
                    .filter(|id| !self.instalado(id))
                    .cloned()
                    .collect();
                for id in &fuera {
                    self.sel.avisos.push(format!("no-instalado:{id}"));
                    self.sel.rutas.remove(id);
                }
                self.sel.revisoras.retain(|id| !fuera.contains(id));
                self.sel.agentes.retain(|id| !fuera.contains(id));
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
                // `revisora-del-dominio` y `aviso=dominio-sin-revisora`: sin
                // dominios no hay nada que anadir.
            }
        }
        self.sel.revisoras.sort();
        self.sel.revisoras.dedup();
        self.sel.agentes.sort();
        self.sel.agentes.dedup();
        self.sel != antes
    }
}

/// La seleccion de revision para las `rutas` del diff. `Err` si las tablas
/// del ADR no se leen (no pasa con el ADR compilado: lo prueba
/// `las_tablas_del_adr_se_leen`).
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
    let filas = tabla::seleccion(adr)?;
    let modelos = tabla::modelo(adr)?;
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
    let (id_modelo, modelo) = fila_modelo(&modelos, ctx.carril, cx)
        .ok_or_else(|| format!("tabla-modelo sin fila para {}/{cx}", ctx.carril.nombre()))?;
    sel.modelo = modelo;
    sel.reglas.push(id_modelo);
    let mut e = Estado {
        sel,
        casan: casar(&todas, cat),
        todas,
        umbral,
        generica,
        inv,
        ctx,
    };
    for f in filas
        .iter()
        .filter(|f| matches!(f.rol, RolFila::Revision | RolFila::Todos))
    {
        let cumple = f.condicion.iter().all(|a| e.atomo(&a.texto) != a.negado);
        if cumple && e.aplicar(f) {
            e.sel.reglas.push(f.id.clone());
        }
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
}
