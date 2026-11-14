# Informe de revision — TASK-001 (ronda 1)

- Commit revisado: bbc1447168d9d2e87e91cc470087ca097082ba57
- Revisor: code-quality-reviewer (agente general-purpose, modelo sonnet, independiente)
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | aceptado | .github/scripts/comprobar-repo.sh:198-219 | implementacion |
| MEN-2 | MENOR | aceptado | .github/scripts/comprobar-repo.sh:204-213 | implementacion |
| MEN-3 | MENOR | aceptado | .github/scripts/comprobar-repo.sh:146-148 | implementacion |
| MEN-4 | MENOR | aceptado | .github/scripts/comprobar-repo.sh:138-141 | implementacion |
| MEN-5 | MENOR | aceptado | .github/scripts/comprobar-repo.sh:100-111 | plan |
| MEN-6 | MENOR | aceptado | .github/workflows/ci.yml:1-29 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Alcance y puerta determinista: el diff es solo un script bash, un workflow y `.gitattributes`. No
hay build, linter ni suite de tests (la CI de código llega con F2-1). Como prueba empírica, el
revisor ejecutó el script sobre una exportación `git archive bbc1447` en
`%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-001-r1` (bash 5.3), ya borrada.

Línea base: sobre el repo real, `Tareas: 72; enlaces relativos: 11; fallos: 0`, rc=0.

Mutantes provocados, todos detectados con rc=1 y un mensaje claro: falta una carpeta de estado;
`estado:` que no corresponde a la carpeta; `id:` distinto de la carpeta; fichero sin `---` inicial;
frontmatter sin cierre (incluso en EOF); dependencia inexistente (en línea y en lista de bloque);
autodependencia; id duplicado; carpeta `TASK-*` sin `tarea.md`; enlace roto en README.md, imagen
rota, dos enlaces rotos en una línea, enlace con título; enlace con `..` fuera del repo y enlace
absoluto; raíz inexistente (rc=2).

Casos que correctamente no dan fallo: tareas con CRLF; `estado: "planificada"` con comillas;
enlaces `http(s)`, `mailto:` y `#ancla`; enlaces dentro de bloques ``` cerrados.

GitHub: el run de la rama (`ci`, 43545676298) está en `success` sobre `5443f0b`. `bbc1447` y
`f1a1969` solo añaden constancia y petición; no tocan el script. `.gitattributes` aplica `eol=lf`
al `.sh` (`git check-attr`), el modo en el índice es 100755 y el SHA de `actions/checkout`
(3d3c42e…) coincide con `v7.0.1` según `gh api`.

Sin CRÍTICO ni IMPORTANTE.

### MEN-1 — Enlaces en código en línea tratados como reales; un ``` sin cerrar silencia el resto
- Reproducción: ``Ver `[x](noexiste.md)` en codigo`` en README.md → `FALLO: enlace roto` (falso
  positivo). Un ``` sin cerrar seguido de `[x](noexiste.md)` → rc=0 (falso negativo). Abrir con ~~~
  y cerrar con ``` invierte el estado.
- Impacto: ruido en docs que documenten sintaxis de enlaces; un descuido de formato desactiva la
  comprobación del resto del fichero. No afecta a los docs actuales.

### MEN-2 — `%20`, esquemas no listados, `//` y paréntesis dan falsos positivos
- Reproducción: `[y](a%20b.md)` con `docs/a b.md` existente → enlace roto. `ftp://x`, `//cdn/x`,
  `tel:1` → enlace roto. `[x](foo_(bar).md)` se trunca → enlace roto.
- Impacto: un doc legítimo con esos destinos deja la CI en rojo; hoy no hay ninguno.

### MEN-3 — `dependencias:` escalar no se valida
- Reproducción: `dependencias: TASK-777` → rc=0 aunque TASK-777 no existe. Tampoco detecta ciclos
  (fuera del alcance del plan).
- Impacto: bajo, porque `taskctl` siempre escribe la forma de lista.

### MEN-4 — Comentario YAML al final de `id:`/`estado:` da un mensaje engañoso
- Reproducción: `estado: planificada # nota` → `falta el campo estado`. `id: TASK-002 # x` → 7
  fallos en cascada. Un BOM UTF-8 o un `tarea.md` vacío producen la misma cascada.
- Impacto: se detecta el fallo, pero el mensaje apunta a la causa equivocada.

### MEN-5 — Lo que no es `TASK-*` en `tareas/` se ignora sin aviso
- Reproducción: `tareas/05-extra/TASK-600` → rc=0. Una carpeta `task-002` deja de contarse y solo
  se nota por las dependencias rotas. Los enlaces por referencia (`[x][r]`) y las anclas `#x`
  inexistentes no se validan (el plan solo pedía «fichero que existe»).
- Impacto: la cobertura es algo menor de lo que sugiere la descripción. No bloquea nada.

### MEN-6 — La CI no se ha ejecutado sobre el commit revisado ni hay prueba de rojo en GitHub
- Reproducción: `gh run list --branch feature/task-001-f1-0-crear-taskcodetwo` → un único run,
  sobre `5443f0b`. `on: push:` sin filtros también dispara con etiquetas y se duplica con
  `pull_request` (decisión documentada en el YAML).
- Impacto: ninguno funcional; conviene confirmar el run verde del push final.

### Motivo de los `aceptado` (orquestador)

- MEN-1 a MEN-5: `comprobar-repo.sh` es transitorio. Las comprobaciones de `tareas/` se moverán o
  se retirarán con F1-1 (rama de estado), y la CI de código y la validación YAML estricta son F2-1
  y F2-5. Los casos reproducidos no se dan con lo que genera `taskctl` ni en los docs actuales, y
  corregirlos en bash añadiría complejidad a un script que tiene fecha de caducidad. Si un doc
  futuro tropieza con MEN-1 o MEN-2, se corrige entonces.
- MEN-6: se comprueba en el cierre que el run del push final queda en verde y se anota en el
  `## Resultado`. Que la CI se dispare en cualquier push es intencionado (cierre por merge local,
  sin PR).

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-001/revision/informe-revision-1.md`: plantilla del informe, pedida en el prompt.
- `.../taskcode-plugin/0.16.1/skills/task-workflow/revision.md`: reglas comunes de revisión, exigidas por la skill.
- `tareas/*/TASK-002/tarea.md` y `tareas/*/TASK-003/tarea.md`: para construir mutantes de frontmatter y dependencias.
- `docs/plan/PLAN-TASKCODE-2.md` y `README.md`: para provocar enlaces rotos.

## Incidente durante la revisión

Un primer intento de clonar falló por un error en el comando (la variable de la ruta temporal no
se resolvió), y el `git checkout -q bbc1447` posterior se ejecutó en el repo real, que quedó un
rato en HEAD desacoplado. El permiso automático denegó al revisor la vuelta a la rama. El
orquestador comprobó después que el repo está en `feature/task-001-f1-0-crear-taskcodetwo` en
`f1a1969`, limpio y sin stash: no hubo commits, push ni cambios en el árbol de trabajo.
