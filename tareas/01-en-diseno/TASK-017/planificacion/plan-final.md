# Plan — TASK-017: F2-2 Capa de operaciones Git deterministas

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

Implementacion en **Rust** (ADR F1-14), sobre el workspace de F2-1. Manda la
ADR F1-4 (lista cerrada de operaciones y contrato de salida).

## Enfoque propuesto

Duracion estimada: 170 min

Senales: unos 10 ficheros de codigo nuevos (contrato, puerto, 6 modulos de
operaciones, helper de tests, tests de integracion), 3 capas (contrato,
puerto Git, operaciones + enrutado), 3 criterios.

- Se crea en la biblioteca `tc2` el puerto `Git` de F1-4 (decision 1): un
  struct que lanza el binario `git` con `std::process::Command` y entorno fijo
  en cada llamada (`LC_ALL=C`, `LANG=C`, `GIT_TERMINAL_PROMPT=0`,
  `GIT_PAGER=cat`, `GIT_EDITOR=true`, `-c color.ui=never`,
  `-c core.quotepath=false`). Sobre el, las operaciones locales de la lista
  cerrada; cada una devuelve un `Informe` que se convierte en la `Salida
  {codigo, stdout, stderr}` que ya existe en `crates/tc2/src/lib.rs`.
- Sin dependencias nuevas (ni de ejecucion ni de desarrollo).
- Binario `git` y no libgit2/gix: F1-4 descarta libgit2 explicitamente
  (worktrees, `ls-remote` con patrones y credenciales del sistema ya
  funcionan con `git`, que es la unica dependencia externa: P12, RR-16).

### Piezas y limites

- `crates/tc2/src/contrato.rs` (nuevo): `Resultado` (ok, sin-cambios,
  rechazada, sinred, agotada, conflicto, esperando, uso, error), la tabla de
  codigos 0–7 en un unico sitio (F2-3 la fija tocando solo este fichero),
  `Informe { comando, resultado, claves, diagnostico }` con la linea
  `op resultado clave=valor`, `--json` e `impl From<Informe> for Salida`.
  Fuera de `git/` porque tambien lo usaran `Plataforma` y los comandos de fase.
- `crates/tc2/src/git/puerto.rs` (nuevo): `GitProceso { binario, registro }`
  con `ejecutar(dir, Subcomando, args)`. `Subcomando` es un enum con solo la
  lista permitida de F1-4 B2b: `reset`, `rebase`, `stash`, `clean` no se
  pueden escribir (RN-05 por construccion, H9). El puerto rechaza argumentos
  prohibidos dentro de los permitidos (`--force*`, `-f`, refspec `+`,
  `--mirror`, `--delete`, `branch -f/-d/-D/-m`, `tag -d/-f`) con `error`.
  Struct, no trait: los tests van contra repos reales.
- `crates/tc2/src/git/`: `rama.rs` (`validar-base`, `comprobar-id`,
  `crear-rama-y-worktree`, `reanudar`, `localizar-principal`), `guardar.rs`,
  `remoto.rs` (`fetch`, `push` con reintento, `actualizar-rama`,
  `actualizar-principal`), `worktree.rs` (`quitar-worktree`), `tag.rs`
  (`tag-anotado`), `diff.rs` (`diff-tarea`). Son `pub` en la biblioteca
  (las usaran F2-3, F3-6, F7-3) pero no subcomandos del binario (D1a).
- `crates/tc2/src/lib.rs` (`ejecutar`): enruta solo `guardar "<mensaje>"
  <rutas…>` y `actualizar-rama` (los dos publicos de D1a); argumentos que no
  encajan → `uso` (2). `main.rs` no cambia.
