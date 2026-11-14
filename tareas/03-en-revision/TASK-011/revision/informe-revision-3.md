# Informe de revision — TASK-011 (ronda 3)

- Commit revisado: 24d05a006aff0a62482dfdd1cd49fcc70088c154
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-10-carriles-y-modo-automatico.md:385 | plan |
| IMP-2 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-011/poc/poc-f1-10.py:256 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 — `tipo: release` desde `feature/*` contra `main` (verificado: corregido)

Estado: corregido. La correccion (e8e3306) cierra el caso de la ronda 2.

- Verificacion: `fila_politica` (poc-f1-10.py:220-232) exige `tipo == release` y `rama == develop` con base `main`; `hotfix` exige `hotfix/*`; cualquier otro PR contra `main` lanza `ErrorCarril`.
- Mutacion M-a (`release` desde `feature/x` contra `main`, carril funcionalidad): `ErrorCarril: PR contra main con tipo 'release' y rama 'feature/x': falla cerrado`.
- Mutacion M-b (`hotfix` desde `develop` contra `main`): `ErrorCarril: ... rama 'develop': falla cerrado`.
- Control (`release` desde `develop` contra `main`, carril funcionalidad): fila `release+funcionalidad`, plan=True, 2 aprobaciones, paradas approve/merge-pr/tag.
- PoC C: 11/11 casos de tipo y base; 1260 combinaciones, 0 `continuar` sobre approve/merge-pr/tag; filas de main por debajo del carril efectivo en la tabla: 0/8.

### IMP-2 — la union de puertas de una fila de `main` deja fuera la politica de configuracion del carril (nuevo, introducido por la correccion)

