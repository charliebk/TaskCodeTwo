//! Campos del frontmatter (ADR F1-9, punto 3) con esquema estricto (punto 5):
//! formato 2 ([`Tarea`], el que escribe la 2.0) y formato 1 ([`TareaV1`], el
//! de la 0.16, que se lee para migrarlo pero no se escribe).

use serde::Deserialize;
use serde::de::IgnoredAny;

use super::ErrorTarea;
use super::cuerpo::Cuerpo;
use super::yaml::{self, Partes};

/// Valores de `tipo` (los de la 0.16, que mandan en el flujo de ramas, F1-3).
pub const TIPOS: [&str; 4] = ["feature", "fix", "hotfix", "release"];
/// Valores de `complejidad` (los de la 0.16).
pub const COMPLEJIDADES: [&str; 5] = ["trivial", "simple", "media", "alta", "critica"];
/// Valores de `estado` del formato 2 (F1-9: gana `descartada`).
pub const ESTADOS: [&str; 6] = [
    "planificada",
    "en-diseno",
    "en-curso",
    "en-revision",
    "terminada",
    "descartada",
];
/// Valores de `estado` del formato 1 (0.16).
const ESTADOS_V1: [&str; 5] = [
    "planificada",
    "en-diseno",
    "en-curso",
    "en-revision",
    "terminada",
];
/// Valores de la regla de seleccion de skill (0.16; el contenido es de F1-15).
pub const REGLAS: [&str; 3] = ["solape", "prioridad", "llm"];

/// Claves del formato 2 en el orden de la plantilla de F1-9 (punto 6).
pub(crate) const CLAVES: [&str; 21] = [
    "formato",
    "id",
    "titulo",
    "tipo",
    "sprint",
    "etiquetas",
    "complejidad",
    "carril",
    "estado",
    "motivo",
    "plan_aprobado",
    "rama",
    "asignado_a",
    "roles",
    "estimacion",
    "tokens_diseno",
    "tokens_implementacion",
    "tokens_revision",
    "creado",
    "actualizado",
    "dependencias",
];
/// Claves opcionales del formato 2: si faltan valen `null` y solo se
/// escriben con valor, despues de `dependencias` (adenda de TASK-029 a F1-9:
/// una tarea de antes se lee y se reescribe byte a byte igual).
pub(crate) const OPCIONALES: [&str; 1] = ["lote"];

// Sin `zonas` (F1-17) ni `hito` (F1-18) ni el `roles` ampliado de F1-15:
// hasta que los recoja la tarea que los usa, `zonas` y `hito` se conservan
// como claves desconocidas.

/// Claves del formato 1 (`TASK_FIELD_ORDER` del plugin 0.16.1).
pub(crate) const CLAVES_V1: [&str; 22] = [
    "id",
    "titulo",
    "tipo",
    "sprint",
    "etiquetas",
    "complejidad",
    "modelo_sugerido",
    "estado",
    "plan_aprobado",
    "rama",
    "asignado_a",
    "agente_revisor",
    "skills_recomendados",
    "regla_seleccion_skill",
    "ultimo_commit_revisado",
    "revision_codex",
    "tokens_diseno",
    "tokens_implementacion",
    "tokens_revision",
    "creado",
    "actualizado",
    "dependencias",
];

/// Si `titulo` vale: no vacio y sin saltos de linea ni caracteres de control
/// (tampoco los separadores U+2028 y U+2029, que YAML 1.1 lee como salto).
pub fn titulo_valido(titulo: &str) -> bool {
    !titulo.trim().is_empty() && texto_valido(titulo)
}

/// Sin caracteres de control ni separadores de linea.
fn texto_valido(texto: &str) -> bool {
    !texto
        .chars()
        .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
}

/// `TASK-NNN` (al menos tres cifras) o `TASK-NNNa` (parte de un troceo).
pub fn id_valido(id: &str) -> bool {
    let Some(resto) = id.strip_prefix("TASK-") else {
        return false;
    };
    let resto = resto
        .strip_suffix(|c: char| c.is_ascii_lowercase())
        .unwrap_or(resto);
    resto.len() >= 3 && resto.bytes().all(|b| b.is_ascii_digit())
}

/// `AAAA-MM-DD` con cifras.
fn fecha_valida(f: &str) -> bool {
    let b = f.as_bytes();
    b.len() == 10
        && b.iter().enumerate().all(|(i, c)| match i {
            4 | 7 => *c == b'-',
            _ => c.is_ascii_digit(),
        })
}

