# Dependencias de la PoC Rust (F1-14, TASK-014)

Consultado en crates.io (API, solo lectura) el 2026-11-14. Fecha = ultima publicacion del crate.

## Directas

| Crate | Version | Ultima publicacion | Estado | Motivo |
|---|---|---|---|---|
| `serde-saphyr` | 2.1.1 | 2026-11-05 | Mantenido (activo) | YAML con `DuplicateKeyPolicy::Error` (es el valor por defecto y ademas se fija explicito). Parser propio en Rust puro (`granit-parser`), sin `unsafe-libyaml`. |
| `serde` (+derive) | 1.0.229 | 2026-07-18 | Mantenido | Deserializar el frontmatter a un `struct`. |
| `ureq` | 3.5.2 | 2026-11-12 | Mantenido (activo) | HTTPS bloqueante con rustls. Features: `default-features = false`, `rustls`, `platform-verifier` (sin gzip). |

## Transitivas relevantes

| Crate | Version | Ultima publicacion | Nota |
|---|---|---|---|
| `granit-parser` | 2.1.0 | 2026-11-02 | Parser YAML de serde-saphyr. |
| `rustls` | 0.23.46 | 2026-11-12 | TLS. |
| `ring` | 0.17.14 | 2025-03-11 | Proveedor criptografico de rustls (mantenido por el equipo de rustls; compila C/asm con MSVC). |
| `rustls-platform-verifier` | 0.7.1 | 2026-09-24 | Verifica certificados contra el almacen de Windows. |
| `webpki-roots` | 1.0.12 | 2026-11-10 | Lo arrastra la feature `rustls` de ureq aunque no se usa. |

Arbol: `cargo tree | wc -l` = 77 lineas; 54 crates distintos (incluido `tablero`); `Cargo.lock` = 90 paquetes (incluye los de otras plataformas).

## Descartadas (YAML)

| Crate | Version | Ultima publicacion | Motivo del descarte |
|---|---|---|---|
| `serde_yml` | 0.0.13 | 2026-05-27 | Su propia descripcion dice "DEPRECATED — unmaintained"; la ultima version es un shim hacia `noyalib`. |
| `serde_norway` | 0.9.42 | 2024-12-21 | Fork de `serde_yaml` sin publicaciones en casi dos anos; depende de `unsafe-libyaml-norway`. |
| `saphyr` | 0.3.4 | 2026-11-11 | Mantenido, pero es un parser de bajo nivel sin serde: habria que recorrer el arbol a mano (mas codigo propio). Su ecosistema es el que usa serde-saphyr. |
