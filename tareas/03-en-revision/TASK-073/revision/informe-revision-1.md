# Informe de revision — TASK-073 (ronda 1)

- Commit revisado: 2c541f98e2eb395180e4b1d28834ea517dcdcc67
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

Pase generico (el diff es un ADR, una PoC en Python, una tabla de clasificacion y un log; ningun revisor de dominio encaja). Profundidad: a fondo la PoC (`seleccionar`, `dominios`, secciones S y G), la clasificacion de los 33 IMPORTANTES contrastada con los informes, las cifras de R y D y el muestreo; por encima la prosa de «Cómo se ve» y «Consecuencias».

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:61 | plan |
| IMP-2 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-073/poc/poc-f1-15.py:1174 | implementacion |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:341 | plan |
| MEN-1 | MENOR | corregido | docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:185 | implementacion |
| MEN-2 | MENOR | corregido | docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:155 | implementacion |
| MEN-3 | MENOR | corregido | docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:414 | plan |
| MEN-4 | MENOR | corregido | docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:416 | implementacion |
| MEN-5 | MENOR | corregido | docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:470 | implementacion |
| MEN-6 | MENOR | corregido | tareas/03-en-revision/TASK-073/poc/clasificacion-hallazgos.md:55 | implementacion |
| MEN-7 | MENOR | corregido | docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:84 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

No hay CRITICO. La falta de la marca de Carlos en la columna «comprobado a mano» NO es hallazgo: el ADR la declara «pendiente de Carlos» en la cabecera (linea 8), en la tabla T (10 de 10 filas), en «No verificado» y en la regla de la PoC G (que falla si alguien escribe «sí» sin persona y fecha). Comprobado: en ningun sitio la da por hecha.

## Reproduccion

### IMP-1 — El motivo con que D1b se descarta no lo sostienen los datos del propio ADR

- Donde: ADR:61 («Descartada: «seguridad» ya suma 2 puntos y F1-13 y F1-18 salieron «simple»; los puntos no distinguen dominio»); el mismo texto viene del plan aprobado.
- Que pasa: `palabras_alto_riesgo` de la 0.16 (`heuristica-complejidad.yml:159`) contiene «seguridad», pero **ni TASK-015 (F1-13) ni TASK-094 (F1-18) contienen «seguridad» ni ninguna otra palabra de esa lista**: `grep -i -o 'segur…|autenticaci…|autorizaci…|cifrad…|rendimiento|migraci…'` sobre sus `tarea.md` (04-terminadas) no devuelve nada. Su puntuacion (1 y 2, log R, columna `pts`) sale solo de criterios y dependencias; «seguridad» no les sumo ningun punto. De las 15 tareas, la unica con una palabra de riesgo es F1-8 (4 puntos, «media»). El ejemplo que justifica descartar D1b dice lo contrario de lo medido: no prueba que «los puntos no distinguen dominio», prueba que la lista de palabras de la 0.16 no tiene las palabras de puertas (puerta, check, ruleset, runner…), que es otro argumento (y mas fuerte a favor de D1a).
- Reproduccion: lectura de `heuristica-complejidad.yml` y del grep anterior en `tareas/04-terminadas/TASK-015/tarea.md` y `TASK-094/tarea.md`; el log R confirma 1 y 2 puntos sin palabra de riesgo.
- Impacto: es una afirmacion de evidencia que la evidencia no sostiene (clase «fidelidad-evidencia» de la propia PoC R, precedente TASK-005 IMP-3 y TASK-007 IMP-3: IMPORTANTE). No cambia la decision (D1a la impuso Carlos), pero el ADR es el contrato que F4-6 leera.
- Correccion: reescribir el motivo con lo medido (p. ej. «la lista de riesgo de la 0.16 no contiene las palabras de puertas: 14 de 15 tareas salieron simple», y, si se quiere el contrafactual de sumar puntos por palabra del detector, calcularlo en la PoC D en vez de afirmarlo; o quitar el ejemplo de F1-13 y F1-18).