/// Texto libre de un `asignado_a` de la 0.16 que no se ha convertido a una
/// persona de la plataforma. Solo lo construye el lector: la 2.0 nunca
/// asigna un texto, y al reescribir una tarea lo conserva tal cual hasta que
/// `reasignar` (F3-10) lo resuelva.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heredado(String);

impl Heredado {
    /// El texto tal como se leyo.
    pub fn texto(&self) -> &str {
        &self.0
    }
}

/// Quien lleva la tarea (F2-13): la cuenta de la plataforma por su `id`
/// numerico, con el `login` al lado para ensenarlo, o el texto heredado de la
/// 0.16.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asignado {
    /// Cuenta de la plataforma. Se compara por `id`; `login` es una copia
    /// para mostrar que refresca la lista de personas.
    Persona {
        /// Id numerico de la cuenta.
        id: u64,
        /// Login de la cuenta.
        login: String,
    },
    /// Texto libre de la 0.16 (solo lo crea el lector).
    Heredado(Heredado),
}

impl Asignado {
    /// Cuenta `id` con `login`; `None` si el id es 0 o el login esta vacio o
    /// lleva espacios o caracteres de control.
    pub fn persona(id: u64, login: &str) -> Option<Self> {
        (id > 0 && login_valido(login)).then(|| Self::Persona {
            id,
            login: login.to_owned(),
        })
    }

    /// Id de la cuenta, si es una persona.
    pub fn id(&self) -> Option<u64> {
        match self {
            Self::Persona { id, .. } => Some(*id),
            Self::Heredado(_) => None,
        }
    }

    /// Login de la cuenta, si es una persona.
    pub fn login(&self) -> Option<&str> {
        match self {
            Self::Persona { login, .. } => Some(login),
            Self::Heredado(_) => None,
        }
    }

    /// Texto heredado de la 0.16, si lo es.
    pub fn heredado(&self) -> Option<&str> {
        match self {
            Self::Persona { .. } => None,
            Self::Heredado(h) => Some(h.texto()),
        }
    }

    /// «Ya la lleva»: la cuenta `id` es la asignada. Compara solo el `id`: el
    /// mismo login con otro id es otra persona y un login renombrado con el
    /// mismo id es la misma. Un texto heredado no lo lleva nadie.
    pub fn la_lleva(&self, id: u64) -> bool {
        self.id() == Some(id)
    }

    /// Si `otro` es la misma persona (mismo `id`).
    pub fn misma_persona(&self, otro: &Self) -> bool {
        otro.id().is_some_and(|id| self.la_lleva(id))
    }

    /// Lo que se escribe en el frontmatter.
    fn yaml(&self) -> String {
        match self {
            Self::Persona { id, login } => {
                format!("{{id: {id}, login: {}}}", yaml::cadena(login))
            }
            Self::Heredado(h) => yaml::cadena(h.texto()),
        }
    }
}

/// Longitud maxima de un login, en caracteres (GitLab admite 255; GitHub,
/// 39).
pub const LOGIN_MAX: usize = 255;

/// Login de una cuenta: no vacio, de [`LOGIN_MAX`] caracteres como mucho, sin
/// espacios ni caracteres de control.
pub fn login_valido(login: &str) -> bool {
    !login.is_empty()
        && login.chars().count() <= LOGIN_MAX
        && texto_valido(login)
        && !login.chars().any(char::is_whitespace)
}

/// `asignado_a` del formato 2: `null`, `{id, login}` o un texto heredado.
fn asignado(v: Option<serde_json::Value>) -> Result<Option<Asignado>, ErrorTarea> {
    use serde_json::Value;
    const CLAVE: &str = "asignado_a";
    match v {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) if !s.trim().is_empty() => {
            Ok(Some(Asignado::Heredado(Heredado(texto(CLAVE, s)?))))
        }
        Some(Value::Object(m)) => {
            if let Some(k) = m.keys().find(|k| *k != "id" && *k != "login") {
                return Err(ErrorTarea::en(
                    CLAVE,
                    format!("clave «{k}» desconocida (se espera {{id, login}})"),
                ));
            }
            let id = m.get("id").and_then(Value::as_u64);
            let login = m.get("login").and_then(Value::as_str);
            match (id, login) {
                (Some(id), Some(login)) => Asignado::persona(id, login).ok_or_else(|| {
                    ErrorTarea::en(
                        CLAVE,
                        "`id` tiene que ser mayor que 0 y `login` sin espacios ni caracteres de control",
                    )
                }),
                _ => Err(ErrorTarea::en(
                    CLAVE,
                    "se espera {id: <entero sin comillas>, login: \"<login>\"}",
                )),
            }
            .map(Some)
        }
        Some(_) => Err(ErrorTarea::en(
            CLAVE,
            "se espera null, {id, login} o el texto de la 0.16",
        )),
    }
}

