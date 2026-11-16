//! Lista de personas (F1-6, decision 3; F2-13): el fichero
//! `.taskcode/personas.yml` de la rama de codigo, bajo CODEOWNERS.
//!
//! Cada entrada guarda el `id` numerico de la cuenta (lo que se compara) y su
//! `login` (lo que se ensena). `nombre` es opcional y solo acompana al login;
//! `correos` son los textos libres de `asignado_a` de la 0.16 que se
//! convierten a esta persona (migracion, F2-13 D5); `claves_ssh` y
//! `claves_gpg`, los ids de sus claves de firma en GitLab (F1-13, decision
//! 6). Aqui solo se lee y se valida el texto: sin E/S.

use std::collections::BTreeSet;

use serde::Deserialize;

use crate::contrato::{Informe, Resultado};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::identidad as t;

/// Ruta de la lista en la rama de codigo.
pub const RUTA: &str = ".taskcode/personas.yml";

/// Operacion en los errores de lectura.
const OP: &str = "personas";

/// Clave de firma registrada en la plataforma, con su tipo: los ids de las
/// claves SSH y GPG de GitLab son espacios distintos y no se comparan entre
/// si.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Clave {
    /// Clave SSH (`key.id`).
    Ssh(u64),
    /// Clave GPG (`gpg_key_id`).
    Gpg(u64),
}

/// Una persona de la lista.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Persona {
    /// Id numerico de la cuenta: el que se compara.
    pub id: u64,
    /// Login de la cuenta: el que se ensena.
    pub login: String,
    /// Nombre para acompanar al login; nunca lo sustituye.
    pub nombre: Option<String>,
    /// Textos de `asignado_a` de la 0.16 que son esta persona.
    pub correos: Vec<String>,
}

impl Persona {
    /// Como se ensena: siempre el login, y el nombre al lado si lo hay.
    pub fn mostrar(&self) -> String {
        match &self.nombre {
            Some(n) => format!("{} ({n})", self.login),
            None => self.login.clone(),
        }
    }
}

/// Lista de personas por id numerico (F1-6, decision 3) y, para GitLab, los
/// ids de sus claves de firma (F1-13, decision 6: la firma de GitLab no trae
/// el usuario).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Personas {
    ids: BTreeSet<u64>,
    claves: BTreeSet<Clave>,
    entradas: Vec<Persona>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ListaCruda {
    personas: Vec<EntradaCruda>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EntradaCruda {
    id: u64,
    login: String,
    #[serde(default)]
    nombre: Option<String>,
    #[serde(default)]
    correos: Vec<String>,
    #[serde(default)]
    claves_ssh: Vec<u64>,
    #[serde(default)]
    claves_gpg: Vec<u64>,
}

/// Sin espacios ni caracteres de control, y no vacio.
fn palabra_valida(s: &str) -> bool {
    !s.is_empty()
        && !s
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || matches!(c, '\u{2028}' | '\u{2029}'))
}

/// Rechazo de la lista con su diagnostico.
fn rechazo(m: Mensaje) -> Informe {
    Informe::nuevo(OP, Resultado::Rechazada)
        .con("motivo", "personas")
        .diagnostico(m)
}

/// Una entrada para los diagnosticos: su numero (desde 1), login e id.
fn cita(n: usize, p: &Persona) -> String {
    format!("{n} ({}, id {})", p.login, p.id)
}

impl Personas {
    /// Lista con estos ids (sin login: para tests y arneses que solo
    /// comparan ids).
    pub fn nuevas(ids: impl IntoIterator<Item = u64>) -> Self {
        Self {
            ids: ids.into_iter().collect(),
            claves: BTreeSet::new(),
            entradas: Vec::new(),
        }
    }

    /// La misma lista con estas claves de firma.
    #[must_use]
    pub fn con_claves(mut self, claves: impl IntoIterator<Item = Clave>) -> Self {
        self.claves = claves.into_iter().collect();
        self
    }

