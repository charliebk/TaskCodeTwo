# Informe de revision — TASK-012 (ronda 3)

- Commit revisado: 72901f6338a1ace9a6f84d7bdbc379517af85c28
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:590 | implementacion |
| IMPORTANTE-2 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:531 | implementacion |
| IMPORTANTE-3 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:600 | implementacion |
| IMPORTANTE-4 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:629 | implementacion |
| IMPORTANTE-5 | IMPORTANTE | corregido | docs/adr/F1-11-extensiones-declaradas.md:620 | plan |
| MEN-1 | MENOR | abierto | docs/adr/F1-11-extensiones-declaradas.md:508 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Sin hallazgos nuevos. Ni CRITICO ni IMPORTANTE introducido por la correccion de IMPORTANTE-5.

Veredicto propuesto: **aprobada** (la linea `- Veredicto:` queda sin cambiar, como pide la peticion).

## Reproduccion de IMPORTANTE-5 (corregido)

- Fallo de la ronda 2: el check `estado` leia el comprobador del arbol del PR. Un PR que cambiaba el script para dejarlo en verde pasaba el check.
- Texto corregido: `docs/adr/F1-11-extensiones-declaradas.md:620-629`. El check toma `derivados` y los scripts de los `comprueba` de la rama base, los ejecuta con cwd en una copia de la base y pasa `TASKCODE_CODIGO` con el arbol del PR como datos.
- Mutacion propia (carpeta temporal, `python -I`, sin tocar el repo):
  - A. Comprobador de la base sobre el arbol del PR con `TASK-404` citada: codigo 1 (rojo).
  - B. PR que quita el comprobador de su config: el check lee la config de la base, sigue en rojo (codigo 1).
  - C. Comprobador que solo existe en el PR: no se ejecuta, porque el check no lee nada del PR. Si la base lo cita y falta el fichero, sale codigo 2 (rojo, no verde).
  - D. PR que reescribe `scripts/c1.py` para devolver 0: el check ejecuta la copia de la base, codigo 1 (rojo).
- La PoC C del ADR tambien lo cubre: `check estado de un PR que cambia el comprobador: rojo (bloquea)`, y el caso «lo vulnerable» sale verde, como debe, para demostrar que el caso es real.

## Verificacion realizada

- PoC: `python -I -B tareas/03-en-revision/TASK-012/poc/poc-f1-11.py <trabajo> docs/adr/F1-11-extensiones-declaradas.md <Personal> <plugin 0.16.1>`, con trabajo en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-012-r3`. Salida 0 y «todo como se espera: A, A2, B, C, C2, D, E, la tabla de ventaja y el resumen». Esta ejecucion cubre IMPORTANTE-1 (C2), IMPORTANTE-2 (B, normalizacion de `escribe`), IMPORTANTE-3 (D) e IMPORTANTE-4 (tabla de ventaja y A). Son los chequeos de la PoC, no una lectura linea a linea de cada hallazgo. Resultado de C: transiciones 11/11, `develop` identico, check `estado` rojo y verde, PR que cambia el comprobador rojo.
- Puerta: `bash .github/scripts/comprobar-repo.sh` -> `Tareas: 93; enlaces relativos: 92; fallos: 0`, salida 0.
- Rechazo de `escribe`: coherente con lo probado. Las uniones de Windows estan probadas (T11: rechazadas sin ejecutar). Los enlaces simbolicos aparecen como «no verificado» (WinError 1314), igual que en el ADR. El ADR no afirma nada mas.
- Mutacion del plan (un `comprueba` que no existe en la base, o que se borra en el PR): ver «Reproduccion de IMPORTANTE-5», casos B y C. Ninguna via nueva de bloqueo: el check solo puede ir a rojo por un comprobador de la base, no por algo que el PR añada.
- Carpeta temporal `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-012-r3`: borrada al terminar. No se hizo ningun commit. Solo se modifico este informe.
- GitHub: no se ha llamado. ToolsaWeb y OpenDockTrading: no se han leido en esta ronda.

## Notas (no abren ronda)

- Un derivado confiado podria crear un enlace dentro de `tareas/` durante la ejecucion y escribir a traves de el. La comprobacion del ADR de `escribe` es estatica (antes de ejecutar); despues, la comparacion por contenido cubre la rama de estado y el codigo. No se ha probado el caso de enlace creado en tiempo de ejecucion. Queda como no verificado; lo tomaria F2 o la PoC siguiente.
- El check `estado` de la PoC lee `develop` local; el ADR dice `origin/develop` para CI, y ya figura como F5-1 en «No verificado».

## Lecturas fuera del contexto permitido

- tareas/03-en-revision/TASK-012/revision/informe-revision-2.md: la peticion lo cita para comprobar IMPORTANTE-5 y los hallazgos de la ronda anterior.
- tareas/03-en-revision/TASK-012/poc/poc-f1-11.py: leido en parte (lineas 1-844 y 1088-1127) para entender el check `estado` y el validador antes de ejecutarlo. Ejecutado tal cual, sin modificar.
- Salida de la PoC (`poc.log`) y de la mutacion: ficheros temporales en la carpeta de trabajo, borrados al final.

### MEN-1 — Nota de la revision 3 registrada como MENOR por quien orquesta

No esta dicho en «No verificado» que un derivado pueda crear un enlace simbolico o una union dentro de `tareas/` durante su propia ejecucion (la comprobacion de `escribe` mira las rutas declaradas antes y lo escrito despues). Correccion: anadirlo a la entrada de enlaces de «No verificado», con la comprobacion prevista.