/// Un rol de `roles` (diseno, implementacion o revision). `agente` y
/// `ronda` son solo de revision: en diseno e implementacion se rechazan al
/// leer y no se escriben.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rol {
    /// Agente que revisa (solo revision).
    pub agente: Option<String>,
    /// Modelo del rol (contenido de F1-15).
    pub modelo: Option<String>,
    /// Skills del rol.
    pub skills: Vec<String>,
    /// Regla que eligio la skill (vocabulario de la 0.16).
    pub regla: Option<String>,
    /// Agentes elegidos (F1-15 decision 10; F4-6).
    #[serde(default)]
    pub agentes: Vec<String>,
    /// Ids de las filas de las tablas de F1-15 que se aplicaron.
    #[serde(default)]
    pub reglas: Vec<String>,
    /// Senales de dominio (`<dominio>:<senal>:<valor>`); en diseno son las
    /// congeladas con el plan aprobado.
    #[serde(default)]
    pub senales: Vec<String>,
    /// Avisos de la seleccion.
    #[serde(default)]
    pub avisos: Vec<String>,
    /// Huella del inventario con el que se calculo.
    pub inventario: Option<String>,
    /// Huella del catalogo con el que se calculo.
    pub catalogo: Option<String>,
    /// Ronda de revision en la que se calculo (solo revision).
    pub ronda: Option<u32>,
}

/// El mapa `roles` (RR-17). Un campo que no conoce el modelo se rechaza: al
/// reescribirlo se perderia.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Roles {
    /// Diseno.
    pub diseno: Rol,
    /// Implementacion.
    pub implementacion: Rol,
    /// Revision.
    pub revision: Rol,
}

/// Tarea en formato 2 (F1-9, punto 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tarea {
    pub id: String,
    pub titulo: String,
    pub tipo: String,
    pub sprint: u32,
    pub etiquetas: Vec<String>,
    pub complejidad: Option<String>,
    pub carril: Option<String>,
    pub estado: String,
    pub motivo: Option<String>,
    pub plan_aprobado: bool,
    pub rama: Option<String>,
    pub asignado_a: Option<Asignado>,
    pub roles: Roles,
    /// Entrada `estimacion:` cruda (su forma la fija F1-17); `None` es `null`.
    /// Privada: solo llega de una lectura, que la ha validado como YAML, o de
    /// [`Tarea::poner_estimacion`].
    estimacion: Option<String>,
    pub tokens_diseno: Option<u64>,
    pub tokens_implementacion: Option<u64>,
    pub tokens_revision: Option<u64>,
    pub creado: String,
    pub actualizado: String,
    pub dependencias: Vec<String>,
    /// `lote/clave` de la tarea importada que la creo (F3-2, D1 de TASK-029):
    /// por ella un import repetido la reconoce. Opcional.
    pub lote: Option<String>,
    /// Claves desconocidas con su texto crudo, en su orden (F1-9 D4a: se
    /// conservan en la ida y vuelta; `doctor` avisa). Privadas: solo llegan
    /// de una lectura, que las ha validado y comprobado sin duplicadas.
    desconocidas: Vec<(String, String)>,
}

#[derive(Deserialize)]
struct CrudoV2 {
    formato: Option<u32>,
    id: Option<String>,
    titulo: Option<String>,
    tipo: Option<String>,
    sprint: Option<u32>,
    etiquetas: Option<Vec<String>>,
    complejidad: Option<String>,
    carril: Option<String>,
    estado: Option<String>,
    motivo: Option<String>,
    plan_aprobado: Option<bool>,
    rama: Option<String>,
    asignado_a: Option<serde_json::Value>,
    roles: Option<Roles>,
    estimacion: Option<IgnoredAny>,
    tokens_diseno: Option<u64>,
    tokens_implementacion: Option<u64>,
    tokens_revision: Option<u64>,
    creado: Option<String>,
    actualizado: Option<String>,
    dependencias: Option<Vec<String>>,
    lote: Option<String>,
}

