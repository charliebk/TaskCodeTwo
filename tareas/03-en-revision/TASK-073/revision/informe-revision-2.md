# Informe de revision — TASK-073 (ronda 2)

- Commit revisado: 3a11b7d5f06ab04c7f968196e2ff98b1822451a3 (HEAD de trabajo 3911c36, solo anade la peticion)
- Revisor: code-quality-reviewer
- Veredicto: aprobada-con-correcciones

Pase generico, verificacion acotada al diff de las correcciones (806f024 y 5768e3c). Profundidad: a fondo el interprete de `tabla-seleccion` (`seleccionar`, `parsear_regla`, `_cumple`), `dominios()`, los controles de reglas y de evasion y la tabla de la muestra; por encima la prosa de «Cómo se ve» y «Consecuencias».

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:61 | plan |
| IMP-2 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-073/poc/poc-f1-15.py:294 | implementacion |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:428 | plan |
| MEN-8 | MENOR | abierto | docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:61 | implementacion |
| MEN-9 | MENOR | abierto | tareas/03-en-revision/TASK-073/poc/poc-f1-15.py:1273 | implementacion |
| MEN-10 | MENOR | abierto | tareas/03-en-revision/TASK-073/poc/poc-f1-15.py:208 | implementacion |
| MEN-11 | MENOR | abierto | docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:280 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

No hay CRITICO ni IMPORTANTE abierto. Las correcciones no abren ningun IMPORTANTE nuevo; lo que encontre va como MENOR al final (no abre otra ronda).

## Reproduccion

### Puerta
Repo documental, sin build ni suite; la verificacion ejecutable es la PoC. La peticion no trae «Commit verificado»: ejecucion propia.

- `python -I -B poc-f1-15.py <trabajo> <repo> <Personal>` con `<trabajo>` = `%LOCALAPPDATA%/pruebas-claude/taskcodetwo/task-073-r2/poc`: «RESULTADO: todo como se espera», 91 s. La salida (normalizado CRLF a LF) es **identica caracter a caracter** a `poc-f1-15.log` (28 102 caracteres ambas). Las 30 lineas «mutacion … falla como se espera» del log (5 de R + 25 de G) se reproducen, cada una por su motivo; la carpeta de trabajo la borra el guion.

### IMP-1 — corregido
El motivo de D1b (ADR:61) ya es lo medido: la lista de riesgo de la 0.16 no trae palabras de puertas y solo 1 de 15 tareas contiene una palabra de riesgo (F1-8). Lo comprobe contra el log (`tareas con alguna palabra de riesgo … : 1 (TASK-009:seguridad)`) y contra `heuristica-complejidad.yml:159` (14 palabras, ninguna es puerta/check/ruleset/runner/permiso). El ADR deja de citar F1-13 y F1-18 como ejemplo de «seguridad suma 2 puntos». La cifra la fija la PoC R (`con_pal` en la prosa). Ver MEN-A por una imprecision de redaccion nueva.

### IMP-2 — corregido
- `seleccionar` ya **interpreta** `tabla-seleccion`: `parsear_regla` valida cada condicion y accion contra un vocabulario cerrado anclado (`ATOMOS`, `ACCIONES`, `re.fullmatch`), `_cumple` evalua los atomos con negacion, y el bucle recorre las filas del ADR en su orden (poc-f1-15.py:347-412). Una condicion o accion fuera del vocabulario es `ErrorTabla` (mutacion `condicion-desconocida`, con «nunca»: falla).
- Hay un caso de control por fila S01..S15 mas cuatro de S13 (`controles_reglas`), y la PoC falla si una fila no se ejercita («la fila X no tiene ningun caso»). El log lo muestra: S13 ronda 2 sin dominio baja `opus`→`sonnet`; en riesgo-alto, con puertas por texto y con puertas por ruta `.claude/settings.json` sigue `opus`. S10 y S06 con skill/revisora sinteticas; S04 con `wasm`; S01 con errata.
- Las tres mutaciones que la ronda 1 dejo sin detectar (S13 «siempre», S03 suelo 3, S10 con la generica) estan ahora en G y fallan (`s13-siempre`, `s03-suelo-3`, `s10-da-la-generica`).
- La frase del log sobre S13 ya no dice «arriba».
- Mutaciones propias sobre una fila de `tabla-seleccion` (copias del repo en la carpeta temporal, ADR mutado, PoC completa):
  - M1: S11 con condicion `dominio-sin-revisora` y no `ronda>=2` → `RESULTADO: 2 fallos` (casos 3 y 4 de la evasion; los controles S no la ven, pero G/S si). **Detectada.**
  - M2: S09 con condicion `rutas-no-casan-o-pasan-umbral` y no `carril=fix` → `RESULTADO: todo como se espera`. **Sobrevive** (ver MEN-B).
