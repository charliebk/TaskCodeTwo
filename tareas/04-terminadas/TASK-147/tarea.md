---
id: TASK-147
titulo: "Auditoria 4D: A10 scripts de CI e instaladores (4 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: fix/task-147-auditoria-4d-a10-scripts-de-ci-e-instala
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 496824
tokens_implementacion: 8406045
tokens_revision: 790164
creado: 2026-11-23
actualizado: 2026-11-25
dependencias: []
---
## Objetivo

Corregir los 4 casos IMPORTANTE del grupo A10 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 4 Disfraces en scripts y workflows que verifican o instalan. Tres están reproducidos con un arnés y uno (A10-05) queda NO VERIFICADO hasta reproducirlo. No se piden los MENOR (A10-03, A10-04). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [x] A10-01: la guarda de `.github/scripts/probar-actualizar.sh:60` para cuando los dos `SHA256SUMS` son iguales (`! cmp -s ... || rojo "..."`). El arnés de la ficha (`tareas/04-terminadas/TASK-138/seguimiento/arneses/a10/d4-cmp.sh`: dos ficheros iguales con `set -eu`), cambiando su copia de la línea 60 por la forma corregida, termina con un código distinto de 0.
- [x] A11-01: la guarda de `.github/workflows/release.yml:284` (`! cmp -s Cargo.toml Cargo.toml.orig`, en mitad del paso «vY») para el paso cuando el `sed` de `:283` no cambia la versión (`... || { echo "::error::..."; exit 1; }`). El arnés de la ficha (`tareas/04-terminadas/TASK-138/seguimiento/arneses/a11/d4-release-284.sh`, con las líneas 281-285 corregidas en su `release-281-285.txt` y con `bash --noprofile --norc -eo pipefail`) deja de imprimir `SIGUE` y sale con un código distinto de 0. Mismo patrón que A10-01: se corrigen juntos.
- [x] A10-02: con `codigo` 0, `sin_version` (`.github/scripts/probar-instalador.sh:268-287`) exige el binario instalado y que su `--version` sea la del tag de `/releases/latest`. El arnés de la ficha (`tareas/04-terminadas/TASK-138/seguimiento/arneses/a10/d4-sin-version.sh`: instalador que sale 0 sin instalar), sobre el script corregido, da rojo, y `linea_unix` (`:45-51`) no esconde un fallo de `curl` detrás del 0 de `sh`.
- [ ] A10-05: primero se reproduce, sin red externa, con un servidor en 127.0.0.1 que responde 302 a una URL `http://` y con `instalar/install.sh` sin `curl` en el `PATH`. Si wget sigue la redirección, la rama `wget` de `descargar` (`instalar/install.sh:47-56`) la rechaza, o el instalador exige `curl` y lo dice en el arreglo, y la reproducción queda como prueba. Si wget no la sigue, el caso se cierra como descartado y se anota en `docs/auditoria-4d.md`.
- [ ] El job que usa cada script (`release.yml`) sigue en verde en la rama, y el `## Resultado` lista los 4 IDs con su commit. (No ejecutado: sin push por decision de Carlos; evidencia local en el Resultado. Los 4 IDs si estan listados. MEN-1 de la revision 1.)

## Resultado

Corregidos 3 de los 4 IMPORTANTE del grupo A10; A10-05 queda abierto por decision de Carlos (2026-11-25: sin Docker).

| Ficha | Commit |
|---|---|
| A10-01 y A11-01 (D4) | dd74835 |
| A10-02 (D4) | 4e94a65 |
| A10-05 | sin corregir (pendiente) |

Arneses en `seguimiento/arneses/` (copias corregidas de las de TASK-138), todos en rojo con el arreglo y en verde/`SIGUE` con su mutante:
- `a10/d4-cmp.sh`: `ROJO: el SHA256SUMS alterado es igual al bueno`; mutante quitar `|| rojo` → `SIGUE`, 0.
- `a11/d4-release-284.sh`: `::error::el sed no cambio la version de Cargo.toml`; mutante quitar `|| { ...; exit 1; }` → `SIGUE`, 0.
- `a10/d4-sin-version.sh`: caso a (sale 0 sin instalar) y caso c (otra version) en rojo; caso d (version correcta, contraste) en verde; mutantes volver al `if` sin la parte nueva y quitar la comparacion → 0.
- `a10/d4-linea-unix.sh`: curl contra 127.0.0.1:1 da `ROJO: no se pudo bajar`; mutante quitar la marca → 0.

