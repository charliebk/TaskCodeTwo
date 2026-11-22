//! Las tablas `tabla-seleccion`, `tabla-modelo` y `tabla-dominios` del ADR
//! F1-15, leidas del propio ADR (decision 1: las reglas no se escriben aparte en el codigo).
//!
//! El ADR entra en el binario con `include_str!`: cambiar una fila del ADR
//! cambia la seleccion al recompilar, y una condicion o una accion fuera del
//! vocabulario cerrado de la decision 1 es un error de lectura (el test
//! `las_tablas_del_adr_se_leen` lo comprueba en cada compilacion de tests).

/// El ADR F1-15 entero.
pub const ADR: &str =
    include_str!("../../../../docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md");

/// Escala de modelos, de menor a mayor (F1-15 decision 7).
pub const ESCALA: [&str; 3] = ["haiku", "sonnet", "opus"];

/// Carriles que puede nombrar un atomo `carril=`.
const CARRILES: [&str; 4] = ["errata", "fix", "funcionalidad", "riesgo-alto"];

/// Dominios cerrados (F1-15 decision 3).
pub const DOMINIOS: [&str; 2] = ["puertas-seguridad", "rendimiento"];

/// Atomos sin argumento del vocabulario cerrado (decision 1).
const ATOMOS_FIJOS: [&str; 9] = [
    "siempre",
    "hay-skill-diseno-del-dominio",
    "hay-skill-ejecucion-con-solape",
    "rutas-casan-entre-1-y-umbral",
    "rutas-no-casan-o-pasan-umbral",
    "dominio-con-revisora",
    "dominio-sin-revisora",
    "hay-no-instalado",
    "modelo-no-permitido",
];

/// Rol de una fila de `tabla-seleccion`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RolFila {
    /// `diseno`.
    Diseno,
    /// `implementacion`.
    Implementacion,
    /// `revision`.
    Revision,
    /// `todos`.
    Todos,
}

/// Un atomo de una condicion, con su negacion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Atomo {
    /// Si va con `no` delante.
    pub negado: bool,
    /// El atomo sin el `no` (`carril=fix`, `ronda>=2`, `siempre`...).
    pub texto: String,
}

/// Una fila de `tabla-seleccion`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilaSeleccion {
    /// `S01`...
    pub id: String,
    /// Rol al que se aplica.
    pub rol: RolFila,
    /// Atomos unidos por «y».
    pub condicion: Vec<Atomo>,
    /// Token de la accion.
    pub accion: String,
    /// Ids del inventario de la fila (vacio con «—»).
    pub ids: Vec<String>,
}

/// Una fila de `tabla-modelo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilaModelo {
    /// `M01`...
    pub id: String,
    /// Carril.
    pub carril: String,
    /// Complejidades a las que se aplica; vacia es «cualquiera».
    pub complejidades: Vec<String>,
    /// Modelo de diseno (columna «Diseño»); `None` con «—» (sin plan).
    pub diseno: Option<String>,
    /// Modelo de implementacion (columna «Implementación»).
    pub implementacion: String,
    /// Modelo de revision (columna «Revisión»).
    pub revision: String,
}

/// Tipo de senal de una fila de `tabla-dominios`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Senal {
    /// Igualdad exacta con una etiqueta de la tarea.
    Etiqueta,
    /// Principio de una palabra del objetivo o de los criterios.
    Palabra,
    /// Glob sobre las rutas del cambio, sin distinguir mayusculas.
    Ruta,
}

impl Senal {
    /// Nombre en la tabla y en las senales anotadas.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Etiqueta => "etiqueta",
            Self::Palabra => "palabra",
            Self::Ruta => "ruta",
        }
    }
}

/// Una fila de `tabla-dominios`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilaDominio {
    /// `D01`...
    pub id: String,
    /// Dominio de la lista cerrada.
    pub dominio: String,
    /// Tipo de senal.
    pub senal: Senal,
    /// Valor (etiqueta, palabra o glob).
    pub valor: String,
}

