# Informe de revision — TASK-046 (ronda 3)

- Commit revisado: d41033b (HEAD de la rama; la peticion cita 3d047d4, y el codigo es el de 1c9f9e9: `git diff 1c9f9e9 HEAD -- crates` esta vacio, el resto son ficheros de tareas/)
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | crates/tc2/src/confianza/cierre.rs | implementacion |
| IMPORTANTE-2 | IMPORTANTE | corregido | crates/tc2/src/confianza/cierre.rs, crates/tc2/src/config/validar.rs | implementacion |
| MENOR-5 | MENOR | corregido | crates/tc2/src/confianza/mod.rs | implementacion |
| MENOR-6 | MENOR | corregido | crates/tc2/tests/verificar.rs (mutantes vivos: `por_nombre(orden)` y mayusculas de `por_nombre`), docs/ci/verificacion-reutilizable.md (nombres con escape y UTF-16) | implementacion |

Detalle del estado:
- IMPORTANTE-1: corregido (1c9f9e9), verificado: los dos casos residuales (apostrofo suelto y comillas anidadas) entran ya en el cierre por el nombre; la mutacion que lo quita pone rojo 2 tests
- IMPORTANTE-2: corregido (c6ffbc1, f7a9be6), verificado en la ronda 2; sin cambios en esta ronda y su test sigue verde
- MENOR-5: corregido (1c9f9e9), verificado: `mod.rs:43-45` tiene un solo doc comment de `VERSION_ALMACEN`
- MENOR-6: corregido (70f7f92; limites en la guia, 85ac8f1)

Sin CRITICO ni IMPORTANTE abiertos. Ningun hallazgo nuevo que introduzca la correccion.

## Notas (fuera de la tabla, no abren ronda)

1. Dos mutantes sobreviven, sin test que los mate (huecos de test, MENOR):
   - Quitar `pendientes.extend(indice.por_nombre(orden))` en `calcular` deja la suite en 14/14 verde: ningun test nombra el script raro en la propia orden configurada (los dos tests nuevos lo nombran dentro del script principal).
   - Cambiar `clave_busqueda(texto)` por `texto.to_string()` en `por_nombre` (perder la insensibilidad a mayusculas del nombre) tambien deja 14/14 verde.
   Ambos son caminos de codigo reales; solo les falta un test que los fije.
2. Residual de la misma familia, no introducido por la correccion: `por_nombre` busca el nombre literal. Un nombre escrito con escape de shell (`mi\ paso.sh`, `mi^ paso.bat`) no coincide, y un script guardado en UTF-16 (NUL entre caracteres tras `from_utf8_lossy`) tampoco, ni por nombre ni por tokens. Es un limite de diseno; convendria anadirlo al apartado de limites de docs/ci/verificacion-reutilizable.md. No lo he reproducido con un caso ejecutado, es lectura del codigo.
3. La prueba P1 pesada (`presupuesto_de_un_segundo_...`) no tiene scripts de nombre raro en su fixture, asi que no ejercita la relectura en caliente; la cifra con un raro la dio el autor "a mano" (doc). No lo he medido yo con raros.

## Alcance

- Ficheros revisados: crates/tc2/src/confianza/cierre.rs (`Indice::raros`, `por_nombre`, `calcular`), crates/tc2/src/confianza/mod.rs, crates/tc2/tests/verificar.rs (tests nuevos y `raro_entra_en_el_cierre`), docs/ci/verificacion-reutilizable.md (solo el parrafo nuevo).
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: no aplica; el diff no toca `#[wasm_bindgen]` ni tipos exportados.
- Lecturas fuera del contexto permitido: la skill revisora y el informe de la ronda 2 (los pide la peticion); `es_de_token`, `entrecomillados` y `es_script` de cierre.rs (mismo fichero permitido).

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-046-r3-rust`, rama feature/task-046-f5-6-hook-local-rapido-opcional-y-con-co, HEAD d41033b. Sin compilar en el checkout original. Se borra al terminar.
- Build/Suite: `cargo test -p tc2 --test verificar --locked` -> 14 passed, 0 failed, 1 ignored (el pesado), 12,10 s. No se ha ejecutado `cargo clippy` ni `cargo fmt --check` en esta ronda (sin verificar).
- Casos construidos: los tests `apostrofo_suelto_no_deja_fuera_un_script_con_espacios` (`echo Don't panic & call "%~dp0lib\mi paso.bat"`) y `comillas_anidadas_no_dejan_fuera_un_script_con_espacios` (`. "$(dirname "$0")/lib/mi paso.sh"`) reproducen el CRITICO-1 de la ronda 2 con cache caliente: versionar el raro tras confiar da `cambiado` sin ejecutar, `confiar` da `ficheros=4`, y modificarlo despues da `cambiado` y salida 3. Pasan.
- Mutacion (cada una sobre el clon, con `git checkout -- .` despues):
  - M2 sin relectura en caliente (`if false && ...`): rojo, 2 tests (apostrofo y anidadas).
  - M3 sin `por_nombre` en la lectura en frio: rojo, 2 tests.
  - M5 `raros` siempre vacio: rojo, 2 tests.
  - M1 sin `por_nombre(orden)`: verde (ver nota 1).
  - M4 `por_nombre` sin insensibilidad a mayusculas: verde (ver nota 1).
- Panics: en el diff no hay `unwrap`, `expect` ni `panic!`; `rsplit('/').next().unwrap_or(r)` no puede fallar; `std::fs::read` en la relectura propaga `ErrorCierre::Lectura` (fichero borrado entre `metadata` y `read` da error, no panic); `from_utf8_lossy` tolera datos no UTF-8.
- Medidas (P1 <= 1 s), maquina a 6-12 % de CPU, release, `cargo test -p tc2 --release --locked --test verificar -- --ignored --nocapture`: sin confiar 338 ms; en frio 297 ms (1 210 462 bytes rehasheados, 51 ficheros); en caliente 234 ms (0 bytes). Dentro del presupuesto, una sola ejecucion de cada. Coste de la relectura en caliente: acotado a los scripts del cierre (<= 2 MB cada uno) y solo si el repo tiene algun script raro; `por_nombre` hace una pasada en minusculas por texto y una comparacion por raro (lineal).

## Revisado sin hallazgos

- Cierre transitivo con `por_nombre`: lo que añade pasa por `pendientes` y por `vistos`, sin bucles ni doble lectura de un fichero.
- Sobre-inclusion por coincidencia de subcadena: solo suma de mas a la huella (falla cerrado), documentado.
- Caso `mtime == 0`: sigue leyendo siempre, y `por_nombre` se aplica en la rama de lectura en frio.
- Documentacion `docs/ci/verificacion-reutilizable.md`: el parrafo nuevo describe lo que hace el codigo.
