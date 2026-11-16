# Brainstorm — TASK-095, rol riesgos (ronda 1)

- Rol: `brainstorm-riesgos`
- Agente: taskcode-plugin:brainstorm-riesgos (ronda 1)

## Modos de fallo, ordenados por gravedad, con el escenario concreto de cada uno

- **Fuga irreversible por el conversor de engram (opción A)**. Escenario: `~/.engram/engram.db` es una sola base por máquina con varios proyectos. Si se elige el proyecto por `sessions.directory` y las rutas no casan (`C:\` frente a `/c/`, mayúsculas en Windows, clones en rutas distintas), observaciones de otro proyecto o secretos que el filtro no detecta acaban en `memoria.md`, y `tc2` los publica en la rama de estado (en F1-2 el CLI hace push directo de esa rama). Consecuencia: el secreto queda en el remoto para siempre. Mitigación: `--simular` obligatorio antes de la primera escritura, comparar rutas ya normalizadas, y nada sin `TASK-NNN` ni sin proyecto resuelto sin ambigüedad entra en git sin que lo confirme una persona.
- **Datos viejos sin ningún aviso tras un merge en la plataforma**. Escenario: el §3.2 de la AMPLIACION compara con `git rev-parse` y no menciona `fetch`. Un merge-pr en GitHub no mueve el `develop` local ni `origin/develop` hasta el siguiente fetch, así que el índice dice «iguales» y contesta con datos viejos. Esto rompe RR-20 en el caso exacto para el que existe.
- **«Mismo hash» falso o imposible**. Escenario: los bytes del `.db` de SQLite no son deterministas (orden de páginas, segmentos internos de FTS5, VACUUM). Además, si el conversor lee `tarea.md` del árbol de trabajo, `core.autocrlf` y las ediciones sin commitear cambian el contenido según la máquina. Mitigación: calcular el hash sobre un volcado canónico ordenado y leer siempre los blobs de git (`git cat-file`), nunca el árbol de trabajo.
- **Etiquetas de IA no reproducibles**. Escenario: si la capa léxica o la de IA se calculan al reindexar, la reconstrucción da otras etiquetas y el hash cambia. Mitigación: el índice solo lee etiquetas ya escritas en git (frontmatter con su `origen`). Las capas 2 y 3 escriben en `finish`, nunca en `reconstruir`.
- **Bloqueo o espera entre búsquedas y transacciones**. Escenario: `buscar` ve el índice desfasado y quiere el cerrojo de `tc2` (`.git/taskcode-estado.lock`, espera máxima de 60 s según F1-2) mientras un `finish` lo tiene; o `finish` reindexa dentro de su transacción y vuelve a pedir el mismo cerrojo no reentrante. Mitigación: cerrojo propio del índice, separado del de estado; mientras alguien reindexa, la búsqueda lee la versión anterior marcada como «desfasada».
- **Historia reescrita o SHA podado**. Escenario: tras un force-push y un `gc`, el SHA guardado ya no existe y `merge-base --is-ancestor` sale con error. Mitigación: cualquier fallo al resolver el SHA viejo lleva a la reconstrucción completa.
- **Atribución de commits incompleta**. Escenario: commits de `develop` sin `tipo(TASK-NNN)` (`docs: ROADMAP regenerado…`, `chore: alta de 18 tareas…`), commits que citan varias tareas y renombrados entre carpetas de estado de la 0.16. Mitigación: los huérfanos van a una categoría explícita y se cuentan en `indice estado`.

## Estados intermedios y fallos parciales

- Reindexado cortado a mitad: el §3.2.4 ya escribe los SHA en la misma transacción. Debe valer también para la reconstrucción completa, que nunca puede dejar el índice vacío con los SHA ya avanzados.
- Reconstrucción por fichero nuevo más `rename` (supuesto): en Windows, `rename` sobre un `.db` abierto por otro proceso falla. Decidir si se reconstruye en sitio dentro de una transacción o se da por fallida.
- Ubicación `.git/taskcodetwo/`: en un worktree enlazado `.git` es un fichero. Usar `git rev-parse --git-common-dir`.
- `finish` con el merge hecho pero muerto antes de escribir `## Archivos tocados` y etiquetas: el índice tiene que poder sacarlos de git sin esa sección.
- Conversor de engram cortado entre commit local y push: la idempotencia por `sync_id` tiene que contar también lo pendiente.
- Copiar `engram.db` sin su `-wal` (supuesto): usar la API de backup o `VACUUM INTO`, no copiar el fichero.

## Compatibilidad hacia atras

- Las 94 `tarea.md` con `etiquetas: []` y el formato de carpetas por estado de la 0.16 frente a la carpeta fija de F1-1: el conversor tiene que leer los dos y seguir los renombrados.
- Añadir `## Archivos tocados` y etiquetas a `tarea.md` mientras `taskctl` 0.16 sigue en uso: no comprobado que la 0.16 tolere secciones desconocidas ni que no reescriba el frontmatter perdiendo campos.
- Mientras las instrucciones del MCP de engram sigan pidiendo `mem_save`, se seguirá escribiendo en engram tras importar: la retirada tiene que cortar esa escritura o el conversor ser incremental indefinidamente.
- Cambios de esquema del índice: versión de esquema en `indice_estado`; si no coincide, se reconstruye.

## Vuelta atras

- Índice: se borra y se reconstruye. Reversible del todo.
- Etiquetas y `memoria.md` en la rama de estado: revertibles con un commit, pero siguen en la historia del remoto. **Sin retorno** si contienen secretos o datos de otro proyecto.
- Quitar engram del manifiesto (F1-14): reversible; pero si se retira antes de F10-13 se pierde la base de comparación de la medición.
- Vocabulario: renombrar o quitar etiquetas deja frontmatter huérfano. Alias, nunca borrar.

## El riesgo que mas te preocupa (UNO solo)

La regla de frescura tal como está escrita (`git rev-parse` del `develop` y la rama de estado locales) **no ve los merges hechos en la plataforma sin un `fetch`**. Sin hooks, nada hace ese fetch. Si se hace fetch en cada lectura, choca con los 200 ms de F10-6, con el trabajo sin red y con los avisos de credenciales. Si no se hace, el criterio 2 solo pasa si quien hace la PoC lanza el fetch a mano: falso verde.

Desacuerdo con arquitectura si adopta el §3.2 tal cual: el ADR tiene que decidir de forma explícita cuándo se hace fetch (en cada lectura con tiempo límite, o con un TTL) y qué responde `buscar` sin red. Posición: responder con la marca «indexado hasta SHA X, remoto no consultado», y no fingir que está al día.
