# Plan — TASK-012: F1-11 ADR extensiones declaradas

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 170 min

Senales: 2 dependencias (F1-1, F1-7) y heuristica «simple», pero el ADR cierra cuatro costuras abiertas por F1-1, F1-7, F1-8 y F1-9 y la PoC repasa ~14 practicas reales una por una. **No se trocea** (ver «Motivo para no trocear»).

**Que se crea**: `docs/adr/F1-11-extensiones-declaradas.md` (plantilla de `docs/adr/README.md`), su fila en el indice y `poc/` en la tarea (guion y log). **No se toca**: codigo de la 2.0, los ADR F1-1..F1-10 (se citan), ToolsaWeb, OpenDockTrading ni el plugin (solo lectura).

### Restriccion de la persona (2026-11-14)

Carlos: lo mas importante es que el plugin sea **eficiente y presente ventajas medibles**, y prefiere **comandos deterministas** que sabe que hacen fielmente a hooks que arrancan algo por el contexto. Consecuencias para este ADR: el contrato de extensiones son comandos declarados que `taskctl` ejecuta en puntos explicitos (transicion, verificacion); **ningun punto de extension depende de un hook**; un hook, si existe, solo es un atajo opcional que llama al mismo comando. El ADR mide la ventaja frente a la 0.16 (practicas cubiertas, claves de configuracion, scripts propios que dejan de hacer falta).

### Inventario de lo que se hace hoy a mano (verificado leyendo, solo lectura)

| # | Practica | Evidencia | Punto de extension en la 2.0 |
|---|---|---|---|
| 1 | Estado propio `05-archivadas` = carpeta + etiqueta `descartada` + `estado: planificada` (incoherente) | TW `tareas/05-archivadas/` 9 tareas, p. ej. `TASK-171/tarea.md:6,9`; ODT 2 (`TASK-068/tarea.md:6,9`) | estado nativo `descartada` con motivo (F3-7); sin estado propio (D1) |
| 2 | Regenerar el Mermaid de `hitos.md` y `ROADMAP.md` en cada transicion | TW `.taskcode/config.yml:31-32`; `scripts/grafo-tareas.py:4-7` | roadmap nativo con Mermaid por hito (F1-1 dec. 3); el resto a `derivados` en la rama de estado (D2) |
| 3 | Comprobaciones que cruzan codigo y estado dentro de la verificacion | TW `verificar.bat:125-128`; ODT `verificar.bat:51-53` → `comprobar-modulos.py:88`, `comprobar-fallos.py:305-307` | `derivados` de tipo `comprueba`, fuera del perfil cacheado (D3) |
| 4 | Lector propio de frontmatter que recorre `tareas/0?-*/` | TW `grafo-tareas.py:71,99`; ODT `lector-tareas.py:15,39` | lectura estable `taskctl tareas --json` (la carpeta fija de F1-1 rompe ese glob) |
| 5 | Cuatro perfiles con seis claves | TW `config.yml:9,16,18,23`; `flujo-de-verificacion.md:9-12` | `verificacion.comando --perfil` (F1-7 dec. 1) |
| 6 | Tiempo maximo por perfil y presupuesto del freno | TW `config.yml:10,17,20`; ODT `config.yml:10` | `verificacion.perfiles.<p>.timeout` |
| 7 | Cache propia por paso y «completa del mismo arbol» | TW `cache-verificacion.py:1-28`; `verificar.bat:66-93`; `flujo...md:164-171` | `verificacion.huella_arbol` + `huella_local` (F1-7 dec. 2) |
| 8 | Canales de texto: x/N en `TASKCTL_PROGRESO` y `# tests N` | TW `verificar.bat:51-52`; `config.yml:7-8`; plugin `fs/progreso.ts:33` | variable de progreso declarada; el recuento al informe JSON (F5-4) |
| 9 | `RAMA_BASE` como entrada | TW `clasificar-verificacion.py:14-15,54-56` | solo `rapida`/`impacto` (F1-7 dec. 1); la PoC comprueba la completa |
| 10 | Pasos que dependen de la red (`cargo audit`, `npm audit`) | ODT `verificar.bat:68-70` | `verificacion.programada` (job programado, F1-7) |
| 11 | Cerrojo propio con codigos 3/4 | TW `cerrojo-verificacion.py:1-19`; `flujo...md:180-185` | dentro del comando del proyecto; para el contrato es rojo (D6) |
| 12 | Ruta de Windows en YAML (`"\v"`) | ODT `config.yml:5-7`; TW `config.yml:9`; F1-8 consecuencias | el formato fija como se escriben las rutas (comillas simples o `/`) |
| 13 | Revisoras propias: ningun proyecto las declara; el catalogo sale del plugin | plugin `core/revisores.ts:69-74,113-117` | skill del proyecto con bloque `rol: revisor` (D5, de acuerdo con F1-15) |
| 14 | Guardas del arnes en `.claude/settings.json` (hook Stop si HEAD desligado, `deny git checkout`, `ask --ronda-extra`) | TW `.claude/settings.json:3-27` | no es extension de TaskCode: se queda en `.claude/`; fuera de alcance. Con la restriccion de la persona, el ADR anota que la 2.0 no depende de esas guardas: lo que protegen lo hacen comandos y puertas (F1-3, F1-5) |

