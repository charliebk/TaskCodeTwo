//! Credenciales por rol (ADR F1-6, decisiones 1 y 5; F6-1).
//!
//! Cada operacion del puerto declara con que [`Rol`] corre y pide su token a
//! un proveedor de [`Credenciales`]. Si falta, la operacion sale `rechazada
//! motivo=identidad` y nunca prueba con otro rol.
//!
//! El proveedor de produccion sale de [`de_entorno`] (F6-1): por rol, una
//! GitHub App (`TC2_APP_<ROL>_ID` y `TC2_APP_<ROL>_CLAVE`, ruta al PEM) si
//! esta configurada y, si no, el token de `TC2_TOKEN_<ROL>`. El token de
//! persona solo se usa con un terminal interactivo y la persona no tiene App.

use std::collections::BTreeMap;
use std::io::IsTerminal;
use std::path::PathBuf;

use super::app_github::CredencialesApp;
use super::http::Base;

/// Identidad con la que corre una operacion (F1-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rol {
    /// El agente implementador.
    Agente,
    /// El revisor IA.
    Revisor,
    /// La persona.
    Persona,
}

impl Rol {
    /// Nombre en la linea de contrato.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Agente => "agente",
            Self::Revisor => "revisor",
            Self::Persona => "persona",
        }
    }

    /// Variable de entorno de su token.
    pub fn variable(self) -> &'static str {
        match self {
            Self::Agente => "TC2_TOKEN_AGENTE",
            Self::Revisor => "TC2_TOKEN_REVISOR",
            Self::Persona => "TC2_TOKEN_PERSONA",
        }
    }

    /// Variables de su GitHub App (id y ruta al PEM). La persona no tiene
    /// App: usa su cuenta (F1-6, tabla 2).
    pub fn variables_app(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Agente => Some(("TC2_APP_AGENTE_ID", "TC2_APP_AGENTE_CLAVE")),
            Self::Revisor => Some(("TC2_APP_REVISOR_ID", "TC2_APP_REVISOR_CLAVE")),
            Self::Persona => None,
        }
    }
}

/// Un token. No tiene `Display` y su `Debug` no lo ensena.
#[derive(Clone, PartialEq, Eq)]
pub struct Secreto(String);

impl Secreto {
    /// Envuelve un token.
    pub fn nuevo(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    /// El valor, solo para la cabecera `Authorization` (de la API o de git).
    pub(crate) fn exponer(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Secreto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secreto(***)")
    }
}

/// Por que no hay token de un rol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FalloCredencial {
    /// El rol no tiene identidad configurada (ni App ni token).
    Falta,
    /// Hay App configurada, pero no se pudo acunar su token de instalacion.
    /// `causa` va a la linea (`causa=`) y es una de [`causas`]; `detalle`,
    /// saneado y sin secretos, al diagnostico.
    Acunar {
        /// Causa corta.
        causa: &'static str,
        /// Detalle para la persona.
        detalle: String,
    },
}

/// Valores de `causa` de [`FalloCredencial::Acunar`].
pub mod causas {
    /// La App esta a medias: id sin clave o al reves, o id no valido.
    pub const APP_INCOMPLETA: &str = "app-incompleta";
    /// La App solo existe en GitHub (en GitLab va un token de proyecto o de
    /// cuenta de servicio en `TC2_TOKEN_<ROL>`).
    pub const APP_SOLO_GITHUB: &str = "app-solo-github";
    /// No se pudo leer el PEM o no es una clave RSA que se entienda.
    pub const CLAVE_ILEGIBLE: &str = "clave-ilegible";
    /// GitHub rechaza el JWT de la App (401).
    pub const JWT_RECHAZADO: &str = "jwt-rechazado";
    /// La App no esta instalada en el repo (404).
    pub const SIN_INSTALACION: &str = "sin-instalacion";
    /// La instalacion no concede los permisos pedidos para el rol (422).
    pub const PERMISOS_NO_CONCEDIDOS: &str = "permisos-no-concedidos";
    /// Sin red al pedir el token.
    pub const SINRED: &str = "sinred";
    /// Respuesta con otro estado o con una forma inesperada.
    pub const RESPUESTA_INESPERADA: &str = "respuesta-inesperada";
}

/// Proveedor de tokens por rol.
pub trait Credenciales {
    /// Token de ese rol, si lo hay.
    fn token(&self, rol: Rol) -> Option<Secreto>;

    /// Token de ese rol, o por que no lo hay: una App que no consigue su
    /// token no se confunde con «falta la identidad».
    fn obtener(&self, rol: Rol) -> Result<Secreto, FalloCredencial> {
        self.token(rol).ok_or(FalloCredencial::Falta)
    }

    /// Si el rol tiene identidad configurada, sin hablar con la red (para
    /// elegir con que rol se pregunta; no es un reintento con otro rol).
    fn tiene(&self, rol: Rol) -> bool {
        self.token(rol).is_some()
    }

