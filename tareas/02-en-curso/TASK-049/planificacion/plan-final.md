# Plan — TASK-049: F6-3 revisor en CI con las skills de dominio

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 150 min
Senales: modulo nuevo de seleccion (catalogo, globs, tablas del ADR), subcomando nuevo, extension de `tc2 review`, workflow nuevo de clase privilegio con App y modelo, adendas; E2E real con App y clave nuevas; 2 criterios.

Dos piezas con una sola frontera:
1. **Seleccion determinista de revisoras** en el nucleo de tc2: funcion pura que recibe las rutas del diff y el catalogo de la base y devuelve revisoras, reglas, modelo y avisos; subcomando con contrato.
2. **Workflow `revision-ia`** de clase privilegio, en `workflow_run` al terminar `verificacion`: llama al subcomando, lanza cada revisora elegida con el modelo de la tabla, anade al prompt `seccion_revisor` del informe de verificacion y publica el check con la App del revisor.
El modelo no decide nada del enrutado.

Piezas y limites:
- `crates/tc2/src/seleccion/` (nuevo): `revision(cambios, catalogo, cfg, carril, cx) -> Seleccion` con las filas S08, S09, S12 y S14 y la columna "Revision" de la `tabla-modelo` (M01-M07) (F1-15 decision 1; F4-6 la ampliara). No reutiliza `plan::roles::seleccionar` (otra responsabilidad).
- Tablas leidas del ADR con `include_str!` de `docs/adr/F1-15-...md` entre los marcadores `tabla-seleccion` y `tabla-modelo` (las reglas no se copian en el codigo).
- `crates/tc2/src/seleccion/catalogo.rs` (nuevo), sobre `inventario::Entrada`: lee el bloque yaml de cada skill instalada (`rol: revisor`, `patrones_archivo`); las revisoras del plugin 0.16.2 ya lo traen. `inventario.rs` no cambia.
- Comparador de globs con la dependencia `globset`, sin distinguir mayusculas.
- `tc2 revisoras --rutas <fichero|->` (nuevo): rutas como dato (los dos nombres de cada renombrado), lee configuracion y `.claude/skills` de la base; linea de contrato F1-4 y JSON. En CI las rutas vienen de `pulls/{n}/files` (con `previous_filename`); en local de `git::diff::ficheros_de_rama`.
- `tc2 review` se extiende: muestra la seleccion, la anota en `roles.revision` de la tarea (F1-15 decision 10) y sigue leyendo `revision-ia`.
- `.github/workflows/revision-ia.yml` (nuevo, reutilizable + llamador de este repo, como `verificacion.yml`: `workflow_call`, `permissions: {}`, acciones por SHA). Pasos: el `GITHUB_TOKEN` con `actions: read` baja `informe-verificacion-<perfil>` con tope de tamano; `seccion_revisor`; SHA de `workflow_run` sin exigir igualdad con `head.sha` (seguimiento de TASK-044); checkout de la base, nunca del head; `tc2 revisoras`; una invocacion del modelo por revisora con tope de tamano del diff; token de instalacion de la App del revisor y check-run `revision-ia`.

Orden de construccion:
1. Adenda F6-3 en F1-15 y nota en F1-8: S10, S11, S13 y S15 se aplazan a F4-6/F6-6; sin F6-4 el disparador es `workflow_run` de `verificacion`; que publica el check en F6-3.
2. `seleccion::catalogo` y globs.
3. `seleccion::revision` y `tc2 revisoras` (criterio Rust + Vue sin modelo).
4. Extension de `tc2 review`.
5. Workflow `revision-ia` y E2E real.

Alternativas descartadas:
- Revisor en local que solo publica el check con la App: contradice F1-6 D5a y F1-8 (activo A2): un agente con la clave al alcance podria publicar `revision-ia` en verde.
- Patrones de cada revisora en el codigo de tc2: doble copia que F1-15 descarto (D8b).

## Troceo

No se trocea: Carlos pidio estas tareas sin trocear (2026-11-18).

## Lo que el rol no cubrio

- RR-17 "medir" (hallazgos por revisora, coste por modelo): F6-5 y F8-6.
- Deteccion por rutas sensibles (S10, S11; D19-D22) aplazada a F4-6.
- Casos limite (riesgos), pruebas finas (testing), reglas de negocio (dominio) sin revisar; prompt injection en F6-7.

## Riesgos aceptados y que los contiene

- `revision-ia` en `neutral` no bloquea hasta F6-5: lo contiene la aprobacion obligatoria de la persona en el ruleset.
- El catalogo en CI depende de la version del plugin en el runner: se fija en la configuracion de la base, que un PR no puede cambiar (F1-10 decision 3).
- Coste: N invocaciones del modelo por ronda (2-3 en un PR Rust + Vue): tope de tamano del diff en el prompt; `seccion_revisor` evita repetir tests.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **C1 (Rust + Vue lanza las dos):** (1) unitario de `seleccion::revision` con `src/a.rs` y `web/App.vue` y fixtures que copian el yaml real de la 0.16.2: `angular-vue-reviewer` y `rust-wasm-reviewer`, regla S08, modelo M04, sin red ni modelo; (2) integracion de `tc2 revisoras --rutas -`: linea de contrato y JSON.
- **C2 (F6-3 y RR-02/RR-17):** (1) `README.md` suelto -> generica (S08); revisora no instalada -> aviso `no-instalado` (S14); (2) **pesado**, PR real en el repo E2E: `revision-ia` publicado por la App del revisor con las dos revisoras, el prompt cita `seccion_revisor` y el job no ejecuta la suite.

## Lo que necesita decision de una persona

Consigna de Carlos (2026-11-18): la opcion recomendada y sin trocear.

- **D1. Como se invoca el modelo.** Decidida por Carlos (2026-11-19): (a) API key de Anthropic como secreto del entorno `revisor` de Actions, solo en el job publicador. La crea y la guarda la persona; nunca pasa por el chat.
- **D2. App del revisor.** Decidida por Carlos (2026-11-19): la crea la persona con `checks: write`, `contents: read`, `pull_requests: read`, `metadata: read` (sin `actions: read`: el artefacto lo baja el `GITHUB_TOKEN`); se instala en este repo y en el repo E2E; id y clave en el entorno `revisor`; se fija `plataforma.revisor_id`.
- **D3. Disparador.** Elegida: `workflow_run` de `verificacion`. Nota: `workflow_run` toma el workflow de la rama por defecto (develop en TaskCodeTwo, main en el repo E2E): el E2E se hace en el repo E2E con el llamador en su `main`.
- **D4. Conclusion del check en F6-3.** Elegida: `neutral` con el resumen (revisoras, reglas, modelo, avisos).

Suposiciones por verificar:
- El plugin taskcode y sus skills se obtienen en CI a una version fijada.
- `workflow_run` recibe los artefactos del run de `pull_request`.
- El bloque yaml cae dentro de `MAX_CABECERA` en las 5 revisoras.
- El modelo de tarea admite `roles.revision` con `agentes`, `regla` y `modelo`.
- Rama por defecto: develop en TaskCodeTwo, main en el repo E2E (verificado 2026-11-19).