### IMP-2 — Nada prueba que `seleccionar` haga lo que dice `tabla-seleccion`, y el log afirma que S13 se ejercita

- Donde: ADR:11 («lee tres tablas de este fichero (`tabla-seleccion`, …)»), `poc-f1-15.py:1174` y `poc-f1-15.log:137` («las S13-S15 se ejercitan arriba»), `poc-f1-15.py:322` (S13), `poc-f1-15.py:312` (S10).
- Que pasa: dos huecos de la misma raiz.
  1. **La PoC no lee `tabla-seleccion`**: `leer_tablas` la parsea a `{id: {rol, ids}}` (poc-f1-15.py:597-610) y `seleccionar` solo recibe `tablas["dominios"]` y `tablas["modelo"]`; la logica de S01..S13 (suelo 2, umbral, S13, S10…) esta escrita a mano en el codigo y el texto de «Cuándo» y «Qué da» no se compara con nada. Solo se comprueba que los ids de la ultima columna estan en el inventario. El ADR dice que la funcion «lee tres tablas», y la PoC G que el ADR «es dato»: para esta tabla no lo es.
  2. **Cinco filas sin ningun caso**: `seleccionar` nunca se llama con `ronda >= 2` (grep `ronda=` en el guion: solo la firma), asi que **S13 no se ejercita en ningun sitio** (el log dice «las S13-S15 se ejercitan arriba»: S14 y S15 si, S13 no). S01 (errata), S04 (rendimiento), S06 y **S10** (revisora del dominio instalada y declarada, el nucleo de D6a) tampoco salen en las 10 tareas ni en ningun control: el propio log las lista como «no salen».
- Reproduccion (copias del repo en `%LOCALAPPDATA%/pruebas-claude/taskcodetwo/task-073-r1`, borradas; PoC completa con `python -I -B`, 150 s cada una):
  - M-A (control, se detecta): borrar la fila D05 `check` de `tabla-dominios` → `RESULTADO: 2 fallos` (`tabla-muestra OpenDockTrading/TASK-014` y `TASK-096` distintas de lo calculado).
  - M-B (**no se detecta**): en la copia del ADR cambiar S13 a «siempre» (sin la salvedad de riesgo-alto y puertas-seguridad), S03 de «suelo de 2 roles» a «suelo de 3 roles» y S10 a «nunca / nada» → `RESULTADO: todo como se espera`, exit 0, sin un solo fallo.
  - M-C (**no se detecta**): en la copia de la PoC, `if ronda >= 2 and carril != "riesgo-alto"…` → `if False and …` (S13 desactivada) → `RESULTADO: todo como se espera`.
  - Control manual de que S13 existe y funciona: `seleccionar("revision", …, ronda=2)` con puertas-seguridad da `opus` y con el dominio ausente `sonnet`, regla S13 (ver IMP-3); simplemente nadie lo llama en la PoC.
- Impacto: «Una proteccion nueva sin ningun test que se ponga rojo al quitarla» (S13: ahi se concentran 5 de los 7 IMPORTANTES de rondas de verificacion, es la regla con mas datos detras) y la tabla que F4-6 implementara puede divergir del texto sin que lo note la PoC. Mismo patron que TASK-007 IMP-3 («la PoC no lee la tabla del ADR»).
- Correccion: (a) leer `tabla-seleccion` de verdad (p. ej. columna `Cuándo` como predicado en lista cerrada) o decir en el ADR y en la cabecera de la PoC que S01..S15 son prosa verificada solo en sus ids, y que `tabla-dominios` y `tabla-modelo` son las unicas tablas-dato; (b) anadir controles: S13 con ronda 2 en riesgo-alto, en `puertas-seguridad` y en el resto; S10 con una revisora sintetica que declare `puertas-seguridad`; S04 con la etiqueta `wasm`; S01 con errata; S06 con una skill `rol: diseno`; (c) corregir la frase del log (`poc-f1-15.py:1174`) para que no atribuya S13 a «arriba».

