//! Manifiesto de recursos (ADR F1-14, punto 8; F1-15 decision 11): una
//! funcion pura, [`evaluar_recursos`], que cruza el inventario de F3-9 con lo
//! que se recomienda y con el suelo del binario, y dice de cada recurso su
//! nivel, como salio y que hacer.
//!
//! | Nivel | Que entra aqui |
//! |---|---|
//! | obligatorio | el plugin `taskcodetwo` (sus skills de fase llegan con el plugin; mientras no este, una sola `falla`, D3a) |
//! | necesario | lo que nombran `recursos:` de la configuracion y `roles.*` de las tareas no terminadas (D1a) |
//! | conveniente | engram, context7 y rtk: avisa y sigue |
//! | estorba | la 0.16 (`taskcode-plugin`) habilitada a la vez: avisa |
//!
//! No depende de `doctor`: `init`, `start` y `seleccionar` lo usan con su
//! propio adaptador. Sin E/S: lo que hay en la maquina entra ya leido
//! (inventario, servidores MCP y ejecutables encontrados en el PATH).

use std::collections::BTreeSet;

use crate::inventario::{Inventario, Tipo};

/// Nombre del plugin de la 2.0 (F1-12).
pub const PLUGIN: &str = crate::huella::NOMBRE;

/// Nombre del plugin de la 0.16.
pub const PLUGIN_016: &str = "taskcode-plugin";

/// Orden que instala el plugin de la 2.0.
pub const ARREGLO_PLUGIN: &str = "claude plugin install taskcodetwo";

/// Motivo de lo que no esta instalado.
pub const NO_INSTALADO: &str = "no-instalado";

/// Motivo de lo que no se puede decidir porque el inventario no se leyo
/// entero (C8: nunca `ok`).
pub const INCOMPLETO: &str = "inventario-incompleto";

/// Nivel de un recurso (F1-14, punto 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Nivel {
    /// Para con el arreglo en `init`, `doctor` y `start`.
    Obligatorio,
    /// Lo declara el proyecto o la tarea.
    Necesario,
    /// Avisa y sigue.
    Conveniente,
    /// Interfiere: avisa.
    Estorba,
}

impl Nivel {
    /// Valor en la salida.
    pub fn texto(self) -> &'static str {
        match self {
            Self::Obligatorio => "obligatorio",
            Self::Necesario => "necesario",
            Self::Conveniente => "conveniente",
            Self::Estorba => "estorba",
        }
    }
}

/// Como salio la comprobacion de un recurso.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Estado {
    /// Falta algo obligatorio o necesario.
    Falla,
    /// Falta algo conveniente o hay algo que estorba.
    Aviso,
    /// No se pudo decidir.
    NoComprobado,
    /// Esta.
    Ok,
}

/// Que es un recurso recomendado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TipoRecurso {
    /// Skill del inventario.
    Skill,
    /// Agente del inventario.
    Agente,
    /// Ejecutable en el PATH.
    Ejecutable,
}

impl TipoRecurso {
    /// Valor en la salida.
    pub fn texto(self) -> &'static str {
        match self {
            Self::Skill => "skill",
            Self::Agente => "agente",
            Self::Ejecutable => "ejecutable",
        }
    }

    /// El tipo de `recursos[].tipo` de la configuracion.
    pub fn de_texto(t: &str) -> Option<Self> {
        match t {
            "skill" => Some(Self::Skill),
            "agente" => Some(Self::Agente),
            "ejecutable" => Some(Self::Ejecutable),
            _ => None,
        }
    }
}

/// Quien recomienda un recurso.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fuente {
    /// `recursos:` de la configuracion del proyecto.
    Config,
    /// `roles.*` de una tarea no terminada (su ID).
    Tarea(String),
}

/// Un recurso que alguien recomienda.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Recomendado {
    /// Id (el del inventario o el nombre del ejecutable).
    pub id: String,
    /// Que es.
    pub tipo: TipoRecurso,
    /// Quien lo recomienda.
    pub fuente: Fuente,
}

/// Lo que se recomienda y lo que hay en la maquina fuera del inventario.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Manifiesto {
    /// Lo recomendado, en cualquier orden (se ordena y no se repite).
    pub recomendados: Vec<Recomendado>,
    /// Ejecutables encontrados en el PATH (de los que se buscaron).
    pub ejecutables: BTreeSet<String>,
    /// Servidores MCP configurados.
    pub mcp: BTreeSet<String>,
    /// Si el inventario se leyo entero. Si no, lo que no aparece queda
    /// `no-comprobado` (C8).
    pub completo: bool,
}

