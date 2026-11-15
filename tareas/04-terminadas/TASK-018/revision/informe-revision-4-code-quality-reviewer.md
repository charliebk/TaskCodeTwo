# Informe de revision — TASK-018 (ronda 4)

- Commit revisado: b2f565d50585629c79df41974f1d7af5ee648930
- Revisor: code-quality-reviewer (agente general-purpose, haiku)
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MENOR-1 | MENOR | abierto | crates/tc2/src/git/remoto.rs | implementacion |

IMPORTANTE-3 de la ronda 3: corregido y verificado. Sin CRITICOS. MENOR-1 lo introduce la corrección; no bloquea el cierre.

### IMPORTANTE-3: verificado, corregido

| Texto | Esperado | Resultado |
|---|---|---|
| curl EACCES: `fatal: unable to access 'https://h/a.git/': Failed to connect to h port 443: Permission denied` | SinRed | SinRed |
| ssh EACCES: `ssh: connect to host h port 22: Permission denied` + `fatal: Could not read from remote repository.` | SinRed | SinRed |
| ssh publickey: `git@h: Permission denied (publickey).` + `fatal: Could not read from remote repository.` | Credenciales | Credenciales |
| `remote: access denied` | Credenciales | Credenciales |
| `error: cannot open .git/FETCH_HEAD: Permission denied` | Local | Local |
| curl 403: `... The requested URL returned error: 403` | Credenciales | Credenciales |

Mutante (`let transporte = false && ...`): `git::remoto::tests::red_bloqueada_con_permiso_denegado_es_sin_red` en rojo (`left: Local`, `right: SinRed`).

### MENOR-1: marca `connection` demasiado general

`git@h: Permission denied, please try again.` + `Connection closed by h port 22` + `fatal: Could not read from remote repository.` da `SinRed` (esperado `Credenciales`); sobre 36d96aa daba `Credenciales` (por lectura). Sin pérdida de datos; diagnóstico «sin red» en vez de «revisa tus credenciales»; caso poco frecuente. Propuesta: quitar `"connection"` o acotarla (`"connection refused"`, `"connection timed out"`).

### Nota sin hallazgo

El `ROADMAP.md` generado se comprueba con `duplicada`: en un repo con `roadmap.md` en minúsculas toda escritura quedaría `rechazada motivo=ruta-duplicada`. Es lo que describe la adenda.

## Estado tras la ronda 4 (lo anota quien implementa)

- MENOR-1 corregido: fuera la marca `connection` de `transporte`; test `red_bloqueada_con_permiso_denegado_es_sin_red` ampliado con los dos textos de la sonda (`Connection closed` tras rechazar la contraseña → credenciales; `cannot lock ref ... Permission denied` + `hint: connection lost` → local).

## Verificación ejecutada

- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-018-r4-calidad` (b2f565d), `CARGO_TARGET_DIR` dentro; borrado al terminar.
- `cargo test -p tc2`: 45 pasados, 0 fallidos (lib 18, cli 2, estado_bordes 9, estado_casos 4, estado_concurrencia 3, git_binario 1, git_casos_limite 5, git_ciclo 1, git_conflicto 1, git_push 1).
- Test temporal con los textos de la tabla y la sonda de MENOR-1; mutante de la marca de transporte: rojo.
- No ejecutado: clippy ni extremo a extremo con remoto real.

## Lecturas fuera del contexto permitido

La petición, el informe de la ronda 3 y `tarea.md` (permitidos); el `SKILL.md` de la revisora. No leyó `revision.md`.
