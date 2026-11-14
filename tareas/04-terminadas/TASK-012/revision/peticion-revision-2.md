# Peticion de revision — TASK-012 (ronda 2)

- Tarea: TASK-012 — F1-11 ADR extensiones declaradas
- Rama revisada: feature/task-012-f1-11-adr-extensiones-declaradas
- Rama base: develop
- Commit revisado (HEAD): 054468e9c3f2315bb351d2ecb0e970649becb914
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-012 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 7c532cc7aa97b984ec835c5159843ecedeb6f4df (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (9 hallazgo(s)))

## Instrucciones para el agente revisor (verificacion acotada)

Eres un revisor INDEPENDIENTE del agente que implemento. La rama ya paso una
revision completa: esta ronda verifica las correcciones de la ronda anterior,
no vuelve a revisar la rama entera ni los MENORES de la ronda anterior (tu
tabla solo trae CRITICOS e IMPORTANTES). Haz esto, y nada mas:

1. Comprueba, uno por uno, que cada hallazgo pendiente de tu tabla de abajo
   queda resuelto por el diff de esta ronda (los marcados `corregido` tambien:
   verificalo, no lo des por bueno). Para un CRITICO o IMPORTANTE, reproduce el
   fallo original y comprueba que ya no ocurre; puedes leer el repo entero si la
   causa no esta en el diff.
2. Ejecuta la suite (o el comando de verificacion del proyecto) sobre el commit
   revisado y di el resultado: verificar sin ejecutar no vale. Si esta peticion
   ya trae la verificacion de la ronda en verde para ese mismo commit, no la
   repitas: cita ese resultado y corre solo los tests de lo que verificas.
3. Busca en el diff de esta ronda el fallo que la correccion abre: es justo
   donde se cuelan los fallos nuevos. Si encuentras uno, construye el caso que
   lo rompe antes de reportarlo.

Solo abre un hallazgo NUEVO si es CRITICO o si lo introduce la propia correccion
(algo que el diff de esta ronda rompe). Lo demas que veas fuera de eso va como
nota al final del informe, no en la tabla: no abre otra ronda.

Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. En la tabla, cada hallazgo pendiente que ya este
resuelto va como `corregido`; el que no, `abierto`. Vuelca tu salida en el
informe de esta ronda (informe-revision-2.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-012/tarea.md
- tareas/03-en-revision/TASK-012/planificacion/
- docs/adr/F1-11-extensiones-declaradas.md
- docs/adr/README.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 5 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:375 | informe-revision-1.md |
| IMPORTANTE-2 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:350 | informe-revision-1.md |
| IMPORTANTE-3 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:452 | informe-revision-1.md |
| IMPORTANTE-4 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:457 | informe-revision-1.md |

## Commits a revisar (git log 7c532cc7aa97b984ec835c5159843ecedeb6f4df..HEAD)

````
054468e chore(TASK-012): constancia de autorrevision ronda 2
81aee93 docs(adr): F1-11 corrige los hallazgos de la revision 1 (hitos en la rama de estado, check estado, rutas, ventaja honesta)
1366c2c chore(TASK-012): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 7c532cc7aa97b984ec835c5159843ecedeb6f4df..HEAD)

````diff
diff --git a/docs/adr/F1-11-extensiones-declaradas.md b/docs/adr/F1-11-extensiones-declaradas.md
index 9939f5b..2dcfb51 100644
--- a/docs/adr/F1-11-extensiones-declaradas.md
+++ b/docs/adr/F1-11-extensiones-declaradas.md
@@ -1,10 +1,10 @@
-# ADR F1-11 — Extensiones declaradas: tres claves, derivados en la transición y lectura estable del estado
+# ADR F1-11 — Extensiones declaradas: tres claves, derivados en la transición, check `estado` y parches que se retiran
 
-- Estado: **aceptada** (2026-11-14)
+- Estado: **aceptada** (2026-11-14; revisada tras la revisión 1 de TASK-012)
 - Tarea: TASK-012 · Plan: [F1-11 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (§4 P8 y tabla de la Fase 1)
-- Requisitos: la fila F1-11 del plan cita P8; este ADR responde a RN-04, RF-07, RF-08, RF-40, RF-41, RF-42, RF-70, RF-71, RF-72 y RR-11 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md) y a H2 y H16 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
-- Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (carpeta fija, rama de estado huérfana, roadmap con un Mermaid por hito) y [ADR F1-7](F1-7-verificacion-ci-perfiles-cache-runners.md) (un comando con perfiles, huella del árbol y local, runner por perfil, job programado); cierra costuras abiertas por [F1-8](F1-8-seguridad-modelo-de-amenazas.md) (confianza local única, globs extra, `comando_*` como entrada no confiable, rutas de Windows en YAML), [F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md) (claves desconocidas del frontmatter) y [F1-10](F1-10-carriles-y-modo-automatico.md) (configuración que falla cerrada, tope de 12 claves)
-- Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a y D6a del plan de TASK-012), «no trocear» y una restricción expresa: el plugin tiene que ser **eficiente y con ventajas medibles**, y prefiere **comandos deterministas** que sabe que hacen lo que dicen a hooks que arrancan algo según el contexto
+- Requisitos: la fila F1-11 del plan cita P8; este ADR responde a RN-04, RF-07, RF-08, RF-40, RF-41, RF-42, RF-70, RF-71 y RR-11 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md) y a H2 y H16 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md)
+- Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (carpeta fija, rama de estado huérfana, roadmap con un Mermaid por hito) y [ADR F1-7](F1-7-verificacion-ci-perfiles-cache-runners.md) (un comando con perfiles, huella del árbol y local, runner por perfil, job programado); cierra costuras abiertas por [F1-8](F1-8-seguridad-modelo-de-amenazas.md) (confianza local única, globs extra, `comando_*` como entrada no confiable, rutas de Windows en YAML), [F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md) (claves desconocidas del frontmatter) y [F1-10](F1-10-carriles-y-modo-automatico.md) (configuración que falla cerrada, tope de 12 claves); cambia la lista de checks obligatorios de [F1-5](F1-5-puertas-en-github.md) (ver «Divergencias con F1-5 y F1-7»)
+- Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a y D6a del plan de TASK-012, y las correcciones de la revisión 1), «no trocear», la restricción de que el plugin sea **eficiente y con ventajas medibles** y prefiera **comandos deterministas** a hooks, y la decisión del 2026-11-14: «todo lo que hace funcionar al plugin 2 debe estar en su repo y debe ser generalista; al actualizar a la 2 hay que quitar los parches que se aplicaron para mejorar la 0.16 dentro del repo»
 
 ## Contexto
 
@@ -23,9 +23,10 @@ Lo que ya está decidido y este ADR no cambia:
   bastan a los proyectos con `05-archivadas` (RF-71).
 - **F1-7**: un solo comando del proyecto llamado con
   `--perfil rapida|impacto|completa|pesada`; código 0 verde, cualquier otro
-  rojo; huella del árbol y huella local; runner por perfil; los pasos que
-  dependen de la red salen a un job programado. Dejó a F1-11 las claves, y
-  que `completa` y `pesada` no dependan de `RAMA_BASE`.
+  rojo; huella del árbol y huella local; runner por perfil (`github` con el
+  sistema, o etiquetas de un runner propio); los pasos que dependen de la red
+  salen a un job programado. Dejó a F1-11 las claves, y que `completa` y
+  `pesada` no dependan de `RAMA_BASE`.
 - **F1-8**: `confianza(repo, comando) -> confiado | cambiado | nunca` es la
   única interfaz para todo `comando_*`; los globs extra de la huella los
   declara el proyecto (F1-11); con un lector YAML estándar,
@@ -36,10 +37,16 @@ Lo que ya está decidido y este ADR no cambia:
   cerrada ante lo desconocido y suma al tope de 12 claves de F2-7
   (`politica_carriles` y `rutas_carril`).
 
-La restricción de la persona fija el criterio: el contrato de extensión son
-**comandos declarados que `taskctl` ejecuta en puntos explícitos**
-(transición, verificación). Ningún punto de extensión depende de un hook; un
-hook, si existe, solo es un atajo opcional que llama al mismo comando.
+Dos restricciones de la persona fijan el criterio. La primera: el contrato de
+extensión son **comandos declarados que `taskctl` ejecuta en puntos
+explícitos** (transición, verificación, check del PR); ningún punto de
+extensión depende de un hook, y un hook solo puede ser un atajo opcional que
+llama al mismo comando. La segunda: lo que hace funcionar a la 2.0 vive en su
+repo y es genérico, y lo que cada proyecto añadió para que la 0.16 funcionara
+se **retira al migrar**. Por eso cada práctica del inventario lleva una
+clasificación: **parche** de TaskCode (existe para que la 0.16 funcione o para
+tapar sus carencias, y la 2.0 lo cubre de forma genérica: se retira) o
+**propio** del proyecto (cómo se verifica o qué es el proyecto: se queda).
 
 ### Inventario de lo que se hace hoy a mano
 
@@ -50,31 +57,101 @@ el fichero existe en ese commit y las líneas contienen el texto entre «».
 dice cómo se hace hoy: con una clave de la configuración, con algo que pone
 el plugin o a mano. «Destino» dice dónde queda en la 2.0: una clave de la
 tabla «Esquema» (decisión 1), algo nativo del CLI, una skill o fuera de
-alcance.
+alcance. «Clasificación» es parche o propio (arriba). «En la PoC» dice qué
+respalda el destino: **comprobado** (la PoC ejecuta o valida la traducción),
+**proyectado** (depende de una interfaz que la PoC simula o que otra tarea
+construye), **dentro del comando** (la 2.0 no le da clave: hoy vive dentro del
+comando del proyecto), **convención** (una regla de escritura más el
+validador) o **fuera**.
 
 <!-- inventario:inicio -->
