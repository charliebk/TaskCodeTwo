# Plan — TASK-008: F1-7 ADR verificacion en CI, perfiles, cache por SHA y runners

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 60 min

Senales: la heuristica dice «simple» (solo la dependencia TASK-006), pero el ADR cierra puntos abiertos de F1-1, F1-3 y F1-5; es un unico documento con una PoC local pequena: no se trocea. Datos de coste ya localizados (lectura hecha en la planificacion, sin tocar nada):

- ToolsaWeb, `C:\Users\nullcad2025\Documents\sources\Personal\ToolsaWeb\docs\verificacion\flujo-de-verificacion.md`: completa con cache caliente 2 min 50 s a 14 min 31 s; primera completa con cache vacia y sin pesados 22 min 22 s; con pesados 44 min 52 s (29 min del paso 18); `.taskcode\config.yml` con cuatro perfiles (rapida, selectiva, completa, pesada).
- OpenDockTrading, `...\OpenDockTrading\.taskcode\config.yml`: 4 min 11 s en frio y 1 min 56 s en caliente (medido el 2026-11-13); un solo perfil; `verificar.bat` solo Windows (H16).
- Ritmo (campo `creado` de los frontmatter): ToolsaWeb 159 terminadas entre 2026-10-16 y 2026-11-13 (~165/mes); OpenDockTrading 40 entre 2026-11-09 y 2026-11-13 (ritmo de arranque, no estable).

