//! Configuracion del proyecto, `.taskcode/config.yml` (F2-7).
//!
//! - [`leer`]: funcion pura. Exige la primera linea `---` y `formato: 2`
//!   (F1-12), lee el YAML con las opciones estrictas de [`crate::yaml`] y lo
//!   valida contra la tabla de [`claves`]. Devuelve la [`Config`] con los
//!   valores por defecto ya aplicados o **todos** los errores con la ruta de
//!   cada clave. Una clave o un valor desconocido falla cerrado (F1-10
//!   decision 2, F1-11 decision 1).
//! - [`de_base`]: la lee de `origin/develop`, nunca del worktree de la tarea
//!   (F1-10 decision 3). No hace `fetch`: lo hace quien la llama.
//! - [`esquema_json`]: el esquema publicado en
//!   `crates/tc2/esquemas/config.schema.json`, generado de la misma tabla.
//!
//! Un fallo sale como `config rechazada motivo=<m> errores=<n> claves=...`
//! (codigo 3, F1-4): no se reintenta.

mod claves;
mod validar;

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Deserializer};
use serde_json::Value;

pub use claves::{CLAVES_016, DOMINIOS, FASES, MODELOS, TOPE_CLAVES, esquema_json};

use crate::carril::{Carril, Perfil};
use crate::contrato::{Informe, Resultado};
use crate::git::{self, Git, Paso, Subcomando};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::config as t;
use crate::plataforma::Proveedor;

/// Nombre de la operacion en la linea de contrato.
pub const NOMBRE: &str = "config";

/// Ruta de la configuracion en el repo.
pub const RUTA: &str = ".taskcode/config.yml";

/// Rama de la que se lee (F1-10 decision 3).
pub const BASE: &str = "origin/develop";

/// Configuracion validada, con los valores por defecto aplicados.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Siempre 2.
    pub formato: u8,
    /// Endurecimiento por carril (F1-10 decision 2).
    pub politica_carriles: BTreeMap<Carril, PoliticaCarril>,
    /// Globs que suben el carril minimo (solo `fix` y `riesgo-alto`).
    pub rutas_carril: BTreeMap<Carril, Vec<String>>,
    /// Contrato de verificacion (F1-11, F1-7); `None` si no se declara.
    pub verificacion: Option<Verificacion>,
    /// Derivados de las transiciones (F1-11).
    pub derivados: Vec<Derivado>,
    /// Ficheros que suma la huella de confianza (F1-8 decision 5).
    pub confianza_incluye: Vec<String>,
    /// Plataforma de un host propio (F1-13 decision 7).
    pub plataforma: Option<PlataformaConfig>,
    /// Recursos necesarios que anade el proyecto (F1-14 D6a).
    pub recursos: Vec<Recurso>,
    /// Seleccion de skills, agentes y modelo (F1-15).
    pub seleccion: Seleccion,
    /// Modo del flujo.
    pub modo_flujo: ModoFlujo,
    /// Rondas de revision sin preguntar.
    pub tope_rondas: u8,
    /// Presupuesto por defecto de los lotes (F1-16); `None` si no se declara.
    pub lote: Option<Lote>,
    /// Dias sin actividad tras los que otra persona puede liberar o
    /// reasignar una tarea y `status`/`board` la marcan (F3-10, D1a).
    pub dias_sin_actividad: u16,
}

/// `politica_carriles.<carril>`: lo que no se declara es el suelo.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PoliticaCarril {
    /// Roles de diseno minimos.
    pub roles_diseno: Option<u8>,
    /// Solo `Some(true)`.
    pub plan_aprobado: Option<bool>,
    /// Paradas que se anaden (`start`, `review`).
    pub paradas_extra: Vec<String>,
    /// Revisoras que se anaden.
    pub revisoras_extra: Vec<String>,
    /// Solo `Some(false)`.
    pub lote: Option<bool>,
    /// Perfil, nunca por debajo del suelo.
    pub perfil: Option<Perfil>,
}

/// `verificacion` (F1-11, tabla `esquema`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verificacion {
    /// Comando del proyecto.
    pub comando: String,
    /// `cmd`, `bash`, `pwsh` o `sh`.
    pub shell: String,
    /// Perfiles.
    pub perfiles: Perfiles,
    /// Globs de la clave de cache.
    pub huella_arbol: Vec<String>,
    /// Orden que imprime lo que la suite lee fuera del arbol.
    pub huella_local: Option<String>,
    /// Verificacion programada.
    pub programada: Option<Programada>,
}

/// `verificacion.perfiles`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Perfiles {
    /// Atajo local.
    pub rapida: Option<PerfilVerificacion>,
    /// Tests afectados.
    pub impacto: Option<PerfilVerificacion>,
    /// Suite completa (obligatoria).
    pub completa: PerfilVerificacion,
    /// Tests pesados.
    pub pesada: Option<PerfilVerificacion>,
}