-| # | Práctica | Evidencia (repo fichero:línea «texto») | En la 0.16 | Destino | Punto de extensión en la 2.0 |
-|---|---|---|---|---|---|
-| 1 | Estado propio `05-archivadas`: carpeta, etiqueta `descartada` y un `estado` que no lo dice | `TW tareas/05-archivadas/TASK-171/tarea.md:6` «descartada»; `TW tareas/05-archivadas/TASK-171/tarea.md:9` «estado: planificada»; `ODT tareas/05-archivadas/TASK-068/tarea.md:6` «descartada»; `ODT tareas/05-archivadas/TASK-068/tarea.md:9` «estado: planificada» | a mano | nativo | estado `descartada` con motivo y `pausa` (F3-7); sin clave de estados propios (D1a) |
-| 2 | Regenerar el Mermaid de `hitos.md` y `ROADMAP.md` en cada transición | `TW .taskcode/config.yml:29-32` «comando_sincronizacion»; `TW scripts/grafo-tareas.py:4-5` «regenera el Mermaid»; `ODT tareas/05-archivadas/TASK-068/tarea.md:38` «la transición sigue y avisa» | clave | clave | roadmap nativo con un Mermaid por hito (F1-1 dec. 3); lo demás, un derivado `genera` (`derivados[].tipo`) que solo escribe bajo `tareas/` (`derivados[].escribe`, D2a) |
-| 3 | Comprobaciones que cruzan código y estado, dentro de la verificación | `TW scripts/verificar.bat:124-127` «grafo-tareas.py --comprobar»; `ODT scripts/verificar.bat:51` «comprobar-modulos.py --comprobar»; `ODT scripts/verificar.bat:53` «comprobar-fallos.py --probar»; `ODT scripts/comprobar-modulos.py:88` «lector-tareas.py»; `ODT scripts/comprobar-fallos.py:302-307` «tareas_vivas» | a mano | clave | derivado `comprueba` (`derivados[].tipo`) en cada transición, fuera del perfil cacheado, más el check `estado` en el PR (D3a) |
-| 4 | Lector propio de frontmatter que recorre `tareas/0?-*/TASK-*/tarea.md` | `TW scripts/grafo-tareas.py:99` «raiz_tareas.glob»; `ODT scripts/lector-tareas.py:39` «raiz_tareas.glob» | a mano | nativo | `taskctl tareas --json` como lectura estable; la forma de las carpetas no es contrato |
-| 5 | Cuatro perfiles con seis claves de comando y tiempo | `TW .taskcode/config.yml:9` «comando_verificacion:»; `TW .taskcode/config.yml:16` «comando_verificacion_freno»; `TW .taskcode/config.yml:18` «comando_verificacion_selectiva»; `TW .taskcode/config.yml:23` «comando_verificacion_pesada»; `TW docs/verificacion/flujo-de-verificacion.md:9-12` «comando_verificacion_freno» | clave | clave | `verificacion.comando` llamado con `--perfil` (F1-7 dec. 1), con su `verificacion.shell` y `verificacion.perfiles.<p>.runner` |
-| 6 | Tiempo máximo por perfil y presupuesto del freno | `TW .taskcode/config.yml:10` «timeout_verificacion: 4800»; `TW .taskcode/config.yml:17` «presupuesto_freno_s: 180»; `TW .taskcode/config.yml:20` «timeout_verificacion_selectiva: 1800»; `ODT .taskcode/config.yml:10` «timeout_verificacion: 1200» | clave | clave | `verificacion.perfiles.<p>.timeout` |
-| 7 | Caché propia por paso y «completa del mismo árbol» | `TW scripts/cache-verificacion.py:1` «Caché por paso de verificar.bat»; `TW scripts/verificar.bat:91-93` «completa-leer»; `TW docs/verificacion/flujo-de-verificacion.md:164-165` «mismo árbol exacto» | a mano | clave | `verificacion.huella_arbol` y `verificacion.huella_local` (F1-7 dec. 2); la caché por paso sigue dentro del comando |
-| 8 | Canales de texto: el x/N de `TASKCTL_PROGRESO` y la línea «# tests N» | `TW scripts/verificar.bat:51-52` «TASKCTL_PROGRESO»; `TW .taskcode/config.yml:7-8` «# tests N»; `PL src/fs/progreso.ts:33` «TASKCTL_PROGRESO» | plugin | nativo | variable de progreso que pasa `taskctl`; el recuento, al informe JSON (F5-4) |
-| 9 | `RAMA_BASE` como entrada del comando | `TW scripts/clasificar-verificacion.py:14-15` «RAMA_BASE (la pasa TaskCode 0.13)»; `TW scripts/clasificar-verificacion.py:54-59` «os.environ.get("RAMA_BASE")» | plugin | nativo | `RAMA_BASE` solo en `rapida` e `impacto` (F1-7 dec. 1); la completa de ToolsaWeb no depende de ella (PoC E) |
-| 10 | Pasos que dependen de la red | `ODT scripts/verificar.bat:68-69` «cargo audit --no-yanked --deny unsound»; `ODT scripts/verificar.bat:70` «npm --prefix web audit» | a mano | clave | `verificacion.programada.comando` con `verificacion.programada.cron`: job programado, fuera del check cacheado (F1-7 dec. 2) |
-| 11 | Cerrojo propio con códigos 3 y 4 | `TW scripts/cerrojo-verificacion.py:6-7` «3 si hay otra»; `TW docs/verificacion/flujo-de-verificacion.md:182` «código 3»; `ODT scripts/verificar.bat:16-17` «impide dos» | a mano | clave | sigue dentro de `verificacion.comando`; para el contrato, cualquier código distinto de 0 es rojo (D6a) |
-| 12 | Ruta de Windows en YAML (`"\v"`) | `ODT .taskcode/config.yml:5-6` «seria un escape»; `TW .taskcode/config.yml:9` «"scripts\verificar.bat"» | a mano | clave | `verificacion.comando` y `derivados[].comando` entre comillas simples o con `/`; el validador rechaza un carácter de control (PoC B) |
-| 13 | Revisoras propias: ningún proyecto las declara; el catálogo sale del plugin | `PL src/core/revisores.ts:69-74` «CLAUDE_PLUGIN_ROOT»; `PL src/core/revisores.ts:113-116` «rol: revisor» | plugin | skill | skill del proyecto con bloque `rol: revisor` y `patrones_archivo`, sin clave en la configuración (D5a); la selección, F1-15 |
-| 14 | Guardas del arnés en `.claude/settings.json` (`deny git checkout`, `ask --ronda-extra`, hook Stop si HEAD está desligado) | `TW .claude/settings.json:3-9` «git checkout»; `TW .claude/settings.json:11-13` «--ronda-extra»; `TW .claude/settings.json:17-23` «symbolic-ref -q HEAD» | a mano | fuera | se quedan en `.claude/`: no son extensión de TaskCode y la 2.0 no depende de ellas; lo que protegen lo hacen comandos y puertas (F1-3, F1-5) |
+| # | Práctica | Evidencia (repo fichero:línea «texto») | En la 0.16 | Destino | Clasificación | En la PoC | Punto de extensión en la 2.0 |
+|---|---|---|---|---|---|---|---|
+| 1 | Estado propio `05-archivadas`: carpeta, etiqueta `descartada` y un `estado` que no lo dice | `TW tareas/05-archivadas/TASK-171/tarea.md:6` «descartada»; `TW tareas/05-archivadas/TASK-171/tarea.md:9` «estado: planificada»; `ODT tareas/05-archivadas/TASK-068/tarea.md:6` «descartada»; `ODT tareas/05-archivadas/TASK-068/tarea.md:9` «estado: planificada» | a mano | nativo | parche | comprobado | estado `descartada` con motivo y `pausa` (F3-7); sin clave de estados propios (D1a) |
+| 2 | Tablas de §1 y Mermaid de §4 de `hitos.md`, y `ROADMAP.md`, regenerados en cada transición | `TW .taskcode/config.yml:29-32` «comando_sincronizacion»; `TW scripts/grafo-tareas.py:4-5` «regenera el Mermaid»; `TW docs/planificacion/hitos.md:26` «Complejidad»; `TW docs/planificacion/hitos.md:665` «grafo-tareas:inicio» | clave | clave | propio | comprobado | las tablas y el Mermaid, a `tareas/hitos.md` con un derivado `genera` (`derivados[].tipo`, `derivados[].escribe`); en `docs/` solo el texto a mano; el `ROADMAP.md`, nativo (F1-1 dec. 3) |
+| 3 | Comprobaciones que cruzan código y estado, dentro de la verificación | `TW scripts/verificar.bat:124-127` «grafo-tareas.py --comprobar»; `ODT scripts/verificar.bat:51` «comprobar-modulos.py --comprobar»; `ODT scripts/verificar.bat:53` «comprobar-fallos.py --probar»; `ODT scripts/comprobar-modulos.py:88` «lector-tareas.py»; `ODT scripts/comprobar-fallos.py:302-307` «tareas_vivas» | a mano | clave | propio | comprobado | derivados `comprueba` (`derivados[].tipo`) en cada transición, fuera del perfil cacheado; en el PR, el check obligatorio `estado` los ejecuta y bloquea (D3a) |
+| 4 | Lector propio de frontmatter que recorre `tareas/0?-*/TASK-*/tarea.md` | `TW scripts/grafo-tareas.py:99` «raiz_tareas.glob»; `ODT scripts/lector-tareas.py:39` «raiz_tareas.glob» | a mano | nativo | parche | proyectado | `taskctl tareas --json` (frontmatter, criterios y texto) como lectura estable; la forma de las carpetas no es contrato |
+| 5 | Cuatro perfiles con seis claves de comando y tiempo | `TW .taskcode/config.yml:9` «comando_verificacion:»; `TW .taskcode/config.yml:16` «comando_verificacion_freno»; `TW .taskcode/config.yml:18` «comando_verificacion_selectiva»; `TW .taskcode/config.yml:23` «comando_verificacion_pesada»; `TW docs/verificacion/flujo-de-verificacion.md:9-12` «comando_verificacion_freno» | clave | clave | propio | comprobado | `verificacion.comando` llamado con `--perfil` (F1-7 dec. 1), con su `verificacion.shell` y `verificacion.perfiles.<p>.runner` |
+| 6 | Tiempo máximo por perfil y presupuesto del freno, que no bloquea | `TW .taskcode/config.yml:10` «timeout_verificacion: 4800»; `TW .taskcode/config.yml:12-13` «no bloquea el turno»; `TW .taskcode/config.yml:20` «timeout_verificacion_selectiva: 1800»; `TW docs/verificacion/flujo-de-verificacion.md:296` «deja parar el turno»; `ODT .taskcode/config.yml:10` «timeout_verificacion: 1200» | clave | clave | propio | comprobado | `verificacion.perfiles.<p>.timeout`: en `rapida`, presupuesto que no bloquea; en las demás, rojo |
+| 7 | Caché propia por paso y «completa del mismo árbol» | `TW scripts/cache-verificacion.py:1` «Caché por paso de verificar.bat»; `TW scripts/verificar.bat:91-93` «completa-leer»; `TW docs/verificacion/flujo-de-verificacion.md:164-165` «mismo árbol exacto»; `TW docs/verificacion/flujo-de-verificacion.md:4` «Repetirla en cada turno» | a mano | clave | parche | dentro del comando | `verificacion.huella_arbol` y `verificacion.huella_local` (F1-7 dec. 2): caché por árbol; lo más fino (por paso) no lo da la 2.0 y se pierde al retirarla |
+| 8 | Canales de texto: el x/N de `TASKCTL_PROGRESO` y la línea «# tests N» | `TW scripts/verificar.bat:51-52` «TASKCTL_PROGRESO»; `TW .taskcode/config.yml:7-8` «# tests N»; `PL src/fs/progreso.ts:33` «TASKCTL_PROGRESO» | plugin | nativo | parche | proyectado | informe JSON de la verificación (F5-4), con pasos y recuento; sin canal de texto |
+| 9 | `RAMA_BASE` como entrada del comando | `TW scripts/clasificar-verificacion.py:14-15` «RAMA_BASE (la pasa TaskCode 0.13)»; `TW scripts/clasificar-verificacion.py:54-59` «os.environ.get("RAMA_BASE")» | plugin | nativo | propio | comprobado | `RAMA_BASE` solo en `rapida` e `impacto` (F1-7 dec. 1); la completa de ToolsaWeb no depende de ella (PoC E) |
+| 10 | Pasos que dependen de la red | `ODT scripts/verificar.bat:68-69` «cargo audit --no-yanked --deny unsound»; `ODT scripts/verificar.bat:70` «npm --prefix web audit» | a mano | clave | propio | comprobado | `verificacion.programada.comando` con `verificacion.programada.cron`: job programado, fuera del check cacheado (F1-7 dec. 2) |
+| 11 | Cerrojo propio con códigos 3 y 4 | `TW scripts/cerrojo-verificacion.py:6-7` «3 si hay otra»; `TW scripts/cerrojo-verificacion.py:19` «el freno de fin de turno de TaskCode»; `TW docs/verificacion/flujo-de-verificacion.md:182` «código 3»; `ODT scripts/verificar.bat:3-4` «lo lanzan el freno de fin de turno»; `ODT scripts/verificar.bat:16-17` «impide dos» | a mano | clave | parche | dentro del comando | hoy, dentro de `verificacion.comando`; las puertas de la 2.0 corren en CI, un job por runner con el directorio limpio (F1-7, F1-8 dec. 6); cualquier código distinto de 0 es rojo (D6a) |
+| 12 | Ruta de Windows en YAML (`"\v"`) | `ODT .taskcode/config.yml:5-6` «seria un escape»; `TW .taskcode/config.yml:9` «"scripts\verificar.bat"» | a mano | clave | parche | convención | `verificacion.comando` y `derivados[].comando` entre comillas simples o con `/`; el validador rechaza un carácter de control (PoC B) |
+| 13 | Revisoras propias: ningún proyecto las declara; el catálogo sale del plugin | `PL src/core/revisores.ts:69-74` «CLAUDE_PLUGIN_ROOT»; `PL src/core/revisores.ts:113-116` «rol: revisor» | plugin | skill | propio | proyectado | skill del proyecto con bloque `rol: revisor` y `patrones_archivo`, sin clave en la configuración (D5a); la selección, F1-15 |
+| 14 | Guardas del arnés en `.claude/settings.json` (`deny git checkout`, `ask --ronda-extra`, hook Stop si HEAD está desligado) | `TW .claude/settings.json:3-9` «git checkout»; `TW .claude/settings.json:11-13` «--ronda-extra»; `TW .claude/settings.json:17-23` «symbolic-ref -q HEAD» | a mano | fuera | parche | fuera | no son extensión de TaskCode y la 2.0 no depende de ellas: lo que protegen lo hacen comandos y puertas (F1-3, F1-5); se retiran al migrar |
 <!-- inventario:fin -->
 