    /// Lee y valida el texto de `.taskcode/personas.yml`: `id` mayor que 0 y
    /// unico, `login` unico sin distinguir mayusculas, `correos` unicos. Un
    /// repetido se rechaza nombrando las dos entradas.
    pub fn leer(texto: &str) -> Result<Self, Informe> {
        let cruda: ListaCruda = serde_saphyr::from_str_with_options(texto, crate::yaml::opciones())
            .map_err(|e| {
                rechazo(
                    Mensaje::nuevo(&t::PERSONAS_YAML)
                        .con("ruta", RUTA)
                        .con("error", e.to_string()),
                )
            })?;
        let mut lista = Self::default();
        for (i, e) in cruda.personas.into_iter().enumerate() {
            let n = i + 1;
            let entrada = |que: &'static crate::i18n::Texto| {
                rechazo(
                    Mensaje::nuevo(&t::PERSONAS_ENTRADA)
                        .con("ruta", RUTA)
                        .con("n", n.to_string())
                        .con_mensaje("que", que),
                )
            };
            if e.id == 0 {
                return Err(entrada(&t::ID_CERO));
            }
            if !palabra_valida(&e.login) {
                return Err(entrada(&t::LOGIN_NO_VALIDO));
            }
            if e.nombre
                .as_deref()
                .is_some_and(|s| s.trim().is_empty() || s.chars().any(char::is_control))
            {
                return Err(entrada(&t::NOMBRE_NO_VALIDO));
            }
            if !e.correos.iter().all(|c| palabra_valida(c)) {
                return Err(entrada(&t::CORREO_NO_VALIDO));
            }
            let nueva = Persona {
                id: e.id,
                login: e.login,
                nombre: e.nombre,
                correos: e.correos,
            };
            for (j, otra) in lista.entradas.iter().enumerate() {
                let (a, b) = (cita(j + 1, otra), cita(n, &nueva));
                if otra.id == nueva.id {
                    return Err(rechazo(
                        Mensaje::nuevo(&t::ID_REPETIDO)
                            .con("ruta", RUTA)
                            .con("a", a)
                            .con("b", b)
                            .con("id", nueva.id.to_string()),
                    ));
                }
                if otra.login.to_lowercase() == nueva.login.to_lowercase() {
                    return Err(rechazo(
                        Mensaje::nuevo(&t::LOGIN_REPETIDO)
                            .con("ruta", RUTA)
                            .con("a", a)
                            .con("b", b),
                    ));
                }
                if let Some(c) = nueva
                    .correos
                    .iter()
                    .find(|c| otra.correos.iter().any(|o| o.eq_ignore_ascii_case(c)))
                {
                    return Err(rechazo(
                        Mensaje::nuevo(&t::CORREO_REPETIDO)
                            .con("ruta", RUTA)
                            .con("a", a)
                            .con("b", b)
                            .con("correo", c.clone()),
                    ));
                }
            }
            lista.ids.insert(nueva.id);
            lista
                .claves
                .extend(e.claves_ssh.into_iter().map(Clave::Ssh));
            lista
                .claves
                .extend(e.claves_gpg.into_iter().map(Clave::Gpg));
            lista.entradas.push(nueva);
        }
        Ok(lista)
    }

    /// Si el id es de una persona de la lista.
    pub fn contiene(&self, id: u64) -> bool {
        self.ids.contains(&id)
    }

    /// Si la clave de firma es de una persona de la lista.
    pub fn contiene_clave(&self, clave: Clave) -> bool {
        self.claves.contains(&clave)
    }

    /// La persona de ese id, con su login de la lista (la lista es la
    /// autoridad que refresca un login renombrado).
    pub fn por_id(&self, id: u64) -> Option<&Persona> {
        self.entradas.iter().find(|p| p.id == id)
    }

    /// La persona que declara ese correo de la 0.16 (sin distinguir
    /// mayusculas).
    pub fn por_correo(&self, correo: &str) -> Option<&Persona> {
        let correo = correo.trim();
        self.entradas
            .iter()
            .find(|p| p.correos.iter().any(|c| c.eq_ignore_ascii_case(correo)))
    }

    /// Las entradas, en el orden del fichero.
    pub fn entradas(&self) -> &[Persona] {
        &self.entradas
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Idioma;

    fn error(texto: &str) -> String {
        Personas::leer(texto)
            .err()
            .and_then(|i| i.diagnostico_en(Idioma::Es))
            .unwrap_or_default()
    }

    /// C3: un `id` repetido, o un `login` repetido sin distinguir
    /// mayusculas, se rechaza nombrando las dos entradas.
    #[test]
    fn id_o_login_repetido_se_rechaza_nombrando_las_dos_entradas() {
        let id = "personas:\n  - {id: 7, login: \"ana\"}\n  - {id: 8, login: \"luis\"}\n  - {id: 7, login: \"otra\"}\n";
        let e = error(id);
        assert!(
            e.contains("1 (ana, id 7)") && e.contains("3 (otra, id 7)") && e.contains("id 7"),
            "{e}"
        );
        let login = "personas:\n  - {id: 7, login: \"Ana\"}\n  - {id: 9, login: \"aNA\"}\n";
        let e = error(login);
        assert!(
            e.contains("1 (Ana, id 7)") && e.contains("2 (aNA, id 9)"),
            "{e}"
        );
        let i = Personas::leer(login).err();
        assert_eq!(
            i.map(|i| i.linea()).as_deref(),
            Some("personas rechazada motivo=personas")
        );
        let correo = "personas:\n  - {id: 7, login: \"a\", correos: [\"x@y\"]}\n  - {id: 9, login: \"b\", correos: [\"X@Y\"]}\n";
        assert!(error(correo).contains("x@y") || error(correo).contains("X@Y"));
    }

    #[test]
    fn entradas_mal_formadas_se_rechazan() {
        for (texto, pista) in [
            ("personas:\n  - {id: 0, login: \"a\"}\n", "mayor que 0"),
            ("personas:\n  - {id: 1, login: \"\"}\n", "login"),
            ("personas:\n  - {id: 1, login: \"a b\"}\n", "login"),
            (
                "personas:\n  - {id: 1, login: \"a\", nombre: \"\"}\n",
                "nombre",
            ),
            (
                "personas:\n  - {id: 1, login: \"a\", correos: [\"\"]}\n",
                "correos",
            ),
            ("personas:\n  - {id: 1, login: \"a\", rol: \"x\"}\n", "YAML"),
            ("personas:\n  - {id: \"uno\", login: \"a\"}\n", "YAML"),
            ("otra: 1\n", "YAML"),
            (
                "personas:\n  - {id: 1, login: \"a\"}\n  - {id: 1, login: \"a\"}\n",
                "id 1",
            ),
        ] {
            let e = error(texto);
            assert!(e.contains(pista), "{texto:?}: {e}");
        }
    }

    /// C4: en pantalla sale siempre el login; el nombre lo acompana y nunca
    /// lo sustituye. Dos personas con el mismo nombre se distinguen por login
    /// e id.
    #[test]
    fn se_ensena_el_login_y_el_nombre_no_lo_sustituye() -> Result<(), Informe> {
        let l = Personas::leer(
            "personas:\n  - {id: 7, login: \"ana-g\", nombre: \"Ana\"}\n  - {id: 8, login: \"ana-p\", nombre: \"Ana\"}\n  - {id: 9, login: \"luis\"}\n",
        )?;
        let (a, b) = (l.por_id(7), l.por_id(8));
        assert_eq!(a.map(Persona::mostrar).as_deref(), Some("ana-g (Ana)"));
        assert_eq!(b.map(Persona::mostrar).as_deref(), Some("ana-p (Ana)"));
        assert_ne!(a.map(|p| p.id), b.map(|p| p.id));
        assert_eq!(l.por_id(9).map(Persona::mostrar).as_deref(), Some("luis"));
        for p in l.entradas() {
            assert!(p.mostrar().starts_with(&p.login));
        }
        assert!(l.contiene(8) && !l.contiene(10));
        Ok(())
    }

    #[test]
    fn correos_y_claves() -> Result<(), Informe> {
        let l = Personas::leer(
            "personas:\n  - id: 7\n    login: \"ana\"\n    correos: [\"Ana@X.com\"]\n    claves_ssh: [3]\n    claves_gpg: [4]\n",
        )?;
        assert_eq!(l.por_correo(" ana@x.com").map(|p| p.id), Some(7));
        assert!(l.por_correo("otra@x.com").is_none());
        assert!(l.contiene_clave(Clave::Ssh(3)) && l.contiene_clave(Clave::Gpg(4)));
        assert!(!l.contiene_clave(Clave::Ssh(4)));
        Ok(())
    }
}
