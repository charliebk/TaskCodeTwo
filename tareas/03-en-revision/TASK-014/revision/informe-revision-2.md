# Informe de revision — TASK-014 (ronda 2)

- Commit revisado: ef826d85f6c32ba273cb3ffd54e9f1da27506f09 (correcciones en 4456a70; base de la ronda: 36828cc)
- Revisor: code-quality-reviewer
- Veredicto: aprobada-con-correcciones

Pase generico, incremental (verificacion acotada al diff de las correcciones). Profundidad: a fondo el guion (`POC`, guardas de `main`, `lanzadores`, `errores_prosa`, las 7 mutaciones nuevas), la decision nueva de no reverificar la cache y sus cifras; por encima el resto del ADR, que no se revisa de nuevo.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | tareas/03-en-revision/TASK-014/poc/poc-f1-14.py:69 | implementacion |
| MEN-6 | MENOR | abierto | docs/adr/F1-14-distribucion-e-instalacion.md:449 | implementacion |
| MEN-7 | MENOR | abierto | tareas/03-en-revision/TASK-014/poc/poc-f1-14.py:864 | implementacion |
| MEN-8 | MENOR | abierto | docs/adr/F1-14-distribucion-e-instalacion.md:233 | implementacion |
| MEN-9 | MENOR | abierto | docs/adr/F1-14-distribucion-e-instalacion.md:626 | implementacion |
| MEN-10 | MENOR | abierto | docs/adr/F1-14-distribucion-e-instalacion.md:463 | implementacion |

Los MEN-1 a MEN-5 de la ronda 1 no estaban en la tabla de la peticion (los comprueba quien coordina); aun asi los mire en el diff y quedan resueltos (ver «Notas»).

## Reproduccion

### IMP-1 — corregido

`POC = os.path.dirname(os.path.abspath(__file__))` (linea 69) sustituye a las cuatro rutas fijas `02-en-curso` (B, `git status`, X y D); `grep 02-en-curso` ya no da nada en el guion ni en el ADR. Reproducido:

- Ejecutado `python -I -B tareas/03-en-revision/TASK-014/poc/poc-f1-14.py <trabajo> <repo> <Personal>` con `<trabajo>` = `%LOCALAPPDATA%/pruebas-claude/taskcodetwo/task-014-r2` (sin mover nada, en el estado `03-en-revision`): compila los dos binarios, clona ToolsaWeb (`8bebfd049`) y OpenDockTrading (`8141a08a7`), `RESULTADO: todo como se espera`, exit 0, carpeta de trabajo borrada al terminar, `git status` del repo y de ToolsaWeb limpios.
- Comparado con `poc-f1-14.log` entregado (`diff` con CRLF normalizado): **la unica diferencia son los tiempos de reloj** (seccion T, los builds y las lineas `lanzador ...`); todo lo demas coincide linea a linea (tamanos 2720768 y 87430656, hashes, 404/0 y 96/0, firma, X, M, D, D1, G).
- Guarda nueva: copiando el guion fuera de `tareas/` y ejecutandolo sale con `ERROR: el guion (...) no esta dentro de tareas/ del repo ...`, sin tocar nada (la carpeta de trabajo no se crea). Mensaje claro, ya no hay `NotADirectoryError` mudo.
- Mutaciones del guion: **las 21 fallan como se espera, 0 `NO FALLA`** (14 de antes mas `prosa-mib`, `prosa-veces`, `prosa-repos`, `prosa-lineas`, `prosa-bytes`, `prosa-criterios`, `prosa-duplicada`).
- Mutaciones propias (importando `errores_adr` y tomando de la propia tabla del ADR las medidas, sin tocar el repo): `87 430 656` → `87 430 657` en la prosa **se detecta** (`prosa: «2 720 768 frente a 87 430 657 bytes» y lo medido es ...`); `15,6 MiB` → `16,6 MiB` **se detecta**; ver MEN-7 y MEN-8 en «Notas» para las que no.
- Tiempos de mi pasada (reloj, una pasada): Rust caliente 48 / 357 / 126 ms frente a TypeScript 135 / 714 / 316 (vacio / ToolsaWeb / OpenDockTrading), frio Rust 86 / 472 / 159 frente a 1165 / 1785 / 1346; diferencia en vacio 87 ms (el ADR dice «unos 85»). Lanzadores sobre Rust: directo 56, `tc2.cmd` +46, sh +48, sh con `stat` +85, sh con `sha256sum` +119, `.ps1` +314 (el ADR: 38 / 47 / 84 / 127 / 304 de la pasada del log). Misma ordenacion y mismos ordenes de magnitud; las cifras del ADR son de una pasada y el ADR las llama «de reloj»: no hay discrepancia.