-Lectura: de 14 prácticas, la 0.16 cubre 3 con una clave y 3 con algo que
-pone el plugin; **8 se hacen a mano**. En la 2.0, 13 tienen un destino
-declarado (8 claves del esquema, 4 nativas, 1 skill) y la 14, las guardas
-del arnés, queda fuera de alcance por decisión (ver «Fuera de alcance»).
+Lectura (PoC A y A2): de 14 prácticas, la 0.16 cubre 3 con una clave y 3 con
+algo que pone el plugin; **8 se hacen a mano**. De esas 8, la PoC respalda
+**3** con un mecanismo declarado y comprobado (1, 3 y 10), **1** queda
+proyectada (4, `tareas --json`), **2** siguen dentro del comando del proyecto
+(7 y 11: la 2.0 no les da clave), **1** es convención (12) y **1** queda fuera
+(14). Por clasificación: **7 parches** que se retiran al migrar (1, 4, 7, 8,
+11, 12 y 14) y **7 propias** que se quedan (2, 3, 5, 6, 9, 10 y 13); las 7 y
+11, que hoy viven dentro del comando, son parches: al retirarlas, la caché
+por árbol de F1-7 cubre la «completa del mismo árbol», pero **la caché por
+paso se pierde** (la 2.0 no la da), y el cerrojo deja de hacer falta porque
+las puertas corren en CI, un job por runner con el directorio limpio.
+
+### Desglose de los ficheros: qué se retira y qué se queda
+
+Para los ficheros de las prácticas, la PoC A2 cuenta las líneas de cada
+clase en el commit fijo. Ficheros enteros o funciones con su rango; en los
+`verificar.bat`, bloques declarados (que la PoC comprueba que contienen el
+texto citado) y, en el de ToolsaWeb, además patrones por línea (las llamadas a
+`cache-verificacion.py` salvo `correr`, `clasificar-verificacion.py`,
+`cerrojo-verificacion.py`, `recuento-tests.py`, `resumen-verificacion.py`, el
+`verificar.lck`, `call :grupo`, `if errorlevel 10`, las etiquetas `:tras_*`,
+`TASKCTL_PROGRESO`, `TW_PLAN`, `TW_G_`, `TW_CACHE`, `TW_AVISO_PESADA`, el
+recuento y los comentarios que nombran `taskctl`, el freno, `TaskCode` o
+`comando_verificacion`). Lo que no se nombra es propio. Las líneas
+«`cache-verificacion.py correr <paso>`» son propias: se quedan como
+`python -I <paso>` sin el envoltorio de la caché.
+
+```text
+practicas: parche=7 (1 4 7 8 11 12 14) ; propio=7 (2 3 5 6 9 10 13)
+las 8 practicas a mano en la 0.16, segun la PoC: comprobado=3 (1 3 10) ; convención=1 (12) ; dentro del comando=2 (7 11) ; fuera=1 (14) ; proyectado=1 (4)
+-- ficheros enteros o por funciones
+  TW scripts/cache-verificacion.py: parche 1063 de 1063 lineas (1-1063) ; cache por paso y completa del mismo arbol, para no repetir lo que la 0.16 relanza en cada turno y ronda (F1-7 dec. 2)
+  TW scripts/cerrojo-verificacion.py: parche 226 de 226 lineas (1-226) ; una verificacion a la vez: la 0.16 la lanza desde el freno, finish y el guardian (F1-7, F1-8 dec. 6)
+  TW scripts/clasificar-verificacion.py: parche 1000 de 1000 lineas (1-1000) ; selectiva para las rondas de review de la 0.16; generica solo si F5-3 da el impacto (pregunta abierta)
+  TW scripts/recuento-tests.py: parche 301 de 301 lineas (1-301) ; linea «# tests N» para el registro de puertas de la 0.16 (F5-4)
+  TW scripts/resumen-verificacion.py: parche 461 de 461 lineas (1-461) ; x/N de TASKCTL_PROGRESO, evidencia de la ronda y «# tests N» (F5-4); incluye el resumen del panel de tareas
+  TW .claude/settings.json: parche 31 de 31 lineas (1-31) ; guardas del arnes frente a fallos de la 0.16 (F1-3, F1-5)
+  TW scripts/grafo-tareas.py: parche 38 de 542 lineas (quitar_comentario 62-68, leer_tarea 71-94, cargar 97-103) ; lector propio (tareas --json)
+  TW scripts/grafo-tareas.py: parche 48 de 542 lineas (roadmap 277-324) ; tareas/ROADMAP.md lo escribe taskctl (F1-1 dec. 3); orden_de_trabajo se queda: lo usa el comprueba
+  TW scripts/verificar-rapida.py: propio 146 de 146 lineas (1-146) ; que corre la rapida del proyecto
+  ODT scripts/lector-tareas.py: parche 44 de 44 lineas (1-44) ; lector propio (tareas --json)
+  ODT scripts/comprobar-modulos.py: parche 17 de 458 lineas (_cargar_grafo 87-92, tareas_vivas 95-105) ; lectura propia del estado (tareas --json)
+  ODT scripts/comprobar-fallos.py: parche 15 de 470 lineas (_cargar_grafo 51-56, tareas_vivas 302-310) ; lectura propia del estado (tareas --json)
+  ODT scripts/comprobar-fallos.py: propio el resto, 455 lineas
+  ODT scripts/comprobar-modulos.py: propio el resto, 441 lineas
+  TW scripts/grafo-tareas.py: propio el resto, 456 lineas
+-- OpenDockTrading scripts/verificar.bat
+  parche 15 de 92 lineas (3-5, 16-18, 25, 35-37, 44-47, 59) ; propio 77
+-- ToolsaWeb scripts/verificar.bat (bloques declarados y patrones por linea)
+  parche 192 de 683 lineas: 34-35, 37-43, 47-52, 56, 61-67, 70, 76, 78, 80-107, 111-112, 254, 256, 260, 262, 264, 276-277, 283-285, 288, 293-294, 296-297, 299-301, 306-308, 310-311, 316-319, 322, 324-325, 328-329, 332-333, 367-368, 371-372, 399, 402, 405, 408, 475, 508-509, 515-517, 519-522, 525-528, 531-533, 535-537, 543-554, 556-557, 560, 562-563, 569, 571-572, 583, 586-587, 590, 592-593, 596, 600-601, 604, 615, 619, 621-622, 625, 630-631, 634-636, 639, 642-649, 651-653, 658-668, 670-675, 678-682
+  propio 491 lineas ; de ellas 53 se quedan sin el envoltorio «cache-verificacion.py correr»
+  ToolsaWeb: lineas que se retiran=3360 ; lineas que se quedan=1093 (de los ficheros desglosados)
+  OpenDockTrading: lineas que se retiran=91 ; lineas que se quedan=973 (de los ficheros desglosados)
+RESULTADO A2 parches=7 ; ficheros enteros que se retiran=7 ; lineas TW 3360/1093 ; ODT 91/973
+```
+
+Lectura: en ToolsaWeb se retiran **3360** líneas (7 ficheros enteros, 2
+funciones de `grafo-tareas.py` y 192 líneas de `verificar.bat`) y se quedan
+**1093** de los ficheros desglosados; en OpenDockTrading, **91** frente a
+**973**. Del `verificar.bat` de ToolsaWeb son parche **192** de 683 líneas:
+caché, cerrojo, clasificación, progreso, recuento y la integración con el
+freno y `taskctl`; lo propio (los pasos del proyecto, `cargo test`, Liquibase,
+wasm, los pesados) se queda. Dos matices: `clasificar-verificacion.py` cuenta
+como parche porque existe para las rondas de `review` de la 0.16, pero F1-7
+(dec. 1) deja hoy el perfil `impacto` en el comando del proyecto: si F5-3 no
+da el impacto de forma genérica, ToolsaWeb lo necesitará y esa parte pasa a
+propia (pregunta abierta); y `resumen-verificacion.py` también alimenta el
+panel de tareas de ToolsaWeb (TASK-089): si el panel se queda, esa parte es
+propia (no verificado).
 
 ## Alternativas
 
@@ -82,11 +159,11 @@ del arnés, queda fuera de alcance por decisión (ver «Fuera de alcance»).
 |---|---|---|
 | **D1a** | Sin clave de estados propios: `descartada` y `pausa` nativos (F3-7) | **Elegida**: cubren los 11 de 11 casos medidos de `05-archivadas` (PoC A) |
 | D1b | Clave `estados` con estados del tablero configurables (lo que sugiere RF-71) | Descartada: una máquina de estados configurable para 11 casos que ya cubre un estado nativo; el validador la rechaza como clave desconocida (PoC B) |
-| **D2a** | Los derivados solo escriben bajo `tareas/` (rama de estado) | **Elegida**: el árbol de `develop` no cambia en 7 transiciones (PoC C, RR-11) |
+| **D2a** | Los derivados solo escriben bajo `tareas/` (rama de estado); lo generado que hoy vive en `docs/` (tablas y Mermaid de `hitos.md` de ToolsaWeb) pasa a `tareas/` | **Elegida**: el árbol de `develop` no cambia en 10 transiciones (PoC C, RR-11) y el `hitos.md` de ToolsaWeb pasa de rojo permanente a 0 errores (PoC C2) |
 | D2b | Los derivados escriben también en ramas de código (`docs/planificacion/hitos.md`) | Descartada: cada transición cambiaría el árbol verificado (F1-1, RR-11) y la escritura saltaría la protección de `develop` |
-| **D3a** | Comprobaciones que cruzan código y estado como derivados `comprueba` en cada transición, más un check `estado` no cacheado en el PR que lee `taskcode/tareas` | **Elegida**: un resultado que depende del estado no puede vivir en un perfil cacheado por árbol (F1-7 dec. 2) |
+| **D3a** | Comprobaciones que cruzan código y estado como derivados `comprueba` en cada transición (avisan) y como check **obligatorio** `estado` en el PR, que los ejecuta, no se cachea y bloquea | **Elegida**: un resultado que depende del estado no puede vivir en un perfil cacheado por árbol (F1-7 dec. 2); el bloqueo pasa de la verificación al check `estado` (PoC C, T6) |
 | D3b | Dejarlas dentro del comando de verificación | Descartada: el check `verificacion` se cachea por árbol y el estado no está en el árbol: un acierto escondería un fallo nuevo |
-| **D4a** | Tres bloques: `verificacion`, `derivados`, `confianza_incluye` | **Elegida**: 3 claves de primer nivel; con las 2 de F1-10, 5 de 12 (PoC B) |
+| **D4a** | Tres bloques: `verificacion`, `derivados`, `confianza_incluye` | **Elegida**: 3 claves de primer nivel (15 hojas); con las 2 de F1-10, 5 de 12 (PoC B) |
 | D4b | Claves planas al estilo 0.16 (`comando_verificacion_*`, `timeout_*`, `comando_sincronizacion`…) | Descartada: el catálogo de la 0.16 ya tiene 12 claves solo de verificación y sincronización; no caben en el tope de 12 |
 | **D5a** | Revisoras propias como skill del proyecto con bloque `rol: revisor` y `patrones_archivo`, sin clave | **Elegida**: es el formato que ya lee el plugin (`revisores.ts`); F1-15 decide la selección |
 | D5b | Clave `revisoras` en la configuración | Descartada: duplica lo que dice la skill y gasta una de las 12 claves para algo que hoy usan 0 proyectos (PoC A) |
@@ -101,31 +178,47 @@ Guion `poc-f1-11.py` y su salida completa `poc-f1-11.log`, en la carpeta
 `poc/` de TASK-012 (sin enlace: la carpeta de la tarea cambia de estado al
 cerrarse). Ejecutado con Python 3.12.10 (`python -I -B`), Git 2.56 y PyYAML
 6.0.3, con la carpeta de trabajo en
-`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-012\poc\` (borrada al
+`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-012-c1\` (borrada al
 terminar). **Solo lectura**: ToolsaWeb y OpenDockTrading se leen con
 `git show`, `git ls-tree`, `git grep` y `git archive` de los commits fijos;
-lo que se ejecuta de ellos (D y E) es una **copia** en la carpeta de trabajo,
-nunca dentro de esos repos. El plugin 0.16.1 solo se lee. No hay llamadas a
-GitHub. Dos ejecuciones seguidas dan el mismo log.
+lo que se ejecuta de ellos (C2, D y E) es una **copia** en la carpeta de
+trabajo, nunca dentro de esos repos. El plugin 0.16.1 solo se lee. No hay
+llamadas a GitHub. Dos ejecuciones seguidas dan el mismo log.
 
 ```bash
 cd sources/Personal/TaskCodeTwo
 python -I -B tareas/<estado>/TASK-012/poc/poc-f1-11.py \
-  "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-012/poc" \
+  "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-012-c1/poc" \
   docs/adr/F1-11-extensiones-declaradas.md .. \
   ~/.claude/plugins/cache/taskcode-marketplace/taskcode-plugin/0.16.1 > poc-f1-11.log
 ```
 
-El guion **lee tres tablas de este fichero** entre marcadores: `inventario`
-(A comprueba sus citas), `esquema` (tiene que coincidir clave a clave con el
-validador del guion) y `ventaja` (cada cifra tiene que ser la medida). Si
-alguna no cuadra, sale con 1.
+El guion **lee cuatro bloques de este fichero** entre marcadores: `inventario`
+(A comprueba sus citas y A2 su clasificación), `esquema` (tiene que coincidir
+clave a clave con el validador del guion), `ventaja` (cada cifra tiene que ser
+la medida, con su tipo medido o proyectado, y toda cifra medida tiene que
+estar en la tabla) y `resumen` (cada número va en negrita y tiene que ser un
+valor de la tabla). Si algo no cuadra, sale con 1.
 
 ### A — Cobertura del inventario
 
 ```text
 claves en la tabla esquema del ADR: 17 ; en el validador: 17 ; iguales: True
