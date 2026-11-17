---
id: TASK-115
titulo: "Seguimiento de F2-12: menores de actualizar --cli y mock HTTP estable"
tipo: fix
sprint: 0
etiquetas: [distribucion, tests]
complejidad: simple
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: fix/task-115-seguimiento-de-f2-12-menores-de-actualiz
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: 28030245
tokens_revision: null
creado: 2026-11-16
actualizado: 2026-11-17
dependencias: []
---
## Objetivo

Cerrar los MENORES aceptados en TASK-027 y estabilizar tests/comun/http_mock.rs, que da sinred intermitente en esta maquina.

## Criterios de aceptacion
- [ ] ejecutar_con_limite devuelve la salida ya leida cuando un nieto retiene stdout y su doc dice lo que hace (test con nieto, MEN-7 r3)
- [ ] Un fallo de TLS no se reintenta y el timeout global no dobla la espera (tests en crates/tc2/src/descarga.rs, MEN-8 y MEN-9 r3)
- [ ] Tests de integracion de redireccion no permitida y de limite de tamano en crates/tc2/tests/actualizar_cli.rs (MEN-5 r1)
- [ ] tests/comun/http_mock.rs no manda FIN tras una respuesta completa (test determinista en tests/comun: tras leer el Content-Length la lectura siguiente agota el plazo y no da Ok(0)) y plataforma y plataforma_gitlab pasan 10 de 10 seguidas (evidencia del script en ## Resultado)
- [ ] El doc de bajar_con_un_reintento esta sobre su funcion y sin lineas duplicadas (test unitario con include_str! en crates/tc2/src/actualizar_cli.rs)

## Resultado

Test de cada criterio (cada uno se puso rojo con una mutacion que revierte
el cambio, salvo C5, rojo por construccion con el doc antiguo):

- C1: `actualizar_cli::tests::ejecutar_con_limite_devuelve_lo_leido_si_un_nieto_retiene_stdout`
  (mutacion: devolver vacio si vence el plazo → `stdout: ""`).
- C2: `descarga::tests::tls_fallido_no_es_transitorio` y
  `descarga::tests::timeout_global_no_se_dobla` (mutacion: `Global` y
  `InvalidData` transitorios → `transitorio: true` en los dos).
- C3: `redireccion_no_permitida_no_se_sigue_ni_reintenta` y
  `binario_mayor_que_el_limite_se_rechaza` en `tests/actualizar_cli.rs`
  (mutacion: reintentar cualquier fallo → 2 peticiones al binario).
- C4: `comun::http_mock::tras_una_respuesta_completa_el_mock_no_manda_fin`
  (mutacion: soltar el socket tras responder → `Ok(0)`).
- C5: `actualizar_cli::tests::el_doc_de_bajar_con_un_reintento_esta_sobre_su_funcion`.

Suposiciones del plan comprobadas:

- Pool del cliente: ureq-proto 0.7.1 marca `must_close_connection` con
  `Connection: close` de la respuesta (`client/prepare.rs`); la conexion no
  vuelve al pool. Por lectura y por las series de abajo (ningun cuelgue de
  30 s atribuible a reutilizar una conexion).
- `Descargador` no admitia un plazo inyectable: se anade
  `Descargador::con_espera` (pub(crate)); `nuevo` la usa con 120 s.
- TLS: el handshake de rustls no llega como `ureq::Error::Tls` sino como
  `ureq::Error::Io` de tipo `InvalidData` con el `rustls::Error` dentro
  (`complete_io`); se distingue por ese tipo, sin dependencia nueva. Un
  cierre de la otra parte a mitad del handshake da un `UnexpectedEof` que
  no se distingue de un corte y se sigue reintentando (documentado).
- El plazo global llega como `Timeout(Global)` en `call()` y, en el cuerpo,
  como `io::Error` que envuelve ese `ureq::Error`; los dos dejan de ser
  transitorios.

Evidencia del «10 de 10» (bucle bash, `cargo test -p tc2 --test <t> -q`,
en serie y sin otro cargo; 2026-11-17, commit 1e81d0e, con el cerrojo
`comun::http_mock::serie()`):

- `plataforma`: **10 de 10**. Tiempos (s, del bucle; entre parentesis el
  `finished in` de libtest): 5,9 (5,02) / 5,8 (4,99) / 5,5 (4,78) /
  5,6 (4,79) / 6,1 (5,20) / 5,3 (4,64) / 5,5 (4,81) / 5,3 (4,58) /
  5,2 (4,55) / 5,2 (4,53); 55,2 s en total.
- `plataforma_gitlab`: **10 de 10**. Tiempos (s): 4,6 (3,80) / 6,3 (5,27) /
  5,6 (4,74) / 4,5 (3,81) / 4,5 (3,85) / 6,9 (5,93) / 8,2 (6,59) /
  7,2 (5,71) / 5,3 (4,30) / 3,9 (3,28); 57,0 s en total.
- Ninguna vuelta con paradas de ~10 s.

Serie anterior, sin el cerrojo (commit cf24f2c, mismas condiciones), que
llevo a ponerlo:

- `plataforma`: **9 de 10**. Tiempos (s): 3,3 / 52,1 (FALLA) / 43,7 / 3,1 /
  13,4 / 3,1 / 13,2 / 13,5 / 43,9 / 74,8. La 2 cae con `sinred` a la vez
  en tres tests (`errores_dirigidos_a_la_persona`,
  `bateria_de_la_poc_m_e_idempotencia`, `el_envoltorio_rechaza_antes_de_la_red`).
- `plataforma_gitlab`: 10 de 10, con paradas: 13,0 / 2,3 / 2,3 / 12,6 /
  42,8 / 2,5 / 2,1 / 12,2 / 2,4 / 12,5 s.
- Linea base con el mock anterior (ee0dc13), sin cerrojo, `plataforma`:
  10 de 10, pero 9 de 10 vueltas con paradas: 23,4 / 3,0 / 23,3 / 13,1 /
  23,3 / 23,1 / 13,3 / 33,3 / 13,2 / 23,3 s.

Causa de las paradas (investigada; en el repo solo se evita, no se corrige):
no es el FIN del mock.
Con trazas temporales en el mock y en `Cliente::llamar`, dos peticiones de
tests distintos se quedan paradas a la vez ~10 s (una ya aceptada por el
servidor sin que llegue la peticion; otra sin llegar a conectar) y se
reanudan en el mismo milisegundo: es un bloqueo de todo el proceso
alrededor de la red. Con un solo hilo de tests (`--test-threads=1`) 8 de 8
vueltas en ~4 s sin paradas; con clientes Python (`python.exe`) en paralelo,
900 peticiones sin ninguna parada. En la maquina esta activo Avast (Defender
en pasivo), coherente con el filtrado de red de un `.exe` sin firma ya visto
en TASK-027. El adaptador no reintenta: una parada de 10 s solo retrasa, y
una llamada falla cuando acumula mas de 30 s (`ESPERA_TOTAL`). El arreglo
del mock (sin FIN) es necesario pero no bastaba para el «10 de 10» en esta
maquina. Decision del coordinador: poner en serie, dentro del repo, los
tests que usan el mock (`comun::http_mock::serie()`, cogido al principio de
cada test de `plataforma.rs`, `plataforma_gitlab.rs` y del test del mock),
sin `--test-threads=1` global ni tocar el antivirus ni la configuracion del
sistema (eso lo decide Carlos). Con el cerrojo, C4 queda cumplido: test
determinista y los dos «10 de 10» de arriba.

## Autorrevision

- rust-wasm-reviewer — 1e81d0e

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T03:47:05Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T03:48:28Z | approve | automatico | persona | 0.16.1 |
| 2026-11-17T03:48:29Z | start | automatico | persona | 0.16.1 |
| 2026-11-17T04:42:50Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-17T04:42:50Z | implementacion | sesion:f1858935-5859-4139-a2e3-cdd4f3fa7d06 | 14 | 11853 | 4062700 | 3225 | 0 | 4077792 | - | - |
| 2026-11-17T04:42:50Z | implementacion | agente:ada497c0c22a4f5e3 | 256 | 270586 | 23580049 | 101562 | 34 | 23952453 | - | - |