/// Un perfil de `verificacion`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerfilVerificacion {
    /// Segundos.
    pub timeout: u32,
    /// Solo en `completa` y `pesada`.
    pub runner: Option<Runner>,
}

/// Runner de la CI (F1-7 decision 5).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Runner {
    /// `{github: windows|linux}`.
    GitHub {
        /// `windows` o `linux`.
        github: String,
    },
    /// `[self-hosted, ...]`.
    Etiquetas(Vec<String>),
}

/// `verificacion.programada`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Programada {
    /// Pasos que dependen de la red.
    pub comando: String,
    /// Cron de 5 campos.
    pub cron: String,
    /// Donde corre.
    pub runner: Runner,
}

/// Un derivado (F1-11).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Derivado {
    /// Que ejecuta.
    pub comando: String,
    /// `genera` o `comprueba`.
    pub tipo: TipoDerivado,
    /// Rutas que puede escribir, normalizadas (vacia en `comprueba`).
    #[serde(default)]
    pub escribe: Vec<String>,
    /// Fases en las que corre.
    pub fases: Vec<String>,
    /// Segundos.
    pub timeout: u16,
}

/// Tipo de un derivado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TipoDerivado {
    /// Escribe ficheros.
    Genera,
    /// Solo comprueba.
    Comprueba,
}

/// `plataforma` (F1-13 decision 7).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlataformaConfig {
    /// GitHub o GitLab.
    #[serde(deserialize_with = "de_proveedor")]
    pub proveedor: Proveedor,
    /// URL de la API.
    pub api: String,
    /// Origen de `revision-ia` (TASK-118): el `app_id` de la App del revisor
    /// (GitHub) o su `user.id` (GitLab). 0 o ausente: no hay.
    #[serde(default)]
    pub revisor_id: Option<u64>,
}

fn de_proveedor<'de, D: Deserializer<'de>>(d: D) -> Result<Proveedor, D::Error> {
    let s = String::deserialize(d)?;
    match s.as_str() {
        "github" => Ok(Proveedor::GitHub),
        "gitlab" => Ok(Proveedor::GitLab),
        otro => Err(serde::de::Error::custom(format!(
            "proveedor desconocido {otro}"
        ))),
    }
}

/// Un recurso declarado (F1-14 D6a).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recurso {
    /// Nombre.
    pub id: String,
    /// `skill`, `agente` o `ejecutable`.
    pub tipo: String,
    /// `revisor`, si lo es.
    pub rol: Option<String>,
    /// Dominios de F1-15 que cubre.
    pub dominios: Vec<String>,
}

/// `seleccion` (F1-15).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seleccion {
    /// Subconjunto de la escala.
    pub modelos_permitidos: Vec<String>,
    /// Senales que se anaden por dominio.
    pub dominios_extra: BTreeMap<String, Senales>,
}

/// Senales de un dominio (F1-15 decision 3).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Senales {
    /// Etiquetas.
    pub etiquetas: Vec<String>,
    /// Palabras.
    pub palabras: Vec<String>,
    /// Globs.
    pub rutas: Vec<String>,
}

/// Modo del flujo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModoFlujo {
    /// Pregunta en cada fase.
    Manual,
    /// Pregunta en algunas.
    Semiautomatico,
    /// Solo las paradas fijas.
    Automatico,
}

impl ModoFlujo {
    /// Nombre en la configuracion y en la columna `modo` de `## Transiciones`.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Semiautomatico => "semiautomatico",
            Self::Automatico => "automatico",
        }
    }
}

/// `lote` (F1-16): sin cifras por defecto.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lote {
    /// Tokens del lote.
    pub presupuesto_tokens: Option<u64>,
    /// Horas del lote.
    pub presupuesto_horas: Option<u64>,
    /// Tokens por tarea.
    pub tope_tokens_tarea: Option<u64>,
}

impl Config {
    /// La configuracion minima (`formato: 2` y nada mas): la politica sin
    /// endurecer. F6 la usa para evaluar el PR que arregla una configuracion
    /// invalida en la base (F1-10, caso abierto; ver [`Invalida`]).
    pub fn suelo() -> Self {
        Self {
            formato: 2,
            politica_carriles: BTreeMap::new(),
            rutas_carril: BTreeMap::new(),
            verificacion: None,
            derivados: Vec::new(),
            confianza_incluye: Vec::new(),
            plataforma: None,
            recursos: Vec::new(),
            seleccion: Seleccion {
                modelos_permitidos: MODELOS.iter().map(|m| (*m).to_owned()).collect(),
                dominios_extra: BTreeMap::new(),
            },
            modo_flujo: ModoFlujo::Manual,
            tope_rondas: 2,
            lote: None,
            dias_sin_actividad: 7,
        }
    }
}

