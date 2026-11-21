---
id: TASK-046
titulo: "F5-6 Hook local rapido, opcional y con confianza"
tipo: feature
sprint: 5
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-046-f5-6-hook-local-rapido-opcional-y-con-co
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1086279
tokens_implementacion: 21548462
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-21
dependencias: [TASK-009]
---
## Objetivo

Hook local rápido, opcional y con confianza

Tarea F5-6 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-8.

## Criterios de aceptacion
- [x] ≤ 1 s
- [x] nada del repo sin confianza
- [x] Cumple lo que pide la tarea F5-6 del plan y los requisitos que cita

## Resultado

Hook local rapido, opcional y con confianza (F5-6): un comando determinista y el hook como forma de llamarlo.
- **Confianza local** (`crates/tc2/src/confianza/`): `estado` y `confiar` por clave, con la huella `sha256` de la orden, la shell y el cierre de scripts versionados que nombra (tokens, rutas entre comillas, `%~dp0`, nombres raros por su nombre literal) mas los globs de `confianza_incluye`. Cache por `(tamano, mtime)`. Almacen fuera del arbol (`%LOCALAPPDATA%\taskcodetwo\confianza\` o XDG), un fichero por repo comun (`dir_comun`): los worktrees comparten confianza y otro clon no.
- **`tc2 confiar verificacion.comando`** y **`tc2 verificar --perfil <p>`**: sin confianza, o con la orden o un script cambiados, no se ejecuta nada (codigo 3 y «ejecuta `tc2 confiar verificacion.comando`»). Un glob invalido es error de la config y del cierre; uno que no casa se avisa.
- **`--hook`** (evento `Stop` de Claude Code, entrada documentada en `settings.json`, sin orden de instalacion): solo bloquea (2) con la rapida en rojo; sin confianza, con la config mal, con un error de uso o con `stop_hook_active` sale con 0 y una linea. En `rapida` un timeout no es rojo.
- **`desinstalar --maquina`** borra tambien el almacen de confianza.
- **Guia:** seccion F5-6 de `docs/ci/verificacion-reutilizable.md` (entrada del hook, codigos, como se forma el cierre, limites declarados y cifras).

Decisiones de Carlos (2026-11-20): todas las recomendadas (D1a a D6), sin trocear.

Evidencia (2026-11-21):
- **≤ 1 s** (P1, release, Windows 11, `tc2 verificar --perfil rapida` entero, cierre de 51 scripts y 1,2 MB como ToolsaWeb): sin confiar 284-401 ms, en frio 297-368 ms, en caliente 234-292 ms; con un script de nombre raro mas, 239-319 ms. La revisora de la ronda 3 midio 338 / 297 / 234 ms.
- **Nada del repo sin confianza:** P3-P5 y los tests de las rondas (nombres con espacios y no ASCII, apostrofo suelto, comillas anidadas, globs `./` y mayusculas, raro en la orden) contra repos git temporales reales; cada uno se pone rojo con su mutacion (11 en la implementacion, 15 en la ronda 1, 3 en la 2, 2 en la 3).
- **Tests:** `cargo test -p tc2 --test verificar` 15 en verde y 1 ignorado (P1, pesado); `--test desinstalar` 19; verificacion completa en verde sobre e33879a; la de `finish` cubre el resto.
- **Suposiciones del plan:** el JSON de `Stop` trae `stop_hook_active` (documentacion oficial de hooks); `desinstalar` no borraba el almacen (corregido); F0-2 no tiene tarea en este repo (F5-6 implementa la confianza).

**No verificado:** la entrada de `settings.json` dentro de un Claude Code real (solo el comando con el JSON por stdin); Linux y macOS (todo se probo en Windows 11); enlaces simbolicos (sin privilegio para `mklink`). Este repo aun no puede estrenar el hook: su config es de la 0.16 y `tc2 verificar` la rechaza (con `--hook` sale con 0 y avisa).

**Limites declarados en la guia** (no se corrigen; la salida es nombrar el script en `confianza_incluye`): scripts no versionados, enlaces y submodulos, cambio que conserva tamano y `mtime` (D2a), TOCTOU, nombres con escape (`\ `, `^`) o formados con variables, referencias sin extension y scripts UTF-16.

Revision por pares (tres rondas; la 3 es extra, ver «## Rondas extra»):
- **Ronda 1:** code-quality aprobada con correcciones (IMP-1: un error de uso con `--hook` bloqueaba la parada; MEN-1 a MEN-5); rust-wasm pidio cambios (IMPORTANTE-1: scripts con espacios o no ASCII fuera del cierre; IMPORTANTE-2: globs invalidos o `./` ignorados en silencio; MENOR-1 a 4). Todo corregido en c6ffbc1, 8fd2d67 y f7a9be6, salvo MENOR-3 y MENOR-4 de rust, aceptados y documentados.
- **Ronda 2:** rust aprobada (MENOR-5 corregido en 1c9f9e9); code-quality pidio cambios por CRITICO-1, introducido por la correccion (un apostrofo suelto o comillas anidadas dejaban fuera un script con espacios): corregido en 1c9f9e9 con la entrada por nombre de los scripts raros. MEN-6 (`create_new` sin test) aceptado: no es observable desde fuera y el nombre es aleatorio de 128 bits.
- **Ronda 3:** las dos aprobadas. MENOR-6 de rust (mutantes vivos y limites) reconocido por `verificar-menores`. MEN-7 de calidad (sin test para `por_nombre(orden)`) esta corregido con el test `script_raro_nombrado_solo_en_la_orden_entra_en_el_cierre` (70f7f92, la mutacion lo pone rojo); `verificar-menores` lo deja en `menores-pendientes.md` solo porque su fila cita `cierre.rs`, que no cambio.

## Divergencias

Con el plan aprobado (`planificacion/plan-final.md`), anotadas al implementar:

1. **Pasos 2 y 3 en un solo commit.** El adaptador `--hook` vive en el mismo
   `verificacion/ejecutar.rs` y P7 compara los dos caminos: se commitearon
   juntos (`74017d5`). El paso 1 (`1245aa5`) y la documentacion van aparte.
2. **El almacen guarda tambien `candidatos` por fichero**, ademas de
   `(ruta, tamano, mtime, sha)`: las rutas que nombra el script, normalizadas
   pero **sin resolver**. Asi un fichero sin cambios no se vuelve a leer para
   seguir el cierre, y un script versionado despues de confiar entra en el
   cierre aunque el que lo nombra no haya cambiado.
3. **Los ficheros de `confianza_incluye` tambien se recorren** en busca de
   rutas (la PoC A1 solo los sumaba): cierre igual o mayor, falla cerrado.
4. **Mayusculas**: en Windows y macOS las rutas del cierre se comparan sin
   distinguir mayusculas (`Scripts\V.bat` ejecuta `scripts/v.bat`). El plan
   lo dejaba como no revisado; el resto de modos de fallo del rol de riesgos
   (enlaces simbolicos, TOCTOU, `mtime` manipulado) sigue sin revisar.
5. **Capa git**: se anade `Subcomando::LsFiles` (lectura) para los ficheros
   versionados, y `config::del_arbol` lee la config del arbol de trabajo
   (D3a) junto a `config::de_base`.
6. **`desinstalar --maquina` no borraba el almacen** (suposicion del plan
   falsa: solo borra `taskcodetwo/cli/` y la raiz si queda vacia). Se anade
   el borrado de `taskcodetwo/confianza/` (`b2f573b`), con su test.
7. **P1 no corre en `completa`**: `cargo xtask verificar` no ejecuta pruebas
   ignoradas y no hay mecanismo de pesadas en el workspace. Se marca
   `#[ignore]` como las demas pruebas pesadas del repo (`tarea.rs`,
   `historico.rs`) y se lanza a mano con
   `cargo test -p tc2 --release --test verificar -- --ignored --nocapture`.
