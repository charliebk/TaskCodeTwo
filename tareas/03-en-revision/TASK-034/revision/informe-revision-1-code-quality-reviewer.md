# Informe de revisión — TASK-034 (ronda 1)

- Commit revisado: db0bc53
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Verificacion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-034-r1-calidad` (borrado). `cargo test -p tc2 --test ciclo --test toma`: 12/12 y 13/13. Salida real sondeada: `pausa ok id=TASK-001 commit=… intentos=1 tareas=sin-montar`; `descartar ok … worktree=quitado|ninguno|conservado worktree-motivo=cambios-sin-guardar|en-uso`; `start rechazada … motivo=pausada arreglo=tc2%20reanudar`.

Mutaciones: quitar el `fetch` previo de `paso_descartar` sobrevive (hallazgo 1); quitar `.rev()` en `Cuerpo::en_pausa` se pone rojo. Ida y vuelta de motivos raros (comillas, `\`, `: `, `#`, `null`, emoji…) correcta.

Criterios: C1, C2, C3, C4, C6, C7, C9, C10 y C11 demostrados; C5 y C8 en parte (hallazgos 1 y 2). Contrato de `descartar` coincide con la adenda de F1-4; diferencia en el hallazgo 3.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| 1 | IMPORTANTE | abierto | crates/tc2/src/ciclo/mod.rs (`hay_remoto` en `paso_descartar`) | implementacion |
| 2 | MENOR | abierto | crates/tc2/tests/ciclo.rs (`c5_...`) | implementacion |
| 3 | MENOR | abierto | docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md (adenda TASK-034) | implementacion |
| 4 | MENOR | abierto | crates/tc2/src/ciclo/operacion.rs frente a toma/operacion.rs y grafo.rs:31 | implementacion |

### 1. IMPORTANTE: el `fetch` previo de `descartar` no tiene test que se ponga rojo al quitarlo

Cambiado a `if false && ...`, `c3_`, `c5_` y `c8_` siguen en verde: c8 porque `exige_red` ya da `sinred` en la transaccion; c5 porque el test hace `git push` desde el worktree y eso actualiza `refs/remotes/origin/<rama>`. Sin el `fetch`, una rama publicada desde otro clon daria `worktree=conservado worktree-motivo=sin-publicar` en vez de `quitado` (fallo en la direccion segura, pero es lo que promete el ADR). Test que falta: publicar la rama desde un segundo clon y descartar desde el primero esperando `worktree=quitado`.

### 2. MENOR: `worktree-motivo=sin-publicar` y `sin-rama` sin test de CLI

El caso comun (descartar justo tras `start`, rama sin publicar) da `conservado sin-publicar` y no se ejercita de punta a punta.

### 3. MENOR: la adenda de F1-4 fija `pausa ok id=… commit=…` y `reanudar ok …`, pero la salida anade `intentos=1 tareas=sin-montar`

Corregir la adenda; ningun test aserta la linea completa.

### 4. MENOR: decision duplicada

El bloque de `asignado_a` ajeno/heredado y el `match` de `terminada`/`descartada`/`estado-no-valido` estan en `Lanzada` y en `Tomar::aplicar`; textos `TERMINADA`, `DESCARTADA`, `ASIGNADO_HEREDADO` en dos catalogos; `ciclo::DESCARTADA` es la tercera copia de `grafo::DESCARTADA`; `DE_ORIGEN` significa cosas distintas en `toma` (2 estados) y `ciclo` (4).

## Descartado tras revisar

Escape del motivo en el ROADMAP (`celda()`), tope de 500 caracteres, cambio legitimo de `referencia.md`, `siguiente_id` cuenta descartadas, `Pausar::aplicar` correcta.
