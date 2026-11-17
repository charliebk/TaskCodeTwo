---
id: TASK-081
titulo: "F3-10 Liberar y reasignar tareas abandonadas"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-081-f3-10-liberar-y-reasignar-tareas-abandon
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2790273
tokens_implementacion: 53213952
tokens_revision: 11437472
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-033]
---
## Objetivo

Liberar y reasignar tareas: `taskctl liberar` y `reasignar`, aviso de tareas tomadas sin actividad en N días, con aviso a quien la tenía (RR-12)

Tarea F3-10 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F3-6.

Contexto (Carlos, 2026-11-14, RR-12 reforzado): el trabajo con varias personas debe estar garantizado: todos saben qué tarea tiene cada uno y no se pueden pisar ni los nombres ni las tareas. Además de lo ya previsto (IDs por push atómico, toma atómica en start, roadmap con quién lleva cada tarea, status), se cierran los huecos: nombres de rama y títulos únicos, identidad real de la plataforma (no texto libre), liberar y reasignar tareas abandonadas, aviso de solape de ficheros entre tareas en curso y estado siempre fresco con hora de sincronización. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 ter.

## Criterios de aceptacion
- [x] C1: quien lleva una tarea `en-curso` lanza `tc2 liberar TASK-NNN`: se publica `estado: planificada`, `asignado_a: null` y la fila `liberar`, con `rama:` y `plan_aprobado` sin cambios y el resto del `tarea.md` byte a byte; `ROADMAP.md` del mismo commit; codigo 0 con `asignado-anterior=`; la rama local y la remota siguen en el mismo commit
- [x] C2: tras C1, otra persona en otro clon hace `tc2 start TASK-NNN` y sale `ok` (no `ocupada`); su rama local apunta a la punta de `origin/<rama>` con los commits del dueno anterior en su historia y el worktree en `.taskcodetwo/worktrees/TASK-NNN`
- [x] C3: `tc2 reasignar TASK-NNN beto` con `beto` en `personas.yml` solo cambia `asignado_a` a `{id, login}` de la lista y anade la fila `reasignar`, sin cambiar el estado; `beto` hace `start` y obtiene la rama existente sin perder commits
- [x] C4: reasignar a un login que no esta en `personas.yml`, mal escrito o bot da 3 `motivo=fuera-de-lista` sin commit en `taskcodetwo/tareas`
- [x] C5: tarea de otra persona con actividad hace menos de N dias: `liberar` y `reasignar` dan 3 `motivo=activa dias=K umbral=N` sin escribir; con N dias o mas (filas y commit de la rama envejecidos) proceden
- [x] C6: lanzado sin terminal (agente) sobre una tarea de otra persona da `motivo=requiere-persona` aunque haya pasado N; sobre una tarea propia procede
- [x] C7: sin red `liberar` y `reasignar` dan 4 `sinred pendiente=no` sin commit ni ref en `refs/taskcodetwo/pendientes/`
- [x] C8: el dueno con commits sin publicar o cambios sin guardar en el worktree recibe `motivo=sin-publicar` antes de la transaccion con el estado intacto; publicado y limpio sale `worktree=quitado` y la rama sigue
- [x] C9: `status` muestra `inactivas=1` cuando una tarea en curso lleva N dias o mas sin actividad y `board` marca su fila; un commit reciente en `origin/<rama>` evita la marca aunque la ultima fila sea antigua
- [x] C10: en el clon del dueno anterior, `status` avisa `ya-no-tuyas=TASK-NNN` con quien la lleva ahora, codigo 0
- [x] C11: `liberar` de `terminada`, `descartada` o `en-revision` da su rechazo por estado; repetir `liberar` sobre una ya libre da `sin-cambios`
- [x] C12: una tarea con `asignado_a` heredado de la 0.16 se reasigna a una persona de la lista y queda como objeto `{id, login}`

## Autorrevision

- rust-wasm-reviewer — bfc11c6
- code-quality-reviewer — bfc11c6

Hallazgos propios (ninguno CRITICO ni IMPORTANTE):

- MENOR: `grafo::listas` no mira el asignado: una `planificada` reasignada
  a otra persona sigue apareciendo como lista para empezar (fuera del
  alcance).
- MENOR: con la configuracion ausente o invalida el umbral cae en silencio
  a 7 dias.
- MENOR: «sin perder su trabajo» solo vale para lo publicado; carrera entre
  el `fetch` y el push del estado; `committerdate` falseable (frena, no
  adelanta). Documentados en la adenda de F1-2.
- Desviaciones del plan: `comprobar_id_salvo` en vez de cambiar la firma de
  `comprobar_id`; la anotacion lleva tambien los dias; `liberar` acepta
  `planificada` (quita el asignado o `sin-cambios`); `sin-publicar` tambien
  en `reasignar`; actividad desconocida no frena; un heredado exige persona
  con terminal.
- Incidencia de proceso: el primer commit salio con un mensaje ajeno (fichero
  de mensaje reutilizado del scratchpad) y el agente lo corrigio con
  `git commit --amend` antes de publicar; solo cambio el mensaje.