-citas comprobadas: 43/43
+   1 Estado propio `05-archivadas`: carpeta, etiqueta `descarta citas=4 0.16=a mano  2.0=nativo  OK
+   2 Tablas de §1 y Mermaid de §4 de `hitos.md`, y `ROADMAP.md` citas=4 0.16=clave   2.0=clave   OK
+   3 Comprobaciones que cruzan código y estado, dentro de la ve citas=5 0.16=a mano  2.0=clave   OK
+   4 Lector propio de frontmatter que recorre `tareas/0?-*/TASK citas=2 0.16=a mano  2.0=nativo  OK
+   5 Cuatro perfiles con seis claves de comando y tiempo        citas=5 0.16=clave   2.0=clave   OK
+   6 Tiempo máximo por perfil y presupuesto del freno, que no b citas=5 0.16=clave   2.0=clave   OK
+   7 Caché propia por paso y «completa del mismo árbol»         citas=4 0.16=a mano  2.0=clave   OK
+   8 Canales de texto: el x/N de `TASKCTL_PROGRESO` y la línea  citas=3 0.16=plugin  2.0=nativo  OK
+   9 `RAMA_BASE` como entrada del comando                       citas=2 0.16=plugin  2.0=nativo  OK
+  10 Pasos que dependen de la red                               citas=2 0.16=a mano  2.0=clave   OK
+  11 Cerrojo propio con códigos 3 y 4                           citas=5 0.16=a mano  2.0=clave   OK
+  12 Ruta de Windows en YAML (`"\v"`)                           citas=2 0.16=a mano  2.0=clave   OK
+  13 Revisoras propias: ningún proyecto las declara; el catálog citas=2 0.16=plugin  2.0=skill   OK
+  14 Guardas del arnés en `.claude/settings.json` (`deny git ch citas=3 0.16=a mano  2.0=fuera   OK
+citas comprobadas: 48/48
 en la 0.16: por clave=3 ; del plugin=3 ; a mano=8 (de 14)
 en la 2.0: clave=8 ; nativo=4 ; skill=1 ; fuera de alcance=1 ; sin destino=0
 -- estados propios (practica 1): 05-archivadas
@@ -135,13 +228,14 @@ en la 2.0: clave=8 ; nativo=4 ; skill=1 ; fuera de alcance=1 ; sin destino=0
 -- revisoras propias (practica 13): skills con `rol: revisor` en los pilotos
   ToolsaWeb: 0
   OpenDockTrading: 0
-RESULTADO A practicas=14 ; con destino=13 ; citas=43/43
+RESULTADO A practicas=14 ; con destino=13 ; citas=48/48
 ```
 
-Lectura: las 43 citas existen y dicen lo que el inventario afirma. Las 11
+Lectura: las 48 citas existen y dicen lo que el inventario afirma. Las 11
 tareas archivadas llevan la etiqueta `descartada` y **ninguna** tiene un
 `estado` que lo diga (10 `planificada`, 1 `en-diseno`): el estado nativo
-`descartada` con motivo las cubre todas y quita la incoherencia.
+`descartada` con motivo las cubre todas y quita la incoherencia. La
+clasificación y el desglose (A2) están arriba, en «Contexto».
 
 ### B — Esquema, hostiles, confianza y presupuesto
 
@@ -152,15 +246,17 @@ validador de la decisión 1:
 ```text
   ToolsaWeb traducida -> valida ; claves de primer nivel: ['confianza_incluye', 'derivados', 'verificacion']
   OpenDockTrading traducida -> valida ; claves de primer nivel: ['confianza_incluye', 'derivados', 'verificacion']
+-- lo que la traduccion nombra existe en el commit fijo
   TW verificacion.huella_local: scripts/huella-local.py NO EXISTE en el commit fijo (a crear)
+-- contrato --perfil frente al verificar.bat de ToolsaWeb (lectura, no se ejecuta)
   verificar.bat:68-79 acepta ['--selectiva', '--pesada', '--rapida'] ; otro argumento -> exit /b 2: True ; «--perfil completa» lo acepta: False
 -- hostiles (falla cerrado)
   clave desconocida (la 0.16 en plano)                               -> rechazada: clave desconocida comando_verificacion_freno
   clave desconocida dentro de verificacion                           -> rechazada: verificacion.presupuesto_freno_s: clave desconocida
   ruta de Windows con \v entre comillas dobles (linea 9 de ToolsaWeb) -> rechazada: verificacion.comando: caracter de control '\x0b' en la orden (¿\v, \t o \n dentro de comillas dobles?): escrib
-  derivado que escribe fuera de tareas/ (hitos.md de ToolsaWeb)      -> rechazada: derivados[0].escribe: 'docs/planificacion/hitos.md' fuera de tareas/ (D2a: solo la rama de estado)
-  derivado que escribe el ROADMAP nativo                             -> rechazada: derivados[0].escribe: 'tareas/ROADMAP.md' lo escribe taskctl (roadmap o carpeta de tarea)
-  derivado que escribe una carpeta de tarea                          -> rechazada: derivados[0].escribe: 'tareas/TASK-001/tarea.md' lo escribe taskctl (roadmap o carpeta de tarea)
+  derivado que escribe fuera de tareas/ (hitos.md de ToolsaWeb)      -> rechazada: derivados[0].escribe: 'docs/planificacion/hitos.md': fuera de tareas/ (D2a: solo la rama de estado)
+  derivado que escribe el ROADMAP nativo                             -> rechazada: derivados[0].escribe: 'tareas/ROADMAP.md': lo escribe taskctl (roadmap o carpeta de tarea)
+  derivado que escribe una carpeta de tarea                          -> rechazada: derivados[0].escribe: 'tareas/TASK-001/tarea.md': lo escribe taskctl (roadmap o carpeta de tarea)
   derivado comprueba que escribe                                     -> rechazada: derivados[0].escribe: un derivado comprueba no escribe nada
   derivado con fase inventada                                        -> rechazada: derivados[0].fases: subconjunto no vacio de ['new', 'import', 'plan', 'approve', 'start', 'review', 'finish', 
   estado propio declarado (D1a: sin clave)                           -> rechazada: clave desconocida estados
@@ -168,6 +264,19 @@ validador de la decisión 1:
   rapida con runner (corre en local)                                 -> rechazada: verificacion.perfiles.rapida.runner: rapida corre en local (F1-7 dec. 4); sin runner
   confianza_incluye que hashea todo el arbol                         -> rechazada: confianza_incluye: globs sin negacion, sin rutas absolutas ni .., y no todo el arbol
   timeout como texto                                                 -> rechazada: verificacion.perfiles.completa.timeout: entero de 1 a 21600 segundos
+  runner con el vocabulario viejo (github-windows)                   -> rechazada: verificacion.perfiles.completa.runner: runner 'github-windows': {github: windows|linux} o [self-hosted, ...] (
+  escribe 'tareas/./ROADMAP.md' (revision 1, IMP-2)                  -> rechazada: derivados[0].escribe: 'tareas/./ROADMAP.md': lo escribe taskctl (roadmap o carpeta de tarea)
+  escribe 'tareas/roadmap.md' (revision 1, IMP-2)                    -> rechazada: derivados[0].escribe: 'tareas/roadmap.md': lo escribe taskctl (roadmap o carpeta de tarea)
+  escribe 'tareas/./TASK-001/tarea.md' (revision 1, IMP-2)           -> rechazada: derivados[0].escribe: 'tareas/./TASK-001/tarea.md': lo escribe taskctl (roadmap o carpeta de tarea)
+  escribe 'tareas//TASK-001/tarea.md' (revision 1, IMP-2)            -> rechazada: derivados[0].escribe: 'tareas//TASK-001/tarea.md': lo escribe taskctl (roadmap o carpeta de tarea)
+  escribe 'tareas/task-001/tarea.md' (revision 1, IMP-2)             -> rechazada: derivados[0].escribe: 'tareas/task-001/tarea.md': lo escribe taskctl (roadmap o carpeta de tarea)
+  escribe 'tareas/' (revision 1, IMP-2)                              -> rechazada: derivados[0].escribe: 'tareas/': tareas/ entera (toda la rama de estado)
+  escribe 'tareas/.git' (revision 1, IMP-2)                          -> rechazada: derivados[0].escribe: 'tareas/.git': toca .git
+  escribe 'tareas/x:ads' (revision 1, IMP-2)                         -> rechazada: derivados[0].escribe: 'tareas/x:ads': ruta absoluta o con un caracter no admitido (\ : * ? " < > | o de contro
+-- rutas de escribe que se aceptan, ya normalizadas
+  'tareas/hitos.md'              -> 'tareas/hitos.md'
+  'tareas/informes/grafo.md'     -> 'tareas/informes/grafo.md'
+  'tareas/./hitos.md'            -> 'tareas/hitos.md'
   lector YAML estandar sobre la config real de ToolsaWeb: comando_verificacion = 'scripts\x0berificar.bat'
   y sobre la de OpenDockTrading (sin comillas): comando_verificacion = 'scripts\\verificar.bat'
 -- confianza (F1-8 dec. 5): una orden sin confianza no se ejecuta
@@ -175,81 +284,148 @@ validador de la decisión 1:
 -- presupuesto de claves y ventaja frente a la 0.16
   claves del catalogo de la 0.16 (src/core/config.ts, tabla de cabecera): 32
   de ellas, de verificacion y sincronizacion (las sustituyen verificacion y derivados): 12
-  ToolsaWeb: claves en su config 0.16 = 12 ; de verificacion y sincronizacion = 10 (comando_sincronizacion, comando_verificacion, comando_verificacion_freno, comando_verificacion_pesada, comando_verificacion_selectiva, presupuesto_freno_s, rutas_sincronizacion, timeout_verificacion, timeout_verificacion_selectiva, verificacion_pesada_en) ; en el esquema nuevo = 2 (derivados, verificacion) ; resto, de F2-7: ['modo_flujo', 'tope_rondas']
-  OpenDockTrading: claves en su config 0.16 = 3 ; de verificacion y sincronizacion = 2 (comando_verificacion, timeout_verificacion) ; en el esquema nuevo = 2 (derivados, verificacion) ; resto, de F2-7: ['modo_flujo']
-  presupuesto: F1-11 3 + F1-10 2 = 5 de 12 (F1-8 no anade clave propia: sus globs son confianza_incluye)
-RESULTADO B traducidas validas=2/2 ; hostiles rechazadas=13/13 ; claves=5/12
+  ToolsaWeb: claves en su config 0.16 = 12 ; de verificacion y sincronizacion = 10 (comando_sincronizacion, comando_verificacion, comando_verificacion_freno, comando_verificacion_pesada, comando_verificacion_selectiva, presupuesto_freno_s, rutas_sincronizacion, timeout_verificacion, timeout_verificacion_selectiva, verificacion_pesada_en) ; bloques en el esquema nuevo = 2 (derivados, verificacion) ; resto, de F2-7: ['modo_flujo', 'tope_rondas']
+  ToolsaWeb: valores a rellenar 0.16 = 10 ; 2.0 = 16 (verificacion.comando, verificacion.shell, verificacion.perfiles.rapida.timeout, verificacion.perfiles.impacto.timeout, verificacion.perfiles.completa.timeout, verificacion.perfiles.completa.runner, verificacion.perfiles.pesada.timeout, verificacion.perfiles.pesada.runner, verificacion.huella_arbol, verificacion.huella_local, derivados[0].comando, derivados[0].tipo, derivados[0].escribe, derivados[1].comando, derivados[1].tipo, confianza_incluye)
+  OpenDockTrading: claves en su config 0.16 = 3 ; de verificacion y sincronizacion = 2 (comando_verificacion, timeout_verificacion) ; bloques en el esquema nuevo = 2 (derivados, verificacion) ; resto, de F2-7: ['modo_flujo']
+  OpenDockTrading: valores a rellenar 0.16 = 2 ; 2.0 = 13 (verificacion.comando, verificacion.shell, verificacion.perfiles.completa.timeout, verificacion.perfiles.completa.runner.github, verificacion.huella_arbol, verificacion.programada.comando, verificacion.programada.cron, verificacion.programada.runner.github, derivados[0].comando, derivados[0].tipo, derivados[1].comando, derivados[1].tipo, confianza_incluye)
+  esquema de F1-11: 3 claves de primer nivel y 15 hojas
+  presupuesto (el tope de 12 de F2-7 cuenta claves de primer nivel): F1-11 3 + F1-10 2 = 5 de 12 (F1-8 no anade clave propia: sus globs son confianza_incluye)
+RESULTADO B traducidas validas=2/2 ; hostiles rechazadas=22/22 ; claves=5/12
 ```
 
-Lectura: las dos traducciones valen y las 13 hostiles se rechazan con un
-mensaje que dice qué arreglar. La línea 9 de ToolsaWeb, leída con un lector
-YAML estándar, da `scripts<TAB-vertical>erificar.bat`; el validador lo para.
-Lo que la traducción nombra existe en el commit fijo salvo una orden:
-ToolsaWeb no tiene hoy una orden de huella local (su lógica está dentro de
-`cache-verificacion.py`), y su `verificar.bat` no acepta `--perfil`.
-OpenDockTrading **no baja de claves** (2 → 2): gana un derivado `comprueba`
-que saca `comprobar-modulos.py` del perfil cacheado (D3a), y el job
-programado para sus auditorías de red va dentro de `verificacion`.
-
-### C — Derivados en la rama de estado
+Lectura: las dos traducciones valen y las 22 hostiles se rechazan con un
+mensaje que dice qué arreglar, también las 8 rutas de `escribe` de la
+revisión 1 (con `.`, mayúsculas, separadores repetidos, `tareas/` entera,
+`.git` y `:`), que el validador **normaliza antes de comparar**. La línea 9 de
+ToolsaWeb, leída con un lector YAML estándar, da
+`scripts<TAB-vertical>erificar.bat`; el validador lo para. Lo que la
+traducción nombra existe en el commit fijo salvo una orden: ToolsaWeb no tiene
+hoy una orden de huella local (su lógica está en `cache-verificacion.py`,
+que se retira), y su `verificar.bat` no acepta `--perfil`.
+
+**Las dos medidas de claves.** Por **bloques**, ToolsaWeb pasa de 10 claves
+planas a 2 bloques y OpenDockTrading de 2 a 2. Por **valores a rellenar**
+(cada escalar o lista de escalares), ToolsaWeb pasa de **10 a 16** y
+OpenDockTrading de **2 a 13**: el esquema nuevo es más ordenado, **no más
+corto**. Lo que se añade son capacidades que la 0.16 no tiene: `shell`, runner
+por perfil, huella de caché, job programado, los `comprueba` y
+`confianza_incluye`. El tope de 12 de F2-7 se lee como **claves de primer
+nivel**; el esquema de F1-11 tiene 3 de primer nivel y 15 hojas.
+
+### C — Derivados en la rama de estado y check `estado`
 
 Repo temporal con `develop` y la rama huérfana `taskcode/tareas` montada en
 `tareas/` (Git 2.56, `worktree add --orphan`). Dos derivados declarados en
 `.taskcode/config.yml` (un `genera` estilo `grafo-tareas`, que escribe
 `tareas/grafo-hitos.md`, y un `comprueba` que mira que cada `TASK-NNN` citada
-en el código exista en el estado) y una función `transicion` que hace lo
-que haría `taskctl <fase>`: escribe la tarea, ejecuta los derivados de esa
-fase **solo si están confiados**, con `TASKCODE_TAREAS` y `TASKCODE_CODIGO`,
-descarta lo que escriban fuera de `escribe`, y commitea todo en un commit de
-la rama de estado.
+en el código exista en el estado) y una función `transicion` que hace lo que
+haría `taskctl <fase>`: escribe la tarea, ejecuta los derivados de esa fase
+**solo si están confiados**, con `TASKCODE_TAREAS` y `TASKCODE_CODIGO`,
+compara **por contenido** lo que cambió (en la rama de estado, todo fichero
+fuera de `escribe`, también el `tarea.md` de la transición; en el código,
+también lo ignorado), descarta lo que no le toca y commitea todo en un commit
+de la rama de estado. `check_estado` hace lo que haría el check del PR.
 
 ```text
 merge-base develop taskcode/tareas: NINGUNA (huerfana)
 -- transiciones
-  T1 plan TASK-001                               commit=['TASK-001/tarea.md', 'grafo-hitos.md'] ; grafo regenerado=True
-  T2 approve TASK-001                            commit=['TASK-001/tarea.md', 'grafo-hitos.md'] ; grafo regenerado=True
-  T3 plan TASK-003 con dependencia rota          commit=['TASK-003/tarea.md'] ; grafo regenerado=False
+  T1 plan TASK-001                                     commit=['TASK-001/tarea.md', 'grafo-hitos.md'] ; grafo regenerado=True
+  T2 approve TASK-001                                  commit=['TASK-001/tarea.md', 'grafo-hitos.md'] ; grafo regenerado=True
+  T3 plan TASK-003 con dependencia rota                commit=['TASK-003/tarea.md'] ; grafo regenerado=False
       aviso: derivados[0] fallo (codigo 1: dependencia inexistente: TASK-003 -> TASK-099); la transicion sigue; arreglalo y ejecuta `taskctl derivados`
-  T4 review TASK-001 con el generador cambiado   commit=['TASK-001/tarea.md'] ; grafo regenerado=False
+  T4 review TASK-001 con el generador cambiado         commit=['TASK-001/tarea.md'] ; grafo regenerado=False
       aviso: derivados[0] cambiado: no se ejecuta; revisa la orden y `taskctl confiar derivados[0]`
-  T5 finish TASK-001 (generador restaurado)      commit=['TASK-001/tarea.md', 'TASK-003/tarea.md', 'grafo-hitos.md'] ; grafo regenerado=True
--- hostiles en ejecucion
-  T6 finish TASK-002, codigo cita TASK-404       commit=['TASK-002/tarea.md', 'grafo-hitos.md'] ; grafo regenerado=True
+  T5 finish TASK-001 (generador restaurado)            commit=['TASK-001/tarea.md', 'TASK-003/tarea.md', 'grafo-hitos.md'] ; grafo regenerado=True
+-- un comprueba en rojo: la transicion avisa; el check `estado` del PR bloquea
+  T6 finish TASK-002, codigo cita TASK-404             commit=['TASK-002/tarea.md', 'grafo-hitos.md'] ; grafo regenerado=True
       aviso: derivados[1] fallo (codigo 1: citadas en el codigo y sin tarea: TASK-404); la transicion sigue; arreglalo y ejecuta `taskctl derivados`
-  T7 pausa TASK-003, derivado que escribe en codigo commit=['TASK-003/tarea.md'] ; grafo regenerado=False
-      aviso: derivados[0] escribio fuera de `escribe` (['src/intruso.txt']): descartado; la transicion sigue
-arbol de develop antes=f7b7e77bd30e despues=f7b7e77bd30e identico=True ; commits en develop 1->1 ; status limpio=True
-commits en taskcode/tareas: 8 (alta + 7 transiciones, uno por transicion)
+  check estado en el PR con ese codigo: ROJO (bloquea): derivados[1] codigo 1: citadas en el codigo y sin tarea: TASK-404
+  check estado sin la cita rota: verde
+-- hostiles en ejecucion (lo escrito se compara por contenido)
+  T7 pausa TASK-003, derivado que escribe en codigo    commit=['TASK-003/tarea.md'] ; grafo regenerado=False
+      aviso: derivados[0] escribio fuera de `escribe` (['codigo:src/intruso.txt']): descartado; la transicion sigue
+  T8 plan TASK-003, derivado que reescribe su tarea.md commit=['TASK-003/tarea.md'] ; grafo regenerado=False
+      aviso: derivados[0] escribio fuera de `escribe` (['TASK-003/tarea.md']): descartado; la transicion sigue
+      estado de TASK-003 tras T8: en-diseno
+  T9 start TASK-003, derivado que escribe en una ruta ignorada commit=['TASK-003/tarea.md'] ; grafo regenerado=False
+      aviso: derivados[0] escribio fuera de `escribe` (['codigo:.estado/x.txt']): descartado; la transicion sigue
+  T10 review TASK-003, escribe una ruta ignorada del estado commit=['TASK-003/tarea.md'] ; grafo regenerado=False
+      aviso: derivados[0] escribe una ruta ignorada en la rama de estado (['tareas/borrador.tmp']): no se commitearia; no se ejecuta
+arbol de develop antes=8f4565fa2d86 despues=8f4565fa2d86 identico=True ; commits en develop 1->1 ; status limpio, tambien lo ignorado=True
+commits en taskcode/tareas: 11 (alta + 10 transiciones, uno por transicion)
 grafo-hitos.md final: TASK-001 fuera (terminada)=True ; TASK-003 en H2=True
-RESULTADO C transiciones=7 ; como se espera=7/7 ; develop identico=True
+RESULTADO C transiciones=10 ; como se espera=10/10 ; develop identico=True
 ```
 
 Lectura: lo generado va en el mismo commit que la transición; un derivado que
 falla, que no está confiado o que escribe fuera de lo declarado no para la