### IMP-3 — El dominio (suelo de roles, S10/S11 y S13) se decide con texto de `tarea.md`, que el agente escribe y el ADR no trata como no confiable

- Donde: ADR:341-345 (decision 3: señales de palabra sobre «el objetivo o los criterios» y de etiqueta), ADR:380-388 (decision 4, suelo), tabla S03/S10/S11/S13, `poc-f1-15.py:196-218` (`dominios`), `poc-f1-15.py:322` (S13 usa `nd`).
- Que pasa: en la motivacion del ADR (ADR de solo `docs/`) **todas** las señales que disparan `puertas-seguridad` son palabras del objetivo y de los criterios (log D: `rutas=['docs']`, senales `palabra:*` en las 9 tareas detectadas). Ese texto vive en `tarea.md`, que el agente puede reescribir; F1-8 (A10) y F1-10 (IMP-1 de TASK-011: «`tipo` (texto de `tarea.md`, no confiable) elige la fila y puede bajar la puerta») ya fijaron que lo declarado en la tarea «solo sirve para subir». Aqui el texto no sube nada: **quita**. Con el mismo trabajo y otro vocabulario desaparece el suelo, el aviso `dominio-sin-revisora` y la proteccion de S13. El aprobado de F1-5 se ata al blob de `plan-final.md`, no al de `tarea.md`, y la revision se recalcula cada ronda con el `tarea.md` de ese momento. El ADR no lo enumera ni en «No verificado» ni en «Fuera de alcance» (el plan lo dejaba en «Riesgos sin enumerar: etiquetas puestas para forzar o evitar el suelo»).
- Reproduccion (`importlib` sobre la PoC y las tablas del ADR, catalogo minimo; sin tocar el repo):
  - texto con «check obligatorio», «token», «aprobacion», «puerta» → diseño `['M04','S02','S03','S05']` con 3 agentes; revision ronda 1 con aviso `dominio-sin-revisora:puertas-seguridad`; ronda 2 `opus` (`S11`).
  - mismo trabajo reescrito («validacion obligatoria», «credencial de conformidad», «filtro») → diseño `['M04','S02']` con 1 agente; revision sin aviso; ronda 2 **`sonnet`** (`S13` se aplica porque `nd` esta vacio).
  - Los avisos medidos por F8-6 («escapados») tambien desaparecen: la metrica no ve lo que el texto oculta.
- Impacto: el agente que implementa (y reescribe `tarea.md`) elige cuantos roles de diseño lleva, si hay aviso y con que modelo se verifica: contradice «el agente no elige quien le revisa» y la regla de F1-10 (solo sube). Es comportamiento incorrecto en un caso real (el motivador del ADR), no de borde.
- Correccion: (a) que las señales de texto solo **añadan** sobre un minimo derivado de lo no escribible por el agente (rutas del diff, carril efectivo de F1-10, etiquetas fijadas al aprobar) y que el texto de `tarea.md` usado sea el del commit firmado de `approve` (hash anotado en `roles`); (b) para la revision, que `dominios()` use solo rutas del diff y el texto del commit aprobado, no el `tarea.md` vivo; (c) anadir el caso a la PoC S (mismo trabajo, otro texto, la seleccion no baja) y a «No verificado».

### MEN-1 — «Regla fijada antes de los resultados» no es comprobable con lo entregado

- Donde: ADR:185 (titulo de la seccion), ADR:518-519 (divergencia 8: «decidido antes de ejecutar T»).
- ADR, PoC, log y tabla llegan en el mismo commit (`90330c6`); el plan D9 fija solo los estratos por nombre, no el desempate «ID más alto» ni el estrato `un-dominio`. La regla, aplicada de nuevo por mi cuenta sobre ToolsaWeb (`93c9618ac156`) y OpenDockTrading (`fab8d7e05eee`), da las mismas 10 tareas (p. ej. ToolsaWeb: 306 es la de ID mas alto con `skills_recomendados`, 399 la mas alta de tipo `fix`, 299 la mas alta solo con `.md` bajo `docs/`; OpenDockTrading: 063 es la mas alta con palabras de seguridad, 062 la siguiente), asi que la muestra es coherente; lo que no se puede demostrar es el «antes». Correccion: decir «regla escrita en el ADR; el orden de los hechos no lo prueba el historial» o dejar el commit de la regla separado del de los resultados.

