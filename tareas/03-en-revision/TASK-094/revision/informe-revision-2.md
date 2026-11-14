# Informe de revision — TASK-094 (ronda 2)

- Commit revisado: bdc5b032d60f7aa0697c778e8398d62e2c9401a6 (HEAD al reproducir: 7d3f0ad, solo anade la peticion)
- Revisor: code-quality-reviewer
- Veredicto: aprobada-con-correcciones

Pase generico, verificacion acotada al diff de la correccion (283d19c). Profundidad: IMP-1, IMP-2 e IMP-3 a fondo y reproducidos; MEN-1..MEN-4 comprobados por lectura y por el log; F1-12, F1-2 y F1-4 contrastados solo en los puntos que el ADR cita.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-18-hito-de-decisiones.md:452 | plan |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-18-hito-de-decisiones.md:476 | implementacion |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-18-hito-de-decisiones.md:416 | plan |
| MEN-5 | MENOR | abierto | docs/adr/F1-18-hito-de-decisiones.md:580 | implementacion |
| MEN-6 | MENOR | abierto | docs/adr/F1-18-hito-de-decisiones.md:576 | implementacion |
| MEN-7 | MENOR | abierto | docs/adr/F1-18-hito-de-decisiones.md:642 | implementacion |

Sin hallazgos nuevos CRITICOS ni IMPORTANTES: nada de lo que introduce la correccion rompe F1-6, F1-10, F1-11 ni F1-12 (ver notas MENOR al final).

## Reproduccion de cada hallazgo

### IMP-1 — corregido (firma de la lista y de los descartes)

El ADR pasa a exigir una version firmada del registro (FI1, punto 5): `cerrado` solo si el registro actual es esa version, la lista no esta vacia, cada tarea esta en la lista firmada y lleva `hito: true`, y las descartadas tienen su motivo en `descartadas` firmado. La puerta usa la lista firmada (punto 7).

Reproducido sobre `hito_estado` de la PoC (ejecutada, no solo leida):
- Los tres supuestos de la ronda 1 salen `abierto`: lista cambiada a `TASK-001`, pendientes descartadas en su `tarea.md` sin motivo firmado, e ID de codigo (TASK-016) anadido a la lista; este ultimo no pasa la puerta (la PoC lo comprueba con `puerta(...)`).
- Ediciones maliciosas validas mias, distintas de las de la PoC, sobre un repo propio (registro firmado por persona; luego commit del agente): lista reordenada, `informe_hash` cambiado, y marca `hito: true` mas ID de codigo en la lista a la vez. Las tres salen `abierto` (`registro distinto de la version firmada`) y la puerta niega TASK-004 (codigo 3). La transicion legitima de una tarea del hito a `terminada` sin tocar el registro sale `cerrado`, como pide el diseno.
- Mutaciones de control: las 4 sobre `hito_estado` (`firma`, `marca`, `descartes`, `por`) las delata algun caso (2, 1, 1 y 1 casos). Las 14 copias mutadas del ADR fallan cada una con su motivo (14 `falla como se espera`, 0 `NO FALLA`).
- Limite que el propio ADR declara en «No verificado»: la identidad es el correo del autor del commit. Lo comprobe: un commit del agente con el correo de la persona y la lista `[TASK-001]` da `cerrado`. Esta anotado («en local cualquiera puede poner») y se delega en F1-6/F2-13, asi que no es hallazgo.

### IMP-2 — corregido (codigo de la negativa)

- ADR (punto 7, divergencia 11) y PoC (`poc-f1-18.py:53`, `COD_RECHAZADA = 3`) usan `rechazada` (3) del contrato de F1-4 y no fijan el 4. La PoC valida que los codigos observados son solo `{0, COD_RECHAZADA}` (`:817`); salida `[0, 3]` en el log.
- La afirmacion nueva de la divergencia 11 es cierta: `docs/adr/F1-12-actualizacion-entre-versiones.md:385-386` dice «**4** precondicion (tarea en diseno, en curso o en revision, o arbol sucio)» (en la ronda 1 la di por ausente porque busque «codigo 4» y no «**4**»; error mio, queda reconocido). F1-2:166 («start: SinRed (codigo 4)») y F1-4:155 y :390 (`sinred` (4)) usan 4 para SinRed. El choque del 4 queda pendiente de F2-3, anotado.
- Matiz: en F1-12 el 3 es «plan cambiado» (`:385`), otro significado que `rechazada` en F1-4; lo cubre la frase «encajan ... cuando F2 los fije». No bloquea.

