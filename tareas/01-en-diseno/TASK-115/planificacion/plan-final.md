# Plan — TASK-115: Seguimiento de F2-12: menores de actualizar --cli y mock HTTP estable

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 120 min

Tres arreglos independientes:
1. `tests/comun/http_mock.rs::atender`: tras una respuesta completa no hace `shutdown`/`drop` (que manda el FIN que provoca el cuelgue); hace `set_read_timeout(30s)` + `read_to_end` y deja cerrar al cliente, como `tests/actualizar_cli.rs:155-165`. `Mock::nuevo` atiende cada conexion en su `thread::spawn` para que la espera no encole al resto. El contrato publico (`Mock`, `Simulada`, `Recibida`) no cambia.
2. Tests de los menores de TASK-027: `ejecutar_con_limite` con nieto que retiene stdout (`src/actualizar_cli.rs`), TLS fallido no transitorio y timeout global que no se dobla (`src/descarga.rs`), redireccion no permitida y cuerpo mayor que el limite sin reintento (`tests/actualizar_cli.rs`, ampliando su mock con 302 y `Location`).
3. Doc de `bajar_con_un_reintento` (`src/actualizar_cli.rs:600-611`) pegado a la funcion, con test de guarda.

Orden: mock compartido y su test de «no hay FIN» → 10 de 10 de `plataforma` y `plataforma_gitlab` con evidencia → doc y su guarda → tests de `descarga.rs` → test del nieto e integracion de `actualizar_cli`.

Alternativas descartadas: keep-alive HTTP/1.1 en el mock (cambia el contrato); helper comun de cierre usado por `actualizar_cli.rs` (su mock corta a proposito; duplicar dos lineas es mas barato).

## Lo que el rol no cubrio

- Sin roles de riesgos ni testing. Suposiciones sin verificar: que el cliente de `plataforma*` cierra tras `Connection: close` (pool del agente); que `Descargador` admite un timeout global inyectable; la forma exacta con la que `Fallo::Sinred` distingue TLS; el mock de `actualizar_cli.rs` hay que ampliarlo con 302.

## Riesgos aceptados y que los contiene

- Un cliente que no cierre retiene un hilo hasta 30 s; con un hilo por conexion no bloquea a las demas.
- Test del nieto: en Windows `start /b` hereda handles; plazo corto para no dejar `ping` huerfanos.
- Test del timeout mide tiempo: margen de 1,5x.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):
- C1: `ejecutar_con_limite_devuelve_lo_leido_si_un_nieto_retiene_stdout`.
- C2: `tls_fallido_no_es_transitorio`, `timeout_global_no_se_dobla`.
- C3: `redireccion_no_permitida_no_se_sigue_ni_reintenta`, `binario_mayor_que_el_limite_se_rechaza` (una sola peticion registrada).
- C4: test determinista en `tests/comun` (tras leer el `Content-Length`, la lectura siguiente da `WouldBlock`/`TimedOut` y no `Ok(0)`); evidencia «10 de 10» con un script fuera de la suite anotado en `## Resultado`.
- C5: unit test con `include_str!` que exige que el bloque `///` sobre `fn bajar_con_un_reintento(` no este vacio, no tenga lineas repetidas y no venga detras de una `const`.

## Lo que necesita decision de una persona

Carlos acepta las recomendaciones (consigna del 2026-11-16): el «10 de 10» se demuestra con el script y su salida en `## Resultado`; el test determinista de «no hay FIN» es el que guarda el criterio en la suite.
