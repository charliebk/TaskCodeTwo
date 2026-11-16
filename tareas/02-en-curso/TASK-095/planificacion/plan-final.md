# Plan — TASK-095: F1-19 ADR memoria del proyecto: indice de contexto, etiquetas y busqueda

(Lo consolida el agente unificador a partir de 2 roles de brainstorm lanzados en paralelo: arquitectura, riesgos; 2 y no mas porque la heuristica da complejidad media por 4 dependencias. Los desacuerdos se senalan, no se promedian.)

## Enfoque propuesto

Duracion estimada: 140 min (de la rama abierta al cierre con merge; senales: 0 ficheros de codigo en el repo, 1 ADR con 7 apartados de F1-19, 2 PoC minimas desechables fuera del repo, 3 criterios, revision por pares con reproduccion)

- Se crea `docs/adr/F1-19-memoria-del-proyecto.md` con alternativas, decision y motivo por apartado: verdad en git, ubicacion, frescura, esquema, vocabulario y tres capas, conversor engram A/B, retirada de engram (arquitectura, riesgos).
- Forma que fija el ADR para F10-1 (arquitectura): modulo `crates/tc2/src/indice/`, fuente siempre objetos de git sobre un SHA (`cat-file --batch`, nunca el arbol de trabajo; tambien riesgos), huella SHA-256 de volcado canonico (no bytes del `.db`; tambien riesgos), cerrojo propio `taskcode-indice.lock`, reindexado en una transaccion con los SHA.
- PoC 1 (criterio 1) y PoC 2 (criterio 2), minimas, en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-095\`; ningun codigo de la 2.0 en el repo (Fase 1). Orden: ADR borrador → PoC 1 → PoC 2 y medicion → ADR final.
- Retirada de engram: el ADR la condiciona a F10-13 y no la ejecuta (arquitectura; riesgos coincide: retirarla antes pierde la base de comparacion).

## Troceo

### Parte a: ADR nucleo y PoC de huella (criterio 1) — unos 60 min
### Parte b: politica de frescura, PoC con remoto bare y medicion de `ls-remote` (criterio 2) — unos 45 min
### Parte c: etiquetado en tres capas, conversor engram A/B y retirada del manifiesto (criterio 3, sin PoC de codigo) — unos 35 min
(Propuesta del unificador.) Decision de Carlos (2026-11-16): NO se trocea; se hace como una sola tarea de unos 140 min. Motivo: decision de la persona al planificar, por ser un unico ADR cuyas tres partes se condicionan entre si (la frescura y el etiquetado fijan el esquema que mide la huella).

## Desacuerdos entre roles, y como se resuelven

- Frescura frente a merges en la plataforma — arquitectura: `ls-remote origin develop taskcode/tareas` en cada lectura, `fetch` dirigido si difiere, sin TTL, sin red marca `fresco=no` / riesgos: el ADR debe elegir entre cada lectura con tiempo limite o TTL; sin red, «indexado hasta SHA X, remoto no consultado» — NO se decide aqui: va a decisiones abiertas (1). Ambos coinciden en el diagnostico: `rev-parse` local no ve el merge sin `fetch`.
- Busqueda con el cerrojo del indice ocupado — arquitectura: reindexa en la lectura bajo cerrojo propio antes de responder (no fija que pasa si otro lo tiene) / riesgos: leer la version anterior marcada «desfasada» — gana riesgos: es la unica posicion que cubre el caso y usa la misma marca que el caso sin red; choca con la letra de RR-20, ver decision abierta (1).
- Escritura del conversor engram (opcion A) — arquitectura: una `Operacion` con el patron de `alta`, que publica en la rama de estado / riesgos: `--simular` obligatorio, rutas normalizadas y confirmacion humana de todo lo ambiguo antes de entrar en git — gana riesgos: un secreto empujado al remoto no tiene vuelta atras; A o B sigue siendo de Carlos (2).
- Cuando se calculan las etiquetas — arquitectura: capas determinista y lexica como funciones puras de `tc2`, sin fijar si corren al reindexar / riesgos: el indice solo lee etiquetas ya escritas en git; capas 2 y 3 escriben en `finish`, nunca en `reconstruir` — gana riesgos: es la unica posicion que garantiza «mismo SHA, misma huella» y la unica explicita.

## Riesgos aceptados y que los contiene

- Fuga irreversible de secretos u otro proyecto via `memoria.md` — riesgos — `--simular`, filtro de secretos, rutas normalizadas, confirmacion humana; o la opcion B si Carlos no acepta el riesgo.
- `ls-remote` en cada lectura contra los 200 ms de F10-6 — arquitectura y riesgos — medicion real contra GitHub desde Windows en la PoC 2; cifra en el ADR.
- Commits sin `tipo(TASK-NNN)`, multitarea o renombrados entre carpetas 0.16 — riesgos — categoria explicita de huerfanos contada en `indice estado`.
- Historia reescrita o SHA podado — riesgos — cualquier fallo al resolver el SHA viejo lleva a reconstruccion completa (tambien arquitectura, via `es_ancestro`).
- El MCP de engram sigue pidiendo `mem_save` tras importar — riesgos — conversor incremental por `sync_id` hasta la retirada en F10-13.
- Un proceso `git` por blob (404 en ToolsaWeb) — arquitectura — `Subcomando::CatFile` con `--batch` y `Log` en flujo; el ADR lo decide.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):
- Criterio 1, 3 tests (pesado el primero) — interfaz de la PoC: `tc2 indice reconstruir --ref develop --prefijo tareas/04-terminadas/` dos veces (borrando el `.db` entre medias) y `tc2 indice estado --huella` tras cada una: misma huella; sobre los blobs de `develop` a un SHA fijado, no el arbol de trabajo — arquitectura y riesgos.
  - (ligero) arbol de trabajo sucio y `core.autocrlf` distinto no cambian la huella — riesgos. (ligero) cambiar una tarea en un commit si cambia la huella — control contra falso verde, unificador.
- Criterio 2, 3 tests (pesados los dos primeros, procesos git reales) — remoto bare local `origen.git` como plataforma; clon A con indice y `core.hooksPath` vacio; clon B crea rama, `merge --no-ff` en `develop` y push al bare; A ejecuta `tc2 buscar <termino solo de esa tarea>` sin `fetch` manual: la encuentra y `indice estado` muestra el SHA nuevo — arquitectura y riesgos.
  - (pesado) force-push en el bare → reconstruccion completa, no error — riesgos. (ligero) bare inaccesible → respuesta con la marca de remoto no consultado y la hora — arquitectura y riesgos.
  - Medicion aparte, no test: latencia de `ls-remote` contra GitHub desde Windows, cifras exactas en el ADR — arquitectura.
- Criterio 3, 1 test (ligero) — revision del ADR contra los 7 apartados de F1-19 y RR-20, RF-77..RF-84, sin codigo; mas la compilacion de `rusqlite` `bundled` con FTS5 que pide arquitectura, en la CI de F1-7 si la PoC lo permite.

## Sin cubrir

- Grafo y visor 3D (RF-83, RF-84, dockview): ningun rol los trato; el ADR los deja fuera de F1-19 y remite a F10-12..F10-17.
- Capa provisional de ramas en curso (§3.2 de la AMPLIACION) y umbrales de `tc2 tags doctor`: sin rol; el ADR los nombra como pendientes de F10.
- Formato de `etiquetas` con `origen` en el frontmatter (riesgos lo da por hecho; hoy es lista plana, arquitectura dice que «ya existe»): cambio de F1-9 que el ADR tiene que fijar.
- Roles no lanzados (dominio, testing): arquitectura preveia choques con ambos (vocabulario, huella); quedan sin contraste.

## Salidas que faltaron

- ninguna.

## Suposiciones no verificadas

- `ls-remote` cabe en el presupuesto de busqueda — arquitectura — PoC 2 contra GitHub.
- `rusqlite` `bundled` con FTS5 compila en los tres sistemas — arquitectura — PoC 1 y CI de F1-7.
- No hay politica de lectura con `fetch` ya decidida — arquitectura — el unificador vio que F1-2 §6 delega la lectura fresca en F3-5b, y F3-5b dice «traen la rama de estado antes de mostrar»; falta confirmar que vale para `buscar`.
- Las tareas terminadas se leen de `develop:tareas/` con formato 0.16 — arquitectura — PoC 1.
- `rename` sobre un `.db` abierto falla en Windows; copiar `engram.db` sin `-wal` pierde datos — riesgos — en la PoC si se usa; si no, en F10-1 y F10-5.
- `taskctl` 0.16 tolera secciones y campos nuevos en `tarea.md` sin perderlos — riesgos — antes de F10-4.

## Lo que necesita decision de una persona

- Este plan no vale hasta que Carlos lo apruebe con `taskctl approve`; el unificador no lo aprueba ni lo da por listo.
- Las decisiones de abajo se toman antes de escribir el ADR final; la (1) condiciona la PoC 2.

## Decisiones abiertas para la persona

1. Politica de frescura, una sola para `buscar` y F3-5b en `git::remoto`: (a) `ls-remote` en cada lectura, (b) TTL, (c) sin consultar el remoto y marca «remoto no consultado». Recomendacion del unificador: (a) con tiempo limite y sin prompt de credenciales, `fetch` dirigido si difiere, y (c) solo como respuesta sin red, con cerrojo ocupado o al agotar el tiempo; sin TTL (arquitectura: un TTL es datos viejos por diseno); si la medicion supera los 200 ms, Carlos elige entre relajar F10-6 o aceptar TTL. Implica leer RR-20 «nunca datos viejos» como «nunca datos viejos sin avisar»: eso solo lo puede aceptar Carlos.
2. Conversor engram A (materializar en git) o B (solo indice local) — recomendacion: A, con las condiciones de riesgos; B si no se acepta un riesgo de fuga sin vuelta atras. Irreversible, por eso no lo decide un rol.
3. Vocabulario en la rama de codigo (`.taskcode/vocabulario.yml`) o en la de estado — recomendacion: codigo (arquitectura: mapea rutas de codigo y lo revisa el PR); el coste es que cambiar el vocabulario exige PR. Sin rol de dominio que lo contraste.
4. Ubicacion `<git-common-dir>/taskcode/indice.db` (arquitectura) o `.git/taskcodetwo/` (AMPLIACION §3.1) — recomendacion: `taskcode/`, por coherencia con `refs/taskcode/` y `taskcode-estado.lock`; ambos roles exigen `--git-common-dir`. Y si se trocea en las partes a, b y c.

## Respuestas de la persona (Carlos, 2026-11-16)

1. Frescura: (a) `ls-remote` en cada lectura, con tiempo limite y sin prompt de credenciales; `fetch` dirigido si difiere; marca «remoto no consultado» solo sin red, con cerrojo ocupado o al agotar el tiempo; sin TTL. Una sola politica en `git::remoto` para `buscar` y F3-5b. Si la medicion de la PoC 2 supera los 200 ms de F10-6, se vuelve a preguntar.
2. Conversor engram: opcion A con las salvaguardas de riesgos (`--simular` obligatorio, rutas normalizadas, filtro de secretos, confirmacion humana de lo ambiguo antes de publicar).
3. Vocabulario: rama de codigo, `.taskcode/vocabulario.yml`.
4. Ubicacion: se adopta la recomendacion `<git-common-dir>/taskcode/indice.db` (no preguntada expresamente; se puede objetar en approve). Troceo: no se trocea (ver `## Troceo`).
