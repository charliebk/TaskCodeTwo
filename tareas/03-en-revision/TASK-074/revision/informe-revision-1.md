# Informe de revision — TASK-074 (ronda 1)

- Commit revisado: b983479fa3ac8a824f267ebabad42e8fc7befb5b
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Pase generico (el diff es un ADR, una PoC en Python y su log; ningun revisor de dominio encaja). Profundidad: a fondo la PoC (`planificar`, `siguiente_paso`, `comprobar_procedencia`, `comprobar_primer_padre`, `Lote.hechos/hacer`, secciones B, C y G), las cifras de prosa, tablas e informe contrastadas con el log, y las decisiones 1, 3, 4, 5, 6 y 7 del ADR contrastadas con `plan-final.md` y con F1-5, F1-6, F1-8 y F1-10; por encima la seccion «Cómo se ve en GitHub y en GitLab» y «Consecuencias». No revise F1-2, F1-3, F1-4, F1-11, F1-13 ni F1-15 mas alla de los puntos que el ADR cita.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | docs/adr/F1-16-lotes-desatendidos.md:454 (tabla-paradas) y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:343 (`siguiente_paso`) | plan |
| IMP-2 | IMPORTANTE | abierto | docs/adr/F1-16-lotes-desatendidos.md:486 y tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:285 (`comprobar_primer_padre`) | plan |
| IMP-3 | IMPORTANTE | abierto | docs/adr/F1-16-lotes-desatendidos.md:412 y :464, tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:378 y :674 | plan |
| IMP-4 | IMPORTANTE | abierto | docs/adr/F1-16-lotes-desatendidos.md:438 y :492, tareas/03-en-revision/TASK-074/poc/poc-f1-16.py:961 | plan |
| IMP-5 | IMPORTANTE | abierto | docs/adr/F1-16-lotes-desatendidos.md:539 y :385 | plan |
| MEN-1 | MENOR | abierto | docs/adr/F1-16-lotes-desatendidos.md:326 | implementacion |
| MEN-2 | MENOR | abierto | docs/adr/F1-16-lotes-desatendidos.md:204 | implementacion |
| MEN-3 | MENOR | abierto | docs/adr/F1-16-lotes-desatendidos.md:537 y :69 | implementacion |
| MEN-4 | MENOR | abierto | docs/adr/F1-16-lotes-desatendidos.md:388 | implementacion |
| MEN-5 | MENOR | abierto | docs/adr/F1-16-lotes-desatendidos.md:255 y :623 | implementacion |
| MEN-6 | MENOR | abierto | docs/adr/F1-16-lotes-desatendidos.md:470 | plan |
| MEN-7 | MENOR | abierto | docs/adr/F1-16-lotes-desatendidos.md:237 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

No hay CRITICO. Lo que SI cuadra, comprobado: (a) la PoC completa reproducida con `python -I -B` termina con `RESULTADO: todo como se espera`, exit 0, y su log es **identico byte a byte** al commiteado salvo el final de linea (el del repo se saca con CRLF; sin `\r` el SHA-256 de los dos es `7529ef8f…2d2920a35`); (b) la PoC no deja cambios en el repo (`git status` limpio) ni carpeta de trabajo; (c) todas las cifras de la prosa, las tablas `tabla-lote` y `tabla-inyecciones`, los 15 recuentos de la lista «frases» y el informe de ejemplo salen del log (recalculadas a mano: suma de tokens de las 15 integradas = 120203077, rondas 3 y 4 = 14007657, cascada 66/17 = 3,88, 7 MENOR = 4 + 3, 12 conflictos, 41 cortes); (d) los datos de rondas y «bloqueantes en la ronda 2 = 0» son correctos: los informes de ToolsaWeb solo tienen IMPORTANTE `abierto` en la ronda 1 (25 filas, 0 CRITICO) y el estado de cada informe es el de ese momento, no el posterior a corregir; (e) lo no verificado (GitHub real, GitLab real, firma, tokens de `claude -p`, filtro de F5-1, plan de ToolsaWeb) esta dicho como tal; (f) las 16 mutaciones del bloque G fallan como se espera.