/// Por que se rechaza la configuracion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motivo {
    /// No hay `.taskcode/config.yml` en la base.
    SinConfig,
    /// Falta `---` o `formato: 2` (un proyecto de la 0.16).
    Formato,
    /// `formato` mayor que 2: el proyecto es de un `tc2` mas nuevo.
    FormatoNuevo,
    /// YAML mal formado, clave duplicada, ancla o alias.
    Yaml,
    /// No cumple el esquema.
    ConfigInvalida,
}

impl Motivo {
    /// Nombre en la linea.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::SinConfig => "sin-config",
            Self::Formato => "formato",
            Self::FormatoNuevo => "formato-nuevo",
            Self::Yaml => "yaml",
            Self::ConfigInvalida => "config-invalida",
        }
    }
}

/// Un error con la ruta de su clave (`politica_carriles.fix.aprobaciones`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorConfig {
    /// Ruta de la clave; vacia si el error es del documento entero.
    pub ruta: String,
    /// Que esta mal, como mensaje del catalogo (F2-8).
    pub mensaje: Mensaje,
}

impl ErrorConfig {
    pub(crate) fn nuevo(ruta: &str, mensaje: impl Into<Mensaje>) -> Self {
        Self {
            ruta: ruta.to_owned(),
            mensaje: mensaje.into(),
        }
    }
}

/// Configuracion rechazada, con todos sus errores.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invalida {
    /// Motivo.
    pub motivo: Motivo,
    /// Errores, en el orden en que se encontraron (al menos uno).
    pub errores: Vec<ErrorConfig>,
}

impl Invalida {
    fn una(motivo: Motivo, ruta: &str, mensaje: impl Into<Mensaje>) -> Self {
        Self {
            motivo,
            errores: vec![ErrorConfig::nuevo(ruta, mensaje)],
        }
    }

    /// Informe `config rechazada` (codigo 3) con el primer error y su arreglo
    /// en el diagnostico.
    pub fn informe(&self) -> Informe {
        let mut rutas: Vec<String> = Vec::new();
        for e in &self.errores {
            if !e.ruta.is_empty() && !rutas.contains(&e.ruta) {
                rutas.push(e.ruta.clone());
            }
        }
        let mut i = Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", self.motivo.nombre())
            .con("errores", self.errores.len().to_string());
        if !rutas.is_empty() {
            i = i.con_lista("claves", rutas);
        }
        let diag = match self.errores.first() {
            Some(e) if !e.ruta.is_empty() => Mensaje::nuevo(&t::DIAGNOSTICO_CON_CLAVE)
                .con("clave", e.ruta.clone())
                .con_mensaje("mensaje", e.mensaje.clone()),
            Some(e) => Mensaje::nuevo(&t::DIAGNOSTICO).con_mensaje("mensaje", e.mensaje.clone()),
            None => Mensaje::nuevo(&t::DIAGNOSTICO).con("mensaje", ""),
        };
        let arreglo = match self.motivo {
            Motivo::SinConfig => Mensaje::nuevo(&t::ARREGLO_SIN_CONFIG),
            Motivo::Formato => Mensaje::nuevo(&t::ARREGLO_FORMATO),
            Motivo::FormatoNuevo => Mensaje::nuevo(&t::ARREGLO_FORMATO_NUEVO),
            Motivo::Yaml => Mensaje::nuevo(&t::ARREGLO_YAML).con("ruta", RUTA),
            Motivo::ConfigInvalida
                if self
                    .errores
                    .iter()
                    .any(|e| CLAVES_016.contains(&e.ruta.as_str())) =>
            {
                Mensaje::nuevo(&t::ARREGLO_CLAVES_016)
            }
            Motivo::ConfigInvalida => Mensaje::nuevo(&t::ARREGLO_ESQUEMA).con("ruta", RUTA),
        };
        let diag = match self.errores.len() {
            0 | 1 => diag.con("extra", ""),
            n => diag.con_mensaje(
                "extra",
                Mensaje::nuevo(&t::MAS_ERRORES).con("n", (n - 1).to_string()),
            ),
        };
        i.diagnostico(diag.con("ruta", RUTA).con_mensaje("arreglo", arreglo))
    }
}

/// Texto de la configuracion minima que escribe `tc2 init` (F3-1): solo
/// `formato: 2`, que [`leer`] da como [`Config::suelo`].
pub fn texto_minimo() -> &'static str {
    "---\nformato: 2\n"
}

