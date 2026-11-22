# Informe de revision — TASK-051 (ronda 1)

- Commit revisado: 02000f5a7c78e8404c5a4257bdb7eb7ecc6bfa1b
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | xtask/src/workflows.rs:2200 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/esquemas/hallazgos.schema.json:134 | implementacion |
| MEN-3 | MENOR | abierto | .github/workflows/revision-ia.yml:686 | implementacion |
| MEN-4 | MENOR | abierto | docs/ci/revision-ia.md:37 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Alcance y metodo: clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-051-r1-calidad`, en 02000f5.
No repeti la suite completa (la peticion declara `verificar --perfil rapida` en verde). Ejecutado por
mi: `cargo test -p xtask` (34 ok) y `cargo test -p tc2 --test hallazgos` (5 ok). Tambien probe el
binario `tc2 hallazgos veredicto` con salidas de `validar` ausentes, malformadas, de version 1, sin
ninguna, duplicadas y con 120 hallazgos (lotes: 50 en `check.json` y 50 en `anotaciones-1.json`;
el 101 se rechaza por `fuera-de-tope`), y las transformaciones `jq` de `publicar` con `jq` local sobre
esos ficheros y sobre `check.json` malformados (`[]`, `"x"`, sin `output`, `conclusion` como lista).
Resultado: el fallback de `publicar` (conclusion que no es `success` ni `failure` o JSON ilegible)
publica `failure` con `incompleta`; un `check.json` ausente, malformado, de version 1 o una
revisora sin fichero dan `failure`/`incompleta` y nunca `success`. La regla del rojo y el rechazo de
`neutral` estan bien. No hay CRITICO ni IMPORTANTE.

### MEN-1 — el lint de `publicar` se salta con la clave entrecomillada

`publicar_transporta_el_veredicto` busca el literal `conclusion:"success"` tras quitar los espacios,
pero en jq tambien es valido `{"conclusion": "success"}` (clave entre comillas), que no coincide.

Reproduccion (en el clon):

1. `sed -i 's/{conclusion: "failure", output/{"conclusion": "success", output/' .github/workflows/revision-ia.yml`
   (el fallback sin `check.json` pasa a publicar `success`).
2. `cargo test -p xtask workflows::el_workflow_del_revisor_pasa_el_lint` da `1 passed`: el lint no lo ve.
3. Contraste: cambiar `success|failure) cp` por `success|failure|neutral) cp` si lo pone en rojo
   (`publicar nombra las conclusiones {"failure","neutral","success"}`).

Es justo la regresion que la regla dice impedir (RR-04: una revision que no ocurrio pasa la puerta). Fix
sugerido: contar apariciones de `success` en `publicar` (solo la del `case`) o normalizar las comillas
antes de comparar. Restaurado el fichero tras la prueba.

### MEN-2 — dos `required` del esquema no los discrimina ningun test

Mutando el esquema y corriendo `cargo test -p tc2 --test hallazgos` y `cargo test -p tc2 --lib`:

- Quitar `raw_details` de `required` en `$defs/anotacion` (linea 134): todo sigue en verde (5 y 373 ok).
- Quitar `aceptados` de `required` en `$defs/informe-tarea` (linea 182): todo sigue en verde.

En cambio quitar `neutral`/anadir una clase falsa si da rojo. `raw_details` es lo que lee `leer-revision`
para reconstruir los aceptados, asi que el contrato de esa clave conviene fijarlo con un caso negativo en
`veredicto_e_informe_validan_contra_el_esquema` (anotacion sin `raw_details`, informe sin `aceptados`).
Esquema restaurado tras las pruebas.

### MEN-3 — `publicar` no tiene salida si la API rechaza la anotacion, y `tc2` no comprueba los limites de GitHub

`anotaciones()` (`crates/tc2/src/hallazgos/mod.rs:1240`) copia sin control el mensaje y el `raw_details`.
Con binario real: un hallazgo cuya `descripcion` son solo caracteres de control o de ancho cero (acepta
`validar`, `aceptados=1`) da `veredicto=rojo` con una anotacion de `message` vacio, que viola el propio
`minLength: 1` del esquema; y una `descripcion` de 70000 caracteres da un `raw_details` de 70210 bytes
(documentado como maximo de 64 KB de la API). Si GitHub rechaza la peticion (422, no verificado en red),
el `curl -f` del POST aborta `publicar` con `set -e`: el check `revision-ia` no existe en el head, en vez
de existir en `failure`. Falla en cerrado (la puerta obligatoria no se cumple), pero el mensaje de la
persona es «sin check», no «rojo». Fix sugerido: que `veredicto` descarte o recorte esos campos
(mensaje no vacio, `raw_details` acotado) y/o que `publicar` caiga al `failure` `incompleta` si el POST
falla. Alcanzable solo por salida del modelo (p. ej. inyectada), por eso MENOR.

### MEN-4 — la guia describe un orden del resumen que el workflow no produce

`docs/ci/revision-ia.md:36-40` dice que tras el veredicto vienen `preparar=success consultar=<r>` y
despues `revisoras= agentes= ...`. En `validar` (`.github/workflows/revision-ia.yml:543-544`) se copia
primero la linea `revisoras=...` de `preparar` y despues se anade `preparar=success consultar=...`, asi que
el orden real es el inverso. Ademas el workflow fija `preparar=success` a mano (correcto porque `validar`
solo corre con `preparar` en exito), pero la guia no lo dice. Solo documentacion.

## Lecturas fuera del contexto permitido

- `xtask/src/workflows.rs` (lint de `publicar`, lineas 2120-2215 y sus mutaciones): para MEN-1 y para
  comprobar que el lint discrimina; la peticion pide razonar sobre el workflow y su lint.
- `crates/tc2/src/hallazgos/mod.rs` (`anotaciones`, `Dictamen`, `InformeTarea`) y `crates/tc2/src/plataforma/github.rs`
  (`leer_revision`): para entender el contrato que consume el `check.json` y reproducir MEN-3 con el binario real.
- `crates/tc2/tests/hallazgos.rs` y `crates/tc2/tests/fixtures/hallazgos/respuesta-prosa.md` y `diff.txt`: para
  construir las entradas de las pruebas.

Veredicto propuesto: aprobada-con-correcciones
