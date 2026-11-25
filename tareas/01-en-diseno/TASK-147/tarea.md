---
id: TASK-147
titulo: "Auditoria 4D: A10 scripts de CI e instaladores (4 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: true
rama: fix/task-147-auditoria-4d-a10-scripts-de-ci-e-instala
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 496824
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-25
dependencias: []
---
## Objetivo

Corregir los 4 casos IMPORTANTE del grupo A10 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 4 Disfraces en scripts y workflows que verifican o instalan. Tres están reproducidos con un arnés y uno (A10-05) queda NO VERIFICADO hasta reproducirlo. No se piden los MENOR (A10-03, A10-04). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [ ] A10-01: la guarda de `.github/scripts/probar-actualizar.sh:60` para cuando los dos `SHA256SUMS` son iguales (`! cmp -s ... || rojo "..."`). El arnés de la ficha (`tareas/04-terminadas/TASK-138/seguimiento/arneses/a10/d4-cmp.sh`: dos ficheros iguales con `set -eu`), cambiando su copia de la línea 60 por la forma corregida, termina con un código distinto de 0.
- [ ] A11-01: la guarda de `.github/workflows/release.yml:284` (`! cmp -s Cargo.toml Cargo.toml.orig`, en mitad del paso «vY») para el paso cuando el `sed` de `:283` no cambia la versión (`... || { echo "::error::..."; exit 1; }`). El arnés de la ficha (`tareas/04-terminadas/TASK-138/seguimiento/arneses/a11/d4-release-284.sh`, con las líneas 281-285 corregidas en su `release-281-285.txt` y con `bash --noprofile --norc -eo pipefail`) deja de imprimir `SIGUE` y sale con un código distinto de 0. Mismo patrón que A10-01: se corrigen juntos.
- [ ] A10-02: con `codigo` 0, `sin_version` (`.github/scripts/probar-instalador.sh:268-287`) exige el binario instalado y que su `--version` sea la del tag de `/releases/latest`. El arnés de la ficha (`tareas/04-terminadas/TASK-138/seguimiento/arneses/a10/d4-sin-version.sh`: instalador que sale 0 sin instalar), sobre el script corregido, da rojo, y `linea_unix` (`:45-51`) no esconde un fallo de `curl` detrás del 0 de `sh`.
- [ ] A10-05: primero se reproduce, sin red externa, con un servidor en 127.0.0.1 que responde 302 a una URL `http://` y con `instalar/install.sh` sin `curl` en el `PATH`. Si wget sigue la redirección, la rama `wget` de `descargar` (`instalar/install.sh:47-56`) la rechaza, o el instalador exige `curl` y lo dice en el arreglo, y la reproducción queda como prueba. Si wget no la sigue, el caso se cierra como descartado y se anota en `docs/auditoria-4d.md`.
- [ ] El job que usa cada script (`release.yml`) sigue en verde en la rama, y el `## Resultado` lista los 4 IDs con su commit.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-25T14:53:54Z | plan | automatico | persona | 0.17.1 |
| 2026-11-25T14:59:20Z | approve | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-25T14:59:19Z | diseno | agente:a85ef397f5f461ba9 | 22 | 62171 | 419494 | 15137 | 25 | 496824 | - | - |