/// Lee y valida el texto de `.taskcode/config.yml`. Sin E/S.
pub fn leer(texto: &str) -> Result<Config, Invalida> {
    let primera = texto.split('\n').next().unwrap_or("");
    if primera.strip_suffix('\r').unwrap_or(primera) != "---" {
        let que = if primera.starts_with('\u{feff}') {
            &t::PRIMERA_LINEA_SIN_BOM
        } else {
            &t::PRIMERA_LINEA
        };
        return Err(Invalida::una(Motivo::Formato, "", que));
    }
    let raiz: Value =
        serde_saphyr::from_str_with_options(texto, crate::yaml::opciones()).map_err(|e| {
            Invalida::una(
                Motivo::Yaml,
                "",
                Mensaje::nuevo(&t::YAML_NO_VALIDO).con("error", e.to_string()),
            )
        })?;
    let formato = match &raiz {
        Value::Object(m) => m.get("formato"),
        Value::Null => None,
        _ => {
            return Err(Invalida::una(Motivo::ConfigInvalida, "", &t::RAIZ_NO_MAPA));
        }
    };
    match formato {
        Some(f) if f.as_u64() == Some(2) => {}
        Some(f) if f.as_u64().is_some_and(|n| n > 2) => {
            return Err(Invalida::una(
                Motivo::FormatoNuevo,
                "formato",
                Mensaje::nuevo(&t::FORMATO_NUEVO).con("formato", f.to_string()),
            ));
        }
        Some(f) => {
            return Err(Invalida::una(
                Motivo::Formato,
                "formato",
                Mensaje::nuevo(&t::SE_ESPERABA)
                    .con("esperado", "2")
                    .con("valor", f.to_string()),
            ));
        }
        None => {
            return Err(Invalida::una(Motivo::Formato, "formato", &t::FALTA_CLAVE));
        }
    }
    let tabla = claves::Nodo::Objeto(claves::tabla());
    let mut errores = Vec::new();
    validar::validar(&tabla, &raiz, "", &mut errores);
    validar::cruzadas(&raiz, &mut errores);
    if !errores.is_empty() {
        return Err(Invalida {
            motivo: Motivo::ConfigInvalida,
            errores,
        });
    }
    let mut v = raiz;
    validar::aplicar_defectos(&tabla, &mut v);
    normalizar_escribe(&mut v);
    serde_json::from_value(v).map_err(|e| {
        Invalida::una(
            Motivo::ConfigInvalida,
            "",
            Mensaje::nuevo(&t::ERROR_INTERNO).con("error", e.to_string()),
        )
    })
}

/// Las rutas de `escribe` se guardan normalizadas (F1-11: se comparan ya
/// normalizadas).
fn normalizar_escribe(v: &mut Value) {
    let Some(ds) = v.get_mut("derivados").and_then(Value::as_array_mut) else {
        return;
    };
    for d in ds {
        if let Some(es) = d.get_mut("escribe").and_then(Value::as_array_mut) {
            for e in es {
                if let Some(n) = e.as_str().and_then(|t| validar::ruta_tareas(t).ok()) {
                    *e = Value::String(n);
                }
            }
        }
    }
}

/// Lee la configuracion de `origin/develop` (F1-10 decision 3), nunca del
/// worktree. `Err` lleva el informe: `config rechazada` si falta o no
/// vale, o el de la capa git si no se pudo leer.
pub fn de_base(git: &Git, dir: &Path) -> Result<Config, Informe> {
    let texto = texto_de_base(git, dir)?;
    leer(&texto).map_err(|e| e.informe())
}

fn texto_de_base(git: &Git, dir: &Path) -> Paso<String> {
    if git::commit_de(git, dir, NOMBRE, BASE)?.is_none() {
        return Err(Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", "sin-base")
            .diagnostico(Mensaje::nuevo(&t::SIN_BASE).con("base", BASE)));
    }
    // `--full-tree`: `RUTA` es desde la raiz del repo, no desde `dir` (que
    // puede ser una subcarpeta del clon o de un worktree).
    let listado = git::exigir(
        git,
        dir,
        NOMBRE,
        Subcomando::LsTree,
        &[
            "--full-tree",
            "--name-only",
            "--end-of-options",
            BASE,
            "--",
            RUTA,
        ],
    )?;
    if listado.trim() != RUTA {
        return Err(Invalida::una(
            Motivo::SinConfig,
            "",
            Mensaje::nuevo(&t::NO_HAY_CONFIG)
                .con("ruta", RUTA)
                .con("base", BASE),
        )
        .informe());
    }
    git::estado::leer_blob(git, dir, NOMBRE, BASE, RUTA)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_minima_da_los_valores_por_defecto() {
        assert_eq!(leer("---\nformato: 2\n"), Ok(Config::suelo()));
        assert_eq!(leer("---\r\nformato: 2\r\n"), Ok(Config::suelo()));
        assert_eq!(leer(texto_minimo()), Ok(Config::suelo()));
    }
}
