//! Heuristica de complejidad de la 0.16 (`scripts/heuristica-complejidad.yml`
//! y `src/core/heuristica.ts` del plugin 0.16.2) y validacion del enunciado
//! que bloquea `plan` (F4-1, D5).
//!
//! Aritmetica sobre lo que ya esta en la tarea (etiquetas, tipo,
//! dependencias, criterios y palabras de riesgo): ninguna llamada a un
//! modelo. Los pesos van como constantes, no en un YAML: la configuracion
//! del proyecto solo puede subir roles (`politica_carriles`), nunca bajar
//! estos numeros.
//!
//! De la validacion de la 0.16 (`validacion-tarea.ts`) se queda lo que
//! bloquea por forma: objetivo vacio, ningun criterio, un criterio vacio y
//! mas de [`MAX_CRITERIOS`]. No se avisa de palabras vagas ni de varios
//! frentes (plan de TASK-036).

use crate::tarea::Cuerpo;
use crate::tarea::texto::plegar;

/// Cada etiqueta mas alla de la primera.
pub const PESO_ETIQUETA_ADICIONAL: u32 = 1;
/// Cada palabra de alto riesgo distinta del objetivo o los criterios.
pub const PESO_PALABRA_ALTO_RIESGO: u32 = 2;
/// Cada dependencia declarada.
pub const PESO_DEPENDENCIA: u32 = 1;
/// Tipo `release`.
pub const PESO_TIPO_RELEASE: u32 = 1;
/// Tipo `hotfix` con alguna palabra de riesgo.
pub const PESO_TIPO_HOTFIX_CON_PALABRA_RIESGO: u32 = 2;
/// [`UMBRAL_CRITERIOS`] criterios o mas, una sola vez.
pub const PESO_CRITERIOS: u32 = 1;
/// Criterios a partir de los que se suma [`PESO_CRITERIOS`].
pub const UMBRAL_CRITERIOS: usize = 5;

/// Ultimo valor de cada nivel: trivial, simple, media y alta; a partir del
/// siguiente, critica (cortes 0/3/5/7, critica desde 8).
pub const CORTES: [u32; 4] = [0, 3, 5, 7];

/// Niveles de complejidad, de menor a mayor (los de la 0.16).
pub const NIVELES: [&str; 5] = crate::tarea::COMPLEJIDADES;

/// Roles por nivel, en el orden de [`NIVELES`] (0-1-2-3-4).
pub const ROLES_POR_NIVEL: [u8; 5] = [0, 1, 2, 3, 4];

/// Roles de un `hotfix` como mucho (tope, no sustitucion).
pub const ROLES_HOTFIX: u8 = 1;

/// Las 14 palabras de alto riesgo, en minusculas y sin tildes; se comparan
/// por subcadena sobre el texto plegado.
pub const PALABRAS_ALTO_RIESGO: [&str; 14] = [
    "migracion",
    "breaking change",
    "seguridad",
    "autenticacion",
    "autorizacion",
    "base de datos",
    "rollback",
    "cifrado",
    "datos personales",
    "concurrencia",
    "rendimiento",
    "compatibilidad hacia atras",
    "integracion externa",
    "irreversible",
];

/// Mas criterios que estos bloquea: la tarea es demasiado grande.
pub const MAX_CRITERIOS: usize = 12;

/// Titulo de la seccion del objetivo.
const OBJETIVO: &str = "Objetivo";
/// Titulo de la seccion de los criterios.
const CRITERIOS: &str = "Criterios de aceptacion";

/// El objetivo y los criterios de una tarea.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Enunciado {
    /// Texto de `## Objetivo`, sin el titulo y recortado.
    pub objetivo: String,
    /// Cada casilla `- [ ]` (o `- [x]`) de `## Criterios de aceptacion`.
    pub criterios: Vec<String>,
}

/// Un hueco de la plantilla (`<que se quiere y por que>`): cuenta como
/// vacio.
fn hueco(s: &str) -> bool {
    s.starts_with('<') && s.ends_with('>') && !s.contains('\n')
}

