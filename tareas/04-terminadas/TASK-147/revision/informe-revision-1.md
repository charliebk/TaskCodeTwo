# Informe de revision — TASK-147 (ronda 1)

- Commit revisado: a633ae9ba48107bb986004ac3359e4cf1434a7b8
- Revisor: code-quality-reviewer (agente independiente, general-purpose/sonnet)
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | corregido | tareas/03-en-revision/TASK-147/tarea.md (criterio 5, casilla `[x]`) | implementacion |
| MEN-2 | MENOR | abierto (no verificado) | .github/scripts/probar-instalador.sh:48-51 y 124-125 (ramas `wget` de `bajar` y `ultima_estable`) | plan |
| MEN-3 | MENOR | aceptado | .github/scripts/probar-instalador.sh:~319 (`[ -e "$instalado" ] \|\| rojo`) | plan |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

No hay hallazgos CRITICO ni IMPORTANTE.

### MEN-1: el criterio 5 figura como cumplido y su texto no se cumplio

El criterio dice "El job que usa cada script (`release.yml`) sigue en verde en la rama" y esta marcado `[x]`. El `## Resultado` declara que el CI de la rama NO se ejecuto (sin push, decision de la persona). La casilla afirma algo no ejecutado. Correccion: reescribir el criterio con la evidencia local aceptada o dejarlo sin marcar con esa nota. (Corregido: sin marcar, con la nota.)

### MEN-2: las ramas `wget` nuevas no tienen arnes (no verificado)

`bajar` sin curl y `ultima_estable` con `wget -S --spider ... | sed ... | tail -n 1` no estan ejercitadas: solo corre la rama `curl`. Caso a ejecutar: `PATH` sin curl y con un wget real, y `/releases/latest` en 127.0.0.1 con 302 a `/releases/tag/v2.0.0`; se espera que `ultima_estable` imprima `2.0.0`. Riesgo teorico: con un `Location` relativo o un salto intermedio, `tail -n 1` toma el ultimo. El plan ya decia "como `install.sh:67-76`", asi que el origen es el plan. (No se corrige: no hay wget en la maquina ni Docker, decision de la persona; va al backlog.)

### MEN-3: mutante superviviente aceptado

Quitar solo `[ -e "$instalado" ] || rojo ...` deja el caso a en rojo porque la comparacion de version lo atrapa (`se instalo ''`). Reproducido (M5). El Resultado ya lo declara y justifica. No bloquea.

## Reproduccion

Clon propio en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-147-r1`, rama `fix/task-147-auditoria-4d-a10-scripts-de-ci-e-instala`, HEAD 397be0e.

- `bash -n` limpio en `probar-actualizar.sh`, `probar-instalador.sh` y los 4 arneses.
- Arneses sobre el codigo corregido (Git Bash):
  - `a10/d4-cmp.sh`: `ROJO: el SHA256SUMS alterado es igual al bueno`, rc=1.
  - `a11/d4-release-284.sh`: `::error::el sed no cambio la version de Cargo.toml (9.9.9 -> 9.9.10)`, rc=1, sin `SIGUE`.
  - `a10/d4-linea-unix.sh`: `ROJO: no se pudo bajar http://127.0.0.1:1/install.sh`, rc=1.
  - `a10/d4-sin-version.sh`: caso a rojo `no se instalo`; caso c rojo `se instalo 'tc2 1.9.0', se esperaba 'tc2 2.0.0'`; caso d verde. Salida 0, "los tres casos dan lo esperado".

| Mutante | Resultado |
|---|---|
| M1: quitar `\|\| rojo` en `probar-actualizar.sh` | `SIGUE`, rc=0 (detectado) |
| M2: neutralizar la comparacion de version en `sin_version` | caso c pasa a 0, `NO ESPERADO` (detectado) |
| M3: quitar la marca `descarga-fallo` en `linea_unix` | `SIGUE`, codigo=0 (detectado) |
| M4: quitar `\|\| { ...; exit 1; }` en `release.yml` | `SIGUE`, salida=0 (detectado) |
| M5: quitar solo `[ -e "$instalado" ]` | sobrevive (MEN-3) |

- Los demas llamantes de `linea_unix` conservan la semantica (el codigo sigue siendo el de `sh`; solo anade rojo si la descarga falla). `rojo` dentro de `$(...)` sale con 1 y `set -e` lo propaga.
- No ejecutado, por indicacion de la peticion: `probar-instalador.sh` en modo local, descargas, `cargo test` y `probar-actualizar.sh` completo (se apoya en lo declarado en el Resultado).
- Limpieza: clon y copias borrados; sin procesos python ni cargo.

## Sobre lo decidido por la persona

- Sin push y criterio 5 con evidencia local: el Resultado lo explica bien; solo la casilla quedaba inexacta (MEN-1).
- A10-05 sin reproducir ni corregir: bien explicado (motivo, alcance, mitigacion por sha256). No se cuenta como hallazgo del codigo.

## Regla 4D

- Duplicacion: cumple. `bajar` evita repetir la tuberia curl/wget. `ultima_estable` reimplementa en una linea la consulta de `install.sh`, justificado porque `install.sh` no se puede cargar desde el test; es el contraste deliberado de una prueba independiente.
- Desviacion: cumple. Diff en los 3 ficheros del plan mas los arneses; `release.yml:214` y `linea_windows` sin tocar, como declara el plan.
- Diluvio: cumple. Dos funciones pequenas, sin modulo nuevo.
- Disfraz: cumple. 4 de 5 mutantes detectados; el quinto declarado (MEN-3). Los arneses extraen las funciones del script real y `d4-release-284.sh` avisa si el txt difiere de `release.yml`.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-147/seguimiento/arneses/**`: los arneses que la peticion manda ejecutar.
- `tareas/03-en-revision/TASK-147/planificacion/plan-final.md`: decisiones de la persona citadas por la peticion.
- `docs/auditoria-4d.md` (solo `grep`): texto antiguo del mensaje VERDE y estado de A10-05.