    /// Ata el proveedor al repo de GitHub `repo` en la API `base`: la App
    /// necesita saber que instalacion pedir. Lo llama el adaptador de
    /// GitHub al crearse; por defecto no hace nada.
    fn atar_github(&mut self, _base: &Base, _repo: &str) {}
}

/// Configuracion de una App leida del entorno.
#[derive(Debug, Clone)]
struct ConfigApp {
    id: String,
    clave: PathBuf,
}

/// Lo que da el entorno para un rol con App: la App ya atada a un repo, la
/// configuracion aun sin atar, o la App a medias.
enum EstadoApp {
    SinAtar(ConfigApp),
    Atada(Box<CredencialesApp>),
    Fallo(FalloCredencial),
}

/// Proveedor de produccion (F6-1): App por rol si esta configurada, si no
/// el token de `TC2_TOKEN_<ROL>`; nunca pasa de un rol a otro. El token de
/// persona solo con terminal (F1-6, decision 5).
pub struct CredencialesDeEntorno {
    tokens: BTreeMap<Rol, Secreto>,
    apps: BTreeMap<Rol, EstadoApp>,
}

impl std::fmt::Debug for CredencialesDeEntorno {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredencialesDeEntorno")
            .field("tokens", &self.tokens.keys().collect::<Vec<_>>())
            .field("apps", &self.apps.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// El proveedor de produccion con las variables y el terminal del proceso.
pub fn de_entorno() -> Box<dyn Credenciales> {
    Box::new(de_entorno_con(
        |v| std::env::var(v).ok(),
        std::io::stdin().is_terminal(),
    ))
}

/// El proveedor con un lector de variables y si hay terminal (los tests no
/// tocan el entorno del proceso: `set_var` exige `unsafe`).
pub fn de_entorno_con(leer: impl Fn(&str) -> Option<String>, tty: bool) -> CredencialesDeEntorno {
    let valor = |v: &str| leer(v).filter(|t| !t.trim().is_empty());
    let mut tokens = BTreeMap::new();
    let mut apps = BTreeMap::new();
    for rol in [Rol::Agente, Rol::Revisor, Rol::Persona] {
        if rol == Rol::Persona && !tty {
            continue;
        }
        if let Some((vid, vclave)) = rol.variables_app() {
            match (valor(vid), valor(vclave)) {
                (None, None) => {}
                (Some(id), Some(clave)) if id_valido(id.trim()) => {
                    apps.insert(
                        rol,
                        EstadoApp::SinAtar(ConfigApp {
                            id: id.trim().to_owned(),
                            clave: PathBuf::from(clave.trim()),
                        }),
                    );
                    continue;
                }
                _ => {
                    apps.insert(
                        rol,
                        EstadoApp::Fallo(FalloCredencial::Acunar {
                            causa: causas::APP_INCOMPLETA,
                            detalle: format!("{vid}, {vclave}"),
                        }),
                    );
                    continue;
                }
            }
        }
        if let Some(t) = valor(rol.variable()) {
            tokens.insert(rol, Secreto::nuevo(t));
        }
    }
    CredencialesDeEntorno { tokens, apps }
}

/// Id de App (numerico) o client id (`Iv1.` y alfanumericos).
pub(crate) fn id_valido(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_')
}

impl Credenciales for CredencialesDeEntorno {
    fn token(&self, rol: Rol) -> Option<Secreto> {
        self.obtener(rol).ok()
    }

    fn obtener(&self, rol: Rol) -> Result<Secreto, FalloCredencial> {
        match self.apps.get(&rol) {
            Some(EstadoApp::Atada(app)) => app.obtener(rol),
            Some(EstadoApp::Fallo(f)) => Err(f.clone()),
            Some(EstadoApp::SinAtar(c)) => Err(FalloCredencial::Acunar {
                causa: causas::APP_SOLO_GITHUB,
                detalle: format!("app {}", c.id),
            }),
            None => self.tokens.get(&rol).cloned().ok_or(FalloCredencial::Falta),
        }
    }

    fn tiene(&self, rol: Rol) -> bool {
        self.apps.contains_key(&rol) || self.tokens.contains_key(&rol)
    }

    fn atar_github(&mut self, base: &Base, repo: &str) {
        for (rol, estado) in &mut self.apps {
            if let EstadoApp::SinAtar(c) = estado {
                *estado = match CredencialesApp::desde_fichero(&c.id, &c.clave, base, repo, *rol) {
                    Ok(app) => EstadoApp::Atada(Box::new(app)),
                    Err(f) => EstadoApp::Fallo(f),
                };
            }
        }
    }
}

/// Tokens fijos (tests y arneses E2E).
#[derive(Debug, Clone, Default)]
pub struct CredencialesFijas(BTreeMap<Rol, Secreto>);

impl CredencialesFijas {
    /// Sin ningun token.
    pub fn nuevas() -> Self {
        Self::default()
    }

    /// Anade el token de un rol.
    #[must_use]
    pub fn con(mut self, rol: Rol, token: Secreto) -> Self {
        self.0.insert(rol, token);
        self
    }
}

impl Credenciales for CredencialesFijas {
    fn token(&self, rol: Rol) -> Option<Secreto> {
        self.0.get(&rol).cloned()
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn el_debug_no_ensena_el_token() {
        let s = Secreto::nuevo("ghp_supersecreto");
        assert_eq!(format!("{s:?}"), "Secreto(***)");
        let c = CredencialesFijas::nuevas().con(Rol::Agente, s);
        assert!(!format!("{c:?}").contains("supersecreto"));
        assert!(c.token(Rol::Persona).is_none());
        assert!(c.token(Rol::Agente).is_some());
        let e = de_entorno_con(
            |v| (v == "TC2_TOKEN_AGENTE").then(|| "ghp_supersecreto".to_owned()),
            false,
        );
        assert!(!format!("{e:?}").contains("supersecreto"));
    }

    fn lector(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let m: BTreeMap<String, String> = vars
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        move |k| m.get(k).cloned()
    }

    /// C2.1 del plan de TASK-047: la App gana al token, sin paso entre
    /// roles, la persona solo con terminal y una App que no acuna no es
    /// «falta».
    #[test]
    fn la_fabrica_da_precedencia_a_la_app_sin_paso_entre_roles() {
        let todo = [
            ("TC2_TOKEN_AGENTE", "tok-agente"),
            ("TC2_TOKEN_PERSONA", "tok-persona"),
            ("TC2_APP_AGENTE_ID", "123"),
            ("TC2_APP_AGENTE_CLAVE", "/no/existe/clave.pem"),
        ];
        // Sin terminal: la persona no tiene token aunque la variable exista.
        let c = de_entorno_con(lector(&todo), false);
        assert!(!c.tiene(Rol::Persona));
        assert_eq!(c.obtener(Rol::Persona), Err(FalloCredencial::Falta));
        // Con terminal, si.
        let c = de_entorno_con(lector(&todo), true);
        assert_eq!(c.token(Rol::Persona), Some(Secreto::nuevo("tok-persona")));
        // El agente tiene App: su token fijo no se usa nunca, y sin atar a
        // un repo de GitHub la App no da token (ni cae al de la variable).
        assert!(c.tiene(Rol::Agente));
        match c.obtener(Rol::Agente) {
            Err(FalloCredencial::Acunar { causa, .. }) => {
                assert_eq!(causa, causas::APP_SOLO_GITHUB);
            }
            otro => panic!("{otro:?}"),
        }
        // Atada, con una clave que no existe: fallo al acunar, no «falta».
        let mut c = de_entorno_con(lector(&todo), true);
        c.atar_github(&Base::github(), "o/r");
        match c.obtener(Rol::Agente) {
            Err(FalloCredencial::Acunar { causa, detalle }) => {
                assert_eq!(causa, causas::CLAVE_ILEGIBLE);
                assert!(!detalle.contains("tok-agente"), "{detalle}");
            }
            otro => panic!("{otro:?}"),
        }
        // El revisor no tiene nada: falta, aunque el agente si tenga.
        assert!(!c.tiene(Rol::Revisor));
        assert_eq!(c.obtener(Rol::Revisor), Err(FalloCredencial::Falta));
        // Sin App, el token del rol; nunca el de otro.
        let c = de_entorno_con(lector(&[("TC2_TOKEN_REVISOR", "tok-rev")]), true);
        assert_eq!(c.token(Rol::Revisor), Some(Secreto::nuevo("tok-rev")));
        assert_eq!(c.obtener(Rol::Agente), Err(FalloCredencial::Falta));
        // App a medias (id sin clave, o id no valido): fallo, no token.
        for vars in [
            vec![("TC2_APP_AGENTE_ID", "123"), ("TC2_TOKEN_AGENTE", "t")],
            vec![
                ("TC2_APP_AGENTE_CLAVE", "/x.pem"),
                ("TC2_TOKEN_AGENTE", "t"),
            ],
            vec![
                ("TC2_APP_AGENTE_ID", "12 3"),
                ("TC2_APP_AGENTE_CLAVE", "/x.pem"),
            ],
        ] {
            let c = de_entorno_con(lector(&vars), true);
            match c.obtener(Rol::Agente) {
                Err(FalloCredencial::Acunar { causa, .. }) => {
                    assert_eq!(causa, causas::APP_INCOMPLETA, "{vars:?}");
                }
                otro => panic!("{vars:?}: {otro:?}"),
            }
        }
        // La persona no tiene App: sus variables de App no existen.
        assert_eq!(Rol::Persona.variables_app(), None);
        // Variables vacias cuentan como ausentes.
        let c = de_entorno_con(lector(&[("TC2_TOKEN_AGENTE", "  ")]), true);
        assert!(!c.tiene(Rol::Agente));
    }
}
