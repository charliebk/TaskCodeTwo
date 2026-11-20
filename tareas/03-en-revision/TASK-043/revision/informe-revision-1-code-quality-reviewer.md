# Informe de revision — TASK-043 (ronda 1)

- Commit revisado: 23039496d45451cb132c8fd041a42333355bf9a1
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | docs/ci/verificacion-reutilizable.md:51 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Debajo, la reproduccion de cada hallazgo, o "sin hallazgos" explicito.

## Reproduccion

Clon en `%LOCALAPPDATA%\pruebas-claude	askcodetwo	ask-043-r1-b`, rama de la tarea (HEAD 23cc5e5, contenido revisado 2303949), `CARGO_TARGET_DIR` dentro del clon. Alcance: los cuatro ficheros de mi dominio; en la plantilla de GitLab solo lectura de reglas, sin pipeline real (no verificado: ni la interpolacion del booleano ni `merge_train`, sin Premium).

- `cargo test -p xtask`: 31 pasan, 0 fallan.
- Extraido el `run:` de «Perfiles del evento» y ejecutado en bash (`GITHUB_OUTPUT` a fichero):
  - `cola=false`: PR a develop -> `["completa"]`; hotfix a main -> `["completa"]`; develop->main -> `["completa","pesada"]`. Igual que antes; solo cambia el texto del `echo` (incluye `cola-de-merge=`).
  - `cola=true`: PR a develop y hotfix a main -> `["impacto"]`; develop->main -> `["completa","pesada"]` (la release no cambia); `merge_group` y `workflow_dispatch` -> `["completa"]`; `push` -> `[]`.
  - Perfiles forzados (`impacto rapida`) con cola -> se respetan; forzado invalido -> rc=2 con su mensaje.
  - `cola=true` sin cola real: el PR corre solo la impacto (riesgo ya aceptado y documentado en la adenda y la guia; la linea `cola-de-merge=true` del resumen lo deja escrito).
- Bucle de `buscar`: `["impacto"]` -> sin perfiles cacheables, `claves={}`, sale sin busqueda con `candidatos={}`; `["impacto","completa"]` -> solo `completa`. El job `ejecutar` y el publicador toleran la ausencia de clave, candidato y linea `arbol=` de la impacto (el paso `cache` usa `[.[$p][]?]`; el publicador hace `continue`).
- La linea `cola-de-merge=` del resumen no empieza por `arbol=`, asi que la busqueda por `awk '/^arbol=/'` no la lee. El salto de linea dentro del `run:` queda sin sangria tras el recorte de YAML, igual que la linea `arbol=` existente.
- Mutaciones (en el clon, revertidas): `elif [ "$COLA" = true ]` -> `false`; quitar `select(. != "impacto")` en buscar; quitar el `continue` de la impacto en el publicador. Las tres ponen rojo exactamente un test de `xtask` (`la_cola_de_merge_elige_los_perfiles_del_evento` y `la_impacto_nunca_se_cachea_y_la_cola_es_explicita`): hay red de regresion.
- Plantilla de GitLab (lectura): la impacto excluye `merge_train` y la release; la completa tiene reglas disjuntas por caso (sin cola, tren, release); `after_script` omite la linea para `TC2_PERFIL=impacto`. Coherente con la adenda.

### MEN-1 — tabla de eventos ambigua

`docs/ci/verificacion-reutilizable.md:51` dice «PR / MR a cualquier rama, con `cola-de-merge: true` | impacto» y la fila siguiente (release) la contradice para develop->main. Se entiende por el orden y por «con o sin cola», pero la ADR (linea 553) es mas precisa: «PR a `develop` o de hotfix a `main`». Sugerencia: «PR / MR que no es la release» en la fila 51. No bloquea.

Sin hallazgos CRITICOS ni IMPORTANTES.


## Lecturas fuera del contexto permitido

ninguna