/// Valor obligatorio (presente y no `null`).
fn exigir<T>(clave: &str, v: Option<T>) -> Result<T, ErrorTarea> {
    v.ok_or_else(|| ErrorTarea::en(clave, "falta o es null"))
}

/// Texto sin caracteres de control.
fn texto(clave: &str, s: String) -> Result<String, ErrorTarea> {
    if texto_valido(&s) {
        Ok(s)
    } else {
        Err(ErrorTarea::en(clave, "lleva caracteres de control"))
    }
}

fn texto_opcional(clave: &str, s: Option<String>) -> Result<Option<String>, ErrorTarea> {
    s.map(|s| texto(clave, s)).transpose()
}

fn textos(clave: &str, v: Vec<String>) -> Result<Vec<String>, ErrorTarea> {
    v.into_iter().map(|s| texto(clave, s)).collect()
}

/// Valor de una lista cerrada.
fn uno_de(clave: &str, s: String, permitidos: &[&str]) -> Result<String, ErrorTarea> {
    if permitidos.contains(&s.as_str()) {
        Ok(s)
    } else {
        Err(ErrorTarea::en(
            clave,
            format!("«{s}» no es uno de: {}", permitidos.join(", ")),
        ))
    }
}

fn uno_de_opcional(
    clave: &str,
    s: Option<String>,
    permitidos: &[&str],
) -> Result<Option<String>, ErrorTarea> {
    s.map(|s| uno_de(clave, s, permitidos)).transpose()
}

fn id(clave: &str, s: String) -> Result<String, ErrorTarea> {
    if id_valido(&s) {
        Ok(s)
    } else {
        Err(ErrorTarea::en(clave, format!("«{s}» no es un ID TASK-NNN")))
    }
}

/// `lote`: `<lote>/<clave>`, sin caracteres de control, o ausente/`null`.
fn lote(s: Option<String>) -> Result<Option<String>, ErrorTarea> {
    match s {
        None => Ok(None),
        Some(s)
            if texto_valido(&s)
                && s.split_once('/')
                    .is_some_and(|(a, b)| !a.is_empty() && !b.is_empty()) =>
        {
            Ok(Some(s))
        }
        Some(s) => Err(ErrorTarea::en(
            "lote",
            format!("«{s}» no es <lote>/<clave>"),
        )),
    }
}

fn fecha(clave: &str, s: String) -> Result<String, ErrorTarea> {
    if fecha_valida(&s) {
        Ok(s)
    } else {
        Err(ErrorTarea::en(
            clave,
            format!("«{s}» no es una fecha AAAA-MM-DD"),
        ))
    }
}

/// `roles.<rol>` validado; `revision` dice si admite `agente` y `ronda`.
fn rol(clave: &str, r: Rol, revision: bool) -> Result<Rol, ErrorTarea> {
    for (k, hay) in [("agente", r.agente.is_some()), ("ronda", r.ronda.is_some())] {
        if hay && !revision {
            return Err(ErrorTarea::en(clave, format!("clave «{k}» desconocida")));
        }
    }
    Ok(Rol {
        agente: texto_opcional(clave, r.agente)?,
        modelo: texto_opcional(clave, r.modelo)?,
        skills: textos(clave, r.skills)?,
        regla: uno_de_opcional(clave, r.regla, &REGLAS)?,
        agentes: textos(clave, r.agentes)?,
        reglas: textos(clave, r.reglas)?,
        senales: textos(clave, r.senales)?,
        avisos: textos(clave, r.avisos)?,
        inventario: texto_opcional(clave, r.inventario)?,
        catalogo: texto_opcional(clave, r.catalogo)?,
        ronda: r.ronda,
    })
}

/// Las claves ampliadas de F4-6, solo las que tienen valor (una tarea de
/// antes se reescribe byte a byte igual).
fn ampliado(r: &Rol) -> String {
    let mut s = String::new();
    for (k, v) in [
        ("agentes", &r.agentes),
        ("reglas", &r.reglas),
        ("senales", &r.senales),
        ("avisos", &r.avisos),
    ] {
        if !v.is_empty() {
            s.push_str(&format!(", {k}: {}", yaml::lista(v)));
        }
    }
    for (k, v) in [("inventario", &r.inventario), ("catalogo", &r.catalogo)] {
        if let Some(v) = v {
            s.push_str(&format!(", {k}: {}", yaml::cadena(v)));
        }
    }
    s
}

