# Brainstorm — TASK-039, rol arquitectura (ronda 1)

- Rol: `brainstorm-arquitectura`
- Agente: brainstorm-arquitectura (definicion del plugin 0.16.2, ejecutado como subagente general)

## Enfoque propuesto, con rutas y nombres concretos

- **`tc2 auto [--json] TASK-NNN`, un paso por llamada y sin estado propio**: deduce la fase del estado real (`estado` de `tarea.md`, `plan-final.md`, rama, PR y `revision-ia`), pregunta a `carril::accion` y o bien **ejecuta** la fase determinista (`toma::start_en`, `revision::review_en`), o bien **delega** (`resultado=delegar rol=diseno|implementacion|correccion agente= modelo= peticion=<ruta>`), o **espera** (`Resultado::Esperando`, ya existe en `contrato.rs`), o **para** (`detener|preguntar motivo=`). Quien llama (skill del plugin o el doble del test) repite `tc2 auto` hasta `detener`.
- El agente **no es un puerto dentro de tc2**: es quien llama. Coherente con `plan/mod.rs` («El CLI no llama a ningun modelo: quien orquesta lanza los agentes») y con la preferencia de Carlos por comandos deterministas frente a hooks. Asi el criterio «fix de planificada a PR listo» se demuestra con un bucle sobre `tc2 auto` y un agente doble determinista, sin modelo de pago.
- Recorrido de un fix (fila `fix` de la tabla de F1-10: 1 rol, sin `approve`, parada solo en `merge-pr`): `planificada` → `plan_en` → delegar rol arquitectura (forma `Redaccion`) → `plan-final.md` ya no es esqueleto → `start_en` → delegar implementacion → `review_en` (empuja y abre PR) → esperar `revision-ia` → `hallazgos=rojo` → delegar correccion → `review_en` (ronda incremental, F6-6) → `hallazgos=verde` → `detener motivo=merge-pr` = «PR listo».
- La seleccion (RR-17) **no se recalcula**: `auto` lee `roles.diseno|implementacion|revision` que ya anotan `plan` y `review` (TASK-085) y la copia en la linea `delegar`. Correccion usa `roles.implementacion`.
- **Orden de construccion**: (1) `carril::politica` + `carril::accion` puras con la tabla de F1-10 decision 1 y 6; (2) `carril::minimo`/`efectivo` sobre `git::diff::ficheros_de_rama` (ya da los dos nombres de un renombrado); (3) `auto::fase_de` pura: estado observado → `Fase`; (4) `auto::paso` que une 1–3 con `start_en`/`review_en`; (5) la orden en `lib.rs` y la skill del plugin que hace el bucle (atajo del mismo comando, nunca el unico camino).

## Que se extiende y que se crea

- `src/carril.rs` — **se extiende** — su cabecera ya dice «F4 y F6 amplian este modulo en vez de copiar la tabla»; hoy solo tiene `roles_min`, `perfil`, `declarado`, `roles_diseno`. Faltan `politica`, `accion`, `minimo`, `efectivo` (verificado: no existen en `src/`). Las usara tambien el check `carril` de F6: una sola fuente.
- `src/auto/mod.rs` (+ `fase.rs`) — **se crea** — no hay orden por tarea que diga «siguiente fase»: `tc2 next` (`siguiente.rs`) lista tareas listas por grafo, no fases. Crear un modulo fino evita meter el encadenado en `plan`, `toma` o `revision`.
- `toma::Entorno` / `revision::Entorno` — **se reutilizan tal cual** como `auto::Entorno` (el de `review` ya contiene el de `start` y `Conectar`; `toma/mod.rs` dice «el orquestador de F4-4 lo usa directamente»).
- `plan::aprobar::es_esqueleto` — **se extiende su visibilidad** a `pub(crate)` para decidir «plan redactado»; no se reescribe otra comprobacion (hoy es privada en `plan/aprobar.rs:110`).
- `toma::operacion::MODO` (`"manual"` fijo, con comentario «hasta que F1-10 lo congele en plan») — **se extiende**: la fila de `start` toma el modo congelado por `plan` y `decidido_por` del orquestador; si no, la tabla de transiciones miente sobre RF-21.
- Lectura del estado de revision: `auto` lee del `Informe` de `review_en` las claves publicas `revision-ia` y `hallazgos` (ya contrato y probadas en `tests/revision.rs`), sin tocar `revision::paso`.
- **Alternativa descartada 1**: bucle largo en Rust que lanza `claude -p` por subproceso — mete el modelo dentro del CLI, contradice `plan/mod.rs`, no se prueba sin pagar y esconde la eleccion de agente/modelo en un proceso hijo.
- **Alternativa descartada 2**: hook `Stop`/`PostToolUse` que encadena fases — no determinista (memoria de Carlos y F1-10 midio 114 `approve` automaticos indebidos en ToolsaWeb); como mucho, atajo que llama a `tc2 auto`.

