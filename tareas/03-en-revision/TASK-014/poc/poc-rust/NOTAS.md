# Notas de la PoC Rust (F1-14, TASK-014)

Implementa `../especificacion-comando.md`. Binario: `tablero.exe`, generado fuera del repo en
`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-014\target-rust\release\tablero.exe`.

## Entorno

- Toolchain ya instalada: `rustc 1.100.0` / `cargo 1.100.0`, `stable-x86_64-pc-windows-msvc`. No se instalo nada
  (ni toolchains, ni targets, ni zig). Solo se bajaron crates de crates.io con `cargo fetch`/`build`.
- `CARGO_TARGET_DIR=%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-014\target-rust`.
- Maquina: Windows 11 Pro, 12 hilos logicos.

## Medidas (tomadas por el agente, 2026-11-14)

| Medida | Valor |
|---|---|
| Minutos de trabajo (reloj, de leer la especificacion a tener binario probado y notas) | ~8 min (14:51:32 a 14:59:15) |
| Iteraciones de compilar y corregir | 3 compilaciones release: 1.a compilo sin errores ni avisos; fallo funcional (`red: no`); 2.a con traza de depuracion; 3.a con la correccion. 0 errores de compilacion. |
| Lineas de codigo propias | `src/main.rs`: 175 lineas totales, 147 sin vacias ni comentarios de linea. `Cargo.toml`: 19 lineas. |
| Crates en el arbol | `cargo tree \| wc -l` = 77; 54 crates distintos; 3 directos (`serde`, `serde-saphyr`, `ureq`). |
| Compilacion release en limpio | 58,5 s (`cargo clean` + `cargo build --release`, dependencias ya descargadas en `~/.cargo/registry`). |
| Recompilacion incremental tras tocar `main.rs` | 24 a 49 s (domina LTO thin + `codegen-units = 1`). |
| Tamano del `.exe` | 2 720 768 bytes (2,59 MiB), con `strip = true`. |
| Tiempo de ejecucion contra TaskCodeTwo (94 tareas) | ~210 ms con `--sin-red` (incluye lanzar `git`); ~0,46 s con red. |

Perfil release: `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`, `strip = true`, `panic = "abort"`.
No se probo `opt-level = "z"`/`"s"` (reduciria tamano a costa de velocidad; queda como dato no medido).

## Decisiones de interpretacion de la especificacion (para que la PoC TS coincida byte a byte)

1. **Frontmatter**: la primera linea del fichero tiene que ser exactamente `---` (se toleran BOM UTF-8 inicial y
   finales `\r\n`). El YAML va hasta la siguiente linea exactamente `---`. Si la primera linea no es `---`
   (aunque haya una linea en blanco antes) o no hay `---` de cierre -> `sin frontmatter`.
2. **Campos**: `id` y `titulo` son obligatorios; si faltan, la deserializacion falla -> `yaml invalido`.
   `estado` opcional -> `desconocido`. Un escalar no texto (`id: 14`) lo acepta serde-saphyr como texto `"14"`.
3. **Clave duplicada** en cualquier clave del mapeo (tambien en las que no se leen, p. ej. `tipo` repetido) ->
   `clave duplicada`. Se detecta por la variante `Error::DuplicateMappingKey` (desenvolviendo `WithSnippet`/`AliasError`).
4. **Orden**: rutas relativas con `/` comparadas por bytes; estados por bytes (`Bloqueada` antes que `desconocido`).
5. **Errores de E/S propios** (sin argumentos, `git` que falla, raiz que no es repo git, fichero ilegible,
   stdout que no se puede escribir) -> mensaje por stderr y codigo de salida 1. Si `tareas/` no existe se cuenta 0 tareas.
6. **Red**: cualquier fallo (DNS, TLS, timeout de 10 s, no 2xx) -> `no`.
7. Ficheros no UTF-8 se leen con sustitucion (`from_utf8_lossy`); no se ha probado ninguno.

## Problemas encontrados

1. **TLS con raices embebidas falla en esta maquina.** Con `ureq` + `rustls` por defecto (raices de Mozilla de
   `webpki-roots`) la peticion a `api.github.com` daba `InvalidCertificate(UnknownIssuer)` y la salida era `red: no`,
   mientras `curl` (Schannel, almacen de Windows) devolvia 200. Hay una CA local en el almacen de Windows (inspeccion
   TLS de antivirus o proxy) que Mozilla no conoce. Correccion: feature `platform-verifier` de ureq y
   `RootCerts::PlatformVerifier` (verifica contra el almacen del sistema). Con eso `red: si`.
   Leccion para el ADR: un binario de escritorio debe usar el almacen del SO; con raices embebidas fallaria en equipos
   corporativos. Coste: un crate mas (`rustls-platform-verifier`) y `webpki-roots` sigue en el arbol sin usarse.
2. `ring` (proveedor cripto de rustls) compila C/asm: funciona porque MSVC Build Tools ya estan instalados; en una
   maquina sin ellos la compilacion desde fuente fallaria (no afecta al `.exe` distribuido).
3. Eleccion del crate YAML: `serde_yml` esta marcado como DEPRECATED por su autor, `serde_norway` lleva desde
   2024-12 sin publicar; `serde-saphyr` esta activo y rechaza claves duplicadas por defecto. Ver `DEPENDENCIAS.md`.
4. La version actual del API de ureq 3 (`Agent::config_builder`, `http_status_as_error(false)`) se consulto en el
   codigo fuente del crate descargado; no hubo errores de compilacion por API.

## Pruebas ejecutadas

- Repo de juguete `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-014\juguete` (8 tareas: validas LF y CRLF, sin estado,
  clave duplicada, sin frontmatter, YAML roto, subdirectorio profundo, estado con mayuscula y carpeta `Zeta`).
- Casos limite `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-014\casos` (duplicada en clave no leida, sin `id`,
  frontmatter sin cierre, linea en blanco antes de `---`, `id` numerico, raiz sin git, raiz inexistente, sin argumentos).
- Repo TaskCodeTwo en solo lectura: 94 tareas, 0 errores. Salida guardada en
  `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-014\salida-rust-taskcodetwo.txt` (123 bytes, 0 CR, sin espacios finales).
