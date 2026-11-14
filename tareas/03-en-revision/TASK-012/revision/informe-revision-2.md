# Informe de revision — TASK-012 (ronda 2)

- Commit revisado: 054468e9c3f2315bb351d2ecb0e970649becb914
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Veredicto propuesto por el revisor: **cambios-solicitados**. Los cuatro IMPORTANTE de la ronda 1 quedan corregidos y verificados, pero la correccion introduce un IMPORTANTE nuevo (IMPORTANTE-5: el check `estado` ejecuta el comprobador de la rama del PR). Nota sobre el commit: HEAD es `5a36de1` (solo anade la peticion y el informe); `git diff 054468e HEAD -- docs` no muestra cambios, asi que el ADR revisado es el de `054468e`.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:590 | implementacion |
| IMPORTANTE-2 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:531 | implementacion |
| IMPORTANTE-3 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:600 | implementacion |
| IMPORTANTE-4 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:629 | implementacion |
| IMPORTANTE-5 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:591 | plan |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Los MENOR de la ronda 1 no se revisan en esta ronda (los comprueba quien coordina).

## Verificacion por hallazgo

### IMPORTANTE-1 (ronda 1) — corregido

Se comprueba que el texto del ADR ya define el check `estado` y que el hito se genera en la rama de estado:

- `hitos.md` en la rama de estado: el ejemplo de ToolsaWeb pasa a `derivados` `genera` con `escribe: ['tareas/hitos.md']` (ADR:556) y el `comprueba` ya no bloquea la transicion. PoC C2: `--comprobar` de ToolsaWeb con `hitos.md` en `docs/` da codigo 1 tras la transicion (hoy); con la disposicion nueva, 0 errores y `docs/` sin tocar.
- Comprobador de ODT: `comprobar-fallos.py` entra en `derivados` (PoC B: `derivados[1].comando`). Ya no queda dentro del perfil cacheado.
- Check `estado` (ADR:590-598): definido, obligatorio en cada PR, no cacheado («su resultado depende del estado, que no esta en el arbol»), bloquea el merge, y sustituye la puerta del paso 1/22. La divergencia con F1-5 esta declarada en «Divergencias con F1-5 y F1-7», punto 1. PoC C, T6: `comprueba` en rojo avisa en la transicion y el check `estado` queda en rojo (`ROJO (bloquea)`) y verde sin la cita rota.
- Pero el ADR dice que el check ejecuta los `comprueba` de la base contra el arbol del PR, y eso abre IMPORTANTE-5.

### IMPORTANTE-2 (ronda 1) — corregido

- El validador normaliza antes de comparar (ADR:531; PoC B). Reproduccion propia con `normalizar_escribe` extraida del guion (copia en la carpeta temporal):
  - Rechazadas: `TAREAS\..\docs\x` (backslash), `tareas/../docs/x`, `tareas/sub/../../docs/x` (fuera de `tareas/`), `tareas/x/..//ROADMAP.md` y `TAREAS/TASK-001/tarea.md` (reservadas, sin distinguir mayusculas), `tareas/ROADMAP.MD`, `tareas/ROADMAP.md.`, `tareas/ROADMAP.md `, `tareas/TASK-001 /tarea.md`, `tareas/TASK-001./tarea.md` (puntos o espacios finales), `tareas/ROADMAP.md::$DATA` (ADS), `tareas/CON.md` (reservado de Windows).
  - Aceptadas: `tareas/TASK-001/../x.md` (resuelve a `tareas/x.md`, correcto), `tareas/hitos.md`, `tareas/informes/grafo.md`.
  - Aceptada y no verificada: `tareas/enlace/planificacion/hitos.md`. El validador es solo de cadenas; un enlace simbolico real no se ha probado (en este Windows `mklink` no se ha ejecutado). La defensa en ejecucion depende de la comparacion por contenido, que la PoC prueba para ficheros, no para enlaces.
- Ejecucion (ADR:579-588; PoC C): T8 (derivado que reescribe el `tarea.md` de la transicion) y T9/T10 (rutas ignoradas, en codigo y en la rama de estado) quedan descartados con aviso. T8 deja el estado en `en-diseno`. La comparacion es por contenido, no por lineas de `git status`.

### IMPORTANTE-3 (ronda 1) — corregido (proyectado)

- `tareas --json` lleva frontmatter, `criterios` y `texto` (ADR:600-603). Lo comprueba PoC D con los tres consumidores copiados (`comprobar-modulos.py`, `comprobar-fallos.py` y `grafo-tareas.py`): el mismo resultado que hoy (ids, valores, texto, criterios, errores, Mermaid y roadmap), frente a 0 tareas con la carpeta fija.
- La cifra `114 -> 11` y `3/3` esta en la tabla como **proyectado**, y el ADR la trata como contrato de F2/F3 (Consecuencias). Con la interfaz simulada por la propia PoC, no medida contra la CLI real: queda como «No verificado» y eso es correcto.

### IMPORTANTE-4 (ronda 1) — corregido

- La ventaja tiene una columna «Tipo» (medido o proyectado) y el guion la comprueba (PoC: `tabla ventaja` 22/22 mas las cifras del resumen).
- Las dos medidas de claves estan: ToolsaWeb 10 -> 2 bloques y 10 -> 16 valores; ODT 2 -> 2 bloques y 2 -> 13 valores. El tope de 12 se lee como primer nivel (5 de 12, 15 hojas).
- El resumen dice donde no gana: la config no es mas corta (10 -> 16, 2 -> 13), la cache por paso se pierde, ToolsaWeb necesita huella local y `--perfil`, y la lectura del estado (114 -> 11) es proyectada. La afirmacion «de 8 a mano se pasa a 0» de la ronda 1 ya no aparece; ahora dice «3 comprobados», que es lo que la PoC respalda.

