# Plan — TASK-018: F2-3 Rama de estado

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 240 min (de la rama abierta al cierre con merge; señales:
unos 10 ficheros de código —3 que se extienden (`git/puerto.rs`,
`git/remoto.rs`, `contrato.rs`), 1 nuevo en `git/`, 5 en un módulo nuevo
`estado/`, más `lib.rs`—; 3 capas (puerto, operación git, protocolo);
3 criterios; 6 tests de integración, 2 pesados con varios procesos; adendas
en 3 ADR; revisión por pares que reproduce).

**Forma.** Un módulo nuevo `crates/tc2/src/estado/` lleva el protocolo de
F1-2 (cerrojo, reintentos, re-ejecución, pendientes y montaje). Se apoya en
una operación nueva de la capa Git, `git/estado.rs`, que implementa
`commit-estado` de F1-4: commit construido con `commit-tree` sobre
`origin/taskcode/tareas` con un índice temporal y
`push <sha>:refs/heads/taskcode/tareas`, **sin commit local ni `reset`**.
Dependencia en un sentido: `estado → git → puerto`.

Se reutilizan: `git/remoto.rs::publicar` (distingue non-fast-forward,
`[remote rejected]` y sin red), `localizar_principal`, `commit_de`,
`es_ancestro`, `worktrees`, `misma_ruta`, `exigir`/`correr`, e
`Informe`/`Resultado` como salida.

### Piezas

1. **`git/puerto.rs` (se extiende).**
   - Subcomandos nuevos: `CommitTree`, `ReadTree`, `WriteTree`,
     `UpdateIndex`, `HashObject`, `LsTree`, `Show`, `UpdateRef` (los de
     fontanería y `show` ya figuran en la lista de la PoC B2b de F1-4;
     `ls-tree` es de lectura; `update-ref` es nuevo y restringido).
   - `ejecutar_con_indice(dir, sub, args, indice)`: solo para
     `ReadTree`/`UpdateIndex`/`WriteTree`, que se rechazan en `ejecutar`
     normal; nunca tocan el índice real.
   - Prohibidos nuevos: `read-tree -u|--reset|-m`; `update-index
     --force-remove` (ya lo cubre `--force`).
   - `update-ref` solo con `[ref, sha]`, `[ref, sha, viejo]` o
     `["-d", ref, viejo]`, con `ref` bajo `refs/taskcode/`; nunca
     `refs/heads/*` ni `--stdin`.
   - No rompe F1-4: sin `reset`, `rebase`, force ni mover ramas; F1-4 D3 solo
     prohíbe `update-ref` sobre `refs/heads/*`.
   - Se actualizan `registro_permitido` y el test de argumentos prohibidos.
2. **`git/estado.rs` (nuevo, `commit-estado`).** `pub(crate)`:
   - `leer_arbol(sha)` con `ls-tree -r` y `show sha:ruta` bajo demanda;
   - `construir_commit(base, cambios, mensaje) -> Option<sha>`: `read-tree`
     en índice temporal, `hash-object -w`, `update-index --add --cacheinfo`,
     `write-tree`, `commit-tree`; si el árbol no cambia, `None` →
     `sin-cambios` sin push;
   - `publicar_commit(sha)` sobre `publicar`.
   - `Cambio` solo añade o modifica (las carpetas no se borran, F1-1); rutas
     relativas sin `..`.
3. **`git/remoto.rs` (se extiende).** `clasificar_fallo_remoto(stderr)`
   compartida por `fetch` y `push`: red caída, credenciales, remoto
   inexistente, rama inexistente (stderr en `LC_ALL=C`).
4. **`contrato.rs` (se extiende).** `Resultado::Denegada`, código **8**: el
   remoto deniega por credenciales, permisos, hook o regla; no se reintenta.
   `motivo=credenciales|remoto-rechaza|remoto-no-existe`. `sinred` gana
   `pendiente=si|no`.
5. **`estado/` (nuevo).**
   - `cerrojo.rs`: `.git/taskcode-estado.lock` en `--git-common-dir`, con
     `File::try_lock` (lock del sistema; se suelta si el proceso muere: no
     queda huérfano). Sondeo con tope de 60 s → `agotada motivo=cerrojo`.
   - `lectura.rs`: trait `Estado` (`listar(prefijo)`, `leer(ruta)`) sobre un
     commit; lo usan la transacción y la lectura pública.
   - `transaccion.rs`, `pendientes.rs`, `montaje.rs`, `mod.rs` (API).

