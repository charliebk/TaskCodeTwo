# Informe de revision — TASK-047 (ronda 2)

- Commit revisado: 5f3521117b2005d83286d3bdf9cbd6d0c21ec125
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido (5f35211, verificado) | crates/tc2/src/plataforma/app_github.rs:34, :252, :292 (tests en crates/tc2/tests/identidades.rs:414-500) | implementacion |
| MEN-1 | MENOR | corregido (5f35211, verificado) | crates/tc2/src/plataforma/credenciales.rs:251-262, crates/tc2/src/plataforma/conexion.rs:95-108, crates/tc2/src/i18n/catalogo/plataforma.rs (APP_SOLO_GITHUB) | implementacion |
| MEN-2 | MENOR | corregido (5f35211, verificado) | crates/tc2/src/plataforma/app_github.rs:393-403 | implementacion |
| MEN-3 | MENOR | corregido (5f35211, verificado) | crates/tc2/src/plataforma/github.rs:309 (usada en :329 y en app_github.rs:125) | implementacion |
| MEN-4 | MENOR | corregido (5f35211, verificado) | crates/tc2/src/git/puerto.rs:255-270 y :479-490 | implementacion |
| MEN-5 | MENOR | corregido (5f35211, verificado) | crates/tc2/tests/agente_git/mod.rs (se elimina `en_claro`) | implementacion |

Sin CRITICO ni IMPORTANTE abiertos: el veredicto es `aprobada`.

### Verificacion de cada hallazgo

**Ejecutado en el clon `task-047-r2-cq` (commit 8e556b5, que solo cambia `tareas/` respecto a 5f35211; codigo identico a 5f35211):**

| Orden | Resultado |
|---|---|
| `cargo test -p tc2 --test identidades` | 12 passed, 0 failed |
| `cargo test -p tc2 --lib plataforma::` | 28 passed, 0 failed |
| `cargo test -p tc2 --lib git::puerto` | 4 passed, 0 failed (cubre MEN-4) |
| `cargo test -p tc2 --test plataforma_e2e --no-run` | compila (exit 0): cubre el cambio de MEN-5 en `tests/agente_git/mod.rs` |

No se ha ejecutado la suite completa ni `cargo xtask`, por indicacion expresa de quien orquesta; la ronda no tiene la pasada completa de la suite.

**Mutaciones (todas revertidas con `git checkout`; `git status` limpio al acabar):**

- IMP-1, margen de renovacion: `MARGEN = 0` (app_github.rs:34). `identidades` pasa a 11 passed, 1 failed: `el_token_que_vence_dentro_del_margen_se_renueva_y_el_de_fuera_no` (identidades.rs:420, `left: 0, right: 300`). La red existe.
- IMP-1, reinicio de la instalacion tras 404: se comenta `self.instalacion.set(None)` (app_github.rs:252). `identidades` pasa a 11 passed, 1 failed: `un_404_al_pedir_el_token_olvida_la_instalacion` (identidades.rs:486, `sin-instalacion` en vez de `Ok`). La red existe.
- MEN-2, BOM: se quita el `trim_start_matches('\u{feff}')` (app_github.rs:396). `identidades` pasa a 11 passed, 1 failed: `el_pem_admite_bom_y_texto_antes_del_begin` (identidades.rs:694). La red existe.
- MEN-4, numeracion de `GIT_CONFIG_*`: `let n = previas + i` cambiado por `let n = i` (puerto.rs). `--lib git::puerto` pasa a 3 passed, 1 failed. La red existe.

**IMP-1 (corregido).** Los tres puntos que el hallazgo pedia proteger tienen ahora red:
1. Margen: test de renovacion dentro/fuera de margen (mutacion roja).
2. Reinicio tras 404: test `un_404_al_pedir_el_token_olvida_la_instalacion` (mutacion roja).
3. Causas: `cada_fallo_al_acunar_tiene_su_causa` (identidades.rs:501-590) cubre `jwt-rechazado` (401), `permisos-no-concedidos` (422 y 201 con `permissions` insuficientes) y `respuesta-inesperada`; `el_adaptador_traduce_sinred_al_acunar` cubre `sinred`. Queda sin test la causa `tls` (ver nota).