impl Enunciado {
    /// El enunciado del cuerpo de una tarea. Los huecos de la plantilla de
    /// F1-9 cuentan como vacios.
    pub fn de(cuerpo: &Cuerpo) -> Self {
        let sin_titulo = |t: &str| -> String {
            t.split_once('\n')
                .map_or("", |(_, resto)| resto)
                .trim()
                .to_owned()
        };
        let objetivo = cuerpo
            .seccion(OBJETIVO)
            .map(|s| sin_titulo(&s.texto))
            .filter(|o| !hueco(o))
            .unwrap_or_default();
        let criterios = cuerpo.seccion(CRITERIOS).map_or_else(Vec::new, |s| {
            s.texto
                .lines()
                .filter_map(|l| {
                    ["- [ ]", "- [x]", "- [X]"]
                        .iter()
                        .find_map(|p| l.strip_prefix(p))
                })
                .map(|c| {
                    let c = c.trim();
                    if hueco(c) {
                        String::new()
                    } else {
                        c.to_owned()
                    }
                })
                .collect()
        });
        Self {
            objetivo,
            criterios,
        }
    }

    /// Lo que bloquea `plan`: el `motivo=` del rechazo y, para un criterio
    /// vacio, su numero (desde 1); `Ok` si el enunciado vale.
    pub fn validar(&self) -> Result<(), (&'static str, usize)> {
        if self.objetivo.trim().is_empty() {
            return Err(("objetivo-vacio", 0));
        }
        if self.criterios.is_empty() {
            return Err(("sin-criterios", 0));
        }
        if let Some(i) = self.criterios.iter().position(|c| c.trim().is_empty()) {
            return Err(("criterio-vacio", i + 1));
        }
        if self.criterios.len() > MAX_CRITERIOS {
            return Err(("demasiados-criterios", self.criterios.len()));
        }
        Ok(())
    }
}

/// Una senal encontrada: clave, detalle y puntos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Senal {
    /// Clave de la senal (las de la 0.16).
    pub clave: &'static str,
    /// De donde sale.
    pub detalle: String,
    /// Puntos que suma.
    pub puntos: u32,
}

/// Complejidad declarada frente a heuristica y los roles que piden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolucion {
    /// Roles por complejidad: el mayor de los dos niveles (con el tope de
    /// `hotfix` aplicado a cada uno).
    pub roles: u8,
    /// Nivel declarado; `None` si la tarea no lo declara (decide la
    /// heuristica).
    pub declarado: Option<String>,
    /// Nivel de la heuristica.
    pub heuristico: &'static str,
    /// Puntos.
    pub puntos: u32,
    /// Senales.
    pub senales: Vec<Senal>,
    /// Si el tope de `hotfix` recorto el numero.
    pub tope_hotfix: bool,
}

impl Resolucion {
    /// Los dos niveles difieren.
    pub fn discrepa(&self) -> bool {
        self.declarado
            .as_deref()
            .is_some_and(|d| d != self.heuristico)
    }
}

/// Las palabras de riesgo distintas que aparecen en el texto, en el orden
/// de la lista: se recorre la lista, no el texto (una vez por entrada).
fn palabras(e: &Enunciado) -> Vec<&'static str> {
    let mut texto = e.objetivo.clone();
    for c in &e.criterios {
        texto.push('\n');
        texto.push_str(c);
    }
    let texto = format!(" {} ", plegar(&texto));
    PALABRAS_ALTO_RIESGO
        .into_iter()
        .filter(|p| texto.contains(&plegar(p)))
        .collect()
}

