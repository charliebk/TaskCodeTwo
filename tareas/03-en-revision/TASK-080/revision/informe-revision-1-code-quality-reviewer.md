# Informe de revisión — TASK-080 (ronda 1)

- Commit revisado: 642ec1c
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | docs/distribucion/instalar.md:106 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/inventario.rs:319 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/tests/recursos.rs:625 | implementacion |
| MEN-4 | MENOR | abierto | docs/adr/F1-14-distribucion-e-instalacion.md:9 | implementacion |
| MEN-5 | MENOR | abierto | crates/tc2/src/doctor/recursos.rs:102 | implementacion |
| MEN-6 | MENOR | abierto | crates/tc2/src/recursos.rs:308 | implementacion |

**MEN-1.** La linea de texto escribe los valores con `%20` (`arreglo=claude%20plugin%20install%20taskcodetwo`) y la guia los muestra con espacios; el ejemplo omite `version=`. Anadir una nota o el ejemplo codificado.

**MEN-2.** `name: foo # comentario` da el id `foo # comentario`. Cortar en ` #` salvo valor entre comillas.

**MEN-3.** El test de integracion de C9 no aserta que `rtk` salga `aviso` conveniente: depende del PATH real. Hacerlo determinista con `PATH` controlado.

**MEN-4.** La adenda de F1-14 no lista como pendientes dos fuentes de «necesario» del punto 8: el comando de `verificacion` con sus ejecutables y las revisoras del proyecto o el runner.

**MEN-5.** `TipoRecurso::de_texto` repite el conjunto `skill|agente|ejecutable` de `config/claves.rs:456`: un tipo nuevo valido para la config se descartaria y saldria `ok`. Fuente unica o `no-comprobado` con tipo no reconocido.

**MEN-6.** Los avisos convenientes de engram y context7 salen `aviso no-instalado` aunque el inventario este incompleto; con `completo=false` deberian ser `no-comprobado`.

Observaciones sin gravedad: `leer_fichero` (`inventario.rs:267`) repite el patron de `desinstalar::leer`; el JSON mezcla `/` y `\` en Windows.

## Verificacion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-080-r1-calidad` (borrado). `cargo test --workspace` en verde (exit 0; vista la cola: recursos 12/12, frescura 12/12, init 21/21, xtask 10/10). C1-C11 demostrados por sus tests (C9 con el matiz de MEN-3). Mutaciones en rojo: `completo: true` (`plugins_ilegibles_no_comprobado`), `aplica` con cualquier `scope` (`lee_lo_instalado_y_habilitado`), `syncClaudeAiPlugins` desactivado (`sincronizados_desde_claude_ai`).

Extraccion desde `desinstalar` sin cambio de comportamiento. `tests/doctor.rs` y `tests/frescura.rs` sin aserciones debilitadas. Sin duplicacion con consecuencia salvo MEN-5. Manifiesto de F1-14 punto 8 cumplido en lo que dice el plan (firma, cuatro niveles, `start` solo obligatorio y necesario, engram conveniente, suelo en el binario). Binario real con `CLAUDE_CONFIG_DIR` falso: codigo 3, primera linea correcta, `y 3 mas` y agregado del `doctor` normal.
