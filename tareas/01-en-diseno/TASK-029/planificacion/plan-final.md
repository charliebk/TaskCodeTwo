# Plan — TASK-029: F3-2 tc2 new e import en bloque con dependencias e hito

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 300 min

`import` es una sola `Operacion` sobre la transaccion de F2-3: un commit, un push y N IDs consecutivos (ADR F1-2 punto 3). El alta de una tarea y la de un lote comparten un nucleo unico `altas(estado, &[Entrada], ocupados)` sacado de `alta.rs`; `new` pasa a ser un lote de 1, asi no hay dos comportamientos para reservar ID, derivar la rama ni avisar de titulos parecidos. El fichero conserva la forma del import de la 0.16 (`### titulo`, `> objetivo`, `- criterio`; AMPLIACION §5) y anade una cabecera YAML de lote y una linea opcional de dependencias por entrada.

Formato: cabecera `---\nlote: fase-3\nsprint: 3\n---`; por entrada `### <clave> <titulo>`, `> objetivo`, `- criterio` y opcional `Dependencias: F2-4, TASK-019`. Sin cabecera se lee como 0.16 y el sprint llega por `--sprint N`.

Referencias locales: la clave es el primer token del `###`; los IDs se reservan en el orden del fichero; una referencia se resuelve al mapa del lote o a una `TASK-NNN` existente; ciclo o inexistente → `uso` (2) sin commit.

Atomicidad: todo o nada en una transaccion.

Idempotencia: una tarea importada se reconoce por `lote/clave` guardado en la tarea; al repetir se crean solo las que faltan, las existentes no se actualizan (un import no sincroniza) y salen en `existentes=`; si no falta ninguna, `SinCambios` sin commit.

Limites: 1 MiB y 200 entradas; titulo segun `titulo_valido`. Cada intento lee todos los `tarea.md` con un solo `leer_varios`; O(tareas + N) por intento, maximo 5 intentos.

Piezas:
- `alta.rs` (se extiende): nucleo puro `altas(...)` con `siguiente_id`, mapa clave → ID, `rama_de` con `tipo` como parametro y `parecidas`; `Alta` mantiene su API y su anotacion.
- `importar/fichero.rs` (nuevo, puro): texto → `Lote { sprint, entradas }`; limites, claves unicas y ciclos antes de tocar la red.
- `importar/mod.rs` (nuevo): `Importar` implementa `Operacion` con `exige_red() = true` y anotacion `import|<clave-de-lote>|<n>`; resuelve claves a los IDs del intento y valida externas contra el estado fresco.
- `tarea::Tarea` y la plantilla: `tarea_md` recibe la `Entrada` (sprint, tipo, dependencias, objetivo, criterios).
- `lib.rs`: `import [--json] [--simular] [--sprint N] <fichero>`; `orden_new` acepta los campos de RF-01.
- Dependencia en un solo sentido: `importar` → `alta` → `tarea`/`estado`.
- Adenda en F1-9 por la clave nueva `lote/clave` (el test que fija el orden de claves se actualiza).

Orden de construccion:
1. Sacar el nucleo `altas` sin cambiar comportamiento.
2. Parser puro con limites, claves, ciclos y compatibilidad 0.16.
3. Operacion `Importar` con resolucion, idempotencia, `SinCambios` y salida `creadas=`/`existentes=`/`ids=`.
4. `--simular` con «plan como dato» (patron de `init` y `desinstalar`).
5. Campos de RF-01 en `new` y rechazo de un enunciado no listo.

Alternativas descartadas: una transaccion por tarea (contradice F1-2; una rafaga de N pushes acaba en `Agotada`); reconocer lo importado por titulo normalizado (un titulo editado duplicaria).

## Lo que el rol no cubrio

- Modos de fallo (YAML hostil, carreras con `start`, BOM/CRLF), que es del rol de riesgos; reglas de dominio (que es un «enunciado no listo» en RF-01; si una dependencia puede apuntar a una tarea descartada); diseno fino de los tests.
- Suposiciones sin verificar: que la 0.16 recibe el sprint por flag y no admite dependencias; que un vector de cambios vacio acaba en `SinCambios` sin commit (`estado/transaccion.rs::ejecutar`); que hay lectura fresca sin transaccion para `--simular`; que `tarea::leer`/`escribir` conserva una clave nueva.
- RF-03 «saber que esta listo» es de F3-3 (`next`): aqui solo se guardan las dependencias.

## Riesgos aceptados y que los contiene

- `lote/clave` en el modelo toca el contrato de F1-9: adenda y test del orden de claves actualizado.
- Generalizar `rama_de` con `tipo` rompe «tipo y rama salen de la misma constante»: test de `rama_valida` por tipo.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):
- `crates/tc2/tests/importar.rs` con remoto bare y fixtures de lote: C1..C8 (C5 reutiliza el arnes de dos clones de F2-4; C6 compara bytes antes y despues como `git/puerto.rs:198`).
- C9 en `tests/alta.rs`.
- Unitarios del parser en `importar/fichero.rs` (limites, claves, ciclos, formato 0.16).

## Lo que necesita decision de una persona

Carlos acepta las recomendaciones (consigna del 2026-11-16, sin trocear):
- D1: lo importado se reconoce por `lote/clave` en el frontmatter, con adenda a F1-9.
- D2: el hito es `sprint`, sin campo nuevo (el «hito de decisiones» de F1-18 queda fuera).
- D3: RF-01 (campos y rechazo de enunciado no listo) en `new` dentro de esta tarea.
- D4: topes de 1 MiB y 200 entradas, como parametro.
