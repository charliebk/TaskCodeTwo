# Informe de revision — TASK-118 (ronda 2)

- Commit revisado: 29d24a7abacf092fb86a6d23d91ddc0b2cc75977
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/plataforma/gitlab.rs:1201 | implementacion |
| MEN-1 | MENOR | corregido | docs/adr/F1-5-puertas-en-github.md:629 | implementacion |
| MEN-2 | MENOR | corregido | docs/adr/F1-13-adaptador-de-plataforma.md:378 | implementacion |
| MEN-3 | MENOR | abierto | docs/adr/F1-5-puertas-en-github.md:641 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Alcance: revision incremental acotada a las correcciones de la ronda 1 (diff 34a6b7b..29d24a7 de
crates, tests y ADR). No se revisa el resto de la rama ni los MENORES de la ronda 1.

## Verificacion

Suite: no repetida. Se cita la ejecucion de la ronda: `cargo xtask verificar --perfil rapida` en verde
(exit=0), log `verif-118.log` (termina en «xtask: verificacion Rapida en verde»; marca `verif-118.log.fin`
con `exit=0`). El log no registra el sha; se comprobo que `git diff --stat ad859f7 29d24a7 -- crates docs`
esta vacio, asi que el codigo de 29d24a7 es el verificado.

Tests concretos en el clon (checkout 29d24a7, `--test-threads=4`):
- `cargo test -p tc2 --test plataforma_gitlab aplicar_puertas`: 2 passed
  (`aplicar_puertas_gitlab_corte_tras_proteger_converge`, `aplicar_puertas_gitlab_premium_y_carrera`). Log `r2-gitlab-original.log`.
- `cargo test -p tc2 --test init aplicar_puertas`: 4 passed
  (`aplicar_puertas_en_github_crea_las_que_faltan`, `aplicar_puertas_en_gitlab_protege_con_niveles_explicitos`,
  `aplicar_puertas_exige_personas_ramas_y_persona`, `aplicar_puertas_sin_origen_se_rechaza_sin_escribir`). Log `r2-init.log`.

Mutaciones (cada una se revirtio despues; el clon queda en 29d24a7 limpio):
- IMP-1: en `gitlab.rs:1201` se cambia `EstadoPuerta::Falta` por `Difiere` (proteccion igual al modelo y solo faltan ajustes o regla).
  Resultado: `aplicar_puertas_gitlab_corte_tras_proteger_converge` en rojo (panic en `plataforma_gitlab.rs:1043`). Log `r2-mutante-imp1.log`.
- Expectativa cambiada en init: la segunda ejecucion de GitLab (`init.rs:1608`) con `AJUSTES_FF` en vez de `AJUSTES_BIEN`.
  Resultado: `aplicar_puertas_en_gitlab_protege_con_niveles_explicitos` en rojo (panic en `init.rs:1616`, aserto de `sin-cambios`).
  El test sigue aseverando algo real: con protecciones y ajustes del modelo, ninguna escritura. Log `r2-mutante-init.log`.

Hallazgos de la ronda 1:
- IMP-1, corregido. Con el caso de la ronda 1 (corte tras proteger `develop`, `PUT` en 500), la repeticion
  completa `develop` con un `PUT /projects` y un `POST /approval_rules`, no repite el `POST` de `develop` y
  la linea sale `aplicadas=` con `develop`. Ver la mutacion anterior: sin la correccion, el test vuelve a rojo.
- MEN-1, corregido. La adenda de F1-5 (lineas 629-632) dice que un 422 de GitHub cuenta como existente solo si
  la relectura de `GET /rulesets` trae el nombre. Coincide con `github.rs:820-826`.
- MEN-2, corregido. La fila de `aplicar-puertas` de F1-13 incluye `GET /user` (ambos adaptadores) y
  `GET /approval_rules` en Premium. Coincide con `gitlab.rs:141`, `gitlab.rs:168` y `github.rs:80`.

Hallazgo nuevo (introducido por la correccion, MENOR):
- MEN-3, `docs/adr/F1-5-puertas-en-github.md:641`. La adenda dice que `PUT /projects/:id` con
  `merge_method=merge` y `only_allow_merge_if_pipeline_succeeds=true` se hace «cuando se crea `develop` o `main`».
  Tras la correccion tambien se hace cuando la proteccion ya es la del modelo y solo faltan los ajustes
  (`gitlab.rs:1201` ya no marca `Difiere` en ese caso). Reproduccion: en el test
  `aplicar_puertas_gitlab_corte_tras_proteger_converge`, la ronda 2 con `develop` ya protegida y el proyecto en `ff`
  escribe un `PUT` (aserto de un solo `PUT`). Ademas `plataforma/mod.rs:332` sigue diciendo «Nunca reescribe ni borra»
  y ese `PUT` cambia `merge_method` y el pipeline de un proyecto que pudo tenerlos puestos a mano. No hay perdida de
  datos: son los ajustes que el modelo exige. Sugerencia: decidir y documentar. O la ADR dice que `aplicar-puertas`
  fija tambien los ajustes del proyecto cuando la proteccion ya es la del modelo, y el comentario de `mod.rs` se
  matiza («no reescribe protecciones»), o los ajustes ajenos se marcan `difiere`.

Veredicto propuesto (no escrito en el fichero, que queda PENDIENTE): aprobada-con-correcciones, por MEN-3.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/plataforma/gitlab.rs` (lineas 1000-1100 y 1140-1230, `aplicar_puertas` y `leer_para_aplicar`): el diff
  de la peticion no basta para reproducir IMP-1; hace falta el flujo de escritura y el estado `Falta`/`Difiere`.
- `crates/tc2/src/plataforma/github.rs` (lineas 815-835, y busqueda del 422 en la linea 429): comprobar el 422 que describe MEN-1.
- `crates/tc2/src/plataforma/mod.rs` (linea 332): el comentario «Nunca reescribe ni borra» que contrasta con MEN-3.
- `crates/tc2/tests/plataforma_gitlab.rs` (lineas 905-1080): los tests de la ronda 2 y el de corte, para la mutacion y el aserto de `PUT`.
- `crates/tc2/tests/init.rs` (diff de la peticion, lineas 1504-1630 leidas): el test de GitLab cuya expectativa cambio, para la mutacion.
- `docs/adr/F1-5-puertas-en-github.md` (lineas 625-650): la adenda que citan MEN-1 y MEN-3.
- Solo busqueda (grep), sin leer el cuerpo: `crates/tc2/src/plataforma/gitlab.rs` (`Endpoint`, lineas 141, 168, 199),
  `crates/tc2/src/plataforma/github.rs` (linea 80) y `crates/tc2/src/plataforma/personas.rs` (linea 264), para confirmar
  las lecturas de MEN-2.
- Logs en `pruebas-claude` fuera del repo: `verif-118.log` y `verif-118.log.fin`, mas los logs de este pase (`r2-*.log`).
