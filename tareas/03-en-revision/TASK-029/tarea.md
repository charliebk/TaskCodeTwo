---
id: TASK-029
titulo: "F3-2 tc2 new e import en bloque con dependencias e hito"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-029-f3-2-new-e-import-en-bloque-con-dependen
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: 24481628
tokens_revision: 5293913
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-019, TASK-020]
---
## Objetivo

`new` e `import` en bloque con dependencias e hito (RF-01..03)

Tarea F3-2 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-4, F2-5.

## Criterios de aceptacion
- [x] C1: `tc2 import hito37.md` sobre un remoto bare da un commit, 37 carpetas con IDs consecutivos y cada `tarea.md` con su sprint, objetivo y criterios (`crates/tc2/tests/importar.rs`)
- [x] C2: las dependencias locales (clave del fichero) quedan resueltas a `TASK-NNN` y las externas validadas; un ciclo o una referencia inexistente dan codigo 2 (uso) sin commit (la rama de estado no cambia de SHA)
- [x] C3: repetir C1 da codigo 0 con `creadas=0 existentes=37` y ningun commit nuevo
- [x] C4: el mismo fichero con 2 entradas mas crea solo esas 2, con sus dependencias hacia las ya importadas resueltas
- [x] C5: una entrada invalida en la posicion 30 no escribe nada; dos clones importando a la vez dan IDs sin solape y sin huecos dentro de cada lote
- [x] C6: `tc2 import --simular` imprime los IDs previstos y no cambia ni un byte en local ni en el remoto
- [x] C7: pasar los limites (1 MiB, 200 entradas) da `uso` con su `motivo=`; sin red da `sinred` con `pendiente=no`
- [x] C8: el fichero de import de formato 0.16 de `docs/plan/AMPLIACION-2026-11-16.md` §5 se importa sin tocarlo
- [x] C9: `tc2 new` acepta los campos de RF-01 y rechaza un enunciado que no esta listo (test)

## Autorrevision

- rust-wasm-reviewer — a2232d9
- code-quality-reviewer — a2232d9

## Resultado

Hecho: nucleo `altas` comun a `new` e `import` (reserva de N IDs consecutivos, rama derivada por tipo, aviso de parecidas); parser puro del fichero de import (`importar/fichero.rs`: formato 0.16 mas cabecera YAML de lote y linea `Dependencias:`, limites de 1 MiB y 200 entradas, claves unicas, ciclos); `tc2 import [--json] [--simular] [--sprint N] <fichero>` como una sola operacion atomica sobre la transaccion de F2-3, idempotente por `lote/clave` (clave opcional `lote` en el modelo, adenda en F1-9); `--simular` sobre el ultimo estado traido sin fetch (lo dice con `base=`); `tc2 new` con los campos de RF-01 y rechazo de un enunciado no listo (sin criterios, criterio vacio o repetido, hueco `<...>` de la plantilla o caracteres de control). Un test por criterio C1..C9 (`tests/importar.rs`, `tests/alta.rs`); `cargo xtask verificar --perfil rapida` en verde.

No verificado: el perfil `completa`; aviso de parecidas en `import` sin E2E propio (nucleo compartido con `new`, probado alli). `--simular` hace `ls-remote`: sin red da `denegada`/`sinred`.

Revision ronda 1 (rust-wasm-reviewer y code-quality-reviewer): aprobada por los dos, sin CRITICOS ni IMPORTANTES, con mutaciones en rojo y prueba manual contra un bare en es y en.

MENORES aceptados (van a seguimiento, TASK-120): cabecera ignorada sin aviso con `--- ` o linea en blanco previa; sin cabecera, el lote sale del slug del nombre del fichero (renombrar duplica, nombres distintos pueden dar el mismo slug); `tarea.md` ilegible hace que un import repetido la vuelva a crear; dependencia a tarea descartada aceptada sin aviso; error de `TASK-NNN` externa sin `referencia=`/`linea=`; sprint 0 sin aviso cuando no hay cabecera ni `--sprint`.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T04:56:46Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T05:00:32Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T05:00:33Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T05:38:56Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T05:38:56Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 8 | 9429 | 2515094 | 1977 | 0 | 2526508 | - | - |
| 2026-11-17T05:38:56Z | implementacion | agente:a7c771d9486edd75c | 200 | 322444 | 21509326 | 123150 | 45 | 21955120 | - | - |
| 2026-11-17T05:54:16Z | revision | agente:a6b9d1c127cf68861 | 32 | 113381 | 1323137 | 12456 | 1 | 1449006 | 1 | rust-wasm-reviewer |
| 2026-11-17T05:54:17Z | revision | agente:a29912eb034a2088b | 70 | 145905 | 3670405 | 28527 | 1 | 3844907 | 1 | code-quality-reviewer |
