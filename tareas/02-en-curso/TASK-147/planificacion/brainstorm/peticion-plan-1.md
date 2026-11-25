# Peticion de redaccion del plan — TASK-147 (ronda 1)

- Tarea: TASK-147 — Auditoria 4D: A10 scripts de CI e instaladores (4 IMPORTANTE)
- Tipo: fix · Complejidad declarada: no declarada (decide la heuristica)
- Ronda: 1
- Fecha: 2026-11-25
- Rol: `brainstorm-arquitectura` — lanzalo con el agente de ese mismo nombre
- Quien orquesta vuelca tu respuesta en: `../plan-final.md`

**Esta tarea se planifica con un solo rol y no hay unificador**: tu respuesta ES el plan final. Redactala con las secciones de abajo; quien orquesta la vuelca tal cual en `plan-final.md` (tu no escribes ficheros), sin salida intermedia que consolidar.

## Tu pregunta

> Dado lo que ya existe, ¿donde encaja este cambio y que forma tiene?

## Que miras

- Los modulos y ficheros que ya resuelven algo parecido, para extenderlos en vez de duplicarlos.
- Que se crea nuevo, que se extiende y en que orden se construye.
- Los limites que el cambio cruza: contratos publicos, formatos de fichero, esquemas.
- El precedente interno mas cercano: como se resolvio la ultima vez un problema de esta forma.

## Que NO miras

Que lo mires solo tu no te autoriza a cubrirlo todo: lo que queda fuera de tu pregunta no se mira, pero se DICE. Anota en el plan lo que no cubres, para que quien lo apruebe sepa que ese lado no se ha revisado:

- Como se prueba (es del rol de testing).
- Por donde se rompe (es del rol de riesgos).
- Las reglas de negocio (son del rol de dominio).

## Complejidad: declarada frente a heuristica

- Declarada en la tarea: **no declarada (decide la heuristica)**
- Heuristica (1 puntos): **simple**
- Senales encontradas:
  - criterios_aceptacion: 5 criterios (umbral: 5) (+1)

Los dos niveles coinciden. Se lanzan 1 rol.

## Que tiene que traer el plan final

- Enfoque propuesto, concreto: que se crea, que se extiende, en que orden.
- Lo que el rol no cubrio, contrastado contra los criterios de aceptacion.
- Reutilizacion (Regla 4D): lo que ya existe en el repo y se va a usar (helpers, funciones, fixtures, tests), buscado ANTES de proponer nada nuevo; y lo que se crea nuevo, con el motivo de no reutilizar lo que hay. Duplicar un helper que ya existe es un hallazgo IMPORTANTE.
- Riesgos aceptados y que los contiene.
- Plan de pruebas.
- Lo que necesita decision de una persona antes de implementar. **El checkpoint humano es obligatorio**: este plan no vale hasta que alguien lo apruebe con `taskctl approve`.

## Reglas

- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es el plan.
- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.

## Enunciado de la tarea

### Objetivo

````
Corregir los 4 casos IMPORTANTE del grupo A10 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 4 Disfraces en scripts y workflows que verifican o instalan. Tres están reproducidos con un arnés y uno (A10-05) queda NO VERIFICADO hasta reproducirlo. No se piden los MENOR (A10-03, A10-04). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.
````

### Criterios de aceptacion

````
A10-01: la guarda de `.github/scripts/probar-actualizar.sh:60` para cuando los dos `SHA256SUMS` son iguales (`! cmp -s ... || rojo "..."`). El arnés de la ficha (`tareas/04-terminadas/TASK-138/seguimiento/arneses/a10/d4-cmp.sh`: dos ficheros iguales con `set -eu`), cambiando su copia de la línea 60 por la forma corregida, termina con un código distinto de 0.
A11-01: la guarda de `.github/workflows/release.yml:284` (`! cmp -s Cargo.toml Cargo.toml.orig`, en mitad del paso «vY») para el paso cuando el `sed` de `:283` no cambia la versión (`... || { echo "::error::..."; exit 1; }`). El arnés de la ficha (`tareas/04-terminadas/TASK-138/seguimiento/arneses/a11/d4-release-284.sh`, con las líneas 281-285 corregidas en su `release-281-285.txt` y con `bash --noprofile --norc -eo pipefail`) deja de imprimir `SIGUE` y sale con un código distinto de 0. Mismo patrón que A10-01: se corrigen juntos.
A10-02: con `codigo` 0, `sin_version` (`.github/scripts/probar-instalador.sh:268-287`) exige el binario instalado y que su `--version` sea la del tag de `/releases/latest`. El arnés de la ficha (`tareas/04-terminadas/TASK-138/seguimiento/arneses/a10/d4-sin-version.sh`: instalador que sale 0 sin instalar), sobre el script corregido, da rojo, y `linea_unix` (`:45-51`) no esconde un fallo de `curl` detrás del 0 de `sh`.
A10-05: primero se reproduce, sin red externa, con un servidor en 127.0.0.1 que responde 302 a una URL `http://` y con `instalar/install.sh` sin `curl` en el `PATH`. Si wget sigue la redirección, la rama `wget` de `descargar` (`instalar/install.sh:47-56`) la rechaza, o el instalador exige `curl` y lo dice en el arreglo, y la reproducción queda como prueba. Si wget no la sigue, el caso se cierra como descartado y se anota en `docs/auditoria-4d.md`.
El job que usa cada script (`release.yml`) sigue en verde en la rama, y el `## Resultado` lista los 4 IDs con su commit.
````