/// Para que se evalua.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Momento {
    /// `tc2 init`: el suelo y lo de la configuracion.
    Init,
    /// `tc2 doctor --recursos`: todo.
    Doctor,
    /// La entrada de `start` de una tarea: lo obligatorio y lo de esa tarea.
    Start(String),
}

/// Un recurso evaluado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recurso {
    /// Id.
    pub id: String,
    /// `skill`, `agente`, `ejecutable`, `plugin` o `mcp`.
    pub tipo: &'static str,
    /// Nivel.
    pub nivel: Nivel,
    /// Como salio.
    pub estado: Estado,
    /// Por que (sin espacios): `no-instalado`, `inventario-incompleto`,
    /// `taskcode-016-habilitado`...; `None` si esta.
    pub comprobacion: Option<&'static str>,
    /// Tarea que lo recomienda, si es de una tarea.
    pub tarea: Option<String>,
    /// Version, si se sabe (plugins).
    pub version: Option<String>,
    /// Que hacer (sin espacios), si algo no esta bien.
    pub arreglo: Option<String>,
}

impl Recurso {
    fn nuevo(id: impl Into<String>, tipo: &'static str, nivel: Nivel, estado: Estado) -> Self {
        Self {
            id: id.into(),
            tipo,
            nivel,
            estado,
            comprobacion: None,
            tarea: None,
            version: None,
            arreglo: None,
        }
    }

    fn por(mut self, c: &'static str) -> Self {
        self.comprobacion = Some(c);
        self
    }

    fn arreglo(mut self, a: impl Into<String>) -> Self {
        self.arreglo = Some(a.into());
        self
    }
}

/// El resultado de [`evaluar_recursos`], en orden fijo: obligatorio,
/// necesario (por id y fuente), conveniente y estorba.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InformeRecursos {
    /// Los recursos evaluados.
    pub recursos: Vec<Recurso>,
}

impl InformeRecursos {
    /// Cuantos `no-instalado` en `falla` hay.
    pub fn no_instalados(&self) -> usize {
        self.recursos
            .iter()
            .filter(|r| r.estado == Estado::Falla && r.comprobacion == Some(NO_INSTALADO))
            .count()
    }
}

/// El arreglo de un recomendado que falta: el plugin si el id lleva su
/// prefijo; si no, instalarlo o quitarlo de quien lo nombra.
fn arreglo_de(r: &Recomendado) -> String {
    if r.tipo != TipoRecurso::Ejecutable
        && let Some((plugin, _)) = r.id.split_once(':')
        && !plugin.is_empty()
    {
        return format!("claude plugin install {plugin}");
    }
    match &r.fuente {
        Fuente::Config => format!("instalar-{}-o-quitarla-de-recursos", r.tipo.texto()),
        Fuente::Tarea(t) => format!("instalar-{}-o-cambiar-roles-de-{t}", r.tipo.texto()),
    }
}

/// Si un servidor o plugin con `nombre` esta configurado (sin mayusculas;
/// deteccion aproximada).
fn hay_mcp(inv: &Inventario, m: &Manifiesto, nombre: &str) -> bool {
    m.mcp
        .iter()
        .chain(inv.plugins().iter().map(|p| &p.nombre))
        .any(|x| x.to_ascii_lowercase().contains(nombre))
}

