---
id: TASK-046
titulo: "F5-6 Hook local rapido, opcional y con confianza"
tipo: feature
sprint: 5
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
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
- [ ] ≤ 1 s
- [ ] nada del repo sin confianza
- [ ] Cumple lo que pide la tarea F5-6 del plan y los requisitos que cita

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

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-21T17:23:45Z | plan | automatico | persona | 0.16.1 |
| 2026-11-21T17:30:31Z | approve | automatico | persona | 0.16.1 |
| 2026-11-21T17:31:00Z | start | automatico | persona | 0.16.1 |
| 2026-11-21T18:56:07Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-21T17:30:30Z | diseno | agente:afc3eb702f92474dd | 50 | 61166 | 1005806 | 19257 | 44 | 1086279 | - | - |
| 2026-11-21T18:56:07Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 28 | 13335 | 4627741 | 5593 | 0 | 4646697 | - | - |
| 2026-11-21T18:56:07Z | implementacion | agente:ab340757eb3eaf848 | 178 | 262054 | 16557072 | 82461 | 49 | 16901765 | - | - |