### MEN-6 — El ADR se contradice sobre si se comprueba el hash antes de ejecutar (lo abre la correccion)

La correccion anade el punto «binario ya en la cache: no se reverifica en cada llamada» (lineas 450-463), pero deja intactas dos frases que dicen lo contrario:

- linea 449: «nunca ejecuta un binario sin comprobar», en el mismo punto del contrato (el primer uso), justo encima de la excepcion nueva: un binario de la cache **si se ejecuta sin comprobar en ese momento**;
- linea 360 («Como se ve en GitHub»): «los instaladores de una linea y el lanzador del plugin comprueban el sha256 de `cli.lock` antes de ejecutar» (sin matiz: se lee como cada llamada).

Quien implemente F2-11/F2-12 desde el texto puede leer cualquiera de las dos cosas. Arreglo: «nunca ejecuta un binario descargado sin comprobar» (449) y «antes de ejecutar lo descargado» (360). MENOR porque la decision esta clara y bien argumentada en su punto; contradice solo dos frases.

## La decision de no reverificar la cache contra `cli.lock` (revision critica)

Conclusion: **coherente con F1-8 y aceptable para la seguridad de la distribucion**, con cuatro matices que no abren hallazgo.

- Coherencia con F1-8: la zona Z1 («Se confia en: el CLI y lo que la persona confio»; el agente corre en la misma sesion, RN-06) y la matriz (A5: el agente del mismo usuario saca el token del llavero) ya aceptan que un proceso del mismo usuario puede cambiar lo que el CLI ejecuta. Quien reescribe `cli/<version>/` puede reescribir `bin/tc2` o `cli.lock` del plugin; reverificar desde el lanzador no sube el listón frente a ese atacante. F1-8 no tiene activo para el binario en cache, y el ADR no lo propone: no es contradiccion, pero F1-8 tampoco lo cubre (ver nota MEN-9).
- La ventaja que protege el primer criterio esta bien cuantificada para la ruta sh de Git for Windows (+85 ms con `stat`, +119/127 con `sha256sum`, frente a ~87 ms de diferencia Rust/TypeScript en vacio): se reproduce.
- Matiz 1: el argumento «se comeria la ventaja que decidio el criterio 1» es impreciso, porque un lanzador va delante de **los dos** candidatos (TypeScript con SEA tambien lo necesita) y el margen relativo no cambia; lo que consume es el presupuesto absoluto de la status line (RR-19). Hay que leerlo asi.
- Matiz 2: «la corrupcion accidental la detecta `doctor`» presupone que `tc2 doctor --recursos` se ejecuta con un binario sano; si la cache esta corrupta, quien comprueba el hash es el lanzador o el comando de la propia cache, y el ADR no dice cual. Un binario corrupto que no arranca falla a la vista; uno manipulado puede mentir a su propio `doctor`. Es honesto («no protege de nada nuevo»), pero conviene que el contrato diga que el hash lo calcula el lanzador (o `tc2 actualizar --cli` desde otro binario), no el binario comprobado.
- Matiz 3: las medidas de `stat`/`sha256sum` son del sh de Git for Windows; un lanzador `.cmd` que hiciera la comprobacion (con `certutil` o PowerShell) no se midio. La conclusion no cambia (el `.ps1` ya cuesta 300 ms), pero el ADR la generaliza algo.
- Matiz 4: la cache por version (`cli/<version>/`) sustituye bien al hash por llamada para el caso real (cambio de version en `cli.lock`), y queda declarado en «No verificado» que el coste del lanzador en el camino real de Claude Code no esta medido.