/// Evalua el manifiesto contra el inventario (funcion pura, determinista).
pub fn evaluar_recursos(inv: &Inventario, m: &Manifiesto, momento: &Momento) -> InformeRecursos {
    let mut v = Vec::new();

    // Obligatorio: el plugin de la 2.0 (D3a: una sola falla mientras no
    // este; sus skills de fase llegan con el).
    match inv.plugin(PLUGIN) {
        Some(p) => {
            let mut r = Recurso::nuevo(PLUGIN, "plugin", Nivel::Obligatorio, Estado::Ok);
            r.version.clone_from(&p.version);
            v.push(r);
        }
        None if !m.completo => v.push(
            Recurso::nuevo(PLUGIN, "plugin", Nivel::Obligatorio, Estado::NoComprobado)
                .por(INCOMPLETO),
        ),
        None => v.push(
            Recurso::nuevo(PLUGIN, "plugin", Nivel::Obligatorio, Estado::Falla)
                .por(NO_INSTALADO)
                .arreglo(ARREGLO_PLUGIN),
        ),
    }

    // Necesario: lo recomendado, ordenado y sin repetir.
    let mut rec = m.recomendados.clone();
    rec.sort();
    rec.dedup();
    for r in rec.iter().filter(|r| match (momento, &r.fuente) {
        (Momento::Doctor, _) | (Momento::Init, Fuente::Config) => true,
        (Momento::Start(t), Fuente::Tarea(x)) => t == x,
        _ => false,
    }) {
        let esta = match r.tipo {
            TipoRecurso::Skill => inv.contiene(Tipo::Skill, &r.id),
            TipoRecurso::Agente => inv.contiene(Tipo::Agente, &r.id),
            TipoRecurso::Ejecutable => m.ejecutables.contains(&r.id),
        };
        let mut x = if esta {
            Recurso::nuevo(r.id.clone(), r.tipo.texto(), Nivel::Necesario, Estado::Ok)
        } else if r.tipo != TipoRecurso::Ejecutable && !m.completo {
            Recurso::nuevo(
                r.id.clone(),
                r.tipo.texto(),
                Nivel::Necesario,
                Estado::NoComprobado,
            )
            .por(INCOMPLETO)
        } else {
            Recurso::nuevo(
                r.id.clone(),
                r.tipo.texto(),
                Nivel::Necesario,
                Estado::Falla,
            )
            .por(NO_INSTALADO)
            .arreglo(arreglo_de(r))
        };
        if let Fuente::Tarea(t) = &r.fuente {
            x.tarea = Some(t.clone());
        }
        v.push(x);
    }
    if matches!(momento, Momento::Start(_)) {
        return InformeRecursos { recursos: v };
    }

    // Conveniente: avisa y sigue (engram no es obligatorio). Los MCP salen
    // de lo leido (settings, `.mcp.json` y plugins): con el inventario
    // incompleto, no encontrarlos no dice que falten (C8). `rtk` se busca en
    // el PATH, que no depende del inventario.
    for (id, tipo, esta, del_inventario) in [
        ("engram", "mcp", hay_mcp(inv, m, "engram"), true),
        ("context7", "mcp", hay_mcp(inv, m, "context7"), true),
        ("rtk", "ejecutable", m.ejecutables.contains("rtk"), false),
    ] {
        v.push(if esta {
            Recurso::nuevo(id, tipo, Nivel::Conveniente, Estado::Ok)
        } else if del_inventario && !m.completo {
            Recurso::nuevo(id, tipo, Nivel::Conveniente, Estado::NoComprobado).por(INCOMPLETO)
        } else {
            Recurso::nuevo(id, tipo, Nivel::Conveniente, Estado::Aviso)
                .por(NO_INSTALADO)
                .arreglo(format!("instalar-{id}"))
        });
    }

    // Estorba: la 0.16 habilitada a la vez.
    if let Some(p) = inv.plugin(PLUGIN_016) {
        let mut r = Recurso::nuevo(PLUGIN_016, "plugin", Nivel::Estorba, Estado::Aviso)
            .por("taskcode-016-habilitado")
            .arreglo(format!("claude plugin disable {}", p.clave));
        r.version.clone_from(&p.version);
        v.push(r);
    }
    InformeRecursos { recursos: v }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventario::{Entrada, Origen, Plugin};

    fn plugin(nombre: &str) -> Plugin {
        Plugin {
            nombre: nombre.to_owned(),
            clave: format!("{nombre}@mk"),
            version: Some("1.0.0".to_owned()),
        }
    }

    fn inventario(plugins: &[&str], skills: &[&str]) -> Inventario {
        Inventario::nuevo(
            skills
                .iter()
                .map(|s| Entrada {
                    tipo: Tipo::Skill,
                    id: (*s).to_owned(),
                    origen: Origen::Usuario,
                })
                .collect(),
            plugins.iter().map(|p| plugin(p)).collect(),
        )
    }

    fn manifiesto(recomendados: Vec<Recomendado>) -> Manifiesto {
        Manifiesto {
            recomendados,
            completo: true,
            ..Manifiesto::default()
        }
    }

    fn lineas(i: &InformeRecursos) -> Vec<String> {
        i.recursos
            .iter()
            .map(|r| {
                format!(
                    "{} {} {} {:?} {} {}",
                    r.id,
                    r.tipo,
                    r.nivel.texto(),
                    r.estado,
                    r.comprobacion.unwrap_or("-"),
                    r.arreglo.as_deref().unwrap_or("-")
                )
            })
            .collect()
    }

    /// C9: sin el plugin `taskcodetwo`, una sola `falla` con el arreglo de
    /// instalarlo; con la 0.16, `aviso` de nivel estorba; sin engram,
    /// context7 ni rtk, `aviso` de nivel conveniente (no son fallos).
    #[test]
    fn suelo_del_manifiesto() {
        let i = evaluar_recursos(
            &inventario(&[PLUGIN_016], &[]),
            &manifiesto(Vec::new()),
            &Momento::Doctor,
        );
        assert_eq!(
            lineas(&i),
            [
                "taskcodetwo plugin obligatorio Falla no-instalado claude plugin install taskcodetwo",
                "engram mcp conveniente Aviso no-instalado instalar-engram",
                "context7 mcp conveniente Aviso no-instalado instalar-context7",
                "rtk ejecutable conveniente Aviso no-instalado instalar-rtk",
                "taskcode-plugin plugin estorba Aviso taskcode-016-habilitado claude plugin disable taskcode-plugin@mk",
            ]
        );
        assert_eq!(
            i.recursos
                .iter()
                .filter(|r| r.estado == Estado::Falla)
                .count(),
            1
        );
        assert_eq!(i.no_instalados(), 1);
        // Con todo: el plugin, un MCP por nombre, otro por plugin y rtk.
        let m = Manifiesto {
            ejecutables: BTreeSet::from(["rtk".to_owned()]),
            mcp: BTreeSet::from(["Engram-Memoria".to_owned()]),
            ..manifiesto(Vec::new())
        };
        let i = evaluar_recursos(
            &inventario(&[PLUGIN, "context7"], &[]),
            &m,
            &Momento::Doctor,
        );
        assert!(i.recursos.iter().all(|r| r.estado == Estado::Ok), "{i:?}");
        assert_eq!(i.recursos[0].version.as_deref(), Some("1.0.0"));
    }

    /// C2 y C3 (parte pura): lo recomendado que no esta es `falla
    /// no-instalado` con arreglo; si esta, `ok`; con el inventario
    /// incompleto, `no-comprobado` (C8); `start` solo mira su tarea.
    #[test]
    fn lo_recomendado_se_cruza_con_el_inventario() {
        let rec = vec![
            Recomendado {
                id: "fantasma".to_owned(),
                tipo: TipoRecurso::Skill,
                fuente: Fuente::Config,
            },
            Recomendado {
                id: "esta".to_owned(),
                tipo: TipoRecurso::Skill,
                fuente: Fuente::Tarea("TASK-002".to_owned()),
            },
            Recomendado {
                id: "otro:x".to_owned(),
                tipo: TipoRecurso::Agente,
                fuente: Fuente::Tarea("TASK-001".to_owned()),
            },
            Recomendado {
                id: "fantasma".to_owned(),
                tipo: TipoRecurso::Skill,
                fuente: Fuente::Config,
            },
        ];
        let inv = inventario(&[PLUGIN], &["esta"]);
        let i = evaluar_recursos(&inv, &manifiesto(rec.clone()), &Momento::Doctor);
        assert_eq!(
            lineas(&i)[1..4],
            [
                "esta skill necesario Ok - -",
                "fantasma skill necesario Falla no-instalado instalar-skill-o-quitarla-de-recursos",
                "otro:x agente necesario Falla no-instalado claude plugin install otro",
            ]
        );
        assert_eq!(i.recursos[3].tarea.as_deref(), Some("TASK-001"));
        assert_eq!(i.no_instalados(), 2);
        let m = Manifiesto {
            completo: false,
            ..manifiesto(rec.clone())
        };
        let i = evaluar_recursos(&inv, &m, &Momento::Doctor);
        assert_eq!(
            i.recursos
                .iter()
                .filter(|r| r.nivel == Nivel::Necesario && r.comprobacion == Some(INCOMPLETO))
                .count(),
            2
        );
        assert!(i.recursos.iter().all(|r| r.estado != Estado::Falla));
        let i = evaluar_recursos(
            &inv,
            &manifiesto(rec),
            &Momento::Start("TASK-001".to_owned()),
        );
        assert_eq!(
            i.recursos.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
            ["taskcodetwo", "otro:x"]
        );
    }

    /// MEN-6 (ronda 1): con el inventario incompleto, engram y context7 no
    /// encontrados salen `no-comprobado inventario-incompleto` (no `aviso
    /// no-instalado`); rtk, que se busca en el PATH, sigue siendo `aviso`.
    /// Si estan, `ok` aunque falte algo por leer.
    #[test]
    fn convenientes_con_el_inventario_incompleto() {
        let m = Manifiesto {
            completo: false,
            ..manifiesto(Vec::new())
        };
        let i = evaluar_recursos(&inventario(&[PLUGIN], &[]), &m, &Momento::Doctor);
        assert_eq!(
            lineas(&i)[1..],
            [
                "engram mcp conveniente NoComprobado inventario-incompleto -",
                "context7 mcp conveniente NoComprobado inventario-incompleto -",
                "rtk ejecutable conveniente Aviso no-instalado instalar-rtk",
            ]
        );
        let m = Manifiesto {
            mcp: BTreeSet::from(["engram".to_owned()]),
            ..m
        };
        let i = evaluar_recursos(&inventario(&[PLUGIN], &[]), &m, &Momento::Doctor);
        assert_eq!(i.recursos[1].estado, Estado::Ok);
    }
}
