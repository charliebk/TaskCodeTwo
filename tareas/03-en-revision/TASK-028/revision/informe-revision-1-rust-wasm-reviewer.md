# Informe de revision — TASK-028 (ronda 1)

- Commit revisado: 2c3706b7cb68b2e455377ec33ab0766aae68db39
- Revisor: rust-wasm-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-028 aprobada | aprobada-con-correcciones | cambios-solicitados)

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | crates/tc2/src/init/inventario.rs:292 | implementacion |
| MEN-1 | MENOR | abierto | crates/tc2/src/init/inventario.rs:176 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/init/inventario.rs:442 | plan |
| MEN-3 | MENOR | abierto | crates/tc2/src/plataforma/puertas.rs:246 | plan |
| MEN-4 | MENOR | abierto | crates/tc2/src/init/inventario.rs:349 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 — `develop` (o `main`) nuevo como commit raiz sin relacion con la historia que ya tiene `origin`

`ramas()` decide «repo vacio» solo por `hay_commits` (refs locales) y mira en `origin` unicamente
si estan `main` y `develop` (`ramas_remotas`). Si el repo local no tiene commits pero `origin`
tiene historia en `main` (el caso de `git init` + `git remote add origin <url>` del arranque
rapido de GitHub), `main` sale `persona en-el-remoto` pero `develop` sale `crea`: se construye un
commit raiz propio y se **publica** `develop` en `origin`. D3 del plan decia «push solo en repo vacio».

Reproduccion (binario de la rama, git real, remoto bare local):
- remoto con `main` = `db3bae6` (un commit); `git init -b master A && git remote add origin <bare>`.
- `tc2 init --simular` -> `rama main ... persona motivo=en-el-remoto`, `rama develop ... decision=crea`.
- `tc2 init` -> `init ok`; `git log --all --graph` en el remoto: `db3bae6 c` (main) y, aparte,
  `7e7ebb2 chore: inicia TaskCodeTwo (tc2 init)` (develop): dos historias sin relacion publicadas.
  No hay force (el push es solo de refs nuevas), pero la `develop` publicada ya no se puede
  fusionar con `main` sin `--allow-unrelated-histories`.

Arreglo propuesto: considerar «vacio» solo si `origin` no tiene ninguna rama (`ls-remote --heads`
sin patron) o, como minimo, si `main` o `develop` estan en el remoto, que la otra sea `persona`
(`git fetch` y `git branch` desde la que existe). Anadir test con remoto con `main` y repo local
sin commits.

### MEN-1 — Orden impresa sin comillas con una ruta (`worktree-estado ... montada-en-otra-ruta`)

`git worktree move {ruta} tareas` se arma con la ruta tal cual. En `--json` (y tras decodificar el
`%20` del texto) la orden queda sin entrecomillar. Reproduccion: `tareas` movida a
`.../con espacios; echo X/tareas` -> `"orden":"git worktree move .../con espacios; echo X/tareas
tareas"`; pegada en un shell parte la ruta y ejecuta `echo X/tareas tareas`. La ruta sale de
`git worktree list` del propio repo de la persona, asi que el riesgo es bajo, pero la linea
afirma ser una orden lista para ejecutar. Entrecomillar (o escapar) la ruta. Las demas ordenes
(`git push origin main`, `git symbolic-ref ...`, `git branch ...`) solo llevan constantes y
nombres de rama fijos: sin problema.

### MEN-2 — La orden de las puertas (`tc2 init --aplicar-puertas`) no existe todavia

Cada linea `ruleset ... decision=persona motivo=sin-checks` trae `orden=tc2 init --aplicar-puertas`
y esa opcion hoy da `uso` (`leer_opciones` la rechaza). Esta documentado en el ADR (la aplica
TASK-118), pero la linea dice a la persona que ejecute algo que falla. Aclararlo en el `motivo` o
no imprimir `orden=` hasta TASK-118.

### MEN-3 — La propuesta de ruleset de GitHub lleva los checks sin `integration_id`

`ruleset_github` emite `{"context": c}` sin `integration_id`, la variante que el ADR F1-5 descarta
(«Checks sin origen: cualquier identidad con permiso de escribir checks publicaria un
`revision-ia` verde»). Esta anotado en el comentario de la funcion, pero la propuesta sale en
`--json` como JSON aplicable: quien la aplique a mano (`gh api`) obtiene el ruleset que el ADR
descarto. Anadir un aviso en la propia propuesta, o no emitir `required_status_checks` hasta F6-1.

### MEN-4 — Se pierde el motivo real cuando `detectar` rechaza la plataforma

`plataforma()` convierte cualquier `Err` de `detectar` en `sin-plataforma`. `detectar` distingue
`plataforma-contradice`, `api-no-cambiable` (una clave `plataforma` que apuntaba a otra API: fallo
cerrado deliberado) y `declara-plataforma`. En las lineas `pendiente` de personas y rulesets todos
salen como `sin-plataforma`; la persona no sabe que ha de corregir `config.yml`. El token no se
envia en ningun caso (comportamiento correcto); solo falta el motivo.

## Alcance

- Ficheros revisados (17 del diff): `config/mod.rs`, `contrato.rs`, `desinstalar/{mod,proyecto}.rs`,
  `estado/{mod,montaje}.rs`, `huella.rs`, `i18n/catalogo/{cli,init,mod}.rs`, `init/{aplicar,inventario,mod}.rs`,
  `lib.rs`, `plataforma/{personas,puertas}.rs`, `tests/init.rs`.
