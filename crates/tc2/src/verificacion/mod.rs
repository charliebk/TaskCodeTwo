//! Verificacion del proyecto vista desde tc2 (F5).
//!
//! - [`informe`]: el informe JSON de verificacion (F5-4) que escribe el
//!   comando del proyecto en `TC2_INFORME` y la CI sube como artefacto; su
//!   lector y la seccion con la que el revisor IA lo cita.

pub mod informe;