- Donde: tareas/03-en-revision/TASK-011/poc/poc-f1-10.py:245-256 (la union solo toma la fila de la tabla `tabla[carril]`; la linea 256 aplica `politica_carriles.<carril>` solo `if fila in ORDEN`, y en una fila `release` o `hotfix` ese condicional es falso). Regla declarada en docs/adr/F1-10-carriles-y-modo-automatico.md:391 («una fila de `main` nunca baja de lo que exige el carril efectivo»).
- Que pasa: con una configuracion valida que endurece el carril efectivo, un PR `hotfix/*` → `main` o `develop` → `main` pierde esas puertas. Para `fix` con `plan_aprobado: true` (clave admitida por el ADR, V1), la fila `hotfix+fix` sale sin plan ni `approve`, mientras la misma tarea contra `develop` los exige. Para `funcionalidad` con `paradas_extra: [review]` y `roles_diseno: 3`, la fila `release+funcionalidad` sale con 1 rol y sin parada `review`.
- Reproduccion: script en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-011-r3\mut.py` (fuera del repo, importa la PoC con `importlib`, llama a `cargar_config` y `politica`). Salida:
  ```
  C0 develop carril fix + plan_aprobado: fila=fix plan=True aprob=2 roles=1 paradas=['approve', 'merge-pr']
  X1 hotfix/* -> main carril fix + plan_aprobado: fila=hotfix+fix plan=False aprob=1 roles=1 paradas=['merge-pr', 'tag']
  X2 release develop->main carril funcionalidad + paradas_extra review, roles 3: fila=release+funcionalidad plan=True aprob=2 roles=1 paradas=['approve', 'merge-pr', 'tag']
  X3 hotfix/* -> main carril funcionalidad + paradas_extra review, roles 3: fila=hotfix+funcionalidad plan=True aprob=2 roles=1 paradas=['approve', 'merge-pr', 'tag']
  ```
  En X2 y X3 `review` no aparece entre las paradas que `politica` devuelve para la fila de `main` (`paradas_extra` no se aplica), y `roles` queda en 1 frente a los 3 exigidos por `roles_diseno`.
- Impacto: un proyecto que endurece `fix` con `plan_aprobado` (o `funcionalidad` con `paradas_extra` o `roles_diseno`) y publica un hotfix o release a `main` se salta esas puertas, aunque el carril efectivo las exige. La puerta de persona en `merge-pr` sigue en pie, asi que no hay merge sin revision humana; lo que se pierde es el plan firmado, el `approve` y los roles de diseno. La afirmacion del ADR «nunca baja» es falsa para la configuracion, y la PoC no lo detecta porque su barrido C y su comprobacion de «filas de main por debajo del carril» solo usan la tabla, con configuracion vacia.
- Sugerencia: aplicar a la fila de `main` las `politica_carriles.<carril>` del carril efectivo (plan, aprobaciones, roles, paradas_extra, lote, perfil, revisoras_extra) con la misma union, o, si la decision es no hacerlo, escribirlo en la decision 1 del ADR y limitar la frase «nunca baja de lo que exige el carril efectivo» a la tabla. Anadir a la PoC C un caso con `politica_carriles.fix.plan_aprobado: true` y un hotfix a `main`, y ampliar la comprobacion de filas por debajo del carril con la configuracion.
- Clasificacion: IMPORTANTE y no CRITICO. Es comportamiento incorrecto con una configuracion real y un caso no de borde, pero no hay perdida de datos ni estado corrupto, y el merge sigue exigiendo a una persona. Se abre aqui porque la union de puertas la declara la propia correccion (e8e3306).

## Verificacion realizada

- Alcance: verificacion acotada de IMP-1 y del delta de la correccion (`08e34e7..24d05a0`). La peticion trae `HEAD` = `24d05a0`; el HEAD actual es `a54736d`, que solo anade la peticion y `tarea.md` (y el informe), sin tocar el codigo: el revisado es `24d05a0`.
- Puerta: el repo no tiene build ni linter aplicables a esta tarea (cambios de documentacion y una PoC Python). Verificacion ejecutable: `poc-f1-10.py` y `comprobar-repo.sh`. No hay suite de tests de codigo; la suite de la ronda es la PoC.
- PoC: `python -I -B tareas/03-en-revision/TASK-011/poc/poc-f1-10.py <carpeta temporal> docs/adr/F1-10-carriles-y-modo-automatico.md <carpeta Personal>` sobre `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-011-r3` (borrada al terminar). Resultado: exit 0, «todo como se espera: A, B, C y D». A: 23/23 casos; B: 6 filas, 0 fallos, 12/12 desvios detectados; C: 11/11 tipo y base, 1260 combinaciones con 0 `continuar`, 160 combinaciones sin fila que fallan cerrado; D: 5/5 aceptadas, 17/17 rechazadas.
- `bash .github/scripts/comprobar-repo.sh`: «Tareas: 93; enlaces relativos: 82; fallos: 0», exit 0.
- Diff de la correccion: `git diff 08e34e7..24d05a0` (ADR y PoC: `fila_politica`, union en `politica`, normalizacion de rutas en `clase_ruta`, casos nuevos en C). Normalizacion de rutas: `./` y componentes vacios se quitan antes de clasificar; la ruta vacia falla cerrado. Solo puede subir de carril, no bajar. Sin hallazgos nuevos ahi.
- Mutaciones de la union de puertas: M-a, M-b, X1 a X3 (ver IMP-1 e IMP-2), en el directorio temporal fuera del repo.
- Sin verificar: el resultado real de GitHub (ruleset, `pull_request_target`, nombre de la rama de origen). Lo que la PoC modela es la logica, no la plataforma.

## Notas (MENOR, no abren ronda)

- La columna «Revisoras» de la fila `release` dice «ninguna» y la del carril efectivo «por dominio del diff»; la union de puertas no incluye esa columna. Un release a `main` desde `develop` con carril funcionalidad sale sin revisoras de dominio. El ADR no dice que las revisoras formen parte de la union; conviene aclararlo. Se deja a criterio de quien coordina.
- `tipo: errata` contra `main` falla cerrado (no hay fila): un cambio solo de `docs/` que vaya directo a `main` necesita `tipo: hotfix` y rama `hotfix/*`. El ADR lo dice ahora («va como hotfix», docs/adr/F1-10-carriles-y-modo-automatico.md:469); es seguro.

## Lecturas fuera del contexto permitido

- `~/.claude/plugins/cache/taskcode-marketplace/taskcode-plugin/0.16.1/skills/task-workflow/revision.md`: reglas comunes de la revision (plantilla, rondas, veredicto). Lo exige la skill revisora.
- `tareas/03-en-revision/TASK-011/revision/informe-revision-2.md`: para comprobar IMP-1, como pide la peticion.
- `tareas/03-en-revision/TASK-011/poc/poc-f1-10.py`: codigo de la PoC, parte del diff revisado (lineas 90-110, 220-300, 540-830).
- `.github/scripts/comprobar-repo.sh`: ejecutado, no leido en detalle.