### IMPORTANTE-5 (nuevo, introducido por la correccion) — abierto

- Donde: docs/adr/F1-11-extensiones-declaradas.md:590-591. El ADR dice: «el check `estado` ... ejecuta los `comprueba` de la configuracion **de la base** contra el arbol del PR y la rama `taskcode/tareas`» y que «bloquea el merge». El comando sale de la base, pero el script que ejecuta (p. ej. `scripts/grafo-tareas.py`) se lee del arbol del PR. Ademas, en CI «no usa la confianza local» (ADR:594-596), asi que nada fija el comprobador.
- Reproduccion (carpeta temporal, `python -I -B`): mismo comando `python -I scripts/comprobar.py --comprobar`. Con el comprobador de la base, que falla por `TASK-404` citada y sin tarea: codigo 1. Con el comprobador del PR, que siempre sale con 0: codigo 0. El PR puede dejar verde el check `estado` cambiando el comprobador en el mismo PR.
- Impacto: el check que el ADR presenta como puerta obligatoria que bloquea (el «guard que deberia bloquear») puede dejarse pasar desde el propio PR. Cambiar el comprobador es visible en el diff, pero ningun texto del ADR lo impide ni lo exige. Es real, no de borde, y viene de la correccion de IMPORTANTE-1.
- Sugerencia: el check `estado` tiene que tomar el comprobador (script y argumentos) de la base (p. ej. un worktree de la base solo para `scripts/` del comprobador, o fijar su hash en la base y fallar si cambia); o, si se acepta que el PR lo cambie, que el cambio del comprobador haga rojo el check o lo exija CODEOWNERS. Anadir a la PoC C un caso «el PR cambia el comprobador: el check sigue en rojo».
- No verificado: la implementacion del job en GitHub (no hay workflow ni PR real, como declara el ADR).

## Clasificacion parche / propio (pedida en la peticion)

Dos citas del desglose de `scripts/verificar.bat` de ToolsaWeb en `cb46c0f1b` (`git show`, solo lectura):

- `verificar.bat:91-93` (completa-leer de la cache): en el rango de parche (80-107). Es la cache por arbol que se retira: **razonable**.
- `verificar.bat:124-127` (paso 1/22: `grafo-tareas.py --probar` y `--comprobar` envueltos en `cache-verificacion.py correr`): fuera de los rangos de parche, es propio. El envoltorio `cache-verificacion.py correr` esta en el parche y el desglose lo quita («se quedan como `python -I <paso>`»): **razonable**. Son las comprobaciones que cruzan codigo y estado del proyecto (practica 3, propia).
- Extra, `verificar.bat:254` (resumen-verificacion `--probar`): en el parche, correcto (x/N de `TASKCTL_PROGRESO`, practica 8).

Matiz (sin hallazgo): la practica 9 (`RAMA_BASE`) figura como **propio**, pero sus citas estan en `clasificar-verificacion.py:14-15, 54-59`, y ese fichero entero figura como **parche** (1000 de 1000 lineas). El ADR lo explica en el desglose (el clasificador cuenta como parche por las rondas de review). Es coherente solo si la variable se conserva en `rapida` e `impacto` y el fichero se retira: conviene decirlo en la fila 9.

## Verificacion realizada

- Puerta: `bash .github/scripts/comprobar-repo.sh` -> `Tareas: 93; enlaces relativos: 92; fallos: 0`, salida 0.
- PoC reproducida: `python -I -B tareas/03-en-revision/TASK-012/poc/poc-f1-11.py <trabajo> docs/adr/F1-11-extensiones-declaradas.md .. ~/.claude/plugins/cache/taskcode-marketplace/taskcode-plugin/0.16.1`, con la carpeta de trabajo en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-012-r2`. Salida 0 en 1 min 26 s, sin cambios en el repo. Resultados: A 14 practicas, 48/48 citas; A2 parches=7; B traducidas 2/2, hostiles 22/22, claves 5/12; C 10/10 transiciones, `develop` identico, check `estado` rojo y verde; C2 hoy 0 -> 1, nueva 0 -> 0; D consumidores 3/3; E grupos iguales; tabla de ventaja OK, resumen OK. El log es el de la ronda 1 con las cifras nuevas, y no se ha comparado linea a linea con el versionado.
- Pruebas propias: normalizacion de `escribe` (listadas en IMPORTANTE-2) y la reproduccion de IMPORTANTE-5. Ambas en la carpeta temporal.
- ToolsaWeb y OpenDockTrading: solo lectura (`git show`). ODT no se ha leido en esta ronda. Nada se ejecuto dentro de los repos.
- GitHub: no se ha llamado.
- Mutaciones: no hechas en esta ronda (verificacion acotada).
- Carpeta temporal `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-012-r2`: borrada al terminar.
- No se ha hecho ningun commit. Unico fichero modificado: este informe.

## Lecturas fuera del contexto permitido

- tareas/03-en-revision/TASK-012/revision/informe-revision-1.md: pedido por la peticion (IMP-1..IMP-4 de la ronda 1).
- tareas/03-en-revision/TASK-012/poc/poc-f1-11.py: pedido por la peticion para reproducir; leido en las partes del validador, de la comparacion y de la seccion D.
- Ficheros de la PoC generados en la carpeta temporal (poc.log): salida de la reproduccion.
- ToolsaWeb `scripts/verificar.bat` en `cb46c0f1b` (`git show`): pedido por la peticion (dos citas del desglose).
- `revision.md` de la skill (referenciado por la skill): no se ha leido; se han seguido la peticion, el informe de la ronda 1 y la plantilla de la tarea.
