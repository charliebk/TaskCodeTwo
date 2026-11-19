# Informe de revision — TASK-118 (ronda 1)

- Commit revisado: 34a6b7b21259c24709fa626eb1939edb7a053ecc
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/plataforma/gitlab.rs:1188 | implementacion |
| MEN-1 | MENOR | corregido | docs/adr/F1-5-puertas-en-github.md:629 | implementacion |
| MEN-2 | MENOR | corregido | docs/adr/F1-13-adaptador-de-plataforma.md:378 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Alcance y puerta: pase generico completo sobre el diff de 17 ficheros. Suite: `cargo xtask verificar --perfil rapida` en verde (exit=0) sobre a7f212e, codigo identico al revisado (log `verif-118.log`, termina en «verificacion Rapida en verde»); citada, no repetida. En un clon de 34a6b7b, con `--test-threads=4`: `--test init aplicar_puertas` 4 passed; `--test plataforma aplicar_puertas` 1 passed; `--test plataforma_gitlab aplicar_puertas` 1 passed; `--lib puertas` 5 passed.

Mutacion (clon, cada mutante contra los tests concretos; todos detectados):
- GitHub, quitar la relectura del 422 (`r.estado == 422` imposible): `plataforma::aplicar_puertas_carrera_corte_y_plan` rojo.
- GitLab, quitar el 409 como «ya existia»: `plataforma_gitlab::aplicar_puertas_gitlab_premium_y_carrera` rojo.
- `no_vistos` ignorando el origen (cualquier App vale): `init::aplicar_puertas_sin_origen_se_rechaza_sin_escribir` rojo.
- GitLab `allow_force_push: true`: `init::aplicar_puertas_en_gitlab_protege_con_niveles_explicitos` rojo.
- Comprobacion de «ramas por delante» desactivada: `init::aplicar_puertas_exige_personas_ramas_y_persona` rojo.

Criterios de aceptacion: 1) cubierto por `aplicar_puertas_en_github_crea_las_que_faltan` y `aplicar_puertas_en_gitlab_protege_con_niveles_explicitos` (doble HTTP; POST de las cinco `taskcodetwo-*` en GitHub con `integration_id`; protecciones en GitLab sin DELETE ni PATCH); 2) cubierto por `aplicar_puertas_sin_origen_se_rechaza_sin_escribir` (cuatro casos, cero escrituras en plataforma y disco, el mensaje dice la orden a repetir) y `aplicar_puertas_exige_personas_ramas_y_persona`. Sin la opcion, `init` conserva su camino (`puertas(&plataforma, ..)` y `ruleset_github(.., None)` con `_nota`); los tests previos de init estan en la suite en verde. El cliente HTTP (`Metodo`) solo tiene Get, Post y Put: nunca DELETE ni PATCH, como dice la adenda. No encontre duplicacion real ni helpers reimplementados.

### IMP-1 — GitLab: tras un corte despues de proteger `develop`/`main`, repetir no converge y los ajustes de merge y la regla de aprobacion no se ponen nunca
- Donde: crates/tc2/src/plataforma/gitlab.rs:1188 (estado `Difiere` de una puerta `solo_pr` cuando `!ajustes_bien || (premium && !con_aprobacion)`), en combinacion con crates/tc2/src/plataforma/gitlab.rs:1049 (los ajustes y la regla solo se escriben dentro de la creacion de una puerta en estado `Falta`).
- Que pasa: el `PUT /projects/:id` (`merge_method=merge`, `only_allow_merge_if_pipeline_succeeds=true`) y el `POST /approval_rules` se hacen solo al crear `develop` o `main`, despues de proteger la rama. Si la ronda se corta entre medias (500, 429, red, 403 de la regla), la rama ya queda protegida; en la repeticion `develop` pasa a `Difiere` (ajustes o regla ausentes) y lo `Difiere` no se toca, asi que nada vuelve a escribir los ajustes. La repeticion termina `sin-cambios ... difieren=taskcodetwo-develop,taskcodetwo-main` y los ajustes siguen mal. La adenda de F1-5 y el doc del trait dicen «repetirla converge». Ademas la linea de la ronda cortada dice `aplicadas=` vacio aunque la proteccion de `develop` si se creo.
- Reproduccion: en el clon, test temporal sobre `gitlab_premium` (borrado con el clon). Ronda 1: nada protegido, proyecto con `merge_method=ff` y pipeline no exigido, `POST /protected_branches` 201 y `PUT /projects/o%2Fr` 500. Salida: `aplicar-puertas error motivo=http estado=500 aplicadas= faltan=taskcodetwo-develop,taskcodetwo-main,taskcodetwo-ramas-de-tarea,taskcodetwo-tareas,taskcodetwo-tags-version`. Ronda 2: `develop` y `main` ya protegidas con push 0 y merge 40, ajustes aun mal, `PUT` listo para devolver 200. Salida: `aplicar-puertas sin-cambios aplicadas= existentes=taskcodetwo-ramas-de-tarea,taskcodetwo-tareas,taskcodetwo-tags-version difieren=taskcodetwo-develop,taskcodetwo-main`, y escrituras de la ronda: `[]` (ningun `PUT`).
- Impacto: en GitLab el resultado visible de la repeticion es «sin cambios» cuando falta justo la puerta que impone el pipeline en el merge (G09), y ningun mensaje dice que hacer (`difiere` no indica que son los ajustes del proyecto ni como arreglarlos). El caso es el que el plan declara como riesgo aceptado («estado parcial entre puertas, GitLab ~10 llamadas») y que dice contener con repeticion idempotente.
- Sugerencia: en `leer_para_aplicar`, separar el estado de los ajustes y de la regla del de la proteccion de rama: si la rama esta protegida como el modelo pero faltan ajustes o regla, la puerta es `Falta` (o hay un paso propio) y se completan; `Difiere` queda para una proteccion ajena. Un test GitLab de corte en el `PUT` y repeticion con las ramas ya protegidas. Si se prefiere dejarlo, la ADR no puede decir que repetirla converge para GitLab, y `difiere` debe decir que los ajustes del proyecto no se tocan.

