# Plan — TASK-032: F3-5 tc2 board y tc2 status

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 240 min

Modulo nuevo `tablero.rs` con dos ordenes de solo lectura que salen de la misma `situacion::calcular` sobre `estado::leer` (misma foto y mismo `commit=` que `next` y `roadmap`). `status` no toca la red: una linea de equipo sin plataforma. `board` es la vista completa y anade los PR abiertos de la plataforma con una operacion nueva de lectura del puerto. Ninguna escribe ni hace `fetch` (la frescura es de F3-5b, TASK-079).

Piezas:
- `situacion.rs` (se extiende): sube el «quien» hoy privado de `roadmap/generar.rs` (`con_quien`) y `Ficha` gana `rama`, para que board, status y roadmap digan lo mismo.
- `tablero.rs` (nuevo): funcion pura `tablero(s, personas, prs) -> Tablero`, vistas `texto_board` y `linea_status`, e `Informe` de cada orden.
- Puerto `Plataforma` (se extiende): `prs_abiertos(&self, rol) -> Paso<Vec<PrAbierto>>` con `PrAbierto = {numero, rama, sha, borrador}`, paginacion existente (`conexion::leer_paginas`, `MAX_PAGINAS`), en GitHub y GitLab con la misma linea (RR-15); adenda a F1-4/F1-13 (operacion nueva en la lista cerrada).
- `lib.rs` (dos ramas mas con `con_json`) e `i18n/catalogo/tablero.rs`.
- `estado/` y `transaccion` no se tocan; tablero → situacion y puerto, nunca al reves.

Contrato de salida:
- `status` texto: una linea acotada a K tareas con «+N»; `status --json`: `status ok en-curso=… en-revision=… listas=… commit=…`.
- `board` texto: tablas por grupo; terminadas y descartadas solo como recuento; avisos en stderr con codigo 0.
- `board --json`: claves planas (`commit=`, `plataforma=`, `en-diseno=`, `en-curso=`, `en-revision=`, `listas=`, `quien=ID@login,…`, `prs=ID#n,…`, avisos, `texto=`), contrato plano de F1-4.
- Sin rama de estado: `rechazada motivo=sin-rama-de-estado`, como `next`.

Orden de construccion:
1. «Quien» y `rama` a `situacion` (`roadmap` sigue dando su referencia byte a byte).
2. `tablero.rs` puro y `tc2 status [--json]` sin red.
3. `tc2 board [--json]` solo con estado, `plataforma=no-consultada`.
4. Adenda ADR y `prs_abiertos` en los dos adaptadores, con fakes.
5. `board` cruza PR por `rama` y anade los modos degradados.

Alternativas descartadas: `estado_puertas` por tarea (no da el numero de PR y cuesta N llamadas); JSON anidado por tarea (rompe el contrato plano de F1-4; la API estable es `tareas --json`, F1-11 decision 5).

## Lo que el rol no cubrio

- Estrategia de pruebas y de fakes HTTP, casos limite (ilegibles, IDs duplicados, PR de forks) y dominio («espera revision» con PR en borrador, textos de grupos).
- Suposiciones sin verificar: que la API de MR de GitLab da rama de origen y borrador con permiso de lectura; que el tope de 30 s del cliente HTTP es global; casar PR por `head.ref == tarea.rama` (falla con rama renombrada o fork); que el rol `Agente` basta para listar PR.

## Riesgos aceptados y que los contiene

- `board` sin red puede tardar hasta el tope HTTP: `--sin-plataforma`.
- Mas de `MAX_PAGINAS` de PR: falla cerrado con `plataforma=error` y la parte de estado sale.
- `status` usado como status line cuesta O(N): se documenta; la status line es la de F3-12.
- Titulos y logins son texto ajeno: se sanean al imprimir (F1-17 decision 8).

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):
- C1: integracion con dos clones y dos caches.
- C2: integracion con instantanea antes y despues.
- C3a: unitario sobre `EnMemoria` con el estado de referencia de `roadmap`.
- C3b: unitario con fake de `Plataforma`.
- C3c: fakes que fallan y test de que `status` no construye adaptador.

## Lo que necesita decision de una persona

Carlos acepta las recomendaciones (consigna del 2026-11-16):
- D1: `status` es una linea de equipo, no la status line de Claude Code (que sigue en F3-12, `estado --stdin`).
- D2: `prs-abiertos` entra en la lista cerrada del puerto con adenda a F1-4/F1-13, campos minimos.
- D3: sin checks ni aprobaciones por PR en F3-5; solo numero y borrador.
- D4: `board` consulta la plataforma por defecto, con `--sin-plataforma`.
- D5: nada personal en ninguna de las dos salidas (por C1).
- D6: criterios C1..C3c en lugar de los vagos.