/// Claves que faltan de `claves` en `partes` (todas son obligatorias).
fn faltan(partes: &Partes<'_>, claves: &[&str]) -> Result<(), ErrorTarea> {
    match claves.iter().find(|c| !partes.tiene(c)) {
        Some(c) => Err(ErrorTarea::en(c, "falta")),
        None => Ok(()),
    }
}

/// Entradas de `partes` cuya clave no esta en `claves`.
fn desconocidas(partes: &Partes<'_>, claves: &[&str]) -> Vec<(String, String)> {
    partes
        .entradas
        .iter()
        .filter(|e| !claves.contains(&e.clave.as_str()))
        .map(|e| (e.clave.clone(), e.crudo.clone()))
        .collect()
}

impl Tarea {
    /// Tarea recien creada con los valores de la plantilla de F1-9.
    pub fn nueva(id: &str, titulo: &str, fecha: &str) -> Self {
        Self {
            id: id.to_owned(),
            titulo: titulo.to_owned(),
            tipo: "feature".to_owned(),
            sprint: 0,
            etiquetas: Vec::new(),
            complejidad: None,
            carril: None,
            estado: "planificada".to_owned(),
            motivo: None,
            plan_aprobado: false,
            // La pone el alta (F2-4b) derivada del ID y del titulo.
            rama: None,
            asignado_a: None,
            roles: Roles {
                diseno: Rol::default(),
                implementacion: Rol {
                    modelo: Some("sonnet".to_owned()),
                    ..Rol::default()
                },
                revision: Rol {
                    agente: Some("general-purpose".to_owned()),
                    ..Rol::default()
                },
            },
            estimacion: None,
            tokens_diseno: None,
            tokens_implementacion: None,
            tokens_revision: None,
            creado: fecha.to_owned(),
            actualizado: fecha.to_owned(),
            dependencias: Vec::new(),
            lote: None,
            desconocidas: Vec::new(),
        }
    }

    /// Claves desconocidas conservadas (para el aviso de `doctor`).
    pub fn claves_desconocidas(&self) -> &[(String, String)] {
        &self.desconocidas
    }

    /// Entrada `estimacion:` cruda tal como se leyo; `None` si es `null`.
    pub fn estimacion(&self) -> Option<&str> {
        self.estimacion.as_deref()
    }

    /// Pone la estimacion con la forma del punto 7 de F1-17 (F3-13): el
    /// unico camino de escritura del campo. Lo escrito se vuelve a leer igual
    /// (`estimacion` en bloque, YAML valido).
    pub fn poner_estimacion(&mut self, e: &crate::estimacion::Estimacion) {
        self.estimacion = Some(e.yaml());
    }

    /// Lee el frontmatter de formato 2 con el esquema estricto.
    pub(crate) fn leer(partes: &Partes<'_>) -> Result<Self, ErrorTarea> {
        faltan(partes, &CLAVES)?;
        canonicos(
            partes,
            &[
                ("formato", Canon::Entero),
                ("sprint", Canon::Entero),
                ("plan_aprobado", Canon::Bool),
                ("tokens_diseno", Canon::EnteroONull),
                ("tokens_implementacion", Canon::EnteroONull),
                ("tokens_revision", Canon::EnteroONull),
            ],
        )?;
        let c: CrudoV2 = yaml::decodificar(&partes.yaml())?;
        let formato = exigir("formato", c.formato)?;
        if formato != 2 {
            return Err(ErrorTarea::en(
                "formato",
                format!(
                    "formato {formato} no soportado (se lee el 2, o sin `formato` el 1 de la 0.16)"
                ),
            ));
        }
        let titulo = exigir("titulo", c.titulo)?;
        if !titulo_valido(&titulo) {
            return Err(ErrorTarea::en(
                "titulo",
                "vacio o con caracteres de control",
            ));
        }
        let estado = uno_de("estado", exigir("estado", c.estado)?, &ESTADOS)?;
        let motivo = texto_opcional("motivo", c.motivo)?;
        if estado == "descartada" && motivo.as_deref().is_none_or(|m| m.trim().is_empty()) {
            return Err(ErrorTarea::en(
                "motivo",
                "obligatorio con `estado: descartada`",
            ));
        }
        let roles = exigir("roles", c.roles)?;
        let estimacion = c.estimacion.and_then(|_| {
            partes
                .entradas
                .iter()
                .find(|e| e.clave == "estimacion")
                .map(|e| e.crudo.clone())
        });
        Ok(Self {
            id: id("id", exigir("id", c.id)?)?,
            titulo,
            tipo: uno_de("tipo", exigir("tipo", c.tipo)?, &TIPOS)?,
            sprint: exigir("sprint", c.sprint)?,
            etiquetas: textos("etiquetas", exigir("etiquetas", c.etiquetas)?)?,
            complejidad: uno_de_opcional("complejidad", c.complejidad, &COMPLEJIDADES)?,
            carril: texto_opcional("carril", c.carril)?,
            estado,
            motivo,
            plan_aprobado: exigir("plan_aprobado", c.plan_aprobado)?,
            rama: texto_opcional("rama", c.rama)?,
            asignado_a: asignado(c.asignado_a)?,
            roles: Roles {
                diseno: rol("roles.diseno", roles.diseno, false)?,
                implementacion: rol("roles.implementacion", roles.implementacion, false)?,
                revision: rol("roles.revision", roles.revision, true)?,
            },
            estimacion,
            tokens_diseno: c.tokens_diseno,
            tokens_implementacion: c.tokens_implementacion,
            tokens_revision: c.tokens_revision,
            creado: fecha("creado", exigir("creado", c.creado)?)?,
            actualizado: fecha("actualizado", exigir("actualizado", c.actualizado)?)?,
            dependencias: exigir("dependencias", c.dependencias)?
                .into_iter()
                .map(|d| id("dependencias", d))
                .collect::<Result<_, _>>()?,
            lote: lote(c.lote)?,
            desconocidas: desconocidas(partes, &[&CLAVES[..], &OPCIONALES[..]].concat()),
        })
    }