- `crates/tc2/tests/comun/mod.rs` (nuevo): fixtures con std puro, carpeta bajo
  `std::env::temp_dir()` (nunca la raiz de `C:\`), nombre con pid y contador,
  `Drop` que la borra; remoto bare local con `receive.denyNonFastForwards=true`;
  aislamiento de la config del usuario solo en tests (`GIT_CONFIG_NOSYSTEM=1`,
  `GIT_CONFIG_GLOBAL` a un fichero de la fixture, identidad fija,
  `core.autocrlf=false`). En produccion no se toca la config del sistema.
- `crates/tc2/tests/git_*.rs` (nuevos): integracion contra repos reales.
- No se tocan `xtask` ni `ci.yml`: `rapida` ya corre `cargo test` en los tres
  sistemas.

### Orden de construccion (misma rama, commits por parte)

1. `contrato.rs` e `Informe` con tests unitarios.
2. `puerto.rs` (lista permitida, entorno fijo, registro) y `tests/comun`.
3. `rama.rs`, `worktree.rs`, `diff.rs`.
4. `guardar.rs`, `remoto.rs` (push con reintento, `actualizar-rama` con
   conflicto), `tag.rs`.
5. Enrutar `guardar` y `actualizar-rama`; `cargo xtask verificar --perfil
   completa` en Windows; CI en los tres sistemas.

### Alcance: que NO se hace aqui

- F2-3: `commit-estado`, `commit-tree` sobre `taskcode/tareas`, pendientes sin
  red, worktree de `tareas/`. Los codigos 0–7 se usan como los propone F1-4;
  los fija F2-3.
- F2-6/F2-9: puerto `Plataforma`, `abrir-pr`, `estado-puertas`, `merge-pr`,
  HTTP, identidad. El 7 (`esperando`) existe en el enum pero nadie lo emite.
- F3-6: `start`, toma atomica, slug desde el titulo (`crear-rama-y-worktree`
  recibe `id`, `tipo`, `slug` y `base` resueltos), aviso de solape.
- F7-3: `release`, `hotfix`, `notas-release`. F2-2 deja `tag-anotado`.
- `doctor` (F3-8) y el hook de D4a.

## Troceo

### Parte a: contrato, puerto y fixtures
Pasos 1 y 2 (unos 50 min).

### Parte b: rama, worktree y diff-tarea
Paso 3 (unos 50 min).

### Parte c: guardar, push, actualizar-rama, tag y comandos publicos
Pasos 4 y 5 (unos 70 min).

**Motivo para no trocear**: la persona decidio no trocear las tareas del H2.
Ademas la parte a no se valida contra el criterio 1 sin al menos una
operacion, y b sin c no cubre ni el «commit de lo escrito» ni el «push con
reintento» que nombra F2-2. Las partes quedan como commits de control.

## Lo que el rol no cubrio

- Sin rol de riesgos: modos de fallo solo los ya ejercitados por F1-4 (B1,
  B1b, B2); sus «Caminos sin ejercitar» siguen abiertos.
- Sin rol de testing: el plan de pruebas no se ha contrastado en cobertura ni
  coste; el criterio 2 descansa en el renderizado comun.
- Sin rol de dominio: formato del mensaje de `guardar` y quien puede lanzar
  `tag-anotado` (F1-6, F1-10) no se deciden aqui.
- Suposiciones no verificadas: que `push --porcelain` marque el
  non-fast-forward como `[rejected]` y el hook como `[remote rejected]` en las
  versiones de la CI (lo confirma el test 1c); que `temp_dir()` en ubuntu y
  macos admita las rutas de worktree (primera CI).

## Riesgos aceptados y que los contiene

- El «Git traducido» de F1-4 puede volver a comprobarse de forma trivial si el
  runner no tiene catalogos ni locale `es_ES`. Lo contienen el `LC_ALL=C` de
  cada llamada y la lectura solo de porcelana o formatos explicitos; si no se
  demuestra, el `## Resultado` dira «no verificado».
- La lista permitida por enum obliga a pasar por codigo para anadir un
  subcomando: intencionado (P8, una operacion nueva entra por ADR).

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **C1 Tests contra repos reales — 3 tests, los 3 pesados** (remoto bare y
  varios procesos `git`):
  - (a) ciclo `crear-rama-y-worktree` → `guardar` → `push` → `tag-anotado` →
    `quitar-worktree`, cada paso dos veces, huella de refs, worktrees y status
    identica tras la segunda;
  - (b) `actualizar-rama` con conflicto provocado: codigo 6, repetir no vuelve a
    mergear, `guardar` rechaza otra ruta y las marcas, merge commit con 2
    padres;
  - (c) `push` con hook `pre-receive` que falla una vez (`intentos=2`),
    non-fast-forward → 3 sin reintento, `tag-anotado` sobre otro commit → 3 sin
    publicar.
- **C2 Salida ≤ 5 lineas — 2 tests, ligeros**:
  - (a) unitario del renderizado: ≤ 5 lineas, sin ESC, primera linea
    `op resultado clave=valor`, valores sin espacios, hashes de 12, `--json`
    con las mismas claves;
  - (b) el binario `tc2 guardar` / `tc2 actualizar-rama` como proceso cumple
    el contrato y deja stderr vacio en el caso normal.
  - Ademas los tests de C1 comprueban el contrato en cada salida con una
    funcion comun del helper.
- **C3 F2-2 y requisitos que cita (RR-01, RF-23, RF-25, RN-05) — 2 tests**:
  - (a) ligero: `guardar` con otros ficheros modificados commitea solo las
    rutas dadas e informa `sin-guardar=N` (RF-23);
  - (b) pesado: el registro del puerto durante el ciclo de 1a no tiene llamadas
    fuera de la lista permitida ni argumentos prohibidos (RN-05, H9), y la
    salida es identica con `LANG=es_ES.UTF-8` en el entorno padre.

Los pesados no se separan en `comando_verificacion_pesada`: son repos locales
de segundos; se miden en la implementacion y, si pasan de unos 30 s, se anota.

## Lo que necesita decision de una persona

Resueltas con la opcion recomendada (consigna de la persona: «aceptamos todas
las preguntas con la respuesta recomendada»):

1. **Comandos publicos**: se enrutan ya `guardar` y `actualizar-rama` en el
   binario (D1a; no tienen fase duena).
2. **`--json`**: ahora, en `Informe`, escrito a mano sin serde.
3. **Reintentos de `push`**: 3 intentos, espera 1 s y 2 s, inyectable en tests.
   Transitorio todo salvo `[rejected]` con `non-fast-forward` o `fetch first`
   en `push --porcelain`. Agotado → `sinred` (4); distinguir el rechazo por
   reglas de la plataforma pasa a F2-6.
4. **Base de `actualizar-rama`**: por prefijo de la rama segun F1-3:
   `hotfix/` → `origin/main`, el resto → `origin/develop`.
5. **Mensaje de `guardar`**: solo se rechaza el vacio (`uso`); el formato es
   de dominio (F1-9).
6. **Version minima de `git`**: no se comprueba en F2-2; las versiones de la
   CI quedan anotadas en el `## Resultado`; la comprobacion es de `doctor`
   (F3-8).
