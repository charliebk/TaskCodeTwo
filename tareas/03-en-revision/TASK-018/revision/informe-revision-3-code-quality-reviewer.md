# Informe de revision — TASK-018 (ronda 3)

- Commit revisado: 36d96aa
- Revisor: code-quality-reviewer (agente general-purpose, haiku)
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-3 | IMPORTANTE | corregido | crates/tc2/src/git/remoto.rs (`clasificar_fallo_remoto`, rama `permission denied` → `Local`) | implementacion |

IMPORTANTE-2 de la ronda 2: cubierto por `remoto_que_responde_access_denied_es_denegada_por_credenciales`, en verde.

### IMPORTANTE-3 (introducido por la corrección de la ronda 2). Un fallo de red con EACCES se clasifica como fallo local

Cualquier stderr con `permission denied` sin marcas del remoto se toma como fallo del clon. Un fallo de conexión de curl con EACCES (firewall, SELinux), p. ej. `fatal: unable to access 'https://...': Failed to connect to host port 443: Permission denied`, antes daba `sinred` y ahora da `error-local`: la operación no queda pendiente y el diagnóstico («revisa los permisos de la carpeta del repo») es falso. Propuesta: no clasificar como `Local` un `permission denied` con marcas de transporte (`failed to connect`, `unable to access`, `could not resolve`); reservar `Local` para errores que citen una ruta de `.git` o del worktree.

**Verificación de quien implementa** (el revisor lo dejó como no verificado): test unitario `red_bloqueada_con_permiso_denegado_es_sin_red` con ese texto sobre 36d96aa:

```
assertion `left == right` failed
  left: Local
 right: SinRed
```

## Estado tras la ronda 3 (lo anota quien implementa)

- IMPORTANTE-3 corregido: con marcas de transporte (`unable to access`, `failed to connect`, `connect to host`, `could not resolve`, `couldn't connect`, `connection`) un `permission denied` es red (`SinRed`), ni local ni credenciales. Test `red_bloqueada_con_permiso_denegado_es_sin_red` (curl y ssh con EACCES), rojo antes de la corrección; `permiso_local_no_es_de_credenciales` sigue dando `Local` para `.git/FETCH_HEAD`.

## Verificación ejecutada

- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-018-r3-calidad` (36d96aa), `CARGO_TARGET_DIR` dentro.
- `cargo test -p tc2`: 43 pasados, 0 fallidos.
- Lectura de los consumidores de `FalloRemoto::Local` (fetch, `intento_push`, `publicar`, `push`, `actualizar_rama`, `tag_anotado`, transacción y `montar`): todos dan `error motivo=error-local`; ninguno acaba en `sinred` ni en pendiente.
- No ejecutado por el revisor: su sonda, el mutante y los extremo a extremo. El revisor no borró su carpeta; la borró quien implementa.

## Lecturas fuera del contexto permitido

`crates/tc2/src/git/mod.rs`, `crates/tc2/src/lib.rs` y `crates/tc2/tests/estado_bordes.rs`.