- Mutaciones comprobadas (8, rojo y restauradas): `ramas_ajenas` descarta
  todas, sin `activa`, sin `requiere-persona`, actividad sin commit de la
  rama, sin `sin-publicar`, `por_login` acepta cualquiera, `ya_no_tuyas`
  invertido, `inactivas` sin commit de la rama.
- Verificacion propia: `--test liberar --test tablero` 18 de 18.

## Resultado

Hecho: `tc2 liberar [--json] TASK-NNN` y `tc2 reasignar [--json] TASK-NNN <login>` (modulo `ciclo/liberar.rs`, forma de `descartar`: `fetch` completo, identidad, comprobaciones previas, transaccion con ROADMAP en el mismo commit, worktree quitado solo si esta publicado; la rama nunca se toca). `liberar` devuelve `en-curso` a `planificada` (y quita el asignado de `planificada`/`en-diseno`), conserva `rama` y `plan_aprobado`; `reasignar` cambia solo el asignado a una persona de `personas.yml` (fuera de la lista: `fuera-de-lista`). El dueno libera o reasigna siempre; otra persona solo con N dias o mas sin actividad (maximo de la ultima fila de Transiciones y del ultimo commit de `origin/<rama>`) y con terminal; el agente nunca quita una tarea; un heredado de la 0.16 lo reasigna cualquier persona verificada. `start` acepta la rama propia de la tarea (`comprobar_id_salvo`) y la retoma sin perder commits; una rama ajena sigue dando `ocupada`. `status`/`board` avisan `inactivas=N` y `ya-no-tuyas=TASK-NNN` al dueno anterior. Clave `dias_sin_actividad` (1-365, defecto 7; configuracion invalida se rechaza en `liberar`/`reasignar` y no marca inactivas en `status`/`board`), `TOPE_CLAVES` 13. Adendas en F1-2, F1-3, F1-9 y F1-11 y nota en el plan (F2-7). D1a-D6a. `cargo xtask verificar --perfil rapida` en verde.

No verificado: plataforma real (sin aviso por PR, D5a); carrera real entre dos clones (sin test concurrente; la transaccion reejecuta sobre el estado fresco). «Sin perder su trabajo» solo vale para lo publicado.

Revision ronda 1: aprobada por los dos, sin CRITICOS ni IMPORTANTES. Siete MENORES corregidos en 2c41d54, 8605feb y 5663e3c y verificados con `verificar-menores`: H1 test de rama propia frente a ajenas del mismo slug o tipo; H2 actividad con fecha mas de un dia en el futuro se trata como desconocida; M-1 borde `d == umbral`; M-2 reasignarse a si misma no toca su worktree; M-3 regla D2a en una sola funcion (`actividad_conocida`) usada por `liberar` y `tablero`; M-4 configuracion invalida ya no cae en silencio a 7 (lector renombrado `umbral_de_inactividad`); M-5 nota del tope 13 en el plan. Redaccion de C9 alineada («N dias o mas»).

Pendiente fuera de esta tarea: `situacion::actividad` quedo sin usos fuera de sus tests y su comentario dice que una fecha futura cuenta como de hoy; quitarla exige tocar `situacion.rs`, que ningun MENOR citaba, y pasa a una tarea de seguimiento.

Incidencias de proceso: el primer commit de la implementacion salio con un mensaje ajeno y se corrigio con `--amend` antes de publicar; durante el cierre el orquestador cambio por error a `develop` con el arbol limpio y volvio a la rama sin perdida.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T21:53:28Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T22:01:41Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T22:01:42Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T23:09:27Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T22:01:40Z | diseno | agente:a16270a0169d2167e | 66 | 118729 | 2645939 | 25539 | 49 | 2790273 | - | - |
| 2026-11-17T23:09:26Z | implementacion | agente:a019c2a1aaf7a8068 | 270 | 823378 | 40736583 | 130780 | 69 | 41691011 | - | - |
| 2026-11-17T23:09:27Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 18 | 17249 | 3241487 | 4022 | 0 | 3262776 | - | - |
| 2026-11-17T23:29:57Z | revision | agente:a6656bc53b7b81e1d | 82 | 148606 | 4833321 | 16945 | 4 | 4998954 | 1 | rust-wasm-reviewer |
| 2026-11-17T23:38:33Z | revision | agente:ac79365609c3536e4 | 98 | 176837 | 6233492 | 28091 | 5 | 6438518 | 1 | code-quality-reviewer |
| 2026-11-17T23:53:54Z | implementacion | agente:ae5d5f74ba4b10e3b | 138 | 137102 | 8083437 | 39488 | 17 | 8260165 | - | - |

## Verificacion ligera

- 2026-11-17T23:53:55Z: ronda 1; commit revisado 6f41ef2e6595a749fa6ea41439152efc1dfb5182; commit verificado 5663e3c875e4eda6fe735271b25e5ef2b9663cd8; ficheros: crates/tc2/src/ciclo/liberar.rs, crates/tc2/src/ciclo/mod.rs, crates/tc2/src/git/rama.rs, crates/tc2/src/tablero.rs, docs/plan/PLAN-TASKCODE-2.md; corregidos: M-1, M-2, M-3, M-4, M-5, H1, H2; pendientes: ninguno
