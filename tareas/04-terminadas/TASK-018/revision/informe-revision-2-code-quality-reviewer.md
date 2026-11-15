# Informe de revision — TASK-018 (ronda 2)

- Commit revisado: 09e487ed1a5f3f43221c6cb4a968ea0549237c73
- Revisor: code-quality-reviewer (agente general-purpose, haiku)
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-2 | IMPORTANTE | corregido | crates/tc2/src/git/remoto.rs (`clasificar_fallo_remoto`) | implementacion |

IMPORTANTE-1 de la ronda 1: corregido y verificado (ver abajo).

### IMPORTANTE-2 (nuevo, introducido por la corrección de MENOR-4). Una denegación de acceso del remoto se diagnostica como falta de red

La corrección quitó `"access denied"` y acotó `"permission to "` a `"remote: permission to "`, sin test ni mención en la adenda. Sonda de `clasificar_fallo_remoto`:

- `"remote: access denied"` => `SinRed`
- `"fatal: unable to access 'https://host/a.git/': access denied"` => `SinRed`
- `"remote: Permission to a/b.git denied to c."` => `Credenciales` (correcto)
- `"git@host: Permission denied (publickey)."` => `Credenciales` (correcto)

Extremo a extremo (`origin` `ssh://fake.host/x.git` con `core.sshCommand` que escribe `remote: access denied` y sale con 1): `estado sinred operacion=anotar pendiente=no`, diagnóstico «reintenta con red»; lo esperado es `estado denegada operacion=anotar motivo=credenciales`. El test nuevo `permiso_local_no_es_de_credenciales` fija otra confusión: un `Permission denied` local (`cannot open .git/FETCH_HEAD`) da `SinRed`, es decir, un fallo de permisos local se presenta como falta de red. Sin pérdida de datos; diagnóstico falso. No verificado contra un servidor real.

Propuesta: restaurar `access denied` en credenciales, dar a un error local un diagnóstico propio (no `SinRed`) y test de `remote: access denied` => `Credenciales`.

## Estado tras la ronda 2 (lo anota quien implementa)

- IMPORTANTE-2 corregido: `access denied` vuelve a credenciales; un `permission denied` sin marcas del remoto es `FalloRemoto::Local` → `error` (1) con `motivo=error-local` y «revisa los permisos de la carpeta del repo», ni `sinred` ni credenciales. Tests: `permiso_local_no_es_de_credenciales` (local → `Local`, tres textos de `access denied` → `Credenciales`) y `estado_bordes::remoto_que_responde_access_denied_es_denegada_por_credenciales` (extremo a extremo con `core.sshCommand`), rojo con el mutante que quita el patrón.

## Verificación de IMPORTANTE-1 (reproducción y mutante)

- Repo montado sin remoto, `git remote add origin <bare vacío>` y una transacción (API de biblioteca): `estado ok ... tareas=al-dia`; el remoto tiene `LEEME.md`, `notas/a.md`, `notas/b.md`.
- Mutante (`RamaInexistente => None`): `remoto_anadido_despues_publica_la_historia_local` ROJO (`... tareas=diverge`, esperado `al-dia`). Revertido. La corrección funciona y el test la protege.

## Notas sin hallazgo

- Pendiente con `Err(_)` a rechazadas: decisión documentada; un error transitorio de git la deja fuera de la cola, pero su contenido queda en `refs/taskcode/rechazadas/`.
- `ruta_valida` con `aux .md` y `choque_de_mayusculas` con renombrados: no verificados.

## Verificación ejecutada

- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-018-r2-calidad` (09e487e), `CARGO_TARGET_DIR` dentro; borrado al terminar.
- `cargo test -p tc2`: 41 pasados, 0 fallidos (lib 17, cli 2, estado_bordes 6, estado_casos 4, estado_concurrencia 3, git_binario 1, git_casos_limite 5, git_ciclo 1, git_conflicto 1, git_push 1).
- Mutante de IMPORTANTE-1: ROJO. Sondas del clasificador y extremo a extremo: arriba.

## Lecturas fuera del contexto permitido

`estado/lectura.rs` (`Vista::rutas`, `cambios`, `superponer`), `estado/pendientes.rs` (`resolver`), `lib.rs` (superficie del CLI), `tests/comun/mod.rs` (fixture), y las instrucciones de revisión del plugin.