### MEN-2 — Recall y precision son dentro de muestra y la frase de las palabras lo oculta

- Donde: ADR:151-156.
- «No se ajustaron a la vista del resultado» es cierto respecto a la PoC, pero la lista de palabras la fijo el plan **despues** de la «lectura preliminar» de esos mismos 15 informes (tabla preliminar de `plan-final.md`), y contiene vocablos que salen de ellos (`ruleset`, `runner`, `amenaza`, `identidad`). Los recuentos son correctos (recalculados: 7 VP, 2 FP, 1 FN, 5 VN → recall 7/8, precision 7/9, 8 tareas pasan de 1 a 2 roles, 1,5 M tokens por tarea que dispara el suelo, 10,7 % de 13,9 M) pero son una medida de entrenamiento, no de validacion. «No verificado» dice «valen para esa muestra»; falta decir que son en-muestra.

### MEN-3 — «La implementacion no lanza subagentes» contradice los datos del repo

- Donde: ADR:413-414.
- `## Coste` de TASK-006, 009, 011, 073 y 094 trae filas `implementacion | agente:…` (la implementacion de esta misma tarea es del agente `af178d5c…`, 12,6 M tokens) y F4-4 encadena la implementacion en modo automatico. Con `agentes: []` siempre en `roles.implementacion`, la fila de modelo de implementacion (M01..M07) no dice a quien se aplica. Definir si F4-4 lanza un implementador (y entonces recibe agente y modelo de `seleccionar`) o si es la sesion del usuario (y entonces el modelo de implementacion es un consejo).

### MEN-4 — Config «de la base» solo se comprueba como texto; el inventario del proyecto no dice de que arbol sale

- Donde: ADR:416-430 (decision 6) y decision 2 (ADR:329-340).
- La PoC G solo exige que aparezca la regla «configuración de la base» (mutacion `sin-config-de-la-base`); `seleccionar` recibe `cat`, `inv` y `cfg` sin distinguir procedencia. La decision 2 lista `.claude/skills` y `.claude/agents` del proyecto como fuente de inventario sin decir «de la base» (la 6 si dice «skills de la base»): una skill `rol: revisor` con `fallback: true` añadida en el PR podria sustituir a la generica (`gen[0]` = primera por id). No lo demuestro (no hay revisor en CI), pero la decision 2 debe decir de que arbol se leen y la PoC un caso (catalogo del PR con un fallback adicional → la seleccion no cambia).

### MEN-5 — Divergencia con F1-14 sin anotar

- Donde: ADR:470-476 (decision 11) y la fila «F1-14 … contrato de `evaluar_recursos` sin cambios».
- F1-14 fija como «obligatorio» las skills de fase del plugin; F1-15 añade «los agentes de brainstorm del plugin» a «obligatorio» y hace que lo elegido por tarea sea «necesario para esa tarea» (nivel por tarea, no por proyecto), con `start` parando. Es una ampliacion del suelo de F1-14 que el ADR no lista en «Divergencias con el plan».

### MEN-6 — Una clasificacion contradice la precedencia que la propia tabla fija

- Donde: `clasificacion-hallazgos.md:55` (TASK-012 IMP-1, «correccion») frente a la regla de `clasificacion-hallazgos.md` (puerta-seguridad gana si trata de una puerta que se baja).
- IMPORTANTE-1 de TASK-012 incluye (punto 3) que el `comprueba` «pasa de bloquear a avisar sin parar la transición» y que no esta definido que ejecuta el check `estado`: un bajado de puerta. Con la precedencia declarada seria puerta-seguridad (17 de 33 en vez de 16; no cambia recall ni precision porque F1-11 ya es positivo). El resto de las filas contrastadas (ver abajo) es razonable. La clasificacion es juicio del implementador y el ADR lo dice; conviene anotar el caso limite.