### Que decide el ADR (contrato)

- **Tres claves para F2-7**, bajo CODEOWNERS y leidas de la base (como F1-10): `verificacion` (bloque: `comando`, `shell`, `perfiles.<p>.{timeout,runner}`, `huella_arbol`, `huella_local`, `programada`); `derivados` (lista de `{comando, tipo: genera|comprueba, escribe: [rutas bajo tareas/], fases}`); `confianza_incluye` (globs de F1-8). Con las dos de F1-10 suman 5 de 12; con claves planas al estilo 0.16 harian falta 10 o mas solo para esto.
- Todo `comando` es entrada no confiable y pasa por `confianza(repo, clave)` (F1-8 dec. 5). Clave desconocida en `tarea.md` se conserva (F1-9 D4a); en `config.yml` falla cerrada (F1-10).
- **Los derivados los ejecuta `taskctl` en la transicion** (punto explicito y determinista, no un hook), en el worktree de la rama de estado, y se commitean con la transicion. Si fallan, la transicion sigue y avisa con el comando que lo arregla (como hoy la 0.16, ODT `TASK-068/tarea.md:35-39`). Reciben `TASKCODE_TAREAS` y `TASKCODE_CODIGO`.
- **Lectura del estado**: `taskctl tareas --json` como API estable para los scripts del proyecto; la forma de las carpetas no es contrato.
- Descartado: maquina de estados configurable y job de CI por transicion (F1-1:140, sin CI en el push de la rama de estado).

**Orden**: 1) inventario en el ADR con la salida de la PoC; 2) PoC; 3) contrato y alternativas; 4) tabla requisitos → evidencia (P8, RN-04, RF-70, RF-71, RF-07, RF-40..43, RF-72, H2/H16; cada ID comprobado antes de citarlo); 5) consecuencias para F2-7, F3-4, F3-7, F5-1, F5-4, F5-6, F0-2, F1-12, F1-15; 6) fila del indice.

### Motivo para no trocear

La persona (Carlos) indico al lanzar el H1 el 2026-11-14: «NO troceamos ninguna tarea». Ademas, el inventario, el contrato y la PoC de cobertura son un mismo hecho: partirlos daria dos fuentes para la misma tabla.

## Lo que el rol no cubrio

- **Riesgos**: como se rompen los derivados (concurrencia en la rama de estado, derivado que no termina, `comprueba` viejo entre transiciones); F1-2 cubre una parte.
- **Testing**: sin rol de pruebas.
- **Dominio**: si `hitos.md` §1 (tablas a mano con el motivo de cada hito) queda bien como documento manual mas un `comprueba`.
- **Criterio 1**: cubierto con lo leido de `config.yml`, `scripts/`, `.claude/` y `tareas/05-archivadas`; no se reviso `docs/` de ODT ni hooks de Git (`.githooks` no existe en TW; en ODT no se vio ninguno).
- **Criterio 2**: el presupuesto de 12 claves solo cuenta F1-10, F1-8 y F1-11; no las claves base de F2-7 (`rama_base`, `modo_flujo`, `tope_rondas`…).