/// Las filas entre `<!-- <marca>:inicio -->` y `<!-- <marca>:fin -->`, sin
/// la cabecera ni el separador, como listas de celdas sin espacios ni
/// comillas invertidas.
fn filas(texto: &str, marca: &str) -> Result<Vec<Vec<String>>, String> {
    let inicio = format!("<!-- {marca}:inicio -->");
    let fin = format!("<!-- {marca}:fin -->");
    let desde = texto
        .find(&inicio)
        .ok_or_else(|| format!("falta la marca {inicio}"))?;
    let resto = texto.get(desde + inicio.len()..).unwrap_or_default();
    let hasta = resto
        .find(&fin)
        .ok_or_else(|| format!("falta la marca {fin}"))?;
    let cuerpo = resto.get(..hasta).unwrap_or_default();
    let mut salida = Vec::new();
    for l in cuerpo.lines().map(str::trim).filter(|l| l.starts_with('|')) {
        let celdas: Vec<String> = l
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim().to_owned())
            .collect();
        let primera = celdas.first().map(String::as_str).unwrap_or_default();
        if primera == "Id" || primera.starts_with("---") {
            continue;
        }
        salida.push(celdas);
    }
    if salida.is_empty() {
        return Err(format!("{marca} sin filas"));
    }
    Ok(salida)
}

fn sin_comillas(c: &str) -> String {
    c.replace('`', "").trim().to_owned()
}

/// `n` de un atomo `<prefijo>n` con n entero.
fn numero(texto: &str, prefijo: &str) -> Option<u32> {
    texto.strip_prefix(prefijo).and_then(|n| n.parse().ok())
}

fn atomo_valido(a: &str) -> bool {
    ATOMOS_FIJOS.contains(&a)
        || a.strip_prefix("carril=")
            .or_else(|| a.strip_prefix("carril!="))
            .is_some_and(|c| CARRILES.contains(&c))
        || a.strip_prefix("dominio=")
            .is_some_and(|d| DOMINIOS.contains(&d))
        || numero(a, "roles>=").is_some()
        || numero(a, "ronda>=").is_some()
}

/// Accion valida y, si necesita candidatos, el atomo que los garantiza.
fn accion_valida(accion: &str) -> Result<Option<&'static str>, String> {
    let fija = match accion {
        "sin-plan"
        | "roles=max(roles_min,complejidad)"
        | "lanzador"
        | "generica-del-plugin"
        | "quitar-no-instalado"
        | "restringir-modelo"
        | "aviso=dominio-sin-revisora" => Some(None),
        "skills-diseno-del-dominio" => Some(Some("hay-skill-diseno-del-dominio")),
        "skill-ejecucion=solape,prioridad,id" => Some(Some("hay-skill-ejecucion-con-solape")),
        "revisora-del-dominio" => Some(Some("dominio-con-revisora")),
        "revisoras-de-rutas+generica-si-sueltas" => Some(Some("rutas-casan-entre-1-y-umbral")),
        _ => None,
    };
    if let Some(f) = fija {
        return Ok(f);
    }
    if numero(accion, "suelo=").is_some() || numero(accion, "modelo-baja=").is_some() {
        return Ok(None);
    }
    if accion
        .strip_prefix("mas-rol=")
        .is_some_and(|r| ["rendimiento", "unificador"].contains(&r))
    {
        return Ok(None);
    }
    Err(format!("accion fuera del vocabulario: {accion}"))
}