8. **`tc2 confiar` solo admite `verificacion.comando`** (`confianza::CLAVES`):
   es la unica orden que `tc2` ejecuta hoy; `derivados[N]` y el resto se
   anaden cuando haya quien los ejecute.
9. **Codigos**: sin hook, rojo, timeout fuera de `rapida` y sin confianza
   salen con 3 (`rechazada`, tabla de F1-4) y `motivo=rojo|timeout|sin-confianza`;
   con `--hook`, el rojo sale con 2 y todo lo que no se ejecuta, con 0.
10. **macOS**: el almacen va a `$XDG_CACHE_HOME` o `~/.cache`, no a
    `~/Library/Caches` (la ultima de `Entorno::raices_de_cache`).
11. **`hasheados=<bytes>`** en la linea de `confiar` y `verificar`: es lo que
    observa P2 (y P1) para la cache de D2a.
12. **Ronda 1 de revision**: el validador de la config rechaza un glob de
    `confianza_incluye` que no compila (`config/validar.rs`, regla
    `GlobAcotado`); el almacen pasa a la version 2 (los `candidatos` ahora
    siguen comillas y letras no ASCII), asi que un almacen de la 1 se lee
    vacio y todo vuelve a `nunca`; `tc2 confiar` anade `globs-sin-ficheros=`
    y un aviso en stderr con codigo 0; con `--hook`, un error de uso sale con
    0 y aviso, y `stop_hook_active` se mira antes que los argumentos.

## Rondas extra

- Ronda 3: la decide quien orquesta, con la consigna de Carlos de seguir con la opcion recomendada; solo verifica la correccion del CRITICO-1 de la ronda 2 (1c9f9e9: scripts de nombre raro entran en el cierre por su nombre) y el MENOR-5, sin codigo nuevo.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-21T17:23:45Z | plan | automatico | persona | 0.16.1 |
| 2026-11-21T17:30:31Z | approve | automatico | persona | 0.16.1 |
| 2026-11-21T17:31:00Z | start | automatico | persona | 0.16.1 |
| 2026-11-21T18:56:07Z | review | automatico | persona | 0.16.1 |
| 2026-11-21T19:32:11Z | review | automatico | persona | 0.16.1 |
| 2026-11-21T19:56:52Z | review | automatico | persona | 0.16.1 |
| 2026-11-21T20:41:30Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-21T17:30:30Z | diseno | agente:afc3eb702f92474dd | 50 | 61166 | 1005806 | 19257 | 44 | 1086279 | - | - |
| 2026-11-21T18:56:07Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 28 | 13335 | 4627741 | 5593 | 0 | 4646697 | - | - |
| 2026-11-21T18:56:07Z | implementacion | agente:ab340757eb3eaf848 | 178 | 262054 | 16557072 | 82461 | 49 | 16901765 | - | - |

## Verificacion ligera

- 2026-11-21T20:16:49Z: ronda 3; commit revisado 3d047d4017e9230ebb149eb4284a71ff22d5f883; commit verificado a4a13fc0050ae8e83f8af29deadd06878222e197; ficheros: crates/tc2/tests/verificar.rs, docs/ci/verificacion-reutilizable.md; corregidos: MENOR-6; pendientes: MEN-7

## Duracion

- Estimada: 270 min
- Real (de la rama abierta al cierre con merge, sin pausas): 191 min
- Paso de la hora: 191 min reales frente a 270 estimados
- Cierre (finish): 1440 s, de 2026-11-21T20:17:30Z a 2026-11-21T20:41:30Z
