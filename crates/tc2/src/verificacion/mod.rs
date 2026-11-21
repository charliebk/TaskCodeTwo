//! Verificacion del proyecto vista desde tc2 (F5).
//!
//! - [`informe`]: el informe JSON de verificacion (F5-4) que escribe el
//!   comando del proyecto en `TC2_INFORME` y la CI sube como artefacto; su
//!   lector y la seccion con la que el revisor IA lo cita.
//! - [`ejecutar`]: `tc2 verificar --perfil <p> [--hook]` (F5-6), que lanza
//!   el comando del proyecto solo con su orden confiada (F1-8 decision 5).

pub mod ejecutar;
pub mod informe;