/// Lee `tabla-seleccion` de `texto` y valida su vocabulario.
pub fn seleccion(texto: &str) -> Result<Vec<FilaSeleccion>, String> {
    let mut v = Vec::new();
    for c in filas(texto, "tabla-seleccion")? {
        let [id, rol, condicion, accion, ids, ..] = c.as_slice() else {
            return Err(format!("fila de tabla-seleccion incompleta: {c:?}"));
        };
        let rol = match sin_comillas(rol).as_str() {
            "diseno" => RolFila::Diseno,
            "implementacion" => RolFila::Implementacion,
            "revision" => RolFila::Revision,
            "todos" => RolFila::Todos,
            otro => return Err(format!("{id}: rol desconocido {otro}")),
        };
        let mut atomos = Vec::new();
        for parte in sin_comillas(condicion).split(" y ") {
            let parte = parte.trim();
            let (negado, texto) = match parte.strip_prefix("no ") {
                Some(r) => (true, r.trim()),
                None => (false, parte),
            };
            if !atomo_valido(texto) {
                return Err(format!("{id}: condicion fuera del vocabulario: {texto}"));
            }
            atomos.push(Atomo {
                negado,
                texto: texto.to_owned(),
            });
        }
        let accion = sin_comillas(accion);
        if let Some(exige) = accion_valida(&accion).map_err(|e| format!("{id}: {e}"))?
            && !atomos.iter().any(|a| !a.negado && a.texto == exige)
        {
            return Err(format!("{id}: la accion {accion} exige el atomo {exige}"));
        }
        let ids = sin_comillas(ids);
        let ids = if ids == "—" || ids.is_empty() {
            Vec::new()
        } else {
            ids.split(',').map(|s| s.trim().to_owned()).collect()
        };
        if v.iter().any(|f: &FilaSeleccion| f.id == *id) {
            return Err(format!("{id} repetida"));
        }
        v.push(FilaSeleccion {
            id: id.clone(),
            rol,
            condicion: atomos,
            accion,
            ids,
        });
    }
    Ok(v)
}

/// Lee `tabla-modelo` de `texto`.
pub fn modelo(texto: &str) -> Result<Vec<FilaModelo>, String> {
    let mut v = Vec::new();
    for c in filas(texto, "tabla-modelo")? {
        let [id, carril, cx, diseno, implementacion, revision, ..] = c.as_slice() else {
            return Err(format!("fila de tabla-modelo incompleta: {c:?}"));
        };
        let carril = sin_comillas(carril);
        if !CARRILES.contains(&carril.as_str()) {
            return Err(format!("{id}: carril desconocido {carril}"));
        }
        let cx = sin_comillas(cx);
        let complejidades = if cx == "cualquiera" {
            Vec::new()
        } else {
            cx.split(',').map(|s| s.trim().to_owned()).collect()
        };
        for x in &complejidades {
            if !crate::tarea::COMPLEJIDADES.contains(&x.as_str()) {
                return Err(format!("{id}: complejidad desconocida {x}"));
            }
        }
        let en_escala = |m: &str| -> Result<String, String> {
            let m = sin_comillas(m);
            if ESCALA.contains(&m.as_str()) {
                Ok(m)
            } else {
                Err(format!("{id}: modelo fuera de la escala {m}"))
            }
        };
        // «—» en diseno: sin plan (errata, S01).
        let diseno = if sin_comillas(diseno) == "—" {
            None
        } else {
            Some(en_escala(diseno)?)
        };
        v.push(FilaModelo {
            id: id.clone(),
            carril,
            complejidades,
            diseno,
            implementacion: en_escala(implementacion)?,
            revision: en_escala(revision)?,
        });
    }
    // Cada combinacion de carril y complejidad, exactamente una fila.
    for carril in CARRILES {
        for cx in crate::tarea::COMPLEJIDADES {
            let n = v
                .iter()
                .filter(|f| {
                    f.carril == carril
                        && (f.complejidades.is_empty() || f.complejidades.iter().any(|x| x == cx))
                })
                .count();
            if n != 1 {
                return Err(format!("tabla-modelo: {carril}/{cx} tiene {n} filas"));
            }
        }
    }
    Ok(v)
}

