---
id: TASK-024
titulo: "F2-8 i18n de mensajes"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-024-f2-8-i18n-de-mensajes
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 672873
tokens_implementacion: 31680220
tokens_revision: 13771443
creado: 2026-11-14
actualizado: 2026-11-16
dependencias: [TASK-016]
---
## Objetivo

i18n de mensajes (es, en)

Tarea F2-8 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-1.

## Criterios de aceptacion
- [x] Test que falla si falta una traducción
- [x] Cumple lo que pide la tarea F2-8 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T22:46:36Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T22:54:50Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T22:55:12Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T23:46:26Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T00:18:40Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T00:30:39Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T22:54:08Z | diseno | agente:aac7fedea7ec4b16b | 28 | 74740 | 581369 | 16736 | 25 | 672873 | - | - |
| 2026-11-15T23:46:26Z | implementacion | sesion:1d6a1fb2-8f71-40e1-bb9f-dce13966fc92 | 116 | 101341 | 10513383 | 43116 | 15 | 10657956 | - | - |
| 2026-11-15T23:46:26Z | implementacion | agente:a0305aaa8d64c32a4 | 108 | 427954 | 8067507 | 50129 | 23 | 8545698 | - | - |
| 2026-11-15T23:46:26Z | implementacion | agente:a2223aabed12a30d0 | 146 | 253210 | 12142932 | 80278 | 13 | 12476566 | - | - |
| 2026-11-16T00:06:14Z | revision | agente:abf52ef889130cb92 | 130 | 232574 | 11152206 | 41059 | 6 | 11425969 | 1 | rust-wasm-reviewer |
| 2026-11-16T00:26:37Z | revision | agente:a4a0ffd59dcf47376 | 42 | 136300 | 2193120 | 16012 | 3 | 2345474 | 2 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — ce5b396
- rust-wasm-reviewer — f46e10e

## Resultado

**Implementado** (plan aprobado entero, sin trocear, por decision de la persona):

- `crates/tc2/src/i18n/`: `Idioma` (es por defecto; se resuelve con
  `TC2_IDIOMA` > `LC_ALL` > `LC_MESSAGES` > `LANG`, solo en `main.rs`), `Texto`,
  `Mensaje` neutro (clave + huecos `{nombre}`, `con`, `con_mensaje`,
  `externo`) y el catalogo por modulos (`catalogo/{cli,comun,config,estado,git,plataforma}.rs`)
  con la macro `textos!`, que exige `es` y `en`: si falta una traduccion no
  compila. Son unas 216 entradas y ninguna dependencia nueva.
- `contrato::Informe`: `diagnostico` es `Option<Mensaje>`, y se anaden `salida_en`, `json_en`
  y `diagnostico_en`. stdout, las claves, `motivo` y los codigos de salida no cambian con el idioma; solo se traduce el
  diagnostico (tambien en `--json`). `lib::ejecutar_en(idioma, args)`.
- Migrados todos los diagnosticos de `lib.rs`, `main.rs`, `git/`, `estado/`, `alta.rs`,
  `plataforma/` y `config/` (incluidos `ErrorConfig`, `Invalida` y los textos de `Regla`). El
  espanol se copia tal cual, sin tildes (decision de la persona). El esquema JSON publicado queda en espanol y sin cambios.
- Cambios de tipo: `estado::Fallo` lleva `Mensaje`, `ErrorConfig.mensaje: Mensaje`,
  `ErrorGit::Prohibido(Mensaje)`, `FalloHttp::Rechazada(&'static Texto)`, `FalloHttp::CuerpoGrande`
  y `plataforma::errores::Permiso`.
- Implementado en tres bloques: a) a mano; b) y c) en paralelo por dos agentes
  en worktrees propios, integrados con merge en la rama de la tarea.

**Criterio 1** (test que falla si falta una traduccion): la macro no compila si falta `es` o `en`.
`i18n::tests::catalogo_completo` falla con un texto vacio, con huecos distintos entre idiomas,
con el ingles copiado del espanol o con claves repetidas. `i18n::guarda::ningun_literal_fuera_del_catalogo`
falla con `.diagnostico("..")`, `.diagnostico(format!..)` o un `Mensaje::externo` fuera de la lista
cerrada `EXTERNOS`. Los mutantes del revisor de la ronda 1 ponen la suite en rojo.

**Criterio 2** (F2-8, RF-73 es/en): `resolucion_de_idioma`, `contrato::mismo_informe_dos_idiomas`,
`lib::new_y_actualizar_rama_en_ingles_mismo_stdout_y_codigo` y `tests/cli.rs`
(`TC2_IDIOMA=en` en el binario: stderr en ingles, mismo stdout y mismo codigo 2).
Prueba manual fuera de un repo: `actualizar-rama` da «git rev-parse failed: ...» en ingles y
«git rev-parse fallo: ...» en espanol, ambos con codigo 1.

**Evidencia**: `cargo test -p tc2`, 103 passed y 5 ignored (E2E con red, no se lanzan).
`cargo clippy --all-targets` sin warnings. `cargo xtask verificar --perfil rapida` en verde; la
primera vez cayo por el `sinred` intermitente de los mocks HTTP en Windows (viene de TASK-021) y al relanzarla salio en verde.

**Revision** (rust-wasm-reviewer, 2 rondas):

- Ronda 1, cambios-solicitados.
  - IMP-1 corregido: el permiso «pull_requests:read y checks:read» salia en espanol en ingles; ahora es la entrada `PERMISO_PR_Y_CHECKS`.
  - MEN-2 corregido: `debug_assert!` si un hueco queda sin valor. El mutante de `config/mod.rs:400` ahora deja 2 tests en rojo.
  - MEN-3 corregido: test en ingles de `new` y `actualizar-rama`.
  - MEN-4 corregido: `ErrorGit::Prohibido` con entradas del catalogo.
- Ronda 2, aprobada, sin hallazgos nuevos.

**No corregido** (decidido):

- MEN-1 aceptado: `SalidaGit::primera_linea_error()` devuelve «sin mensaje» en espanol cuando git falla con stderr vacio (caso raro). Arreglarlo exige pasar a `Mensaje` unas 15 firmas que hoy llevan el detalle de git como `String`.
- No revisado (fuera del alcance de la tarea): que cada mensaje diga «el siguiente paso» (RF-73) ni la calidad del ingles por una persona nativa.
- Las tildes del catalogo en espanol van en una tarea aparte (decision de la persona).
- Detectar el idioma de Windows sin `LANG` no entra en esta tarea; `TC2_IDIOMA` lo cubre.

## Duracion

- Estimada: 165 min
- Real (de la rama abierta al cierre con merge, sin pausas): 95 min
- Paso de la hora: 95 min reales frente a 165 estimados
- Cierre (finish): 203 s, de 2026-11-16T00:27:16Z a 2026-11-16T00:30:39Z