-transición, deja su aviso con el comando que lo arregla y no ensucia nada; el
-árbol de `develop` es idéntico tras 7 transiciones (RR-11).
+transición, deja su aviso con el comando que lo arregla y no ensucia nada,
+**también si reescribe el `tarea.md` de la propia transición** (T8: el estado
+se queda en `en-diseno`) **o escribe en una ruta ignorada** (T9). Un
+`comprueba` en rojo **avisa en la transición y bloquea en el PR** (T6). El
+árbol de `develop` es idéntico tras 10 transiciones (RR-11).
+
+### C2 — `hitos.md` de ToolsaWeb, hoy y con la disposición nueva
+
+Copia del commit fijo de ToolsaWeb (`grafo-tareas.py`,
+`docs/planificacion/hitos.md`, `tareas/`). Hoy: el `--comprobar` real tras
+una transición sin poder regenerar `docs/` (D2a). Con la disposición nueva:
+en `docs/` solo el texto a mano; las tablas de §1 y el Mermaid de §4, en
+`tareas/hitos.md`, generados con las **funciones reales del script copiado**
+(`mermaid`, `pendientes`, `titulos_de_hitos`); el `comprueba` es la parte que
+cruza código y estado (`errores` y `errores_de_titulos`: los títulos
+«### Hn — …» de `docs/` frente a los hitos de las tareas); y la coherencia de
+lo generado, con `errores_de_tablas` y `tablas_de_hitos` del propio script.
+Las tareas se leen del `tareas --json` simulado (D).
+
+```text
+-- hoy (0.16, hitos.md en docs/): transicion simulada = finish de TASK-011 (lista para empezar)
+  --comprobar antes: codigo 0 ; [ok] grafo sin ciclos, dependencias existentes y hacia hitos no posteriores, hitos.md al d
+  --comprobar despues, sin poder regenerar docs/ (D2a): codigo 1 ; errores=2
+    [error] el Mermaid de docs\planificacion\hitos.md está desfasado: ejecuta --escribir
+    [error] tareas\ROADMAP.md está desfasado: ejecuta --escribir
+-- disposicion nueva: en docs/ solo el texto a mano; tablas y Mermaid en tareas/hitos.md (genera); titulos de docs/ frente al estado (comprueba)
+  docs/planificacion/hitos.md: 1779 lineas hoy ; 225 a mano ; 1554 generadas salen a la rama de estado
+  antes de la transicion: comprueba=0 errores ; tablas generadas incoherentes=0
+  tras el finish de TASK-011: comprueba=0 errores ; tablas incoherentes=0 ; tareas/hitos.md regenerado=True ; docs/ sin tocar=True
+RESULTADO C2 hoy=0->1 ; nueva=0->0 errores ; docs sin tocar=True
+```
+
+Lectura: con `hitos.md` en `docs/`, la primera transición deja el
+`--comprobar` real en rojo para siempre (código 1), porque nadie puede
+regenerar `docs/`. Con la disposición nueva, el mismo cambio da **0 errores**,
+`tareas/hitos.md` se regenera y `docs/` no se toca; salen de `docs/` **1554 de
+1779** líneas generadas. Lo que la PoC no ejecuta es el `main` adaptado del
+script (sus `--escribir` y `--comprobar` nuevos): es trabajo de ToolsaWeb al
+migrar (F1-12).
 
 ### D — Lectura del estado
 
-Los lectores propios **copiados** (`lector-tareas.py` de OpenDockTrading,
-`grafo-tareas.py` de ToolsaWeb) sobre las tareas reales de cada piloto, en la
-estructura de la 0.16 y en la carpeta fija de F1-1; frente a un
-`taskctl tareas --json` simulado (frontmatter leído con YAML estándar; las de
+Los tres consumidores reales del estado (`comprobar-modulos.py` y
+`comprobar-fallos.py` de OpenDockTrading, con su `lector-tareas.py`, y
+`grafo-tareas.py` de ToolsaWeb), **copiados**, sobre las tareas reales de cada
+piloto, en la estructura de la 0.16 y en la carpeta fija de F1-1; frente a un
+`taskctl tareas --json` simulado con todo el frontmatter, los `criterios` y
+el `texto` de `tarea.md` (lo que leen esos consumidores; las de
 `05-archivadas` migran con estado `descartada`).
 
 ```text
-  OpenDockTrading (scripts/lector-tareas.py): tareas.md=96 ; vivas hoy (0.16)=94 ; mismo glob en carpeta fija=0 ; desde tareas --json=94 ; mismo conjunto que hoy=True
-  ToolsaWeb (scripts/grafo-tareas.py): tareas.md=404 ; vivas hoy (0.16)=395 ; mismo glob en carpeta fija=0 ; desde tareas --json=395 ; mismo conjunto que hoy=True
-  lineas que deja de necesitar un lector propio: OpenDockTrading lector-tareas.py entero = 44 ; ToolsaWeb grafo-tareas.py {'quitar_comentario': 7, 'leer_tarea': 24, 'cargar': 7} = 38 ; el consumidor de tareas --json = 3
+  OpenDockTrading comprobar-modulos.py: tareas.md=96 ; tareas_vivas() hoy=94 ; en carpeta fija=0 ; desde tareas --json=94 ; mismo resultado (ids y valores)=True
+  OpenDockTrading comprobar-fallos.py: tareas.md=96 ; tareas_vivas() hoy=94 ; en carpeta fija=0 ; desde tareas --json=94 ; mismo resultado (ids y valores)=True
+  ToolsaWeb grafo-tareas.py: tareas.md=404 ; cargar() hoy=404 ; en carpeta fija=0 ; desde tareas --json=404 ; errores, Mermaid y roadmap iguales a hoy=True
+  lectura propia hoy: lector-tareas.py=44 ; comprobar-modulos.py {'_cargar_grafo': 6, 'tareas_vivas': 11} ; comprobar-fallos.py {'_cargar_grafo': 6, 'tareas_vivas': 9} ; grafo-tareas.py {'quitar_comentario': 7, 'leer_tarea': 24, 'cargar': 7} ; total=114
+  consumidores de tareas --json (modulos_desde_json, fallos_desde_json, grafo_desde_json): 11 lineas
   ToolsaWeb grafo-tareas.py que solapa con el roadmap nativo (F1-1 dec. 3, F3-4) {'mermaid': 25, 'orden_de_trabajo': 17, 'roadmap': 48} = 90 (no se cuenta como ahorro: falta ver que F3-4 cubra su orden)
   paso de OpenDockTrading que desaparece con el roadmap fuera del arbol (F1-7): verificar.bat:[59]
-RESULTADO D ok
+RESULTADO D consumidores iguales=3/3
 ```
 
-Lectura: con la carpeta fija los dos lectores **dejan de ver todas las
-tareas** (0 de 94 y 0 de 395), sin error: un fallo silencioso. Con
-`tareas --json`, el mismo cálculo da el mismo conjunto con un consumidor de 3
-líneas.
+Lectura: con la carpeta fija los tres consumidores **dejan de ver todas las
+tareas** (0), sin error: un fallo silencioso. Con `tareas --json`, cada uno da
+**el mismo resultado que hoy**, comparado en lo que calcula (el texto por
+tarea, el recuento de criterios, y los errores, el Mermaid y el roadmap de
+ToolsaWeb), con 11 líneas de consumidor frente a 114 de lectura propia. La
+interfaz es simulada: la cifra es **proyectada**.
 
 ### E — La completa de ToolsaWeb y `RAMA_BASE`
 
@@ -278,40 +454,44 @@ como el clasificador escribe la completa con plan por grupos desde TASK-344.)
 1. **Práctica 1**: el plan dice «`estado: planificada`» en todas las
    archivadas; TASK-097 de ToolsaWeb está en `en-diseno`. Las 11 llevan la
    etiqueta `descartada`, así que D1a no cambia (11 de 11).
-2. **Citas reajustadas** a la línea exacta del commit fijo (p. ej. «la
-   transición sigue y avisa» está en la línea 38 de `TASK-068/tarea.md`, no
-   en 35-39; el aviso de comillas de OpenDockTrading, en 5-6). El inventario
-   lleva las que la PoC comprueba.
-3. **PoC C con 7 transiciones**, no 5: las 5 del plan más dos hostiles en
-   ejecución (un `comprueba` que falla y un derivado que escribe en el
-   código).
-4. **PoC D con los dos pilotos**: el plan solo pedía `lector-tareas.py`.
-5. **B mide también lo que la traducción necesita y hoy no existe**: una
-   orden de huella local en ToolsaWeb y la aceptación de `--perfil` en su
-   `verificar.bat`. Lo hereda F5-1 (ver «Consecuencias» y «Preguntas
-   abiertas»).
-6. **OpenDockTrading no baja de claves** (2 → 2); la ventaja en claves es de
-   ToolsaWeb (10 → 2). El plan hablaba de ventaja en general.
+2. **Citas reajustadas** a la línea exacta del commit fijo (p. ej. el aviso de
+   comillas de OpenDockTrading, en 5-6). El inventario lleva las que la PoC
+   comprueba.
+3. **PoC C con 10 transiciones**, no 5: las 5 del plan, un `comprueba` en rojo
+   con el check `estado`, y cuatro hostiles en ejecución.
+4. **PoC C2 y D con los dos pilotos y los consumidores reales**: el plan solo
+   pedía `lector-tareas.py`.
+5. **`hitos.md` de ToolsaWeb** (revisión 1, IMP-1): sus tablas y su Mermaid
+   pasan a la rama de estado; el plan dejaba las tablas como pregunta abierta.
+6. **Las dos medidas de claves** (revisión 1, IMP-4): el plan esperaba una
+   ventaja en claves; por valores a rellenar, la configuración crece.
 7. **Nombre propuesto `taskctl derivados`** para volver a ejecutar los
    derivados tras un fallo (el aviso tiene que nombrar un comando, como hoy
    `taskctl roadmap --escribir` en OpenDockTrading). El plan no lo nombraba.
+8. **Clasificación parche/propio y desglose de líneas**: decisión de la
+   persona posterior al plan (2026-11-14).
 
 ### No verificado
 
-- **`taskctl tareas --json` real**: la PoC D lo simula; la forma exacta de la
-  salida la fija F2-7 o F3 (campos mínimos: `id`, `estado`, `etiquetas`,
-  `dependencias`, `sprint`).
-- **El check `estado` en GitHub** (D3a): diseño, sin workflow ni PR real.
+- **`taskctl tareas --json` real**: la PoC D lo simula; los campos (todo el
+  frontmatter, `criterios` y `texto`) son contrato para F2/F3.
+- **El check `estado` en GitHub** (D3a): la PoC C lo simula en local; no hay
+  workflow ni PR real.
+- **El `main` adaptado de `grafo-tareas.py`**: C2 usa sus funciones reales,
+  no su línea de órdenes nueva.
 - **La concurrencia de derivados** en la rama de estado (dos clones en
   transición a la vez, un derivado que no termina): sin rol de riesgos; el
   reintento por rebase es de F1-2, y lo generado se **regenera** tras el
   rebase, como el `ROADMAP.md` (F1-1).
+- **El coste de comparar por contenido** en un árbol de código con muchos
+  ficheros ignorados (`target/`, `node_modules/`): la PoC lo hace en un repo
+  pequeño; F2 decide cómo acotarlo.
 - **La caché de la completa de ToolsaWeb con `RAMA_BASE`**: E la prueba con
-  la caché vacía; si su clave de caché incluye `RAMA_BASE` (lo dice el
-  docstring de `cache-verificacion.py` para los pasos de documentación), un
-  cambio de base solo daría un fallo de caché, que es seguro.
+  la caché vacía.
 - **El shell de los derivados**: la PoC los lanza con `cmd /d /c` en Windows;
   en Linux y macOS no se ha ejecutado (H16).
+- **La parte del panel de tareas** en `resumen-verificacion.py` (ver el
+  desglose).
 - **`taskctl doctor`** no se ejecutó en esta implementación (la tarea pidió no
   lanzar `taskctl`); queda para el cierre.
 
@@ -320,8 +500,8 @@ como el clasificador escribe la completa con plan por grupos desde TASK-344.)
 No hay PoC en GitHub. Lo que se deduce de F1-1: los commits de los derivados
 van en `taskcode/tareas`, una rama sin `.github/`, así que no disparan
 Actions; se leen como cualquier fichero de esa rama