/// Lee `tabla-dominios` de `texto`: dominio de la lista cerrada, senal
/// `etiqueta|palabra|ruta` y valor no vacio.
pub fn dominios(texto: &str) -> Result<Vec<FilaDominio>, String> {
    let mut v: Vec<FilaDominio> = Vec::new();
    for c in filas(texto, "tabla-dominios")? {
        let [id, dominio, senal, valor, ..] = c.as_slice() else {
            return Err(format!("fila de tabla-dominios incompleta: {c:?}"));
        };
        let dominio = sin_comillas(dominio);
        if !DOMINIOS.contains(&dominio.as_str()) {
            return Err(format!("{id}: dominio fuera de la lista cerrada {dominio}"));
        }
        let senal = match sin_comillas(senal).as_str() {
            "etiqueta" => Senal::Etiqueta,
            "palabra" => Senal::Palabra,
            "ruta" => Senal::Ruta,
            otra => return Err(format!("{id}: senal desconocida {otra}")),
        };
        let valor = sin_comillas(valor);
        if valor.is_empty() {
            return Err(format!("{id}: valor vacio"));
        }
        if v.iter().any(|f| f.id == *id) {
            return Err(format!("{id} repetida"));
        }
        v.push(FilaDominio {
            id: id.clone(),
            dominio,
            senal,
            valor,
        });
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Las dos tablas del ADR real se leen con el vocabulario cerrado.
    #[test]
    fn las_tablas_del_adr_se_leen() {
        let s = seleccion(ADR).unwrap_or_default();
        assert_eq!(s.len(), 15, "{:?}", seleccion(ADR).err());
        let s08 = s.iter().find(|f| f.id == "S08");
        assert_eq!(
            s08.map(|f| (f.rol, f.accion.as_str())),
            Some((RolFila::Revision, "revisoras-de-rutas+generica-si-sueltas"))
        );
        let s13 = s
            .iter()
            .find(|f| f.id == "S13")
            .map(|f| f.condicion.clone());
        assert_eq!(s13.map(|c| c.len()), Some(3));
        let m = modelo(ADR).unwrap_or_default();
        assert_eq!(m.len(), 7, "{:?}", modelo(ADR).err());
        // Las tres columnas: M01 (errata) sin diseno; M05 opus/sonnet/opus.
        let col = |id: &str| {
            m.iter().find(|f| f.id == id).map(|f| {
                (
                    f.diseno.clone(),
                    f.implementacion.clone(),
                    f.revision.clone(),
                )
            })
        };
        assert_eq!(
            col("M01"),
            Some((None, "haiku".to_owned(), "sonnet".to_owned()))
        );
        assert_eq!(
            col("M05"),
            Some((
                Some("opus".to_owned()),
                "sonnet".to_owned(),
                "opus".to_owned()
            ))
        );
        let d = dominios(ADR).unwrap_or_default();
        assert_eq!(d.len(), 26, "{:?}", dominios(ADR).err());
        assert_eq!(
            d.iter().find(|f| f.id == "D22").map(|f| (
                f.dominio.as_str(),
                f.senal,
                f.valor.as_str()
            )),
            Some(("puertas-seguridad", Senal::Ruta, ".claude/**"))
        );
        // Fuera de la lista cerrada o de las senales, no se lee.
        assert!(dominios(&ADR.replace("| D23 | `rendimiento`", "| D23 | `datos`")).is_err());
        assert!(
            dominios(&ADR.replace("| ruta | `.claude/**`", "| fichero | `.claude/**`")).is_err()
        );
    }

    /// Lo que no es del vocabulario no se lee.
    #[test]
    fn fuera_del_vocabulario_es_error() {
        let base = "<!-- tabla-seleccion:inicio -->\n| Id | Rol | Condición | Acción | Ids | Qué |\n|---|---|---|---|---|---|\n| S01 | `revision` | COND | ACC | — | x |\n<!-- tabla-seleccion:fin -->\n";
        let con = |c: &str, a: &str| seleccion(&base.replace("COND", c).replace("ACC", a));
        assert!(con("`siempre`", "`lanzador`").is_ok());
        assert!(con("`a-ojo`", "`lanzador`").is_err());
        assert!(con("`siempre`", "`preguntar-al-modelo`").is_err());
        assert!(con("`carril=otro`", "`lanzador`").is_err());
        // Una accion con candidatos exige su atomo.
        assert!(con("`siempre`", "`revisora-del-dominio`").is_err());
        assert!(con("`dominio-con-revisora`", "`revisora-del-dominio`").is_ok());
        assert!(con("`siempre`", "`revisoras-de-rutas+generica-si-sueltas`").is_err());
    }
}