Firmas orientativas:

```rust
pub trait Operacion { fn nombre(&self) -> &'static str; fn anotacion(&self) -> String;
    fn exige_red(&self) -> bool; fn aplicar(&self, e: &dyn Estado) -> Result<Vec<Cambio>, Rechazo>; }
pub fn transaccion(c: &Contexto, op: &dyn Operacion) -> Informe;
pub fn montar(git: &Git, dir: &Path) -> Informe;
```

### Protocolo de `transaccion`, por intento

1. Cerrojo.
2. `fetch -q origin refs/heads/taskcode/tareas:refs/remotes/origin/taskcode/tareas` (sin `+`).
3. Re-ejecutar las pendientes y después `op.aplicar` sobre el estado fresco.
4. Si hay generador inyectado, regenerar `ROADMAP.md`.
5. `construir_commit` y `publicar_commit`.
6. Non-fast-forward: soltar el cerrojo, dormir `base·2^(n-1)·U(0,5;1)`
   (jitter con `RandomState`, sin dependencias) y volver a 1. Tras el último
   intento, `agotada` sin nada escrito.
7. Publicada: `merge --ff-only -q <sha>` en `tareas/` si está montada; si no
   se puede, `tareas=sin-montar|con-cambios`. La corrección no depende de
   `tareas/`.

**Sin remoto** (`ls-remote --get-url origin` devuelve el nombre tal cual):
la base es la rama local y el commit entra por `merge --ff-only` (RN-06).

### Pendientes (D2b)

Sin red, una operación con `exige_red()=false` se guarda como commit en
`refs/taskcode/pendientes/NNN` con la anotación → `sinred pendiente=si`. La
siguiente transacción con red la reconstruye con `catalogo(anotacion)`, la
re-ejecuta y, publicada, borra la ref con `update-ref -d ref viejo` (CAS). Si
al re-ejecutarla se rechaza, pasa a `refs/taskcode/rechazadas/NNN`: el
contenido queda en un objeto de git (responde «dónde queda», F1-2).

### Montaje (`montar`, idempotente)

- `tareas/` ya es el worktree de `taskcode/tareas`: `sin-cambios`.
- Rama local sin worktree (desmontado o `clean -ffdx`): `worktree prune` y
  `worktree add tareas taskcode/tareas`.
- Rama solo en el remoto: `branch taskcode/tareas origin/taskcode/tareas` y
  `worktree add`.
- Rama inexistente: transacción de inicialización con commit raíz
  (`commit-tree` sin padre) y push; dos clones a la vez → el segundo recibe
  non-ff, re-ejecuta, la ve y la monta.
- `tareas/` ocupada por una carpeta normal (0.16): `rechazada
  motivo=tareas-ocupada` (la migración es de F9-2).

**Git < 2.42** resuelto por construcción: la raíz con `commit-tree` no
necesita `worktree add --orphan`. Relaja F1-1 D2; se dice en una adenda.

**`ROADMAP.md`**: gancho inyectable, `None` en producción hasta F3-4; el test
de concurrencia usa un generador mínimo propio.

**Binario**: no cambia (D1a de F1-4).

**Coste**: ~10 llamadas a `git` por transacción, 2 de red; `ls-tree` crece
con N. El trait `Estado` es la costura para `cat-file --batch` en F3-4.

### Orden de construcción

1. `contrato.rs` (`Denegada`) y `puerto.rs` con tests unitarios.
2. `clasificar_fallo_remoto` y `git/estado.rs`.
3. `estado/cerrojo.rs`, `lectura.rs`, `transaccion.rs` con los tests de concurrencia.
4. `pendientes.rs` y `montaje.rs`.
5. Adendas a F1-1 (2.42), F1-2 (preguntas abiertas resueltas, código 8) y F1-4 (lista ampliada, `update-ref` acotado, códigos 0-8).

### Alternativas descartadas

- Commit en `tareas/` y descartarlo tras el rechazo: exige `reset --hard`.
- Leer el estado del disco de `tareas/`: mezcla lo no commiteado.
- Cerrojo `create_new` con caducidad: carrera al robarlo; PID vivo en Windows pide dependencia.
- Orden oculta en el binario para el test: amplía D1a.