## Limites que cruza

- **Contrato de linea/JSON** (`contrato.rs`): nueva orden `auto` y valor de `resultado` `delegar` (o reutilizar `esperando` + clave `delegar=`); hay que decidir cual sin inventar un tercer vocabulario. Claves: `fase`, `accion`, `motivo`, `carril`, `carril-minimo`, `paradas` (las que F1-10 asigna a `siguiente --json`).
- **`tarea.md`** (esquema F1-9): solo escribe lo que ya escriben `plan`, `start` y `review`; `auto` no anade campos. La fila de `## Transiciones` pasa a llevar `modo=automatico` y `decidido_por=agente` en `start` y `review`.
- **Plataforma** (`Plataforma`): ninguna capacidad nueva; usa las de `review` (push, abrir PR, `estado_puertas`, `leer_revision`).
- **Identidad**: `auto` corre siempre como agente; `approve` y `merge-pr` quedan fuera de su alcance por construccion (`accion` las resuelve antes que el modo, F1-10 decision 6).
- **Dependencias**: `auto` → `carril`, `toma`, `plan`, `revision`; ninguno de ellos depende de `auto` (sin ciclos). `carril` sigue sin E/S.
- **Plugin**: la skill de bucle vive fuera del binario y solo interpreta `delegar|esperando|detener`.
- **Coste fijado por la forma**: cada llamada hace `fetch` y lee la rama de estado (como `plan`/`start`/`review`); un fix son ~8–10 llamadas. Aceptable; un lote (F4-7) lo multiplica, pero es el mismo patron que ya pagan las ordenes sueltas.
- **Desacuerdo previsto con riesgos**: probablemente pida estado persistido del orquestador (fichero de progreso) para reanudar; mi posicion es no tenerlo: la fase se deduce siempre del estado real, y un estado paralelo es una segunda fuente que diverge (Duplicacion de la regla 4D).
- **Suposiciones no verificadas**: (a) que `review_en` repetido con `revision-ia` pendiente sale `sin-cambios` sin efectos (doc de `revision/mod.rs`; no ejecutado); (b) que el check `carril` de F6 no tiene ya tarea propia que implemente `minimo`/`efectivo` (mirar las tareas F6 en `tareas/`); (c) que `plan` con un rol deja forma `Redaccion` y la salida del rol se publica con `plan guardar` como `plan-final.md` (visto en `plan/operacion.rs:282,431`, no ejecutado).

## La decision de diseño que mas te preocupa (UNA sola)

**Como sabe `auto` que el paso delegado (implementacion o correccion) ha terminado.** Con «un paso por llamada» la propia llamada a `tc2 auto` es la señal, y la fase se deduce de hechos (commits de la rama sobre `develop` desde el ultimo `head` revisado, worktree limpio). Es la opcion sin estado y sin duplicar fuentes, pero hace que «implementacion hecha» sea una inferencia: un agente que llama a `auto` a medias empuja trabajo incompleto a revision. La alternativa —que el agente declare `tc2 auto TASK --hecho implementacion`— es explicita, pero añade una entrada que el agente puede afirmar sin comprobarse (Disfraz). Propongo la inferencia por hechos y que `revision-ia` sea quien juzgue; el unificador debe decidirlo con lo que traiga riesgos.
