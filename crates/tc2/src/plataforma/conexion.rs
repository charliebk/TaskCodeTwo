//! Transporte comun de los adaptadores (F2-6 y F2-9): el token de cada rol,
//! la llamada, la lectura tipada, la paginacion, la cache de identidades y
//! la comprobacion de que la identidad de persona esta en la lista.
//!
//! Cada adaptador conserva su lista cerrada de endpoints y solo le pasa el
//! metodo y la ruta ya construidos: asi los dos paginan, manejan el token y
//! fallan cerrado igual.

use std::cell::RefCell;
use std::collections::BTreeMap;

use serde::de::DeserializeOwned;

use super::credenciales::{Credenciales, Rol, Secreto};
use super::errores;
use super::http::{Base, Cliente, Metodo, Respuesta};
use super::{Paso, Personas, Proveedor, Usuario};
use crate::contrato::{Informe, Resultado};

/// Paginas como mucho al seguir `Link rel=next` (100 elementos cada una).
pub const MAX_PAGINAS: usize = 10;

/// Conexion de un adaptador con su plataforma.
pub struct Conexion {
    cliente: Cliente,
    proveedor: Proveedor,
    credenciales: Box<dyn Credenciales>,
    personas: Personas,
    identidades: RefCell<BTreeMap<Rol, Usuario>>,
}

impl std::fmt::Debug for Conexion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Conexion")
            .field("cliente", &self.cliente)
            .finish_non_exhaustive()
    }
}

impl Conexion {
    /// Conexion a `base` para `proveedor`.
    pub fn nueva(
        base: Base,
        proveedor: Proveedor,
        credenciales: Box<dyn Credenciales>,
        personas: Personas,
    ) -> Self {
        Self {
            cliente: Cliente::para(base, proveedor),
            proveedor,
            credenciales,
            personas,
            identidades: RefCell::new(BTreeMap::new()),
        }
    }

    /// La plataforma.
    pub fn proveedor(&self) -> Proveedor {
        self.proveedor
    }

    /// La lista de personas.
    pub fn personas(&self) -> &Personas {
        &self.personas
    }

    /// Llamadas hechas, `METODO ruta`.
    pub fn registro(&self) -> Vec<String> {
        self.cliente.registro()
    }

    /// Token del rol, o el rechazo dirigido si falta (nunca se prueba con el
    /// de otro rol).
    pub fn token(&self, op: &str, rol: Rol) -> Paso<Secreto> {
        self.credenciales.token(rol).ok_or_else(|| {
            let extra = if rol == Rol::Persona {
                " (el de persona solo se usa desde un terminal interactivo)"
            } else {
                ""
            };
            Informe::nuevo(op, Resultado::Rechazada)
                .con("motivo", "identidad")
                .con("falta", rol.nombre())
                .diagnostico(format!(
                    "esta operacion corre con la identidad de {}: define {}{extra}",
                    rol.nombre(),
                    rol.variable()
                ))
        })
    }

    /// Llamada cruda: cualquier estado HTTP llega como respuesta.
    pub fn pedir(
        &self,
        op: &str,
        rol: Rol,
        (metodo, ruta): (Metodo, String),
        cuerpo: Option<&str>,
    ) -> Paso<Respuesta> {
        let token = self.token(op, rol)?;
        self.cliente
            .llamar(metodo, &ruta, Some(&token), cuerpo)
            .map_err(|f| errores::fallo(op, &f))
    }

    /// Traduce una respuesta que no es 2xx.
    pub fn traducir(&self, op: &str, rol: Rol, permiso: &str, r: &Respuesta) -> Informe {
        errores::traducir(self.proveedor, op, rol, permiso, r)
    }

    /// Respuesta 2xx con una forma inesperada.
    pub fn invalida(&self, op: &str, que: &str) -> Informe {
        errores::respuesta_invalida(self.proveedor, op, que)
    }

    /// Lee un cuerpo con la forma `T`.
    pub fn parsear<T: DeserializeOwned>(&self, op: &str, cuerpo: &str) -> Paso<T> {
        serde_json::from_str(cuerpo)
            .map_err(|e| self.invalida(op, &errores::sanear(&e.to_string())))
    }

    /// Lectura que exige 2xx y la forma `T`.
    pub fn leer<T: DeserializeOwned>(
        &self,
        op: &str,
        rol: Rol,
        permiso: &str,
        ep: (Metodo, String),
    ) -> Paso<T> {
        let r = self.pedir(op, rol, ep, None)?;
        if !r.exito() {
            return Err(self.traducir(op, rol, permiso, &r));
        }
        self.parsear(op, &r.cuerpo)
    }

