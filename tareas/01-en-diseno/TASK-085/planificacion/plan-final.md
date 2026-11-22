# Plan — TASK-085: F4-6 plan propone skills, agentes y modelo por rol

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

F4-6 completa la decision 1 de F1-15 sobre lo que F6-3 dejo construido: `seleccion::revision` ya interpreta `tabla-seleccion` y `tabla-modelo` del ADR (filas de `revision` y `todos`), con el catalogo de la base y el inventario de F3-9. Falta lo que la adenda F6-3 dejo apagado: el detector `dominios()` (S10, S11 y la excepcion de S13 nunca se cumplen), las filas de `diseno` (S01-S06) e `implementacion` (S07), las columnas «Diseño» e «Implementación» de `tabla-modelo` (el lector las ignora: `_diseno, _impl` en `tabla::modelo`), y que `plan` escriba y use el resultado. No se crea un selector nuevo: se generaliza el que hay. Precedente: TASK-049/051/052 (funcion pura que lee las tablas del ADR con `include_str!`, orden fina que transporta).

Estado actual (leido en el codigo):
- `plan` (`crates/tc2/src/plan/operacion.rs`, `Planificar::aplicar`) calcula `n = carril.roles_diseno(resolucion.roles, ...)` y `roles::seleccionar(n, &t.etiquetas)`: sin suelo por dominio (D1a), sin modelo, sin anotar nada en `roles`. Es una `Operacion` de la rama de estado: no lee el disco, asi que inventario y catalogo tienen que entrarle como dato (como hoy entra `config`).
- `seleccion/mod.rs`: `Estado::atomo` devuelve `false` para `dominio-*`, `hay-skill-*`, `roles>=`; `aplicar` no conoce `roles=`, `suelo=`, `mas-rol=`, `sin-plan`, `skills-diseno-del-dominio`, `skill-ejecucion=...`, `revisora-del-dominio`, `aviso=dominio-sin-revisora` (caen en el brazo por defecto).
- `seleccion/catalogo.rs`: solo lee revisoras (`rol: revisor`); `config.recursos[].rol` solo admite `revisor` (`config/claves.rs`). En el plugin 0.16.2 instalado solo las 5 revisoras llevan bloque yaml: `task-workflow` y los `brainstorm-*` no declaran metadatos.
- `tarea/modelo.rs`: `Rol {modelo, skills, regla}` y `RolRevision {agente, modelo, skills, regla}` con `deny_unknown_fields` y `regla` del vocabulario 0.16 (`solape|prioridad|llm`); la divergencia 1 de F1-15 (mapa ampliado) sigue pendiente (adenda F6-3, divergencia 2).
- `config.seleccion.dominios_extra` (`Senales {etiquetas, palabras, rutas}`) ya existe y se valida; nadie lo consume.
- `doctor --recursos` ya recoge `roles.*.skills` y `roles.revision.agente` de las tareas en curso como «necesario» (`doctor/recursos.rs`).
- `review` y `tc2 revisoras --tarea` (CI) ya llaman a `seleccion::revision` con carril y complejidad de `orden::de_tarea`.