## Troceo

Si se troceara: **A** (60 min) pasos 1-2; **B** (100 min) paso 3, criterio
central; **C** (80 min) pasos 4-5.

### Motivo para no trocear

Consigna de Carlos: el lote H2 va sin trocear. Además A no tiene criterio
propio verificable y C reutiliza la transacción de B: trocear añadiría tres
ciclos de revisión sobre el mismo contrato.

## Lo que el rol no cubrio

- C1: modos de fallo intermedios (proceso que muere entre el push y el ff de
  `tareas/`, disco lleno con el índice temporal, temporales que quedan);
  `File::lock` en unidades de red.
- C2: no diseña la estrategia de pruebas, solo el presupuesto.
- C3: reglas de dominio (qué ve la persona con una pendiente en
  `rechazadas/`; transiciones propias sin red y RR-12; si credenciales merece
  código propio) y regresión de seguridad de ampliar la lista permitida (F1-8).

## Riesgos aceptados y que los contiene

- Clasificación de fallos por stderr: `LC_ALL=C`; lo no reconocido cae en
  `sinred` (reintentar). Git traducido sin verificar (ya anotado en F1-4).
- `publicar` ante `[remote rejected]` pasa a `denegada` sin reintento:
  cambia `push`/`actualizar-rama` de F2-2; se ajustan sus tests y la adenda.
- Espera base 2 s calibrada con el guion bash: es parámetro; se revisa en F9-4.
- Sin `ROADMAP.md` en producción hasta F3-4: gancho probado en el test.
- Concurrencia solo en una máquina: F9-4.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **C1 «dos procesos escriben a la vez sin perder nada»: 2 tests, ambos pesados.**
  - Dos clones contra un bare con `denyNonFastForwards`, K escrituras cada
    uno en procesos reales a la vez: 2K en `origin`, historia lineal sin
    merges, 0 refs o commits locales sobrantes, `ROADMAP.md` con 2K entradas.
  - Dos procesos del mismo clon: el cerrojo serializa y no se pierde nada
    (test propio del cerrojo, F1-2).
  - Procesos hijos reinvocando el ejecutable de test (`current_exe` + variable
    de entorno), sin dependencias ni ampliar el binario; espera corta por `Politica`.
- **C2 «test»**: lo cubren C1 y C3.
- **C3 «cumple F2-3»: 3 de integración y 1 unitario** (se pasa de 3 porque
  reúne las 5 preguntas abiertas de F1-2):
  - montaje: desde cero, repetir → `sin-cambios`, borrar `tareas/` y
    remontar, `tareas/` ocupada → `rechazada`, repo sin remoto;
  - remoto: hook `pre-receive` → `denegada` código 8 en un intento;
    `agotada` determinista (otro clon publica dentro de `aplicar`) sin nada escrito;
  - sin red con pendiente: `sinred pendiente=si`, al volver la red se publica y la ref desaparece;
  - unitario del puerto: formas nuevas prohibidas y permitidas;
    `ejecutar_con_indice` limitado a tres subcomandos.

Pesados: los dos de C1 (varios procesos); van en la suite normal con espera corta.

## Lo que necesita decision de una persona

Consigna de Carlos: se aceptan las recomendadas. Decidido:

1. Rechazo por credenciales o hook: **(a) `denegada`, código 8, con motivo**.
2. Pendientes sin red: **(a) aquí, con `refs/taskcode/pendientes/` y `update-ref` restringido**.
3. `ROADMAP.md`: **(a) gancho inyectable, `None` en producción hasta F3-4**.
4. Git 2.42: **(a) raíz con `commit-tree`, sin `--orphan`, adenda que relaja F1-1**.
5. `publicar` ante `[remote rejected]`: **(a) `denegada` sin reintento en todas las operaciones**.
6. Cerrojo: **(a) `File::try_lock` con tope de 60 s**.

Suposiciones a comprobar al implementar: `File::try_lock` estable en 1.100;
si `push <sha>:refs/heads/...` actualiza la ref remota de seguimiento (si no,
un `fetch` más); `ls-remote --get-url origin` sin remoto; textos de stderr de
Git 2.56 para credenciales y rama remota inexistente.