### IMP-3 — corregido (quien crea el registro en `actualizar`)

Punto 3 y punto 6: `tc2 hito abrir --desde <propuesta>` crea las tareas con `hito: true` y escribe el registro en el mismo commit firmado; `tc2 hito no-necesario` crea el registro si no existe. Reproducido (R2, log): `1. migrado: ausente (start 0/50)`, `1b. agente escribe no-necesario: abierto`, `2. la persona abre el hito con la propuesta: abierto (5/50)`, `3. tareas terminadas: cerrado (50/0)`; y `2'. no-necesario sin registro previo: no-necesario (50/0)`. El flujo no se atasca y no toca la transaccion de F1-12 (CR1).

## Lo que reproduje

- **PoC completa** (`python -I -B ... poc-f1-18.py` con trabajo en `$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-094-r2/poc`): exit 0, unos 150 s en esta maquina, `RESULTADO: todo como se espera`. Tras quitar el CR de los finales de linea, el log es **identico** (diff vacio) al entregado en `tareas/03-en-revision/TASK-094/poc/poc-f1-18.log`.
- **MEN-2** comprobado: el log termina con `carpeta de trabajo borrada al terminar: si`; el guion la borra en un `finally`.
- **MEN-1, MEN-3 y MEN-4** (los comprueba quien coordina; solo lectura): MEN-1 dice ahora «4 huecos de `verificar.bat` (4 tareas con la 2.0 de hoy; 15 lineas en total)»; MEN-3 reformulado como «motivo que anade este ADR, medido»; MEN-4 anotado en «No verificado» (parada local de `start` solo medida sin carril declarado).
- **Suite del proyecto**: no aplica (el diff es un ADR y una PoC); la verificacion es la PoC.
- Carpeta temporal borrada; no se ejecuto nada dentro de OpenDockTrading (la PoC clona con `--no-hardlinks` a la carpeta temporal), ni GitHub, ni push, ni commit.

## Notas MENOR (fuera de la tabla, no abren ronda)

- **`tc2 hito descartar` y `abrir` podrian blanquear una edicion del agente.** El ADR no dice sobre que base reescribe el registro la persona. Reproducido con la PoC: el agente reduce la lista a `[TASK-001]` (`abierto`); la persona ejecuta `descartar` sobre el fichero tal como esta en la rama y firma el resultado; el estado pasa a `cerrado` (1 de 1). Cambio sugerido: que los comandos de persona partan de la version firmada, no del fichero actual, y muestren el diff antes de firmar. Lo introducen los comandos nuevos del punto 6, pero exige que la persona firme sin mirar un hito que estaba en rojo, por eso es MENOR.
- `tc2 hito abrir` con registro existente «sustituye la lista»: no dice si borra un `no_necesario` previo ni si conserva `descartadas`. Concretarlo en F2.
- El 3 de `rechazada` (F1-4) y el 3 de «plan cambiado» (F1-12) coinciden en numero; F2-3 deberia recibirlo como entrada explicita.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-094/poc/poc-f1-18.py` y `poc-f1-18.log`: para reproducir y ejecutar la PoC que pide el encargo.
- `docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md`, `F1-4-operaciones-git-y-contrato-de-salida.md`, `F1-6-identidades-agente-revisor-persona.md` y `F1-12-actualizacion-entre-versiones.md`: para contrastar los codigos de salida y la identidad de persona que cita el ADR.
- `tareas/03-en-revision/TASK-094/revision/informe-revision-1.md` y `peticion-revision-2.md`: el encargo los pide.

> Nota del orquestador: MEN-5 (descartar/abrir pueden blanquear una edicion del agente: partir de la version firmada y mostrar el diff), MEN-6 (que hace `abrir` con registro existente con `no_necesario` y `descartadas`) y MEN-7 (el 3 de `rechazada` coincide con el 3 de «plan cambiado» de F1-12) son las notas MENOR del revisor, pasadas a la tabla para corregirlas.