- Contexto adicional pedido: `git/remoto.rs` (`publicar`, `intento_push`: comprobar que no hay
  force), `git/estado.rs` (`construir_commit`), `plataforma/deteccion.rs` (fallo cerrado de la API)
  y el ADR F1-5 (modelo de puertas). Ver «Lecturas fuera del contexto permitido».
- Limite JS-WASM: sin cambios; el diff no toca `#[wasm_bindgen]` ni hay objetivo `wasm32` en este crate.

## Reproduccion

- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-028-r1-rust`, rama
  `feature/task-028-f3-1-taskctl-init-setup` (HEAD 9fff220, sobre 2c3706b).
- Build/tests: `cargo test -p tc2`. Todo en verde salvo `tests/plataforma.rs` (tres fallos en la
  primera ejecucion y dos en la segunda: `estado-puertas sinred` e `identidad sinred` en lugar de
  `esperando`/`limite-peticiones`; cada test tardaba mas de 60 s): es el cuelgue conocido de
  127.0.0.1. Repetido de forma aislada (`cargo test -p tc2 --test plataforma`): 4 de 4 en verde
  en la rama (33 s) y en `origin/develop` (12 s). `tests/init.rs`: 12 de 12 en verde. El resto de
  binarios, `ok`. Sin `LNK1104`.
- Formato y lints: `cargo fmt --check` sin diferencias; `cargo clippy --all-targets --locked -- -D warnings`
  sin avisos ni errores.
- Casos construidos (binario `tc2.exe` copiado del build, git real, remoto bare en carpeta temporal):
  1. Remoto con `main` y repo local sin commits (IMP-1): `develop` raiz publicado.
  2. `tareas/` montada en una ruta con espacios y `;` (MEN-1): orden sin comillas.
  3. `.gitignore` existente sin salto final (`node_modules`): `init` anade con `\n` las dos lineas
     sin tocar la primera y el commit raiz lleva el mismo contenido. Correcto.
- Mutacion: `else if hay_commits` -> `else if false && hay_commits` en `init/inventario.rs`: se pone
  rojo `con_historia_las_ramas_que_faltan_son_de_la_persona` (1 de 12). Revertido.
- Medidas: no aplican (sin camino caliente ni WASM).

## Revisado sin hallazgos

- `estado::previsto` / `montar`: `montar` parte de `previsto_en` bajo el cerrojo y solo difiere en
  lo que queda fuera del plan (`hay_remoto` y `prune` de registros muertos); `Montada`, `OtraRuta`,
  `Ocupada`, `SinRed` y `Denegada` dan lo mismo en el plan y en la ejecucion; la carpeta vacia o
  `tareas` como fichero se clasifican bien. `--simular` y la ejecucion coinciden
  (`simular_no_toca_nada_y_coincide`).
- Commit raiz: `construir_commit` con indice temporal; las ramas se crean con `git branch <r> <sha>`
  (falla si existe, no pisa); el push usa `publicar` con refspec explicito `refs/heads/r:refs/heads/r`,
  `--porcelain`, sin `--force` ni `+` (revisado `init/` y `git/remoto.rs`); un rechazo del remoto
  devuelve `Error` con diagnostico y un `init` repetido ve la rama local como `persona sin-publicar`.
- `.gitignore`: `anadir` solo agrega las lineas que faltan, respeta `\r\n`, no duplica y no toca
  el resto. `escribir_nuevo` usa `create_new`: no pisa una config creada despues del plan.
- `contrato::LineaAccion`: texto y JSON equivalentes a los anteriores de `desinstalar` (sus tests
  siguen en verde); los valores libres pasan por `escapar_valor`/`cadena_json`; las claves son
  constantes. El `strip_suffix("}}")` de `Accion::json` es fragil pero correcto mientras
  `LineaAccion::json` cierre con `}}`; lo cubre `linea_y_json_con_propuesta`.
- Modelo de puertas y `ruleset_github`: los cinco nombres llevan el prefijo de la huella; reglas
  `deletion`, `non_fast_forward`, `pull_request`, `required_status_checks` (`strict` true en
  `develop`, false en `main`), tags con `update` y `update_allows_fetch_and_merge:false`;
  `bypass_actors` vacio. Coincide con la PoC C del ADR salvo lo anotado en MEN-3.
- La propuesta no escribe en la plataforma: `puertas()` solo llama a `leer_puertas` e `identidad`
  (GET); el doble HTTP de `tests/init.rs` comprueba que todas las peticiones empiezan por `GET `.
  `detectar` falla cerrado si una clave contradice el host conocido.
- Produccion sin `unwrap`, `expect`, `panic!`, `unreachable!` ni `unsafe` en los ficheros del diff
  (los `unwrap` de `tests/init.rs` son de test).
- `personas::texto_de`: escapa `\` y `"` del login y el llamador lo revalida con `Personas::leer`.
- `desinstalar`: reconoce `.gitignore`, workflow y rulesets como `persona`; sin escrituras.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/git/remoto.rs` (`publicar`, `intento_push`): comprobar que el push no usa force.
- `crates/tc2/src/git/estado.rs` (`construir_commit`): revisar la creacion del commit raiz.
- `crates/tc2/src/plataforma/deteccion.rs`: comprobar el fallo cerrado de la API antes de enviar el token.
- `docs/adr/F1-5-puertas-en-github.md`: modelo de puertas y variante sin `integration_id` descartada.