    /// Frontmatter en la forma fija de F1-9 seguido de `cuerpo`. Las claves
    /// desconocidas van al final, en su orden y con su texto original.
    /// No valida: un texto con caracteres de control sale escapado (YAML
    /// valido) pero [`super::leer`] lo rechaza. Lo que no conserva: los
    /// comentarios del frontmatter, el BOM y los `\r\n` del frontmatter (sale
    /// con `\n`; el cuerpo, byte a byte como se leyo).
    pub fn escribir(&self, cuerpo: &Cuerpo) -> String {
        let r = &self.roles;
        // Revision lleva `agente` delante; `ronda` solo sale con valor.
        let rol = |x: &Rol, revision: bool| {
            let agente = if revision {
                format!("agente: {}, ", yaml::opcional(x.agente.as_deref()))
            } else {
                String::new()
            };
            let ronda = x.ronda.map(|n| format!(", ronda: {n}")).unwrap_or_default();
            format!(
                "{{{agente}modelo: {}, skills: {}, regla: {}{}{ronda}}}",
                yaml::opcional(x.modelo.as_deref()),
                yaml::lista(&x.skills),
                yaml::opcional(x.regla.as_deref()),
                ampliado(x)
            )
        };
        let mut s = String::from("---\nformato: 2\n");
        let mut linea = |clave: &str, valor: String| {
            s.push_str(clave);
            s.push_str(": ");
            s.push_str(&valor);
            s.push('\n');
        };
        linea("id", yaml::cadena(&self.id));
        linea("titulo", yaml::cadena(&self.titulo));
        linea("tipo", yaml::cadena(&self.tipo));
        linea("sprint", self.sprint.to_string());
        linea("etiquetas", yaml::lista(&self.etiquetas));
        linea("complejidad", yaml::opcional(self.complejidad.as_deref()));
        linea("carril", yaml::opcional(self.carril.as_deref()));
        linea("estado", yaml::cadena(&self.estado));
        linea("motivo", yaml::opcional(self.motivo.as_deref()));
        linea("plan_aprobado", self.plan_aprobado.to_string());
        linea("rama", yaml::opcional(self.rama.as_deref()));
        linea(
            "asignado_a",
            self.asignado_a
                .as_ref()
                .map_or_else(|| "null".to_owned(), Asignado::yaml),
        );
        let roles = format!(
            "\n  diseno: {}\n  implementacion: {}\n  revision: {}",
            rol(&r.diseno, false),
            rol(&r.implementacion, false),
            rol(&r.revision, true)
        );
        s.push_str("roles:");
        s.push_str(&roles);
        s.push('\n');
        match &self.estimacion {
            Some(crudo) => s.push_str(crudo),
            None => s.push_str("estimacion: null\n"),
        }
        let mut linea = |clave: &str, valor: String| {
            s.push_str(clave);
            s.push_str(": ");
            s.push_str(&valor);
            s.push('\n');
        };
        linea("tokens_diseno", yaml::numero(self.tokens_diseno));
        linea(
            "tokens_implementacion",
            yaml::numero(self.tokens_implementacion),
        );
        linea("tokens_revision", yaml::numero(self.tokens_revision));
        linea("creado", yaml::cadena(&self.creado));
        linea("actualizado", yaml::cadena(&self.actualizado));
        linea("dependencias", yaml::lista(&self.dependencias));
        if let Some(l) = &self.lote {
            linea("lote", yaml::cadena(l));
        }
        for (_, crudo) in &self.desconocidas {
            s.push_str(crudo);
        }
        s.push_str("---\n");
        s.push_str(&cuerpo.escribir());
        s
    }
}