/// Puntos y senales de una tarea.
pub fn puntuar(
    etiquetas: &[String],
    tipo: &str,
    dependencias: &[String],
    e: &Enunciado,
) -> (u32, Vec<Senal>) {
    let mut s = Vec::new();
    let cuantos = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    let adicionales = etiquetas.len().saturating_sub(1);
    if adicionales > 0 {
        s.push(Senal {
            clave: "etiquetas_adicionales",
            detalle: format!(
                "{} etiquetas ({adicionales} mas alla de la primera)",
                etiquetas.len()
            ),
            puntos: cuantos(adicionales).saturating_mul(PESO_ETIQUETA_ADICIONAL),
        });
    }
    let encontradas = palabras(e);
    for p in &encontradas {
        s.push(Senal {
            clave: "palabra_alto_riesgo",
            detalle: (*p).to_owned(),
            puntos: PESO_PALABRA_ALTO_RIESGO,
        });
    }
    if !dependencias.is_empty() {
        s.push(Senal {
            clave: "dependencias",
            detalle: format!(
                "{} dependencias ({})",
                dependencias.len(),
                dependencias.join(", ")
            ),
            puntos: cuantos(dependencias.len()).saturating_mul(PESO_DEPENDENCIA),
        });
    }
    if tipo == "release" {
        s.push(Senal {
            clave: "tipo_release",
            detalle: "tipo release".to_owned(),
            puntos: PESO_TIPO_RELEASE,
        });
    }
    if tipo == "hotfix" && !encontradas.is_empty() {
        s.push(Senal {
            clave: "tipo_hotfix_con_palabra_riesgo",
            detalle: format!("hotfix que toca {}", encontradas.join(", ")),
            puntos: PESO_TIPO_HOTFIX_CON_PALABRA_RIESGO,
        });
    }
    if e.criterios.len() >= UMBRAL_CRITERIOS {
        s.push(Senal {
            clave: "criterios_aceptacion",
            detalle: format!(
                "{} criterios (umbral: {UMBRAL_CRITERIOS})",
                e.criterios.len()
            ),
            puntos: PESO_CRITERIOS,
        });
    }
    let puntos = s.iter().map(|x| x.puntos).fold(0, u32::saturating_add);
    (puntos, s)
}

/// El nivel de una puntuacion: cada corte es el ultimo valor del nivel.
pub fn nivel(puntos: u32) -> &'static str {
    let i = CORTES
        .iter()
        .position(|c| puntos <= *c)
        .unwrap_or(CORTES.len());
    NIVELES.get(i).copied().unwrap_or("critica")
}

/// Roles que pide un nivel sin el tope de `hotfix`.
fn por_nivel(nivel: &str) -> u8 {
    NIVELES
        .iter()
        .position(|n| *n == nivel)
        .and_then(|i| ROLES_POR_NIVEL.get(i).copied())
        .unwrap_or(0)
}

/// Roles que pide un nivel para un tipo: el tope de `hotfix` es un minimo.
fn con_tope(nivel: &str, tipo: &str) -> u8 {
    let n = por_nivel(nivel);
    if tipo == "hotfix" {
        n.min(ROLES_HOTFIX)
    } else {
        n
    }
}

