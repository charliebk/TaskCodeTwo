# Informe de revision — TASK-085 (ronda 1)

- Commit revisado: 0689c2cec9429cd0695cd68458dcdad880695989
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | corregido | crates/tc2/tests/seleccion.rs:118 | implementacion |
| MEN-2 | MENOR | aceptado | crates/tc2/tests/fixtures/seleccion-taskcodetwo/estratos.txt:1 | implementacion |
| MEN-3 | MENOR | corregido | crates/tc2/src/plan/operacion.rs:169 | implementacion |
| MEN-4 | MENOR | corregido | crates/tc2/tests/seleccion.rs:1 | plan |

Sin CRITICO ni IMPORTANTE. Alcance: completa; el diff es de un solo dominio (Rust con docs), no llego por exceso de dominios.

## Puerta y reproduccion

- Puerta: `cargo xtask verificar --perfil rapida` en verde en 7345f6a segun la peticion; no se repitio la suite. El commit revisado (0689c2c) solo anade la tabla de evidencias respecto a 7345f6a.
- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-085-r1-calidad` en 0689c2c, con su propio `target`.
- `cargo test -p tc2 --test seleccion`: 2 passed, 0 failed (`diez_tareas_de_taskcodetwo` y el mock comun).
- Tabla de 10 tareas: se ejecuto `TC2_TABLA_SELECCION=1 cargo test -p tc2 --test seleccion diez -- --nocapture`, que lanza el binario real `tc2 seleccion --json --tarea-fichero --rutas` dos veces por tarea. Las 10 filas generadas son byte a byte iguales a las de `evidencias/tabla-10-tareas-taskcodetwo.md` (diff vacio). La tabla es salida real y reproducible; no es redaccion a mano.
- `tc2 seleccion --rutas` con rutas `a/b` y con `a\b`: misma salida (la normalizacion funciona en la orden nueva).
- Mutacion 1: `casa_palabra` pasado de principio-de-palabra a subcadena rompe `seleccion::dominios::tests::tabla_adversaria` (rojo, como debe).
- Mutacion 2: la fila de modelo devuelve `sonnet` donde la tabla dice `opus`. Rojo en 4 unitarios (`la_muestra_del_adr_se_reproduce`, `modelo_por_rol_carril_y_complejidad`, `modelo_por_carril_complejidad_y_permitidos`, `la_generica_por_descarte_y_por_exceso`) pero VERDE en `tests/seleccion.rs` (MEN-1).
- Mutaciones restauradas; no se toco el repo original.

### MEN-1 — `diez_tareas_de_taskcodetwo` apenas discrimina
- Donde: crates/tc2/tests/seleccion.rs:118
- Que pasa: solo asevera determinismo, que el modelo esta en el vocabulario y tres estratos (`fix`, `toca-github`, `solo-adr`); el resto cae en `_ => {}`. Nada compara la tabla impresa con `evidencias/tabla-10-tareas-taskcodetwo.md` ni con un golden.
- Reproduccion: mutacion 2 (todo `opus` pasa a `sonnet`): `cargo test -p tc2 --test seleccion` sigue en `2 passed`. La red de regresion del modelo la dan los unitarios de la muestra del ADR, no este test.
- Impacto: la tabla que Carlos va a marcar puede dejar de ser la que produce el binario sin que ningun test se entere. El docstring del test es honesto (dice que solo comprueba contrato y determinismo), por eso es MENOR y no un disfraz. Sugerencia: golden de la tabla en la fixture, o no presentarlo como medida del criterio 1.

### MEN-2 — La «regla de muestra» no es mecanica ni esta escrita completa
- Donde: crates/tc2/tests/fixtures/seleccion-taskcodetwo/estratos.txt:1
- Que pasa: la evidencia dice «ID mas alto por estrato». No hay guion que la aplique; `estratos.txt` se fijo a mano, con definiciones de estrato subjetivas (`seguimiento`, `puertas-sin-ruta`, `sustituto-H5`) y un orden de asignacion sin escribir. Segun la memoria del proyecto (cadenas H2 y H6), TASK-078 cierra H2 y TASK-053 esta en H6, y ambas aparecen en otros estratos mientras H2 y H6 se representan con 077 y 052. Eso solo encaja si cada tarea se usa una vez y los estratos se asignan en cierto orden, y eso no consta. Dato no verificado en `tarea.md` (no se contrasto el sprint de 078 y 053).
- Reproduccion: lectura de `estratos.txt` frente a `tareas/04-terminadas`; la fila `fix` = TASK-137 si es el ID mas alto de tipo fix. La regla completa no se pudo reproducir porque no es ejecutable.
- Impacto: Carlos no puede comprobar que la muestra no esta elegida a conveniencia (D9a pedia estratos fijados antes de mirar resultados). Sugerencia: anotar en la evidencia el orden de asignacion y la exclusion de tareas ya usadas.

### MEN-3 — Doble construccion de las senales de dominio y error de tabla tragado
- Donde: crates/tc2/src/plan/operacion.rs:169
- Que pasa: `Planificar::seleccion` arma `Texto` y llama a `dominios::dominios` por su cuenta, y `seleccion::orden::dominios_de` (orden.rs:189) hace lo mismo para la revision, con la `union`. Ademas `unwrap_or_default()` sobre la tabla y sobre `seleccionar` convierte un fallo en lista vacia, y `roles_de` (operacion.rs:333) vuelve a la heuristica de antes si `reglas` esta vacio.
- Reproduccion: lectura; la tabla esta compilada y `las_tablas_del_adr_se_leen` cubre el caso, asi que hoy no se dispara.
- Incoherencia concreta: si cambia como se arma el texto vivo (por ejemplo, anadir el titulo), hay que tocar los dos sitios; si se toca solo uno, `plan` congela unas senales y `review` calcula otras. El camino de respaldo a `roles::seleccionar` es un segundo modelo del mismo hecho, justo lo que la alternativa 1 del plan descarto. Sin consecuencia demostrable hoy.

### MEN-4 — El criterio 1 queda sin discriminar en implementacion
- Donde: crates/tc2/tests/seleccion.rs:1
- Que pasa: en las 10 filas la columna «Implementacion» es «—» (sin skills): sin metadatos en el catalogo S07 no elige nada, como el plan acepto. La tabla para Carlos no permite juzgar la propuesta de skills de implementacion ni de diseno (S06) con tareas reales de este repo; solo diseno (agentes) y revision.
- Reproduccion: columna `Implementación` de la tabla reproducida: 10 de 10 con «—».
- Impacto: el criterio 1 («coincide con la de una persona») se mide de hecho para agentes de diseno y revision, no para skills. Esta dicho en el plan («riesgos aceptados»), pero la adenda del ADR y la tabla no lo avisan al lector. Sugerencia: una linea en la evidencia que lo diga.

## Regla 4D

- Duplicacion: MEN-3 (menor, sin consecuencia hoy)
- Desviacion: cumple (el criterio 1 queda pendiente de Carlos, la adenda lista lo no verificado y las divergencias; MEN-2 y MEN-4 son notas de transparencia, no incumplimientos)
- Diluvio: cumple (los tres roles usan una sola funcion `seleccionar_con`; sin selectores paralelos)
- Disfraz: MEN-1 (la tabla es salida real y reproducible; el test que la imprime es debil pero honesto sobre su alcance)

## Lecturas fuera del contexto permitido

- crates/tc2/src/seleccion/mod.rs, dominios.rs, orden.rs, plan/operacion.rs y tarea/texto.rs: para construir las mutaciones y comprobar la duplicacion.
- tareas/04-terminadas/*/tarea.md (solo cabeceras): para contrastar los estratos de `estratos.txt`.

Veredicto propuesto: aprobada-con-correcciones