/// Tarea en formato 1, el de la 0.16 (los 22 campos de `TASK_FIELD_ORDER`).
/// Se lee para migrarla (F1-12); no se escribe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TareaV1 {
    pub id: String,
    pub titulo: String,
    pub tipo: String,
    pub sprint: u32,
    pub etiquetas: Vec<String>,
    pub complejidad: Option<String>,
    pub modelo_sugerido: String,
    pub estado: String,
    pub plan_aprobado: bool,
    pub rama: String,
    /// El texto libre de la 0.16, siempre [`Asignado::Heredado`].
    pub asignado_a: Option<Asignado>,
    pub agente_revisor: String,
    pub skills_recomendados: Vec<String>,
    pub regla_seleccion_skill: Option<String>,
    pub ultimo_commit_revisado: Option<String>,
    pub revision_codex: bool,
    pub tokens_diseno: Option<u64>,
    pub tokens_implementacion: Option<u64>,
    pub tokens_revision: Option<u64>,
    pub creado: String,
    pub actualizado: String,
    pub dependencias: Vec<String>,
    /// Claves desconocidas con su texto crudo, en su orden.
    pub desconocidas: Vec<(String, String)>,
}

#[derive(Deserialize)]
struct CrudoV1 {
    id: Option<String>,
    titulo: Option<String>,
    tipo: Option<String>,
    sprint: Option<u32>,
    etiquetas: Option<Vec<String>>,
    complejidad: Option<String>,
    modelo_sugerido: Option<String>,
    estado: Option<String>,
    plan_aprobado: Option<bool>,
    rama: Option<String>,
    asignado_a: Option<String>,
    agente_revisor: Option<String>,
    skills_recomendados: Option<Vec<String>>,
    regla_seleccion_skill: Option<String>,
    ultimo_commit_revisado: Option<String>,
    revision_codex: Option<bool>,
    tokens_diseno: Option<u64>,
    tokens_implementacion: Option<u64>,
    tokens_revision: Option<u64>,
    creado: Option<String>,
    actualizado: Option<String>,
    dependencias: Option<Vec<String>>,
}

