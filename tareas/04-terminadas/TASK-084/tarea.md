---
id: TASK-084
titulo: "F3-12 Progreso en vivo: fases que faltan y tiempo restante"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-084-f3-12-progreso-en-vivo-fases-que-faltan
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1559802
tokens_implementacion: 58903472
tokens_revision: 13953784
creado: 2026-11-14
actualizado: 2026-11-18
dependencias: [TASK-018, TASK-083]
---
## Objetivo

Progreso en vivo en el terminal: fase actual, **fases que faltan** y **tiempo estimado restante**, en la status line de Claude Code y en `taskctl status --seguir`, escrito por el CLI (RR-19)

Tarea F3-12 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F3-13, F2-3.

Contexto (Carlos, 2026-11-14, RR-19, imprescindible): mientras se trabaja lo importante no es un porcentaje sino qué fases faltan y el tiempo estimado de cada tarea, como información para el programador, visible en el terminal de trabajo (status line de Claude Code y `taskctl status --seguir`), escrito por el CLI sin gastar tokens; en un lote, además, tareas que faltan y hora estimada de fin. Al cerrar, métricas finales: estimado frente a real por fase, tokens, rondas y hallazgos, con la calibración de la estimación en el informe por versión. Base en la 0.16: `taskctl estado [--seguir]`, la status line (fase, paso, x/N, minutos) y la «Duración estimada» del plan. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 quater.

## Criterios de aceptacion
- [x] C1: `tc2 estado --stdin` con `{"workspace":{"current_dir":<worktree>}}` no lanza procesos ni abre red (su firma no recibe `Git`); con 10 y con 1000 tareas en la rama de estado da la misma salida leyendo los mismos ficheros; tiempo medido en Windows anotado en el Resultado y el test falla por encima de 200 ms
- [x] C2: una tarea en curso con muestras sinteticas conocidas y 20 min transcurridos da exactamente la linea esperada (fase, fases que faltan, ≈mediana, p80, n y nivel)
- [x] C3: en un E2E con remoto local, tras `start`, `pausa` y `reanudar` el fichero se reescribe, la linea cambia y el anterior queda en `.anterior.json`; tras `descartar` o `liberar` la linea dice «sin tarea en curso»
- [x] C4: `calcular` sobre una tarea `en-revision` da la fase rondas con solo «cierre» por delante
- [x] C5: sin tareas parecidas la linea dice `sin estimacion (n=N)`
- [x] C6: pasado lo de todas las parecidas la linea dice `≥X min (fase en curso por encima de sus parecidas)`
- [x] C7: sin tarea en curso o fuera de un repo sale `tc2 · sin tarea en curso` con codigo 0 y stderr vacio
- [x] C8: un fichero corrupto o a medias da el ultimo valor bueno con su antiguedad
- [x] C9: `tc2 status --seguir` imprime la linea inicial, vuelve a imprimir al reescribirse el fichero y termina cuando `parar()` es verdadero (test de biblioteca con reloj inyectado); en el test de proceso, la salida contiene las dos lineas
- [x] C10: avisos con caracteres de control o bidi salen saneados y con longitud acotada
- [x] C11: ninguna orden nueva usa modelos ni red; `init` propone la status line y deja `settings.json` igual byte a byte

## Autorrevision

- rust-wasm-reviewer — 37f97c1
- code-quality-reviewer — 37f97c1

Hallazgos propios (ninguno CRITICO ni IMPORTANTE de esta tarea):

- Fuera de esta tarea (anterior): `estado::transaccion` con 1000 ficheros
  en una sola operacion falla con `git=hash-object` (probable longitud de la
  linea de ordenes en Windows); afectaria a un `import` muy grande. Pasa a
  una tarea nueva.
- MENOR: Ctrl+C en `status --seguir` no sale con codigo 0 (sin manejador de
  senales); con stdout cerrado si.
- MENOR: `liberar`/`reasignar` desde otro clon no borran el fichero de
  progreso del clon de la persona anterior hasta su siguiente transicion.
- MENOR: `tc2 desinstalar` solo quita la `statusLine` de
  `~/.claude/settings.json`; la de `.claude/settings.local.json` (la
  recomendada) la quita la persona (documentado en `instalar.md`).
- Desviaciones del plan: `start` escribe el progreso dentro de la toma con el
  mismo historico (para mantener un solo `git log`); `init` propone con
  `decision=pendiente motivo=pegar-a-mano`; `--seguir` recalcula cada 2 s;
  tope de 45 min en el hueco abierto (divergencia de F1-17 punto 5, anotada).
- Tiempo de `tc2 estado --stdin`: mediana 17 ms en release (15,7-18,8 ms),
  83 ms en frio; 18-20 ms con 10 y con 1000 tareas.
- Mutaciones comprobadas (8, rojo y restauradas): reintento del `rename`,
  tope del hueco abierto, vuelta al anterior, saneado de avisos, fallo del
  refresco que cambia la transicion, `commondir`, rama desbordada, pausa que
  congela.
- Verificacion propia: `--test progreso --test estimacion` 19 de 19.

## Resultado

