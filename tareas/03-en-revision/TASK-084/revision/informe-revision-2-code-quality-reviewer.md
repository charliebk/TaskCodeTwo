# Informe de revisión — TASK-084 (ronda 2)

- Commit revisado: f44ce49
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| M1 | MENOR | corregido | crates/tc2/src/progreso/linea.rs:67 | implementacion |
| M2 | MENOR | corregido | crates/tc2/src/progreso/linea.rs:107 | implementacion |
| M3 | MENOR | corregido | docs/distribucion/instalar.md:225 | implementacion |
| M4 | MENOR | corregido | crates/tc2/src/progreso/mod.rs:326 | implementacion |
| R1-01 | IMPORTANTE | corregido | crates/tc2/src/progreso/mod.rs:416 | implementacion |
| R1-02 | MENOR | corregido | crates/tc2/src/tablero.rs:282 | implementacion |
| R1-03 | MENOR | corregido | crates/tc2/src/main.rs:127 | implementacion |
| R1-04 | MENOR | corregido | crates/tc2/src/progreso/fichero.rs:177 | implementacion |
| R1-05 | MENOR | corregido | crates/tc2/src/progreso/linea.rs:72 | implementacion |
| N1 | MENOR | abierto | docs/distribucion/instalar.md:231 | implementacion |

**N1.** `instalar.md:231-233` dice que Claude Code lee el `settings.local.json` de la raiz del repo (en un worktree, la del principal); eso esta en la subseccion `#where-claude-code-keeps-the-local-file-in-a-git-repository`, no en el ancla citada, y omite que vale desde Claude Code v2.1.211 y sus excepciones (fuera de repo, raiz igual al directorio personal, dueno distinto). Citar esa ancla y anadir «desde v2.1.211».

## Ejecucion

Clon (borrado). `--lib` 285 ok, 2 ignorados; `--test progreso` 11/11; `fmt --check` y `clippy` limpios.

| ID | Mutacion | Test en rojo |
|---|---|---|
| M1 | `floor` → `round` | `m1_a_mitad_de_tramo_vale_el_punto_anterior` |
| M2 | `> UMBRAL_MIN * 2.0` | `m2_frontera_de_sin_actividad` |
| R1-05 | sin aviso; `>=` en vez de `>` | `r1_05_pasado_el_tope_avisa_saturada` |
| R1-02 | sin U+00AD, U+061C, U+180E | `sanear_quita_controles_y_formato_y_acota` |
| R1-04 | `?` del anterior | `sin_anterior_escribe_el_progreso` |
| M4 | ID por nombre de carpeta; sin `REASIGNAR` | `m4_actividad_con_carpeta_administrativa_de_otro_nombre`; `m4_reasignar_borra_el_progreso` |
| R1-01 | `repetir` ignorado | `seguir` y `r1_01_status_seguir_termina_al_cerrarse_stdout...` |
| R1-03 | `es_status_line` desactivado | `r1_03_estado_stdin_con_stdout_cerrado_sale_con_0_y_calla` |

Salida real: `tc2 status --seguir | head -1` termina en 4,2 s con 0; a fichero escribe en 0, 4 y 8 s; en terminal no repite; `estado --stdin` con stdout cerrado sale 0; aviso de saturada igual en codigo, adenda e `instalar.md`; https://code.claude.com/docs/en/settings#settings-files-and-who-they-affect existe (consultada el 2026-11-18) y confirma el ambito «Project local» y la precedencia.

Sin fallos nuevos: `es_status_line` solo ignora el error de escritura; un `.anterior.json` mas viejo tras un fallo es solo el respaldo; `gitdirs_de_tareas` es O(worktrees).