/// La 0.16 escapa `"` pero no `\` (H11): un `\` que no va seguido de `"`
/// lo leeria mal un parser estandar (`C:\ruta` daria un retorno de carro).
/// Mejor un error claro que un valor cambiado en silencio.
/// Solo mira dentro de cadenas entre comillas dobles: en una cadena plana,
/// de comilla simple o en un comentario la barra es literal tambien para un
/// parser estandar.
fn escape_ambiguo(partes: &Partes<'_>) -> Result<(), ErrorTarea> {
    for e in partes
        .entradas
        .iter()
        .filter(|e| CLAVES_V1.contains(&e.clave.as_str()))
    {
        let mut cs = e.crudo.chars().peekable();
        let mut dobles = false;
        while let Some(c) = cs.next() {
            match (dobles, c) {
                (true, '\\') if cs.next_if_eq(&'"').is_none() => {
                    return Err(ErrorTarea::en(
                        &e.clave,
                        "lleva `\\` con el escape de la 0.16, que un parser YAML estandar lee distinto (H11)",
                    ));
                }
                (true, '"') | (false, '"') => dobles = !dobles,
                (false, '\'') => {
                    // Comilla simple: hasta la siguiente (`''` es una comilla).
                    while cs.next().is_some_and(|c| c != '\'') {}
                }
                (false, '#') => {
                    // Comentario: hasta el final de la linea.
                    while cs.next().is_some_and(|c| c != '\n') {}
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// Grafia que se exige al texto crudo de un campo no textual.
#[derive(Clone, Copy)]
enum Canon {
    /// `true` o `false`.
    Bool,
    /// `0` o `[1-9][0-9]*`.
    Entero,
    /// Un [`Canon::Entero`] o `null`.
    EnteroONull,
}

/// Los campos bool y enteros llevan su grafia canonica, sin comillas: un
/// `'7'` o un `"false"` serian cadenas para un parser estandar y el crate los
/// convertiria (F1-9 punto 5: tipo incorrecto da error).
fn canonicos(partes: &Partes<'_>, campos: &[(&str, Canon)]) -> Result<(), ErrorTarea> {
    let entero = |v: &str| {
        v == "0" || (!v.starts_with('0') && !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
    };
    for (clave, canon) in campos {
        let Some(v) = partes.valor_plano(clave) else {
            // Falta (lo dice `faltan`/`exigir`) o ocupa varias lineas.
            if partes.tiene(clave) {
                return Err(ErrorTarea::en(clave, "tiene que ir en una sola linea"));
            }
            continue;
        };
        let vale = match canon {
            Canon::Bool => v == "true" || v == "false",
            Canon::Entero => entero(v),
            Canon::EnteroONull => v == "null" || entero(v),
        };
        if !vale {
            let esperado = match canon {
                Canon::Bool => "`true` o `false`",
                Canon::Entero => "un entero sin comillas",
                Canon::EnteroONull => "un entero sin comillas o `null`",
            };
            return Err(ErrorTarea::en(clave, format!("«{v}» no es {esperado}")));
        }
    }
    Ok(())
}

impl TareaV1 {
    /// Lee el frontmatter de formato 1 con las reglas de la 0.16
    /// (`validateTask`): obligatorios los que la 0.16 exige.
    pub(crate) fn leer(partes: &Partes<'_>) -> Result<Self, ErrorTarea> {
        escape_ambiguo(partes)?;
        canonicos(
            partes,
            &[
                ("sprint", Canon::Entero),
                ("plan_aprobado", Canon::Bool),
                ("revision_codex", Canon::Bool),
                ("tokens_diseno", Canon::EnteroONull),
                ("tokens_implementacion", Canon::EnteroONull),
                ("tokens_revision", Canon::EnteroONull),
            ],
        )?;
        let c: CrudoV1 = yaml::decodificar(&partes.yaml())?;
        Ok(Self {
            id: id("id", exigir("id", c.id)?)?,
            titulo: texto("titulo", exigir("titulo", c.titulo)?)?,
            tipo: uno_de("tipo", exigir("tipo", c.tipo)?, &TIPOS)?,
            sprint: exigir("sprint", c.sprint)?,
            etiquetas: textos("etiquetas", exigir("etiquetas", c.etiquetas)?)?,
            complejidad: uno_de_opcional("complejidad", c.complejidad, &COMPLEJIDADES)?,
            modelo_sugerido: texto(
                "modelo_sugerido",
                exigir("modelo_sugerido", c.modelo_sugerido)?,
            )?,
            estado: uno_de("estado", exigir("estado", c.estado)?, &ESTADOS_V1)?,
            plan_aprobado: exigir("plan_aprobado", c.plan_aprobado)?,
            rama: texto("rama", exigir("rama", c.rama)?)?,
            asignado_a: texto_opcional("asignado_a", c.asignado_a)?
                .filter(|s| !s.trim().is_empty())
                .map(|s| Asignado::Heredado(Heredado(s))),
            agente_revisor: texto(
                "agente_revisor",
                exigir("agente_revisor", c.agente_revisor)?,
            )?,
            skills_recomendados: textos(
                "skills_recomendados",
                exigir("skills_recomendados", c.skills_recomendados)?,
            )?,
            regla_seleccion_skill: uno_de_opcional(
                "regla_seleccion_skill",
                c.regla_seleccion_skill,
                &REGLAS,
            )?,
            ultimo_commit_revisado: texto_opcional(
                "ultimo_commit_revisado",
                c.ultimo_commit_revisado,
            )?,
            revision_codex: exigir("revision_codex", c.revision_codex)?,
            tokens_diseno: c.tokens_diseno,
            tokens_implementacion: c.tokens_implementacion,
            tokens_revision: c.tokens_revision,
            creado: texto("creado", exigir("creado", c.creado)?)?,
            actualizado: texto("actualizado", exigir("actualizado", c.actualizado)?)?,
            dependencias: textos("dependencias", exigir("dependencias", c.dependencias)?)?,
            desconocidas: desconocidas(partes, &CLAVES_V1),
        })
    }
}