## Reproduccion

Carpeta de trabajo: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-074-r1` (la PoC en `poc/` y mis copias en `mut/`), borrada al terminar. ToolsaWeb solo lectura (`git clone --no-hardlinks --no-checkout` y `git show`). Mis guiones estan en el scratchpad de la sesion, no en el repo. Todo lo que sigue usa las clases de la propia PoC (`Datos`, `lanzar`, `Lote`) importadas por ruta, sobre el sublote TASK-187 → TASK-205 (→ TASK-210) y el commit fijo `93c9618ac`.

### IMP-1 — Nada re-deriva el carril efectivo del diff del agente durante el lote, y el plan lo pedia

- Donde: `plan-final.md` §«Forma» punto 2 («si el carril efectivo del diff sube a riesgo alto (`.taskcode/`, workflows), tambien [se aparta]»); F1-10 decision 5 («subida de carril» → `detener`, F1-10:499-502 y tabla de paradas fijas) y F1-5 decision 4.2; ADR:454 (`tabla-paradas`: solo `carril-no-admitido`, y solo «no entra» al planificar); `poc-f1-16.py:343-412` (`siguiente_paso` no recibe ninguna ruta ni carril: `h["tareas"][t]` solo trae hash, ramas, commits y worktree); `poc-f1-16.py:209-219` (`carril_minimo` solo se usa en `Datos.tarea`, sobre el diff **ya integrado** de ToolsaWeb).
- Que pasa: en el momento de planificar solo se conoce el tipo declarado; el diff real del agente aparece despues de `trabajo`. El ADR no tiene ningun motivo de parada ni paso que mire, tras `trabajo` o antes de integrar, si el diff ha subido a riesgo alto (workflows, `.taskcode/`, CODEOWNERS), que es exactamente la salida de una inyeccion en una tarea de documentacion (F1-8 A13). En un privado gratuito (ToolsaWeb, OpenDockTrading) `carril` no se impone (F1-5 D2a), y en el lote no hay persona.
- Reproduccion (ejecutada): subclase de `Lote` cuyo `trabajo` de TASK-187 deja ademas `.github/workflows/evil.yml` en el worktree y el CLI lo comita. Resultado: `fin fin integradas ['TASK-187', 'TASK-205'] apartadas {}`; `.github/workflows/evil.yml` esta en el arbol de `lote/*`; `procedencia` `limpia` las dos veces. La PoC no tiene ninguna inyeccion de este tipo (C1–C8 y B1–B5 no tocan rutas de riesgo alto).
- Limite de mi prueba: la PoC reproduce los veredictos del `tarea.md` real, asi que no dice si el `carril` o el revisor lo habrian parado en una plataforma que los imponga; lo que muestra es que **el diseño del lote no lo contempla**.
- Impacto: comportamiento incorrecto en un caso real y el motivador de seguridad del ADR; ademas es un compromiso del plan aprobado por Carlos que el ADR no cumple ni declara como divergencia.
- Correccion: añadir a `tabla-paradas` un motivo (p. ej. `carril-subio`, «aparta») que `siguiente_paso` calcule con las rutas del diff de la tarea contra la base anotada antes de `review` y antes de integrar (la regla de F1-5 4.2: solo `docs/` = errata; `.github/`, `.taskcode/`, CODEOWNERS = riesgo alto), con su caso en la PoC (diff con un workflow → se aparta con sus dependientes), o declarar en «Divergencias» que se difiere y a que tarea.

### IMP-2 — La guarda del primer padre (D3a) no mira el arbol del merge: un merge falsificado con los padres, el trailer y el committer correctos pasa, y es la unica defensa en los privados gratuitos

- Donde: ADR:486-491 (decision 6: «cada commit tiene que ser un merge de la Plataforma, con primer padre el anterior, trailer `Tarea:` ... y segundo padre el SHA revisado»), ADR:385-387 («lo único que queda es la comprobación del primer padre (D3a)»), ADR:630-632 («C5b lo detecta por el SHA revisado, no por la firma»); `poc-f1-16.py:285-308` (`comprobar_primer_padre`: compara numero de padres, primer padre, email del committer, trailer y `padres[1]`; **no compara el arbol**).
- Que pasa: C5b solo prueba un merge falso cuyo segundo padre es OTRO commit (`base187`). Un merge con `padres == [anterior, revisado]`, mismo mensaje y trailers y `committer = noreply@github.com` pero con un arbol que lleva un fichero que nadie reviso cumple todas las comprobaciones. Quien lo puede escribir: cualquiera con `contents:write` sin ruleset (el agente, por F1-6:209-212, o un companero). El ADR reconoce que el committer se falsifica en local y apoya la deteccion en el SHA revisado, que el atacante no tiene por que falsificar.
- Reproduccion (ejecutada): hook en el paso `final` que sustituye el ultimo merge de `lote/*` por otro con los mismos dos padres, el mismo mensaje y trailers, committer de la Plataforma y el arbol del original mas `scripts/instalar-malo.sh`. Resultado: `fin: fin | PR final abierto: True | fichero no revisado dentro de lote/*: True | paradas: []`.
- Impacto: «nunca se integra nada en rojo / nada ajeno en `lote/*`» (decisiones 4 y 6, criterio de Carlos) deja de ser una garantia justo donde el ADR dice que es la unica. El PR final lo veria una persona y su CI, pero el informe diria «solo merges del lote».
- Correccion: la guarda debe comparar el arbol de cada merge con `git merge-tree` del primer y el segundo padre (determinista, no depende de la plataforma) y, donde la plataforma lo ofrezca, exigir `verification.verified` del merge con `web-flow`; añadir a la PoC el caso «mismos padres, otro arbol» (C5c) y a la tabla de inyecciones.

### IMP-3 — El manifiesto fija el blob entero de `tarea.md`, que el propio CLI reescribe en cada transicion: el diseño apartaria todas las tareas

- Donde: ADR:412-415 («id del blob de cada `tarea.md`»), ADR:464 (`manifiesto-cambiado`: «el `tarea.md` de la tarea no es el del manifiesto»), ADR:416-418 (reserva y `asignado` de cada tarea al lote); F1-9:237 (`asignado_a` vive en el frontmatter), F1-9 punto 4 (`## Transiciones` y `## Coste` «la escribe el CLI en cada transición»), F1-2 decision 4 (`start` escribe `asignado` y estado); `poc-f1-16.py:378` (`f["hash"] != m["hashes"][t]` sobre el blob completo) y `poc-f1-16.py:674-686` (`crear_estado` escribe cada `tarea.md` una sola vez y nunca vuelve a tocarlo).
- Que pasa: en la 2.0 `tarea.md` de la rama de estado cambia con `start` (estado, `asignado_a`, fila de transicion), con `review` (transicion, `ultimo_commit_revisado`, `## Coste`) y al cerrar (`## Resultado`). El ADR no dice que parte se fija; tal como esta escrito, el primer `commit-estado` legitimo del CLI sobre una tarea del lote cambia su blob y `siguiente_paso` la aparta con `manifiesto-cambiado` (y a sus dependientes). La PoC no lo ve porque ningun escenario modela las escrituras del propio CLI en `tarea.md`; solo C6 escribe, y es el ataque.
- Reproduccion (ejecutada): control sin cambios, `integradas ['TASK-187', 'TASK-205'] apartadas {}`. Con un hook antes de `trabajo` que hace lo que haria `tc2 start` (`estado: en-curso` y una fila en `## Transiciones` en `tareas/TASK-187/tarea.md`, via el `escribir_estado` de la PoC): `fin fin integradas [] apartadas {'TASK-187': 'manifiesto-cambiado'}`; TASK-205 se aparta con ella.
- Impacto: la proteccion de C6/C6b (Carlos escribiendo en `tarea.md`, caso rector) no es implementable como esta escrita; o el CLI no escribe estado durante el lote (y entonces `start`, `review` y el coste de F1-2/F1-9 no valen para el lote) o el hash tiene que ser de una parte. F4-7 lo descubriria al implementar.
- Correccion: definir que se fija: el hash del contenido que el agente no debe cambiar (`## Objetivo`, `## Criterios de aceptacion`, `tipo`, `dependencias`, `etiquetas`, y el plan aprobado, ya atado por F1-5 al blob firmado), calculado sobre esas secciones y claves y no sobre el blob; que `asignado_a`, `estado`, `## Transiciones`, `## Coste` y `ultimo_commit_revisado` queden fuera y los escriba solo el CLI con su CAS; y añadir a la PoC la escritura legitima del CLI como paso del bucle (para que C6 distinga la escritura del CLI de la del atacante).

### IMP-4 — Lo que una persona o un companero escribe en el worktree MIENTRAS el agente trabaja no se detecta: el CLI lo comita como trabajo del agente

- Donde: ADR:438-440 (`trabajo`: «el agente se lanza sin stdin sobre el worktree ... y el CLI hace el commit ... y lo empuja»), ADR:475-481 (guarda: «el worktree está limpio (`status --porcelain`) y en ese SHA»), ADR:492-497 (decision 7: «se detecta el resto antes de `review`»), ADR:255-264 (C1–C3 «el caso de Carlos»: la inyeccion se hace «entre dos pasos del CLI»); `poc-f1-16.py:961-984` (en `trabajo` el «agente» es un `read-tree -u` de un arbol y el `commit` solo comita ese indice; no hay agente corriendo ni ventana de escritura).
- Que pasa: el caso que motiva el ADR es escribir mientras la tarea trabaja. C0 cierra el prompt (sin stdin), pero la persona o un companero tambien pueden escribir **ficheros** en `.taskcode/worktrees/TASK-NNN` durante la ventana en que corre el agente. Cuando acaba, el CLI comita lo que haya en el worktree; despues la guarda ve el worktree limpio en el SHA anotado por el CLI y declara `limpia`. D4c (bloquear el sistema de ficheros) se descarta con razon, pero el ADR no dice que esta ventana queda sin detectar ni la pone en «No verificado»; C1–C3 prueban la ventana de despues, que la PoC inyecta con un gancho entre pasos.
- Reproduccion (ejecutada, con la salvedad de abajo): subclase de `Lote` cuyo `trabajo` de TASK-187 (ronda 2) hace `read-tree -u` del arbol del agente, escribe `notas-de-carlos.txt`, `git add -A` y comita (como dice la decision 3). Resultado: `integradas ['TASK-187', 'TASK-205'] apartadas {}`, `procedencia` `limpia`, y `notas-de-carlos.txt` esta en `lote/*`. (Con ronda 1 el fichero desaparece porque la PoC reconstruye el arbol completo en la ronda 2: es un artefacto de la PoC, no una defensa.)
- Salvedad: en el flujo real el revisor veria el fichero extra en el diff; la PoC reproduce veredictos grabados y no lo ve. El problema es que la guarda declarada no atribuye ni detecta el cambio, no que llegue sin revisar.
- Impacto: el ADR vende D4a como cobertura del caso de Carlos y la cobertura tiene un hueco justo en la ventana larga (el agente tarda minutos). Es comportamiento incorrecto en un caso real, no de borde.
- Correccion: o bien (a) declararlo en «No verificado»/decision 7 como limite aceptado y medir en la PoC la ventana, o (b) detectarlo: foto del worktree (`git status`/arbol) antes de lanzar y comprobar tras el agente que lo nuevo lo produjo el proceso lanzado (p. ej. worktree en un directorio al que solo escribe el proceso hijo, o diff de mtime/propietario con el que corrio el agente). Lo que no vale es que el ADR diga «se detecta» y el diseño no pueda.

### IMP-5 — «Ninguna IA mergea en `develop` ni en `main`» no esta garantizado en los repos del ADR, y el ADR no lo dice

- Donde: ADR:539-546 (Divergencia 1: «La puerta humana sigue siendo una, el PR del lote»), ADR:385-387 y :402-405 (solo avisa de «puertas no impuestas ... en `lote/*`»); F1-6:209-212 (la App del agente alcanza `PUT /pulls/{n}/merge`, mover y borrar refs) y F1-6:460 (clave privada de la App «en la máquina del agente»); F1-5 D2a (privados gratuitos: rulesets no impuestos, 403); F1-8 «Preguntas abiertas» («el diseño (F1-16) tiene que mantener que el lote no integra en `develop` sin el PR final»).
- Que pasa: D2a del ADR solo cierra el token de la **persona** (A5). La App del agente, con `contents:write`, y su clave en la misma maquina y el mismo usuario que el agente, puede mergear un PR contra `develop` en ToolsaWeb y OpenDockTrading, que son los repos para los que se escribio el lote y donde el ruleset no se impone. D3a solo mira `lote/*`; B5 solo mira que `develop` avance (conflictos), no quien lo movio. «Una sola revision humana al final» es una garantia del CLI cooperante, no de la plataforma, en esos repos.
- Reproduccion: **no verificada** (GitHub solo lectura; no se puede ejecutar un merge). Sale de la lectura de F1-5 D2a, F1-6:209-212 y :460 y de las dos decisiones del ADR citadas.
- Impacto: es el invariante de seguridad que Carlos pidio («seguridad ... imprescindibles») y que A16 de F1-8 dejo como tarea de F1-16; el ADR lo da por mantenido.
- Correccion: declararlo en Divergencia 1 y en «No verificado» (la App del agente, con su clave a su alcance, no esta contenida en privados gratuitos), añadir una comprobacion barata y determinista al final (`develop` no se movio por identidades de agente/App entre la base y el PR final: autores/committers de `base..origin/develop` primer padre) y que `doctor`/`tc2 lote` avisen de «`develop` no protegido» y no solo de `lote/*`.

### MEN-1 — «(13 filas más)» del informe de ejemplo son 12

ADR:326. El informe recortado enseña 3 de las 15 terminadas (TASK-173, TASK-174 y TASK-210); las omitidas son TASK-175 a TASK-182, TASK-187 a TASK-189 y TASK-205: 12. El bloque va en un fence y la PoC no lo compara con el log (G lo quita de la prosa). Mutado a «99 filas más», G no falla. Correccion: «12 filas más» y, si se quiere blindar, que G compare las filas del bloque con el informe impreso.

### MEN-2 — Los porcentajes de la cascada excluyen la tarea en rojo y el lote perdido es mayor

ADR:204-208: «apartaría con ella a 3,88 tareas de media, el 22,8 % del lote de media; el máximo es el 82,4 % con TASK-174». Son dependientes: 66/17 = 3,88 y 14/17 = 82,4 %. Lo que se pierde incluye la propia tarea: 15 de 17 = 88,2 % (el log lo imprime: «apartadas 15 (88,2 %; en cascada 14, 82,4 %)») y de media 28,7 %. La frase siguiente («se apartan 15») convive con el 82,4 %. Correccion: decir «dependientes» junto a cada porcentaje o dar los dos.

### MEN-3 — Dos paradas fijas de F1-10 decision 6 se cambian en el lote sin declarar divergencia

ADR:537 (Divergencias, que no trata F1-10) y :69 («reutiliza `politica`/`accion` (F1-10)»). F1-10 decision 6 fija «en cualquier configuración y modo» `merge-pr` → `detener` («ninguna IA mergea») y tope de rondas → `preguntar` (`cerrar`, `ronda-extra` o `volver-a-diseno`), y su PoC C comprueba 1260 combinaciones con 0 `continuar`. El lote mergea por la App (`integrar-en-lote`, decision 5) y convierte el tope en «apartar o integrar» (decision 10). Es intencionado y aprobado (D6a), pero el ADR dice que reutiliza `accion` y no que sobreescribe dos de sus filas; F4-7 recibiria dos contratos contrarios. Correccion: una divergencia con F1-10 (paradas `merge-pr` y tope de rondas, y subida de carril si se atiende IMP-1) y decir que `tc2 lote` no consulta `accion` para ellas.

### MEN-4 — GitLab: el hueco que se declara es el de las aprobaciones, pero «nunca rojo» depende del origen del check (G09), que F1-13 da «en ningún tier»

ADR:388-392 y la fila «Nunca integrar en rojo» de «Requisitos y evidencia». F1-13 (tabla de garantias, G09) dice que en GitLab «Pipelines must succeed» no distingue el origen y que un Developer puede crear estados de commit. El ADR cubre GitLab con «la misma comprobación del CLI» (primer padre), que no mira si `verificacion` salio verde (IMP-2 tambien aplica). No esta verificado (GitLab no se probo) pero ni se nombra G09. Correccion: nombrar G09 en el parrafo de GitLab y en «No verificado».

### MEN-5 — C0 prueba que `sys.stdin` de un Python esta vacio, no que el agente real no admita entrada

ADR:255-257 («Es lo que impide «escribir en el prompt»») y :623 (`No verificado`, que no lo lista). `poc-f1-16.py:1573`: el «agente» es `python -c "sys.stdin.read()"` con `stdin=DEVNULL`. No se comprobo que `claude -p` no lea de la consola (en Windows un hijo puede abrir `CONIN$` aunque se redirija stdin) ni que no haya otro canal de entrada (IDE, control remoto). Correccion: en «No verificado»: «que el agente lanzado con `claude -p` no acepte entrada por otro canal que stdin».

### MEN-6 — No hay salida documentada de `lote-con-commits-ajenos`: `--reanudar` se queda parado para siempre

ADR:470 (`para`) y decision 8. Ejecutado: C4 (commit de un companero en `lote/*`), y despues dos `--reanudar` seguidos: las tres veces `lote-con-commits-ajenos`, `integradas ['TASK-187']`, tres eventos `parada`. El primer padre sigue teniendo el commit ajeno y `siguiente_paso` para siempre; con el ruleset impuesto no se puede forzar `lote/*`. Para `presupuesto` el ADR da un evento de la persona; para esto, nada (¿revertir el commit ajeno?, ¿aceptarlo con un evento firmado?, ¿rehacer el lote?). Correccion: un procedimiento para la mañana (p. ej. evento `aceptado-ajeno` de la persona con el SHA, o revert) y su caso en la PoC.

### MEN-7 — «Ninguna merge real posterior a la serie entra sobre la base del lote» no sale del log y parece falsa

ADR:237-239 (B5). La PoC elige el primer commit posterior que entra limpio y no toca ficheros del lote y no imprime cuantos otros hay (`poc-f1-16.py:1529-1541` hace `break` en el primero). Aproximado con la misma regla (sin parar y usando el arbol real de `HASTA` como conjunto de ficheros del lote, que es una cota mas dura): de 40 merges y 485 commits posteriores, 3 commits y **1 merge** (`3587ef913`, `feature/task-278`, solo `docs/topo/particion-h15.md` fuera de `tareas/`) entran limpios sin tocar nada del lote. Es una aproximacion mia y no cambia la conclusion (se usan dos avances reales), pero la frase es una afirmacion de evidencia que el log no sostiene. Correccion: quitar «ninguna merge real ...» o que la PoC imprima el recuento.

## Mutaciones propias (ninguna esta entre las 16 de la PoC)

Hechas importando la PoC por ruta; ADR y PoC del repo intactos.

| Mutacion | La PoC la detecta | Nota |
|---|---|---|
| Escritura legitima del CLI en `tarea.md` (estado y transicion) antes de `trabajo` | **No** (la PoC no la modela) | muestra IMP-3: se apartan las dos tareas por `manifiesto-cambiado` |
| Merge falsificado con los mismos padres y otro arbol (ultimo merge) | **No** | IMP-2 |
| Fichero de riesgo alto (`.github/workflows/evil.yml`) en el trabajo del agente | **No** | IMP-1 |
| Fichero de una persona en el worktree durante `trabajo` | **No** | IMP-4 |
| ADR: `con ronda extra automática` en la decision 10 (invierte D6a) | No | G solo lee tablas, IDs, enlaces, secciones y cifras; la prosa de las decisiones no es dato (el ADR lo dice) |
| ADR: ruleset «con `bypass_actors` de la persona» | No | idem |
| ADR: `merge_method=squash` (rompe la comprobacion de dos padres) | No | idem |
| ADR: integrar «sin `sha`» (quita el CAS) | No | idem |
| ADR: `merge-pr` pasa a la App del agente | No | idem; F1-10 lo cubre en su propia PoC, no esta |
| ADR: «(99 filas más)» en el informe | No | MEN-1 |

Las seis ultimas no son un hallazgo por si solas: la cabecera del ADR y la seccion G declaran que solo comprueban lo que dicen. Se anotan para que quien lea «la PoC lee este fichero» sepa hasta donde.

## Criterios de aceptacion y plan-final

- «Diseño probado a mano con un lote real de ToolsaWeb»: cumplido en lo que declara (17 tareas reales sobre una copia y Plataforma simulada, sin agente ni CI reales). Con las salvedades de IMP-1, IMP-3 y IMP-4: tres cosas que el plan pedia o que el caso de Carlos exige no estan en la PoC.
- «Cumple lo que pide F1-16 y los requisitos que cita»: rama, carriles, que para y que aparta, presupuesto, reanudacion, informe y GitLab estan; GitLab y GitHub reales se declaran no verificados. Faltan IMP-1 (carril subido) y la divergencia con F1-10 (MEN-3).
- Las ocho decisiones D1a–D8a estan recogidas con la misma letra que el plan y D2a, D6a y D7a conservan su condicion.
- La fila del indice (`docs/adr/README.md:26`) existe, resuelve y cuadra con el ADR.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-074/poc/poc-f1-16.py` y `poc-f1-16.log`: es mi parte del diff (excluida del diff embebido por `excluir_de_revision`); sin ellos no se puede reproducir.
- `docs/adr/F1-5-puertas-en-github.md`, `F1-6-identidades-agente-revisor-persona.md`, `F1-8-seguridad-modelo-de-amenazas.md`, `F1-9-formato-de-tarea-plan-y-especificacion.md`, `F1-10-carriles-y-modo-automatico.md`, `F1-13-adaptador-de-plataforma.md` y `F1-2-escrituras-atomicas-ids-y-toma.md` (solo los pasajes que el ADR cita): lo pidio expresamente el encargo («coherencia con los ADR de los que depende»).
- `docs/plan/PLAN-TASKCODE-2.md`, `REQUISITOS-FUNCIONALES.md` y `AMPLIACION-2026-11-14.md` §5 bis: para los criterios F1-16, RR-18 y RF-27.
- `tareas/04-terminadas/TASK-073/revision/informe-revision-1.md`: formato del informe.
- ToolsaWeb (`C:\Users\nullcad2025\Documents\sources\Personal\ToolsaWeb`), solo lectura con `git show`/`ls-tree`/clon local en la carpeta temporal: los `informe-revision-N.md` de las 17 tareas en `93c9618ac` (estados de IMPORTANTE, rondas de TASK-174, 189 y 191) y los commits posteriores a la serie (MEN-7).