Que se hace:
1. **`seleccion/dominios.rs` (nuevo, puro)**: `dominios(entrada: &Senalable, cambios: &[String], tabla: &[FilaDominio], extra: &BTreeMap<String, Senales>) -> Vec<(String, Vec<String>)>` con la `tabla-dominios` del ADR (lector nuevo en `tabla.rs`, mismas marcas y mismo error por vocabulario). Palabra: al principio de palabra de objetivo y criterios, plegada con `tarea::texto::plegar` (sin tildes ni mayusculas); etiqueta: igualdad; ruta: glob sin mayusculas con el `globs()` que ya existe. `dominios_extra` solo anade.
2. **`seleccion/mod.rs` generalizado**: `seleccionar_con(adr, rol, &Entrada, cat, inv, ctx) -> Seleccion` recorre las filas de `rol` y `todos`; `revision()` y `revision_con()` quedan como envolturas con la misma firma (CI y `review` no cambian de llamada). `Entrada` lleva rutas, dominios con senal, etiquetas y roles de complejidad. `Seleccion` gana `skills` y `senales` (las revisoras siguen en `revisoras`). `FilaModelo` gana `diseno: Option<String>` (`—` en errata) e `implementacion`.
3. **Catalogo ampliado (decision 8)**: el bloque yaml y `recursos:` admiten `rol` ∈ {`diseno`, `ejecucion`, `revisor`}, `etiquetas`, `dominios` y `prioridad`. Las filas S02-S05 usan los ids fijos de su columna (agentes `taskcode-plugin:brainstorm-*`) validados contra el inventario (S14); S06/S07 solo eligen entre entradas con metadatos. Con el plugin 0.16.2 tal cual, S07 no elige nada salvo que el proyecto declare `task-workflow` en `recursos:` (es lo que dice la decision 2: «lo instalado sin metadatos no se elige»).
4. **Mapa `roles` ampliado en `tarea.md` (divergencia 1 de F1-15)**: claves opcionales nuevas `agentes`, `reglas` (lista de ids de fila), `senales`, `avisos`, `inventario`, `catalogo` y, en revision, `ronda`; se escriben solo con valor, asi que una tarea de antes se lee y se reescribe byte a byte igual (regla de la adenda de TASK-029 a F1-9). `regla` (vocabulario 0.16) se conserva y queda `null` en lo que calcula tc2.
5. **`plan` usa la seleccion**: `plan_en` lee inventario (`inventario::leer`) y catalogo de la base (`seleccion::orden::fuentes`, la misma funcion de `review`) y se los pasa a `Planificar`; `aplicar` llama a `seleccionar` para `diseno` e `implementacion`, toma el numero y los agentes de diseno de la seleccion (suelo 2 con `puertas-seguridad`, rol de rendimiento con su dominio) y mapea los ids a `roles::ESCALERA`/`CONDICIONALES` por sufijo; escribe `roles.diseno` y `roles.implementacion`. Las peticiones dicen «lanzalo con el agente `<id del inventario>` y el modelo `<m>`»; la linea de `plan` gana `dominios=`, `modelo-diseno=`, `modelo-implementacion=` y `avisos=`. Un agente no instalado no rompe `plan`: se anota el aviso `no-instalado` (S14) y la peticion se escribe igual.
6. **Revision con dominios**: `seleccion_de` (`review`) y `orden::de_tarea` (`tc2 revisoras --tarea`, CI) calculan `dominios()` con la union de `roles.diseno.senales` (congeladas con el plan aprobado) y las del texto vivo y las rutas del diff, con la configuracion de la base: S10, S11 y la excepcion de S13 pasan a evaluarse. El workflow `revision-ia.yml` no cambia.
7. **`approve` muestra la seleccion** que firma la persona: «dominios: ninguno; roles 1; modelos D/I» (o los detectados con su senal), leida de `roles.diseno`/`roles.implementacion`; no recalcula.
8. **`tc2 seleccion TASK-NNN [--rutas <f>] [--tarea-fichero <tarea.md>] [--json]`** en `seleccion/orden.rs`: los tres roles sin escribir nada (contrato para F4-4, F1-16 y la medida del criterio 1); `--tarea-fichero` acepta formato 1 y 2 para medir sobre tareas reales de la 0.16. `tc2 revisoras` no cambia de contrato.
9. **`doctor --recursos`** recoge tambien `roles.*.agentes`. Adenda F4-6 en el ADR F1-15 (dominios activos, mapa ampliado, ids de agente, divergencias).

Sin modelo de pago: nada de esta tarea llama a un modelo ni a la red (la seleccion es pura y el criterio 1 se mide sin modelo).

Duracion estimada: 330 min

Senales: ~14 ficheros de codigo (`seleccion/{mod,tabla,catalogo,dominios,orden}.rs`, `plan/{mod,operacion,plantillas,aprobar}.rs`, `tarea/modelo.rs`, `config/{claves,mod}.rs`, `revision/mod.rs`, `doctor/recursos.rs`), 4 capas (seleccion pura, modelo de tarea y config, ordenes `plan`/`approve`/`review`/`seleccion`, ADR), 3 criterios. Precedentes: TASK-049 150/930, TASK-051 220/130, TASK-052 260/723, TASK-036 210/131, TASK-080 300/137.

### Piezas y limites