- Determinismo y permutacion: mismo hash en el log (`539d5857d7cd` en la ronda 1; la PoC lo vuelve a comprobar y sale «iguales»).

### IMP-3 — corregido
ADR decision 3 («Texto de la tarea: no confiable, solo sube»), decision 9 (texto congelado con la firma, `roles.<rol>.texto`) y `dominios()` (poc-f1-15.py:208-234: la union de las senales del texto aprobado y del vivo; solo anade). Los cinco escenarios de la tabla del ADR estan en `caso_vocabulario` con su resultado fijado y la PoC falla si cambia: 1 y 3 y 4 contenidos; 2 (plan, otro vocabulario) y 5 (`fix` sin `approve`, diff solo en `src/`) **declarados como no contenidos por el binario**, con quien los contiene (la persona en `approve`, que ve «dominios: ninguno; roles 1»; la persona del `merge-pr`) y metrica F8-6. Reproducido en el log: caso 5 da `sonnet`, sin aviso y con S13 (riesgo aceptado, no escondido). Tambien `Catalogo del arbol del PR` (MEN-4 de la ronda 1) con tres casos: la funcion contiene la generica ajena y **no** contiene una revisora ajena, y el ADR dice que por eso la base se lee siempre con `git show`.
Evasion propia distinta de las cinco (script en el scratchpad, `importlib` sobre la PoC y las tablas del ADR, inventario real):
- E6, palabra incrustada («recheck», «subtoken», «autoworkflow»; la senal es prefijo de palabra con `\b`): 1 rol, sin senales, ronda 2 `sonnet`. Misma clase que el caso 2 del ADR (el binario no la contiene; la persona en `approve`): **declarada por clase**.
- E7, sinonimos en ingles («gate approval … credential secret»): 1 rol, sin senales. Misma clase, declarada.
- E8b, homoglifos cirilicos en «puerta» y «token»: 1 rol, sin senales; con homoglifo solo en «check» pero «puerta» intacto, sigue dando el suelo. Misma clase, declarada.
- E9, ruta con otra capitalizacion (`.Claude/settings.json`): sin senal, sin aviso, S13 baja a `sonnet`. **No declarada** (MEN-C). Con `.claude/…` como renombrado: se detecta (aviso y `opus`).

### Cambio de clase de TASK-012 IMP-1 y cifras
Recalculado en todo el repo (grep de «16 de 33», «correccion 6», «12 de ellos»): quedan 17 de 33 de puerta-seguridad, 13 de ellos en planes de un solo rol, 13 escapados, correccion 5, y 17/3/5/6/2 = 33. Coincide con el log de la PoC (puerta-seguridad 17; el `.log` y `clasificacion-hallazgos.md` estan recalculados) y las cifras de la prosa estan guardadas por la PoC G (mutacion `prosa-importantes`). Ninguna cifra vieja (16, 12, 6 de correccion) queda en el ADR, el README ni los ficheros de `poc/`. Recall 7/8 y precision 7/9 no cambian (F1-11 ya era positivo).

### Columna «comprobado a mano (Carlos)» y las 10 filas
- `tarea.md` tiene la seccion «Decisiones de la persona» (commit `5768e3c`, 4 lineas anadidas): «2026-11-14, Carlos … "Coincide las 10"», con los 10 ID. El ADR la cita (cabecera linea 7, parrafo tras la tabla T y «No verificado») y cada fila dice «coincide (Carlos, 2026-11-14)»; la regla de la PoC G exige persona y fecha.
- Compare las 10 filas de la tabla T en `90330c6` con las de HEAD, columna a columna (decodificando UTF-8): **las 11 columnas anteriores a «comprobado a mano» son identicas en las 10 filas**; solo cambia la ultima («pendiente de Carlos» → «coincide (Carlos, 2026-11-14)»). Es decir, ninguna propuesta cambio respecto a la que Carlos marco, y la PoC las recalcula con las tablas actuales (si cambiara una, G falla con «fila distinta de lo calculado»).
- Limite menor: la afirmacion «Las correcciones no cambiaron ninguna de las 10 propuestas» la valida la PoC, no solo la prosa. Bien.

### Lo que busque como fallo nuevo de la correccion
- El interprete: acciones y condiciones son cadenas cerradas y ancladas; las semanticas que lleva fijas el codigo (`roles=max…` usa `ROLES_MIN`/`NCX`, `suelo=n`, `modelo-baja=n`) siguen las cifras del texto de la accion solo en los numeros; el resto de la «Qué significa» es prosa. Esto es declarado (el ADR dice «vocabulario cerrado»), no es un fallo.
- `dominios()` con texto aprobado: el union solo anade; sin `aprobado` (plan, `fix`) se comporta como antes. No hay camino por el que el texto vivo quite una senal aprobada.
- S13 con puertas por ruta/texto/carril: reproducido en los cuatro controles.

