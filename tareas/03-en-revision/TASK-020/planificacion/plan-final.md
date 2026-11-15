# Plan — TASK-020: F2-5 Modelo de tarea y parser/serializador de frontmatter YAML valido

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 55 min (de la rama abierta al cierre con merge; señales: ~4 ficheros de código nuevos en `crates/tc2/src/tarea/` más `alta.rs` y `Cargo.toml`, 1 capa (modelo puro de texto a texto, sin git ni red), 3 criterios; TASK-019 se estimó en 60 y se cerró en 23 reales; si entraran Spec Kit o el corpus de los otros repos en la CI pasaría de 60).

**Forma.** Módulo nuevo `crates/tc2/src/tarea/`, hoja pura (no depende de `estado`, `git` ni `contrato`); `alta` pasa a depender de él.

- `tarea::yaml` (privado): la costura que pide F1-14 («el YAML vive detrás de un módulo propio… para poder cambiar de crate»).
  - Lectura: un separador propio parte el frontmatter en entradas de primer nivel (línea `clave:` en columna 0 más sus líneas sangradas) y guarda el texto crudo de cada una; `serde-saphyr` valida el documento entero (YAML válido, claves duplicadas como error) y decodifica las conocidas en un `struct`.
  - Escritura: emisor propio, porque F1-9 fija la forma byte a byte (comillas dobles siempre con `\` y `"` escapados, orden de claves, `roles` en flujo); un serializador genérico no da esa forma.
- `tarea::modelo`: `Tarea` (formato 2) y `TareaV1` (los 22 campos de `TASK_FIELD_ORDER`); claves desconocidas como `Vec<(clave, texto crudo)>` en ambos. El esquema estricto de los campos conocidos vive aquí.
- `tarea::cuerpo`: `Cuerpo { preambulo, secciones: Vec<Seccion { titulo, texto }> }`, reconocidas por título y no por posición (F1-9 punto 4); escribir es concatenar los bytes originales.
- API pública mínima (formato 1 y 2 se distinguen por la presencia de `formato:`; el 1 se lee pero no se escribe, F1-9 punto 3):

```
pub fn leer(texto: &str) -> Result<Documento, ErrorTarea>; // Documento::{Formato1(TareaV1, Cuerpo), Formato2(Tarea, Cuerpo)}
impl Tarea { pub fn nueva(id: &str, titulo: &str, fecha: &str) -> Self; pub fn escribir(&self, c: &Cuerpo) -> String; }
impl Tarea { pub fn claves_desconocidas(&self) -> &[(String, String)]; } // para el aviso de doctor (D4a), que no es de aquí
```

**Absorción de `alta.rs`.** Desaparecen `PLANTILLA` y `escapar`; `tarea_md()` pasa a ser `Tarea::nueva(..).escribir(&Cuerpo::plantilla())`. Anotación y catálogo no cambian. El primer paso exige salida **byte a byte igual** a la plantilla actual (incluido `rama: null`): los tests existentes de `alta` (`plantilla_escapa_el_titulo_y_sigue_f1_9`, `tests/alta.rs`) son la red del cambio.

**Orden de construcción.**
1. Dependencias (D1) y `tarea::yaml` con el emisor; prueba dorada: misma salida que `alta::tarea_md` hoy.
2. `Cuerpo` por secciones y `Tarea::nueva`; `alta` los usa y se borra la plantilla.
3. Lector del formato 2: esquema estricto, duplicadas rechazadas, desconocidas conservadas; ida y vuelta.
4. Lector del formato 1 (`TareaV1`) y corpus de las tareas 0.16 de este repo.
5. Prueba con el parser estándar independiente (D2).

**«Ida y vuelta con un parser estándar»** se comprueba con: (i) `escribir(leer(t)) == t` byte a byte para un formato 2 canónico; (ii) `leer(escribir(m)) == m`; (iii) un parser YAML **de otra base de código** que el de producción, solo en `[dev-dependencies]`, carga lo que escribe el emisor y da los mismos valores (`C:\ruta\`, `"`, `: #`, ñ, claves desconocidas). Con el mismo crate de producción la prueba sería circular.

**Alternativas descartadas.** Parser propio de subconjunto sin crate (contradice F1-14, fila «F1-9 YAML estándar que rechaza duplicadas → serde-saphyr con duplicadas como error», y rechazaría YAML válido escrito a mano). Serializar con serde y guardar las desconocidas como `Value` (no da la forma fija de F1-9 y obliga a un emisor genérico).

## Lo que el rol no cubrio

- **Testing**: la batería no la ha diseñado un rol de testing; el presupuesto de abajo es de arquitectura.
- **Riesgos**: los modos de fallo del YAML hostil (anclas y alias que crecen, etiquetas `!!`, ficheros enormes, BOM, CRLF, tareas 0.16 mal formadas) no los ha revisado un rol de riesgos. Se cubren en implementación con tests mínimos: BOM y CRLF se aceptan; anclas, alias y etiquetas se rechazan en tareas (no aparecen en ninguna real).
- **Dominio**: valores permitidos de `estado`, `tipo` y `complejidad`, y `motivo` obligatorio con `descartada`, se toman de la tabla de F1-9 tal cual.
- **Criterio 3**: F1-9 («Consecuencias») pide además a F2-5 una ida y vuelta con un proyecto Spec Kit y ver en GitHub cómo se pinta `roles`; quedan fuera (D3) y se anotan como divergencia en el `## Resultado`.
- La plantilla de `plan-final.md` (F1-9 punto 7) no se convierte aquí: la escribe `plan` (F4-1).

## Riesgos aceptados y que los contiene

- **Crate joven** (`serde-saphyr` 2.1.1): lo contienen la costura `tarea::yaml` y la batería de ida y vuelta.
- **Títulos del formato 1 con `\`** (escape de la 0.16, H11): si el texto crudo de una cadena entre comillas de un formato 1 trae `\`, el lector falla con un error claro en vez de leer mal en silencio.
- **Posición de las claves desconocidas**: se reescriben al final del frontmatter en su orden original; la ida y vuelta byte a byte se garantiza solo para el formato 2 canónico.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **Ida y vuelta con un parser estándar — 3**: (i) byte a byte sobre la plantilla y sobre una tarea con secciones fuera de orden y texto libre; (ii) `leer(escribir(m)) == m` con títulos hostiles y claves desconocidas; (iii) parser independiente de `dev-dependencies` contra el modelo. Ninguno pesado.
- **Compatible con tareas de la 0.16 — 2**: corpus de todas las `tareas/0*/TASK-*/tarea.md` de este repo (22 campos, cuerpo byte a byte; no pesado, unas 100 lecturas en memoria); corpus externo (ToolsaWeb, OpenDockTrading) por `TC2_CORPUS` con `#[ignore]`, **pesado y manual**, con la salida en `## Resultado`.
- **Cumple F2-5 y lo que cita — 1**: esquema estricto (tipo incorrecto, `id` o `titulo` ausente y clave duplicada dan error). Los tests de `alta` existentes se conservan sin cambios como regresión.

## Lo que necesita decision de una persona

Decididas con la opción recomendada, por la consigna de Carlos para el H2 («aceptamos todas las preguntas con la respuesta recomendada»):

- **D1 → (a)**: `serde` + `serde-saphyr` en `[dependencies]` solo para leer, con `Cargo.lock` versionado (la CI ya usa `--locked`), emisor propio y todo detrás de `tarea::yaml`. Es lo que decidió F1-14 (punto 1 y fila F1-9).
- **D2 → (a)**: parser de otra base de código en `[dev-dependencies]`. Al volcar el plan se vio que `saphyr` deriva de `yaml-rust2`, así que no son independientes; se usa `serde_yaml` 0.9 (port de libyaml, la misma base que el PyYAML del PoC de F1-9; archivado, pero solo en tests y congelado en el lock). PyYAML queda como cruce manual opcional.
- **D3 → (a)**: entran el modelo, el formato 1, el corpus de este repo y el corpus externo manual; Spec Kit y la vista en GitHub pasan a otra tarea o a F2-6, con la divergencia escrita en el `## Resultado`.

## Suposiciones verificadas al volcar

- `serde-saphyr` 2.1.1 y `yaml-rust2` 0.13.0 están en crates.io (`cargo search`).
- La CI y `xtask verificar` usan `--locked`: bajan los crates fijados en el lock versionado.
- Pendiente de comprobar en implementación: que `serde-saphyr` decodifica a `String` un escalar plano con aspecto de fecha (`creado: 2026-11-14` del formato 1).