- `seleccion/dominios.rs` (nuevo): `dominios`, `FilaDominio`; no conoce a `plan` ni a `revision`.
- `seleccion/tabla.rs` (se extiende): lector de `tabla-dominios`; `FilaModelo` con las tres columnas; vocabulario de acciones de diseno e implementacion ya validado hoy.
- `seleccion/mod.rs` (se extiende): `seleccionar_con` por rol; `revision*` como envolturas.
- `seleccion/catalogo.rs` y `config/{claves,mod}.rs` (se extienden): metadatos `rol`, `etiquetas`, `dominios`, `prioridad`.
- `tarea/modelo.rs` (se extiende): claves opcionales del mapa `roles`.
- `plan/{mod,operacion,plantillas,aprobar}.rs` (se extienden): entrada de inventario y catalogo, anotacion, peticiones con modelo, `approve` que muestra.
- `revision/mod.rs`, `seleccion/orden.rs` (se extienden): dominios en revision; orden `tc2 seleccion`.
- `doctor/recursos.rs` (se extiende): agentes de `roles`.

Fronteras: `seleccion` no depende de `plan` ni de `revision` (hoy importa `plan::heuristica` desde `orden.rs`; se mantiene solo en la capa de orden, no en el nucleo puro); `plan::roles` sigue siendo la fuente de los textos de cada rol y `seleccion` la del numero, los ids y el modelo. Dependencia nueva: `plan` -> `seleccion` (antes no existia).

### Orden de construccion

1. `tabla.rs` (dominios y tres columnas de modelo) y `dominios.rs`, con la tabla adversaria; `revision()` sigue dando lo mismo con dominios vacios (los tests de F6-3 no cambian).
2. `seleccionar_con` para diseno e implementacion y catalogo ampliado; fixtures de la muestra del ADR reproducen sus 10 filas.
3. Mapa `roles` ampliado con ida y vuelta byte a byte de tareas antiguas.
4. `plan` (entrada, anotacion, peticiones) y `approve` (muestra).
5. Revision con dominios, `tc2 seleccion`, `doctor`, adenda del ADR y tabla de las 10 tareas de TaskCodeTwo.

Cada paso deja el arbol en verde; hasta el 4, `plan` se comporta como hoy.

### Alternativas descartadas

- Selector de diseno aparte en `plan/` (sin tocar `seleccion`): dos lectores de la misma tabla y dos modelos del mismo hecho; F1-15 decision 1 pide una sola funcion.
- Recalcular diseno e implementacion en cada orden en vez de congelarlos: D5b, descartada en F1-15 (un plan aprobado cambiaria de agentes sin aprobarlo).
- Blob de `tarea.md` en `roles.<rol>.texto` como «texto aprobado»: el blob del commit de `approve` contiene la propia anotacion (circular) y para recalcular haria falta releer el historial; se congelan las `senales` ya calculadas, que es lo unico que la union necesita. Divergencia con la decision 3, se nombra en la adenda.
- Polimorfismo de `regla` (texto 0.16 o lista de ids): complica el lector con `deny_unknown_fields` y la convivencia con la 0.16; clave nueva `reglas`.

## Troceo

Pasa de 60 min (330). **La persona decidio no trocear**: consigna de Carlos para el H4 (2026-11-22), como en TASK-036. El orden deja la seleccion pura completa y probada (pasos 1-3) antes de tocar `plan` y `review`; un corte a medias no cambia ningun comportamiento visible.

## Lo que el rol no cubrio

- Criterio 1 y 2, por donde se rompe (riesgos, no revisado): inventario incompleto (`no-comprobado`) frente a S14; agente instalado con otro prefijo; tarea con `roles` escritos a mano o reescritos despues de `approve`; evasion por vocabulario en `fix`/`errata` (residual ya aceptado en F1-15, casos 2 y 5 de la PoC S).
- Como se prueba (testing, no revisado): el plan de pruebas de abajo es del rol de arquitectura y no ha pasado por el de testing.
- Criterio 1, dominio (no revisado): si las 10 tareas de TaskCodeTwo elegidas por la regla son representativas; si el suelo 2 y la `tabla-modelo` son lo que haria una persona fuera de la muestra del ADR. Solo lo juzga Carlos.
- Criterio 3, «el orquestador los usa»: F4-4 (TASK-039) no existe. F4-6 deja el contrato (`roles.*` en la tarea, agente y modelo en cada peticion, `tc2 seleccion --json`); que el orquestador lo consuma es de F4-4. La revision (local y CI) si los usa en esta tarea.
- `start` parando si falta algo de la seleccion (decision 9 y 11 de F1-15): fuera de alcance; `doctor --recursos` lo dice.
- Medir el acierto (RF-15): es de F8-6.

