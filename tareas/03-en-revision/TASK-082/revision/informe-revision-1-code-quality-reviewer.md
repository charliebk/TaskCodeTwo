# Informe de revisión — TASK-082 (ronda 1)

- Commit revisado: c574e8d
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| R1-01 | MENOR | abierto | crates/tc2/tests/liberar.rs:475 | implementacion |
| R1-02 | MENOR | abierto | crates/tc2/src/solape.rs:187 | implementacion |
| R1-03 | MENOR | abierto | crates/tc2/tests/solape.rs:515 | implementacion |
| R1-04 | MENOR | abierto | crates/tc2/tests/solape.rs:535 | implementacion |

**R1-01.** El cambio en `tests/liberar.rs:475-481` debilita el test mas de lo necesario: en `develop`, `tc2()` exigia una sola linea en stdout, codigo 0 y stderr vacio; ahora solo codigo y `err.lines().all(|l| l.starts_with("nota: "))`, que tambien es verdadero con stderr vacio (en `TASK-002` no exige que aparezca la `nota:`). Anadir `assert_eq!(out.lines().count(), 1)` y exigir exactamente una `nota:` en `TASK-002`.

**R1-02.** `calcular` cruza tambien las tareas de la misma persona: avisa a alguien contra si mismo («hablad antes de seguir»). Anotado en la autorrevision pero no en la adenda de F1-17. Documentarlo o filtrar por identidad.

**R1-03.** C10 llama a `solape::calcular` con una lista hecha a mano, no a traves de `start`: no prueba los 1 + k + 1 procesos del flujo real ni el enlace `op.en_curso()` → `calcular`. Medir el registro sobre el `start` completo.

**R1-04.** C11 solo con un unitario (`Solape::avisos`, `con_claves`): ningun test del binario ejerce el cableado de `con_aviso_de_solape` (`lib.rs:247`) con una ruta o login con caracteres de control. Anadir un caso de integracion.

## Verificacion

Clon (borrado). `--test solape` 10/10, `--test liberar` 12/12, `--test toma` 13/13, unitarios de solape verdes. Mutacion: quitar `.filter(|m| !ya.contains(*m))` en `cruzar` → 2 unitarios en rojo. C1-C9 demostrados (C5 en la ruta `SinCambios`); C10 y C11 con los matices de R1-03 y R1-04. Claves y textos coinciden con la adenda de F1-17; `:` en valores de lista y escape de coma coherentes con F1-4; divergencia sin F4-1 (decision de Carlos, 2026-11-18) bien documentada (S6, S7 y S9 de F1-17 quedan fuera por la misma divergencia, sin nombrarlas una a una). `hotfix`: el `...` toma el merge-base, sin fallo. Sin duplicacion real con `situacion.rs` ni `tablero.rs`.