Que se crea: `docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md` (plantilla de `docs/adr/README.md`) y su fila en el indice; `poc/poc-f1-7.sh` y `poc/poc-f1-7.log`, ejecutados en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-008\` (borrada al terminar).

Que no se toca: el ADR F1-5 (lo que corrige va a «Divergencias» y «Consecuencias» de F1-7); `.github/ci.yml` de este repo (RR-14); ToolsaWeb y OpenDockTrading solo lectura; GitHub solo `GET`.

### Que decide el ADR

1. **Contrato del comando del proyecto**: un solo comando con `--perfil rapida|impacto|completa|pesada` y el codigo de salida como resultado; sustituye a los seis `comando_*` (H18, RF-41); la declaracion en la configuracion es de F1-11. El workflow reutilizable y la plantilla de GitLab solo llaman a ese comando (P6, P11). Inclusion: rapida ⊂ impacto ⊂ completa; la pesada es aparte y no sustituye a la completa.
2. **Clave de cache = (arbol verificado, perfil, huella del entorno declarada por el proyecto, version del workflow)**: el SHA del arbol solo no basta (ToolsaWeb ya mete herramientas y carpetas ignoradas que leen sus tests, como `exampleData` de 34 GB). El proyecto declara su huella (extension de F1-11); se reutiliza lo que hace `cache-verificacion.py`.
3. **Donde vive el resultado**: en el propio check `verificacion` publicado por Actions (`integration_id` 15368, F1-5), con `arbol=`, `perfil=`, `clave=` en el resumen. Busqueda por padres: un check verde de la App 15368 sobre un padre con el mismo arbol y la misma clave es acierto y se publica en verde en segundos («reutilizado de <sha>»). Con `strict` en `develop` el arbol del head del PR coincide con el del merge commit; con la release por merge (F1-3) el de `main` coincide con el de `develop`: PR, cola y release comparten resultado sin indice por arbol.
4. **Perfil segun el evento**: hook local rapida, opcional y solo con confianza (F1-8); PR a `develop` completa (D1); PR a `main` completa desde la cache mas la pesada, en el mismo check (D4).
5. **Runner segun el perfil**: cada perfil declara su runner (`github` o etiquetas `[self-hosted, windows, ...]`) que el workflow recibe en `runs-on`. Repo publico: solo runners de GitHub (F1-8). La pesada de ToolsaWeb solo en la maquina de Carlos (datos locales y umbrales de tiempo, RN-02, F5-5). Runner self-hosted caido: falla cerrado (D3).
6. **Tabla de coste**: ejecuciones/mes × minutos/ejecucion × precio, con fuente y fecha de cada entrada, generada por el guion (seccion C).

### Orden

1. PoC A (git local): arbol del head del PR al dia = merge `--no-ff` en `develop` = release en `main`; en «PR desactualizado» el arbol verificado es el del merge ref.
2. PoC B: simulacion de la busqueda por padres con controles negativos (otro perfil, otra huella, check en rojo, otra App).
3. PoC C: tabla de coste desde las entradas medidas; descarga de la OpenAPI de GitHub en su propia carpeta vacia para confirmar que `GET /commits/{ref}/check-runs` admite `check_name` y `app_id` y que existe `output.summary`.
4. ADR con alternativas, decisiones, divergencias con F1-5 y tareas que lo ejecutan (F5-1, F5-2, F5-5, F1-8, F1-11, F1-13, F6).
5. Fila del indice.

Estimacion preliminar (el ADR la rehace): ToolsaWeb en runners de GitHub ~12 700 min/mes de Windows (~127 $/mes: 165 tareas × ~3,5 ejecuciones × ~22 min en frio; factor de velocidad del runner frente a la maquina de Carlos no verificado); ToolsaWeb en su maquina con cache caliente 0 $/min (tasa self-hosted de ~0,002 $/min si GitHub la aplicara); OpenDockTrading ~4 min en frio, del orden de 2 000-3 500 min/mes. Precios (Windows 2 nucleos 0,010 $/min desde 2026-01; tasa self-hosted anunciada y aplazada) **no verificados**: se comprueban en docs.github.com/billing con fecha.

## Lo que el rol no cubrio

- **Criterio «coste estimado»**: modos de fallo de la estimacion (factor de velocidad del runner, ritmo de OpenDockTrading con cinco dias de datos, cero releases medidas para estimar la pesada).
- **Criterio «cumple F1-7»**: casos borde de la busqueda por padres, estrategia de pruebas (solo esbozada) y reglas de negocio de cada perfil.
- Seguridad del runner self-hosted con codigo del PR, `pull_request` frente a `pull_request_target`, secretos: F1-8.
- Equivalentes en GitLab: F1-13; el ADR deja el modelo neutral.

## Riesgos aceptados y que los contiene

- **Pasos que no dependen solo del arbol** (`cargo audit`, `npm audit`, la red de OpenDockTrading): cachearlos por arbol esconde avisos nuevos; se sacan a un job programado fuera del check cacheado (detalle en F1-11).
- **Cola de merge**: no verificado que el arbol del commit `merge_group` coincida con el del head; si no coincide, falla la cache y se verifica otra vez (caro pero seguro); lo comprueba F1-13 o F2-6.
- **`taskctl roadmap --comparar` de OpenDockTrading** deja de tener sentido cuando el roadmap sale del arbol (F1-1): consecuencia para la migracion (F9).
- **Precios no verificados**: cada cifra con fuente y fecha, o «no verificado».

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio). Ninguno pesado.

- **Coste estimado con las suites de ToolsaWeb y OpenDockTrading → 2**
  1. T1: PoC C calcula la tabla desde las entradas citadas (fichero y linea) y la compara con la del ADR.
  2. T2: control negativo: al cambiar una entrada, el total cambia y la comparacion lo detecta.
- **Cumple F1-7 y los requisitos que cita → 3**
  1. T3: PoC A: arboles iguales en PR al dia, merge en `develop` y release en `main`; distintos en el PR desactualizado.
  2. T4: PoC B con al menos 6 casos de acierto o fallo y al menos 4 controles negativos.
  3. T5: tabla «Requisitos y evidencia» con RR-07, RF-41, RF-42, RN-02, las cuatro vinetas de P6, RR-15 y H16 (cada ID comprobado antes de citarlo), y la comprobacion en la OpenAPI de los filtros de check-runs; al cerrar, `bash .github/scripts/comprobar-repo.sh` en verde y `taskctl doctor` sin errores.

## Lo que necesita decision de una persona

Decisiones respondidas por la persona (Carlos) el 2026-11-14 con la consigna «responder la opcion recomendada a todas las preguntas»:

- **D1. Perfil de `verificacion` en los PR a `develop`** → **(a)** completa, proporcionada por el proyecto (puerta que cachea RF-42 y se reutiliza en la release). Descartadas (b) rapida en `develop` y completa en `main` (lo literal de las consecuencias de F1-5: se corrige como divergencia) y (c) impacto.
- **D2. Donde vive el resultado de la cache** → **(a)** en el check de Actions, buscando por los padres con el mismo arbol. Descartadas (b) `actions/cache` (alcance por rama) y (c) ref, nota o `taskcode/tareas` (el agente puede escribir).
- **D3. Runner self-hosted caido** → **(a)** check pendiente, falla cerrado, `doctor` lo dice.
- **D4. Pesada en la release** → **(a)** dentro de `verificacion` cuando la base es `main`, sin check nuevo ni enmienda a F1-5.
- **D5. Clave de cache** → **(a)** arbol, perfil y huella declarada por el proyecto (y version del workflow).
- **D6. Medir las suites** → **(a)** no se ejecutan; tiempos locales medidos y factor de velocidad «no verificado».

Sin troceo: la duracion estimada no pasa de 60 min.