## Riesgos aceptados y que los contiene

- Con el plugin 0.16.2, S06/S07 no eligen nada sin `recursos:` declarado (no hay metadatos en `task-workflow` ni en skills de diseno): lo contiene que es la regla escrita (decision 2) y que el aviso sale en la linea; la muestra del ADR obtuvo `task-workflow` del catalogo 0.16, y la fixture lo declara en `recursos:` para reproducirla.
- Congelar `senales` en vez del texto: un agente que reescribe `roles.diseno.senales` y el texto despues de `approve` quita el suelo en revision. Lo contienen las rutas del diff evaluadas en cada ronda con la base y la aprobacion del PR por la persona (F1-8 decision 4); mismo residual que el caso 2 de F1-15.
- Ids de agente con prefijo `taskcode-plugin:` en la tabla del ADR (0.16) frente a `taskcodetwo:` en la 2.0: el mapeo por sufijo a `roles::ESCALERA` lo aguanta; el cambio de prefijo es de F2-12/F9.
- `plan` pasa a leer el disco (inventario) antes de la operacion: la operacion sigue pura y re-ejecutable; lo leido entra como dato con su huella anotada.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

Criterio «En 10 tareas reales la propuesta coincide con la que haria una persona» (2 tests + tabla para Carlos):
1. `seleccion::tests::la_muestra_del_adr_se_reproduce`: fixtures `crates/tc2/tests/fixtures/muestra-f1-15/` con lo que la PoC T uso de las 10 tareas de ToolsaWeb y OpenDockTrading (etiquetas, tipo, complejidad, objetivo, criterios, rutas, inventario y catalogo de la seccion I), copiados en solo lectura; la salida de `seleccionar` para los tres roles debe dar exactamente las columnas Diseño, Implementación, Revisión, Modelo D/I/R y Reglas de `tabla-muestra` (leida del ADR entre sus marcas). Es la medida objetiva: Carlos ya marco «Coincide las 10» sobre esas filas.
2. `tests/seleccion.rs::diez_tareas_de_taskcodetwo` (binario real, `tc2 seleccion --tarea-fichero --rutas --json`): 10 tareas terminadas de este repo por regla fija (ID mas alto por estrato: `fix`, toca `.github/`, solo `docs/adr`, Rust + workflow, palabra de puertas sin ruta, `rendimiento` si existe y si no `simple` sin dominio, complejidad `alta`, seguimiento, H2, H6), con `tarea.md` y la lista de rutas del merge copiadas como fixture; comprueba determinismo (dos ejecuciones, mismo JSON) y genera la tabla que va a `## Resultado` con la columna «Comprobado a mano (Carlos)» vacia.

Criterio «el modelo cambia segun complejidad y carril» (2 tests):
1. `seleccion::tests::modelo_por_rol_carril_y_complejidad`: las 7 filas M01-M07 en los tres roles (errata sin diseno), S13 con `dominio=puertas-seguridad` (ya no baja) y sin el (baja), S15 en diseno e implementacion.
2. `tests/plan.rs::plan_anota_modelos_distintos`: `tc2 plan` sobre una `fix`/`simple` y una `riesgo-alto` en un repo Git temporal: `roles.diseno.modelo` y `roles.implementacion.modelo` distintos en `tarea.md` y el modelo en cada peticion.

Criterio «Cumple F4-6 y lo que cita (RF-12, RF-13, RR-17)» (3 tests):
1. `seleccion::dominios::tests::tabla_adversaria`: tildes y mayusculas, palabra a mitad de otra (no casa), etiqueta exacta, renombrado con los dos nombres, `.Claude/settings.json`, `dominios_extra` que solo anade, union con senales congeladas que no deja quitar un dominio.
2. `tests/plan.rs::plan_approve_review_usan_la_seleccion`: `plan` con «check» en un criterio da 2 roles + unificador y `roles.diseno.senales`; `approve` muestra dominios y roles; `review` con el texto reescrito sin la palabra mantiene el aviso `dominio-sin-revisora` (S11); una tarea de antes de F4-6 se lee y se reescribe byte a byte igual; `doctor --recursos` lista los agentes anotados.
3. `seleccion::tests::sin_red_ni_modelo`: el nucleo de `seleccion` no nombra `std::process`, `std::net`, clientes HTTP ni `modelo_api` fuera de la orden (comprobacion sobre el fuente con `include_str!`), y un agente quitado del inventario sale con `no-instalado` (S14) en diseno.