    /// Lista paginada: sigue `Link rel=next` en la misma base, hasta
    /// [`MAX_PAGINAS`]; con mas, falla cerrado.
    pub fn leer_paginas<T, F>(
        &self,
        op: &str,
        rol: Rol,
        permiso: &str,
        primera: String,
        mut extraer: F,
    ) -> Paso<Vec<T>>
    where
        F: FnMut(&str) -> Paso<Vec<T>>,
    {
        let token = self.token(op, rol)?;
        let mut ruta = primera;
        let mut todo = Vec::new();
        for _ in 0..MAX_PAGINAS {
            let r = self
                .cliente
                .llamar(Metodo::Get, &ruta, Some(&token), None)
                .map_err(|f| errores::fallo(op, &f))?;
            if !r.exito() {
                return Err(self.traducir(op, rol, permiso, &r));
            }
            todo.extend(extraer(&r.cuerpo)?);
            match r.siguiente() {
                Some(url) => {
                    ruta = self
                        .cliente
                        .ruta_de(url)
                        .map_err(|f| errores::fallo(op, &f))?;
                }
                None => return Ok(todo),
            }
        }
        Err(Informe::nuevo(op, Resultado::Error)
            .con("motivo", "demasiadas-paginas")
            .diagnostico(format!(
                "la lista tiene mas de {} elementos: no se evalua a medias",
                MAX_PAGINAS * 100
            )))
    }

    /// Identidad del rol, de la cache o leida con `leer` (una vez por rol y
    /// proceso).
    pub fn identidad(&self, rol: Rol, leer: impl FnOnce() -> Paso<Usuario>) -> Paso<Usuario> {
        if let Some(u) = self.identidades.borrow().get(&rol) {
            return Ok(u.clone());
        }
        let u = leer()?;
        self.identidades.borrow_mut().insert(rol, u.clone());
        Ok(u)
    }

    /// La identidad de persona (`persona`) tiene que estar en la lista.
    pub fn exigir_persona(&self, op: &str, persona: Paso<Usuario>) -> Paso<()> {
        let u = persona.map_err(|mut i| {
            op.clone_into(&mut i.comando);
            i
        })?;
        if u.es_persona(&self.personas) {
            Ok(())
        } else {
            Err(Informe::nuevo(op, Resultado::Rechazada)
                .con("motivo", "identidad")
                .con("rol", "persona")
                .diagnostico(format!(
                    "el token de persona es de «{}», que no esta en la lista de personas del proyecto",
                    errores::sanear(&u.login)
                )))
        }
    }
}

/// Rama aceptable en una ruta de la API (la misma forma que exige Git, mas
/// estricta).
pub fn rama_valida(r: &str) -> bool {
    !r.is_empty()
        && r.len() <= 200
        && !r.starts_with(['-', '/', '.'])
        && !r.ends_with(['/', '.'])
        && !r.contains("..")
        && !r.contains("//")
        && !r.ends_with(".lock")
        && r.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-._/".contains(&b))
}

/// Sha completo (40 hex).
pub fn sha_valido(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Informe de uso (2) con su motivo.
pub fn uso(op: &str, motivo: &str, diag: &str) -> Informe {
    Informe::nuevo(op, Resultado::Uso)
        .con("motivo", motivo)
        .diagnostico(diag.to_owned())
}

/// `merge-pr` rechazado porque el head no es el evaluado.
pub fn head_cambiado(pr: u64) -> Informe {
    Informe::nuevo("merge-pr", Resultado::Rechazada)
        .con("pr", pr.to_string())
        .con("motivo", "head-cambiado")
        .diagnostico(
            "la rama ha cambiado desde que se evaluaron las puertas: vuelve a lanzar estado-puertas",
        )
}

/// Rechazo de un rol que no corresponde a la operacion.
pub fn rol_no_valido(op: &str, rol: Rol) -> Informe {
    Informe::nuevo(op, Resultado::Rechazada)
        .con("motivo", "identidad")
        .con("rol", rol.nombre())
        .diagnostico(format!(
            "{op} corre con la identidad de agente o de persona"
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramas_y_shas() {
        for r in ["develop", "feature/task-021-f2-6-adaptador-github", "a.b_c"] {
            assert!(rama_valida(r), "{r}");
        }
        for r in [
            "", "-x", "/x", "x/", "a..b", "a//b", "a b", "a%2e", "x.lock", ".x",
        ] {
            assert!(!rama_valida(r), "{r}");
        }
        assert!(sha_valido(&"a".repeat(40)));
        assert!(!sha_valido(&"g".repeat(40)));
        assert!(!sha_valido("abc"));
    }
}
