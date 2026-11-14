# Informe de revision — TASK-008 (ronda 1)

- Commit revisado: fe0cbabe6390cf053a0d08c3bd2a582ad0681f52
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md:285 | plan |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md:269 | plan |
| MEN-1 | MENOR | corregido | docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md:319 | implementacion |
| MEN-2 | MENOR | corregido | tareas/03-en-revision/TASK-008/poc/poc-f1-7.sh:299 | implementacion |
| MEN-3 | MENOR | corregido | docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md:292 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Pase generico completo (el diff es un ADR, una fila de indice y una PoC en Bash/Python; ningun dominio con revisor propio).

## Reproduccion de los hallazgos

**IMP-1. Un acierto de `completa` puede dar por bueno el check de la release sin correr la pesada.**
Decision 3 (lineas 285-290): con acierto, `verificacion` "corre en Linux y sale verde con reutilizado de <sha>"; la clave lleva solo `perfil`. La decision 4 (linea 299) dice que en el PR `develop`->`main` el mismo check corre completa (desde la cache) mas pesada. Las dos reglas chocan: en el PR de release el arbol de `V` coincide con el de `develop`, que ya tiene un check verde `perfil=completa`. La PoC B lo afirma ("PR de release develop->main ... acierto", `tareas/03-en-revision/TASK-008/poc/poc-f1-7.sh:160`, y la misma linea del log) y `buscar` (`poc-f1-7.sh:132`) devuelve acierto sin ningun parametro que exprese "ademas, la pesada". Resultado literal del ADR: el check obligatorio de `main` sale verde en segundos sin ejecutar la pesada. Tampoco se dice que `perfil=` escribe el check de una release: si es `completa`, una re-ejecucion del PR de release tambien acierta y salta la pesada; si es `pesada`, el acierto de `develop` nunca casa. Ningun caso de la PoC cubre release + pesada.
Correccion: en las decisiones 3 y 4, definir que el acierto solo exime del perfil `completa`; en un evento con pesada, `verificacion` corre siempre la `pesada` en su runner (o la busca con `perfil` y `clave` propios de la pesada), y el resumen escribe ambos perfiles. Anadir a la PoC B un caso negativo "verde `completa` de develop en el PR de release: la pesada no se da por hecha".

**IMP-2. La huella (y el runner) no pueden calcularse donde corre `buscar`, y D5a existe justo para eso.**
La clave = sha256(perfil | huella | version) (lineas 269-277) usa una huella que es "lo que la suite lee fuera del arbol" (versiones de herramientas, carpetas ignoradas como `exampleData`, 34 GB). Pero `buscar` corre en `ubuntu-latest` (divergencia 4, linea 186) para no despertar el runner, y es quien decide si `perfil` y `clave` son los actuales (lineas 285-286). En Linux no esta el `rustc` de la maquina de Carlos ni sus carpetas ignoradas: o la huella se calcula en el entorno equivocado (falso acierto justo en el caso que D5a/D5b dice evitar: cambiar de `rustc` o de datos reutilizaria un verde viejo), o hay que ejecutar la orden de huella en el runner del perfil, lo que contradice la razon de ser del job aparte. La PoC B trata la huella como un parametro (`huella='H1'`, `poc-f1-7.sh:132`): prueba la comparacion, no que se pueda calcular. Ademas el runner declarado por perfil (decision 5; C1 y C2 son alternativas reales para la misma completa) no entra en la clave: cambiar `github` por `self-hosted` reutiliza el verde del otro entorno. El ADR no lo recoge ni en "No verificado".
Correccion: decision 2/3: la orden de huella se ejecuta en el runner que correria el perfil (job corto en ese runner; si es self-hosted caido, el check ya queda pendiente por D3a, sin perdida) o se parte en una huella determinable desde el arbol (lockfiles, toolchain fijados) y otra local que se calcula con ese job; incluir el runner declarado en la clave; anadir a "No verificado" donde se calcula la huella y a la PoC B un caso "misma huella en Linux, distinta en el runner".

**MEN-1. E3 esta mal etiquetada y su salto a ejecuciones de CI no se declara como supuesto.**
La fila `E3` (linea 319) dice "maximo medido"; la fuente (`ToolsaWeb/docs/verificacion/flujo-de-verificacion.md:291`, abierta) dice "las rondas de 2,2 a 3,0" segun `taskctl metricas` sobre 153 tareas: es la evolucion de un promedio, no un maximo. Ademas el ADR usa rondas de revision como ejecuciones de CI por tarea (E3+S1), pero las rondas del agente antes de `review` son locales (decision 4); solo S1 figura como supuesto. Afecta a todas las filas de la tabla (3,5 ejecuciones).
Correccion: rotular E3 "rondas medias por tarea (taskctl metricas, 153 tareas)" y anadir un supuesto "1 ejecucion de CI por ronda" a las entradas y a "No verificado".

**MEN-2. La comprobacion de coste de la PoC es mas debil de lo que dice el ADR.**
Reproducido con mutaciones propias sobre copias del ADR (carpeta temporal, no el repo) con el `coste.py` de la PoC: (a) cambiar en la prosa "unas 145 horas" por "unas 99 horas" y "133,63 $ de 175,69 $" por "100,63 $ de 175,69 $": NO detectada (la PoC solo compara tablas); (b) cambiar E4 de `4 min 11 s` a `1 min 56 s` y recalcular a mano C4, T1 y T2 en la tabla: NO detectada, porque la comprobacion de fuente es `v in linea` (`poc-f1-7.sh:299`) y esa misma linea de `OpenDockTrading/.taskcode/config.yml:8` contiene ambas cifras; (c) E3 `3,0` -> `2,2` sin tocar la tabla: detectada, pero solo por la tabla (la linea 291 contiene `2,2`). El ADR afirma "11 entradas comprobadas contra su fuente" (linea 389): para E3 y E4 es una subcadena en una linea con varias cifras.
Correccion: anclar cada entrada a su contexto (regex con "en frio" para E4 y "rondas" para E3) o reducir la afirmacion a "la cifra aparece en la linea citada"; opcionalmente comprobar tambien las cifras de "Lectura".