## Notas (no abren ronda; segun las reglas de esta ronda)

- **MEN-1 a MEN-5 de la ronda 1, verificados en el diff**: MEN-1 (winget: «documento de 2021-05-22, version 1.0; pagina actualizada el 2026-08-30»; Azure: lista completa de paises de organizaciones) corregido; MEN-3 (coste del lanzador medido con 5 lanzadores en la seccion T, reverificacion fijada en el contrato, «No verificado» ampliado) corregido; MEN-4 (la fila L1 y la fila del indice `docs/adr/README.md:23` ya dicen «sin el 4.º la regla no decide»; criterios 4 y 5 etiquetados «juicio, no medida» en la tabla; criterio 5 cita el fallo TLS de Rust y las 3 iteraciones extra de TypeScript, 147 frente a 215 coincide con la seccion D y con `NOTAS.md`; `useCodeCache`/`useSnapshot` en «No verificado») corregido; MEN-5 (lista de red con `openssl-probe`, `cargo build` sin `--offline`) corregido; MEN-2 corregido salvo lo de MEN-7/MEN-8.
- **MEN-7 (MENOR)** — `errores_prosa` valida los MiB de la prosa por pertenencia a un conjunto (`mib_ok`), no por posicion. Mutacion propia: `15,6 MiB` → `83,4 MiB` en D3b (otro MiB medido, el del `.exe` de TypeScript): **no se detecta**. Lo mismo valdria intercambiar `15,6` y `500,3`. El ADR («los MiB») sobrestima algo el alcance; basta cambiar la frase a «que cada MiB de la prosa sea una cifra medida» o comprobar el contexto.
- **MEN-8 (MENOR)** — las cifras de reloj de la prosa (`unos 85 ms`, `47`, `84`, `127`, `38`, `304`) no estan vigiladas; es coherente con el diseno (el reloj no entra en la tabla) y lo dice el ADR; mutar `85` → `95` no se detecta, como se espera. Solo se anota.
- **MEN-9 (MENOR)** — la tabla de activos de F1-8 no tiene el binario del CLI en cache ni `cli.lock` como activo; el ADR, que si argumenta contra Z1, podria listar en «Consecuencias» una fila propuesta para F1-8 (activo: binario en cache, zona Z1, mitigacion: descarga comprobada y cache por version, riesgo residual: mismo usuario), como ya hace con otros cambios al plan.
- Los hallazgos MEN-6 a MEN-9 son las unicas cosas que cambiaria; ninguna toca los datos ni el ganador de D1a. No encontre un fallo nuevo en el codigo de `lanzadores` (limpia sus ficheros bajo la carpeta de trabajo, comprueba `returncode` de cada medicion y se salta sh si no hay Git for Windows) ni en las guardas de `main`.
- Suite: la tarea no tiene suite de proyecto; la «puerta» es el guion de la PoC, ejecutado entero (exit 0).

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-014/poc/` (guion, log) y `revision/informe-revision-1.md`: reproducir la PoC y los hallazgos de la ronda 1, lo pedia el encargo.
- `docs/adr/F1-8-seguridad-modelo-de-amenazas.md` (zonas Z1-Z5, matriz de activos A1-A16, huella de D4b): contrastar la coherencia de la decision de no reverificar la cache, pedido en el encargo.
- Clones de ToolsaWeb y OpenDockTrading hechos por el propio guion en la carpeta temporal (solo lectura); los originales no se tocaron.

> Nota del orquestador: MEN-7 (MiB de la prosa por pertenencia, no por posicion), MEN-8 (cifras de reloj de la prosa sin vigilar: marcarlas como de reloj o rango), MEN-9 (proponer el binario en cache y `cli.lock` como activo de F1-8 en «Consecuencias») y MEN-10 (precisar que el hash lo calcula el lanzador o un proceso que no sea el binario sospechoso, y el matiz del presupuesto absoluto de la status line) son notas del revisor pasadas a la tabla para corregirlas.
