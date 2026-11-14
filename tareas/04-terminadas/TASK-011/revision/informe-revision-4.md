# Informe de revision — TASK-011 (ronda 4)

- Commit revisado: 8ef8213643323201b94815602d928e4cffb6fb8a
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

Verificacion acotada: IMP-1 e IMP-2 de la ronda 3 frente al delta de la correccion `4f79940`. Los dos quedan resueltos. No hay hallazgos nuevos.

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-10-carriles-y-modo-automatico.md:407 | plan |
| IMP-2 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-011/poc/poc-f1-10.py:258 | implementacion |
| MEN-1 | MENOR | abierto | docs/adr/F1-10-carriles-y-modo-automatico.md:331 | implementacion |

Nota sobre las ubicaciones: la peticion cita `ADR:385` y `poc-f1-10.py:256`; esas lineas se han desplazado con las ediciones de la ronda 3 y 4. Las de la tabla son las vigentes en el commit revisado.

### IMP-1 — corregido (verificado)

- Regla comprobada: una fila de `main` solo existe con `release` desde `develop` o `hotfix` desde `hotfix/*`; cualquier otro PR contra `main` falla cerrado (`fila_politica`, poc-f1-10.py:220-232).
- Mutaciones: `release` desde `feature/x` contra `main` → `ErrorCarril` (M1 de la PoC C); `hotfix` desde `develop` contra `main` → `ErrorCarril`.
- Sin cambios en esta ronda respecto de lo que ya estaba verificado en la ronda 3.

### IMP-2 — corregido (verificado)

- Fallo original reproducido con la PoC de esta ronda (X1–X3): con `politica_carriles.fix.plan_aprobado: true`, un hotfix a `main` sale sin plan ni `approve`; con `funcionalidad` endurecida (`paradas_extra: [review]`, `roles_diseno: 3`), la fila de `main` pierde `review` y queda con 1 rol.
- Tras la correccion: X1 sale `plan=True`, 2 aprobaciones, paradas `approve/merge-pr/tag`; X2 y X3 salen con `review` y 3 roles.
- Codigo: `politica()` (poc-f1-10.py:258-280) toma `politica_carril(carril efectivo, config)` y une columna a columna: plan, aprobaciones, roles, checks, paradas (incluidas `paradas_extra`), lote (fuera del lote si cualquiera de las dos lo esta), perfil (el mayor), revisoras (las dos, descartando «ninguna») y `revisoras_extra`. Con esto se cierra tambien la nota MENOR de la ronda 3 sobre las revisoras.
- Mutaciones propias (script fuera del repo, importa la PoC con `importlib`):
  - M1: `hotfix/x` → `main` con `.taskcode/config.yml` y `riesgo-alto` endurecido (`paradas_extra: [review]`, `roles_diseno: 4`, `revisoras_extra`): fila `hotfix+riesgo-alto`, plan=True, 2 aprobaciones, roles 4, paradas `approve/merge-pr/review/tag`, `revisoras_extra` presente. Correcto.
  - M1b: `release` desde `develop` → `main`, mismo caso: `release+riesgo-alto`, mismas puertas, perfil `completa y pesada` (el mayor). Correcto.
  - M1c: el mismo PR contra `develop`: `riesgo-alto` con `review` y 4 roles. Correcto.
  - M2: `hotfix/x` → `main` con `src/` y `fix` endurecido (`revisoras_extra`, `paradas_extra: [start]`, `lote: false`, `perfil`): `hotfix+fix`, plan=False (no se pidio), `start` incluido, `lote=False`, `revisoras_extra` presente. Correcto.
  - M3: `release` desde `develop` → `main` con `funcionalidad` endurecida (`roles_diseno: 3`, `revisoras_extra`): plan, 2 aprobaciones, 3 roles, `revisoras_extra`. Correcto.
  - M4: `hotfix/x` → `main` con `.github/` sin configuracion: `hotfix+riesgo-alto`. Correcto.
  - M5: `rutas_carril.fix` sobre `.taskcode/**`: la configuracion se acepta, pero el minimo sigue en `riesgo-alto` (el suelo esta en el binario). Correcto.
  - M6: `./.TASKCODE/x` → `riesgo-alto`. Correcto.

## Sin hallazgos nuevos

No hay CRITICO. Ningun fallo nuevo introducido por la correccion: en el diff de la ronda (`4f79940` y el delta de `poc-f1-10.py`) no encontre ninguna union de puertas que baje una fila de `main` por debajo de `politica(carril efectivo, config)`, ni caso de borde que la rompa, en las mutaciones de arriba.

## Verificacion realizada

- Commit revisado: `8ef8213`. Entre ese commit y HEAD (`64b5273`) solo cambian `peticion-revision-4.md`, `informe-revision-3.md`, `tarea.md` y este informe; ningun fichero de codigo ni del ADR. La verificacion se hizo sobre el arbol de trabajo en ese commit. No se clono el repo: la PoC lee los repos de trabajo con `git -C` sobre commits fijos, en solo lectura.
- Puerta: el repo no tiene build ni linter aplicables a esta tarea. La suite de la ronda es la PoC.
- `python -I -B tareas/03-en-revision/TASK-011/poc/poc-f1-10.py <carpeta temporal> docs/adr/F1-10-carriles-y-modo-automatico.md <carpeta Personal>` en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-011-r4` (borrada al terminar): exit 0, «todo como se espera: A, B, C y D».
  - A: 23/23 casos; ToolsaWeb errata=22 fix=134 riesgo-alto=4; OpenDockTrading errata=1 fix=41 riesgo-alto=1.
  - B: 6 filas, 0 fallos, 12/12 desvios detectados.
  - C: 1260 combinaciones, 0 `continuar`, 160 sin fila que fallan cerrado; tipo y base 14/14; filas de main por debajo del carril configurado 0/40 (5 configuraciones × 4 carriles × 2 filas de main); X1–X3 como dice el ADR; ToolsaWeb 0,85 → 2,52, OpenDockTrading 2,12 → 3,19, TaskCodeTwo 1,22 → 2,22.
  - D: 5/5 aceptadas, 17/17 rechazadas.
- Las cifras citadas en el ADR (lineas 125, 177, 227, 233, 252, 253, 294, 305-306, 353, 601, 612-613) coinciden exactamente con esa salida. El log comiteado `poc/poc-f1-10.log` coincide en todas las cifras; solo cambia el nombre de la carpeta temporal.
- `bash .github/scripts/comprobar-repo.sh`: «Tareas: 93; enlaces relativos: 82; fallos: 0», exit 0.

## Notas (MENOR, no abren ronda)

- `docs/adr/F1-10-carriles-y-modo-automatico.md:331` dice «PoC C, 8/8», cifra de la revision 1. Es historico y no contradice la salida actual (14/14, que incluye los casos nuevos), pero conviene anotar que es la cifra de entonces para que no se lea como vigente.

## Lecturas fuera del contexto permitido

- `~/.claude/plugins/cache/taskcode-marketplace/taskcode-plugin/0.16.1/skills/task-workflow/revision.md`: reglas comunes de la revision (plantilla del informe, veredicto, rondas). Lo exige la skill revisora.
- `tareas/03-en-revision/TASK-011/revision/informe-revision-3.md`: para comprobar IMP-1 e IMP-2, como pide la peticion.

### MEN-1 — Nota de la revision 4 registrada como MENOR por quien orquesta

ADR:331 cita «PoC C, 8/8», cifra de la revision 1; el log actual da 14/14. Correccion: indicar que 8/8 es de la revision 1 y la cifra vigente.
