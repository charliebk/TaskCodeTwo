# Plan — TASK-028: F3-1 tc2 init

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 300 min

`tc2 init` sigue el patron de `tc2 desinstalar`: inventario de solo lectura → plan como dato (una linea `accion ruta=… clase=… dueno=… decision=…` por artefacto) → aplicacion de ese mismo plan; `--simular` imprime el plan y no escribe. Lo que `init` crea sale de los nombres de `huella.rs` que `desinstalar` tambien lee, asi que se reconocen sin mas. `init` no reimplementa: monta con `estado::montar`, excluye con `git::rama::excluir` y valida con `config::leer` y el lector de `personas`. Los rulesets solo se proponen (D1).

Que crea `init`:
- Rama de estado y su worktree con `estado::montar` (trae la del remoto si existe; si no, la crea).
- `main` y `develop`, solo si faltan (F1-3).
- `.taskcode/config.yml` con `---\nformato: 2\n`, solo si no existe.
- `personas.yml` con la persona de `GET /user`, solo con plataforma y token; si no, `pendiente`.
- `.git/info/exclude` con `/tareas/` y `/.taskcodetwo/worktrees/` (`excluir`).
- `.gitignore` con las mismas rutas (F1-3 encargaba esto a F3-1, con el nombre corregido a `.taskcodetwo`), anadiendo lineas, nunca reescribiendolo.
- Workflow de CI: ranura `pendiente motivo=sin-plantilla` (D2).
- Rulesets: solo propuesta (D1).

Piezas:
- `huella.rs` (se extiende): `RUTA_WORKFLOW`, `LINEAS_GITIGNORE`, `LINEAS_EXCLUDE` (hoy duplicada en privado en `desinstalar/proyecto.rs::lineas_exclude`) y `PREFIJO_RULESETS = "taskcodetwo-"`.
- `estado/montaje.rs` (se extiende): `pub fn previsto(git, dir) -> Previsto {Montada, Crear, DelRemoto, Ocupada, OtraRuta, SinRed}` de solo lectura, usada por `montar` y por el inventario de `init` (costura que impide que `--simular` y la ejecucion se separen).
- `init/` (nuevo: `mod.rs`, `inventario.rs`, `aplicar.rs`): `Opciones {simular}`, `orden(args, json, idioma) -> Salida`, entrada en `lib.rs`; decisiones `crea`, `existe`, `persona`, `pendiente`, `rechaza`.
- `contrato` (se extiende): el formato de la linea `accion …` (texto y JSON) se mueve desde `desinstalar::Accion::linea/json`; los enums de decision de cada orden siguen propios.
- `config::texto_minimo()` (test `leer(texto_minimo()) == Config::suelo()`) y `personas::texto_de(&Usuario)` junto al lector.
- `plataforma/puertas.rs` (se extiende): modelo neutral de F1-5 (decision 1) y `ruleset_github(&modelo) -> Value`, solo para generar la propuesta.
- `desinstalar/proyecto.rs` (se extiende): reconoce el workflow y las lineas de `.gitignore` (como `persona`, estan versionados) y los rulesets `taskcodetwo-*` (como `persona`, el remoto no se toca).

Orden de construccion:
1. `huella`, formato de linea a `contrato`, generadores de config y personas (funciones puras; los tests de `desinstalar` siguen en verde).
2. `montaje::previsto` y `montar` sobre el.
3. `init --simular`: inventario, plan, detecciones de la 0.16, sin remoto y sin red (C3, C4, C5).
4. Aplicacion: rama de estado, `main`/`develop` con commit raiz, exclude, `.gitignore`, config y personas (C1, C2).
5. Propuesta de puertas y extension de `desinstalar` (C6, C7).

Alternativas descartadas: `init` imperativo paso a paso (`--simular` duplicaria la logica; el plan como dato ya esta decidido en F1-12 PL1 y probado en `desinstalar`); aplicar los rulesets por defecto (exigen checks que no existen hasta F5-1/F6 y bloquearian todo PR; F1-5 se niega con el `integration_id` de relleno).

## Lo que el rol no cubrio

- Riesgos sin especialista; aceptados: dos `init` concurrentes (lo contiene la transaccion de `montar`), token de persona ausente (`pendiente`), `.gitignore` del usuario (solo se anaden lineas).
- Testing sin estrategia propia: git real en directorio temporal y el doble HTTP existente; nada contra la red real.
- Dominio sin revisar: lista de areas del hito, regla de personas y si RN-06 implica «sin remoto».
- Suposiciones sin verificar: que `estado::transaccion` sin `origin` cree la rama solo en local (`intento()` con `remoto=false`); que `localizar_principal` y `worktrees` funcionen en un repo sin commits; que `detectar` falle cerrado con `origin` `file://`; la lectura de RN-06 (`REQUISITOS-FUNCIONALES.md:232`).

## Riesgos aceptados y que los contiene

- `--simular` y aplicacion divergentes: `montaje::previsto` comun y C3 compara el plan simulado con las acciones reales.
- Pisar la 0.16 o una config ajena: C4 rechaza sin escribir.
- Push a `develop`: solo en repo vacio; con historia, `persona`, nunca push directo.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):
- `crates/tc2/tests/init.rs`: `repo_vacio_sin_remoto` (C1), `init_dos_veces_sin_cambios` (C2), `simular_no_toca_nada_y_coincide` (C3), `init_en_repo_016_rechaza` y `init_con_config_invalida_no_la_pisa` (C4), `sin_remoto_completa_al_anadirlo` y `sin_red_no_escribe` (C5), propuesta de rulesets con el doble HTTP y 0 escrituras (C6), `init_y_desinstalar_se_reconocen` (C7).

## Lo que necesita decision de una persona

Carlos acepta las recomendaciones (consigna del 2026-11-16):
- D1: rulesets solo como propuesta en F3-1; su aplicacion va a una tarea nueva tras F5-1, F6 y F6-1 (divergencia con F1-5 «init aplica las puertas» anotada en el ADR).
- D2: workflow de CI como ranura `pendiente` hasta que F5-1 entregue el reutilizable.
- D3: en repo vacio, commit raiz en `main` y `develop` y push; con historia, `persona`.
- D4: hito de decisiones (F1-18) y manifiesto de recursos (F1-14 punto 8) fuera de F3-1, con la divergencia anotada.
- D5: solo `tc2 init` (sin alias `setup`) y opcion `--simular` (sin alias `--dry-run`).
- D6: rulesets `taskcodetwo-*` por F1-12 D1a.
