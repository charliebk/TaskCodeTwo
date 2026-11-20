# Informe de revision — TASK-049 (ronda 2)

- Commit revisado: fc65d9e024c04921628c2087ca6622c7efaaade0
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | corregido | .github/workflows/revision-ia.yml:362-364 | implementacion |
| MEN-2 | MENOR | corregido | xtask/src/main.rs:29-32, xtask/src/workflows.rs:482-498, docs/adr/F1-8-seguridad-modelo-de-amenazas.md | plan |
| MEN-3 | MENOR | corregido | .github/workflows/revision-ia.yml:362-364 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### Hallazgos de la ronda 1: verificados

- **IMPORTANTE 1 (rutas no ASCII): corregido.** `core.quotepath=false`, `-z`, `tc2 revisoras --nul` y `mapfile -d ''` estan en su sitio. Reproducido con rutas `é ñ.rs` y `a b.rs`: llegan intactas. El test `ruta_no_ascii_separada_por_nul` ejercita el binario real.
- **IMPORTANTE 2 (fetch sin credencial): corregido en el codigo.** `GIT_CONFIG_COUNT/KEY_0/VALUE_0` se exportan solo en el paso, con el valor enmascarado. No queda en argv ni en `.git/config`. La autenticacion contra GitHub no se ha reproducido aqui.
- **MENORES de la ronda 1: corregidos.** La cerca del diff es mas larga que la racha maxima de acentos graves. `publicar` valida la instalacion y el token. Con `pipefail` todos los caminos repasados siguen bien.

### Excepcion a R9: cuan acotada es

- Mutacion A: `&& !hereda_admitido(...)` cambiado por `&& false` pone 2 tests en rojo.
- Mutacion B: sin `es_sha(version)` pone el test nuevo en rojo.
- El perfil `LLAMADOR_REVISION` admite solo `workflow_run`, sin escritura y sin entornos.
- La comparacion del destino es exacta.

### MEN-1 — el comentario decia «sin la clave» y el codigo no la quitaba

La tuberia solo recortaba y quitaba los saltos de linea: un mensaje que citara la clave iria tal cual al `summary` del check, donde no actua el enmascarado de GitHub. Hoy la API no devuelve la clave en sus errores, pero no se ha comprobado.

Corregido en 9c14787: jq sustituye `env.ANTHROPIC_API_KEY` por `***`, leyendo la clave del entorno y no de argv. Probado con un mensaje que contiene la clave: sale `Your key *** bad`.

### MEN-2 — la excepcion la vigila un lint que solo corre en tests

`mod workflows` esta bajo `#[cfg(test)]`, y el llamador real de un proyecto no pasa por ningun lint. Ademas, `inherit` entrega todos los secretos del repositorio y de la organizacion, y el ADR no lo decia.

Corregido en 9c14787: la adenda F6-3 de F1-8 y la guia dicen los dos limites. En un proyecto consumidor el llamador lo restringen la revision humana y el SHA fijado, hasta que `doctor` lo compruebe (F3-8).

### MEN-3 — `": "` sin `.error` y recorte por bytes

Un JSON sin `.error` daba `error-http-500 : ` en lugar de `sin-cuerpo`, y `cut -c` puede partir un caracter multibyte.

Corregido en 9c14787: jq une solo las cadenas no vacias y recorta con `.[0:200]`, por caracteres. Probado: JSON sin error, no JSON y vacio dan `sin-cuerpo`; 250 `é` dan 200 caracteres y 400 bytes validos.

## Alcance y reproduccion

- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-049-r2-b`, con `CARGO_TARGET_DIR` propio, borrado al terminar.
- `cargo test -p xtask`: 29 ok. `cargo test -p tc2 --test revisoras`: 5 ok.
- No se ha lanzado nada en GitHub. Lo que depende de GitHub se apoya en el E2E real que cita el ADR.

## Lecturas fuera del contexto permitido

- `xtask/src/main.rs`, para ver que el lint es `#[cfg(test)]`.
- `crates/tc2/src/seleccion/orden.rs`, `crates/tc2/src/lib.rs` y `crates/tc2/tests/revisoras.rs`, para verificar las correcciones de la ronda 1.
