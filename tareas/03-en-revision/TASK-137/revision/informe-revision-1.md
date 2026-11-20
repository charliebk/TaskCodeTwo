# Informe de revision — TASK-137 (ronda 1)

- Commit revisado: 92f28c9b8bf2191a5531bcf016dbbb47efe50a07
- Revisor: code-quality-reviewer (agente independiente, general-purpose)
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | aceptado | .github/workflows/revision-ia.yml:194 | plan |
| MEN-2 | MENOR | abierto | .github/workflows/revision-ia.yml:217 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Sin CRITICOS ni IMPORTANTES.

MEN-1 aceptado (decision de quien implementa): el paso es shell de GitHub Actions contra la API real; el proyecto no tiene arnes para probar pasos de workflow fuera de GitHub y el lint de xtask solo cubre la lista blanca. La red es el E2E anotado en el Resultado; un arnes de pasos de workflow seria una tarea propia.

## Reproduccion

Entorno: clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-137-r1` en el commit 92f28c9, `CARGO_TARGET_DIR` dentro de la misma carpeta. `cargo test -q -p xtask`: 29 pasados, 0 fallos (lint de workflows incluido). Script del paso `Informe de verificacion` extraido (lineas 193-248) y ejecutado en bash contra la API real de `charliebk/taskcodetwo-e2e-github`, solo lectura, con `PERFIL=completa`:

| Run (VRUN) | Resultado |
|---|---|
| 44760343474 (ejecuto) | artefacto directo; `informe-de.txt` = `44760343474 ` (sin origen); baja `informe.json` |
| 44760498241 (reutilizada) | log con curl da origen b7a4186a...; cita `informe-verificacion-completa` del run 44760343474; `informe-de.txt` = `44760343474 b7a4186a...` |
| 44761381047 (reutilizada en cadena) | mismo resultado (el origen de la linea es el ejecutor, no el intermedio) |
| 44740343425 (TASK-049) | cita el run 44733492697 (origen f4139270...) con su informe |

Casos que intente romper y no rompen:

- Perfil sin job (`rapida`): `sin artefacto ... (sin job del perfil)`, rc 0, sin `informe-de.txt`.
- Perfil no valido (`Completa`, `a b`, vacio): `perfil no valido`, rc 1 (los perfiles de la plataforma son `rapida|impacto|completa|pesada`, todos pasan el filtro `[a-z0-9-]`).
- `VRUN` no numerico (`1/../../x`): 404 de gh, cae a `sin artefacto`, rc 0. En el workflow real el input es `type: number`.
- Nombres de job: el run lista `verificacion / buscar`, `verificacion / completa`, `verificacion / verificacion` y `verificacion`; el filtro `== PERFIL or endswith(" / PERFIL")` elige solo `verificacion / completa`.
- Artefactos caducados o ausentes en los runs del origen (shim de `gh` que devuelve vacio en `/artifacts`): emite `::warning::`, rc 0, no deja `informe-de.txt` ni `informe.json`.
- Fallo de curl (conexion rechazada): `origen=""` por la rama `||`, mensaje `sin artefacto ... ni linea de reutilizacion`, rc 0; sin `pipefail` el estado del pipeline es el de `tail`, pero el resultado es el mismo (origen vacio).
- `for r in $(gh api ...)` bajo `set -eu`: un fallo de gh dentro de la sustitucion no aborta y deja `art` vacio, lo que lleva al `::warning::`.
- `read -r de origen < informe-de.txt`: con solo un campo (run que ejecuto) deja `origen` vacio y rc 0; no se anade el sufijo de reutilizacion. `informe-de.txt` se escribe antes del tope de 1 MiB, pero entonces no hay `informe.json` y `-s` es falso, asi que no se usa.
- `wf` y los ids se validan como numericos antes de entrar en el jq/URL; `PERFIL` validado antes de entrar en el `sed`. Sin inyeccion.
- La linea de log imitable: el codigo del PR corre antes en el mismo job y, ademas, `tail -n 1` hace ganar la ultima linea, incluida una impresa por el paso `comando`. Solo importa cuando el run no sube artefacto y, como el informe ya es dato del autor, lo maximo es citar el informe de otro run del mismo workflow en ese `head_sha`. Esta documentado en el workflow y en la guia; no lo cuento como hallazgo.

### MEN-1 — la logica del paso no tiene ninguna red de regresion automatica

El lint de xtask solo mira la lista blanca del workflow, no el comportamiento del script. Mutacion: borrado el filtro de `PERFIL` (lineas 194-196) y quitado `and .conclusion == "success"` de la busqueda de runs; `cargo test -q -p xtask` sigue en 29 pasados. La unica evidencia de que el paso funciona es el E2E manual descrito en la tarea. Es un paso de shell en un workflow y probarlo exige la API, asi que es razonable aceptarlo como esta; se deja constancia. Arreglo posible: un test de xtask con un `gh` y un `curl` simulados, o fijar al menos con un test de texto que el filtro de perfil y el filtro de `success` siguen en el YAML.

### MEN-2 — si el log del job no se puede leer, no hay `::warning::`

Reproduccion: `GITHUB_API_URL=https://127.0.0.1:9` (curl falla) con el run 44760498241. El paso sale con un `echo` plano, `sin artefacto ... ni linea de reutilizacion en el log de su job 136371525610`, igual que si el run hubiera ejecutado el perfil sin informe. El criterio de aceptacion 2 y la guia piden `::warning::` solo cuando el origen se conoce y su artefacto falta, asi que el caso cumple lo escrito. Pero un log caducado, un 5xx o un 403 se esconden como un caso normal, y es el mismo sintoma (`informe=sin-informe`) que la persona vera. Arreglo posible: distinguir el fallo de curl (`curl -f` falla) de la ausencia de la linea y avisar con `::warning::` en el primero. No bloquea.

## Veredicto recomendado

aprobada. No hay hallazgos CRITICOS ni IMPORTANTES; los dos MENORES quedan documentados y no hace falta corregirlos para cerrar. Evidencia propia: 29 tests de xtask en verde y el paso ejecutado contra la API real en los cuatro runs indicados (con y sin artefacto, reutilizacion simple y en cadena).

## Lecturas fuera del contexto permitido

- `crates/tc2/src/seleccion/orden.rs` y `crates/tc2/src/verificacion/informe.rs` (solo lectura): comprobar como `tc2 revisoras` usa `--artefacto` y que `informe=` pasa a `invalido` o `grande` cuando el informe no vale (para valorar el sufijo `informe-de=`). Sin cambios que reportar.
- `.github/workflows/verificacion.yml` (lineas 255-330, ya en el contexto permitido) y los logs y jobs de los runs 44760498241 y 44760343474 con `gh` (solo lectura), para reproducir.