## MENORES (no abren ronda; fuera de la tabla)

- **MEN-A — ADR:61 y ADR:147 («la lista … no tiene ninguna palabra de puertas») es inexacta con «seguridad».** «seguridad» esta en la lista de la 0.16 (`heuristica-complejidad.yml:159`) y tambien es senal de puertas (D01 etiqueta y D18 palabra). La propia frase cita a F1-8 y «seguridad» como la unica coincidencia. El log lo oculta porque `poc-f1-15.py:868` comprueba una lista de 14 palabras de puertas de la que **excluye `seguridad`** y concluye «no tiene ninguna de las palabras de puertas de tabla-dominios: True». Correccion: decir «ninguna salvo `seguridad`» y calcular la lista de puertas desde `tabla-dominios` en vez de escribirla en el guion.
- **MEN-B — La PoC garantiza que cada fila de `tabla-seleccion` se dispara en algun caso, no que cada atomo de su condicion este acotado.** M2 (S09 con «y no `carril=fix`») sobrevive: ningun caso de revision usa carril `fix` sin que casen las rutas (TASK-399 pasa por S08). Efecto real de M2: una tarea `fix` solo con ficheros sin revisora, p. ej. un `.md`, se queda sin ninguna revisora. El ADR dice «La PoC falla si una fila no la ejercita ningún caso», que es verdad pero no es «las condiciones estan cubiertas». Correccion: un caso de S09 con `fix` y diff sin dominio (y, en general, un caso por cada atomo de carril de las filas de revision), o acotar la frase.
- **MEN-C — La senal de ruta es sensible a mayusculas y el ADR dice «diga lo que diga el texto».** `.Claude/settings.json`, `.TaskCode/…` no dan `puertas-seguridad` (E9); en Windows y macOS (sistema de ficheros que no distingue mayusculas) es la misma carpeta que `.claude/` y `.taskcode/`, y la plataforma de quien escribe el ADR es Windows. Es una evasion por ruta, distinta de las de vocabulario y no declarada. Correccion: casar las rutas de `tabla-dominios` sin distinguir mayusculas (y un caso en E), o declararlo con las otras evasiones.
- **MEN-D — ADR:428 / decision 3, «puertas por texto» y `etiquetas` aprobadas:** `dominios()` toma las etiquetas aprobadas como parte del texto aprobado; el ADR dice «etiquetas fijadas al aprobar» solo en la anotacion de `roles.<rol>.texto` como blob de `tarea.md`. Es coherente, pero queda sin caso en la PoC (ningun caso de `caso_vocabulario` usa etiqueta aprobada distinta de la viva). Sin efecto hoy.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-073/poc/poc-f1-15.py`, `poc-f1-15.log`, `clasificacion-hallazgos.md`: el entregable a reproducir (petición de la tarea).
- `tareas/03-en-revision/TASK-073/revision/informe-revision-1.md`: el informe de la ronda 1 que pide verificar la peticion.
- `git show 90330c6` (tabla T) y `git show 5768e3c` (`tarea.md`): comparacion de las 10 filas y constancia de Carlos pedida en el encargo.
- Cache del plugin 0.16.1: `scripts/heuristica-complejidad.yml` (palabras de riesgo) para IMP-1, y el inventario real de la maquina (`~/.claude`, `installed_plugins.json`) que la propia PoC lee.
- Ni ToolsaWeb ni OpenDockTrading se tocaron en esta ronda (los lee la PoC en solo lectura desde sus clones en la carpeta temporal). GitHub no se consulto.

## Limpieza

Carpeta `%LOCALAPPDATA%/pruebas-claude/taskcodetwo/task-073-r2` (copias mutadas del repo `m1` y `m2`, salidas de la PoC y trabajo de la PoC) borrada al terminar; no habia commits sin subir ni nada que conservar en ella. `git status` del repo limpio salvo este informe.

> Nota del orquestador: MEN-8 (MEN-A: la lista de la 0.16 si tiene `seguridad`; la PoC usa una lista escrita a mano en poc-f1-15.py:868), MEN-9 (MEN-B: cada atomo de la condicion de cada fila debe estar acotado; la mutacion S09 con «y no carril=fix» sobrevive), MEN-10 (MEN-C: la senal de ruta distingue mayusculas; `.Claude/` no da puertas en Windows y macOS) y MEN-11 (MEN-D: las etiquetas aprobadas no se ejercitan en ningun caso) son las notas del revisor pasadas a la tabla para corregirlas.
