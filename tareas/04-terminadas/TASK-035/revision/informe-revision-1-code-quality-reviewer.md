# Informe de revisión — TASK-035 (ronda 1)

- Commit revisado: fe169a0
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Verificacion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-035-r1-calidad` (borrado). `cargo test -p tc2 --test doctor --test tablero`: 11/11 y 5/5.

Criterios: C1, C2, C3, C4 (GitLab sin lectura de permisos de CI, asumido por el test), C5, C6 (aviso < 2.56 solo unitario), C7, C8, C9 (ver R1-05) y C10 demostrados; C11 parcial (R1-01). Tabla G01..G20 de `doctor/capacidades.rs:84-105` comparada a mano con F1-13 (lineas 303-322): coincide en filas, tiers y sustitutos; el bloque de capacidades coincide (19/1/0, 9/1/10, 20/0/0, 11/9/0, 17/3/0). Salida coincide con la adenda de F1-4. Sin duplicacion con `tablero.rs`, `init/` ni `plataforma/` salvo R1-03.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| R1-01 | IMPORTANTE | corregido | crates/tc2/src/doctor/capacidades.rs:84-105 | implementacion |
| R1-02 | MENOR | corregido | crates/tc2/src/doctor/capacidades.rs:171-181, :337-345; docs/adr/F1-13-adaptador-de-plataforma.md:8 | implementacion |
| R1-03 | MENOR | corregido | crates/tc2/src/doctor/estado.rs:23-31 | implementacion |
| R1-04 | MENOR | corregido | crates/tc2/src/doctor/plataforma.rs:128-134 | implementacion |
| R1-05 | MENOR | corregido | crates/tc2/tests/doctor.rs:932-953 | implementacion |
| R1-06 | MENOR | corregido | crates/tc2/src/doctor/plataforma.rs:243-258; plataforma/github.rs:624-667 | implementacion |

### R1-01 (IMPORTANTE): los sustitutos de la tabla compilada no tienen red de regresion

Los tests solo comprueban los conteos y las lineas de G01 y G03. Mutacion: G06 `carril-propietarios`→`zzz`, G09 `autor-del-estado`→`al-dia-por-cli`, G12 `cuenta-de-servicio`→`merge-solo-persona` en GitLab: `--lib capacidades` 3/3 y `capacidades_por_plan` 1/1 en verde. Arreglo: test que parsee el bloque `tabla-equivalencias` del ADR y lo compare con `TABLA` (tier GitHub, tier GitLab y sustituto por fila).

### R1-02 (MENOR): garantias `impuesta` por plan sin leer la configuracion; documentacion imprecisa

Solo G01 (GitHub), G02, G03, G06 (GitLab) y G09 (GitHub) leen configuracion; las demas salen `impuesta` por el plan (p. ej. G04/G05 viajan en la regla `pull_request` ya leida). La adenda dice «G01, G02, G03, G06 en GitLab y G09 en GitHub», lo que sugiere G01 tambien en GitLab.

### R1-03 (MENOR): la eleccion de la ref de estado esta escrita dos veces

`doctor/estado.rs:23-31` repite `estado/lectura.rs:147-150`; si `estado::leer` cambia de ref, `doctor` decidiria sobre otra. Exponer la ref desde `estado/`.

### R1-04 (MENOR): un check requerido ajeno al modelo y sin publicar da `falla`

`doctor/plataforma.rs:128-134` (deducido de la lectura).

### R1-05 (MENOR): C9 sin red solo se aserta para `proteccion` y `permisos`

`check` y `garantia` se comprueban en `no-configurada` y `sin-credenciales`, no con sin-red real.

### R1-06 (MENOR): lecturas repetidas por rama

`leer_puertas` por cada rama solo-PR repite `GET /rulesets` y `check-runs` (en `main` no se usan); en GitLab `projects/:id`, `approval_rules` y estados. Solo coste de llamadas.
