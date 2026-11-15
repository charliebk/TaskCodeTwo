---
id: TASK-023
titulo: "F2-7 Configuracion minima con esquema validado y valores por defecto"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-023-f2-7-configuracion-minima-con-esquema-va
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1976052
tokens_implementacion: 10920973
tokens_revision: 20259729
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-011]
---
## Objetivo

Configuración mínima con esquema validado y valores por defecto

Tarea F2-7 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-10.

## Criterios de aceptacion
- [x] ≤ 12 claves
- [x] esquema publicado
- [x] tests
- [x] Cumple lo que pide la tarea F2-7 del plan y los requisitos que cita

## Resultado

**Qué se hizo.** Un módulo de biblioteca `config` para leer `.taskcode/config.yml`. No añade subcomandos (D3).
- `config/claves.rs`: la tabla declarativa con las **12 claves de primer nivel**, cada una con su ADR de origen, tipo, valor por defecto y validación. Es la fuente única del validador, de los valores por defecto y del esquema. Las claves son:
  - `formato`;
  - `politica_carriles` y `rutas_carril`;
  - `verificacion`, `derivados` y `confianza_incluye`;
  - `plataforma`;
  - `recursos`;
  - `seleccion`;
  - `modo_flujo` y `tope_rondas`;
  - `lote`.
- `config/validar.rs`: recorre el árbol y junta **todos** los errores con la ruta de cada clave. Además:
  - comprueba las reglas que cruzan campos: `escribe` según el `tipo`, e `id` de recurso repetido;
  - aplica los valores por defecto de la tabla.
  - Reglas de texto:
    - orden sin caracteres de control ni saltos de línea;
    - glob sin `!`, rutas absolutas ni `..`;
    - `confianza_incluye` nunca abarca todo el árbol;
    - `escribe` normalizado bajo `tareas/`, sin las rutas reservadas ni sus alias de Windows;
    - cron de 5 campos con rangos;
    - URL de la API validada con `plataforma::http::Base`.
- `config/mod.rs`: la API pública.
  - `leer` es pura. Exige que la primera línea sea `---` y que exista `formato: 2`.
  - `de_base` lee de `origin/develop`, nunca del worktree, y no hace `fetch`.
  - `Config::suelo()` da la configuración por defecto. `Invalida::informe()` da `config rechazada motivo=<m> errores=<n> claves=…`, con código 3 y el primer error y su arreglo en stderr.
  - Motivos: `sin-config`, `formato` (es de la 0.16: `tc2 actualizar`), `formato-nuevo` (`formato` > 2: actualizar `tc2`), `yaml`, `config-invalida` y `sin-base`.
- `carril.rs` (nuevo): el sitio de la tabla de suelo de F1-10. Tiene los cuatro carriles, `roles_min` y el perfil.
- `yaml.rs` (nuevo): `opciones()`, que sale de `tarea::yaml` sin cambiar su comportamiento. Las usan `tarea` y `config`.
- **Esquema publicado**: `crates/tc2/esquemas/config.schema.json`.
  - Es JSON Schema 2020-12, generado con `config::esquema_json()`.
  - Lleva `additionalProperties: false` en cada objeto. El suelo de cada carril aparece como `minimum` o `enum`.
  - Se fija LF en `.gitattributes`.

**Evidencia.**
- `cargo xtask verificar --perfil rapida` en verde con la implementación. Tras cada corrección se repitieron clippy con `-D warnings` y la suite de `config`, `tarea` y la biblioteca.
- `tests/config.rs`, 8 tests:
  - el esquema publicado es igual que el generado (`TC2_ESCRIBIR_ESQUEMA=1` lo reescribe);
  - el esquema está cerrado, con `formato` como única clave obligatoria y el ADR en cada clave;
  - la tabla `esquema:inicio/fin` de F1-11 coincide hoja a hoja con el esquema;
  - valores por defecto, también los anidados;
  - 43 configuraciones hostiles, cada una con su motivo y su ruta: las de la PoC D de F1-10, las de F1-11, las claves de la 0.16, duplicadas, alias, falta de `---`, `formato` 1 o 3, BOM y dos errores a la vez;
  - la traducción de ToolsaWeb de F1-11 es válida;
  - `de_base` lee `origin/develop` y no un commit local ni el worktree;
  - contrato: `sin-config`, la 0.16, el esquema y `sin-base`, con código 3.
- Tests unitarios: el tope de 12 claves (`TOPE_CLAVES == 12`), las reglas de texto, las rutas de `escribe` y los carriles.

**Revisión.** Dos rondas, con `rust-wasm-reviewer` y `code-quality-reviewer`.
- **Ronda 1**: los dos pidieron cambios.
  - IMP-1 de Rust: «`escribe` prohibida en `comprueba`» no tenía test. Corregido.
  - IMP-1 de calidad: las reservas de `escribe` se saltaban con un punto o un espacio final (`tareas/ROADMAP.md.`, `tareas/.git./x`). Corregido.
  - MENORES corregidos:
    - alias `~` de 8.3;
    - mensaje de una orden con saltos de línea;
    - «todo el árbol» por segmentos y no con una lista;
    - cron con rangos;
    - listas sin repetidos;
    - arreglo de la 0.16 por la ruta y no por el texto del valor;
    - `formato: 3` con su propio motivo;
    - tests de normalización, `self-hosted` y el tope.
  - MEN-3 de calidad y MEN-5 de Rust, aceptados: ver «Divergencias con el plan».