-(`.../blob/taskcode/tareas/<fichero>`). El check `estado` de D3a es un job del
-workflow de TaskCode en el PR (F5-1), no cacheado, que lee la rama de estado.
+(`.../blob/taskcode/tareas/<fichero>`). El check `estado` es un job del
+workflow de TaskCode en cada PR (F5-1).
 
 ## Decisión
 
@@ -329,7 +509,8 @@ workflow de TaskCode en el PR (F5-1), no cacheado, que lee la rama de estado.
    CODEOWNERS y **leídas de la base** como las de F1-10; YAML estándar
    (F1-9); ante una clave o un valor que no reconoce, **falla cerrado**. En
    `tarea.md`, en cambio, una clave desconocida se conserva (F1-9 D4a): son
-   dos ficheros con dos reglas.
+   dos ficheros con dos reglas. El tope de 12 de F2-7 cuenta claves de primer
+   nivel.
 
    <!-- esquema:inicio -->
    | Clave | Tipo | Qué declara | Validación (falla cerrado) |
@@ -337,17 +518,17 @@ workflow de TaskCode en el PR (F5-1), no cacheado, que lee la rama de estado.
    | `verificacion` | mapa | el contrato de F1-7 | claves conocidas solamente |
    | `verificacion.comando` | orden | el comando del proyecto; `taskctl` y el workflow lo llaman con `--perfil <p>` | texto no vacío, sin caracteres de control |
    | `verificacion.shell` | `cmd`, `bash`, `pwsh` o `sh` | con qué shell se lanza en todas partes (H16); el workflow no la reinterpreta | uno de los cuatro |
-   | `verificacion.perfiles.<p>.timeout` | segundos | tiempo máximo del perfil `rapida`, `impacto`, `completa` o `pesada`; sustituye a `timeout_*` y `presupuesto_freno_s` | entero de 1 a 21600; `completa` obligatoria; ningún otro perfil |
-   | `verificacion.perfiles.<p>.runner` | `github-windows`, `github-linux`, `github-macos` o `[self-hosted, ...]` | runner de `completa` y `pesada` (F1-7 dec. 5); `rapida` e `impacto` corren en local y no lo llevan | obligatorio en `completa` y `pesada`; prohibido en las otras |
+   | `verificacion.perfiles.<p>.timeout` | segundos | tiempo máximo del perfil `rapida`, `impacto`, `completa` o `pesada`. En `rapida` (atajo local opcional, no es puerta) es un **presupuesto que no bloquea**: agotarlo es «excedido», como hoy `presupuesto_freno_s`; en las demás, agotarlo es **rojo** (D6a) | entero de 1 a 21600; `completa` obligatoria; ningún otro perfil |
+   | `verificacion.perfiles.<p>.runner` | `{github: windows}`, `{github: linux}` o `[self-hosted, ...]` | runner de `completa` y `pesada`, con el vocabulario de F1-7 dec. 5; `rapida` e `impacto` corren en local y no lo llevan | obligatorio en `completa` y `pesada`; prohibido en las otras |
    | `verificacion.huella_arbol` | globs | ficheros versionados que entran en la clave de caché (F1-7 dec. 2) | sin negación, rutas absolutas ni `..` |
    | `verificacion.huella_local` | orden | imprime lo que la suite lee fuera del árbol en el runner | como `comando` |
    | `verificacion.programada.comando` | orden | pasos que dependen de la red, en un job programado fuera del check cacheado | como `comando` |
    | `verificacion.programada.cron` | cron | cuándo corre | expresión de 5 campos |
    | `verificacion.programada.runner` | como `perfiles.<p>.runner` | dónde corre | como `perfiles.<p>.runner` |
-   | `derivados` | lista | comandos que `taskctl` ejecuta en las transiciones | lista de mapas |
+   | `derivados` | lista | comandos que `taskctl` ejecuta en las transiciones y que el check `estado` repite en el PR | lista de mapas |
    | `derivados[].comando` | orden | qué ejecuta | como `verificacion.comando` |
    | `derivados[].tipo` | `genera` o `comprueba` | si escribe ficheros o solo comprueba | uno de los dos |