## Riesgos aceptados y que los contiene

- F1-15 elige otra forma para las revisoras propias: D5 solo fija «skill del proyecto con `rol: revisor`» y deja la seleccion a F1-15.
- La completa de ToolsaWeb depende de `RAMA_BASE` (`plan --completa`): la PoC lo comprueba en una copia; si no se puede, «no verificado» y pregunta abierta para F5-1.
- El aviso de un derivado fallido pasa desapercibido: lo contiene `doctor` (F3-8) con `roadmap --comparar`.
- `comprueba` no detecta en el propio PR una cita a una tarea inexistente: se acepta con D3a (lo pilla la siguiente transicion o el check `estado`).

## Plan de pruebas

PoC en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-012\` con `python -I -B`; copias de los ficheros de los pilotos, nunca ejecucion dentro de ellos; guion y log en `poc/`; carpeta borrada al terminar. Log determinista.

- **PoC A, cobertura**: tabla de las 14 practicas (con fichero:linea, que el guion comprueba que existen) frente al esquema propuesto; falla si alguna practica no tiene punto de extension o apunta a una clave que no esta en el esquema.
- **PoC B, presupuesto y esquema**: valida 2 configs candidatas (TW y ODT traducidas al esquema nuevo) y 4 hostiles (clave desconocida, ruta con `\v`, derivado que escribe fuera de `tareas/`, `comando` sin confianza); cuenta las claves frente a 12 y frente a las de la 0.16 (ventaja medible).
- **PoC C, derivado en la rama de estado**: repo temporal con rama huerfana montada en `tareas/`; 5 transiciones con un derivado `genera` (estilo `grafo-tareas`) ejecutado por un comando explicito; el arbol de `develop` queda identico (RR-11) y lo generado va en el commit de la transicion.
- **PoC D, lectura del estado**: el glob de `lector-tareas.py` (copiado) sobre la carpeta fija `TASK-NNN/` encuentra 0 tareas; el mismo calculo sobre un `tareas --json` simulado da el mismo resultado que hoy.
- **PoC E**: `clasificar-verificacion.py plan --completa` en una copia con `RAMA_BASE` en dos valores.

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio):

- **Criterio 1 (cubre lo que hacen a mano) → 2**: PoC A en verde con 14 de 14; PoC D.
- **Criterio 2 (F1-11 y requisitos) → 3**: PoC B (esquema, tope, confianza de F1-8 y ventaja en claves); PoC C (RF-70 y RR-11); PoC E (condicion de F1-7 sobre `RAMA_BASE`). Al cerrar, `bash .github/scripts/comprobar-repo.sh` en verde y `taskctl doctor` sin errores.

## Lo que necesita decision de una persona

Decisiones respondidas por la persona (Carlos) el 2026-11-14 con la consigna «responder la opcion recomendada a todas las preguntas», «no trocear» y la restriccion de comandos deterministas:

- **D1. Estados propios** → **(a)** sin clave: `descartada` y `pausa` nativos cubren los 11 de 11 casos medidos.
- **D2. Donde escriben los derivados** → **(a)** solo bajo `tareas/` (rama de estado).
- **D3. Comprobaciones que cruzan codigo y estado** → **(a)** `derivados` de tipo `comprueba` en cada transicion, mas un check `estado` no cacheado en el PR que lee `taskcode/tareas`.
- **D4. Forma de la configuracion** → **(a)** tres bloques: `verificacion`, `derivados`, `confianza_incluye`.
- **D5. Revisoras propias** → **(a)** skill del proyecto con bloque `rol: revisor` y `patrones_archivo`, sin clave en la configuracion; F1-15 decide la seleccion.
- **D6. «No se pudo verificar»** → **(a)** cualquier codigo distinto de 0 es rojo, como F1-7.
