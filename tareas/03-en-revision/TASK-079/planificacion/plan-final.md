# Plan — TASK-079: F3-5b status y roadmap siempre frescos

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 300 min

Un solo punto de lectura fresca en `estado/` que usan las cuatro ordenes de lectura, apoyado en el fetch que ya hace la transaccion (no se copia) y con la hora guardada como la cache de identidad.

Existente (leido): `estado/transaccion.rs` (`intento`, 274-305) hace `fetch -q origin refs/heads/taskcodetwo/tareas:refs/remotes/origin/taskcodetwo/tareas` en linea y clasifica con `clasificar_fallo_remoto`; `estado::leer` (`lectura.rs:158`) lee esa ref sin fetch; `status`, `board`, `next` y `roadmap` usan `estado::leer` y dan `commit=` (C3a de `tablero.rs` exige el mismo `commit=` en board/next/roadmap); `identidad::Cache` (`identidad.rs:207-295`) guarda en `<git-common-dir>/taskcodetwo/` con temporal y `rename`; `alta::fecha_de_dias` da la fecha civil; el puerto fija `GIT_TERMINAL_PROMPT=0`, sin tiempo limite ni `GCM_INTERACTIVE`.

Orden:
1. `huella.rs`: `FICHERO_SINCRONIZACION = "sincronizacion"` en `.git/taskcodetwo/` (F1-12 D1a; `desinstalar` lo reconoce).
2. `estado/frescura.rs` (nuevo): `Marca` (`leer() -> Option<u64>`, `registrar(ahora)`, temporal y `rename`, fichero corrupto = «nunca»); `enum Remoto { Consultado, SinRed, Denegado(motivo), ErrorLocal, SinRemoto }`; `Frescura { remoto, sincronizado }` con `anotar(Informe)` (claves `remoto=`, `sincronizado=`) y `aviso(idioma, ahora)`, unica fuente del formato. Hora `AAAA-MM-DDTHH:MM:SSZ` ampliando el helper de `alta` (no se copia).
3. `traer_estado(git, principal) -> Result<Remoto, Informe>` extraido de `transaccion::intento` a `estado`; la transaccion lo usa sin cambiar comportamiento. Transaccion y lectura llaman a `Marca::registrar` cuando el fetch sale bien.
4. `estado::leer_fresco(git, dir, ahora) -> Result<(Vista, Frescura), Informe>`: `referencia_de_lectura`, `traer_estado` si hay remoto, `registrar`, y la `Vista` sobre la misma ref; un fallo del remoto no es error (se lee lo ultimo y `Frescura` dice por que); no toma el cerrojo de estado. `leer` se queda sin fetch (doctor y tests).
5. Consumidores: `tablero.rs` (`status`, `board`), `siguiente.rs` y `roadmap/mod.rs` (`imprimir_en`, `comparar_en`) pasan a `leer_fresco` con `anotar` y aviso. `status` anade `pendientes=N` (`pendientes::listar`, F1-2 D5) y los avisos `tareas=con-cambios|diverge` (D5a). El stdout de `roadmap` sigue siendo byte a byte el `ROADMAP.md`: la hora va en stderr y en JSON, nunca dentro del ROADMAP (si no, `--comparar` saldria siempre desfasado). `doctor` no hace fetch y solo anade `sincronizado=` leyendo la `Marca` (D4a). Despues: i18n, adenda de F1-4 (claves nuevas), nota en F1-2 y comentarios «la frescura es de F3-5b».

Dependencias: ordenes → `estado` → `git`, `huella`; sin ciclos. Coste: un fetch de una ref por orden de lectura (~0,8-1,4 s contra GitHub, extrapolado de la PoC de F1-19; milisegundos contra el bare). F1-19 (TTL) esta descartado desde el 2026-11-16: se queda «traen antes de mostrar».

## Lo que el rol no cubrio

- Riesgos sin analizar (solo anotados): un `cannot lock ref` entre dos fetch a la vez clasificado como `sin-red`.
- Profundidad de las pruebas.
- Dominio: umbral de «hace horas» para un aviso mas llamativo (no lo pide el criterio; no se propone).
- Sin verificar: que `doctor/estado.rs` es donde `doctor` saca `commit=`; que `desinstalar` borra todo `.git/taskcodetwo/`.
- Nota de quien orquesta: el caso sin red no se simula con `origin` a una ruta inexistente (eso da `denegada`, aprendido en TASK-033); se usa el ssh que responde «Connection refused» de `tests/toma.rs`/`tests/ciclo.rs`.

## Riesgos aceptados y que los contiene

| Riesgo | Que lo contiene |
|---|---|
| ~1 s mas por orden de lectura | Es la letra del plan; el TTL era D2b. |
| Sin tiempo limite en el puerto, un host mudo cuelga `status` el timeout de TCP | La transaccion ya lo arrastra; arreglarlo es tocar el puerto. |
| En Windows el gestor de credenciales puede abrir ventana | El puerto no fija `GCM_INTERACTIVE=never`; mismo caso que la transaccion; no verificado. |
| Un `git fetch` a mano no renueva la marca | La hora es conservadora: nunca mas fresca de lo real. |
| Reloj desajustado | La hora es local a la maquina. |

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Unitarias en `frescura.rs`: ida y vuelta de la marca, fichero corrupto o enorme = «nunca», formato UTC (algoritmo de Hinnant), `anotar` y `aviso` por cada `Remoto` en es y en.
- Integracion con remoto bare (como `start` y `descartar`): criterios C1..C9; regresion de la transaccion tras extraer `traer_estado`; C3a de `tablero` sigue verde.

## Lo que necesita decision de una persona

Carlos dejo dicho que en el H3 se acepta la opcion recomendada y que no se trocea ninguna tarea; se aplican:

- D1 que ordenes traen antes de mostrar: (a) `status`, `roadmap`, `next` y `board` (con solo dos, C3a dejaria de valer). **Elegida.**
- D2 cuando se hace el fetch: (a) en cada lectura. **Elegida.**
- D3 donde se guarda la hora: (a) fichero propio `.git/taskcodetwo/sincronizacion`. **Elegida.**
- D4 `doctor`: (a) sin fetch y con `sincronizado=`. **Elegida.**
- D5 avisos de `tareas/` (F1-2 los encarga a F3-5b): (a) dentro de esta tarea, solo en `status`, aviso con codigo 0. **Elegida.**
- D6 formato de la hora: (a) UTC ISO mas «hace N min», sin dependencias. **Elegida.**