-   | `derivados[].escribe` | rutas | lo único que un `genera` puede escribir, siempre bajo `tareas/` | obligatoria en `genera`, prohibida en `comprueba`; bajo `tareas/`, sin `*` ni `..`; nunca `tareas/ROADMAP.md` ni `tareas/TASK-NNN/` |
+   | `derivados[].escribe` | rutas | lo único que un `genera` puede escribir, siempre bajo `tareas/` | obligatoria en `genera`, prohibida en `comprueba`; se **normaliza** antes de comparar (segmentos `.` y vacíos, `..` resuelto, sin `\`, `:`, comodines, caracteres de control ni nombres reservados de Windows); bajo `tareas/` y nunca `tareas/` entera ni `.git`; las reservadas (`tareas/ROADMAP.md`, `tareas/TASK-NNN/`) se comparan **sin distinguir mayúsculas**; en ejecución, una ruta ignorada en la rama de estado no se acepta |
    | `derivados[].fases` | lista | en qué transiciones corre (por defecto, en todas) | subconjunto no vacío de las fases de `taskctl` |
    | `derivados[].timeout` | segundos | tiempo máximo (por defecto 60, como `timeout_sincronizacion`) | entero de 1 a 600 |
    | `confianza_incluye` | globs | ficheros que la huella de confianza suma al cierre de cada orden (F1-8 dec. 5) | sin negación, rutas absolutas ni `..`, y nunca todo el árbol (`**`, `*`) |
@@ -356,7 +537,7 @@ workflow de TaskCode en el PR (F5-1), no cacheado, que lee la rama de estado.
    Las rutas de Windows se escriben **entre comillas simples o con `/`**; una
    orden con un carácter de control (el `\v` de `"scripts\verificar.bat"`)
    es un error con ese arreglo en el mensaje. Ejemplo (la traducción de
-   ToolsaWeb que valida la PoC B):
+   ToolsaWeb que valida la PoC B; su `genera` y su `comprueba` son los de C2):
 
    ```yaml
    verificacion:
@@ -370,148 +551,224 @@ workflow de TaskCode en el PR (F5-1), no cacheado, que lee la rama de estado.
      huella_arbol: ['Cargo.lock', 'web/package-lock.json']
      huella_local: 'python -I scripts/huella-local.py'
    derivados:
+     - comando: 'python -I scripts/grafo-tareas.py --escribir'
+       tipo: 'genera'
+       escribe: ['tareas/hitos.md']
      - comando: 'python -I scripts/grafo-tareas.py --comprobar'
        tipo: 'comprueba'
-       fases: ['plan', 'approve', 'finish']
    confianza_incluye: ['scripts/cache-verificacion.py', 'scripts/memo_ficheros.py']
    ```
 
+   (`confianza_incluye` de ToolsaWeb nombra hoy `cache-verificacion.py`; al
+   retirar ese parche, sale de la lista.)
+
 2. **Confianza.** Toda orden de estas claves es **entrada no confiable** y
-   pasa por `confianza(repo, clave)` de F1-8 (dec. 5) antes de ejecutarse,
-   con `clave` = la ruta de la clave (`verificacion.comando`,
+   pasa por `confianza(repo, clave)` de F1-8 (dec. 5) antes de ejecutarse en
+   local, con `clave` = la ruta de la clave (`verificacion.comando`,
    `derivados[0]`…). `cambiado` o `nunca`: no se ejecuta y el aviso nombra
    `taskctl confiar <clave>` (F0-2). `confianza_incluye` es la única forma de
    sumar ficheros a la huella (lo que se carga por nombre de módulo, como el
-   `lector-tareas.py` de OpenDockTrading o el `cache-verificacion.py` de
-   ToolsaWeb).
+   `lector-tareas.py` de OpenDockTrading).
 
 3. **Los derivados los ejecuta `taskctl` en la transición**: un punto
    explícito y determinista del comando que la persona o el agente lanzan,
    **no un hook**. Orden: escribe la tarea, ejecuta los derivados de esa fase
    en el worktree principal con `TASKCODE_TAREAS` (la carpeta montada de la
    rama de estado) y `TASKCODE_CODIGO` (el árbol de código), y commitea la
-   tarea y lo generado en **un solo commit** de `taskcode/tareas`. Si un
-   derivado falla, no está confiado o escribe fuera de `escribe` (o un
-   `comprueba` escribe algo), lo suyo se descarta, **la transición sigue** y
-   el aviso dice qué pasó y qué comando lo arregla (`taskctl derivados` para
-   volver a ejecutarlos; `taskctl confiar <clave>`), como hoy la 0.16 con el
-   roadmap. `doctor` (F3-8) repite el aviso mientras siga pendiente.
-
-4. **Lectura del estado**: `taskctl tareas --json` es la **API estable** para
-   los scripts del proyecto; la forma de las carpetas de `tareas/` no es
-   contrato (la PoC D muestra por qué: el glob de hoy encuentra 0 tareas en la
-   carpeta fija, sin error).
-
-5. **Sin estados propios** (D1a): `descartada` (con motivo) y `pausa` nativos
+   tarea y lo generado en **un solo commit** de `taskcode/tareas`. Lo que un
+   derivado escribe se comprueba **por contenido** (hash de cada fichero de
+   la rama de estado fuera de `escribe`, incluido el `tarea.md` de la
+   transición; y de lo modificado, nuevo o ignorado del árbol de código), no
+   por las líneas de `git status`. Si un derivado falla, no está confiado o
+   escribe fuera de `escribe` (o un `comprueba` escribe algo), lo suyo se
+   restaura, **la transición sigue** y el aviso dice qué pasó y qué comando
+   lo arregla (`taskctl derivados` para volver a ejecutarlos; `taskctl
+   confiar <clave>`). `doctor` (F3-8) repite el aviso mientras siga
+   pendiente. Un derivado que necesite ficheros temporales usa la carpeta
+   temporal del sistema, nunca los dos árboles.
+
+4. **Check `estado`** (D3a): un check **obligatorio** en cada PR que ejecuta
+   los `comprueba` de la configuración **de la base** contra el árbol del PR
+   y la rama `taskcode/tareas`; **no se cachea** (su resultado depende del
+   estado, que no está en el árbol) y un `comprueba` en rojo lo pone en rojo
+   y **bloquea** el merge. Es un job de la clase que ejecuta código del PR sin
+   privilegios (F1-8 dec. 1: sin secretos, permisos de solo lectura), así que
+   en CI no usa la confianza local. Sustituye la puerta que hoy da el paso
+   1/22 de `verificar.bat` en ToolsaWeb: el bloqueo pasa de la verificación al
+   check `estado`, y en la transición un `comprueba` solo avisa.
+
+5. **Lectura del estado**: `taskctl tareas --json` es la **API estable** para
+   los scripts del proyecto, con todo el frontmatter, `criterios` (las líneas
+   de «## Criterios de aceptacion») y `texto` (el `tarea.md` entero); la
+   forma de las carpetas de `tareas/` no es contrato.
+
+6. **Sin estados propios** (D1a): `descartada` (con motivo) y `pausa` nativos
    (F3-7). La migración (F1-12) lleva cada tarea de `05-archivadas` a
    `descartada`.
 
-6. **Revisoras propias** (D5a): una skill del proyecto con el mismo bloque
+7. **Revisoras propias** (D5a): una skill del proyecto con el mismo bloque
    ```` ```yaml ```` que ya lee el plugin (`rol: revisor` y
    `patrones_archivo`), sin clave en la configuración. Dónde se buscan y cómo
    se eligen lo decide F1-15.
 
-7. **Códigos** (D6a): cualquier código distinto de 0 es rojo, también los 3
-   y 4 del cerrojo de ToolsaWeb; el cerrojo sigue dentro del comando del
-   proyecto.
+8. **Códigos** (D6a): cualquier código distinto de 0 es rojo, también los 3
+   y 4 del cerrojo de ToolsaWeb, y un `timeout` agotado salvo en `rapida`.
+
+9. **Parches que se retiran.** Las prácticas clasificadas como **parche** y
+   las líneas de parche del desglose son lo que el proyecto añadió para la
+   0.16; la 2.0 las cubre de forma genérica y `taskctl actualizar` las retira
+   al migrar. Lo **propio** se queda.
 
-8. **Principio: comandos sí, hooks solo como atajo opcional.** Ningún punto
-   de extensión depende de un hook. El único hook previsto en la 2.0 (F5-6,
-   la `rapida` en local) es un atajo opcional que llama a
-   `verificacion.comando --perfil rapida`, con confianza; quitarlo no cambia
-   ninguna garantía, porque las puertas son la transición, el check
-   `verificacion` y el check `estado`. Las guardas que un proyecto ponga en
-   `.claude/` (práctica 14) son suyas y la 2.0 no cuenta con ellas.
+10. **Principio: comandos sí, hooks solo como atajo opcional.** Ningún punto
+    de extensión depende de un hook. El único hook previsto en la 2.0 (F5-6,
+    la `rapida` en local) es un atajo opcional que llama a
+    `verificacion.comando --perfil rapida`, con confianza; quitarlo no cambia
+    ninguna garantía, porque las puertas son la transición, el check
+    `verificacion` y el check `estado`.
 
 ### Ventaja medida
 
-Cifras de la PoC; el guion falla si alguna de esta tabla no es la medida.
+Cifras de la PoC. «medido» es lo que la PoC observa sobre datos reales o
+sobre una copia; «proyectado», lo que depende de una interfaz simulada. El
+guion falla si una cifra no es la medida, si su tipo no es el que le toca o si
+una cifra medida falta de la tabla.
 
 <!-- ventaja:inicio -->
-| Cifra | Valor | Fuente |
-|---|---|---|
-| practicas | `14` | A |
-| practicas a mano en la 0.16 | `8` | A |
-| practicas con destino declarado en la 2.0 | `13` | A |
-| citas comprobadas | `43/43` | A |
-| casos de 05-archivadas cubiertos por descartada | `11/11` | A |
-| ToolsaWeb: archivadas con estado distinto de descartada | `9/9` | A |
-| OpenDockTrading: archivadas con estado distinto de descartada | `2/2` | A |
-| claves de la 0.16 en el catalogo | `32` | B |
-| claves de la 0.16 de verificacion y sincronizacion | `12` | B |
-| ToolsaWeb: claves de verificacion y sincronizacion 0.16 -> 2.0 | `10 -> 2` | B |
-| OpenDockTrading: claves de verificacion y sincronizacion 0.16 -> 2.0 | `2 -> 2` | B |
-| claves de primer nivel de F1-10 y F1-11 frente al tope de 12 | `5 de 12` | B |
-| configuraciones hostiles rechazadas | `13/13` | B |
-| ordenes de la traduccion que no existen hoy | `1` | B |
-| argumentos de perfil que ToolsaWeb acepta hoy | `--selectiva, --pesada, --rapida` | B |
-| transiciones con derivados en la rama de estado | `7` | C |
-| arbol de develop tras las transiciones | `identico` | C |
-| ToolsaWeb: tareas que encuentra su lector en la carpeta fija | `0` | D |
-| OpenDockTrading: tareas que encuentra su lector en la carpeta fija | `0` | D |
-| lineas de lectores propios que sustituye tareas --json | `82 -> 3` | D |
-| pasos de verificacion que desaparecen (OpenDockTrading) | `1` | D |
-| grupos de la completa de ToolsaWeb segun RAMA_BASE | `iguales` | E |
+| Cifra | Valor | Tipo | Fuente |
+|---|---|---|---|
+| OpenDockTrading: archivadas con estado distinto de descartada | `2/2` | medido | A |
+| ToolsaWeb: archivadas con estado distinto de descartada | `9/9` | medido | A |
+| casos de 05-archivadas cubiertos por descartada | `11/11` | medido | A |
+| citas comprobadas | `48/48` | medido | A |
+| practicas | `14` | medido | A |
+| practicas a mano en la 0.16 | `8` | medido | A |
+| practicas a mano en la 0.16: comprobado | `3` | medido | A |
+| practicas a mano en la 0.16: convención | `1` | medido | A |
+| practicas a mano en la 0.16: dentro del comando | `2` | medido | A |
+| practicas a mano en la 0.16: fuera | `1` | medido | A |
+| practicas a mano en la 0.16: proyectado | `1` | medido | A |
+| practicas con destino declarado en la 2.0 | `13` | medido | A |
+| OpenDockTrading: lineas que se retiran frente a las que se quedan | `91 / 973` | medido | A2 |
+| ToolsaWeb verificar.bat: lineas de parche frente a propias | `192 / 491` | medido | A2 |
+| ToolsaWeb: lineas que se retiran frente a las que se quedan | `3360 / 1093` | medido | A2 |
+| ficheros enteros que se retiran | `7` | medido | A2 |
+| practicas propias del proyecto | `7` | medido | A2 |
+| practicas que son parche de TaskCode | `7` | medido | A2 |
+| OpenDockTrading: claves planas 0.16 -> bloques 2.0 | `2 -> 2` | medido | B |
+| OpenDockTrading: valores a rellenar 0.16 -> 2.0 | `2 -> 13` | medido | B |
+| ToolsaWeb: claves planas 0.16 -> bloques 2.0 | `10 -> 2` | medido | B |
+| ToolsaWeb: valores a rellenar 0.16 -> 2.0 | `10 -> 16` | medido | B |
+| argumentos de perfil que ToolsaWeb acepta hoy | `--selectiva, --pesada, --rapida` | medido | B |
+| claves de la 0.16 de verificacion y sincronizacion | `12` | medido | B |
+| claves de la 0.16 en el catalogo | `32` | medido | B |
+| claves de primer nivel de F1-10 y F1-11 frente al tope de 12 | `5 de 12` | medido | B |
+| claves hoja del esquema de F1-11 | `15` | medido | B |
+| configuraciones hostiles rechazadas | `22/22` | medido | B |
+| ordenes de la traduccion que no existen hoy | `1` | medido | B |
+| arbol de develop tras las transiciones | `identico` | medido | C |
+| check estado con un comprueba en rojo | `rojo (bloquea)` | medido | C |
+| hostiles en ejecucion descartados | `4/4` | medido | C |
+| transiciones con derivados en la rama de estado | `10` | medido | C |
+| comprobar de ToolsaWeb tras una transicion, hitos en docs (hoy) | `codigo 1` | medido | C2 |
+| comprueba de ToolsaWeb tras una transicion, disposicion nueva | `0 errores` | medido | C2 |
+| lineas generadas que salen de docs/planificacion/hitos.md | `1554 de 1779` | medido | C2 |
+| OpenDockTrading comprobar-fallos.py: tareas que ve en la carpeta fija | `0` | medido | D |
+| OpenDockTrading comprobar-modulos.py: tareas que ve en la carpeta fija | `0` | medido | D |
+| ToolsaWeb grafo-tareas.py: tareas que ve en la carpeta fija | `0` | medido | D |
+| consumidores reales con el mismo resultado desde tareas --json (interfaz simulada) | `3/3` | proyectado | D |
+| lineas de lectura propia que sustituye tareas --json (interfaz simulada) | `114 -> 11` | proyectado | D |
+| pasos de verificacion que desaparecen (OpenDockTrading) | `1` | medido | D |
+| grupos de la completa de ToolsaWeb segun RAMA_BASE | `iguales` | medido | E |
 <!-- ventaja:fin -->
 
-En resumen: **de 8 prácticas a mano se pasa a 0** dentro de TaskCode (la que
-queda, las guardas de `.claude/`, está fuera por decisión); las 12 claves de
-verificación y sincronización de la 0.16 caben en **2 bloques**, y con F1-10
-el esquema usa **5 de las 12** claves; los lectores propios de frontmatter
-(**82 líneas** en los dos pilotos) se sustituyen por un consumidor de **3**
-líneas de `taskctl tareas --json`; y un paso de la verificación de
-OpenDockTrading (`taskctl roadmap --comparar`) desaparece. El coste: una
-orden de huella local nueva en ToolsaWeb y la adaptación de su
-`verificar.bat` a `--perfil`.
+<!-- resumen:inicio -->
+Dónde gana la versión nueva: de las prácticas a mano, **3** pasan a un mecanismo
+declarado y comprobado, y los **7** parches se retiran al migrar: líneas que se
+retiran frente a las que se quedan, **3360 / 1093** en ToolsaWeb y **91 / 973**
+en OpenDockTrading. Las comprobaciones que cruzan código y estado bloquean en
+el PR (check `estado`) y no esconden fallos tras la caché; el `hitos.md` de
+ToolsaWeb pasa de un rojo permanente a **0 errores**; la configuración ocupa
+**5 de 12** claves de primer nivel. Dónde no gana: la configuración no es más
+corta (valores a rellenar: ToolsaWeb **10 -> 16**, OpenDockTrading
+**2 -> 13**); la caché por paso de ToolsaWeb se pierde al retirarla; ToolsaWeb
+necesita una orden de huella local y `--perfil`; y la lectura del estado
+(**114 -> 11** líneas) es proyectada hasta que exista `tareas --json`.
+<!-- resumen:fin -->
 
 ## Requisitos y evidencia
 
 | Requisito | Decisión | Evidencia |
 |---|---|---|
-| P8 / RN-04 extensiones declaradas en vez de convenciones a mano | tres claves, derivados, `tareas --json`, skill revisora (decisiones 1, 3, 4 y 6) | PoC A: 14 prácticas, 13 con destino declarado, 43/43 citas; 8 a mano hoy |
-| RF-70 regenerar derivados en cada transición | `derivados` que ejecuta `taskctl` en la transición, en la rama de estado (decisión 3); **no** un job de CI ni un hook, como sugería RF-70 | PoC C: 7 transiciones, lo generado en el commit de la transición, avisos con el comando que lo arregla |
-| RR-11 roadmap sin interferir con la verificación | los derivados solo escriben bajo `tareas/` (D2a) | PoC C: árbol de `develop` idéntico, 1 → 1 commits; PoC B: `docs/planificacion/hitos.md` rechazado |
-| RF-71 estados propios / RF-07 descartar con motivo / RF-08 pausar | sin clave de estados (D1a): `descartada` y `pausa` nativos (F3-7) | PoC A: 11/11 archivadas llevan `descartada`; 11/11 con un `estado` incoherente hoy; PoC B: `estados` rechazada |
-| RF-40 / RF-41 comando del proyecto con perfiles y timeout | `verificacion.comando` + `perfiles.<p>.timeout` (decisión 1) | PoC B: 10 claves de ToolsaWeb → 2 bloques; perfil fuera de F1-7 rechazado |
-| RF-42 no repetir sobre el mismo árbol | `huella_arbol` y `huella_local` (F1-7 dec. 2); D3a saca lo que depende del estado del perfil cacheado; `completa` de ToolsaWeb independiente de `RAMA_BASE` | PoC E: 7 grupos en las 4 variantes; PoC A, práctica 3 |
-| RF-72 skills instalables sin referencias internas | revisoras propias como skill del proyecto con el bloque que ya lee el plugin (D5a) | PoC A: 0 revisoras propias hoy; `revisores.ts:113-116` |
-| H2 ejecución de código desde la config | toda orden pasa por `confianza` (decisión 2) | PoC B: nunca → confiado → cambiado → confiado; PoC C: T4 no ejecuta el generador cambiado |
+| P8 / RN-04 extensiones declaradas en vez de convenciones a mano; revisoras propias | tres claves, derivados, check `estado`, `tareas --json`, skill revisora (decisiones 1 a 7) | PoC A: 14 prácticas, 13 con destino, 48/48 citas; de 8 a mano, 3 comprobadas; A: 0 revisoras propias hoy, `revisores.ts:113-116` |
+| RF-70 regenerar derivados en cada transición | `derivados` que ejecuta `taskctl` en la transición, en la rama de estado (decisión 3); **no** un job de CI ni un hook, como sugería RF-70 | PoC C: 10 transiciones, lo generado en el commit de la transición; C2: `hitos.md` de ToolsaWeb |
+| RR-11 roadmap sin interferir con la verificación | los derivados solo escriben bajo `tareas/` (D2a) | PoC C: árbol de `develop` idéntico; PoC B: `docs/planificacion/hitos.md` y 8 rutas disfrazadas rechazadas; C2: `docs/` sin tocar |
+| RF-71 estados propios / RF-07 descartar con motivo / RF-08 pausar | sin clave de estados (D1a): `descartada` y `pausa` nativos (F3-7) | PoC A: 11/11 archivadas llevan `descartada`, ninguna con un `estado` coherente hoy; PoC B: `estados` rechazada |
+| RF-40 / RF-41 comando del proyecto con perfiles y timeout | `verificacion.comando` + `perfiles.<p>.timeout` (decisión 1) | PoC B: 10 claves de ToolsaWeb → 2 bloques y 16 valores; perfil fuera de F1-7 rechazado |
+| RF-42 no repetir sobre el mismo árbol | `huella_arbol` y `huella_local` (F1-7 dec. 2); D3a saca lo que depende del estado del perfil cacheado; `completa` de ToolsaWeb independiente de `RAMA_BASE` | PoC E: 7 grupos en las 4 variantes; la caché por paso se pierde (A2) |
+| H2 ejecución de código desde la config | toda orden pasa por `confianza` en local (decisión 2); en CI, job sin privilegios (decisión 4) | PoC B: nunca → confiado → cambiado → confiado; PoC C: T4 no ejecuta el generador cambiado |
 | H16 shell distinto según quién ejecuta | `verificacion.shell` declarado y usado en todas partes | diseño; Linux y macOS no verificados |
 
+## Divergencias con F1-5 y F1-7
+
+Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
+«Consecuencias»:
+
+1. **F1-5**: la lista de checks obligatorios (`verificacion`, `revision-ia`,
+   `carril`) suma **`estado`** en todos los carriles que llevan tarea (D3a).
+2. **F1-7, decisión 5**: el vocabulario de runner es el suyo (`github` con el
+   sistema, o etiquetas). La primera versión de este ADR usaba
+   `github-windows`, `github-linux` y `github-macos`; se alinea con F1-7 y
+   macOS queda fuera (F1-7 no lo calcula).
+3. **F1-7, decisión 1**: deja el perfil `impacto` en el comando del proyecto;
+   el desglose clasifica el clasificador de ToolsaWeb como parche suponiendo
+   que F5-3 dé el impacto de forma genérica (pregunta abierta).
+
 ## Consecuencias
 
 - **F2-7** suma `verificacion`, `derivados` y `confianza_incluye` al esquema
-  (3 de las 12 claves; con F1-10, 5) con el validador de la decisión 1, y
-  fija los nombres finales de las fases de `derivados[].fases`.
+  (3 de las 12 claves de primer nivel; con F1-10, 5) con el validador de la
+  decisión 1, y fija los nombres finales de las fases de `derivados[].fases`.
+- **F1-5 / F5-1**: el check `estado` es obligatorio y lo implementa el
+  workflow de TaskCode (decisión 4). **F5-1** llama además a
+  `verificacion.comando --perfil <p>` con `shell` y crea el job programado.
+  Para ToolsaWeb exige `--perfil` en `verificar.bat` (hoy acepta `--rapida`,
+  `--selectiva` y `--pesada`; otro argumento sale con 2) y una orden de huella
+  local: el criterio de F5-1 «sin scripts nuevos» queda en duda para
+  ToolsaWeb.
+- **F1-12 / F9-1 / F9-2 (`taskctl actualizar`)**: retira los parches de la
+  lista (decisión 9; prácticas parche e inventario de A2: ficheros enteros,
+  funciones y líneas de los `verificar.bat`), migra `05-archivadas` a
+  `descartada`, traduce las claves planas al esquema nuevo y avisa de los
+  lectores que usan el glob `0?-*/TASK-*/tarea.md`; **`--simular` enumera**
+  cada parche con su fichero y sus líneas antes de tocar nada. Para ToolsaWeb,
+  mueve las tablas y el Mermaid de `hitos.md` a `tareas/hitos.md` y adapta el
+  `main` de `grafo-tareas.py`.
 - **F0-2** implementa `confianza` con `confianza_incluye` y `taskctl confiar
   <clave>`; **F5-6** (hook) la usa sin otra propia y solo llama a
   `verificacion.comando --perfil rapida`.
-- **F3-4** (roadmap) genera el Mermaid por hito que hoy hace
-  `grafo-tareas.py` (90 líneas de ToolsaWeb solapan) y decide si adopta su
-  orden topológico; **F3-7** da `descartada` con motivo y `pausa`; **F3-8**
-  (`doctor`) avisa de derivados pendientes, de órdenes sin confianza y de
-  una configuración inválida en la base.
-- **F2/F3** añaden `taskctl tareas --json` (lectura estable) y `taskctl
-  derivados` (volver a ejecutar los derivados de la fase actual).
-- **F5-1** llama a `verificacion.comando --perfil <p>` con `shell`, crea el
-  job programado y el check `estado` (D3a). Para ToolsaWeb exige dos cambios
-  que hoy no existen: `--perfil` en `verificar.bat` (hoy acepta `--rapida`,
-  `--selectiva` y `--pesada`; otro argumento sale con 2) y una orden de huella
-  local. El criterio de F5-1 «sin scripts nuevos» queda en duda para
-  ToolsaWeb (ver «Preguntas abiertas»).
-- **F5-4** recoge el recuento «# tests N» en el informe JSON.
-- **F1-12** migra `05-archivadas` a `descartada`, traduce las claves planas
-  al esquema nuevo y avisa de los lectores que usan el glob
-  `0?-*/TASK-*/tarea.md` (PoC D: 0 tareas sin error).
+- **F3-4** (roadmap) escribe `tareas/ROADMAP.md`, que sustituye el `roadmap()`
+  de `grafo-tareas.py` (parche), y decide si adopta su orden topológico;
+  **F3-7** da `descartada` con motivo y `pausa`; **F3-8** (`doctor`) avisa de
+  derivados pendientes, de órdenes sin confianza y de una configuración
+  inválida en la base.
+- **F2/F3** añaden `taskctl tareas --json` (con frontmatter, `criterios` y
+  `texto`) y `taskctl derivados` (volver a ejecutar los derivados de la fase
+  actual).
+- **F5-3** decide si el impacto es genérico; si no, el clasificador de
+  ToolsaWeb pasa de parche a propio.
+- **F5-4** recoge en el informe JSON los pasos y el recuento que hoy van por
+  los canales de texto (práctica 8).
 - **F1-15** decide dónde se buscan las skills revisoras del proyecto y cómo
   se eligen; este ADR solo fija el formato.
 - **Este repo no cambia** (RR-14).
 
 ## Fuera de alcance
 
-- Las guardas del arnés en `.claude/` (permisos `deny` y `ask`, hooks de
-  Claude Code): no son extensión de TaskCode. La 2.0 no depende de ellas; lo
-  que protegen lo imponen `taskctl` y las puertas de GitHub (F1-3, F1-5).
+- Las guardas del arnés en `.claude/` como extensión: no lo son. La 2.0 no
+  depende de ellas y, como parche, se retiran al migrar. Si `.claude/` debe
+  entrar en el suelo de riesgo alto de F1-10 (pregunta que F1-10 dejó a F1-8
+  o F1-11): F1-11 **no lo decide** y lo devuelve a **F1-8**; mientras tanto, un
+  proyecto lo puede subir con `rutas_carril`.
 - Las claves base de F2-7 (`rama_base`, `modo_flujo`, `tope_rondas`…): el
   presupuesto de este ADR solo cuenta F1-10 y F1-11.
 - Selección de skills y revisoras: F1-15. Formato del informe JSON: F5-4.
@@ -522,14 +779,13 @@ orden de huella local nueva en ToolsaWeb y la adaptación de su
 Este ADR se planificó con un solo rol (arquitectura); quedan sin cubrir:
 
 - **Riesgos de los derivados**: concurrencia en la rama de estado, un
-  derivado que no termina antes del `timeout`, un `comprueba` que queda viejo
-  entre transiciones (lo pilla la siguiente o el check `estado`).
-- **Dominio**: si las tablas a mano de `hitos.md` §1 de ToolsaWeb, que viven
-  en `docs/` (rama de código), quedan bien como documento manual más un
-  `comprueba`, o tienen que ir a la rama de estado.
-- **ToolsaWeb y F5-1**: si la orden de huella local y `--perfil` se resuelven
-  con un subcomando de sus scripts actuales (`cache-verificacion.py` ya
-  calcula la clave) o con un fichero nuevo.
+  derivado que no termina antes del `timeout`, el coste de comparar por
+  contenido en árboles grandes.
+- **Impacto genérico (F5-3)**: si no llega, el clasificador de ToolsaWeb se
+  queda como propio y el desglose cambia.
+- **Caché por paso**: la 2.0 no la da; si su pérdida pesa en ToolsaWeb
+  (selectivas de 26 min), se mide con datos de la 2.0 (P9).
+- **ToolsaWeb y F5-1**: la orden de huella local y `--perfil`.
 - **RF-43** (`rutas_documentacion`): ningún piloto la usa en su
   configuración (PoC B lista sus claves) y este esquema no le da clave; si
   hace falta, F5-3 o F2-7.
diff --git a/docs/adr/README.md b/docs/adr/README.md
index c3474c6..1c73570 100644
--- a/docs/adr/README.md
+++ b/docs/adr/README.md
@@ -18,7 +18,7 @@ directo a la tabla de la Fase 1 y a su tarea.
 | [F1-8](F1-8-seguridad-modelo-de-amenazas.md) | Seguridad: modelo de amenazas por fronteras de confianza, dos clases de jobs (ejecuta código del PR / tiene privilegio), revisor partido en dos jobs, confianza local única con hash de la orden y de lo que ejecuta, prompt injection por capacidades, runner propio aislado y rotación de secretos | TASK-009 | aceptada |
 | [F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md) | Formato de tarea, plan y especificación: `tarea.md` como único modelo canónico (formato 2, YAML estándar, cadenas entre comillas, texto como dato no confiable), tabla de los 22 campos de la 0.16 con su uso medido, plantillas de `tarea.md` y `plan-final.md`, y correspondencia con Spec Kit | TASK-010 | aceptada |
 | [F1-10](F1-10-carriles-y-modo-automatico.md) | Carriles y modo automático: cuatro carriles ordenados más release y hotfix por tipo, tabla única de política por carril con su función pura (`accion` en `siguiente --json`), configuración que solo endurece (`politica_carriles`, `rutas_carril`), carril mínimo por ficheros con los dos nombres de cada renombrado, y paradas fijas (`approve`, `merge-pr`, tag, tope de rondas, subida de carril) | TASK-011 | aceptada |
-| [F1-11](F1-11-extensiones-declaradas.md) | Extensiones declaradas: inventario comprobado de las 14 prácticas que los pilotos hacen a mano, tres claves (`verificacion`, `derivados`, `confianza_incluye`) con esquema que falla cerrado, derivados que ejecuta `taskctl` en la transición solo en la rama de estado, `taskctl tareas --json` como lectura estable, sin estados propios, revisoras como skill del proyecto, «comandos sí, hooks solo como atajo opcional» y la ventaja medida frente a la 0.16 | TASK-012 | aceptada |
+| [F1-11](F1-11-extensiones-declaradas.md) | Extensiones declaradas: inventario comprobado de las 14 prácticas que los pilotos hacen a mano, clasificadas en parche de TaskCode (se retira al migrar) o propio del proyecto, con el desglose de líneas; tres claves (`verificacion`, `derivados`, `confianza_incluye`) con esquema que falla cerrado y rutas normalizadas; derivados que ejecuta `taskctl` en la transición solo en la rama de estado, comparados por contenido; check obligatorio `estado`; `taskctl tareas --json` como lectura estable; sin estados propios; revisoras como skill del proyecto; «comandos sí, hooks solo como atajo opcional»; y la ventaja medida frente a la 0.16, con lo que no gana | TASK-012 | aceptada |
 
 ## Plantilla
````

## Excluido del diff (8 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 7c532cc7aa97b984ec835c5159843ecedeb6f4df..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/02-en-curso/TASK-012/poc/poc-f1-11.log                                               | 140 -------------------
 tareas/{02-en-curso => 03-en-revision}/TASK-012/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-012/planificacion/plan-final.md                 |   0
 tareas/03-en-revision/TASK-012/poc/poc-f1-11.log                                            | 232 +++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-012/poc/poc-f1-11.py                            | 763 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-----------------
 tareas/03-en-revision/TASK-012/revision/informe-revision-1.md                               |  21 +++
 tareas/03-en-revision/TASK-012/revision/peticion-revision-1.md                              | 623 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-012/tarea.md                                    |  11 +-
 8 files changed, 1517 insertions(+), 273 deletions(-)
````