### MEN-1 — La ADR dice que todo 422 de GitHub cuenta como «otro llego antes»
- Donde: docs/adr/F1-5-puertas-en-github.md:629
- Que pasa: la adenda afirma que un 422 de GitHub o un 409 de GitLab al crear cuenta como existente. En GitHub el codigo solo lo trata asi si, ademas, una relectura de `GET /rulesets` contiene el nombre (github.rs, `r.estado == 422 && rulesets_de(..).is_ok_and(..)`); un 422 por cuerpo invalido (p. ej. un `integration_id` que no es una App instalada) se traduce a error. El codigo es correcto; la frase de la ADR es imprecisa.
- Reproduccion: lectura de `aplicar_puertas` de github.rs y del test `aplicar_puertas_carrera_corte_y_plan` (el 422 con relectura que contiene el nombre da `existentes`). No ejecute un 422 sin nombre repetido (no verificado, solo lectura).
- Impacto: quien lea la ADR esperara que un 422 se ignore, y no que corte con `aplicadas=/faltan=`.
- Sugerencia: «un 422 de GitHub cuyo nombre aparece al releer los rulesets, o un 409 de GitLab».

### MEN-2 — La fila de `aplicar-puertas` de F1-13 omite lecturas que el adaptador hace
- Donde: docs/adr/F1-13-adaptador-de-plataforma.md:378
- Que pasa: la columna de GitLab lista `GET /protected_branches`, `GET /protected_tags`, `GET /repository/branches/develop` y estados, `GET /projects/:id` y namespace, pero el adaptador tambien hace `GET /approval_rules` en Premium (gitlab.rs, `leer_para_aplicar`) y `GET /user` de la identidad de persona (ambos adaptadores). La fila existe para decir lo que se llama y esta incompleta.
- Reproduccion: comparacion de la fila con `leer_para_aplicar` y con `preparar_puertas` de github.rs (lectura).
- Impacto: documentacion de permisos incompleta para quien configure el token.
- Sugerencia: anadir `GET /approval_rules` (Premium) y `GET /user`.

## Lecturas fuera del contexto permitido

- crates/tc2/src/plataforma/gitlab.rs, github.rs, mod.rs, puertas.rs; crates/tc2/src/init/inventario.rs, mod.rs, aplicar.rs; crates/tc2/src/i18n/catalogo/init.rs y plataforma.rs: el diff de la peticion solo embebia 3 ficheros (esquema y dos ADR); revisar correccion exige leer el codigo (via diff en el clon).
- crates/tc2/tests/init.rs, plataforma.rs, plataforma_gitlab.rs: comprobar que los tests discriminan y para la mutacion.
- crates/tc2/src/plataforma/http.rs (enum `Metodo`): comprobar que el cliente no tiene DELETE ni PATCH.
- verif-118.log (en pruebas-claude, fuera del repo): resultado de la suite de la ronda.
