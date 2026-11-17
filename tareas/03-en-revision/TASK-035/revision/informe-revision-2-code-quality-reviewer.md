# Informe de revisión — TASK-035 (ronda 2)

- Commit revisado: f5b75c0
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| R1-01 | IMPORTANTE | corregido | crates/tc2/src/doctor/capacidades.rs:84-105 (test :457-510) | implementacion |
| H1 | IMPORTANTE | corregido | crates/tc2/src/doctor/capacidades.rs:175-205, :353-366; crates/tc2/src/doctor/plataforma.rs:254-307 | implementacion |
| H2 | IMPORTANTE | corregido | crates/tc2/src/doctor/capacidades.rs:323-355 | implementacion |
| R2-1 | MENOR | aceptado | crates/tc2/src/doctor/capacidades.rs:153 y :184-199 | implementacion |
| R2-2 | MENOR | aceptado | docs/adr/F1-13-adaptador-de-plataforma.md (adenda); crates/tc2/src/plataforma/github.rs:~263 | implementacion |

Sin hallazgos nuevos introducidos por la correccion. R2-1: `lee_config` y el `match` de `config` listan las mismas garantias por separado; hoy coherentes y fijadas por `sin_puertas_lo_que_se_lee_no_esta_comprobado` (aceptado: anadir una garantia exige tocar ambos y el test lo detecta). R2-2: la adenda habla de «cuatro parametros» de `pull_request` y el comentario de «cinco booleanos y el contador» (redaccion; aceptado).

## Verificacion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-035-r2-calidad` (borrado). `--lib` 193 ok, 1 ignorado; `--test doctor` 12/12.

| Hallazgo | Mutacion | Test en rojo |
|---|---|---|
| R1-01 | sustitutos G06, G09, G12; tier GitLab de G06 | `la_tabla_compilada_es_la_de_f1_13` (y `el_bloque_de_capacidades_de_f1_13`) |
| R1-04 | anular `!modelo.contains(&c)` | `check_ajeno_al_modelo_sin_publicar_es_aviso` |
| R1-06 | check-runs en todas las ramas | `ramas_protegidas_sin_fallos_dan_cero` |
| H1 | sin puertas → `Config::Tiene` | `sin_puertas_lo_que_se_lee_no_esta_comprobado` |
| H4 | anular la rama del 403 de G03 | `el_403_de_aprobaciones_con_premium_es_un_permiso` |
| H5 | quitar `nulo_por_defecto` | `nulos_y_403_parciales_no_dan_nada_por_bueno` |

H2, H3 y R1-05 comprobados por lectura de sus tests. Adendas: las garantias que leen config (GitHub G01-G05, G09; GitLab G02, G03, G06) coinciden con `lee_config`; `no-comprobadas=N` solo con N>0; motivos nuevos coinciden con `Motivo::texto`. Sin decision duplicada (ref de estado desde `estado::referencia_de_lectura`; tabla comparada con el ADR por test).

Nota sin severidad: si el plan da rulesets pero `leer-puertas` devuelve `plan_da_reglas=false`, G03 sale `config`; lado conservador.