**MEN-3. La tabla de eventos omite los eventos de los que depende la cache.**
La decision 4 (lineas 292-300) no tiene fila para `push` a `develop`/`main` ni para `merge_group`, pero la decision 3 y las PoC A/B basan aciertos en "push del merge commit en develop", "push del merge de la release en main" y la cola (RR-07). No se dice si esos eventos ejecutan `verificacion` ni con que perfil (F1-5 solo obliga el check en PR). Ademas el comando recibe `RAMA_BASE` en la `completa` (decision 1) sin exigir que el resultado no dependa de ella; si depende, el resultado cacheado por arbol deja de ser funcion del arbol (el propio ADR descarta `impacto` por eso).
Correccion: anadir filas `push` a `develop`/`main` y `merge_group` (completa, desde cache) y una frase "`completa` y `pesada` no pueden variar con `RAMA_BASE`".

## Verificacion realizada

- `bash poc-f1-7.sh` ejecutado por mi en `%LOCALAPPDATA%/pruebas-claude/taskcodetwo/task-008-r1/poc` (25 s, salida 0): A 10/10, B 10/10 y negativos 8/8, C1 9/9, C2 errores 0 y mutaciones 5/5, 1 llamada `gh api --method GET`, 0 de otro metodo, 3 descargas GET. Mi salida es identica al `poc-f1-7.log` del repo salvo la fecha y los SHA abreviados (dependen del instante de cada ejecucion).
- Fuentes abiertas (solo lectura): `ToolsaWeb/docs/verificacion/flujo-de-verificacion.md:224,263,291,299,307` (29 min 01 s; 22 min 22 s y 14 min 53 s del paso 18; rondas 2,2 a 3,0; hotfix sin pesados; 14 min 31 s), `ToolsaWeb/.taskcode/config.yml:9,16,18,23` (cuatro perfiles), `OpenDockTrading/.taskcode/config.yml:8` (4 min 11 s), `OpenDockTrading/scripts/verificar.bat:69-70` (`cargo audit`, `npm audit`). Todas coinciden. Recuentos 160 y 43 terminadas con las ventanas por `creado` reproducidos por la PoC.
- Tabla de coste recalculada a mano (581 y 903 ejecuciones; 13 363, 8 715, 581, 4 515 y 903 min; 19 362 y 14 714; 175,69 y 42,06 $; 145 h): correcta. Precios y redondeo estan en las paginas descargadas con GET (E7-E11 "en la pagina"), con fecha 2026-11-14.
- IDs citados existen: RR-07, RR-14, RR-15, RF-41, RF-42, RN-02, H2, H16, H18, P6 y F1-7, F1-8, F1-11, F1-13, F2-6, F3-8, F5-1..F5-6, F6-3, F9. Las cuatro vinetas de P6 estan en la tabla de evidencia.
- Coherencia con F1-5: `integration_id` 15368, `strict` en `develop` y no en `main`, perfil elegido por la base del PR coinciden; las divergencias 1-3 (completa en `develop`, la cache no abarata `strict`, pesada dentro de `verificacion`) estan declaradas. Con F1-3 (release por merge, arbol de `main` = `develop`) coincide (PoC A, release 2).
- Casos que el guion no cubre: acierto de `completa` en el PR de release sin pesada (IMP-1, a partir del propio caso de la PoC) y huella dependiente del runner (IMP-2, por analisis del flujo; no ejecutable sin GitHub: no verificado empiricamente). Mutaciones del ADR de MEN-2 ejecutadas.
- `bash .github/scripts/comprobar-repo.sh`: "Tareas: 93; enlaces relativos: 55; fallos: 0".
- No ejecutado: `taskctl doctor` (el ADR lo declara en la divergencia 9; el plan de pruebas T5 lo pedia al cerrar).

Criterios de `tarea.md`: coste con las suites de ToolsaWeb y OpenDockTrading, cumplido (con S1-S3 marcados como supuestos); cumple F1-7 y los requisitos citados, cumplido salvo IMP-1 e IMP-2, que afectan a la correccion de la cache que el ADR ofrece como evidencia de RR-07/RF-42.

## Lecturas fuera del contexto permitido

Motivo comun: comprobar fuentes citadas y coherencia con otros ADR. `docs/plan/PLAN-TASKCODE-2.md` y `docs/plan/REQUISITOS-FUNCIONALES.md` (existencia de IDs, P6, filas F5-x), `docs/auditoria/AUDITORIA-EXTERNA-2026-11-13.md` (H2, H16, H18), `docs/adr/F1-5-puertas-en-github.md` (strict, perfil, 15368), `tareas/03-en-revision/TASK-008/poc/poc-f1-7.sh` y `.log` (parte de la entrega), `tareas/03-en-revision/TASK-008/revision/peticion-revision-1.md`, y en solo lectura ToolsaWeb (`docs/verificacion/flujo-de-verificacion.md`, `.taskcode/config.yml`, `tareas/04-terminadas/*/tarea.md`) y OpenDockTrading (`.taskcode/config.yml`, `scripts/verificar.bat`, `tareas/04-terminadas/*/tarea.md`).
