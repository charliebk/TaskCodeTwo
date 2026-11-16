# Plan — TASK-076: F2-4b Nombres unicos de tarea y rama

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 240 min

Se amplia `Alta` (`crates/tc2/src/alta.rs`), sin orden nueva. En `aplicar`, sobre el estado fresco de cada intento, se escribe `rama: feature/task-nnn-<slug>` (ADR F1-9 punto 6; hoy `rama: None` «hasta F2-4b», `tarea/modelo.rs:320`) y se recogen las tareas abiertas con titulo parecido. El aviso sale en la linea de `new` (`parecidas=`), con codigo 0. Antes de la transaccion se leen una vez las ramas `*/task-*` del remoto (`ls-remote --heads`) y locales; `siguiente_id` toma el maximo de carpetas y ramas: ninguna alta nombra una rama que ya existe.

Regla de «casi igual»: plegar el titulo (minusculas, sin tildes, lo no alfanumerico a espacio, espacios colapsados). Plegados iguales → `duplicado`. Si no, coeficiente de Dice sobre bigramas de caracteres del plegado sin espacios; >= 0,80 → `parecido`. Solo contra tareas en `planificada`, `en-diseno`, `en-curso` o `en-revision`.

Piezas:
- `alta.rs::Alta` (se extiende): `aplicar` escribe `rama` y llena `parecidas: RefCell<Vec<Parecida>>` (precedente: `id: Cell`); recibe `ocupados: BTreeSet<u32>`.
- `tarea::texto` (nuevo, puro): `plegar`, `slug` (ASCII `[a-z0-9-]`, maximo 40 bytes cortando en guion; vacio → `tarea`) y `parecido(a, b)`. Un solo plegado para slug y comparacion.
- `git::nombre_de_rama` y `rama_valida`: se reutilizan tal cual.
- `estado::Estado::leer_varios(&[String])` (se extiende): por defecto llama a `leer`; `Vista` usa un solo `git cat-file --batch` (hoy cada `leer` es un `git show`).
- `git/rama.rs::ids_con_rama` (nuevo, comparte parser con `ramas_del_id`): sin remoto mira solo la local; un fallo del remoto da `sinred` (4) o `denegada` (8) como `fetch`.
- `lib.rs::orden_new` e i18n: `rama=`, `parecidas=` y un diagnostico por parecida («TASK-007 "titulo" (duplicado|0,86)»).
- Adenda al ADR F1-2 punto 3: el ID es el maximo de carpetas y ramas + 1.

Orden de construccion:
1. `tarea::texto` y `rama` en `tarea.md` (C7).
2. `ids_con_rama`, `siguiente_id(rutas, ocupados)` y adenda F1-2 (C2, C3, C6).
3. `Estado::leer_varios` (mismo resultado con y sin lote; un solo proceso por intento segun el registro de `Git`).
4. `parecido`, su recogida en `aplicar` y la salida (C4, C5).
5. E2E de altas simultaneas desde dos clones (C1).

Alternativas descartadas: avisar cuando la rama del ID nuevo ya existe sin saltarse el ID (la tarea naceria bloqueada en `start`, ADR F1-3 PoC 1d); comparar titulos fuera de la transaccion (dos altas simultaneas no se verian, incumple C1); `avisos()` en el trait `Operacion` (obliga a tocar todas las operaciones).

## Lo que el rol no cubrio

- Sin roles de riesgos, testing ni dominio. Riesgo aceptado: una rama creada a mano entre el `ls-remote` y el push la debe parar `comprobar_id` en `start` (F3-6, aun no existe).
- Suposiciones sin verificar: `ls-remote --heads origin '*/task-*'` casa por sufijo (probar `refs/heads/a/b/task-1-x`); el puerto `Git` admite `cat-file --batch` por stdin (si no, ampliarlo); plegado sin crates con tabla fija para Latin-1 y Latin Extended-A (otros alfabetos se comparan en minuscula pero no entran en el slug); `tarea.md` ilegibles o en formato 1 se saltan sin fallar; re-derivar la rama si cambia el tipo queda fuera.

## Riesgos aceptados y que los contiene

- Un viaje de red mas por `new` (`ls-remote`), una vez y no por intento; sin red, `sinred` sin efectos (C6).
- N tareas crece siempre: lectura en lote con `cat-file --batch`.
- La primera de dos altas simultaneas no puede avisar (la otra aun no existe); lo avisa la segunda (C1).

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):
- C1: E2E en `tests/alta.rs` con dos clones en paralelo.
- C2, C3: ramas creadas a mano (remoto y local) sin carpeta.
- C4: duplicado frente a tarea abierta y frente a terminada.
- C5: tabla unitaria del umbral con los dos pares de ejemplo.
- C6: remoto inalcanzable, sin refs nuevas.
- C7: unitarios de `slug` (incluido titulo sin letras latinas) y `rama=` en los E2E.

## Lo que necesita decision de una persona

Carlos acepta las recomendaciones (consigna del 2026-11-16):
- D1: el duplicado exacto es solo aviso con codigo 0, no bloquea (RR-12 pide detectar).
- D2: umbral 0,80 como constante en el codigo hasta que F9-4 mida el ruido.
- D3: se reabre por escrito el ADR F1-2 punto 3 con una adenda (ID = maximo de carpetas y ramas + 1).