/// La resolucion de la 0.16 (`resolverNumeroAgentes`): sin complejidad
/// declarada decide solo la heuristica; con ella, el mayor de los dos.
pub fn resolver(
    complejidad: Option<&str>,
    etiquetas: &[String],
    tipo: &str,
    dependencias: &[String],
    e: &Enunciado,
) -> Resolucion {
    let (puntos, senales) = puntuar(etiquetas, tipo, dependencias, e);
    let heuristico = nivel(puntos);
    let niveles: Vec<&str> = complejidad
        .into_iter()
        .filter(|c| NIVELES.contains(c))
        .chain([heuristico])
        .collect();
    let roles = niveles.iter().map(|n| con_tope(n, tipo)).max().unwrap_or(0);
    let sin_tope = niveles.iter().map(|n| por_nivel(n)).max().unwrap_or(0);
    Resolucion {
        roles,
        declarado: complejidad.map(str::to_owned),
        heuristico,
        puntos,
        senales,
        tope_hotfix: roles < sin_tope,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(objetivo: &str, criterios: &[&str]) -> Enunciado {
        Enunciado {
            objetivo: objetivo.to_owned(),
            criterios: criterios.iter().map(|c| (*c).to_owned()).collect(),
        }
    }

    fn v(x: &[&str]) -> Vec<String> {
        x.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn niveles_por_cortes() {
        for (p, n) in [
            (0, "trivial"),
            (1, "simple"),
            (3, "simple"),
            (4, "media"),
            (5, "media"),
            (6, "alta"),
            (7, "alta"),
            (8, "critica"),
            (40, "critica"),
        ] {
            assert_eq!(nivel(p), n, "{p}");
        }
    }

    /// Cada senal con sus puntos; una palabra cuenta una vez aunque salga
    /// en el objetivo y en los criterios, con tildes o mayusculas.
    #[test]
    fn senales_y_puntos() {
        let en = e(
            "Una Migración de la base de datos",
            &["la migracion no rompe", "b", "c", "d", "e"],
        );
        let (p, s) = puntuar(&v(&["a", "b", "c"]), "hotfix", &v(&["TASK-001"]), &en);
        let claves: Vec<(&str, u32)> = s.iter().map(|x| (x.clave, x.puntos)).collect();
        assert_eq!(
            claves,
            [
                ("etiquetas_adicionales", 2),
                ("palabra_alto_riesgo", 2),
                ("palabra_alto_riesgo", 2),
                ("dependencias", 1),
                ("tipo_hotfix_con_palabra_riesgo", 2),
                ("criterios_aceptacion", 1),
            ]
        );
        assert_eq!(p, 10);
        let (p, s) = puntuar(&v(&["a"]), "release", &[], &e("x", &["y"]));
        assert_eq!((p, s.len()), (1, 1));
        assert_eq!(puntuar(&[], "feature", &[], &e("x", &["y"])).0, 0);
    }

    /// La resolucion: el mayor de declarado y heuristico; sin declarar,
    /// solo la heuristica; el tope de hotfix recorta y lo dice.
    #[test]
    fn resolucion() {
        let simple = e("x", &["y"]);
        let r = resolver(None, &[], "feature", &[], &simple);
        assert_eq!((r.roles, r.heuristico, r.discrepa()), (0, "trivial", false));
        let r = resolver(Some("alta"), &[], "feature", &[], &simple);
        assert_eq!((r.roles, r.discrepa(), r.tope_hotfix), (3, true, false));
        let r = resolver(Some("critica"), &[], "hotfix", &[], &simple);
        assert_eq!((r.roles, r.tope_hotfix), (1, true));
        let r = resolver(Some("trivial"), &[], "hotfix", &[], &simple);
        assert_eq!((r.roles, r.tope_hotfix), (0, false));
        let riesgo = e("seguridad y cifrado y rollback", &["y"]);
        let r = resolver(Some("simple"), &[], "feature", &[], &riesgo);
        assert_eq!((r.roles, r.heuristico, r.puntos), (3, "alta", 6));
    }

    /// La validacion que bloquea, con los huecos de la plantilla como
    /// vacios.
    #[test]
    fn enunciado_y_validacion() {
        let c = Cuerpo::con_enunciado("Que haga algo", &v(&["uno", "dos"]));
        let en = Enunciado::de(&c);
        assert_eq!(en, e("Que haga algo", &["uno", "dos"]));
        assert_eq!(en.validar(), Ok(()));
        let plantilla = Enunciado::de(&Cuerpo::plantilla());
        assert_eq!(plantilla, e("", &[""]));
        assert_eq!(plantilla.validar(), Err(("objetivo-vacio", 0)));
        assert_eq!(e("x", &[]).validar(), Err(("sin-criterios", 0)));
        assert_eq!(e("x", &["a", " "]).validar(), Err(("criterio-vacio", 2)));
        let doce: Vec<String> = (1..=12).map(|i| format!("c{i}")).collect();
        let trece: Vec<String> = (1..=13).map(|i| format!("c{i}")).collect();
        let con = |c: &[String]| Enunciado {
            objetivo: "x".into(),
            criterios: c.to_vec(),
        };
        assert_eq!(con(&doce).validar(), Ok(()));
        assert_eq!(con(&trece).validar(), Err(("demasiados-criterios", 13)));
    }
}