- **Ronda 2** (verificación acotada):
  - Rust: aprobada, sin hallazgos nuevos.
  - Calidad: aprobada con correcciones. Sus dos MENORES venían de la corrección: `*.*` o `.*` se rechazaban como todo el árbol, y `~` en cualquier nombre. Corregidos y registrados con `verificar-menores`.

**Divergencias con el plan.**
- **`modo_flujo` y `tope_rondas` van planas** y no agrupadas en `flujo` como pedía D1. Quedan 12 de 12 claves, sin margen.
  - Motivo: F1-12 (decisión 6 y tabla de la configuración migrada) y F1-11 las nombran planas, y la 0.16 sigue leyendo `modo_flujo` mientras conviven.
  - El test del tope falla con la clave número 13. RF-43 y F2-8 no tienen sitio libre: si lo necesitan, tendrán que agrupar algo.
- **`modo_flujo` vale `manual` por defecto** y no `automatico` como recomendaba D10. Es el valor por defecto de la 0.16, verificado en el `CONFIG_DEFAULTS` de taskctl 0.16.1; el plan lo dejaba sin verificar.
- **`plataforma.api` es obligatoria** cuando se declara `plataforma`, porque F1-13 dice «con la URL de la API».
- **Suelo de roles de riesgo alto: 4**, el valor de la PoC de F1-10 («todos» son los cuatro roles base).
- **Perfiles**: `politica_carriles.<c>.perfil` usa la escala de F1-10 (`rapida`, `impacto`, `completa` y `completa y pesada`). Los perfiles de `verificacion` son los de F1-7.

**Pendiente o sin verificar.**
- **Dos descripciones del esquema publicado se han quedado atrás** frente a la corrección de la ronda 2. `confianza_incluye` dice «ningún segmento solo de `*` y `.`» y `escribe` dice «sin `~`». El validador ahora admite `*.*` y `.*` y solo rechaza `~` seguido de una cifra. Se revirtió el texto para que `verificar-menores` no integrara código sin revisar. Hay que alinearlo en la próxima tarea que toque `config`.
- **El esquema es más laxo que el validador** en lo que JSON Schema no expresa: reglas de texto (solo descritas), cron y `escribe` según el `tipo`. El que manda es el validador.
- **No se ha comprobado que un editor** (yaml-language-server) cargue el esquema.
- **`formato: -1`** recibe el arreglo de la 0.16, igual que cualquier otro valor que no sea 2 ni mayor que 2.
- **Este repo sigue con la configuración de la 0.16** (RR-14). Leída con `config` sale `motivo=formato`, que es lo esperado: nada del repo la lee.
- **Se planificó con 240 min** y sin trocear, por decisión de Carlos para el H2.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T21:13:01Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T21:23:05Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T21:23:08Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T21:41:24Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T22:06:51Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T22:19:19Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T21:22:48Z | diseno | agente:a24b1e29bef7bcb47 | 56 | 111726 | 1835655 | 28615 | 51 | 1976052 | - | - |
| 2026-11-15T21:41:24Z | implementacion | sesion:0c02a589-064f-41b2-898e-aecc1141ae21 | 104 | 112784 | 10748806 | 59279 | 26 | 10920973 | - | - |
| 2026-11-15T22:06:39Z | revision | agente:aa511526881b78cf4 | 126 | 189621 | 9443208 | 34022 | 14 | 9666977 | 1 | rust-wasm-reviewer |
| 2026-11-15T22:06:39Z | revision | agente:add420c2a5a24921f | 94 | 172612 | 6216763 | 29281 | 7 | 6418750 | 1 | code-quality-reviewer |
| 2026-11-15T22:15:02Z | revision | agente:a693fd6dabd87d706 | 28 | 131593 | 1427784 | 20660 | 2 | 1580065 | 2 | rust-wasm-reviewer |
| 2026-11-15T22:15:02Z | revision | agente:a7ca65a90c98fa2da | 42 | 152779 | 2403853 | 37263 | 5 | 2593937 | 2 | code-quality-reviewer |

## Autorrevision

- rust-wasm-reviewer — 29cf97f
- code-quality-reviewer — 29cf97f
- rust-wasm-reviewer — 8671970
- code-quality-reviewer — 8671970

## Verificacion ligera

- 2026-11-15T22:15:29Z: ronda 2; commit revisado 964898f7d7e9298ce912ae2a54c2b8b39e88f998; commit verificado 0fdb170ded4cce845ffbfcc87bc512a68bb52b9d; ficheros: crates/tc2/src/config/validar.rs, crates/tc2/tests/config.rs; corregidos: MEN-6, MEN-7; pendientes: ninguno

## Duracion

- Estimada: 240 min
- Real (de la rama abierta al cierre con merge, sin pausas): 56 min
- Cierre (finish): 187 s, de 2026-11-15T22:16:12Z a 2026-11-15T22:19:19Z

**Tras el cierre.** CI de develop en verde: run 43768661055, los 4 jobs en success (comprobar-repo, codigo en ubuntu, windows y macos).