### MEN-7 — El ADR cita una ruta del guion que ya no existe

- Donde: ADR:84.
- «`tareas/02-en-curso/TASK-073/poc/poc-f1-15.py`»: el guion esta hoy en `03-en-revision` y acabara en `04-terminadas`. El guion se ejecuta bien desde cualquier estado (usa `__file__`), pero el ADR deberia decir `tareas/<estado>/…` como F1-14. La PoC G no comprueba rutas en codigo.

## Verificacion realizada

- Puerta: repo documental, sin build ni suite. Verificacion ejecutable = la PoC. Peticion sin «Commit verificado»: corrida propia.
- **PoC**: `python -I -B tareas/03-en-revision/TASK-073/poc/poc-f1-15.py <trabajo> <repo> <Personal>` con `<trabajo>` = `%LOCALAPPDATA%/pruebas-claude/taskcodetwo/task-073-r1/poc`: exit 0, «RESULTADO: todo como se espera», 150 s en esta maquina (el ADR no da tiempo; la peticion decia ~60 s). Salida frente a `poc-f1-15.log`: **identica byte a byte tras normalizar CRLF** (`md5` `b59de5a2…` ambos; la copia de trabajo lleva CRLF por `core.autocrlf`, el objeto del commit es LF). Las 5 mutaciones de R y las 15 de G fallan cada una por su motivo (como el log). Carpeta de trabajo de la PoC borrada por el guion.
- **Mutaciones propias**: M-A detectada; M-B y M-C no detectadas (IMP-2). Control positivo de la guardia de auditoria: `socket.socket()` con la guardia activa lanza `guardia: socket.__new__ durante seleccionar` (la PoC no lo trae; funciona).
- **`seleccionar` sin red ni modelo**: leido entero (`poc-f1-15.py:238-340`): funcion sin E/S, sin imports de red; la guardia, el grep del fuente (0) y el grep de imports (0) cuadran. Determinismo y permutacion: mismo hash `539d5857d7cd`. Empate total por id: reproducido por el guion.
- **Clasificacion de los 33 IMPORTANTES** (contraste con `tareas/04-terminadas/TASK-0*/revision/`): recontados por mi cuenta las tablas de los 15 informes (identificacion por ID, igual que la PoC): 33 distintos; por tarea 0,0,1,3,5,3,2,4,0,2,5,2,1,2,3 (= tabla R). Leidos los textos de **TASK-006 (5), TASK-009 (4), TASK-012 (5), TASK-011 (2), TASK-015 (2), TASK-014 (1), TASK-007 (3), TASK-008 (2), TASK-094 (3), TASK-005 (3), TASK-013 (2), TASK-004 (1)**: 33 de 33. Existen; no hay doble cuenta (los IDs que reaparecen en rondas siguientes son el mismo hallazgo «corregido» con las lineas movidas; los IDs nuevos de cada ronda estan marcados «nuevo, introducido por la correccion» en TASK-006 IMP-3..5, TASK-009 IMP-4, TASK-011 IMP-2, TASK-012 IMP-5, TASK-013 IMP-2); no encontre un ID reutilizado para otro hallazgo ni dos IDs para uno. Clases razonables salvo MEN-6. TASK-006: 3 puerta (IMP-1, 3, 4), 1 correccion (IMP-2), 1 contrato (IMP-5). TASK-009: 4 de puerta. TASK-012: puerta IMP-2 e IMP-5 (la lectura preliminar del plan decia 1; la divergencia 6 lo explica y es correcta). Limite: la PoC identifica por ID y no puede detectar un ID reutilizado; lo comprobe a mano en las tareas citadas y el ADR no lo dice.
- **Recalculo con la tabla del ADR**: retrospectiva 16/3/6/6/2 = 33; puerta-seguridad en 8 tareas; 12 en planes sin rol de riesgos; 7 IMPORTANTES en rondas de verificacion, 5 de puerta (TASK-006 IMP-3 e IMP-4, TASK-009 IMP-4, TASK-011 IMP-2, TASK-012 IMP-5; mas TASK-006 IMP-5 y TASK-013 IMP-2). Detector: 9 disparos (7 VP + F1-4 y F1-14), FN F1-11, 5 VN → **recall 7/8, precision 7/9**; suelo sube de 1 a 2 en 8 tareas; 8 × 2 × 747 469 = 11 959 504; 1,495 M por tarea = 10,7 % de la mediana 13 942 446,5 de F1-18. Todo coincide con la prosa. Heuristica reimplementada igual que la peticion de plan en 15 de 15.
- **Muestreo**: aplicada la regla del ADR por mi cuenta con `git ls-tree` y el reparto de merges de primer padre (`merge(...): rama -> develop`) en los commits fijos: ToolsaWeb 404 `tarea.md` (160 terminadas), OpenDockTrading 96 (43 terminadas); `skills_recomendados` 21 (15/3/3) y 8 `opus` frente a 0 en OpenDockTrading, como dice el ADR; ficheros de TASK-306/299/305/300 fuera de `tareas/` coinciden con el log. La columna «comprobado a mano» queda «pendiente de Carlos» en las 10 filas; el ADR no afirma lo contrario (ver el parrafo tras la tabla de hallazgos).
- **Coherencia con ADR previos**: F1-9 (`roles` plantilla), F1-10 (`tabla-politica`, `roles_min`, carril efectivo, «solo sube»), F1-11 (D5a bloque yaml) y F1-14 (cuatro niveles, `evaluar_recursos`) contrastados; divergencias 1 a 8 del ADR existen y son ciertas; faltan MEN-5 y lo de IMP-3/MEN-4. F1-18 (mediana 13,9 M) y requisitos RR-17, RF-12..RF-15 y RR-05 cubiertos por decision y evidencia, salvo lo de IMP-2 (S13) y IMP-3.
- 0.16.1 leida en solo lectura: `catalogo-skills.ts`, `modelo-revision.ts` (escalera opus > sonnet > haiku, un escalon), `heuristica.ts` (mayor entre declarado y calculado), `revisores.ts` (umbral): coinciden con «Cómo lo hace hoy la 0.16.1».
- `git status` del repo limpio al terminar (salvo este informe). Carpeta `%LOCALAPPDATA%/pruebas-claude/taskcodetwo/task-073-r1` borrada (clones de ToolsaWeb y OpenDockTrading `--no-checkout`, tres copias del repo para las mutaciones y los scripts propios); ToolsaWeb y OpenDockTrading solo lectura, GitHub no consultado.

## Lecturas fuera del contexto permitido

- `tareas/04-terminadas/TASK-0*/revision/informe-revision-*.md` y `tarea.md` de las 15 tareas: contraste de la clasificacion y de `## Coste` pedido en el encargo.
- `docs/adr/F1-9…`, `F1-10…`, `F1-11…`, `F1-14…`, `F1-18…`, `docs/plan/AMPLIACION-2026-11-14.md`, `docs/plan/REQUISITOS-FUNCIONALES.md`, `docs/plan/PLAN-TASKCODE-2.md` (por grep): contraste de divergencias y requisitos pedido en el encargo.
- `tareas/03-en-revision/TASK-073/poc/poc-f1-15.py`, `.log` y `clasificacion-hallazgos.md`: entregable a reproducir.
- Caché del plugin 0.16.1 (`scripts/heuristica-complejidad.yml`, `src/core/*.ts`): comprobar lo que el ADR dice de la 0.16.
- Clones solo lectura de ToolsaWeb y OpenDockTrading en los commits fijos: reaplicar el muestreo y contar `skills_recomendados`.