Mutante superviviente aceptado: quitar solo `[ -e "$instalado" ] || rojo` en `sin_version` deja el caso a en rojo igualmente (lo atrapa la comparacion de version, `se instalo ''`); la linea se mantiene porque da un mensaje mas claro y el plan pide exigir primero el binario.

Criterio 5, sin push (decision de Carlos): el CI de la rama NO se ha ejecutado. En local: `bash -n`/`sh -n` de los dos scripts; `probar-actualizar.sh` completo en verde (vX empaquetado, vY con las lineas corregidas del paso de `release.yml`, los cuatro casos en verde); `cargo test -p xtask` 35 ok; fmt y clippy limpios. `probar-instalador.sh` modo `local` no se ejecuto: en Git Bash entra por la rama Windows, que cambia el PATH de usuario en el registro, y lo cambiado (`linea_unix`, `sin_version`) es de la rama Unix; su evidencia son los arneses.

A10-05 (wget de `install.sh` que sigue una redireccion a `http://`): sin reproducir y sin tocar `install.sh`, porque no hay BusyBox wget ni Docker en la maquina. Queda para la segunda auditoria 4D de comprobacion. Lo acota que nada se instala sin pasar el sha256.

### Divergencias

- La descarga de `linea_unix` va a una funcion `bajar` (curl o wget) en vez de repetir la tuberia en las dos ramas.
- Funcion nueva `ultima_estable` en `probar-instalador.sh` para la version de `/releases/latest`.
- Los arneses extraen la guarda o las funciones del script real en vez de copiarlas a mano; `d4-release-284.sh` avisa si el txt difiere de `release.yml`. Caso b en arnes aparte (`d4-linea-unix.sh`). Quitado el contraste `release-208-214.txt`.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-25T14:53:54Z | plan | automatico | persona | 0.17.1 |
| 2026-11-25T14:59:20Z | approve | automatico | persona | 0.17.1 |
| 2026-11-25T14:59:22Z | start | automatico | persona | 0.17.1 |
| 2026-11-25T15:18:58Z | review | automatico | persona | 0.17.1 |
| 2026-11-25T15:46:43Z | finish | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-25T14:59:19Z | diseno | agente:a85ef397f5f461ba9 | 22 | 62171 | 419494 | 15137 | 25 | 496824 | - | - |
| 2026-11-25T15:18:58Z | implementacion | sesion:f5474097-7166-4a1c-ae42-79e139302dce | 10 | 13145 | 754613 | 3717 | 0 | 771485 | - | - |
| 2026-11-25T15:18:58Z | implementacion | agente:a4cfa12ccd90e5979 | 116 | 166214 | 7425121 | 43109 | 12 | 7634560 | - | - |
| 2026-11-25T15:21:37Z | revision | agente:a27fdb3985229fc1f | 16 | 69231 | 712714 | 8203 | 3 | 790164 | 1 | - |

## Autorrevision

- code-quality-reviewer — 4e94a65
- 4D — 4e94a65: helper buscado, criterios citados, 5 tests mutados

## Verificacion ligera

- 2026-11-25T15:21:40Z: ronda 1; commit revisado a633ae9ba48107bb986004ac3359e4cf1434a7b8; commit verificado a501376c7cfa6d508d2c7f79d77aaf57cc03e287; ficheros: ninguno; corregidos: ninguno; pendientes: MEN-2

## Duracion

- Estimada: 120 min
- Real (de la rama abierta al cierre con merge, sin pausas): 47 min
- Cierre (finish): 1491 s, de 2026-11-25T15:21:52Z a 2026-11-25T15:46:43Z