Pesado: ninguno. El revisor en CI no cambia de workflow; su camino queda cubierto por `tc2 revisoras --tarea` en el test 2 del criterio 3. Sin saldo, no hay E2E con modelo.

### Tras el cierre

- **Columna «Comprobado a mano (Carlos)» de la tabla de las 10 tareas de TaskCodeTwo** en `## Resultado`: solo la puede marcar Carlos (D10a de F1-15). El revisor por pares contrasta la aritmetica (que cada fila es la salida de `tc2 seleccion` y que la regla de muestra da esas 10). Hasta entonces el criterio 1 queda cumplido dentro de muestra (test 1) y pendiente fuera de muestra.

## Lo que necesita decision de una persona

Todas decididas con la opcion recomendada, delegada por Carlos, 2026-11-22; sujetas a `taskctl approve`.

1. Generalizar `seleccion::revision` en `seleccionar_con` por rol, sin selector nuevo en `plan` (opcion recomendada, delegada por Carlos, 2026-11-22).
2. Mapa `roles` con claves opcionales nuevas (`agentes`, `reglas`, `senales`, `avisos`, `inventario`, `catalogo`, `ronda`) escritas solo con valor; `regla` 0.16 se conserva (opcion recomendada, delegada por Carlos, 2026-11-22).
3. «Texto aprobado» como `senales` congeladas en `roles.diseno`, no como blob (divergencia con F1-15 decision 3, nombrada en la adenda) (opcion recomendada, delegada por Carlos, 2026-11-22).
4. `recursos:` y bloque yaml admiten `rol: diseno|ejecucion|revisor`, `etiquetas` y `prioridad`; sin metadatos no se elige (opcion recomendada, delegada por Carlos, 2026-11-22).
5. Agente no instalado: `plan` no falla, anota `no-instalado` y escribe la peticion (opcion recomendada, delegada por Carlos, 2026-11-22).
6. Orden nueva `tc2 seleccion` (con `--tarea-fichero` para formato 1); `tc2 revisoras` conserva su contrato (opcion recomendada, delegada por Carlos, 2026-11-22).
7. Medida del criterio 1 en dos capas: reproduccion exacta de la muestra del ADR (objetiva) y tabla de 10 tareas de TaskCodeTwo que confirma Carlos tras el cierre; ToolsaWeb y OpenDockTrading solo en lectura (opcion recomendada, delegada por Carlos, 2026-11-22).
8. `start` que para por recursos y consumo por el orquestador, fuera de alcance (F3-9/F1-14 y F4-4) (opcion recomendada, delegada por Carlos, 2026-11-22).
9. Sin trocear (decision de Carlos para el H4, 2026-11-22).

## Suposiciones no verificadas

- Que las fixtures de la PoC T se pueden reconstruir desde `tareas/04-terminadas/TASK-073/poc/` y los commits fijos de los pilotos (`93c9618ac156`, `fab8d7e05eee`) sin escribir en ToolsaWeb ni OpenDockTrading: no se ha abierto el guion `poc-f1-15.py` ni su log.
- Que la fila de TASK-306 (S07 `task-workflow`) se reproduce declarando `task-workflow` en `recursos:` con las etiquetas que la PoC tomo de `catalogo-skills.yml` de la 0.16.
- Que `orden::de_tarea` en CI lee `roles.diseno.senales` de la tarea en la rama de estado igual que lee carril y complejidad (no se ha mirado como `revision-ia.yml` obtiene la tarea).
- Que existen en `tareas/04-terminadas` de este repo tareas para los 10 estratos (en particular `fix` y `rendimiento`); si falta uno, el siguiente estrato de la lista lo sustituye y se dice.
- Que el inventario de esta maquina contiene los cinco `taskcode-plugin:brainstorm-*` y el unificador con esos ids (TASK-080 cuenta 46 agentes, no se listaron aqui).