Hecho: modulo `progreso/` (F1-17 para F3-12). Las transiciones (start, descartar, pausa, reanudar, liberar, reasignar) escriben `.git/taskcodetwo/progreso/TASK-NNN.json` (anterior en `.anterior.json`) con fase, fases que faltan y una tabla de lo que falta sacada de `estimacion::restante` (sin segunda formula); un fallo al escribirlo nunca cambia la transicion (`progreso=no-escrito`). `tc2 estado --stdin` (la orden de status line ya fijada en `distribucion::STATUS_LINE`) lee ese fichero sin red ni subprocesos (su firma no recibe `Git`) y da una linea con el formato L5 de F1-17: `TASK-NNN · <fase> · fases que faltan: ... · faltan ≈X min (p80 Y) · n=N <nivel>`, con variantes desbordada, sin estimacion, pausada, sin actividad, saturada (mas de 480 min) y sin tarea; siempre codigo 0, tambien con stdout cerrado. Medido en Windows: ~17 ms en caliente (83 ms en frio), igual con 10 que con 1000 tareas. `tc2 status --seguir` repinta cada 2 s y, sin terminal, repite la linea cada 4 s para enterarse del cierre de stdout y terminar. `init` propone la `statusLine` sin tocar ningun `settings.json`; `instalar.md` lo documenta con las citas oficiales (status line y ajustes, donde se lee `settings.local.json`, version v2.1.211 y excepciones, incluida Windows). `huella::escribir_local` (temporal + rename con reintento) lo usan progreso, frescura e identidad. Adenda de F1-17 (fichero en `.git/taskcodetwo/progreso/` en vez del gitdir del worktree; tope de 45 min en el hueco abierto; repeticion de `--seguir`; saturada). D1A-D4A. `cargo xtask verificar --perfil rapida` en verde.

No verificado: durante una tarea real (las tareas se hacen con la 0.16 y faltan `review` y `finish` en la 2.0): queda para F9-7; la fase de rondas solo con datos de prueba. Ctrl+C en `--seguir` no sale con 0. Liberar/reasignar desde otro clon no borran el fichero del clon anterior hasta su siguiente transicion. `desinstalar` no quita la status line de `.claude/settings.local.json` (documentado).

Revision ronda 1: rust-wasm-reviewer cambios-solicitados (R1-01 IMPORTANTE: `--seguir` no terminaba al cerrarse stdout con la linea fija; R1-02 a R1-05 MENORES: saneado bidi, `estado --stdin` con stdout cerrado, fallo del anterior que impedia escribir, saturacion sin aviso); code-quality-reviewer aprobada (M1 a M4: tramo a mitad, frontera de 45 min, cita de `settings.local.json`, test de `reasignar` y carpeta administrativa del worktree). Corregidos en 8d43627 y f44ce49. Ronda 2: aprobada por los dos (573 tests del workspace en verde; R1-01 reproducido con el binario: termina en 4 s); N1 (cita y version de donde se lee `settings.local.json`) corregido en ed42bd0 y verificado con `verificar-menores`.

Hallazgo fuera de esta tarea: `estado::transaccion` con 1000 ficheros en una operacion falla con `git=hash-object` en Windows (probable longitud de linea de ordenes); pasa a una tarea nueva.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-18T02:49:02Z | plan | automatico | persona | 0.16.1 |
| 2026-11-18T02:55:29Z | approve | automatico | persona | 0.16.1 |
| 2026-11-18T02:55:30Z | start | automatico | persona | 0.16.1 |
| 2026-11-18T03:58:33Z | review | automatico | persona | 0.16.1 |
| 2026-11-18T04:39:39Z | review | automatico | persona | 0.16.1 |
| 2026-11-18T05:18:44Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-18T02:55:28Z | diseno | agente:a0d38b427619c51cc | 46 | 106136 | 1429509 | 24111 | 39 | 1559802 | - | - |
| 2026-11-18T03:58:31Z | implementacion | agente:abf6bb12d06fab0e2 | 310 | 445019 | 46720462 | 134065 | 76 | 47299856 | - | - |
| 2026-11-18T03:58:33Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 8 | 633875 | 1990572 | 3989 | 0 | 2628444 | - | - |
| 2026-11-18T04:11:31Z | revision | agente:ab4ba9ab6dcade8d3 | 48 | 133889 | 2696334 | 19300 | 5 | 2849571 | 1 | rust-wasm-reviewer |
| 2026-11-18T04:13:40Z | revision | agente:a16c9ebcc7a604819 | 82 | 168175 | 5047820 | 19105 | 10 | 5235182 | 1 | code-quality-reviewer |
| 2026-11-18T04:39:39Z | implementacion | agente:a7182021395bba7bc | 138 | 364319 | 8565118 | 45597 | 15 | 8975172 | - | - |
| 2026-11-18T04:49:47Z | revision | agente:a999dcc8914feedd9 | 42 | 116468 | 1777119 | 11666 | 2 | 1905295 | 2 | code-quality-reviewer |
| 2026-11-18T05:07:04Z | revision | agente:a9aa90e0b8aad0c71 | 84 | 109863 | 3843554 | 10235 | 10 | 3963736 | 2 | rust-wasm-reviewer |

## Verificacion ligera

- 2026-11-18T05:07:22Z: ronda 2; commit revisado f44ce49bb8f74f58264b879674eb039a7a5492f6; commit verificado 6e85a671a4a66e41c37d20ac524b554bb8478b05; ficheros: docs/distribucion/instalar.md; corregidos: N1; pendientes: ninguno

## Duracion

- Estimada: 300 min
- Real (de la rama abierta al cierre con merge, sin pausas): 143 min
- Paso de la hora: 143 min reales frente a 300 estimados
- Cierre (finish): 663 s, de 2026-11-18T05:07:41Z a 2026-11-18T05:18:44Z