**MEN-1 (corregido, con un cambio de comportamiento mas alla de lo sugerido).** El texto de `app-solo-github` ya es propio y nombra `TC2_TOKEN_<ROL>`, y el `detalle` nombra la variable. Ademas, la correccion cambia el comportamiento: con App configurada y destino no GitHub, `CredencialesDeEntorno::obtener` usa `TC2_TOKEN_<ROL>` del mismo rol (credenciales.rs:251-262), en vez de fallar. El test pasa de asertar el fallo a asertar el token, y asserta explicitamente el fallo cuando no hay token. No es regresion: `GitHub::nuevo` llama siempre a `atar_github` (github.rs:336), asi que en GitHub la App sigue siendo la unica identidad. La decision esta recogida en la adenda del ADR; la anoto aqui por ser un cambio de contrato, no por bloquear.

**MEN-2 (corregido).** BOM y texto antes de `-----BEGIN` se toleran (app_github.rs:393-403). Verificado por el test permanente y por la mutacion.

**MEN-3 (corregido).** Hay una sola definicion de `repo_valido` (github.rs:309). `GitHub::nuevo` y `app_github` la usan. El comportamiento es el mismo.

**MEN-4 (corregido).** `con_entorno` ahora acumula igual que `con_entorno_extra`; los `GIT_CONFIG_*` se separan y renumeran tras las previas del puerto y del proceso. Los llamadores de `con_entorno` del repo (ciclo/mod.rs, git/estado.rs, tests de `tests/`) hacen una sola llamada sobre un `Git::nuevo()` vacio, asi que el cambio de "sustituir" a "acumular" no les afecta. Los que pasan `GIT_CONFIG_COUNT/KEY_n/VALUE_n` (estado_casos.rs:518-523, aprobar.rs:70-73) siguen dando los mismos pares, renumerados desde 0.

**MEN-5 (corregido).** `en_claro` se elimina. `grep` no encuentra ningun llamador en `crates/tc2`, y `plataforma_e2e` compila.

## Notas (no son hallazgos de la tabla)

- **MENOR, sin test: causa `tls`.** La correccion anade `FalloHttp::Tls` -> `causa=tls` con el texto `APP_TLS` (app_github.rs `fallo_transporte`), y el mapeo de `ureq::Error::Tls`/`TlsRequired` a `FalloHttp::Tls` esta en http.rs:350-353. Lo he leido y parece correcto, pero no hay ningun test que lo ejercite, ni lo he reproducido (haria falta un servidor TLS con certificado no confiable). Propuesta: un test o anotarlo en `menores-pendientes.md`. No bloquea.
- **MENOR, entrada incompleta silenciosa.** `separar_config` (puerto.rs) descarta sin aviso los `GIT_CONFIG_KEY_n`/`VALUE_n` si falta `GIT_CONFIG_COUNT` o si `COUNT` no es un numero. Hoy ningun llamador lo hace.

## Lecturas fuera del contexto permitido

- `~/.claude/plugins/.../task-workflow/revision.md`: plantilla y reglas del informe (lo exige la skill).
- `crates/tc2/src/plataforma/http.rs` (lineas 165-175 y 325-355): mapeo de `FalloHttp::Tls`, para la nota de `tls`.
- `crates/tc2/src/plataforma/github.rs` (linea 336 y 309-330): llamada a `atar_github` y `repo_valido`, para confirmar el comportamiento de MEN-1 y MEN-3.
- `crates/tc2/src/ciclo/mod.rs`, `crates/tc2/src/git/estado.rs`, `crates/tc2/tests/estado_casos.rs`, `crates/tc2/tests/aprobar.rs` (grep de `con_entorno` y `GIT_CONFIG_*`): comprobar que el cambio de `con_entorno` no rompe a sus llamadores.
