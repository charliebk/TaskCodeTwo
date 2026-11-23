# Auditoría 4D de TaskCodeTwo

Commit auditado: d82ea9dba1f77577d39b32a4f5caea2e140ae8da

Tarea: TASK-138. Fecha: 2026-11-22. Todo `fichero:línea` de este documento se lee contra ese commit. Las líneas caducan en cuanto otra tarea toca el código.

La regla 4D pide que no haya **D**uplicación (D1), **D**esviación de lo decidido (D2), **D**iluvio de código (D3) ni **D**isfraz (D4), es decir, código o tests que aparentan hacer un trabajo y no lo hacen. Esta auditoría solo diagnostica: no corrige código. Las correcciones van en las tareas de seguimiento (columna `Tarea`).

## Índice

- [Método](#método)
- [Resumen por regla y severidad](#resumen-por-regla-y-severidad)
- [Tabla principal](#tabla-principal)
- [Tareas de seguimiento](#tareas-de-seguimiento)
- [Detalle por grupo](#detalle-por-grupo)
- [Cobertura por módulo](#cobertura-por-módulo)
- [Ficheros sin lectura completa](#ficheros-sin-lectura-completa)
- [Descartes justificados por un ADR](#descartes-justificados-por-un-adr)
- [Límites](#límites)
- [Normalización del consolidador](#normalización-del-consolidador)

## Método

- **Pase 0** (un solo agente, determinista y sin descargas). `ast-grep` 0.44.1 extrae cada `fn`. Un script propio normaliza los cuerpos, calcula su hash y agrupa los iguales entre módulos. Además, grep de nombres repetidos, `cargo clippy --all-targets` con `cognitive_complexity`, `too_many_lines`, `too_many_arguments`, `type_complexity`, `redundant_clone` y `needless_pass_by_value` (con su propio `CARGO_TARGET_DIR`), y reglas `sg` de Disfraz: `#[test]` sin `assert*`, `let _ =` en tests, aserciones que solo miran `is_ok()`/`is_some()`, `#[ignore]` y `Ok(())` de una línea. Los candidatos y los scripts quedan fuera del repo.
- **10 auditores** con rutas exclusivas y lectura en paralelo, sin compilar. A1-A7 se reparten por dominio. A8 lleva lo transversal (raíz de `src`, `xtask`, `distribucion`, workflows, `tests/comun`) y además la Duplicación entre dueños: recibió las pistas de los otros siete e hizo una pasada por conceptos (escritura atómica, frontmatter/YAML, rutas de tarea, dobles de `Plataforma`, invocación de `git`, fechas). A9 relee entero lo que A8 había leído solo por zonas en `lib.rs`, `actualizar_cli.rs`, `descarga.rs`, `distribucion`, `xtask` y `tests/comun`. A10 cubre lo que no tenía dueño: `.github/scripts`, `instalar/`, `plantillas/`, configuración y fixtures. Un undécimo auditor, **A11**, cierra los huecos del primer cruce de cobertura: lee enteros, línea a línea, los 14 ficheros que quedaban (10 que A8 había leído por zonas y los 4 documentos de la raíz sin dueño: README, CHANGELOG, CLAUDE.md y LICENSE), 3 922 líneas.
- **D2** se mide contra la tabla `módulo → ADR → TASK`, sin reabrir ningún ADR. **D3** exige una versión reducida concreta: un aviso de clippy por sí solo no cuenta como caso. **D1** exige como mínimo 10 líneas o 50 tokens normalizados y lógica que no dependa de la API del adaptador.
- **Mutación dirigida, manual y en serie**, solo sobre los candidatos D4 de test que el auditor confirmó leyendo. Se hizo en un único clon con un único `CARGO_TARGET_DIR` bajo `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-138-mutantes\`, nunca en el checkout principal, restaurando el clon con `git checkout -- .` entre pasos. Cada D4 de test trae el commit base, el diff del mutante, la orden `cargo test` exacta y su salida en verde con el mutante puesto. Trae también el control: el mismo test en rojo con el código roto de verdad, para probar que el test corre. Los E2E `#[ignore]` con tokens reales quedan fuera.
- **Sin descargas ni instalaciones**: ni `cargo-mutants` ni `jscpd`. Los D4 de los scripts de shell se demostraron con arneses que ejecutan las líneas del commit tal cual, en una carpeta temporal y sin red.
- **Severidad** (plan, decisiones 4 y 5). D1 y D4 reales son IMPORTANTE como mínimo, y CRITICO solo si tapan un fallo de datos o de seguridad. Una D1 con copias que ya divergen sigue siendo IMPORTANTE, con la marca `divergente`, y va la primera de su grupo. D2 y D3 son MENOR salvo que cambien el comportamiento.
- **Consolidación**: fusión de duplicados según el campo `absorbe:`, normalización de severidades (sección final) y cruce de cobertura contra `git ls-tree -r d82ea9d` fuera de `tareas/` y `docs/`.

## Resumen por regla y severidad

Hay 108 casos distintos: 82 IMPORTANTE (18 de ellos `divergente`), 0 CRITICO y 26 MENOR. Los auditores escribieron 116 fichas; 8 se fusionaron en otras (aparecen en la tabla como «fusionado en»). El seguimiento se reparte en 12 tareas.

| Regla | CRITICO | IMPORTANTE | MENOR | Total |
|---|---|---|---|---|
| D1 Duplicación | 0 | 54 | 0 | 54 |
| D2 Desviación | 0 | 3 | 9 | 12 |
| D3 Diluvio | 0 | 0 | 17 | 17 |
| D4 Disfraz | 0 | 25 | 0 | 25 |
| **Total** | **0** | **82** | **26** | **108** |

No hay ningún CRITICO. Hubo dos candidatos y ninguno lo alcanza. A4-10, la guarda de RN-05 contra `push -f`/`branch -D`, falla con opciones agrupadas, pero hoy ninguna operación pasa esos argumentos. A10-05, la redirección sin https en la rama `wget`, no se pudo reproducir, y `SHA256SUMS` llega por el mismo canal. Ninguno de los dos tapa un fallo de datos o de seguridad que ya exista.

## Tabla principal

`fichero:línea` es la primera ubicación de la ficha; las demás y la evidencia literal están en la sección de detalle (enlace en el ID).

| ID | Severidad | Regla | fichero:línea | Resumen | Evidencia | Verificado | Tarea |
|---|---|---|---|---|---|---|---|
| [A1-01](#a1-01) | IMPORTANTE | D1 | `crates/tc2/src/plataforma/github.rs:742` | Lógica neutral de `leer_puertas_de` (validar rama y rol, leer lo común, repartir por rama) escrita en los dos adaptadores | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A1] |
| [A1-02](#a1-02) | IMPORTANTE `divergente` | D1 | `crates/tc2/src/plataforma/github.rs:434` | `abrir-pr` (validación, idempotencia, `sin-cambios`/`pr-cerrado`) duplicado y ya divergente: en la carrera GitHub da `error motivo=validacion` y GitLab `conflicto motivo=mr-existente` (contra RR-15) | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A1] |
| [A1-03](#a1-03) | IMPORTANTE | D1 | `crates/tc2/src/plataforma/github.rs:412` | Reglas neutrales de `prs-abiertos` (rama inválida se salta, sha no hex es `respuesta-invalida`) implementadas dos veces | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A1] |
| [A1-04](#a1-04) | IMPORTANTE | D1 | `crates/tc2/tests/plataforma.rs:468` | Bucle de la tabla de errores HTTP y servido de fixtures copiado entre las baterías de GitHub y GitLab | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A1] |
| [A1-05](#a1-05) | IMPORTANTE | D1 | `crates/tc2/tests/plataforma.rs:32` | `adaptador(mock, roles)` reescribe `con_personas(mock, roles, &[PERSONA])` en el mismo fichero | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A1] |
| [A1-06](#a1-06) | IMPORTANTE | D1 | `crates/tc2/tests/plataforma_e2e.rs:74` | Cliente HTTP del arnés E2E copiado entre los E2E de GitHub y GitLab | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A1] |
| [A1-07](#a1-07) | IMPORTANTE | D4 | `crates/tc2/src/plataforma/github.rs:960` | `rulesets` lee solo la primera página (100) sin `Link rel=next` ni fallo cerrado; su hermana `rulesets_de` sí pagina | fragmento y entrada que no hace el trabajo | lectura: entrada derivada del código, sin ejecutar | pendiente [A1] |
| [A1-08](#a1-08) | IMPORTANTE | D4 | `crates/tc2/src/plataforma/http.rs:473` | Test de bases inválidas solo mira `is_err()`: quitar la regla de host que empieza por `.` o `-` de `Base::nueva` sigue en verde | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A1] |
| [A1-09](#a1-09) | IMPORTANTE | D4 | `crates/tc2/tests/prs_abiertos.rs:205` | `prs_abiertos.rs` dice probar que los dos adaptadores dan la misma lista y el caso de sha inválido y rama rara solo usa GitHub | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A1] |
| [A1-10](#a1-10) | IMPORTANTE | D4 | `crates/tc2/tests/plataforma_gitlab.rs:842` | La aserción «el revisor no lee puertas» pasa por falta de token, no por la comprobación de rol | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A1] |
| [A1-11](#a1-11) | IMPORTANTE | D4 | `crates/tc2/tests/plataforma.rs:410` | Ningún caso ejercita el 403 con `retry-after` y sin `x-ratelimit-remaining` (límite secundario) de `es_limite` | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A1] |
| [A1-12](#a1-12) | MENOR | D3 | `crates/tc2/src/plataforma/gitlab.rs:704` | `estado_puertas` reescribe en línea `solo_nivel` del mismo fichero | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A2-01](#a2-01) | (IMPORTANTE) | D1 | `crates/tc2/src/revision/guardar.rs:96` | Preámbulo de `aplicar` copiado en las dos operaciones de `revision` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | fusionado en A8-07 |
| [A2-02](#a2-02) | IMPORTANTE | D1 | `crates/tc2/src/hallazgos/mod.rs:695` | `validar_uno` y `validar_cierre` repiten la extracción del id y la closure del `Fallo` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A2] |
| [A2-03](#a2-03) | IMPORTANTE | D1 | `crates/tc2/src/hallazgos/mod.rs:956` | Los dos bucles de `validar_con` repiten el corte por `TOPE_HALLAZGOS` y la conversión `Fallo -> Rechazo` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A2] |
| [A2-04](#a2-04) | IMPORTANTE | D1 | `crates/tc2/src/hallazgos/mod.rs:1193` | Tres funciones pintan la misma fila Markdown de un hallazgo con los mismos topes de `plano` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A2] |
| [A2-05](#a2-05) | IMPORTANTE | D1 | `crates/tc2/src/hallazgos/orden.rs:664` | Siete subórdenes de `tc2 hallazgos` escriben a mano el mismo analizador de opciones | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A2] |
| [A2-06](#a2-06) | (IMPORTANTE) | D1 | `crates/tc2/src/revision/mod.rs:553` | Doble de `Plataforma` copiado entre el test unitario y el de integración de `revision` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | fusionado en A8-05 |
| [A2-07](#a2-07) | MENOR | D2 | `crates/tc2/src/hallazgos/mod.rs:11` | La cabecera del módulo contradice la decisión revisada del ADR y la versión de contrato que el código usa | cita del ADR o criterio y línea que se aparta | lectura: cita del ADR o criterio | — (MENOR: solo documento) |
| [A2-08](#a2-08) | MENOR | D2 | `crates/tc2/src/hallazgos/mod.rs:924` | El esquema exige `hallazgos` en la respuesta incremental y `validar_con` acepta solo `cierres` | cita del ADR o criterio y línea que se aparta | lectura: cita del ADR o criterio | — (MENOR: solo documento) |
| [A2-09](#a2-09) | MENOR | D3 | `crates/tc2/src/hallazgos/orden.rs:200` | Siete brazos de `match` idénticos salvo la subórden | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A2-10](#a2-10) | MENOR | D3 | `crates/tc2/src/revision/mod.rs:232` | `paso` pinta dos veces a mano la conversión `Fallo -> Informe` | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A2-11](#a2-11) | IMPORTANTE | D4 | `crates/tc2/src/hallazgos/mod.rs:1561` | Ningún test comprueba el dictamen de `veredicto_ronda` en modo `tope` ni `ligera` | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A2] |
| [A2-12](#a2-12) | IMPORTANTE | D4 | `crates/tc2/src/hallazgos/orden.rs:653` | La rama «mezcla con conflictos -> `None`» de `desde_incremental` no la ejecuta ningún test | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A2] |
| [A2-13](#a2-13) | IMPORTANTE | D4 | `crates/tc2/src/hallazgos/mod.rs:982` | «Cada regla del cierre rechaza con su motivo» no prueba el tope de cierres (`fuera-de-tope`) | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A2] |
| [A3-01](#a3-01) | IMPORTANTE | D4 | `crates/tc2/src/alta.rs:1051` | `anotacion_ida_y_vuelta` solo mira `is_some()`; `desde_anotacion` pierde todo menos el título | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A3] |
| [A3-02](#a3-02) | IMPORTANTE | D4 | `crates/tc2/src/plan/aprobar.rs:117` | La rama de `es_esqueleto` del plan con unificador no la ejercita ningún test (se llama con `rutas = &[]`) | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A3] |
| [A3-03](#a3-03) | IMPORTANTE `divergente` | D1 | `crates/tc2/src/plan/mod.rs:139` | `leer_fichero` de `plan` e `import` copiado y divergente: `import` no sanea ruta ni error (F1-8) ni rechaza vacío | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A3] |
| [A3-04](#a3-04) | (IMPORTANTE) | D1 | `crates/tc2/src/plan/guardar.rs:113` | Preámbulo de las operaciones de diseño copiado entre `guardar` y `aprobar` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | fusionado en A8-07 |
| [A3-05](#a3-05) | IMPORTANTE | D1 | `crates/tc2/src/plan/mod.rs:283` | Secuencia de `approve` copiada para `--hito` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A3] |
| [A3-06](#a3-06) | IMPORTANTE `divergente` | D1 | `crates/tc2/tests/alta.rs:29` | Resto tras la fusión: `valor` divergente (importar con `trim_end`, alta sin él) y `como_persona` ~ `hito_como_persona` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XT] |
| [A3-07](#a3-07) | MENOR | D2 | `crates/tc2/src/tarea/modelo.rs:65` | Comentarios que se apartan del código y del ADR que el código ya cumple (F4-6) | cita del ADR o criterio y línea que se aparta | lectura: cita del ADR o criterio | — (MENOR: solo documento) |
| [A3-08](#a3-08) | IMPORTANTE | D2 | `crates/tc2/src/alta.rs:75` | Dos definiciones de «listo»: `new`/`import` aceptan enunciados que `plan` rechaza después (RF-01) | cita del ADR o criterio y línea que se aparta | lectura: cita del ADR o criterio | pendiente [A3] |
| [A3-09](#a3-09) | MENOR | D3 | `crates/tc2/src/alta.rs:503` | Envoltorio `leidas` que solo reenvía a `estado::tareas` | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A4-01](#a4-01) | IMPORTANTE `divergente` | D1 | `crates/tc2/src/git/rama.rs:88` | Consulta de ramas de tarea locales y remotas duplicada y fallo del remoto tratado en cuatro sitios, dos siempre como `sinred` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A4] |
| [A4-02](#a4-02) | IMPORTANTE `divergente` | D1 | `crates/tc2/src/git/estado.rs:243` | Envoltorio «lanzar git y traducir el fallo» cuatro veces; `leer_blobs` usa la primera línea cruda de stderr | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A4] |
| [A4-03](#a4-03) | IMPORTANTE | D1 | `crates/tc2/src/estado/transaccion.rs:353` | 19 líneas idénticas en el punto único de escritura (`intento`) de la transacción | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A4] |
| [A4-04](#a4-04) | IMPORTANTE | D1 | `crates/tc2/src/git/tag.rs:81` | Informe de un `publicar` fallido copiado en `tag` y `remoto` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A4] |
| [A4-05](#a4-05) | IMPORTANTE | D1 | `crates/tc2/src/git/estado.rs:451` | `hay_remoto` y `url_origin` lanzan el mismo `ls-remote --get-url origin` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A4] |
| [A4-06](#a4-06) | IMPORTANTE | D1 | `crates/tc2/src/git/rama.rs:318` | `crear` y `reanudar` del worktree de tarea repiten arranque y bloque de carpeta ocupada | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A4] |
| [A4-07](#a4-07) | IMPORTANTE | D1 | `crates/tc2/src/git/guardar.rs:158` | Analizador de `git status --porcelain -z` escrito dos veces | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A4] |
| [A4-08](#a4-08) | IMPORTANTE | D1 | `crates/tc2/tests/estado_bordes.rs:19` | Ayudantes de los tests de estado (`rapida`, `escribir_estado`, `rama_en`) copiados entre ficheros | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A4] |
| [A4-09](#a4-09) | (IMPORTANTE) | D1 | `crates/tc2/tests/toma.rs:33` | Ayudantes para lanzar `tc2` contra el remoto de la fixture copiados entre `toma` y `liberar` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | fusionado en A8-01, A8-16 |
| [A4-10](#a4-10) | IMPORTANTE | D4 | `crates/tc2/src/git/puerto.rs:613` | `es_prohibido` no ve opciones cortas agrupadas: `push -qf`, `branch -qD` y `worktree remove -ff` pasan la guarda y git las ejecuta | tabla de `es_prohibido`, cita F1-4 y salida de git | sí: git 2.56 real (tc2 sin compilar) | pendiente [A4] |
| [A4-11](#a4-11) | MENOR | D3 | `crates/tc2/src/estado/montaje.rs:63` | Dos funciones de 4 líneas que repiten `renombrar(i, OP)` | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A4-12](#a4-12) | MENOR | D3 | `crates/tc2/src/git/puerto.rs:262` | `Git::con_entorno` solo reenvía a `con_entorno_extra` | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A5-01](#a5-01) | IMPORTANTE `divergente` | D1 | `crates/tc2/src/seleccion/catalogo.rs:232` | Leer skills de una carpeta, en `inventario` y `catalogo`, ya divergente: el catálogo descarta en silencio lo ilegible (contra C8) | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A5] |
| [A5-02](#a5-02) | IMPORTANTE `divergente` | D1 | `crates/tc2/src/seleccion/catalogo.rs:265` | Resolución de la carpeta instalada del plugin con reglas distintas (el catálogo no exige el `scope` del proyecto) | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A5] |
| [A5-03](#a5-03) | IMPORTANTE | D2 | `crates/tc2/src/seleccion/mod.rs:344` | Con revisora de dominio declarada y no instalada, sale `no-instalado` en vez del aviso `dominio-sin-revisora` que exige F1-15 (S11) | cita del ADR o criterio y línea que se aparta | lectura: cita del ADR o criterio | pendiente [A5] |
| [A5-04](#a5-04) | MENOR | D2 | `crates/tc2/src/seleccion/orden.rs:126` | `fuentes` filtra `Origen::Proyecto` y no elige agentes del proyecto; hoy ninguna fila los pide | cita del ADR o criterio y línea que se aparta | lectura: cita del ADR o criterio | — (MENOR: solo documento) |
| [A5-05](#a5-05) | MENOR | D2 | `crates/tc2/src/recursos.rs:13` | La cabecera de `recursos.rs` afirma que `init` y `start` ya paran por recursos | cita del ADR o criterio y línea que se aparta | lectura: cita del ADR o criterio | — (MENOR: solo documento) |
| [A5-06](#a5-06) | IMPORTANTE | D4 | `crates/tc2/src/seleccion/mod.rs:849` | `las_reglas_se_leen_de_la_tabla` convierte el `Err` en `default()` y comprueba justo lo que da el `default` | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A5] |
| [A5-07](#a5-07) | (IMPORTANTE) | D1 | `crates/tc2/tests/recursos.rs:143` | Lanzador de `tc2` con entorno aislado copiado en los tests de `recursos` y `doctor` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | fusionado en A8-02 |
| [A5-08](#a5-08) | IMPORTANTE | D1 | `crates/tc2/tests/doctor.rs:958` | Comprobación de «5 líneas, JSON con todo y `y N mas`» copiada en dos tests | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A5] |
| [A5-09](#a5-09) | IMPORTANTE | D1 | `crates/tc2/tests/doctor.rs:1017` | `doctor_no_escribe_nada` copia el cuerpo de `doctor_contra` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A5] |
| [A5-10](#a5-10) | MENOR | D3 | `crates/tc2/src/seleccion/orden.rs:491` | Cierres y bloques repetidos entre `paso` y `paso_seleccion` | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A5-11](#a5-11) | MENOR | D3 | `crates/tc2/src/doctor/recursos.rs:41` | Dos enums con las mismas cuatro variantes y un `match` que traduce | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A5-12](#a5-12) | MENOR | D3 | `crates/tc2/src/doctor/recursos.rs:162` | Dos bucles de 13-14 líneas que solo cambian la lista y el `TipoRecurso` | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A5-13](#a5-13) | MENOR | D3 | `crates/tc2/src/seleccion/mod.rs:1066` | Vector `hash` que se rellena y nunca se lee en un test | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A5-14](#a5-14) | MENOR | D3 | `crates/tc2/src/inventario.rs:1160` | Tres tests escriben en línea el ayudante `incidencias` | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A6-01](#a6-01) | (IMPORTANTE) | D1 | `crates/tc2/src/roadmap/mod.rs:303` | Conversión `Fallo -> Informe` escrita a mano en cuatro órdenes de A6 | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | fusionado en A8-06 |
| [A6-02](#a6-02) | IMPORTANTE | D1 | `crates/tc2/src/grafo.rs:642` | `linea_esperada` (test) reimplementa `ids_de_la_linea` del crate | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A6] |
| [A6-03](#a6-03) | IMPORTANTE | D1 | `crates/tc2/tests/estimacion.rs:278` | Operación de prueba `Poner`, `fila` y `terminada` copiadas entre los tests de estimación y progreso | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A6] |
| [A6-04](#a6-04) | IMPORTANTE `divergente` | D1 | `crates/tc2/tests/roadmap.rs:437` | Juego de tareas de `tests/siguiente.rs` copiado en `tests/roadmap.rs` y ya distinto de lo que dice su comentario | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A6] |
| [A6-05](#a6-05) | IMPORTANTE | D1 | `crates/tc2/src/siguiente.rs:139` | Doble `EnMemoria` copiado en los tests de `siguiente` aunque el de `roadmap::tests` es `pub(crate)` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A6] |
| [A6-06](#a6-06) | IMPORTANTE | D1 | `crates/tc2/tests/solape.rs:514` | `c5_sin_red_y_sin_origin_sin_claves` reescribe `start_de_luis` del mismo fichero | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A6] |
| [A6-07](#a6-07) | IMPORTANTE | D4 | `crates/tc2/src/progreso/mod.rs:479` | `c7_fuera_de_un_repo` dice probar el respaldo a la carpeta del proceso y todas sus entradas dan `SIN_TAREA` igual | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A6] |
| [A6-08](#a6-08) | MENOR | D3 | `crates/tc2/src/solape.rs:405` | Función privada que solo reenvía a `tablero::sanear` | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A7-01](#a7-01) | IMPORTANTE | D1 | `crates/tc2/src/init/mod.rs:187` | Esqueleto «plan como dato» copiado a mano entre `init` y `desinstalar` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A7] |
| [A7-02](#a7-02) | (IMPORTANTE) | D1 | `crates/tc2/src/desinstalar/mod.rs:571` | Escritura atómica (temporal + rename) en `desinstalar` y `confianza`, ya divergente | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | fusionado en A8-04 |
| [A7-03](#a7-03) | IMPORTANTE | D1 | `crates/tc2/src/confianza/mod.rs:198` | `estado` y `evaluar` de `confianza` repiten lectura, cierre, huella y `Evaluacion` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A7] |
| [A7-04](#a7-04) | IMPORTANTE `divergente` | D1 | `crates/tc2/src/ciclo/liberar.rs:423` | Cinco ayudantes de test copiados entre los módulos de `ciclo`, dos ya divergentes | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A7] |
| [A7-05](#a7-05) | IMPORTANTE | D1 | `crates/tc2/src/ciclo/mod.rs:505` | Los dos tests unitarios de `ciclo/mod.rs` montan el mismo escenario git línea a línea | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A7] |
| [A7-06](#a7-06) | (IMPORTANTE) | D1 | `crates/tc2/tests/init.rs:210` | Instantánea de carpeta copiada entre los tests de `init` y `desinstalar`, ya divergente | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | fusionado en A8-12 |
| [A7-07](#a7-07) | IMPORTANTE | D2 | `crates/tc2/src/init/aplicar.rs:25` | Texto renderizado en español en el sitio y metido en `{error}`: con `TC2_IDIOMA=en` el diagnóstico mezcla idiomas (F2-8) | cita del ADR o criterio y línea que se aparta | lectura: cita del ADR o criterio | pendiente [A7] |
| [A7-08](#a7-08) | MENOR | D3 | `crates/tc2/src/init/mod.rs:227` | `linea()` y `json()` repiten el literal `extra` y `..self.comun()` | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A7-09](#a7-09) | IMPORTANTE | D4 | `crates/tc2/src/config/mod.rs:534` | `del_arbol` da `motivo=sin-config` («crea la configuración con tc2 init») ante cualquier error de lectura, también no UTF-8 o sin permiso | fragmento y entrada que no hace el trabajo | lectura: entrada derivada del código, sin ejecutar | pendiente [A7] |
| [A7-10](#a7-10) | IMPORTANTE | D4 | `crates/tc2/src/i18n/guarda.rs:17` | `ningun_literal_fuera_del_catalogo` solo busca cuatro formas textuales; ya se le escapa el literal de A7-07 | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A7] |
| [A7-11](#a7-11) | IMPORTANTE | D4 | `crates/tc2/tests/init.rs:1693` | El test dice cubrir `develop` sin publicar y provoca `ramas-por-delante`; `ramas-sin-remoto` (TASK-118) no lo ejercita nadie | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A7] |
| [A8-01](#a8-01) | IMPORTANTE | D1 | `crates/tc2/tests/aprobar.rs:42` | Escenario «remoto bare con ssh falso» (`entorno_ssh`, `identificar`, `git`...) copiado en 10 ficheros de test en vez de `tests/comun/` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XT] |
| [A8-02](#a8-02) | IMPORTANTE `divergente` | D1 | `crates/tc2/tests/aprobar.rs:78` | Lanzador `CARGO_BIN_EXE_tc2` reescrito en 26 ficheros de test y divergente en qué tokens quita del entorno | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XT] |
| [A8-03](#a8-03) | IMPORTANTE | D1 | `crates/tc2/src/alta.rs:673` | Formateo `AAAA-MM-DDTHH:MM:SSZ` dos veces en producción (`alta`, `toma`) y el algoritmo civil otra vez en un test y en xtask | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XU] |
| [A8-04](#a8-04) | IMPORTANTE `divergente` | D1 | `crates/tc2/src/huella.rs:89` | Escritura atómica tres veces (`huella`, `desinstalar`, `confianza`); solo `huella` reintenta el `rename` en Windows | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XU] |
| [A8-05](#a8-05) | IMPORTANTE `divergente` | D1 | `crates/tc2/src/identidad.rs:649` | Cinco dobles de `trait Plataforma` reescriben los métodos que no usan y divergen al llamarlos (`sin_uso`, a mano, `panic!`) | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XT] |
| [A8-06](#a8-06) | IMPORTANTE | D1 | `crates/tc2/src/roadmap/mod.rs:303` | Conversión `Fallo -> Informe` de la rama de estado escrita a mano en nueve sitios de cinco dueños | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XO] |
| [A8-07](#a8-07) | IMPORTANTE | D1 | `crates/tc2/src/revision/guardar.rs:96` | Preámbulo de `Operacion::aplicar` con dueño copiado seis veces; `ciclo` ya lo tiene extraído | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XO] |
| [A8-08](#a8-08) | IMPORTANTE | D1 | `crates/tc2/src/revision/mod.rs:201` | Adaptador «informe de la transacción -> informe de la orden» cuatro veces | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XO] |
| [A8-09](#a8-09) | IMPORTANTE | D1 | `crates/tc2/src/ciclo/mod.rs:376` | `con_dueno` y `escribir` idénticos en `ciclo` y `plan` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XO] |
| [A8-10](#a8-10) | IMPORTANTE | D1 | `crates/tc2/src/ciclo/mod.rs:166` | Política de lectura de la configuración de la base escrita dos veces (`ciclo`, `plan`) | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XO] |
| [A8-11](#a8-11) | IMPORTANTE `divergente` | D1 | `crates/tc2/tests/identidad.rs:33` | Dos servidores HTTP simulados con `Servidor::nuevo` idéntico y `atender` divergente, además de `comun::http_mock` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XT] |
| [A8-12](#a8-12) | IMPORTANTE `divergente` | D1 | `crates/tc2/tests/init.rs:211` | Seis recorridos de carpeta para fotografiar «lo que la orden no puede cambiar», ya distintos en lo que ven | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XT] |
| [A8-13](#a8-13) | IMPORTANTE | D1 | `crates/tc2/src/actualizar_cli.rs:788` | Comando de shell según el sistema copiado tres veces en los tests de `ejecutar_con_limite` | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A8] |
| [A8-14](#a8-14) | MENOR | D3 | `crates/tc2/src/lib.rs:355` | Siete bloques «carpeta actual o informe» en `lib.rs` donde basta `tablero::carpeta_actual` | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A8-15](#a8-15) | IMPORTANTE | D4 | `crates/tc2/src/instalacion.rs:56` | El único test de `detectar_modo` no pasa por las ramas `opt/homebrew` ni `winget/links` | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A8] |
| [A8-16](#a8-16) | IMPORTANTE `divergente` | D1 | `crates/tc2/tests/ciclo.rs:198` | Operación de prueba `Escribir`, `salida`, `con_fila` y `preparar` copiadas entre siete ficheros, ya divergentes | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XT] |
| [A8-17](#a8-17) | IMPORTANTE `divergente` | D1 | `crates/tc2/src/identidad.rs:229` | «Leer con tope» escrito ocho veces y divergente en qué aceptan | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XU] |
| [A8-18](#a8-18) | IMPORTANTE | D1 | `crates/tc2/src/ciclo/operacion.rs:297` | Ayudantes `estado` + `motivo` de los tests unitarios de `Operacion` copiados en varios módulos | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XT] |
| [A8-19](#a8-19) | IMPORTANTE | D1 | `crates/tc2/src/revision/operacion.rs:185` | Formato de la anotación con dueño escrito y parseado a mano en cada operación | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XO] |
| [A8-20](#a8-20) | IMPORTANTE `divergente` | D1 | `crates/tc2/tests/agente_git/mod.rs:40` | Git de sistema para preparar escenarios escrito cuatro veces, divergente en qué aísla | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XT] |
| [A8-21](#a8-21) | MENOR | D3 | `crates/tc2/src/lib.rs:134` | `ejecutar_en` repite `con_json(resto)` 25 veces | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A9-01](#a9-01) | IMPORTANTE `divergente` | D1 | `crates/tc2/src/actualizar_cli.rs:574` | «Esperar a un hijo con plazo y matarlo» dos veces; solo `verificacion` mata el árbol entero | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [XU] |
| [A9-02](#a9-02) | IMPORTANTE | D4 | `crates/tc2/src/actualizar_cli.rs:406` | `comprobar_nuevo` (el binario nuevo arranca y dice versión, target y sha256) no tiene test que lo haga fallar | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A8] |
| [A9-03](#a9-03) | IMPORTANTE | D1 | `xtask/src/workflows.rs:2712` | El cierre `runs_de` de un test reescribe `runs_del_job` del mismo módulo | los fragmentos y su diff normalizado | lectura: fragmentos literales del commit | pendiente [A8] |
| [A9-04](#a9-04) | MENOR | D3 | `crates/distribucion/src/lib.rs:258` | `clase_de_tag` repite el predicado de `version_valida` y un mensaje | antes y después (versión reducida) | lectura: versión reducida | — (MENOR: solo documento) |
| [A10-01](#a10-01) | IMPORTANTE | D4 | `.github/scripts/probar-actualizar.sh:60` | `! cmp -s` bajo `set -eu` nunca para: la guarda del ensayo de `actualizar --cli` no comprueba nada | líneas del script y salida del arnés | sí: arnés que ejecuta las líneas del commit | pendiente [A10] |
| [A10-02](#a10-02) | IMPORTANTE | D4 | `.github/scripts/probar-instalador.sh:268` | El modo `publico` de `probar-instalador.sh` imprime VERDE si el instalador sale 0, sin comprobar binario ni versión | líneas del script y salida del arnés | sí: arnés que ejecuta las líneas del commit | pendiente [A10] |
| [A10-03](#a10-03) | MENOR | D2 | `plantillas/gitlab/verificacion.gitlab-ci.yml:103` | La plantilla de GitLab calcula la clave de reutilización sin las órdenes (contrato de TASK-042) | cita del ADR o criterio y línea que se aparta | lectura: cita del ADR o criterio | — (MENOR: solo documento) |
| [A10-04](#a10-04) | MENOR | D3 | `crates/tc2/tests/fixtures/plataforma/github/observado.json:1` | `observado.json` y `observado-e2e.json` (518 líneas) no los lee nadie; los fixtures de GitLab citan un fichero que no existe | antes y después (versión reducida) | grep contra el commit | — (MENOR: solo documento) |
| [A10-05](#a10-05) | IMPORTANTE | D4 | `instalar/install.sh:47` | La rama `wget` de `install.sh` no limita la redirección a https como promete el comentario | fragmento y entrada que no hace el trabajo | **no**: wget sin reproducir | pendiente [A10] |
| [A11-01](#a11-01) | IMPORTANTE | D4 | `.github/workflows/release.yml:284` | `! cmp -s` en mitad del paso «vY» de `release.yml` nunca para: la guarda de que el `sed` cambió la versión no comprueba nada (mismo patrón que A10-01) | líneas del script y salida del arnés | sí: arnés que ejecuta las líneas del commit | pendiente [A10] |
| [A11-02](#a11-02) | IMPORTANTE | D4 | `crates/tc2/src/instalacion.rs:77` | Cuatro de las cinco órdenes de `orden_del_gestor` solo se prueban «no vacías» y la deduplicación de `dirs_de_instalacion` no la ejercita ningún test | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A8] |
| [A11-03](#a11-03) | IMPORTANTE | D4 | `crates/tc2/src/dato.rs:73` | Ningún test escapa `*`, `_`, `~` ni `&` en `plano`: quitar `&` dejaría pasar `&#x202E;` y la suite sigue en verde | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A8] |
| [A11-04](#a11-04) | IMPORTANTE | D4 | `crates/tc2/src/contrato.rs:336` | El brazo de `cadena_json` que escapa los controles U+0000-U+001F no lo pasa ningún test: sin él, `--json` sale con ESC crudo y JSON inválido | diff del mutante, orden `cargo test` y salida en verde y en rojo | sí: mutante en verde y control en rojo | pendiente [A8] |
| [A11-05](#a11-05) | MENOR | D2 | `crates/tc2/src/main.rs:34` | `main.rs` escribe a mano los códigos de salida 2 y 1, contra «la tabla vive aquí y solo aquí» de `contrato.rs` (F1-4, F2-3) | cita del ADR o criterio y línea que se aparta | lectura: cita del ADR o criterio | — (MENOR: solo documento) |
| [A11-06](#a11-06) | MENOR | D2 | `.github/workflows/ci.yml:3` | La cabecera de `ci.yml` anuncia como futuro el reutilizable `verificacion` de F5-1, ya entregado | cita del ADR o criterio y línea que se aparta | lectura: cita del ADR o criterio | — (MENOR: solo documento) |
| [A11-07](#a11-07) | MENOR | D2 | `CHANGELOG.md:49` | El CHANGELOG nombra `taskctl actualizar --cli`, orden que no existe; la entregada es `tc2 actualizar --cli` | cita del ADR o criterio y línea que se aparta | lectura y `taskctl --help` ejecutado | — (MENOR: solo documento) |

## Tareas de seguimiento

> Por decisión de Carlos (2026-11-22) las correcciones esperan a que el plugin TaskCode termine de arreglar su versión actual; las tareas de seguimiento quedan preparadas en tareas/04-terminadas/TASK-138/seguimiento/ y no se crean todavía.

Hay una tarea por grupo o dueño con todos sus IMPORTANTE y CRITICO. Las D1 entre dueños se agrupan por la abstracción común que hay que extraer (grupos XT, XO y XU, que salen del reparto por abstracción de A8). Los MENOR solo constan en este documento. Ninguna tarea depende de otra ni reordena la cadena H4 ni TASK-040. La columna `Tarea` dice `pendiente [grupo]` hasta que cada tarea se cree con `taskctl import`; entonces se cambia por su `TASK-NNN`.

| Grupo | Tarea | Casos (en orden: `divergente` primero) |
|---|---|---|
| A1 — Plataforma e identidad | pendiente [A1] | [A1-02](#a1-02), [A1-01](#a1-01), [A1-03](#a1-03), [A1-04](#a1-04), [A1-05](#a1-05), [A1-06](#a1-06), [A1-07](#a1-07), [A1-08](#a1-08), [A1-09](#a1-09), [A1-10](#a1-10), [A1-11](#a1-11) |
| A2 — Hallazgos y revisión | pendiente [A2] | [A2-02](#a2-02), [A2-03](#a2-03), [A2-04](#a2-04), [A2-05](#a2-05), [A2-11](#a2-11), [A2-12](#a2-12), [A2-13](#a2-13) |
| A3 — Plan, importar, tarea y alta | pendiente [A3] | [A3-03](#a3-03), [A3-05](#a3-05), [A3-08](#a3-08), [A3-01](#a3-01), [A3-02](#a3-02) |
| A4 — Git, rama de estado y toma | pendiente [A4] | [A4-01](#a4-01), [A4-02](#a4-02), [A4-03](#a4-03), [A4-04](#a4-04), [A4-05](#a4-05), [A4-06](#a4-06), [A4-07](#a4-07), [A4-08](#a4-08), [A4-10](#a4-10) |
| A5 — Selección, inventario, recursos y doctor | pendiente [A5] | [A5-01](#a5-01), [A5-02](#a5-02), [A5-03](#a5-03), [A5-08](#a5-08), [A5-09](#a5-09), [A5-06](#a5-06) |
| A6 — Estimación, progreso, roadmap y tablero | pendiente [A6] | [A6-04](#a6-04), [A6-02](#a6-02), [A6-03](#a6-03), [A6-05](#a6-05), [A6-06](#a6-06), [A6-07](#a6-07) |
| A7 — Config, init, desinstalar, i18n, confianza, ciclo y verificación | pendiente [A7] | [A7-04](#a7-04), [A7-01](#a7-01), [A7-03](#a7-03), [A7-05](#a7-05), [A7-07](#a7-07), [A7-09](#a7-09), [A7-10](#a7-10), [A7-11](#a7-11) |
| A8 — Transversal: actualizar_cli, instalacion, contrato, dato y xtask | pendiente [A8] | [A8-13](#a8-13), [A9-03](#a9-03), [A8-15](#a8-15), [A9-02](#a9-02), [A11-02](#a11-02), [A11-03](#a11-03), [A11-04](#a11-04) |
| A10 — Scripts de CI e instaladores | pendiente [A10] | [A10-01](#a10-01), [A10-02](#a10-02), [A10-05](#a10-05), [A11-01](#a11-01) |
| Abstracción común — soporte de tests compartido (tests/comun y módulos `prueba`) | pendiente [XT] | [A8-02](#a8-02), [A8-11](#a8-11), [A8-12](#a8-12), [A8-16](#a8-16), [A8-20](#a8-20), [A8-05](#a8-05), [A3-06](#a3-06), [A8-01](#a8-01), [A8-18](#a8-18) |
| Abstracción común — capa de órdenes y operaciones con dueño | pendiente [XO] | [A8-07](#a8-07), [A8-19](#a8-19), [A8-06](#a8-06), [A8-08](#a8-08), [A8-09](#a8-09), [A8-10](#a8-10) |
| Abstracción común — utilidades de E/S y tiempo | pendiente [XU] | [A8-04](#a8-04), [A8-17](#a8-17), [A9-01](#a9-01), [A8-03](#a8-03) |

## Detalle por grupo

Cada ficha es la del auditor tal cual, con las notas `normalizacion:` del consolidador cuando las hay. Los encabezados internos van rebajados de nivel.

### A1 — Plataforma e identidad

<a id="a1-02"></a>

#### A1-02 · D1 · IMPORTANTE divergente · pendiente [A1]

_`abrir-pr` (validación, idempotencia, `sin-cambios`/`pr-cerrado`) duplicado y ya divergente: en la carrera GitHub da `error motivo=validacion` y GitLab `conflicto motivo=mr-existente` (contra RR-15)_

- id: A1-02
- regla: D1
- ambito: codigo
- marca: divergente
- ubicacion: crates/tc2/src/plataforma/github.rs:434 ~ crates/tc2/src/plataforma/gitlab.rs:561
- modulo: plataforma (adaptadores GitHub y GitLab: `abrir-pr` y `merge-pr`)
- severidad: IMPORTANTE (divergente)
- motivo_severidad: D1 real de logica de contrato que no depende de la API (validacion de entrada, idempotencia y lineas `sin-cambios`/`pr-cerrado`). Las copias ya divergen en comportamiento: en la carrera de `abrir-pr` sin PR que releer, GitHub da `abrir-pr error motivo=validacion` (codigo 1) y GitLab `abrir-pr conflicto motivo=mr-existente` (codigo 6), contra RR-15 («dan la misma linea byte a byte», mod.rs:9). Por la decision 5 del plan queda IMPORTANTE con marca `divergente`, no CRITICO (no tapa fallo de datos ni de seguridad).
- estado: confirmado
- grupo_seguimiento: A1 — Plataforma e identidad (va primera en su grupo por `divergente`)
- evidencia: fragmentos literales con fichero:linea en la seccion «Evidencia» de abajo (contra d82ea9d)

##### Evidencia

##### abrir-pr: validacion e idempotencia (13 lineas identicas salvo `pr_abierto`/`mr_abierto`)
```text
crates/tc2/src/plataforma/github.rs:434:         if !rama_valida(rama) || !rama_valida(base) {
crates/tc2/src/plataforma/github.rs:435:             return uso(ABRIR_PR, "rama-invalida", &t::RAMA_NO_VALIDA);
crates/tc2/src/plataforma/github.rs:436:         }
crates/tc2/src/plataforma/github.rs:437:         if titulo.trim().is_empty() || titulo.chars().any(char::is_control) {
crates/tc2/src/plataforma/github.rs:438:             return uso(ABRIR_PR, "titulo-invalido", &t::TITULO_NO_VALIDO);
crates/tc2/src/plataforma/github.rs:439:         }
crates/tc2/src/plataforma/github.rs:440:         match self.pr_abierto(rama, base) {
crates/tc2/src/plataforma/github.rs:441:             Ok(Some(n)) => {
crates/tc2/src/plataforma/github.rs:442:                 return Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string());
crates/tc2/src/plataforma/github.rs:443:             }
```
```text
crates/tc2/src/plataforma/gitlab.rs:561:         if !rama_valida(rama) || !rama_valida(base) {
crates/tc2/src/plataforma/gitlab.rs:562:             return uso(ABRIR_PR, "rama-invalida", &t::RAMA_NO_VALIDA);
crates/tc2/src/plataforma/gitlab.rs:563:         }
crates/tc2/src/plataforma/gitlab.rs:564:         if titulo.trim().is_empty() || titulo.chars().any(char::is_control) {
crates/tc2/src/plataforma/gitlab.rs:565:             return uso(ABRIR_PR, "titulo-invalido", &t::TITULO_NO_VALIDO);
crates/tc2/src/plataforma/gitlab.rs:566:         }
crates/tc2/src/plataforma/gitlab.rs:567:         match self.mr_abierto(rama, base) {
crates/tc2/src/plataforma/gitlab.rs:568:             Ok(Some(n)) => {
crates/tc2/src/plataforma/gitlab.rs:569:                 return Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string());
crates/tc2/src/plataforma/gitlab.rs:570:             }
```

##### merge-pr: entrada y lineas de resultado (solo cambia la condicion leida de la API y el texto del diagnostico)
```text
crates/tc2/src/plataforma/github.rs:689:         if p.merged {
crates/tc2/src/plataforma/github.rs:690:             return Informe::nuevo(MERGE_PR, Resultado::SinCambios)
crates/tc2/src/plataforma/github.rs:691:                 .con("pr", pr.to_string())
crates/tc2/src/plataforma/github.rs:692:                 .con("estado", "mergeado");
crates/tc2/src/plataforma/github.rs:693:         }
crates/tc2/src/plataforma/github.rs:694:         if p.state != "open" {
crates/tc2/src/plataforma/github.rs:695:             return Informe::nuevo(MERGE_PR, Resultado::Rechazada)
crates/tc2/src/plataforma/github.rs:696:                 .con("pr", pr.to_string())
crates/tc2/src/plataforma/github.rs:697:                 .con("motivo", "pr-cerrado")
crates/tc2/src/plataforma/github.rs:698:                 .diagnostico(&t::PR_CERRADO);
```
```text
crates/tc2/src/plataforma/gitlab.rs:792:         if m.state == "merged" {
crates/tc2/src/plataforma/gitlab.rs:793:             return Informe::nuevo(MERGE_PR, Resultado::SinCambios)
crates/tc2/src/plataforma/gitlab.rs:794:                 .con("pr", pr.to_string())
crates/tc2/src/plataforma/gitlab.rs:795:                 .con("estado", "mergeado");
crates/tc2/src/plataforma/gitlab.rs:796:         }
crates/tc2/src/plataforma/gitlab.rs:797:         if m.state != "opened" {
crates/tc2/src/plataforma/gitlab.rs:798:             return Informe::nuevo(MERGE_PR, Resultado::Rechazada)
crates/tc2/src/plataforma/gitlab.rs:799:                 .con("pr", pr.to_string())
crates/tc2/src/plataforma/gitlab.rs:800:                 .con("motivo", "pr-cerrado")
crates/tc2/src/plataforma/gitlab.rs:801:                 .diagnostico(&t::MR_CERRADO);
```

Ademas son identicos la entrada (`if !sha_valido(sha) { return uso(MERGE_PR, "sha-invalido", &t::SHA_ESPERADO_COMPLETO); }` y `exigir_persona(MERGE_PR)`: github.rs:674-679 = gitlab.rs:777-782) y el `!r.exito()` que inserta `pr=` delante (github.rs:713-718 ~ gitlab.rs:836-841, solo cambia el permiso). Para `estado-puertas` ya existen `puertas::mergeado` y `puertas::pr_cerrado` (puertas.rs:121-133); `merge-pr` no tiene equivalente y lo construye dos veces.

##### Las copias ya divergen: carrera de `abrir-pr` sin PR que releer
```text
crates/tc2/src/plataforma/github.rs:460:         if r.estado == 422 && r.cuerpo.contains("already exists") {
crates/tc2/src/plataforma/github.rs:461:             // Otro proceso lo abrio entre la lectura y la escritura.
crates/tc2/src/plataforma/github.rs:462:             return match self.pr_abierto(rama, base) {
crates/tc2/src/plataforma/github.rs:463:                 Ok(Some(n)) => {
crates/tc2/src/plataforma/github.rs:464:                     Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string())
crates/tc2/src/plataforma/github.rs:465:                 }
crates/tc2/src/plataforma/github.rs:466:                 Ok(None) => self
crates/tc2/src/plataforma/github.rs:467:                     .con
crates/tc2/src/plataforma/github.rs:468:                     .traducir(ABRIR_PR, Rol::Agente, "pull_requests:write", &r),
crates/tc2/src/plataforma/github.rs:469:                 Err(i) => i,
crates/tc2/src/plataforma/github.rs:470:             };
crates/tc2/src/plataforma/github.rs:471:         }
```
```text
crates/tc2/src/plataforma/gitlab.rs:599:         if r.estado == 409 {
crates/tc2/src/plataforma/gitlab.rs:600:             // Otro proceso lo abrio entre la lectura y la escritura.
crates/tc2/src/plataforma/gitlab.rs:601:             return match self.mr_abierto(rama, base) {
crates/tc2/src/plataforma/gitlab.rs:602:                 Ok(Some(n)) => {
crates/tc2/src/plataforma/gitlab.rs:603:                     Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string())
crates/tc2/src/plataforma/gitlab.rs:604:                 }
crates/tc2/src/plataforma/gitlab.rs:605:                 Ok(None) => Informe::nuevo(ABRIR_PR, Resultado::Conflicto)
crates/tc2/src/plataforma/gitlab.rs:606:                     .con("motivo", "mr-existente")
crates/tc2/src/plataforma/gitlab.rs:607:                     .diagnostico(
crates/tc2/src/plataforma/gitlab.rs:608:                         Mensaje::nuevo(&t::MR_EXISTENTE).con("api", errores::mensaje_api(&r)),
crates/tc2/src/plataforma/gitlab.rs:609:                     ),
crates/tc2/src/plataforma/gitlab.rs:610:                 Err(i) => i,
crates/tc2/src/plataforma/gitlab.rs:611:             };
```

Entrada que lo muestra: el `POST` de crear responde «ya existe» (GitHub: 422 con `already exists` en el cuerpo; GitLab: 409) y la relectura de abiertos sigue vacia (lectura retrasada). GitHub cae en `traducir` con 422, que en errores.rs:177 es `422 => validacion(op, nombre, &cita)` → `abrir-pr error motivo=validacion` (Resultado::Error, codigo 1). GitLab da `Resultado::Conflicto` `motivo=mr-existente` (codigo 6; lo fija el test plataforma_gitlab.rs:538-541: `assert_eq!(i.linea(), "abrir-pr conflicto motivo=mr-existente"); assert_eq!(i.resultado.codigo(), 6);`). Ningun test cubre el caso equivalente en GitHub.

##### Arreglo sugerido (no se aplica en TASK-138)

Funciones comunes en `conexion.rs`/`puertas.rs` para la entrada de `abrir-pr` y `merge-pr`, la carrera de «ya existe» y las lineas `merge-pr sin-cambios|rechazada motivo=pr-cerrado`; cada adaptador solo aporta sus lecturas. Decidir una sola linea para la carrera (RR-15).

<a id="a1-01"></a>

#### A1-01 · D1 · IMPORTANTE · pendiente [A1]

_Lógica neutral de `leer_puertas_de` (validar rama y rol, leer lo común, repartir por rama) escrita en los dos adaptadores_

- id: A1-01
- regla: D1
- ambito: codigo
- ubicacion: crates/tc2/src/plataforma/github.rs:742 ~ crates/tc2/src/plataforma/gitlab.rs:864
- modulo: plataforma (adaptadores GitHub y GitLab)
- severidad: IMPORTANTE
- motivo_severidad: D1 real (regla 4D: Duplicacion real es IMPORTANTE como minimo). Es logica que no depende de la API del adaptador: validar rama y rol, leer lo comun una vez si alguna rama vale, y repartir por rama. No tapa fallo de datos ni de seguridad (no CRITICO). Las copias no divergen en comportamiento (sin marca `divergente`).
- estado: confirmado
- grupo_seguimiento: A1 — Plataforma e identidad
- evidencia: fragmentos literales con fichero:linea en la seccion «Evidencia» de abajo (contra d82ea9d)

##### Que se repite

`leer_puertas_de` (lote de `leer-puertas`, R1-06 de TASK-035) y `validar_puertas` estan escritas dos veces, una por adaptador. Lo unico que cambia es el tipo de lo comun (`Option<Vec<String>>` de rulesets en GitHub, `PuertasComunes` en GitLab) y el brazo `Some(Ok(None))` del plan de GitHub. F1-13 (decision 1, D1a) impone un trait con dos adaptadores y su lista cerrada de endpoints, no que el reparto en lote ni la validacion de entrada se copien: el propio modulo ya comparte lo neutral en `conexion.rs` y `puertas.rs` ("Los dos adaptadores comparten el transporte ([`conexion`]) y la evaluacion ([`puertas`])", mod.rs:7-8). Ademas, en las dos copias el brazo `None => Err(rol_no_valido(...))` es inalcanzable: `comun` solo es `None` si todas las `validas` son `Err`, y entonces `v?` ya devolvio antes.

##### Evidencia

Fragmento 1 (GitHub, 10 lineas):
```text
crates/tc2/src/plataforma/github.rs:750:         let validas: Vec<Paso<()>> = ramas.iter().map(|r| validar_puertas(rol, r)).collect();
crates/tc2/src/plataforma/github.rs:751:         let comun = if validas.iter().any(Result::is_ok) {
crates/tc2/src/plataforma/github.rs:752:             Some(self.rulesets(rol))
crates/tc2/src/plataforma/github.rs:753:         } else {
crates/tc2/src/plataforma/github.rs:754:             None
crates/tc2/src/plataforma/github.rs:755:         };
crates/tc2/src/plataforma/github.rs:756:         ramas
crates/tc2/src/plataforma/github.rs:757:             .iter()
crates/tc2/src/plataforma/github.rs:758:             .zip(validas)
crates/tc2/src/plataforma/github.rs:759:             .map(|(rama, v)| {
```

Fragmento 2 (GitLab, 10 lineas):
```text
crates/tc2/src/plataforma/gitlab.rs:872:         let validas: Vec<Paso<()>> = ramas.iter().map(|r| validar_puertas(rol, r)).collect();
crates/tc2/src/plataforma/gitlab.rs:873:         let comun = if validas.iter().any(Result::is_ok) {
crates/tc2/src/plataforma/gitlab.rs:874:             Some(self.puertas_comunes(rol))
crates/tc2/src/plataforma/gitlab.rs:875:         } else {
crates/tc2/src/plataforma/gitlab.rs:876:             None
crates/tc2/src/plataforma/gitlab.rs:877:         };
crates/tc2/src/plataforma/gitlab.rs:878:         ramas
crates/tc2/src/plataforma/gitlab.rs:879:             .iter()
crates/tc2/src/plataforma/gitlab.rs:880:             .zip(validas)
crates/tc2/src/plataforma/gitlab.rs:881:             .map(|(rama, v)| {
```

Diff normalizado: identicos salvo `self.rulesets(rol)` frente a `self.puertas_comunes(rol)`.

Continuacion del mismo patron (brazo comun e inalcanzable):
```text
crates/tc2/src/plataforma/github.rs:760:                 v?;
crates/tc2/src/plataforma/github.rs:761:                 match comun.as_ref() {
crates/tc2/src/plataforma/github.rs:762:                     Some(Ok(Some(rs))) => self.puertas_de_rama(rol, rama, rs, *rama == checks_de),
crates/tc2/src/plataforma/github.rs:763:                     Some(Ok(None)) => Ok(sin_reglas_por_plan()),
crates/tc2/src/plataforma/github.rs:764:                     Some(Err(i)) => Err(i.clone()),
crates/tc2/src/plataforma/github.rs:765:                     None => Err(rol_no_valido(LEER_PUERTAS, rol)),
crates/tc2/src/plataforma/github.rs:766:                 }
crates/tc2/src/plataforma/github.rs:767:             })
crates/tc2/src/plataforma/github.rs:768:             .collect()
```
```text
crates/tc2/src/plataforma/gitlab.rs:882:                 v?;
crates/tc2/src/plataforma/gitlab.rs:883:                 match comun.as_ref() {
crates/tc2/src/plataforma/gitlab.rs:884:                     Some(Ok(c)) => Ok(self.puertas_de_rama(rol, rama, c, *rama == checks_de)),
crates/tc2/src/plataforma/gitlab.rs:885:                     Some(Err(i)) => Err(i.clone()),
crates/tc2/src/plataforma/gitlab.rs:886:                     None => Err(rol_no_valido(LEER_PUERTAS, rol)),
crates/tc2/src/plataforma/gitlab.rs:887:                 }
crates/tc2/src/plataforma/gitlab.rs:888:             })
crates/tc2/src/plataforma/gitlab.rs:889:             .collect()
```

`validar_puertas`, texto identico en los dos ficheros (9 lineas cada una; se cita como parte del mismo bloque duplicado):
```text
crates/tc2/src/plataforma/github.rs:940: fn validar_puertas(rol: Rol, rama: &str) -> Paso<()> {
crates/tc2/src/plataforma/github.rs:941:     if !rama_valida(rama) {
crates/tc2/src/plataforma/github.rs:942:         return Err(uso(LEER_PUERTAS, "rama-invalida", &t::RAMA_NO_VALIDA));
crates/tc2/src/plataforma/github.rs:943:     }
crates/tc2/src/plataforma/github.rs:944:     if rol == Rol::Revisor {
crates/tc2/src/plataforma/github.rs:945:         return Err(rol_no_valido(LEER_PUERTAS, rol));
crates/tc2/src/plataforma/github.rs:946:     }
crates/tc2/src/plataforma/github.rs:947:     Ok(())
crates/tc2/src/plataforma/github.rs:948: }
```
```text
crates/tc2/src/plataforma/gitlab.rs:1247: fn validar_puertas(rol: Rol, rama: &str) -> Paso<()> {
crates/tc2/src/plataforma/gitlab.rs:1248:     if !rama_valida(rama) {
crates/tc2/src/plataforma/gitlab.rs:1249:         return Err(uso(LEER_PUERTAS, "rama-invalida", &t::RAMA_NO_VALIDA));
crates/tc2/src/plataforma/gitlab.rs:1250:     }
crates/tc2/src/plataforma/gitlab.rs:1251:     if rol == Rol::Revisor {
crates/tc2/src/plataforma/gitlab.rs:1252:         return Err(rol_no_valido(LEER_PUERTAS, rol));
crates/tc2/src/plataforma/gitlab.rs:1253:     }
crates/tc2/src/plataforma/gitlab.rs:1254:     Ok(())
crates/tc2/src/plataforma/gitlab.rs:1255: }
```

Pase 0: `fn casi iguales` Jaccard 0.81 (`github.rs:742-769` ~ `gitlab.rs:864-890`) y `nombre repetido fn validar_puertas` similitud 1.00.

##### Arreglo sugerido (no se aplica en TASK-138)

Una funcion generica en `plataforma/mod.rs` (`leer_en_lote(ramas, checks_de, validar, leer_comun, por_rama)`) y una sola `validar_puertas`; cada adaptador solo aporta `leer_comun` y `por_rama`.

<a id="a1-03"></a>

#### A1-03 · D1 · IMPORTANTE · pendiente [A1]

_Reglas neutrales de `prs-abiertos` (rama inválida se salta, sha no hex es `respuesta-invalida`) implementadas dos veces_

- id: A1-03
- regla: D1
- ambito: codigo
- ubicacion: crates/tc2/src/plataforma/github.rs:412 ~ crates/tc2/src/plataforma/gitlab.rs:538
- modulo: plataforma (adaptadores GitHub y GitLab: `prs-abiertos`)
- severidad: IMPORTANTE
- motivo_severidad: D1 real. Las reglas de `prs-abiertos` que fija la adenda de TASK-032 a F1-13 son neutrales («Una rama que no pasa `rama_valida` se salta [...]; un sha que no es de 40 hex es `respuesta-invalida`», docs/adr/F1-13-adaptador-de-plataforma.md:7) y estan implementadas dos veces; solo la lectura del campo (`head.repo.id` frente a `source_project_id`, `draft` frente a `draft || work_in_progress`) depende de la API. No divergen en la linea (solo en el texto del diagnostico: `RAMA_BASE_O_SHA` frente a `RAMA_DESTINO_O_SHA`), asi que sin marca `divergente`. La copia de GitLab ademas no tiene test de estas reglas (ver A1-09).
- estado: confirmado
- grupo_seguimiento: A1 — Plataforma e identidad
- evidencia: fragmentos literales con fichero:linea en la seccion «Evidencia» de abajo (contra d82ea9d)

##### Evidencia

Fragmento 1 (GitHub):
```text
crates/tc2/src/plataforma/github.rs:416:             if !propio || !rama_valida(&p.head.nombre) {
crates/tc2/src/plataforma/github.rs:417:                 continue;
crates/tc2/src/plataforma/github.rs:418:             }
crates/tc2/src/plataforma/github.rs:419:             if !sha_valido(&p.head.sha) {
crates/tc2/src/plataforma/github.rs:420:                 return Err(self.con.invalida_por(PRS_ABIERTOS, &t::RAMA_BASE_O_SHA));
crates/tc2/src/plataforma/github.rs:421:             }
crates/tc2/src/plataforma/github.rs:422:             v.push(PrAbierto {
crates/tc2/src/plataforma/github.rs:423:                 numero: p.number,
crates/tc2/src/plataforma/github.rs:424:                 rama: p.head.nombre,
crates/tc2/src/plataforma/github.rs:425:                 sha: p.head.sha.to_ascii_lowercase(),
crates/tc2/src/plataforma/github.rs:426:                 borrador: p.draft,
crates/tc2/src/plataforma/github.rs:427:             });
crates/tc2/src/plataforma/github.rs:428:         }
crates/tc2/src/plataforma/github.rs:429:         v.sort_by_key(|p| p.numero);
crates/tc2/src/plataforma/github.rs:430:         Ok(v)
```

Fragmento 2 (GitLab):
```text
crates/tc2/src/plataforma/gitlab.rs:542:             if m.source_project_id != m.target_project_id || !rama_valida(&m.source_branch) {
crates/tc2/src/plataforma/gitlab.rs:543:                 continue;
crates/tc2/src/plataforma/gitlab.rs:544:             }
crates/tc2/src/plataforma/gitlab.rs:545:             let sha = m.sha.unwrap_or_default();
crates/tc2/src/plataforma/gitlab.rs:546:             if !sha_valido(&sha) {
crates/tc2/src/plataforma/gitlab.rs:547:                 return Err(self.con.invalida_por(PRS_ABIERTOS, &t::RAMA_DESTINO_O_SHA));
crates/tc2/src/plataforma/gitlab.rs:548:             }
crates/tc2/src/plataforma/gitlab.rs:549:             v.push(PrAbierto {
crates/tc2/src/plataforma/gitlab.rs:550:                 numero: m.iid,
crates/tc2/src/plataforma/gitlab.rs:551:                 rama: m.source_branch,
crates/tc2/src/plataforma/gitlab.rs:552:                 sha: sha.to_ascii_lowercase(),
crates/tc2/src/plataforma/gitlab.rs:553:                 borrador: m.draft || m.work_in_progress,
crates/tc2/src/plataforma/gitlab.rs:554:             });
crates/tc2/src/plataforma/gitlab.rs:555:         }
crates/tc2/src/plataforma/gitlab.rs:556:         v.sort_by_key(|p| p.numero);
crates/tc2/src/plataforma/gitlab.rs:557:         Ok(v)
```

Diff normalizado: el filtro de propio (`h.id == b.id` / `source_project_id != target_project_id`), `p.head.sha` frente a `m.sha.unwrap_or_default()`, el texto `RAMA_BASE_O_SHA`/`RAMA_DESTINO_O_SHA` y `p.draft` frente a `m.draft || m.work_in_progress`; el resto (saltar rama no valida, fallar con sha no valido, minusculas, push y orden por numero) es igual. Pase 0: fragmento repetido de 178 tokens normalizados (`github.rs:426-446` ~ `gitlab.rs:553-573`).

##### Arreglo sugerido (no se aplica en TASK-138)

Que cada adaptador traduzca su lista a `(numero, rama, sha, borrador, propio)` y una sola funcion neutral aplique las reglas de la adenda.

<a id="a1-04"></a>

#### A1-04 · D1 · IMPORTANTE · pendiente [A1]

_Bucle de la tabla de errores HTTP y servido de fixtures copiado entre las baterías de GitHub y GitLab_

- id: A1-04
- regla: D1
- ambito: test
- ubicacion: crates/tc2/tests/plataforma.rs:468 ~ crates/tc2/tests/plataforma_gitlab.rs:476
- modulo: plataforma (tests de integracion de los adaptadores)
- severidad: IMPORTANTE
- motivo_severidad: D1 real en codigo de test (≥10 lineas, ≥50 tokens). El bucle que comprueba la tabla de errores HTTP (linea, codigo, arreglo, sin controles y sin token) y el que sirve un fixture al mock son los mismos en las dos baterias; solo cambia el adaptador y la ruta de /user. No es logica de la API: es el arnes. Sin divergencia de comportamiento.
- estado: confirmado
- grupo_seguimiento: A1 — Plataforma e identidad
- evidencia: fragmentos literales con fichero:linea en la seccion «Evidencia» de abajo (contra d82ea9d)

##### Evidencia

##### Bucle de la tabla de errores (10 lineas de cada lado)
```text
crates/tc2/tests/plataforma.rs:468:     for (respuesta, linea, codigo, arreglo) in casos {
crates/tc2/tests/plataforma.rs:469:         let mock = Mock::nuevo();
crates/tc2/tests/plataforma.rs:470:         mock.responder("GET", "/user", respuesta);
crates/tc2/tests/plataforma.rs:471:         let gh = adaptador(&mock, &[Rol::Agente]);
crates/tc2/tests/plataforma.rs:472:         let i = gh.identidad(Rol::Agente).unwrap_err();
crates/tc2/tests/plataforma.rs:473:         let s = i.salida(false);
crates/tc2/tests/plataforma.rs:474:         assert_eq!((s.stdout.trim_end(), s.codigo), (linea, codigo));
crates/tc2/tests/plataforma.rs:475:         assert!(s.stderr.contains(arreglo), "{linea}: {}", s.stderr);
crates/tc2/tests/plataforma.rs:476:         assert!(!s.stderr.contains('\u{7}'), "sin controles de la API");
crates/tc2/tests/plataforma.rs:477:         for texto in [
```
```text
crates/tc2/tests/plataforma_gitlab.rs:476:     for (respuesta, linea, codigo, arreglo) in casos {
crates/tc2/tests/plataforma_gitlab.rs:477:         let mock = Mock::nuevo();
crates/tc2/tests/plataforma_gitlab.rs:478:         mock.responder("GET", "/api/v4/user", respuesta);
crates/tc2/tests/plataforma_gitlab.rs:479:         let gl = gitlab(&mock, &[Rol::Agente]);
crates/tc2/tests/plataforma_gitlab.rs:480:         let i = gl.identidad(Rol::Agente).unwrap_err();
crates/tc2/tests/plataforma_gitlab.rs:481:         let s = i.salida(false);
crates/tc2/tests/plataforma_gitlab.rs:482:         assert_eq!((s.stdout.trim_end(), s.codigo), (linea, codigo));
crates/tc2/tests/plataforma_gitlab.rs:483:         assert!(s.stderr.contains(arreglo), "{linea}: {}", s.stderr);
crates/tc2/tests/plataforma_gitlab.rs:484:         assert!(!s.stderr.contains('\u{7}'), "sin controles de la API");
crates/tc2/tests/plataforma_gitlab.rs:485:         for texto in [
```

Diff normalizado: `"/user"` frente a `"/api/v4/user"`, `adaptador(&mock, &[Rol::Agente])` frente a `gitlab(&mock, &[Rol::Agente])`, y `gh`/`gl`. Sigue igual hasta plataforma.rs:485 / plataforma_gitlab.rs:493 (el `for texto in [...]` que busca el token). Pase 0: fragmento repetido de 265 tokens normalizados (`plataforma.rs:444-490` ~ `plataforma_gitlab.rs:452-498`), el mayor de A1.

##### Servir un fixture al mock (bucle dentro del test en plataforma.rs; `fn servir` en plataforma_gitlab.rs)
```text
crates/tc2/tests/plataforma.rs:115:         let mock = Mock::nuevo();
crates/tc2/tests/plataforma.rs:116:         for (ruta, r) in f["respuestas"].as_object().unwrap() {
crates/tc2/tests/plataforma.rs:117:             mock.responder(
crates/tc2/tests/plataforma.rs:118:                 "GET",
crates/tc2/tests/plataforma.rs:119:                 ruta,
crates/tc2/tests/plataforma.rs:120:                 Simulada::json(
crates/tc2/tests/plataforma.rs:121:                     r["estado"].as_u64().unwrap() as u16,
crates/tc2/tests/plataforma.rs:122:                     &r["cuerpo"].to_string(),
crates/tc2/tests/plataforma.rs:123:                 ),
crates/tc2/tests/plataforma.rs:124:             );
crates/tc2/tests/plataforma.rs:125:         }
```
```text
crates/tc2/tests/plataforma_gitlab.rs:83:     let mock = Mock::nuevo();
crates/tc2/tests/plataforma_gitlab.rs:84:     for (ruta, r) in f["respuestas"].as_object().unwrap() {
crates/tc2/tests/plataforma_gitlab.rs:85:         mock.responder(
crates/tc2/tests/plataforma_gitlab.rs:86:             "GET",
crates/tc2/tests/plataforma_gitlab.rs:87:             ruta,
crates/tc2/tests/plataforma_gitlab.rs:88:             Simulada::json(
crates/tc2/tests/plataforma_gitlab.rs:89:                 r["estado"].as_u64().unwrap() as u16,
crates/tc2/tests/plataforma_gitlab.rs:90:                 &r["cuerpo"].to_string(),
crates/tc2/tests/plataforma_gitlab.rs:91:             ),
crates/tc2/tests/plataforma_gitlab.rs:92:         );
crates/tc2/tests/plataforma_gitlab.rs:93:     }
```

Diff normalizado: identico; plataforma.rs lo escribe dentro del test y plataforma_gitlab.rs lo saca a `fn servir`, que plataforma.rs no reutiliza.

##### Arreglo sugerido (no se aplica en TASK-138)

Llevar `servir(fixture)` y una `comprobar_tabla_de_errores(casos, fabrica_de_adaptador)` a `tests/comun/` (dueno A8) y usarlas desde las dos baterias.

<a id="a1-05"></a>

#### A1-05 · D1 · IMPORTANTE · pendiente [A1]

_`adaptador(mock, roles)` reescribe `con_personas(mock, roles, &[PERSONA])` en el mismo fichero_

- id: A1-05
- regla: D1
- ambito: test
- ubicacion: crates/tc2/tests/plataforma.rs:32 ~ crates/tc2/tests/plataforma.rs:568
- modulo: plataforma (tests de integracion del adaptador GitHub)
- severidad: IMPORTANTE
- motivo_severidad: D1 real (13 lineas cada una, ≥50 tokens) dentro del mismo fichero: `adaptador(mock, roles)` es `con_personas(mock, roles, &[PERSONA])`. Sin divergencia.
- estado: confirmado
- grupo_seguimiento: A1 — Plataforma e identidad
- evidencia: fragmentos literales con fichero:linea en la seccion «Evidencia» de abajo (contra d82ea9d)

##### Evidencia
```text
crates/tc2/tests/plataforma.rs:32: fn adaptador(mock: &Mock, roles: &[Rol]) -> GitHub {
crates/tc2/tests/plataforma.rs:33:     let mut c = CredencialesFijas::nuevas();
crates/tc2/tests/plataforma.rs:34:     for r in roles {
crates/tc2/tests/plataforma.rs:35:         c = c.con(*r, Secreto::nuevo(TOKEN));
crates/tc2/tests/plataforma.rs:36:     }
crates/tc2/tests/plataforma.rs:37:     GitHub::nuevo(
crates/tc2/tests/plataforma.rs:38:         Base::nueva(&mock.base()).unwrap(),
crates/tc2/tests/plataforma.rs:39:         "o/r",
crates/tc2/tests/plataforma.rs:40:         Box::new(c),
crates/tc2/tests/plataforma.rs:41:         Personas::nuevas([PERSONA]),
crates/tc2/tests/plataforma.rs:42:     )
crates/tc2/tests/plataforma.rs:43:     .unwrap()
crates/tc2/tests/plataforma.rs:44: }
```
```text
crates/tc2/tests/plataforma.rs:568: fn con_personas(mock: &Mock, roles: &[Rol], personas: &[u64]) -> GitHub {
crates/tc2/tests/plataforma.rs:569:     let mut c = CredencialesFijas::nuevas();
crates/tc2/tests/plataforma.rs:570:     for r in roles {
crates/tc2/tests/plataforma.rs:571:         c = c.con(*r, Secreto::nuevo(TOKEN));
crates/tc2/tests/plataforma.rs:572:     }
crates/tc2/tests/plataforma.rs:573:     GitHub::nuevo(
crates/tc2/tests/plataforma.rs:574:         Base::nueva(&mock.base()).unwrap(),
crates/tc2/tests/plataforma.rs:575:         "o/r",
crates/tc2/tests/plataforma.rs:576:         Box::new(c),
crates/tc2/tests/plataforma.rs:577:         Personas::nuevas(personas.iter().copied()),
crates/tc2/tests/plataforma.rs:578:     )
crates/tc2/tests/plataforma.rs:579:     .unwrap()
crates/tc2/tests/plataforma.rs:580: }
```

Diff normalizado: la firma (`personas: &[u64]`) y `Personas::nuevas([PERSONA])` frente a `Personas::nuevas(personas.iter().copied())`; el resto, identico.

##### Arreglo sugerido (no se aplica en TASK-138)

`fn adaptador(mock, roles) -> GitHub { con_personas(mock, roles, &[PERSONA]) }`.

<a id="a1-06"></a>

#### A1-06 · D1 · IMPORTANTE · pendiente [A1]

_Cliente HTTP del arnés E2E copiado entre los E2E de GitHub y GitLab_

- id: A1-06
- regla: D1
- ambito: test (E2E)
- ubicacion: crates/tc2/tests/plataforma_e2e.rs:74 ~ crates/tc2/tests/plataforma_gitlab_e2e.rs:83
- modulo: plataforma (arneses E2E contra GitHub y GitLab reales)
- severidad: IMPORTANTE
- motivo_severidad: D1 real (funcion identica de 15 lineas y lectura de respuesta casi identica). Es el cliente HTTP del arnes, no la API del adaptador. Los dos E2E son ignorados por defecto (no verificables por mutacion), lo que no cambia que el codigo este duplicado. Sin divergencia que afecte a lo que comprueban.
- estado: confirmado
- grupo_seguimiento: A1 — Plataforma e identidad
- evidencia: fragmentos literales con fichero:linea en la seccion «Evidencia» de abajo (contra d82ea9d)

##### Evidencia

##### Arnes::nuevo: texto identico (pase 0: fn igual, hash 640965e537, 15 lineas, 76 tokens)
```text
crates/tc2/tests/plataforma_e2e.rs:75:         let agente: ureq::Agent = ureq::Agent::config_builder()
crates/tc2/tests/plataforma_e2e.rs:76:             .http_status_as_error(false)
crates/tc2/tests/plataforma_e2e.rs:77:             .tls_config(
crates/tc2/tests/plataforma_e2e.rs:78:                 ureq::tls::TlsConfig::builder()
crates/tc2/tests/plataforma_e2e.rs:79:                     .root_certs(ureq::tls::RootCerts::PlatformVerifier)
crates/tc2/tests/plataforma_e2e.rs:80:                     .build(),
crates/tc2/tests/plataforma_e2e.rs:81:             )
crates/tc2/tests/plataforma_e2e.rs:82:             .build()
crates/tc2/tests/plataforma_e2e.rs:83:             .into();
crates/tc2/tests/plataforma_e2e.rs:84:         Self {
```
```text
crates/tc2/tests/plataforma_gitlab_e2e.rs:84:         let agente: ureq::Agent = ureq::Agent::config_builder()
crates/tc2/tests/plataforma_gitlab_e2e.rs:85:             .http_status_as_error(false)
crates/tc2/tests/plataforma_gitlab_e2e.rs:86:             .tls_config(
crates/tc2/tests/plataforma_gitlab_e2e.rs:87:                 ureq::tls::TlsConfig::builder()
crates/tc2/tests/plataforma_gitlab_e2e.rs:88:                     .root_certs(ureq::tls::RootCerts::PlatformVerifier)
crates/tc2/tests/plataforma_gitlab_e2e.rs:89:                     .build(),
crates/tc2/tests/plataforma_gitlab_e2e.rs:90:             )
crates/tc2/tests/plataforma_gitlab_e2e.rs:91:             .build()
crates/tc2/tests/plataforma_gitlab_e2e.rs:92:             .into();
crates/tc2/tests/plataforma_gitlab_e2e.rs:93:         Self {
```

##### Cabeceras y lectura de la respuesta del arnes
```text
crates/tc2/tests/plataforma_e2e.rs:116:         let r = match (metodo, cuerpo) {
crates/tc2/tests/plataforma_e2e.rs:117:             ("GET", _) => cab(self.agente.get(url), &auth).call(),
crates/tc2/tests/plataforma_e2e.rs:118:             ("DELETE", _) => cab(self.agente.delete(url), &auth).call(),
crates/tc2/tests/plataforma_e2e.rs:119:             ("POST", Some(c)) => cab(self.agente.post(url), &auth).send(c.to_string()),
crates/tc2/tests/plataforma_e2e.rs:120:             ("PUT", Some(c)) => cab(self.agente.put(url), &auth).send(c.to_string()),
crates/tc2/tests/plataforma_e2e.rs:121:             ("PATCH", Some(c)) => cab(self.agente.patch(url), &auth).send(c.to_string()),
crates/tc2/tests/plataforma_e2e.rs:122:             _ => return None,
crates/tc2/tests/plataforma_e2e.rs:123:         };
crates/tc2/tests/plataforma_e2e.rs:124:         let mut r = r.ok()?;
crates/tc2/tests/plataforma_e2e.rs:125:         let estado = r.status().as_u16();
crates/tc2/tests/plataforma_e2e.rs:126:         let texto = r.body_mut().read_to_string().unwrap_or_default();
crates/tc2/tests/plataforma_e2e.rs:127:         Some((estado, serde_json::from_str(&texto).unwrap_or(Value::Null)))
```
```text
crates/tc2/tests/plataforma_gitlab_e2e.rs:113:         let r = match (metodo, cuerpo) {
crates/tc2/tests/plataforma_gitlab_e2e.rs:114:             ("GET", _) => cab(self.agente.get(&url), &auth).call(),
crates/tc2/tests/plataforma_gitlab_e2e.rs:115:             ("DELETE", _) => cab(self.agente.delete(&url), &auth).call(),
crates/tc2/tests/plataforma_gitlab_e2e.rs:116:             ("POST", Some(c)) => cab(self.agente.post(&url), &auth).send(c.to_string()),
crates/tc2/tests/plataforma_gitlab_e2e.rs:117:             ("PUT", Some(c)) => cab(self.agente.put(&url), &auth).send(c.to_string()),
crates/tc2/tests/plataforma_gitlab_e2e.rs:118:             _ => return None,
crates/tc2/tests/plataforma_gitlab_e2e.rs:119:         };
crates/tc2/tests/plataforma_gitlab_e2e.rs:120:         let mut r = r.ok()?;
crates/tc2/tests/plataforma_gitlab_e2e.rs:121:         let estado = r.status().as_u16();
crates/tc2/tests/plataforma_gitlab_e2e.rs:122:         let texto = r.body_mut().read_to_string().unwrap_or_default();
crates/tc2/tests/plataforma_gitlab_e2e.rs:123:         Some((estado, serde_json::from_str(&texto).unwrap_or(Value::Null)))
```

Diff normalizado: `url` frente a `&url`, el brazo PATCH (solo en GitHub) y la cabecera Accept dentro de `cab` (plataforma_e2e.rs:110-115 frente a plataforma_gitlab_e2e.rs:107-112). Tambien identicas `fn api` (plataforma_e2e.rs:90-92 = plataforma_gitlab_e2e.rs:99-101), `fn sufijo` (220-223 = 303-306) y `fn ver` (225-229 ~ 308-312), cada una por debajo del umbral pero del mismo arnes.

##### Descartes del mismo par

Los escenarios E2E (persona_aprueba_pr_del_agente ~ persona_aprueba_mr_del_agente, review_abre_el_pr... ~ review_abre_el_mr..., Jaccard 0.86) no se cuentan como D1: son la misma bateria a proposito en cada plataforma (F2-9: «La bateria es la misma de F2-6», RR-15) y sus pasos llaman a la API propia de cada una.

##### Arreglo sugerido (no se aplica en TASK-138)

Un modulo comun de arnes E2E (junto a tests/agente_git) con nuevo, intentar_con(url, accept), api, sufijo y ver.

<a id="a1-07"></a>

#### A1-07 · D4 · IMPORTANTE · pendiente [A1]

_`rulesets` lee solo la primera página (100) sin `Link rel=next` ni fallo cerrado; su hermana `rulesets_de` sí pagina_

- id: A1-07
- regla: D4
- ambito: codigo
- ubicacion: crates/tc2/src/plataforma/github.rs:960
- modulo: plataforma (adaptador GitHub, `leer-puertas`)
- severidad: IMPORTANTE
- motivo_severidad: D4 real: la funcion dice devolver «Los nombres de los rulesets del repo» y solo lee la primera pagina (100) sin seguir `Link rel=next` ni fallar cerrado. Su hermana `rulesets_de`, a 18 lineas, lee el mismo endpoint paginando con `leer_paginas`, que falla cerrado con mas de `MAX_PAGINAS`. No es CRITICO: el efecto es un falso «falta el ruleset» en `doctor`/`leer-puertas`, no una puerta que se da por buena ni un secreto expuesto.
- estado: confirmado (por lectura; la entrada de abajo es reproducible contra el mock)
- grupo_seguimiento: A1 — Plataforma e identidad
- evidencia: fragmentos literales con fichero:linea en la seccion «Evidencia» de abajo (contra d82ea9d)

##### Evidencia

La que no pagina (usa `pedir`, una sola llamada):
```text
crates/tc2/src/plataforma/github.rs:959:     /// Los nombres de los rulesets del repo; `None` si el plan no los da.
crates/tc2/src/plataforma/github.rs:960:     fn rulesets(&self, rol: Rol) -> Paso<Option<Vec<String>>> {
crates/tc2/src/plataforma/github.rs:961:         let r = self
crates/tc2/src/plataforma/github.rs:962:             .con
crates/tc2/src/plataforma/github.rs:963:             .pedir(LEER_PUERTAS, rol, self.ep(&Endpoint::Rulesets), None)?;
crates/tc2/src/plataforma/github.rs:964:         if es_aviso_de_plan(&r) {
crates/tc2/src/plataforma/github.rs:965:             return Ok(None);
crates/tc2/src/plataforma/github.rs:966:         }
crates/tc2/src/plataforma/github.rs:967:         if !r.exito() {
crates/tc2/src/plataforma/github.rs:968:             return Err(self
crates/tc2/src/plataforma/github.rs:969:                 .con
crates/tc2/src/plataforma/github.rs:970:                 .traducir(LEER_PUERTAS, rol, PERMISO_LEER_PUERTAS, &r));
crates/tc2/src/plataforma/github.rs:971:         }
crates/tc2/src/plataforma/github.rs:972:         let rulesets: Vec<RulesetApi> = self.con.parsear(LEER_PUERTAS, &r.cuerpo)?;
crates/tc2/src/plataforma/github.rs:973:         Ok(Some(rulesets.into_iter().map(|r| r.name).collect()))
crates/tc2/src/plataforma/github.rs:974:     }
```

La que pagina el mismo endpoint (`Endpoint::Rulesets` es `{r}/rulesets?per_page=100`, github.rs:101):
```text
crates/tc2/src/plataforma/github.rs:976:     /// Los nombres de todos los rulesets del repo para `op`, paginados; el
crates/tc2/src/plataforma/github.rs:977:     /// 403 de plan es su `rechazada motivo=plan` (`errores::traducir`).
crates/tc2/src/plataforma/github.rs:978:     fn rulesets_de(&self, op: &str, rol: Rol, permiso: &'static str) -> Paso<Vec<String>> {
crates/tc2/src/plataforma/github.rs:979:         let (_, ruta) = self.ep(&Endpoint::Rulesets);
crates/tc2/src/plataforma/github.rs:980:         let rulesets: Vec<RulesetApi> = self
crates/tc2/src/plataforma/github.rs:981:             .con
crates/tc2/src/plataforma/github.rs:982:             .leer_paginas(op, rol, permiso, ruta, |c| self.con.parsear(op, c))?;
crates/tc2/src/plataforma/github.rs:983:         Ok(rulesets.into_iter().map(|r| r.name).collect())
crates/tc2/src/plataforma/github.rs:984:     }
```

Contrato de la paginacion en el transporte comun (conexion.rs:181-182): «Lista paginada: sigue `Link rel=next` en la misma base, hasta [`MAX_PAGINAS`]; con mas, falla cerrado.»

`leer_puertas` y `leer_puertas_de` usan la que no pagina (github.rs:736 `match self.rulesets(rol)? {` y github.rs:752 `Some(self.rulesets(rol))`); `doctor` llega por `leer_puertas_de` (src/doctor/plataforma.rs:260).

##### Entrada con la que no hace el trabajo

`GET /repos/o/r/rulesets?per_page=100` responde 200 con 100 rulesets que no son de tc2 y `Link: <{base}/repos/o/r/rulesets?per_page=100&page=2>; rel="next"`, y la pagina 2 trae `[{"id":101,"name":"taskcodetwo-develop"}]`. `leer_puertas(Rol::Agente, "develop")` devuelve `Ok(ReglasLeidas { rulesets: <los 100 primeros>, .. })` sin `taskcodetwo-develop` y sin error ni aviso: la pagina 2 no se pide. Con `rulesets_de` la misma entrada da los 101 nombres.

Ningun test de A1 tiene mas de una pagina de rulesets (plataforma.rs:793-825 sirve una lista de 0 o 1).

Nota del mismo tipo, menor: en GitLab `puertas_comunes` lee `approval_rules?per_page=100` con `pedir` (gitlab.rs:1276-1281), tambien sin paginar; no se abre caso aparte porque 100 reglas de aprobacion en un proyecto es poco realista.

##### Arreglo sugerido (no se aplica en TASK-138)

Que `rulesets` use `leer_paginas` (como `rulesets_de`) conservando el `None` del 403 de plan en la primera pagina, o que las dos sean una sola funcion.

<a id="a1-08"></a>

#### A1-08 · D4 · IMPORTANTE · pendiente [A1]

_Test de bases inválidas solo mira `is_err()`: quitar la regla de host que empieza por `.` o `-` de `Base::nueva` sigue en verde_

- id: A1-08
- regla: D4
- ambito: test
- ubicacion: crates/tc2/src/plataforma/http.rs:473 ~ crates/tc2/src/plataforma/http.rs:100
- ubicacion_detalle: test `bases_validas_e_invalidas` (http.rs:473-496); regla no comprobada en http.rs:100
- modulo: plataforma (envoltorio HTTP, `Base::nueva`)
- severidad: IMPORTANTE
- motivo_severidad: D4 de test si el mutante sobrevive: el test de bases invalidas solo mira `is_err()` y ninguna de sus 14 entradas (ni las 4 de plataforma.rs:329-336) tiene un host que empiece por `.` o `-`, asi que quitar esa regla de `Base::nueva` no lo pondria rojo. No es CRITICO: la base sigue exigiendo https y host con caracteres validos.
- estado: confirmado
- grupo_seguimiento: A1 — Plataforma e identidad
- evidencia: fragmentos literales con fichero:linea en la seccion «Evidencia» de abajo (contra d82ea9d)

##### Evidencia

La regla:
```text
crates/tc2/src/plataforma/http.rs:96:         let host_valido = !host.is_empty()
crates/tc2/src/plataforma/http.rs:97:             && host
crates/tc2/src/plataforma/http.rs:98:                 .bytes()
crates/tc2/src/plataforma/http.rs:99:                 .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
crates/tc2/src/plataforma/http.rs:100:             && !host.starts_with(['.', '-'])
crates/tc2/src/plataforma/http.rs:101:             && puerto.is_none_or(puerto_valido);
crates/tc2/src/plataforma/http.rs:102:         if !host_valido {
crates/tc2/src/plataforma/http.rs:103:             return Err(&t::HOST_NO_VALIDO);
crates/tc2/src/plataforma/http.rs:104:         }
```

El test (solo `is_ok`/`is_err`; ningun host empieza por `.` ni `-`):
```text
crates/tc2/src/plataforma/http.rs:478:         for b in [
crates/tc2/src/plataforma/http.rs:479:             "http://api.github.com",
crates/tc2/src/plataforma/http.rs:480:             "http://127.0.0.1:80/../x",
crates/tc2/src/plataforma/http.rs:481:             "http://127.0.0.1:80/x?y",
crates/tc2/src/plataforma/http.rs:482:             "http://127.0.0.1:80/a%2Fb",
crates/tc2/src/plataforma/http.rs:483:             "http://localhost:80",
crates/tc2/src/plataforma/http.rs:484:             "https://user:pw@host",
crates/tc2/src/plataforma/http.rs:485:             "https://",
crates/tc2/src/plataforma/http.rs:486:             "ftp://x",
crates/tc2/src/plataforma/http.rs:487:             "https://host/../x",
crates/tc2/src/plataforma/http.rs:488:             "https://host/api?x=1",
crates/tc2/src/plataforma/http.rs:489:             "https://host:0",
crates/tc2/src/plataforma/http.rs:490:             "https://host:99999",
crates/tc2/src/plataforma/http.rs:491:             "https://host:8a",
crates/tc2/src/plataforma/http.rs:492:             "http://127.0.0.1:70000",
crates/tc2/src/plataforma/http.rs:493:         ] {
crates/tc2/src/plataforma/http.rs:494:             assert!(Base::nueva(b).is_err(), "{b}");
crates/tc2/src/plataforma/http.rs:495:         }
```

`grep '"https://[-.]' crates/tc2` no da ninguna linea (ninguna base con host que empiece por `.` o `-` en todo el crate).

##### Mutante propuesto (para el agente de mutacion)

- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da
- Fichero y linea: crates/tc2/src/plataforma/http.rs:100
- Cambio: borrar la linea `            && !host.starts_with(['.', '-'])`
- Orden: `cargo test -p tc2 --lib plataforma::http::tests::bases_validas_e_invalidas` y, para descartar que otro test lo cace, `cargo test -p tc2 --test plataforma el_envoltorio_rechaza_antes_de_la_red` y la suite completa `cargo test -p tc2`.
- Esperado si es D4: las dos ordenes en verde con el mutante (`test result: ok. 1 passed`).
- Rojo de control (para probar que el test corre): con el codigo roto de verdad en una regla que si cubre, p. ej. cambiar en http.rs:105 `if ruta.contains('?') || ruta.contains('#') {` por `if false {`, `bases_validas_e_invalidas` tiene que fallar en `"https://host/api?x=1"`.
- Test que deberia ponerse rojo y no lo hace: `bases_validas_e_invalidas` (le falta un caso como `"https://.evil.example"` o `"https://-x.example"`).

##### Mutación

- Resultado: **confirmado** (el mutante sobrevive en verde y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` propio).

##### Mutante

```diff
--- a/crates/tc2/src/plataforma/http.rs
+++ b/crates/tc2/src/plataforma/http.rs
@@ -97,7 +97,6 @@ impl Base {
                 .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
-            && !host.starts_with(['.', '-'])
             && puerto.is_none_or(puerto_valido);
```

Orden 1: `cargo test -p tc2 --lib plataforma::http::tests::bases_validas_e_invalidas`

```text
running 1 test
test plataforma::http::tests::bases_validas_e_invalidas ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.00s
```

Orden 2: `cargo test -p tc2 --test plataforma el_envoltorio_rechaza_antes_de_la_red`

```text
running 1 test
test el_envoltorio_rechaza_antes_de_la_red ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.04s
```

Orden 3: `cargo test -p tc2` (suite completa con el mutante): 32 binarios con `test result: ok` y 0 tests fallidos (lib: `ok. 397 passed; 0 failed; 2 ignored`), hasta que el binario `plataforma_gitlab_e2e` (todos sus tests son E2E `#[ignore]`) no arrancó por el entorno: `exit code: 0xc0000142, STATUS_DLL_INIT_FAILED`. Los binarios posteriores a ese no se ejecutaron; ningún test se puso rojo.

##### Control (código roto de verdad en una regla que sí está cubierta)

```diff
--- a/crates/tc2/src/plataforma/http.rs
+++ b/crates/tc2/src/plataforma/http.rs
@@ -105 +105 @@
-        if ruta.contains('?') || ruta.contains('#') {
+        if false {
```

Orden: `cargo test -p tc2 --lib plataforma::http::tests::bases_validas_e_invalidas`

```text
---- plataforma::http::tests::bases_validas_e_invalidas stdout ----

thread 'plataforma::http::tests::bases_validas_e_invalidas' (35884) panicked at crates\tc2\src\plataforma\http.rs:494:13:
https://host/api?x=1

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.00s
```

Clon restaurado con `git checkout -- .` tras cada paso.

<a id="a1-09"></a>

#### A1-09 · D4 · IMPORTANTE · pendiente [A1]

_`prs_abiertos.rs` dice probar que los dos adaptadores dan la misma lista y el caso de sha inválido y rama rara solo usa GitHub_

- id: A1-09
- regla: D4
- ambito: test
- ubicacion: crates/tc2/tests/prs_abiertos.rs:205 ~ crates/tc2/src/plataforma/gitlab.rs:546
- ubicacion_detalle: test `sha_invalido_falla_y_rama_rara_se_salta` (prs_abiertos.rs:205-234); reglas no comprobadas en gitlab.rs:542 y gitlab.rs:546, y minusculas en github.rs:425 y gitlab.rs:552
- modulo: plataforma (`prs-abiertos`, adaptador GitLab)
- severidad: IMPORTANTE
- motivo_severidad: D4 de test si los mutantes sobreviven. El fichero dice probar que «Los dos adaptadores dan la misma lista (RR-15)» (prs_abiertos.rs:2-3) y la adenda de TASK-032 a F1-13 dice que esas reglas estan «Probado contra el mock (`tests/prs_abiertos.rs`)» (docs/adr/F1-13-adaptador-de-plataforma.md:7), pero el test de sha invalido y rama rara solo usa GitHub, y en la lista de GitLab todas las ramas y shas son validos. Quitar las dos reglas en GitLab no pondria rojo ningun test de A1. La normalizacion a minusculas del sha tampoco se comprueba en ninguno de los dos adaptadores (todas las entradas ya van en minusculas).
- estado: confirmado
- grupo_seguimiento: A1 — Plataforma e identidad
- evidencia: fragmentos literales con fichero:linea en la seccion «Evidencia» de abajo (contra d82ea9d)

##### Evidencia

Las reglas en GitLab:
```text
crates/tc2/src/plataforma/gitlab.rs:542:             if m.source_project_id != m.target_project_id || !rama_valida(&m.source_branch) {
crates/tc2/src/plataforma/gitlab.rs:543:                 continue;
crates/tc2/src/plataforma/gitlab.rs:544:             }
crates/tc2/src/plataforma/gitlab.rs:545:             let sha = m.sha.unwrap_or_default();
crates/tc2/src/plataforma/gitlab.rs:546:             if !sha_valido(&sha) {
crates/tc2/src/plataforma/gitlab.rs:547:                 return Err(self.con.invalida_por(PRS_ABIERTOS, &t::RAMA_DESTINO_O_SHA));
crates/tc2/src/plataforma/gitlab.rs:548:             }
crates/tc2/src/plataforma/gitlab.rs:549:             v.push(PrAbierto {
crates/tc2/src/plataforma/gitlab.rs:550:                 numero: m.iid,
crates/tc2/src/plataforma/gitlab.rs:551:                 rama: m.source_branch,
crates/tc2/src/plataforma/gitlab.rs:552:                 sha: sha.to_ascii_lowercase(),
crates/tc2/src/plataforma/gitlab.rs:553:                 borrador: m.draft || m.work_in_progress,
```

El test que las nombra solo monta GitHub:
```text
crates/tc2/tests/prs_abiertos.rs:205: #[test]
crates/tc2/tests/prs_abiertos.rs:206: fn sha_invalido_falla_y_rama_rara_se_salta() {
crates/tc2/tests/prs_abiertos.rs:207:     let _serie = serie();
crates/tc2/tests/prs_abiertos.rs:208:     let m = Mock::nuevo();
crates/tc2/tests/prs_abiertos.rs:209:     m.responder(
crates/tc2/tests/prs_abiertos.rs:210:         "GET",
crates/tc2/tests/prs_abiertos.rs:211:         GH,
crates/tc2/tests/prs_abiertos.rs:212:         Simulada::json(
crates/tc2/tests/prs_abiertos.rs:213:             200,
crates/tc2/tests/prs_abiertos.rs:214:             &format!("[{}]", pr_gh(1, "rama con espacio", "x", false, false)),
crates/tc2/tests/prs_abiertos.rs:215:         ),
crates/tc2/tests/prs_abiertos.rs:216:     );
crates/tc2/tests/prs_abiertos.rs:217:     let p = github(&m.base(), &[Rol::Agente]);
crates/tc2/tests/prs_abiertos.rs:218:     assert_eq!(p.prs_abiertos(Rol::Agente), Ok(Vec::new()));
```

(el resto, prs_abiertos.rs:219-233, otra vez `GH` y `github(...)`). En `misma_lista_en_github_y_gitlab_sin_forks_y_paginada` los MR de GitLab son `mr_gl(5, "feature/task-002-y", SHA_A, ..)`, `mr_gl(9, "feature/task-003-x", SHA_C, ..)` y `mr_gl(3, "feature/task-003-x", SHA_B, ..)` (prs_abiertos.rs:113-116): ramas validas y shas de 40 hex en minusculas.

##### Mutantes propuestos (para el agente de mutacion; uno a uno)

- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da
1. crates/tc2/src/plataforma/gitlab.rs:546: `if !sha_valido(&sha) {` → `if false {`
2. crates/tc2/src/plataforma/gitlab.rs:542: `if m.source_project_id != m.target_project_id || !rama_valida(&m.source_branch) {` → `if m.source_project_id != m.target_project_id {`
3. crates/tc2/src/plataforma/github.rs:425: `sha: p.head.sha.to_ascii_lowercase(),` → `sha: p.head.sha,` (y el analogo en gitlab.rs:552 `sha: sha.to_ascii_lowercase(),` → `sha,`)
- Orden: `cargo test -p tc2 --test prs_abiertos` (los tres tests del fichero) y la suite completa `cargo test -p tc2` (por si `board`/`tests/tablero*` lo cazan con el adaptador real).
- Esperado si es D4: `test result: ok. 3 passed` en `--test prs_abiertos` con cada mutante.
- Rojo de control: el mismo cambio que 1 pero en GitHub (github.rs:419 `if !sha_valido(&p.head.sha) {` → `if false {`) tiene que poner rojo `sha_invalido_falla_y_rama_rara_se_salta` (espera `prs-abiertos error motivo=respuesta-invalida`).
- Tests que deberian ponerse rojos y no lo hacen: `sha_invalido_falla_y_rama_rara_se_salta` (le falta el recorrido por GitLab, como hacen los otros dos tests con `for plataforma in ["github", "gitlab"]`) y `misma_lista_en_github_y_gitlab_sin_forks_y_paginada` (le falta un sha en mayusculas).

##### Mutación

- Resultado: **confirmado** (los tres mutantes sobreviven en verde y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` propio). Clon restaurado con `git checkout -- .` entre pasos.

##### Mutante 1

```diff
--- a/crates/tc2/src/plataforma/gitlab.rs
+++ b/crates/tc2/src/plataforma/gitlab.rs
@@ -546 +546 @@ impl Plataforma for GitLab {
-            if !sha_valido(&sha) {
+            if false {
```

Orden: `cargo test -p tc2 --test prs_abiertos`

```text
test comun::http_mock::tras_una_respuesta_completa_el_mock_no_manda_fin ... ok
test fallos_tipados ... ok
test misma_lista_en_github_y_gitlab_sin_forks_y_paginada ... ok
test sha_invalido_falla_y_rama_rara_se_salta ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.52s
```

Orden: `cargo test -p tc2 --no-fail-fast` (suite completa con este mutante, 33 min): 47 `test result: ok`, ningún `FAILED` ni `panicked` (los E2E `#[ignore]` salen como ignorados, no se ejecutan).

##### Mutante 2

```diff
--- a/crates/tc2/src/plataforma/gitlab.rs
+++ b/crates/tc2/src/plataforma/gitlab.rs
@@ -542 +542 @@
-            if m.source_project_id != m.target_project_id || !rama_valida(&m.source_branch) {
+            if m.source_project_id != m.target_project_id {
```

Orden: `cargo test -p tc2 --test prs_abiertos`

```text
test comun::http_mock::tras_una_respuesta_completa_el_mock_no_manda_fin ... ok
test fallos_tipados ... ok
test misma_lista_en_github_y_gitlab_sin_forks_y_paginada ... ok
test sha_invalido_falla_y_rama_rara_se_salta ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.55s
```

##### Mutante 3

```diff
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -425 +425 @@
-                sha: p.head.sha.to_ascii_lowercase(),
+                sha: p.head.sha,
--- a/crates/tc2/src/plataforma/gitlab.rs
+++ b/crates/tc2/src/plataforma/gitlab.rs
@@ -552 +552 @@
-                sha: sha.to_ascii_lowercase(),
+                sha,
```

Orden: `cargo test -p tc2 --test prs_abiertos`

```text
test comun::http_mock::tras_una_respuesta_completa_el_mock_no_manda_fin ... ok
test fallos_tipados ... ok
test misma_lista_en_github_y_gitlab_sin_forks_y_paginada ... ok
test sha_invalido_falla_y_rama_rara_se_salta ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.53s
```

##### Control (código roto de verdad en GitHub, que sí está cubierto)

```diff
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -419 +419 @@
-            if !sha_valido(&p.head.sha) {
+            if false {
```

Orden: `cargo test -p tc2 --test prs_abiertos sha_invalido_falla_y_rama_rara_se_salta`

```text
test sha_invalido_falla_y_rama_rara_se_salta ... FAILED
thread 'sha_invalido_falla_y_rama_rara_se_salta' (20372) panicked at crates\tc2\tests\prs_abiertos.rs:230:5:
assertion `left == right` failed
  left: None
 right: Some("prs-abiertos error motivo=respuesta-invalida")
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.02s
```

<a id="a1-10"></a>

#### A1-10 · D4 · IMPORTANTE · pendiente [A1]

_La aserción «el revisor no lee puertas» pasa por falta de token, no por la comprobación de rol_

- id: A1-10
- regla: D4
- ambito: test
- ubicacion: crates/tc2/tests/plataforma_gitlab.rs:842 ~ crates/tc2/src/plataforma/gitlab.rs:1251
- ubicacion_detalle: asercion de `protecciones_de_gitlab`; regla no comprobada en gitlab.rs:1251-1253; en GitHub, github.rs:944-946 sin ningun test
- modulo: plataforma (`leer-puertas`, rol del revisor)
- severidad: IMPORTANTE
- motivo_severidad: D4 de test si el mutante sobrevive. La asercion que parece probar que el revisor no lee puertas solo mira `is_err()`, y el adaptador del test no tiene token de revisor: aunque se quite la comprobacion de rol, la llamada sigue fallando, por `falta=revisor` en vez de por el rol. Con un token de revisor configurado (el caso real), el revisor podria leer puertas, que el puerto reserva a agente o persona (mod.rs:465: «`leer-puertas` (agente o persona).»). En GitHub la misma regla (github.rs:944-946) no tiene ningun test.
- estado: confirmado
- grupo_seguimiento: A1 — Plataforma e identidad
- evidencia: fragmentos literales con fichero:linea en la seccion «Evidencia» de abajo (contra d82ea9d)

##### Evidencia

La regla (GitLab):
```text
crates/tc2/src/plataforma/gitlab.rs:1247: fn validar_puertas(rol: Rol, rama: &str) -> Paso<()> {
crates/tc2/src/plataforma/gitlab.rs:1248:     if !rama_valida(rama) {
crates/tc2/src/plataforma/gitlab.rs:1249:         return Err(uso(LEER_PUERTAS, "rama-invalida", &t::RAMA_NO_VALIDA));
crates/tc2/src/plataforma/gitlab.rs:1250:     }
crates/tc2/src/plataforma/gitlab.rs:1251:     if rol == Rol::Revisor {
crates/tc2/src/plataforma/gitlab.rs:1252:         return Err(rol_no_valido(LEER_PUERTAS, rol));
crates/tc2/src/plataforma/gitlab.rs:1253:     }
crates/tc2/src/plataforma/gitlab.rs:1254:     Ok(())
crates/tc2/src/plataforma/gitlab.rs:1255: }
```

La asercion, con un adaptador que solo tiene token de agente:
```text
crates/tc2/tests/plataforma_gitlab.rs:838:     let gl = gitlab(&mock, &[Rol::Agente]);
crates/tc2/tests/plataforma_gitlab.rs:839:     let l = gl.leer_puertas(Rol::Agente, "develop").unwrap();
crates/tc2/tests/plataforma_gitlab.rs:840:     assert!(!l.plan_da_reglas);
crates/tc2/tests/plataforma_gitlab.rs:841:     assert_eq!(l.reglas_rama, ["push=0", "merge=40", "code-owner"]);
crates/tc2/tests/plataforma_gitlab.rs:842:     assert!(gl.leer_puertas(Rol::Revisor, "develop").is_err());
```

`gitlab(&mock, &[Rol::Agente])` crea `CredencialesFijas` solo con el agente (plataforma_gitlab.rs:25-31 y 37-45). Sin la regla de rol, `leer_puertas(Rol::Revisor, ..)` llega a `puertas_comunes` → `leer_paginas` → `Conexion::token(op, Rol::Revisor)`, que falla con `rechazada motivo=identidad falta=revisor` (conexion.rs:78-92): sigue siendo `Err`.

`grep -n 'leer_puertas(_de)?\(\s*Rol::Revisor' crates/tc2` solo da plataforma_gitlab.rs:842; no hay ningun test de `leer_puertas` con `Rol::Revisor` en GitHub.

##### Mutantes propuestos (para el agente de mutacion; uno a uno)

- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da
1. crates/tc2/src/plataforma/gitlab.rs:1251-1253: borrar el bloque `if rol == Rol::Revisor { return Err(rol_no_valido(LEER_PUERTAS, rol)); }`
2. crates/tc2/src/plataforma/github.rs:944-946: borrar el mismo bloque.
- Orden: `cargo test -p tc2 --test plataforma_gitlab protecciones_de_gitlab`, `cargo test -p tc2 --test plataforma` y la suite completa `cargo test -p tc2` (por si `tests/doctor.rs` lo caza).
- Esperado si es D4: `protecciones_de_gitlab` en verde con el mutante 1 (`test result: ok. 1 passed`); con el 2, ningun test rojo.
- Rojo de control (para probar que el test corre): el mutante crates/tc2/src/plataforma/gitlab.rs:705 `!p.allow_force_push` → `true` tiene que poner rojo `protecciones_de_gitlab` (caso G01 con force-push, plataforma_gitlab.rs:602-612, espera `estado-puertas rechazada pr=12 motivo=config garantia=G01`). Aparte: tampoco hay ningun caso de rama invalida en `leer_puertas` (`rama-invalida` no sale en ningun test de A1).
- Test que deberia ponerse rojo: `protecciones_de_gitlab` si la asercion fuera `assert_eq!(gl.leer_puertas(Rol::Revisor, "develop").unwrap_err().linea(), "leer-puertas rechazada motivo=identidad rol=revisor")`, o con un adaptador que si tenga token de revisor.

##### Mutación

- Resultado: **confirmado** (los dos mutantes sobreviven en verde y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` propio). Clon restaurado con `git checkout -- .` entre pasos. No se ejecutó la suite completa (33 min por pase); sí los binarios de plataforma.

##### Mutante 1 (GitLab)

```diff
--- a/crates/tc2/src/plataforma/gitlab.rs
+++ b/crates/tc2/src/plataforma/gitlab.rs
@@ -1251,3 +1250,0 @@
-    if rol == Rol::Revisor {
-        return Err(rol_no_valido(LEER_PUERTAS, rol));
-    }
```

Orden: `cargo test -p tc2 --test plataforma_gitlab protecciones_de_gitlab`

```text
test protecciones_de_gitlab ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.55s
```

Orden: `cargo test -p tc2 --test plataforma`

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.50s
```

##### Mutante 2 (GitHub)

```diff
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -944,3 +943,0 @@
-    if rol == Rol::Revisor {
-        return Err(rol_no_valido(LEER_PUERTAS, rol));
-    }
```

Órdenes y salida:

```text
cargo test -p tc2 --test plataforma
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.15s
cargo test -p tc2 --test plataforma_gitlab
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.93s
cargo test -p tc2 --lib plataforma
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 367 filtered out; finished in 0.93s
```

##### Control (código roto de verdad)

```diff
--- a/crates/tc2/src/plataforma/gitlab.rs
+++ b/crates/tc2/src/plataforma/gitlab.rs
@@ -705 +705 @@
-            !p.allow_force_push
+            true
```

Orden: `cargo test -p tc2 --test plataforma_gitlab protecciones_de_gitlab`

```text
test protecciones_de_gitlab ... FAILED
thread 'protecciones_de_gitlab' (39520) panicked at crates\tc2\tests\plataforma_gitlab.rs:609:9:
assertion `left == right` failed
  left: "estado-puertas ok pr=12 aprobaciones=1 ignoradas=0"
 right: "estado-puertas rechazada pr=12 motivo=config garantia=G01"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 10.98s
```

<a id="a1-11"></a>

#### A1-11 · D4 · IMPORTANTE · pendiente [A1]

_Ningún caso ejercita el 403 con `retry-after` y sin `x-ratelimit-remaining` (límite secundario) de `es_limite`_

- id: A1-11
- regla: D4
- ambito: test
- ubicacion: crates/tc2/tests/plataforma.rs:410 ~ crates/tc2/src/plataforma/errores.rs:57
- ubicacion_detalle: test `errores_dirigidos_a_la_persona` (plataforma.rs:410-566) y tabla de plataforma_gitlab.rs:424-493; rama no comprobada en errores.rs:57
- modulo: plataforma (traduccion de errores HTTP, limite de peticiones)
- severidad: IMPORTANTE
- motivo_severidad: D4 de test si el mutante sobrevive. `es_limite` trata un 403 con `retry-after` y sin `x-ratelimit-remaining` como limite agotado (el limite secundario de GitHub), pero ningun caso de las tablas de errores lo ejercita: los 403 con `retry-after` traen tambien `remaining: 0`, y los 429 entran por `r.estado == 429`. Quitar esa condicion convertiria ese 403 en `denegada motivo=permisos` (diagnostico de permisos en vez de «espera N s») sin que ningun test de A1 se ponga rojo.
- estado: confirmado
- grupo_seguimiento: A1 — Plataforma e identidad
- evidencia: fragmentos literales con fichero:linea en la seccion «Evidencia» de abajo (contra d82ea9d)

##### Evidencia

La regla:
```text
crates/tc2/src/plataforma/errores.rs:51: /// Si la respuesta es un limite de peticiones agotado.
crates/tc2/src/plataforma/errores.rs:52: fn es_limite(r: &Respuesta) -> bool {
crates/tc2/src/plataforma/errores.rs:53:     // El limite secundario de GitHub puede llegar sin cabeceras: entonces lo
crates/tc2/src/plataforma/errores.rs:54:     // dice el mensaje. GitLab da 429.
crates/tc2/src/plataforma/errores.rs:55:     (r.estado == 403 || r.estado == 429)
crates/tc2/src/plataforma/errores.rs:56:         && (r.restantes.as_deref() == Some("0")
crates/tc2/src/plataforma/errores.rs:57:             || r.reintentar.is_some()
crates/tc2/src/plataforma/errores.rs:58:             || r.estado == 429
crates/tc2/src/plataforma/errores.rs:59:             || mensaje_api(r).to_ascii_lowercase().contains("rate limit"))
crates/tc2/src/plataforma/errores.rs:60: }
```

Los unicos casos con `retry-after` (todos entran tambien por otra condicion):
```text
crates/tc2/tests/plataforma.rs:426:         (
crates/tc2/tests/plataforma.rs:427:             Simulada::json(403, "{\"message\":\"API rate limit exceeded\"}")
crates/tc2/tests/plataforma.rs:428:                 .con("x-ratelimit-remaining", "0")
crates/tc2/tests/plataforma.rs:429:                 .con("retry-after", "42"),
crates/tc2/tests/plataforma.rs:430:             "identidad agotada motivo=limite-peticiones reintenta-en=42",
crates/tc2/tests/plataforma.rs:431:             5,
crates/tc2/tests/plataforma.rs:432:             "espera 42",
crates/tc2/tests/plataforma.rs:433:         ),
crates/tc2/tests/plataforma.rs:434:         (
crates/tc2/tests/plataforma.rs:435:             Simulada::json(429, "{}").con("retry-after", "7"),
crates/tc2/tests/plataforma.rs:436:             "identidad agotada motivo=limite-peticiones reintenta-en=7",
crates/tc2/tests/plataforma.rs:437:             5,
crates/tc2/tests/plataforma.rs:438:             "relanza",
```
```text
crates/tc2/tests/plataforma_gitlab.rs:443:         (
crates/tc2/tests/plataforma_gitlab.rs:444:             Simulada::json(429, "{\"message\":\"Retry later\"}").con("Retry-After", "30"),
crates/tc2/tests/plataforma_gitlab.rs:445:             "identidad agotada motivo=limite-peticiones reintenta-en=30",
crates/tc2/tests/plataforma_gitlab.rs:446:             5,
crates/tc2/tests/plataforma_gitlab.rs:447:             "espera 30 s",
crates/tc2/tests/plataforma_gitlab.rs:448:         ),
```

`grep -i 'retry-after\|ratelimit-remaining' crates/tc2` solo da esas lineas, plataforma.rs:838 (403 con `x-ratelimit-remaining: 0`) y plataforma_gitlab.rs:451 (403 con `RateLimit-Remaining: 0`).

##### Mutante propuesto (para el agente de mutacion)

- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da
- Fichero y linea: crates/tc2/src/plataforma/errores.rs:57
- Cambio: borrar la linea `            || r.reintentar.is_some()`
- Orden: `cargo test -p tc2 --test plataforma errores_dirigidos_a_la_persona`, `cargo test -p tc2 --test plataforma protecciones_y_bordes`, `cargo test -p tc2 --test plataforma_gitlab envoltorio_y_errores_de_gitlab` y la suite completa `cargo test -p tc2`.
- Esperado si es D4: las tres en verde con el mutante (`test result: ok. 1 passed` cada una).
- Rojo de control (para probar que los tests corren): en errores.rs:56 cambiar `&& (r.restantes.as_deref() == Some("0")` por `&& (false` tiene que poner rojo `envoltorio_y_errores_de_gitlab` (caso 403 con `RateLimit-Remaining: 0`, sin `retry-after` y con mensaje `x`, plataforma_gitlab.rs:449-456, espera `identidad agotada motivo=limite-peticiones reintenta-en=0`). Se lanza por separado del mutante de arriba.
- Test que deberia ponerse rojo y no lo hace: `errores_dirigidos_a_la_persona` (le falta un caso `Simulada::json(403, "{}").con("retry-after", "30")` con linea `identidad agotada motivo=limite-peticiones reintenta-en=30`).

##### Mutación

- Resultado: **confirmado** (el mutante sobrevive en verde y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` propio). Clon restaurado con `git checkout -- .` entre pasos. No se ejecutó la suite completa (33 min por pase).

##### Mutante

```diff
--- a/crates/tc2/src/plataforma/errores.rs
+++ b/crates/tc2/src/plataforma/errores.rs
@@ -56,3 +56,2 @@
         && (r.restantes.as_deref() == Some("0")
-            || r.reintentar.is_some()
             || r.estado == 429
```

Órdenes y salida con el mutante:

```text
cargo test -p tc2 --test plataforma errores_dirigidos_a_la_persona
test errores_dirigidos_a_la_persona ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 2.68s

cargo test -p tc2 --test plataforma protecciones_y_bordes
test protecciones_y_bordes ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 2.51s

cargo test -p tc2 --test plataforma_gitlab envoltorio_y_errores_de_gitlab
test envoltorio_y_errores_de_gitlab ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.97s

cargo test -p tc2 --lib plataforma
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 367 filtered out; finished in 1.17s
```

##### Control (código roto de verdad)

```diff
--- a/crates/tc2/src/plataforma/errores.rs
+++ b/crates/tc2/src/plataforma/errores.rs
@@ -56 +56 @@
-        && (r.restantes.as_deref() == Some("0")
+        && (false
```

Orden: `cargo test -p tc2 --test plataforma_gitlab envoltorio_y_errores_de_gitlab`

```text
test envoltorio_y_errores_de_gitlab ... FAILED
thread 'envoltorio_y_errores_de_gitlab' (41668) panicked at crates\tc2\tests\plataforma_gitlab.rs:482:9:
assertion `left == right` failed
  left: ("identidad denegada motivo=permisos rol=agente", 8)
 right: ("identidad agotada motivo=limite-peticiones reintenta-en=0", 5)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.21s
```

### A2 — Hallazgos y revisión

<a id="a2-02"></a>

#### A2-02 · D1 · IMPORTANTE · pendiente [A2]

_`validar_uno` y `validar_cierre` repiten la extracción del id y la closure del `Fallo`_

- id: A2-02
- regla: D1
- ubicacion: crates/tc2/src/hallazgos/mod.rs:695-704 ~ crates/tc2/src/hallazgos/mod.rs:828-837
- modulo: hallazgos
- severidad: IMPORTANTE
- motivo_severidad: D1 real (regla 4D: minimo IMPORTANTE). `validar_uno` y `validar_cierre` repiten literalmente la extraccion del id valido y la closure que construye el `Fallo`; ningun ADR lo impone (F1-8 adenda F6-4/F6-6 fija las reglas, no su codigo). No tapa fallo de datos ni de seguridad; no `divergente`.
- evidencia:

  `crates/tc2/src/hallazgos/mod.rs:695-704` (en `fn validar_uno`)
  ```rust
      let id = obj
          .get("id")
          .and_then(Value::as_str)
          .filter(|s| es_id(s))
          .map(str::to_owned);
      let fallo = |motivo: &'static str, detalle: String| Fallo {
          id: id.clone(),
          motivo,
          detalle,
      };
  ```

  `crates/tc2/src/hallazgos/mod.rs:828-837` (en `fn validar_cierre`)
  ```rust
      let id = c
          .get("id")
          .and_then(Value::as_str)
          .filter(|s| es_id(s))
          .map(str::to_owned);
      let fallo = |motivo: &'static str, detalle: String| Fallo {
          id: id.clone(),
          motivo,
          detalle,
      };
  ```

  Diff normalizado: identicos salvo el receptor (`obj`, un `Map`, frente a `c`, un `Value`; los dos admiten `.get("id")`). 10 lineas, 67 tokens normalizados (pase 0). Version unica: `fn id_valido(v: &Value) -> Option<String>` y `Fallo::de(id)` o una closure comun.
- estado: confirmado
- grupo_seguimiento: A2

<a id="a2-03"></a>

#### A2-03 · D1 · IMPORTANTE · pendiente [A2]

_Los dos bucles de `validar_con` repiten el corte por `TOPE_HALLAZGOS` y la conversión `Fallo -> Rechazo`_

- id: A2-03
- regla: D1
- ubicacion: crates/tc2/src/hallazgos/mod.rs:956-977 ~ crates/tc2/src/hallazgos/mod.rs:980-1013
- modulo: hallazgos
- severidad: IMPORTANTE
- motivo_severidad: D1 real (regla 4D: minimo IMPORTANTE). Los dos bucles de `validar_con` (hallazgos y cierres) repiten el corte por `TOPE_HALLAZGOS` con su `Rechazo` y la conversion `Fallo -> Rechazo`; ningun ADR lo impone. No tapa fallo de datos ni de seguridad; no `divergente`. Ademas la copia de los cierres no tiene test (ver A2-13).
- evidencia:

  `crates/tc2/src/hallazgos/mod.rs:957-968`
  ```rust
          if i >= TOPE_HALLAZGOS {
              v.rechazados.push(Rechazo {
                  indice: Some(i),
                  id: None,
                  motivo: motivo::FUERA_DE_TOPE,
                  detalle: format!(
                      "{} hallazgos pasan del tope de {TOPE_HALLAZGOS}",
                      lista.len() - TOPE_HALLAZGOS
                  ),
              });
              break;
          }
  ```

  `crates/tc2/src/hallazgos/mod.rs:982-993`
  ```rust
          if j >= TOPE_HALLAZGOS {
              v.rechazados.push(Rechazo {
                  indice: Some(indice),
                  id: None,
                  motivo: motivo::FUERA_DE_TOPE,
                  detalle: format!(
                      "{} cierres pasan del tope de {TOPE_HALLAZGOS}",
                      cierres.len() - TOPE_HALLAZGOS
                  ),
              });
              break;
          }
  ```

  Y la conversion del error, igual en los dos: `crates/tc2/src/hallazgos/mod.rs:971-976` y `:1007-1012`:
  `Err(f) => v.rechazados.push(Rechazo { indice: Some(i|indice), id: f.id, motivo: f.motivo, detalle: f.detalle, }),`

  Diff normalizado: difieren en la variable del indice (`i` / `indice`), la palabra (`hallazgos` / `cierres`) y la lista (`lista` / `cierres`). 12+6 lineas por copia, 61 tokens (pase 0). Version unica: `fn fuera_de_tope(indice, que, sobran) -> Rechazo` e `impl Fallo { fn rechazo(self, indice) -> Rechazo }`.
- estado: confirmado
- grupo_seguimiento: A2

<a id="a2-04"></a>

#### A2-04 · D1 · IMPORTANTE · pendiente [A2]

_Tres funciones pintan la misma fila Markdown de un hallazgo con los mismos topes de `plano`_

- id: A2-04
- regla: D1
- ubicacion: crates/tc2/src/hallazgos/mod.rs:1193-1204 ~ crates/tc2/src/hallazgos/mod.rs:1866-1881 ~ crates/tc2/src/hallazgos/ronda.rs:325-337
- modulo: hallazgos
- severidad: IMPORTANTE
- motivo_severidad: D1 real (regla 4D: minimo IMPORTANTE). Tres funciones pintan la misma fila Markdown de un hallazgo (id, severidad, clase, fichero, linea, descripcion) con los mismos topes de `plano` (64, 200, 400) a mano; cambiar un tope o una columna exige tocar las tres. Ningun ADR impone tres copias (F1-8 adenda F6-7 impone pasar el texto por `plano`, no repetirlo). No tapa fallo de datos ni de seguridad: las tres pasan por `plano`. No `divergente` en comportamiento: las columnas extra (evidencia, revisora, ronda, estado) son deliberadas.
- evidencia:

  `crates/tc2/src/hallazgos/mod.rs:1193-1203` (`resumen_markdown`)
  ```rust
          for a in &v.aceptados {
              s.push_str(&format!(
                  "| {} | {} | {} | {} | {} | {} | {} |\n",
                  plano(&a.id, 64),
                  a.severidad.texto(),
                  a.clase.texto(),
                  plano(&a.fichero, 200),
                  a.linea,
                  a.evidencia.tipo.texto(),
                  plano(&a.descripcion, 400)
  ```

  `crates/tc2/src/hallazgos/ronda.rs:325-335` (`abiertos_markdown`)
  ```rust
      for a in abiertos {
          let h = &a.hallazgo;
          s.push_str(&format!(
              "| {} | {} | {} | {} | {} | {} | {} |\n",
              plano(&h.id, 64),
              a.ronda,
              h.severidad.texto(),
              h.clase.texto(),
              plano(&h.fichero, 200),
              h.linea,
  ```

  `crates/tc2/src/hallazgos/mod.rs:1873-1879` (`InformeTarea::markdown`, las mismas seis celdas tras revisora, ronda y estado):
  `plano(&h.id, 64), h.severidad.texto(), h.clase.texto(), plano(&h.fichero, 200), h.linea, h.evidencia.tipo.texto(), plano(&h.descripcion, 400)`

  Diff normalizado: mismas celdas y topes; difieren en la columna extra (`evidencia.tipo` / `a.ronda` / revisora+ronda+estado) y en `a` frente a `a.hallazgo`. Version unica: `fn celdas(h: &Aceptado) -> [String; 6]` (o `fn fila(h, extra)`) usada por las tres.
- estado: confirmado
- grupo_seguimiento: A2

<a id="a2-05"></a>

#### A2-05 · D1 · IMPORTANTE · pendiente [A2]

_Siete subórdenes de `tc2 hallazgos` escriben a mano el mismo analizador de opciones_

- id: A2-05
- regla: D1
- ubicacion: crates/tc2/src/hallazgos/orden.rs:664-678 ~ crates/tc2/src/hallazgos/orden.rs:717-731 (y el mismo esqueleto en :117-139, :287-302, :435-446, :488-511, :759-798)
- modulo: hallazgos
- severidad: IMPORTANTE
- motivo_severidad: D1 real (regla 4D: minimo IMPORTANTE). Siete suborders de `tc2 hallazgos` escriben a mano el mismo analizador de opciones (`it`, `valor`, `match`, `argumento-desconocido`); `desde` y `abiertos` son la misma funcion con otros nombres de opcion. Ningun ADR lo impone (F1-4 fija la linea de contrato y `uso`, no el analizador). No tapa fallo de datos ni de seguridad; no `divergente`.
- evidencia:

  `crates/tc2/src/hallazgos/orden.rs:664-675` (`fn desde`)
  ```rust
      let (mut anterior, mut head, mut base): (Option<String>, Option<String>, Option<String>) =
          (None, None, None);
      let mut it = a.iter();
      while let Some(x) = it.next() {
          let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
          match x.as_str() {
              "--anterior" if anterior.is_none() => anterior = Some(valor()?),
              "--head" if head.is_none() => head = Some(valor()?),
              "--base" if base.is_none() => base = Some(valor()?),
              _ => return Err(uso("argumento-desconocido")),
  ```

  `crates/tc2/src/hallazgos/orden.rs:717-728` (`fn abiertos`)
  ```rust
      let (mut r, mut detalle, mut salida): (Option<PathBuf>, Option<PathBuf>, Option<PathBuf>) =
          (None, None, None);
      let mut it = a.iter();
      while let Some(x) = it.next() {
          let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
          match x.as_str() {
              "--ronda" if r.is_none() => r = Some(valor()?.into()),
              "--detalle" if detalle.is_none() => detalle = Some(valor()?.into()),
              "--salida" if salida.is_none() => salida = Some(valor()?.into()),
              _ => return Err(uso("argumento-desconocido")),
  ```

  y las dos siguen igual: `let (Some(..), Some(..), Some(..)) = (..) else { return Err(uso("faltan-argumentos")); };` (`:676-678` y `:729-731`).

  Diff normalizado: identicos salvo los nombres de las tres opciones y `String` frente a `PathBuf` (`.into()`). 126 tokens normalizados (pase 0). La linea `let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));` sale 7 veces en el fichero (`:121`, `:294`, `:440`, `:495`, `:668`, `:721`, `:766`). Version unica: un `fn opciones(a, unicas: &[&str], repetibles: &[&str]) -> Result<BTreeMap<&str, Vec<String>>, Informe>`.
- estado: confirmado
- grupo_seguimiento: A2 (la misma linea esta tambien en crates/tc2/src/seleccion/orden.rs:350: pistas/A2.txt punto 4)

<a id="a2-11"></a>

#### A2-11 · D4 · IMPORTANTE · pendiente [A2]

_Ningún test comprueba el dictamen de `veredicto_ronda` en modo `tope` ni `ligera`_

- id: A2-11
- regla: D4
- ubicacion: crates/tc2/src/hallazgos/mod.rs:1561-1564 (test que deberia cubrirlo: crates/tc2/src/hallazgos/mod.rs:2457-2520 `el_informe_de_la_tarea_cuenta_como_el_check`)
- modulo: hallazgos (tests)
- severidad: IMPORTANTE
- motivo_severidad: D4 en tests (regla 4D: minimo IMPORTANTE). Ningun test comprueba el dictamen que `veredicto_ronda` da en modo `tope` ni en modo `ligera`: el unico test que construye esas rondas compara solo nuevos/persisten/cerrados y el modo leido del resumen. Un cambio que pusiera `verde` (`success`) en una ronda `tope` con un IMPORTANTE abierto dejaria pasar la puerta `revision-ia`, y la suite seguiria en verde (por lectura; falta ejecutarlo). No CRITICO: el codigo de hoy es correcto, el test no tapa un fallo existente.
- evidencia:

  Codigo, `crates/tc2/src/hallazgos/mod.rs:1561-1564`
  ```rust
      v.dictamen = if v.modo == ronda::Modo::Tope {
          Dictamen::Tope
      } else if v.modo == ronda::Modo::Ligera && v.filas.is_empty() {
          Dictamen::Verde
  ```

  Test, `crates/tc2/src/hallazgos/mod.rs:2476-2491` (`cuenta`, lo unico que se mira de cada veredicto):
  ```rust
          let cuenta = |v: &Veredicto| {
              let resumen = resumen_veredicto(v, "");
              assert_eq!(ronda::modo_de_resumen(&resumen), Some(v.modo), "{resumen}");
              let i = InformeTarea::nuevo(
                  "TASK-001",
                  (1, &sha, v.ronda),
                  (v.dictamen.conclusion(), v.dictamen),
                  &resumen,
                  v.aceptados.clone(),
              );
  ```
  y `:2486-2491` compara solo `(i.nuevos, i.persisten, i.cerrados)` con `(v.nuevos, v.persisten, v.cerrados)`. El caso `tope` es `:2514-2519` (`rojo` con I1 IMPORTANTE, `decidir(Some(&rojo), &[], 2, false)`, `cuenta(&veredicto_ronda(&[], Some(&d)))`) y el `ligera`, `:2493-2497`; ninguno mira `v.dictamen`. Busqueda en todo el repo (`*.rs`): no hay `veredicto=tope` ni un `assert` sobre `Dictamen::Tope` salido de `veredicto_ronda`; `Dictamen::Tope` solo aparece en tests de `dictamen_publicado` (mod.rs:2558-2571).

  Mutante propuesto (commit base d82ea9d):
  ```diff
  --- a/crates/tc2/src/hallazgos/mod.rs
  +++ b/crates/tc2/src/hallazgos/mod.rs
  @@ -1561,3 +1561,3 @@
       v.dictamen = if v.modo == ronda::Modo::Tope {
  -        Dictamen::Tope
  +        Dictamen::Verde
       } else if v.modo == ronda::Modo::Ligera && v.filas.is_empty() {
  ```
  Ordenes: `cargo test -p tc2 --lib hallazgos::tests::el_informe_de_la_tarea_cuenta_como_el_check` (el test que deberia ponerse rojo y se espera verde con el mutante), y despues `cargo test -p tc2 --lib hallazgos` y `cargo test -p tc2 --test hallazgos` para ver que ningun otro lo detecta. Segundo mutante del mismo hueco: `:1564` `Dictamen::Verde` -> `Dictamen::Incompleta` (ligera).
  Contraste en rojo con el codigo roto de verdad: anadir al test `assert_eq!(veredicto_ronda(&[], Some(&d)).dictamen, Dictamen::Tope)` en el caso `tope` y ver que el mutante lo pone rojo.
- estado: confirmado
- grupo_seguimiento: A2

##### Mutación

- Resultado: **confirmado** (los dos mutantes sobreviven en verde y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` propio). Clon restaurado con `git checkout -- .` entre pasos.
- El contraste propuesto (añadir una aserción al test) no se aplica: el control usado rompe de verdad una regla que el test sí cubre, para probar que corre.

##### Mutante 1 (tope)

```diff
--- a/crates/tc2/src/hallazgos/mod.rs
+++ b/crates/tc2/src/hallazgos/mod.rs
@@ -1562 +1562 @@
-        Dictamen::Tope
+        Dictamen::Verde
```

```text
cargo test -p tc2 --lib hallazgos::tests::el_informe_de_la_tarea_cuenta_como_el_check
test hallazgos::tests::el_informe_de_la_tarea_cuenta_como_el_check ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.01s

cargo test -p tc2 --lib hallazgos
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 376 filtered out; finished in 0.21s

cargo test -p tc2 --test hallazgos
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.74s
```

##### Mutante 2 (ligera)

```diff
--- a/crates/tc2/src/hallazgos/mod.rs
+++ b/crates/tc2/src/hallazgos/mod.rs
@@ -1564 +1564 @@
-        Dictamen::Verde
+        Dictamen::Incompleta
```

```text
cargo test -p tc2 --lib hallazgos::tests::el_informe_de_la_tarea_cuenta_como_el_check
test hallazgos::tests::el_informe_de_la_tarea_cuenta_como_el_check ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.00s

cargo test -p tc2 --lib hallazgos
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 376 filtered out; finished in 0.16s

cargo test -p tc2 --test hallazgos
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.87s
```

##### Control (código roto de verdad)

```diff
--- a/crates/tc2/src/hallazgos/mod.rs
+++ b/crates/tc2/src/hallazgos/mod.rs
@@ -1556 +1556 @@
-            v.cerrados += 1;
+            v.cerrados += 0;
```

Orden: `cargo test -p tc2 --lib hallazgos::tests::el_informe_de_la_tarea_cuenta_como_el_check`

```text
test hallazgos::tests::el_informe_de_la_tarea_cuenta_como_el_check ... FAILED
thread 'hallazgos::tests::el_informe_de_la_tarea_cuenta_como_el_check' (4716) panicked at crates\tc2\src\hallazgos\mod.rs:2496:9:
assertion `left == right` failed
  left: (0, 1, 0)
 right: (0, 1, 1)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.00s
```

<a id="a2-12"></a>

#### A2-12 · D4 · IMPORTANTE · pendiente [A2]

_La rama «mezcla con conflictos -> `None`» de `desde_incremental` no la ejecuta ningún test_

- id: A2-12
- regla: D4
- ubicacion: crates/tc2/src/hallazgos/orden.rs:653-657 (test que deberia cubrirlo: crates/tc2/tests/hallazgos.rs:862-933 `el_diff_incremental_no_arrastra_lo_que_trae_la_base`)
- modulo: hallazgos (tests)
- severidad: IMPORTANTE
- motivo_severidad: D4 en tests (regla 4D: minimo IMPORTANTE). La rama «la mezcla tiene conflictos -> `None` (falla en cerrado: se revisa todo el PR)» que promete el doc de `desde_incremental` y de la que depende el workflow (`.github/workflows/revision-ia.yml:337-344`) no la ejecuta ningun test: el test prueba `merge`, `directo` y `no-ancestro`. Un mutante que trate el conflicto como `merge` (diff contra un arbol con marcas de conflicto) seguiria en verde (por lectura; falta ejecutarlo). No CRITICO: el codigo de hoy hace lo que dice.
- evidencia:

  Codigo, `crates/tc2/src/hallazgos/orden.rs:653-657`
  ```rust
      match s.codigo {
          0 if es_sha(&arbol) => (Some(arbol), "merge"),
          1 => (None, "conflicto"),
          _ => (None, "git"),
      }
  ```
  Promesa, `crates/tc2/src/hallazgos/orden.rs:608-609`: «Si esa mezcla tiene conflictos, o git falla, `None` (falla en cerrado: se revisa todo el PR).»

  Test, `crates/tc2/tests/hallazgos.rs:893-910`: los tres casos son `motivo == "merge"` (`:894`), `(Some(a.clone()), "directo")` (`:903`) y `(None, "no-ancestro")` (`:909`); en el repo de la fixture `develop` solo toca `otro.rs` y la rama solo `pr.rs` (`:872-886`), asi que `merge-tree` nunca sale con codigo 1. Busqueda en `crates/tc2/tests`: ningun test de `hallazgos` contiene `conflicto`.

  Mutante propuesto (commit base d82ea9d):
  ```diff
  --- a/crates/tc2/src/hallazgos/orden.rs
  +++ b/crates/tc2/src/hallazgos/orden.rs
  @@ -654,3 +654,3 @@
           0 if es_sha(&arbol) => (Some(arbol), "merge"),
  -        1 => (None, "conflicto"),
  +        1 => (Some(arbol), "merge"),
           _ => (None, "git"),
  ```
  Orden: `cargo test -p tc2 --test hallazgos el_diff_incremental_no_arrastra_lo_que_trae_la_base` (se espera verde con el mutante) y `cargo test -p tc2 --test hallazgos` entero.
  Contraste en rojo: un caso nuevo en el test donde `develop` y la rama editan la misma linea de `pr.rs` tras A, con `assert_eq!(desde_incremental(..), (None, "conflicto"))`, que el mutante pone rojo.
- estado: confirmado
- grupo_seguimiento: A2

##### Mutación

- Resultado: **confirmado** (el mutante sobrevive en verde y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` propio). Clon restaurado con `git checkout -- .` entre pasos.
- El contraste propuesto (un caso nuevo de conflicto en el test) no se aplica: el control usado rompe de verdad la rama `merge`, que el test sí cubre, para probar que corre.

##### Mutante

```diff
--- a/crates/tc2/src/hallazgos/orden.rs
+++ b/crates/tc2/src/hallazgos/orden.rs
@@ -655 +655 @@
-        1 => (None, "conflicto"),
+        1 => (Some(arbol), "merge"),
```

```text
cargo test -p tc2 --test hallazgos el_diff_incremental_no_arrastra_lo_que_trae_la_base
test el_diff_incremental_no_arrastra_lo_que_trae_la_base ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 6.12s

cargo test -p tc2 --test hallazgos
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.69s

cargo test -p tc2 --lib hallazgos
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 376 filtered out; finished in 0.13s
```

##### Control (código roto de verdad)

```diff
--- a/crates/tc2/src/hallazgos/orden.rs
+++ b/crates/tc2/src/hallazgos/orden.rs
@@ -654 +654 @@
-        0 if es_sha(&arbol) => (Some(arbol), "merge"),
+        0 if es_sha(&arbol) => (None, "merge"),
```

Orden: `cargo test -p tc2 --test hallazgos el_diff_incremental_no_arrastra_lo_que_trae_la_base`

```text
test el_diff_incremental_no_arrastra_lo_que_trae_la_base ... FAILED
thread 'el_diff_incremental_no_arrastra_lo_que_trae_la_base' (43584) panicked at crates\tc2\tests\hallazgos.rs:895:23:
un arbol desde el que sacar el diff
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 4.56s
```

<a id="a2-13"></a>

#### A2-13 · D4 · IMPORTANTE · pendiente [A2]

_«Cada regla del cierre rechaza con su motivo» no prueba el tope de cierres (`fuera-de-tope`)_

- id: A2-13
- regla: D4
- ubicacion: crates/tc2/src/hallazgos/mod.rs:982-993 (test que deberia cubrirlo: crates/tc2/src/hallazgos/mod.rs:2351-2455 `cada_regla_del_cierre_rechaza_con_su_motivo`)
- modulo: hallazgos (tests)
- severidad: IMPORTANTE
- motivo_severidad: D4 en tests (regla 4D: minimo IMPORTANTE). El test se llama «cada regla del cierre rechaza con su motivo» y no prueba el tope de cierres (`fuera-de-tope` con mas de `TOPE_HALLAZGOS` cierres), que el prompt anuncia («Como mucho 100 hallazgos y 100 cierres», mod.rs:1108). El tope de hallazgos si tiene su caso (mod.rs:2037-2043); el de cierres, ninguno. Un mutante que quite o mueva ese tope seguiria en verde (por lectura; falta ejecutarlo). No CRITICO: el codigo de hoy corta bien; es la cota de una entrada del modelo, no un fallo de datos.
- evidencia:

  Codigo, `crates/tc2/src/hallazgos/mod.rs:980-983`
  ```rust
      for (j, c) in cierres.iter().enumerate() {
          let indice = lista.len() + j;
          if j >= TOPE_HALLAZGOS {
              v.rechazados.push(Rechazo {
  ```

  Test de hallazgos (si existe), `crates/tc2/src/hallazgos/mod.rs:2037-2043`:
  ```rust
          // Mas de TOPE_HALLAZGOS: los que sobran, un rechazo.
          let muchos: Vec<String> = (0..=TOPE_HALLAZGOS)
  ```
  Test de cierres, `crates/tc2/src/hallazgos/mod.rs:2384-2411`: los casos son `cierre-desconocido` (x2), `cierre-fuera-del-diff`, `paso-rojo-sigue-rojo`, `hallazgo-invalido` (x2), `id-repetido` y `json-invalido` (x2); ninguno pasa de dos cierres. Busqueda en el repo (`*.rs`, `*.yml`): `"cierres pasan"` solo esta en mod.rs:988 y `fuera-de-tope` solo en mod.rs:64 y en el test de hallazgos (:2043).

  Mutante propuesto (commit base d82ea9d):
  ```diff
  --- a/crates/tc2/src/hallazgos/mod.rs
  +++ b/crates/tc2/src/hallazgos/mod.rs
  @@ -982,1 +982,1 @@
  -        if j >= TOPE_HALLAZGOS {
  +        if j >= TOPE_HALLAZGOS * 10 {
  ```
  Ordenes: `cargo test -p tc2 --lib hallazgos::tests::cada_regla_del_cierre_rechaza_con_su_motivo` (se espera verde con el mutante), y `cargo test -p tc2 --lib hallazgos`, `cargo test -p tc2 --test hallazgos` y `cargo test -p tc2 --test inyeccion` para ver que ninguno lo detecta.
  Contraste en rojo: 101 cierres de `I1` en el test con `assert_eq!(v.motivos().last(), Some(&format!("#{TOPE_HALLAZGOS}:fuera-de-tope")))` (sin hallazgos el indice del cierre 100 es 100; con el mutante el ultimo motivo pasa a `I1:id-repetido`), que el mutante pone rojo.
- estado: confirmado
- grupo_seguimiento: A2

##### Mutación

- Resultado: **confirmado** (el mutante sobrevive en verde y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` propio). Clon restaurado con `git checkout -- .` entre pasos.
- El contraste propuesto (101 cierres en el test) no se aplica: el control usado rompe de verdad el motivo de los cierres rechazados, que el test sí cubre, para probar que corre.

##### Mutante

```diff
--- a/crates/tc2/src/hallazgos/mod.rs
+++ b/crates/tc2/src/hallazgos/mod.rs
@@ -982 +982 @@
-        if j >= TOPE_HALLAZGOS {
+        if j >= TOPE_HALLAZGOS * 10 {
```

```text
cargo test -p tc2 --lib hallazgos::tests::cada_regla_del_cierre_rechaza_con_su_motivo
test hallazgos::tests::cada_regla_del_cierre_rechaza_con_su_motivo ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.01s

cargo test -p tc2 --lib hallazgos
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 376 filtered out; finished in 0.21s

cargo test -p tc2 --test hallazgos
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.77s

cargo test -p tc2 --test inyeccion
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.05s
```

##### Control (código roto de verdad, rama `Err` del bucle de cierres)

```diff
--- a/crates/tc2/src/hallazgos/mod.rs
+++ b/crates/tc2/src/hallazgos/mod.rs
@@ -1010 +1010 @@
-                motivo: f.motivo,
+                motivo: motivo::FUERA_DE_TOPE,
```

Orden: `cargo test -p tc2 --lib hallazgos::tests::cada_regla_del_cierre_rechaza_con_su_motivo`

```text
test hallazgos::tests::cada_regla_del_cierre_rechaza_con_su_motivo ... FAILED
thread 'hallazgos::tests::cada_regla_del_cierre_rechaza_con_su_motivo' (47532) panicked at crates\tc2\src\hallazgos\mod.rs:2414:13:
assertion `left == right` failed: Informe.
{"hallazgos": [], "cierres": [{"id":"X9","fichero":"src/calculo.rs","linea":12,"motivo":"arreglado"}]}
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.01s
```

### A3 — Plan, importar, tarea y alta

<a id="a3-03"></a>

#### A3-03 · D1 · IMPORTANTE divergente · pendiente [A3]

_`leer_fichero` de `plan` e `import` copiado y divergente: `import` no sanea ruta ni error (F1-8) ni rechaza vacío_

- id: A3-03
- regla: D1
- ubicacion: crates/tc2/src/plan/mod.rs:139-170 (`leer_fichero` de `plan guardar` y `approve --pedir-cambios`) ~ crates/tc2/src/importar/mod.rs:246-271 (`leer_fichero` de `import`); tope duplicado en crates/tc2/src/plan/guardar.rs:33-34 ~ crates/tc2/src/importar/fichero.rs:32-33
- modulo: plan, importar
- severidad: IMPORTANTE (divergente)
- motivo_severidad: D1 real (la misma lectura acotada de un fichero externo, 25-30 lineas, copiada) cuyas copias ya divergen: `plan` sanea la ruta y el error antes de meterlos en el diagnostico (dato no confiable, F1-8) y rechaza el fichero vacio; `import` mete la ruta y el error del sistema tal cual. No llega a CRITICO: la ruta la pasa quien lanza la orden y `una_linea` (`contrato.rs:322-324`) ya junta las lineas al imprimir.
- estado: confirmado (por lectura contra d82ea9d; no se ejecuto)
- grupo_seguimiento: A3

##### Evidencia

Copia 1, `crates/tc2/src/plan/mod.rs:152-158`:

```rust
    let f = std::fs::File::open(ruta).map_err(no_se_lee)?;
    let tope = u64::try_from(guardar::MAX_BYTES).unwrap_or(u64::MAX);
    let mut bytes = Vec::new();
    f.take(tope.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(no_se_lee)?;
    if bytes.len() > guardar::MAX_BYTES {
```

Copia 2, `crates/tc2/src/importar/mod.rs:257-263`:

```rust
    let f = std::fs::File::open(ruta).map_err(no_se_lee)?;
    let tope = u64::try_from(limites.bytes).unwrap_or(u64::MAX);
    let mut bytes = Vec::new();
    f.take(tope.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(no_se_lee)?;
    if bytes.len() > limites.bytes {
```

Diff normalizado: identicas salvo el nombre del tope (`guardar::MAX_BYTES` frente a `limites.bytes`, ambos `1 << 20`: `guardar.rs:34` «el del import, D4 de TASK-029» copia `fichero.rs:33` en vez de usarlo). Alrededor:

- `plan/mod.rs:141-151`: el diagnostico lleva `sanear(&ruta.display().to_string())` y `sanear(&e.to_string())` (`plataforma/errores.rs:37-44`: controles a espacio, 200 caracteres).
- `importar/mod.rs:248-256` y `:266-270`: `.con("ruta", ruta.display().to_string())` y `.con("error", e.to_string())`, sin sanear.
- `plan/mod.rs:166-168` rechaza `vacio`; `import` no (lo rechaza despues `fichero::leer` con `sin-entradas`).

Entrada con la que ya se comportan distinto: una ruta con un caracter de control que no es salto de linea, p. ej. `ESC` (`tc2 import "$(printf 'x\033[31m.md')"` frente a `tc2 plan guardar TASK-001 plan-final.md "$(printf 'x\033[31m.md')"`). `plan` deja un espacio en su lugar (lo fija `plan::tests::fichero_vacio_no_utf8_grande_y_ruta_saneada`, `plan/mod.rs:717-726`, para `\n`); `import` lo pasa al stderr tal cual, porque `una_linea` solo parte por lineas. Derivado del codigo, no ejecutado.

Arreglo esperado (para el seguimiento): una sola `leer_acotado(ruta, tope) -> Result<String, ...>` con el saneado, y una sola constante de tope.

<a id="a3-05"></a>

#### A3-05 · D1 · IMPORTANTE · pendiente [A3]

_Secuencia de `approve` copiada para `--hito`_

- id: A3-05
- regla: D1
- ubicacion: crates/tc2/src/plan/mod.rs:283-295 (`paso_aprobar_hito`) ~ crates/tc2/src/plan/mod.rs:349-362 (`paso_aprobar`); tambien crates/tc2/src/plan/mod.rs:268-281 (`aprobar_hito_en`) ~ :542-551 (`de_la_orden`)
- modulo: plan (mod)
- severidad: IMPORTANTE
- motivo_severidad: D1 real: la secuencia de `approve` (quien primero, rechazo `identidad` si no es persona, `fetch`, configuracion de la base y transaccion) esta copiada para `--hito`; el propio comentario lo dice («Como `approve`»). 11 lineas y mas de 50 tokens iguales salvo el constructor. Hoy no divergen.
- estado: confirmado (por lectura contra d82ea9d)
- grupo_seguimiento: A3

##### Evidencia

Copia 1, `crates/tc2/src/plan/mod.rs:284-295`:

```rust
    // Como `approve`: quien, lo primero.
    let quien = match (e.quien)() {
        Ok(q) => q,
        Err(i) if i.resultado == Resultado::Rechazada => return Err(sin_persona(Some(i))),
        Err(i) => return Err(i),
    };
    let Some(op) = aprobar::AprobarHito::nueva(hito, &quien, e.ahora) else {
        return Err(sin_persona(None));
    };
    traer(e.git, e.dir)?;
    let op = op.con_config(&config_de_base(e.git, e.dir)?);
    let t = escribir(e, &op);
```

Copia 2, `crates/tc2/src/plan/mod.rs:350-362`:

```rust
    // Lo primero, quien: el agente (o sin identidad) no llega a leer ni a
    // escribir nada.
    let quien = match (e.quien)() {
        Ok(q) => q,
        Err(i) if i.resultado == Resultado::Rechazada => return Err(sin_persona(Some(i))),
        Err(i) => return Err(i),
    };
    let Some(op) = Aprobar::nueva(id, &quien, e.ahora) else {
        return Err(sin_persona(None));
    };
    traer(e.git, e.dir)?;
    let op = op.con_config(&config_de_base(e.git, e.dir)?);
    let t = escribir(e, &op);
```

Diff normalizado: `aprobar::AprobarHito::nueva(hito, ..)` frente a `Aprobar::nueva(id, ..)`; el resto, igual. Lo mismo, mas corto, en el posproceso del informe: `aprobar_hito_en` (`:272-279`: `clone_into(&mut i.comando)`, `retain` de `operacion`/`id`, `insert(0, ..)` y `pendiente=no` con `sinred`) repite `de_la_orden` (`:543-549`) cambiando la clave `id` por `hito`.

Arreglo esperado: un `persona_y_config(e, |quien| Op::nueva(..))` comun, y `de_la_orden` con la clave de cabeza como parametro.

<a id="a3-08"></a>

#### A3-08 · D2 · IMPORTANTE · pendiente [A3]

_Dos definiciones de «listo»: `new`/`import` aceptan enunciados que `plan` rechaza después (RF-01)_

- id: A3-08
- regla: D2
- ubicacion: crates/tc2/src/alta.rs:75-108 (`Entrada::listo`, usado por `new` e `import`) frente a crates/tc2/src/plan/heuristica.rs:130-144 (`Enunciado::validar`, usado por `plan`)
- modulo: alta, plan (heuristica)
- severidad: IMPORTANTE
- normalizacion: consolidador TASK-138, 2026-11-22: sube de MENOR a IMPORTANTE: es D2 que cambia el comportamiento observable (`new`/`import` aceptan un enunciado que RF-01 manda rechazar al dar de alta, y el rechazo llega una fase tarde, en `plan`). La regla del plan (decision 4) deja D2 en MENOR solo si no cambia el comportamiento.
- motivo_severidad: RF-01 pide «rechazar un enunciado no listo» al dar de alta, pero hay dos definiciones de «listo» y `new`/`import` aceptan enunciados que `plan` rechaza despues. No esconde un fallo (el rechazo llega, con motivo, en `plan`), solo lo retrasa una fase; por eso se queda en MENOR. No es D1: las dos funciones no son copia textual, son dos reglas para lo mismo.
- estado: confirmado (por lectura contra d82ea9d; las salidas se derivan del codigo, no se ejecutaron)
- grupo_seguimiento: A3 (solo documento: MENOR)

##### Evidencia

Requisito, `docs/plan/REQUISITOS-FUNCIONALES.md:118`: «RF-01 | Dar de alta una tarea con título, tipo, objetivo, criterios de aceptación comprobables [...]; rechazar un enunciado no listo».

Regla del alta, `crates/tc2/src/alta.rs:76-82` (no exige objetivo ni pone tope de criterios):

```rust
        let objetivo = self.objetivo.trim();
        if objetivo.is_empty() && self.criterios.is_empty() && !exige_criterios {
            return Ok(());
        }
        if self.criterios.is_empty() {
            return Err(NoListo::SinCriterios);
        }
```

Regla de `plan`, `crates/tc2/src/plan/heuristica.rs:131-142`:

```rust
        if self.objetivo.trim().is_empty() {
            return Err(("objetivo-vacio", 0));
        }
        ...
        if self.criterios.len() > MAX_CRITERIOS {
            return Err(("demasiados-criterios", self.criterios.len()));
        }
```

Entradas con las que divergen:
1. `tc2 new "X" --criterio "c"` (sin `--objetivo`): `listo(false)` devuelve `Ok` (objetivo vacio con criterios no entra en ningun `Err` de `:80-106`) y la tarea se publica; `tc2 plan TASK-NNN` sale `rechazada motivo=objetivo-vacio`. Igual con una entrada de `import` sin lineas `>` (`listo(true)` tampoco mira el objetivo; `importar/fichero.rs:404-408`).
2. `tc2 new "X" --objetivo "o"` con 13 `--criterio` distintos: `listo` da `Ok`; `plan` sale `rechazada motivo=demasiados-criterios` (`MAX_CRITERIOS = 12`, `heuristica.rs:67`).

Version reducida del arreglo (5 lineas): que `Entrada::listo` llame a `Enunciado { objetivo, criterios }.validar()` despues de sus comprobaciones propias (huecos, repetidos, controles) y traduzca `objetivo-vacio`/`demasiados-criterios` a dos `NoListo` nuevos.

<a id="a3-01"></a>

#### A3-01 · D4 · IMPORTANTE · pendiente [A3]

_`anotacion_ida_y_vuelta` solo mira `is_some()`; `desde_anotacion` pierde todo menos el título_

- id: A3-01
- regla: D4
- ubicacion: crates/tc2/src/alta.rs:1051-1058 (test `anotacion_ida_y_vuelta`) y crates/tc2/src/alta.rs:567-571 (`desde_anotacion`)
- modulo: alta
- severidad: IMPORTANTE
- motivo_severidad: D4 real en test. El test se llama «ida y vuelta» pero solo mira `is_some()`: un `desde_anotacion` que reconstruyera otra operacion (otro titulo, otro tipo) sigue en verde. Ademas el codigo que prueba aparenta reconstruir el alta y pierde todo menos el titulo. No sube a CRITICO: por F1-2 D2b `new` exige red y nunca queda pendiente, asi que el catalogo de produccion (`lib.rs:696-702`) no llega a reconstruir un `new` en la practica; si llegara, perderia datos.
- estado: confirmado
- grupo_seguimiento: A3

##### Evidencia

Test, `crates/tc2/src/alta.rs:1052-1055`:

```rust
    fn anotacion_ida_y_vuelta() {
        let a = Alta::con_fecha("con | barra", "2026-11-15");
        assert_eq!(a.anotacion(), "new|con | barra");
        assert!(desde_anotacion(&a.anotacion()).is_some());
```

Lo que prueba, `crates/tc2/src/alta.rs:355-357` y `:568-571`:

```rust
    fn anotacion(&self) -> String {
        format!("{NOMBRE}|{}", self.entrada.titulo)
    }
...
pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
    let titulo = a.strip_prefix(NOMBRE)?.strip_prefix('|')?;
    titulo_valido(titulo).then(|| Box::new(Alta::nueva(titulo)) as Box<dyn Operacion>)
}
```

Entrada con la que el codigo no hace lo que aparenta: `Alta::con_entrada(Entrada { tipo: "fix", criterios: vec!["c"], dependencias: vec!["TASK-001"], .. })`. Su anotacion es `new|<titulo>` y `desde_anotacion` devuelve un `Alta::nueva(titulo)`: tipo `feature`, sin objetivo, criterios, etiquetas, sprint, complejidad, dependencias ni `lote` (`alta.rs:55-66`). `importar::Importar`, la otra operacion que exige red por F1-2 D2b, ni siquiera esta en el catalogo (`lib.rs:697-701`); la entrada de `new` es la unica que queda y es la de F2-4, anterior a los campos de RF-01 (TASK-029).

Contraste: los tests de `ciclo` comparan la reconstruccion completa (`crates/tc2/src/ciclo/operacion.rs:474-478`: nombre, anotacion y `exige_red` de la reconstruida).

##### Mutante propuesto (pendiente de ejecutar, en serie)

- fichero:linea: `crates/tc2/src/alta.rs:570`
- cambio: `Box::new(Alta::nueva(titulo))` → `Box::new(Alta::nueva("otro"))`
- orden: `cargo test -p tc2 --lib alta::tests::anotacion_ida_y_vuelta`
- esperado si el Disfraz es real: `test result: ok. 1 passed` con el mutante puesto.
- contraste en rojo (codigo roto de verdad): `crates/tc2/src/alta.rs:570` → `None::<Box<dyn Operacion>>` (sin `then`) debe poner rojo el mismo test en `:1055`.
- test que deberia ponerse rojo: `alta::tests::anotacion_ida_y_vuelta` (si comparara `desde_anotacion(&a.anotacion()).map(|o| o.anotacion())` con `Some(a.anotacion())`).

##### Mutación

- Resultado: **confirmado** (el mutante sobrevive en verde y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` propio). Clon restaurado con `git checkout -- .` entre pasos.

##### Mutante

```diff
--- a/crates/tc2/src/alta.rs
+++ b/crates/tc2/src/alta.rs
@@ -570 +570 @@
-    titulo_valido(titulo).then(|| Box::new(Alta::nueva(titulo)) as Box<dyn Operacion>)
+    titulo_valido(titulo).then(|| Box::new(Alta::nueva("otro")) as Box<dyn Operacion>)
```

```text
cargo test -p tc2 --lib alta::tests::anotacion_ida_y_vuelta
test alta::tests::anotacion_ida_y_vuelta ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.00s

cargo test -p tc2 --lib alta
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 386 filtered out; finished in 0.04s

cargo test -p tc2 --test alta
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 58.34s
```

##### Control (código roto de verdad)

```diff
--- a/crates/tc2/src/alta.rs
+++ b/crates/tc2/src/alta.rs
@@ -570 +570 @@
-    titulo_valido(titulo).then(|| Box::new(Alta::nueva(titulo)) as Box<dyn Operacion>)
+    None::<Box<dyn Operacion>>
```

Orden: `cargo test -p tc2 --lib alta::tests::anotacion_ida_y_vuelta`

```text
test alta::tests::anotacion_ida_y_vuelta ... FAILED
thread 'alta::tests::anotacion_ida_y_vuelta' (31836) panicked at crates\tc2\src\alta.rs:1055:9:
assertion failed: desde_anotacion(&a.anotacion()).is_some()
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.02s
```

<a id="a3-02"></a>

#### A3-02 · D4 · IMPORTANTE · pendiente [A3]

_La rama de `es_esqueleto` del plan con unificador no la ejercita ningún test (se llama con `rutas = &[]`)_

- id: A3-02
- regla: D4
- ubicacion: crates/tc2/src/plan/aprobar.rs:117 (rama de `es_esqueleto` que usa `juegos_de_rondas`), sin test que la cubra: crates/tc2/src/plan/aprobar.rs:686-709 (`esqueleto_de_cualquier_juego_de_roles`), crates/tc2/tests/aprobar.rs:328-338 (`c3_el_esqueleto_no_se_aprueba`)
- modulo: plan (aprobar)
- severidad: IMPORTANTE
- motivo_severidad: D4 real en test: la parte de `es_esqueleto` que reconoce el esqueleto de una ronda con unificador (F4-6, roles elegidos por dominio) no la ejercita ningun test, y el que dice probarlo llama a `es_esqueleto` con `rutas = &[]`. Si esa linea se rompe, `approve` aprueba un `plan-final.md` sin redactar y la suite sigue en verde. No es CRITICO: la persona sigue firmando a mano (RR-04) y ve el plan.
- estado: confirmado
- grupo_seguimiento: A3

##### Evidencia

Codigo, `crates/tc2/src/plan/aprobar.rs:110-121`:

```rust
fn es_esqueleto(plan: &str, t: &Tarea, rutas: &[String]) -> bool {
    let plan = plan.replace("\r\n", "\n");
    let mut juegos: Vec<Vec<Rol>> = Vec::new();
    for n in 0..=5u8 {
        juegos.push(roles::seleccionar(n, &t.etiquetas));
        juegos.push(roles::seleccionar(n, &[]));
    }
    juegos.extend(juegos_de_rondas(rutas, &t.id));
```

El test que dice cubrirlo, `crates/tc2/src/plan/aprobar.rs:686-697`: todas las llamadas a `es_esqueleto` van con `&[]` (sin rondas) y con juegos que ya da `roles::seleccionar`; `juegos_de_rondas` se prueba aparte (`:708`), no a traves de `es_esqueleto`:

```rust
    /// El esqueleto de una ronda con unificador (cuatro roles) tambien se
    /// reconoce; un plan redactado, no.
    ...
        for r in [&cinco, &cuatro, &Vec::new()] {
            assert!(es_esqueleto(&p::plan_final("TASK-001", "Uno", r), &t, &[]));
        }
        assert!(!es_esqueleto("# Plan\n", &t, &[]));
```

Los otros dos usos (`aprobar.rs:620` y `tests/aprobar.rs:330-338`) son esqueletos de un rol, `seleccionar(1, ..)` = `[arquitectura]`, que reconoce el bucle de `:113-116` sin `juegos_de_rondas` (`grep -rn "es_esqueleto\|plan-esqueleto" crates` da solo esas lineas).

Por que la rama importa: con F4-6 `plan` toma los roles de la seleccion (`plan/operacion.rs:429`, `roles_de`), y S04 de F1-15 («`dominio=rendimiento` → mas-rol=rendimiento», `docs/adr/F1-15-...md:521`) suma rendimiento aunque haya un solo rol de escalera. Con carril `fix` (1 rol, `carril.rs:91`) y la etiqueta `wasm`, `plan` escribe el esqueleto de `[arquitectura, rendimiento]`; `roles::seleccionar(1, ..)` da solo `[arquitectura]` (`roles.rs:200-201`, sin condicionales por debajo de 2), asi que ese esqueleto solo lo reconoce `juegos_de_rondas`.

##### Mutante propuesto (pendiente de ejecutar, en serie)

- fichero:linea: `crates/tc2/src/plan/aprobar.rs:117`
- cambio: borrar la linea `juegos.extend(juegos_de_rondas(rutas, &t.id));` (queda `juegos_de_rondas` sin usar fuera de tests: si el aviso `dead_code` rompe la compilacion con `-D warnings`, poner `let _ = juegos_de_rondas(rutas, &t.id);` en su lugar)
- ordenes: `cargo test -p tc2 --lib plan::aprobar` y `cargo test -p tc2 --test aprobar`
- esperado si el Disfraz es real: las dos en verde con el mutante.
- contraste en rojo: borrar `:114` y `:115` (el bucle de `seleccionar`) debe poner rojo `plan::aprobar::tests::aplicar_aprueba_y_rechaza` (`:673`, `plan-esqueleto`).
- test que deberia ponerse rojo: `esqueleto_de_cualquier_juego_de_roles`, si llamara a `es_esqueleto` con un juego que `seleccionar` no da y las rutas de su ronda: p. ej. `[arquitectura, rendimiento]` con `peticion-brainstorm-arquitectura-1.md`, `peticion-brainstorm-rendimiento-1.md` y `peticion-unificador-1.md` (el juego `dos` de `:704-707`, `[arquitectura, riesgos]`, no vale: tambien lo da `seleccionar(2, &[])`).

##### Mutación

- Resultado: **confirmado** (el mutante sobrevive en verde y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` propio). Clon restaurado con `git checkout -- .` entre pasos.
- El mutante compila con dos avisos (`unused variable: rutas`, `function juegos_de_rondas is never used`), sin `-D warnings` en `cargo test`: no hizo falta la variante `let _ = ...`.

##### Mutante

```diff
--- a/crates/tc2/src/plan/aprobar.rs
+++ b/crates/tc2/src/plan/aprobar.rs
@@ -117 +116,0 @@
-    juegos.extend(juegos_de_rondas(rutas, &t.id));
```

```text
cargo test -p tc2 --lib plan::aprobar
warning: unused variable: `rutas`
test plan::aprobar::tests::esqueleto_de_cualquier_juego_de_roles ... ok
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 389 filtered out; finished in 0.58s

cargo test -p tc2 --test aprobar
warning: unused variable: `rutas`
warning: function `juegos_de_rondas` is never used
test c3_el_esqueleto_no_se_aprueba ... ok
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 307.56s
```

##### Control (código roto de verdad)

```diff
--- a/crates/tc2/src/plan/aprobar.rs
+++ b/crates/tc2/src/plan/aprobar.rs
@@ -114,2 +113,0 @@
-        juegos.push(roles::seleccionar(n, &t.etiquetas));
-        juegos.push(roles::seleccionar(n, &[]));
```

Orden: `cargo test -p tc2 --lib plan::aprobar::tests::aplicar_aprueba_y_rechaza`

```text
test plan::aprobar::tests::aplicar_aprueba_y_rechaza ... FAILED
thread 'plan::aprobar::tests::aplicar_aprueba_y_rechaza' (34896) panicked at crates\tc2\src\plan\aprobar.rs:671:9:
assertion `left == right` failed
  left: None
 right: Some("plan-esqueleto")
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.02s
```

### A4 — Git, rama de estado y toma

<a id="a4-01"></a>

#### A4-01 · D1 · IMPORTANTE divergente · pendiente [A4]

_Consulta de ramas de tarea locales y remotas duplicada y fallo del remoto tratado en cuatro sitios, dos siempre como `sinred`_

- id: A4-01
- regla: D1
- ubicacion: crates/tc2/src/git/rama.rs:88-110 (ramas_del_id) ~ crates/tc2/src/git/rama.rs:160-196 (ids_con_rama) ~ crates/tc2/src/git/remoto.rs:175-181 (fallo_fetch) ~ crates/tc2/src/git/tag.rs:100-114 (commit_remoto)
- modulo: git (rama, remoto, tag)
- severidad: IMPORTANTE (divergente)
- motivo_severidad: D1 real cuyas copias ya divergen en comportamiento. La consulta «ramas de tarea locales y remotas» (ls-remote --heads + branch --list + nombres) esta dos veces, y el tratamiento del fallo del remoto esta en cuatro sitios: dos lo clasifican (sinred / error local / denegada) y dos lo dan siempre como `sinred`. No tapa un fallo de datos ni de seguridad (cambia el resultado y el codigo de salida, 4 en vez de 8 o `ok`), asi que no sube a CRITICO.
- estado: confirmado (por lectura contra d82ea9d; las salidas de tc2 de abajo se derivan del codigo, no se ejecutaron: no se compila en esta fase)
- grupo_seguimiento: A4

##### Evidencia

Copia 1 — `crates/tc2/src/git/rama.rs:89-101` (ramas_del_id, usada por `comprobar_id`/`comprobar_id_salvo` y por `start`):

```rust
    let patron = format!("*/{}-*", id.to_lowercase());
    let remoto = correr(git, dir, op, Subcomando::LsRemote,
        &["--heads", REMOTO, &patron])?;          // 90-96
    if !remoto.exito() {
        return Err(Informe::nuevo(op, Resultado::Sinred)
            .con("remoto", REMOTO)
            .diagnostico(super::remoto::sin_red_remoto(op)));
    }
```

Copia 2 — `crates/tc2/src/git/rama.rs:162-179` (ids_con_rama):

```rust
    let remoto = if hay_remoto(git, &principal, op)? {
        let s = correr(git, &principal, op, Subcomando::LsRemote,
            &["--heads", REMOTO, PATRON_TAREAS])?;     // 163-169
        if !s.exito() {
            return Err(match clasificar_fallo_remoto(&s.stderr) {
                FalloRemoto::SinRed | FalloRemoto::RamaInexistente => {
                    Informe::nuevo(op, Resultado::Sinred)
                        .con("remoto", REMOTO)
                        .diagnostico(super::remoto::sin_red_remoto(op))
                }
                FalloRemoto::Local => super::remoto::error_local(op, &s.primera_linea_error()),
                otro => super::remoto::denegada(op, otro, &s.primera_linea_error()),
```

Copia 3 — `crates/tc2/src/git/remoto.rs:175-181` (fallo_fetch) y `:33-37` (sinred), de la que la copia 2 es literal con `sinred(op)` en linea:

```rust
fn fallo_fetch(op: &str, s: &SalidaGit) -> Informe {
    match clasificar_fallo_remoto(&s.stderr) {
        FalloRemoto::SinRed | FalloRemoto::RamaInexistente => sinred(op),
        FalloRemoto::Local => error_local(op, &s.primera_linea_error()),
        otro => denegada(op, otro, &s.primera_linea_error()),
    }
}
```

Copia 4 — `crates/tc2/src/git/tag.rs:110-114` (commit_remoto, igual que la copia 1):

```rust
    if !s.exito() {
        return Err(Informe::nuevo(op, Resultado::Sinred)
            .con("remoto", REMOTO)
            .diagnostico(super::remoto::sin_red_remoto(op)));
    }
```

Diff normalizado:
- copia 2 frente a copia 3: identicas salvo que `sinred(op)` esta escrito en linea (3 lineas de `remoto.rs:34-36`).
- copias 1 y 4 frente a 2 y 3: no llaman a `clasificar_fallo_remoto`; todo fallo de `ls-remote` es `sinred`. La copia 1 tampoco mira `hay_remoto` antes de `ls-remote` (la copia 2 si: rama.rs:162).

Entradas con las que ya se comportan distinto:
1. `origin` apuntando a una ruta que no es un repositorio. `git::fetch` da `fetch denegada remoto=origin motivo=remoto-no-existe` (codigo 8; lo fija el test `crates/tc2/tests/git_push.rs:116-131`). Con el mismo `origin`, `git::tag_anotado(&g, &clon, "v1", "HEAD", ..)` con el tag sin crear llega a `commit_remoto` y sale `tag-anotado sinred remoto=origin` (codigo 4), y `git::comprobar_id(&g, &clon, "TASK-007")` sale `comprobar-id sinred remoto=origin`: piden reintentar con red algo que reintentar no arregla. El propio modulo dice lo contrario en `remoto.rs:168-169`: «remoto inexistente o sin permiso → `denegada`».
2. Repo sin `origin`: `ids_con_rama` devuelve los IDs locales (`Ok`), mientras `comprobar_id` lanza `ls-remote --heads origin` igualmente y sale `sinred` (no hay `hay_remoto` en la copia 1).

Arreglo esperado (para la tarea de seguimiento, no se aplica aqui): una sola funcion «ramas de tarea» y una sola clasificacion del fallo (`fallo_fetch`) para `ls-remote`.

<a id="a4-02"></a>

#### A4-02 · D1 · IMPORTANTE divergente · pendiente [A4]

_Envoltorio «lanzar git y traducir el fallo» cuatro veces; `leer_blobs` usa la primera línea cruda de stderr_

- id: A4-02
- regla: D1
- ubicacion: crates/tc2/src/git/estado.rs:243-265 (con_indice) ~ crates/tc2/src/git/mod.rs:52-89 (correr + exigir) ~ crates/tc2/src/git/estado.rs:269-298 (con_entrada) ~ crates/tc2/src/git/estado.rs:135-150 (leer_blobs)
- modulo: git (mod, estado)
- severidad: IMPORTANTE (divergente)
- motivo_severidad: D1 real (el envoltorio «lanzar git; fallo al lanzar → error con `git=<sub>`; codigo != 0 → error con `git_fallo`» esta cuatro veces) y una copia ya diverge: `leer_blobs` arma el diagnostico con la primera linea cruda de stderr en vez de `primera_linea_error()`. Afecta solo al texto del diagnostico, no a datos ni a seguridad.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A4

##### Evidencia

Original — `crates/tc2/src/git/mod.rs:59-63` y `:81-88`:

```rust
    git.ejecutar(dir, sub, args).map_err(|e| {
        Informe::nuevo(op, Resultado::Error)
            .con("git", sub.nombre())
            .diagnostico(e.mensaje())
    })
...
    let s = correr(git, dir, op, sub, args)?;
    if s.exito() {
        Ok(s.stdout)
    } else {
        Err(Informe::nuevo(op, Resultado::Error)
            .con("git", sub.nombre())
            .diagnostico(git_fallo(sub, &s)))
    }
```

Copia — `crates/tc2/src/git/estado.rs:251-264` (con_indice):

```rust
    let s = git
        .ejecutar_con_indice(dir, sub, args, indice)
        .map_err(|e| {
            Informe::nuevo(op, Resultado::Error)
                .con("git", sub.nombre())
                .diagnostico(e.mensaje())
        })?;
    if s.exito() {
        Ok(s.stdout)
    } else {
        Err(Informe::nuevo(op, Resultado::Error)
            .con("git", sub.nombre())
            .diagnostico(git_fallo(sub, &s)))
```

Copia — `crates/tc2/src/git/estado.rs:281-296` (con_entrada): el mismo `map_err` (281-285) y la misma rama de error (294-296), con un `SalidaGit` de paso para poder llamar a `git_fallo` (289-293).

Copia divergente — `crates/tc2/src/git/estado.rs:137-149` (leer_blobs):

```rust
            .map_err(|e| {
                Informe::nuevo(op, Resultado::Error)
                    .con("git", Subcomando::CatFile.nombre())
                    .diagnostico(e.mensaje())
            })?;
        if s.codigo != 0 {
            return Err(Informe::nuevo(op, Resultado::Error)
                .con("git", Subcomando::CatFile.nombre())
                .diagnostico(
                    Mensaje::nuevo(&txt::GIT_FALLO)
                        .con("subcomando", Subcomando::CatFile.nombre())
                        .con("error", s.stderr.lines().next().unwrap_or_default()),
```

frente a `git_fallo` (`mod.rs:67-71`), que usa `s.primera_linea_error()` (`puerto.rs:151-158`: recorta, salta las lineas vacias y, sin ninguna, `"sin mensaje"`).

Diff normalizado: `con_indice` = `correr`+`exigir` cambiando `ejecutar` por `ejecutar_con_indice`; `con_entrada` = lo mismo con `ejecutar_con_entrada`/`ejecutar_con_indice_y_entrada`; `leer_blobs` = lo mismo escrito en linea, salvo `.con("error", ...)`.

Entrada con la que divergen: un `cat-file --batch` que termina con codigo != 0 y stderr vacio (proceso muerto) o que empieza por una linea en blanco (`"\nfatal: x"`). `leer_blobs` da `error=` vacio; cualquier otra operacion por `exigir`/`con_indice`/`con_entrada` daria `sin mensaje` o `fatal: x` con el mismo stderr.

<a id="a4-03"></a>

#### A4-03 · D1 · IMPORTANTE · pendiente [A4]

_19 líneas idénticas en el punto único de escritura (`intento`) de la transacción_

- id: A4-03
- regla: D1
- ubicacion: crates/tc2/src/estado/transaccion.rs:353-371 (intento) ~ crates/tc2/src/estado/transaccion.rs:533-551 (sin_red)
- modulo: estado (transaccion)
- severidad: IMPORTANTE
- motivo_severidad: D1 real de 19 lineas identicas en el punto unico de escritura (ADR F1-2): aplicar la operacion, comprobar el choque por mayusculas y traducir `Fallo` a informe. Las copias aun no divergen; un cambio en una (p. ej. un motivo nuevo de `Fallo`) se queda sin la otra. No tapa fallo de datos.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A4

##### Evidencia

`crates/tc2/src/estado/transaccion.rs:353-362` (intento; sigue igual hasta 371):

```rust
    match op.aplicar(&vista) {
        Ok(cambios) => {
            if let Some(i) = duplicada(&vista.rutas(), &cambios, op) {
                return Ok(Intento::Hecho(i));
            }
            vista.superponer(cambios);
        }
        Err(Fallo::Rechazada {
            motivo,
            diagnostico,
```

`crates/tc2/src/estado/transaccion.rs:533-542` (sin_red; sigue igual hasta 551):

```rust
    match op.aplicar(&vista) {
        Ok(cambios) => {
            if let Some(i) = duplicada(&vista.rutas(), &cambios, op) {
                return Ok(Intento::Hecho(i));
            }
            vista.superponer(cambios);
        }
        Err(Fallo::Rechazada {
            motivo,
            diagnostico,
```

Diff normalizado de 353-371 frente a 533-551: ninguna diferencia (mismos 19 lineas, misma indentacion; la variable es `vista` en los dos). El candidato del pase 0 (`intento` 353-371 ~ `sin_red` 533-551, 123 tokens) queda confirmado.

Reducida: `fn aplicar_en(vista: &mut Vista, op: &dyn Operacion) -> Result<Option<Intento>, Informe>` llamada desde los dos sitios.

<a id="a4-04"></a>

#### A4-04 · D1 · IMPORTANTE · pendiente [A4]

_Informe de un `publicar` fallido copiado en `tag` y `remoto`_

- id: A4-04
- regla: D1
- ubicacion: crates/tc2/src/git/tag.rs:81-91 ~ crates/tc2/src/git/remoto.rs:516-526 (y la misma idea en remoto.rs:379-385 con push_fallo 162-166)
- modulo: git (tag, remoto)
- severidad: IMPORTANTE
- motivo_severidad: D1 real: el informe de un `publicar` que no salio (resultado, `motivo` solo en Denegada/Error, `intentos`, diagnostico con `ultimo` y `que_hacer`) esta copiado en dos operaciones con 11 lineas cada una; solo cambian la clave (`tag`/`rama`) y el texto del catalogo. Aun no divergen. No tapa fallo de datos.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A4

##### Evidencia

`crates/tc2/src/git/tag.rs:82-91`:

```rust
            let que_hacer = super::remoto::que_hacer_al_publicar(p.resultado);
            let mut i = Informe::nuevo(OP, p.resultado).con("tag", tag);
            if matches!(p.resultado, Resultado::Denegada | Resultado::Error) {
                i = i.con("motivo", p.motivo);
            }
            return Ok(i.con("intentos", p.intentos.to_string()).diagnostico(
                Mensaje::nuevo(&txt::TAG_SIN_PUBLICAR)
                    .con("ultimo", p.ultimo)
                    .con_mensaje("que_hacer", que_hacer),
            ));
```

`crates/tc2/src/git/remoto.rs:517-526`:

```rust
                let que_hacer = que_hacer_al_publicar(otro);
                let mut i = Informe::nuevo(OP, otro).con("rama", rama.as_str());
                if matches!(otro, Resultado::Denegada | Resultado::Error) {
                    i = i.con("motivo", p.motivo);
                }
                return Ok(i.con("intentos", intentos.to_string()).diagnostico(
                    Mensaje::nuevo(&txt::PUSH_FALLO_ACTUALIZADA_EN_LOCAL)
                        .con("ultimo", ultimo)
                        .con_mensaje("que_hacer", que_hacer),
                ));
```

Diff normalizado: `p.resultado` ↔ `otro`, `("tag", tag)` ↔ `("rama", rama)`, `p.intentos`/`p.ultimo` ↔ `intentos`/`ultimo` (desestructurados en 507), `TAG_SIN_PUBLICAR` ↔ `PUSH_FALLO_ACTUALIZADA_EN_LOCAL`. Lo demas, igual.

Reducida: `fn informe_publicacion(op, p: Publicacion, clave: (&str, &str), texto: &Texto) -> Informe` en remoto.rs, usada por `tag_paso`, `actualizar` y `push_paso`.

<a id="a4-05"></a>

#### A4-05 · D1 · IMPORTANTE · pendiente [A4]

_`hay_remoto` y `url_origin` lanzan el mismo `ls-remote --get-url origin`_

- id: A4-05
- regla: D1
- ubicacion: crates/tc2/src/git/estado.rs:451-460 (hay_remoto) ~ crates/tc2/src/git/estado.rs:463-473 (url_origin)
- modulo: git (estado)
- severidad: IMPORTANTE
- motivo_severidad: D1 real: dos funciones seguidas lanzan el mismo `ls-remote --get-url origin` con la misma condicion; `hay_remoto` es `url_origin(..)?.is_some()`. Aun no divergen (si se cambia la condicion en una, `hay_remoto` y la URL que se detecta dejarian de casar). No tapa fallo de datos.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A4

##### Evidencia

`crates/tc2/src/git/estado.rs:451-460`:

```rust
pub(crate) fn hay_remoto(git: &Git, dir: &Path, op: &str) -> Paso<bool> {
    let s = correr(
        git,
        dir,
        op,
        Subcomando::LsRemote,
        &["--get-url", super::REMOTO],
    )?;
    Ok(s.exito() && s.stdout.trim() != super::REMOTO && !s.stdout.trim().is_empty())
}
```

`crates/tc2/src/git/estado.rs:463-473`:

```rust
pub(crate) fn url_origin(git: &Git, dir: &Path, op: &str) -> Paso<Option<String>> {
    let s = correr(
        git,
        dir,
        op,
        Subcomando::LsRemote,
        &["--get-url", super::REMOTO],
    )?;
    let url = s.stdout.trim();
    Ok((s.exito() && url != super::REMOTO && !url.is_empty()).then(|| url.to_owned()))
}
```

Diff normalizado: el cuerpo es el mismo; `hay_remoto` devuelve el booleano y `url_origin` la URL bajo ese booleano.

Reducida (1 linea): `pub(crate) fn hay_remoto(git, dir, op) -> Paso<bool> { Ok(url_origin(git, dir, op)?.is_some()) }`.

<a id="a4-06"></a>

#### A4-06 · D1 · IMPORTANTE · pendiente [A4]

_`crear` y `reanudar` del worktree de tarea repiten arranque y bloque de carpeta ocupada_

- id: A4-06
- regla: D1
- ubicacion: crates/tc2/src/git/rama.rs:318-380 (crear) ~ crates/tc2/src/git/rama.rs:388-456 (reanudar_paso)
- modulo: git (rama)
- severidad: IMPORTANTE
- motivo_severidad: D1 real: las dos operaciones de worktree de tarea (casos b y c de F1-3) repiten el mismo arranque (validar, principal, ruta del worktree, cierre de rechazo; 9 lineas seguidas) y el mismo bloque de «carpeta ocupada + excluir + ruta UTF-8 + worktree add». Aun no divergen. No tapa fallo de datos.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A4

##### Evidencia

`crates/tc2/src/git/rama.rs:324-331` (crear):

```rust
    let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, OP))?;
    let wt = principal.join(CARPETA_WORKTREES).join(id);
    let rel = worktree_relativo(id);
    let rechazo = |motivo: &str| {
        Informe::nuevo(OP, Resultado::Rechazada)
            .con("rama", rama.as_str())
            .con("motivo", motivo)
    };
```

`crates/tc2/src/git/rama.rs:391-398` (reanudar_paso): las mismas 8 lineas, identicas. Las dos van precedidas de `let rama = rama_valida(OP, id, tipo, slug)?;` (320 y 390).

Segundo bloque, `rama.rs:357-359` + `367-368` (crear):

```rust
    if wt.exists() {
        return Ok(rechazo("carpeta-ocupada")
            .diagnostico(Mensaje::nuevo(&txt::CARPETA_OCUPADA).con("ruta", rel.as_str())));
    ...
    excluir_worktrees(git, &principal, OP)?;
    let wt_txt = ruta_texto(OP, &wt)?;
```

y `rama.rs:425-430` (reanudar_paso), mismas lineas con un nivel mas de sangria.

Diff normalizado: arranque, identico (salvo el `if base...` de 321-323, solo en `crear`); bloque del worktree, identico salvo los argumentos de `worktree add` (`-q --no-track -b <rama> <wt> <commit>` frente a `-q <wt> <rama>`).

<a id="a4-07"></a>

#### A4-07 · D1 · IMPORTANTE · pendiente [A4]

_Analizador de `git status --porcelain -z` escrito dos veces_

- id: A4-07
- regla: D1
- ubicacion: crates/tc2/src/git/guardar.rs:158-176 (entradas_con_origen) ~ crates/tc2/src/git/mod.rs:216-228 (entradas_status)
- modulo: git (guardar, mod)
- severidad: IMPORTANTE
- motivo_severidad: D1 real: el analizador de `git status --porcelain -z` (XY, ruta, salto del segundo campo de R/C) esta dos veces; `entradas_con_origen` es `entradas_status` mas una linea. Aun no divergen; un arreglo del formato en uno no llega al otro. No tapa fallo de datos.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A4

##### Evidencia

`crates/tc2/src/git/mod.rs:217-226`:

```rust
    let mut v = Vec::new();
    let mut campos = salida.split('\0').filter(|c| !c.is_empty());
    while let Some(c) = campos.next() {
        let xy = c.get(..2).unwrap_or(c).to_owned();
        let ruta = c.get(3..).unwrap_or("").to_owned();
        if xy.starts_with('R') || xy.starts_with('C') {
            campos.next();
        }
        v.push((xy, ruta));
    }
```

`crates/tc2/src/git/guardar.rs:159-173`:

```rust
    let mut v = Vec::new();
    let mut campos = salida.split('\0').filter(|c| !c.is_empty());
    while let Some(c) = campos.next() {
        let xy = c.get(..2).unwrap_or(c).to_owned();
        let ruta = c.get(3..).unwrap_or("").to_owned();
        // La copia (`C`) tambien trae el origen, pero el origen sigue ahi.
        let origen = if xy.starts_with('R') || xy.starts_with('C') {
            campos.next()
        } else { None };                                   // 167-169
        if let (Some(o), true) = (origen, xy.starts_with('R')) {
            v.push(("D ".to_owned(), o.to_owned()));
        }
        v.push((xy, ruta));
```

Diff normalizado: identicos salvo que la copia de guardar.rs guarda el campo saltado y, si es un renombrado, anade `("D ", origen)`.

Reducida: `entradas_status` = `entradas_con_origen` sin las entradas de origen (o un parametro `con_origen: bool`), en un solo sitio (mod.rs).

<a id="a4-08"></a>

#### A4-08 · D1 · IMPORTANTE · pendiente [A4]

_Ayudantes de los tests de estado (`rapida`, `escribir_estado`, `rama_en`) copiados entre ficheros_

- id: A4-08
- regla: D1
- ubicacion: crates/tc2/tests/estado_bordes.rs:19-53 ~ crates/tc2/tests/estado_casos.rs:19-44 y :124-129 ~ crates/tc2/tests/estado_concurrencia.rs:216-220
- modulo: tests de estado (estado_bordes, estado_casos, estado_concurrencia)
- severidad: IMPORTANTE
- motivo_severidad: D1 real en tests: 26 lineas de ayudantes (`rapida`, `escribir_estado`, `rama_en`) copiadas byte a byte entre dos ficheros, y el «repo sin remoto» armado en tres. Aun no divergen (salvo el tipo de retorno escrito de dos formas). No tapa fallo de datos.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A4

##### Evidencia

`crates/tc2/tests/estado_bordes.rs:19-33`:

```rust
fn rapida() -> Politica {
    Politica {
        intentos: 3,
        base: Duration::from_millis(1),
        espera_cerrojo: Duration::from_secs(30),
    }
}

fn escribir_estado(g: &Git, dir: &Path, op: &dyn Operacion) -> Informe {
    let mut c = Contexto::nuevo(g, dir, &catalogo, None);
```

`crates/tc2/tests/estado_casos.rs:19-33`: las mismas lineas; la unica diferencia es el retorno `-> tc2::contrato::Informe` (linea 27). `rama_en` (estado_bordes.rs:35-44 y estado_casos.rs:35-44) es identica (similitud de cuerpos 1.00 en el pase 0).

Repo sin remoto, tres veces:
- `estado_bordes.rs:46-53` (`repo_sin_remoto`): `init -q -b develop`, `README.md`, `add`, `commit -q -m inicial`.
- `estado_casos.rs:124-129`: las mismas cuatro ordenes en linea.
- `estado_concurrencia.rs:216-220`: las mismas cuatro ordenes en linea.

Diff normalizado: ninguno en los ayudantes salvo la ruta del tipo `Informe`. El sitio comun natural es `tests/comun` (dueno A8); la tarea de seguimiento lo coordina con A8.

<a id="a4-10"></a>

#### A4-10 · D4 · IMPORTANTE · pendiente [A4]

_`es_prohibido` no ve opciones cortas agrupadas: `push -qf`, `branch -qD` y `worktree remove -ff` pasan la guarda y git las ejecuta_

- id: A4-10
- regla: D4 (en codigo; con D2 frente a ADR F1-4, decision 3)
- ubicacion: crates/tc2/src/git/puerto.rs:613-646 (es_prohibido), con su uso en puerto.rs:560-595 (comprobar_argumentos)
- modulo: git (puerto)
- severidad: IMPORTANTE
- normalizacion: consolidador TASK-138, 2026-11-22: regla normalizada a D4 (el apartamiento de F1-4 decision 3 va dentro del mismo caso). Se valoro CRITICO por ser una defensa de RN-05, y se deja en IMPORTANTE: hoy ninguna operacion pasa esos argumentos, asi que no tapa un fallo ya existente.
- motivo_severidad: el guardian de argumentos prohibidos aparenta cerrar `push -f`, `branch -D`, `worktree remove -f`... y no lo hace con opciones cortas agrupadas, que git acepta. Hoy ninguna operacion de la capa pasa esos argumentos, asi que no tapa un fallo ya existente (no CRITICO), pero la defensa de RN-05 que documentan el modulo y el ADR no existe para esa forma.
- estado: confirmado (lectura de tc2 contra d82ea9d + git 2.56.0.windows.4 ejecutado en un repo temporal, ya borrado). No se ejecuto tc2 (no se compila en esta fase).
- grupo_seguimiento: A4

##### Evidencia

Lo que promete el ADR — `docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md:418-425` (decision 3): prohibidas `push --force` (y `-f`...), borrar ramas, mover ramas a la fuerza...; «La PoC lo comprueba con una **lista permitida** de subcomandos y argumentos, no con una lista de cadenas prohibidas (H9).»

Lo que hace el codigo — lista de cadenas prohibidas comparadas enteras, `crates/tc2/src/git/puerto.rs:613-628`:

```rust
fn es_prohibido(sub: Subcomando, a: &str) -> bool {
    if a.starts_with("--force") {
        return true;
    }
    match sub {
        Subcomando::Push => {
            matches!(
                a,
                "-f" | "-d" | "--delete" | "--mirror" | "--prune" | "--all"
            ) || a.starts_with('+')
                || a.starts_with(':')
        }
        Subcomando::Branch => matches!(
            a,
            "-f" | "-d" | "-D" | "-m" | "-M" | "-c" | "-C" | "--delete" | "--move" | "--copy"
```

y `Subcomando::Worktree => a == "-f"` (puerto.rs:629). El comentario de puerto.rs:555-556 lo dice: «Se mira cada argumento entero, no subcadenas (H9)».

Entrada con la que no hace su trabajo: `Git::nuevo().ejecutar(dir, Subcomando::Push, &["-qf", "origin", "r"])`. `-qf` no es ninguna de las cadenas de la lista ni empieza por `--force`, `+` o `:`, asi que `comprobar_argumentos` devuelve `Ok(())` y se lanza `git push -qf origin r`. Igual con `Branch` y `["-qD", "x"]`, y con `Worktree` y `["remove", "-ff", "w"]`.

Que git las ejecuta (comprobado con git 2.56.0.windows.4 en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-138\a4-git`, borrado al terminar; configuracion aislada con `GIT_CONFIG_NOSYSTEM=1` y `GIT_CONFIG_GLOBAL=/dev/null`):

```text
--- push -qf (agrupado):
codigo=0
remoto ahora: 61861c2f78b822f0ec8d582e96d82c8ec3aea452 antes: b15e45b9266144070c07a2fe5be3491324636435
--- branch -qD:
codigo=0
0
```

(`push -qf` publico un commit reescrito con `--amend` encima de uno que no desciende: push forzado; `branch -qD x` borro la rama, `git branch --list x | wc -l` = 0.)

El test `rechaza_cada_argumento_prohibido_sin_lanzar_git` (puerto.rs:759-846) no tiene ninguna forma agrupada, asi que no lo detecta. Comprobacion propuesta para la tarea de seguimiento (necesita cargo; no ejecutada aqui): anadir a esa tabla `(Subcomando::Push, &["-qf", "origin", "r"])`; hoy daria `Err(ErrorGit::Lanzar(_))` con `git_sin_binario()` y el test se pondria rojo.

### A5 — Selección, inventario, recursos y doctor

<a id="a5-01"></a>

#### A5-01 · D1 · IMPORTANTE divergente · pendiente [A5]

_Leer skills de una carpeta, en `inventario` y `catalogo`, ya divergente: el catálogo descarta en silencio lo ilegible (contra C8)_

- id: A5-01
- regla: D1 (Duplicacion)
- ubicacion: crates/tc2/src/seleccion/catalogo.rs:232 (y crates/tc2/src/inventario.rs:423)
- modulo: seleccion::catalogo / inventario (lectura de skills de una carpeta)
- severidad: IMPORTANTE (divergente)
- normalizacion: consolidador TASK-138, 2026-11-22: se marca `divergente`: su propio motivo dice que las copias ya divergen en comportamiento (plan, decision 5). Va primera en su grupo.
- marca: divergente
- motivo_severidad: D1 real (regla 4D: Duplicacion real es IMPORTANTE como minimo). Las dos copias ya divergen en comportamiento: `inventario::leer_carpetas` deja incidencia (`no-se-lee`, `nombre-no-utf8`, `nombre-distinto`) de lo que no puede leer (C8: "Lo que no se puede leer no se da por instalado ni por ausente", inventario.rs:28-30) y `catalogo::de_carpeta` lo descarta en silencio. No sube a CRITICO (decision 5 del plan): no tapa un fallo de datos ni de seguridad, solo hace que en `tc2 revisoras --plugin` (revisor en CI) una skill ilegible desaparezca sin aviso.
- estado: confirmado
- grupo_seguimiento: A5 — Seleccion, inventario, recursos y doctor (abstraccion: lectura de skills de una carpeta)

##### Que se repite

Las dos funciones recorren `<dir>/<n>/SKILL.md` ordenado, leen la cabecera con tope `MAX_CABECERA`, calculan el id como `prefijo + (name: del frontmatter | nombre de la carpeta)` y empujan una `Entrada { tipo: Skill, id, origen }`. El modulo del catalogo dice apoyarse en el inventario "sin cambiarlo" (catalogo.rs:8-9), pero reimplementa la lectura en vez de reutilizarla. Ningun ADR impone la copia: F1-15 decision 2 dice que el inventario es "la unica fuente de lo que existe" (ADR F1-15:418) y la adenda F6-3 que el catalogo va "sobre el inventario sin cambiarlo" (ADR F1-15:767). Ademas `de_carpeta` no es solo catalogo: con `--plugin`, sus `entradas` SON el inventario (seleccion/orden.rs:140-141, `entradas.extend(l.entradas)`). Tercera copia del calculo del id, sobre blobs de git: catalogo.rs:332-341 (`de_base`). Y una cuarta mini-copia: `catalogo::cabecera` (catalogo.rs:220-228) frente a `inventario::cabecera` (inventario.rs:365-371), mismo `File::open(p)?.take(MAX_CABECERA).read_to_end` + `from_utf8_lossy`, una con `Option` y otra con `io::Result`.

##### Evidencia

Fragmento 1 (catalogo, 10 lineas):
```text
crates/tc2/src/seleccion/catalogo.rs:238:     for d in hijos {
crates/tc2/src/seleccion/catalogo.rs:239:         let f = d.join("SKILL.md");
crates/tc2/src/seleccion/catalogo.rs:240:         let (Some(carpeta), Some(texto)) = (
crates/tc2/src/seleccion/catalogo.rs:241:             d.file_name().and_then(|n| n.to_str()).map(str::to_owned),
crates/tc2/src/seleccion/catalogo.rs:242:             f.is_file().then(|| cabecera(&f)).flatten(),
crates/tc2/src/seleccion/catalogo.rs:243:         ) else {
crates/tc2/src/seleccion/catalogo.rs:244:             continue;
crates/tc2/src/seleccion/catalogo.rs:245:         };
crates/tc2/src/seleccion/catalogo.rs:246:         let nombre = inventario::nombre_del_frontmatter(&texto).unwrap_or(carpeta);
crates/tc2/src/seleccion/catalogo.rs:247:         let id = format!("{prefijo}{nombre}");
```

Fragmento 2 (inventario, 10 lineas, extracto de 425-471):
```text
crates/tc2/src/inventario.rs:425:     for d in hijos(skills, &format!("{prefijo}skills"), a) {
crates/tc2/src/inventario.rs:426:         let f = d.join("SKILL.md");
crates/tc2/src/inventario.rs:427:         if d.is_dir()
crates/tc2/src/inventario.rs:428:             && f.is_file()
crates/tc2/src/inventario.rs:429:             && let Some(n) = nombre_utf8(d.file_name(), prefijo, &d, a)
crates/tc2/src/inventario.rs:443:         let nombre = match cabecera(&f) {
crates/tc2/src/inventario.rs:444:             Ok(c) => nombre_del_frontmatter(&c),
crates/tc2/src/inventario.rs:446:                 a.ilegibles.push(Incidencia {
crates/tc2/src/inventario.rs:448:                     motivo: "no-se-lee",
crates/tc2/src/inventario.rs:469:             id: format!("{prefijo}{id}"),
```

Diff normalizado: mismo recorrido y mismo id; difieren en el tratamiento de lo ilegible:
- nombre de carpeta no UTF-8: catalogo `to_str()` -> `None` -> `continue` (silencio); inventario `nombre_utf8` -> incidencia `nombre-no-utf8` (inventario.rs:413-417).
- `SKILL.md` que no se lee: catalogo `cabecera(&f)` -> `None` -> `continue` (silencio); inventario -> incidencia `no-se-lee` (inventario.rs:445-451).
- carpeta `skills` que no se lista: catalogo `read_dir(...).unwrap_or_default()` (catalogo.rs:234-236, vacio en silencio); inventario `hijos` -> incidencia `no-se-lee` (inventario.rs:385-396).
- `name:` distinto de la carpeta: inventario aviso `nombre-distinto` (inventario.rs:456-462); catalogo nada.

Entrada que lo prueba (lectura, sin ejecutar): `tc2 revisoras --rutas r.txt --plugin taskcode-plugin=<dir>` con `<dir>/skills/rust-wasm-reviewer/SKILL.md` sin permiso de lectura (o con un nombre de carpeta no UTF-8). Por `de_carpeta` la revisora no entra ni en el inventario ni en el catalogo y no queda rastro (no hay `ilegibles` en `Fuentes`, seleccion/orden.rs:90-98); el mismo arbol leido por `inventario::leer` daria `no-se-lee` y `doctor --recursos` lo diria `no-comprobado`.

##### Arreglo sugerido

Un unico lector de "skills de una carpeta" en `inventario` que devuelva entradas, incidencias y el texto de la cabecera; `catalogo::de_carpeta` lo usa y solo anade `recurso_de`. `de_base` reutiliza el calculo del id.

<a id="a5-02"></a>

#### A5-02 · D1 · IMPORTANTE divergente · pendiente [A5]

_Resolución de la carpeta instalada del plugin con reglas distintas (el catálogo no exige el `scope` del proyecto)_

- id: A5-02
- regla: D1 (Duplicacion)
- ubicacion: crates/tc2/src/seleccion/catalogo.rs:265 (y crates/tc2/src/inventario.rs:815)
- modulo: seleccion::catalogo / inventario (que instalacion de `installed_plugins.json` vale para un plugin)
- severidad: IMPORTANTE (divergente)
- normalizacion: consolidador TASK-138, 2026-11-22: se marca `divergente`: las dos copias resuelven la carpeta del plugin con reglas distintas (plan, decision 5).
- marca: divergente
- motivo_severidad: D1 real (IMPORTANTE como minimo). Las dos copias resuelven "la carpeta instalada del plugin habilitado" con reglas distintas: el inventario exige el `scope` de este proyecto (`aplica`), el catalogo no. No es CRITICO (decision 5 del plan): no tapa fallo de datos ni de seguridad; puede leer las revisoras de una instalacion distinta de la que el inventario dio por habilitada.
- estado: confirmado
- grupo_seguimiento: A5 — Seleccion, inventario, recursos y doctor (abstraccion: lectura de skills de una carpeta / instalacion del plugin)

##### Que se repite

`inventario::leer_plugins` ya elige, por cada `plugins.<clave>` de `installed_plugins.json`, la instalacion que vale y comprueba que su `installPath` es absoluto y existe; pero solo guarda `Plugin { nombre, clave, version }` (inventario.rs:584-588), sin la carpeta. `catalogo::de_instalados` vuelve a leer el mismo fichero (`inventario::leer_instalados`, catalogo.rs:266) y vuelve a elegir instalacion con su propia regla. Ningun ADR pide la segunda lectura (F1-15 decision 2: el inventario es "la unica fuente de lo que existe", ADR F1-15:418).

##### Evidencia

Fragmento 1 (catalogo, 10 lineas):
```text
crates/tc2/src/seleccion/catalogo.rs:278:         let ruta = |x: &Value| {
crates/tc2/src/seleccion/catalogo.rs:279:             x.get("installPath")
crates/tc2/src/seleccion/catalogo.rs:280:                 .and_then(Value::as_str)
crates/tc2/src/seleccion/catalogo.rs:281:                 .map(PathBuf::from)
crates/tc2/src/seleccion/catalogo.rs:282:                 .filter(|d| d.is_absolute() && d.is_dir())
crates/tc2/src/seleccion/catalogo.rs:283:         };
crates/tc2/src/seleccion/catalogo.rs:284:         let misma = lista.iter().find(|x| {
crates/tc2/src/seleccion/catalogo.rs:285:             p.version.is_some() && x.get("version").and_then(Value::as_str) == p.version.as_deref()
crates/tc2/src/seleccion/catalogo.rs:286:         });
crates/tc2/src/seleccion/catalogo.rs:287:         let Some(dir) = misma.and_then(ruta).or_else(|| lista.iter().find_map(ruta)) else {
```

Fragmento 2 (inventario, 10 lineas, extracto de 867-898):
```text
crates/tc2/src/inventario.rs:867:         let Some(x) = lista.iter().find(|x| aplica(x, raiz)) else {
crates/tc2/src/inventario.rs:868:             continue;
crates/tc2/src/inventario.rs:870:         let Some(install) = x.get("installPath").and_then(Value::as_str) else {
crates/tc2/src/inventario.rs:878:         let dir = PathBuf::from(install);
crates/tc2/src/inventario.rs:883:         if dir.is_relative() {
crates/tc2/src/inventario.rs:891:         if !dir.is_dir() {
crates/tc2/src/inventario.rs:899:         cargar_plugin(
crates/tc2/src/inventario.rs:900:             &dir,
crates/tc2/src/inventario.rs:903:             x.get("version").and_then(Value::as_str),
```

`aplica` (inventario.rs:804-812): de usuario o sin `scope`, siempre; de proyecto o local, solo si `projectPath` es este proyecto.

Diff normalizado: los dos toman `installPath` absoluto y existente de una entrada de la lista; el inventario la elige por `aplica(x, raiz)` (la primera que vale para este proyecto); el catalogo por `version` igual a la del inventario y, si no, la primera con carpeta, sin mirar `scope` ni `projectPath`.

Entrada que lo prueba (lectura, sin ejecutar): `installed_plugins.json` con
`{"version": 2, "plugins": {"p@mk": [{"scope": "project", "projectPath": "/otro", "installPath": "/A"}, {"scope": "user", "installPath": "/B"}]}}`, sin `version` en las entradas ni en `/B/.claude-plugin/plugin.json`, y `/A` y `/B` existentes. El inventario carga `/B` (la de usuario; `p.version = None`). En el catalogo `misma` es `None` (catalogo.rs:285 exige `p.version.is_some()`) y `lista.iter().find_map(ruta)` devuelve `/A`, la instalacion de otro proyecto: las revisoras se leen de una carpeta que el inventario no habilito. Con las dos entradas a la misma `version`, `misma` tambien elige `/A` (la primera con esa version).

##### Arreglo sugerido

Guardar en `inventario::Plugin` la carpeta elegida (`dir`) y que `de_instalados` la use, sin releer `installed_plugins.json`.

<a id="a5-03"></a>

#### A5-03 · D2 · IMPORTANTE · pendiente [A5]

_Con revisora de dominio declarada y no instalada, sale `no-instalado` en vez del aviso `dominio-sin-revisora` que exige F1-15 (S11)_

- id: A5-03
- regla: D2 (Desviacion)
- ubicacion: crates/tc2/src/seleccion/mod.rs:344 (atomo `dominio-con-revisora`; `revisoras_de` en mod.rs:260)
- modulo: seleccion (S10 y S11 de `tabla-seleccion`)
- severidad: IMPORTANTE
- motivo_severidad: D2 que cambia el comportamiento (regla 4D: MENOR salvo que cambie comportamiento). Con una revisora de dominio declarada pero no instalada, la seleccion no da el aviso `dominio-sin-revisora` que el ADR exige y que F8-6 cuenta como escapado; da `no-instalado`. No es CRITICO: no tapa un fallo de datos ni de seguridad (la persona ve los avisos en el check), pero la medicion de escapados de F1-15 decision 12 pierde el caso.
- estado: confirmado (por lectura; D2 no requiere mutacion)
- grupo_seguimiento: A5 — Seleccion, inventario, recursos y doctor

##### ADR

```text
docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:527:   | S10 | `revision` | `dominio-con-revisora` | `revisora-del-dominio` | — | por cada dominio detectado con revisora declarada e instalada, esa revisora, aunque las rutas no la pidan |
docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:551:   por dominio (S10): si `dominios()` da un dominio y hay una revisora
docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:552:   instalada que lo declara en sus metadatos, entra aunque las rutas no la
docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:553:   pidan; si no la hay, va la genérica (siempre la del plugin: una skill del
docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:554:   proyecto con `fallback: true` no la sustituye) y la tarea lleva el aviso
docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:555:   `dominio-sin-revisora` (S11), que F8-6 cuenta.
```

La adenda F4-6 (ADR F1-15:891-982) no cambia S10 ni S11 (solo dice que se evaluan una vez por dominio).

##### Codigo que se aparta

```text
crates/tc2/src/seleccion/mod.rs:260:     fn revisoras_de(&self, d: &str) -> Vec<String> {
crates/tc2/src/seleccion/mod.rs:261:         self.cat
crates/tc2/src/seleccion/mod.rs:262:             .revisoras()
crates/tc2/src/seleccion/mod.rs:263:             .iter()
crates/tc2/src/seleccion/mod.rs:264:             .filter(|r| r.dominios.iter().any(|x| x == d))
crates/tc2/src/seleccion/mod.rs:344:             "dominio-con-revisora" => self
crates/tc2/src/seleccion/mod.rs:345:                 .dom
crates/tc2/src/seleccion/mod.rs:346:                 .as_deref()
crates/tc2/src/seleccion/mod.rs:347:                 .is_some_and(|d| !self.revisoras_de(d).is_empty()),
crates/tc2/src/seleccion/mod.rs:348:             "dominio-sin-revisora" => self
crates/tc2/src/seleccion/mod.rs:351:                 .is_some_and(|d| self.revisoras_de(d).is_empty()),
```

`revisoras_de` mira solo el catalogo (lo declarado), nunca `self.instalado(id)` (mod.rs:251-253). El catalogo incluye lo que declara `recursos:` de la base aunque no este instalado (seleccion/orden.rs:153-155, `catalogo::de_config`).

##### Entrada que lo prueba (lectura, sin ejecutar)

El ejemplo del propio ADR (F1-15:556-558): la base declara en `recursos:` `{id: security-review, tipo: skill, rol: revisor, dominios: [puertas-seguridad]}`, el revisor en CI corre `tc2 revisoras --plugin taskcode-plugin=<dir> --tarea TASK-N` (con `--plugin`, el inventario son solo esos plugins, las skills de la base y los integrados: `security-review` no esta), y la tarea dice «token». Recorrido:
1. S10: `dominio-con-revisora` verdadero (declarada) -> anade `security-review` y anota S10 (mod.rs:404-416).
2. S11: `dominio-sin-revisora` falso -> no hay aviso `dominio-sin-revisora:puertas-seguridad`.
3. S14: `security-review` no instalada -> se quita con `no-instalado:security-review` (mod.rs:441-458).
Salida: `reglas` con S10 y S14, sin S11; `avisos` sin `dominio-sin-revisora`. Lo que dice el ADR: S10 no aplica (no esta instalada) y S11 da el aviso. Ningun test lo cubre: los de S11 (mod.rs:1212-1221) usan un catalogo sin revisora de puertas declarada.

##### Arreglo sugerido

`revisoras_de` filtra tambien por `self.instalado(&r.id)` (o el atomo lo hace), de modo que lo declarado y no instalado cuente como "sin revisora" para S10/S11.

<a id="a5-08"></a>

#### A5-08 · D1 · IMPORTANTE · pendiente [A5]

_Comprobación de «5 líneas, JSON con todo y `y N mas`» copiada en dos tests_

- id: A5-08
- regla: D1 (Duplicacion)
- ubicacion: crates/tc2/tests/doctor.rs:958 (y crates/tc2/tests/recursos.rs:378)
- modulo: tests de doctor y recursos (comprobacion de la salida acotada a 5 lineas, F1-4 decision 4)
- severidad: IMPORTANTE
- motivo_severidad: D1 real (IMPORTANTE como minimo): la misma comprobacion de "5 lineas como mucho, JSON con todos los hallazgos y `y N mas`" copiada en dos tests. Sin divergencia de comportamiento, no tapa fallo de datos ni de seguridad. Es codigo de test.
- estado: confirmado
- grupo_seguimiento: A5 — Seleccion, inventario, recursos y doctor

##### Evidencia

Fragmento 1 (tests/doctor.rs, test `salida_acotada_y_json_con_todo`, 8 lineas):
```text
crates/tc2/tests/doctor.rs:958:     let n = |k: &str| v["claves"][k].as_str().unwrap().parse::<usize>().unwrap();
crates/tc2/tests/doctor.rs:959:     let malos = n("fallos") + n("avisos") + n("no-comprobado");
crates/tc2/tests/doctor.rs:960:     let todos = v["hallazgos"].as_array().unwrap();
crates/tc2/tests/doctor.rs:961:     let oks = todos.iter().filter(|h| h["estado"] == "ok").count();
crates/tc2/tests/doctor.rs:962:     assert_eq!(todos.len(), malos + oks);
crates/tc2/tests/doctor.rs:963:     assert!(malos > 4, "el caso tiene que no caber: {json}");
crates/tc2/tests/doctor.rs:964:     assert_eq!(lineas.len(), 5);
crates/tc2/tests/doctor.rs:965:     assert_eq!(lineas[4], format!("y {} mas: tc2 doctor --json", malos - 3));
```

Fragmento 2 (tests/recursos.rs, test `texto_acotado_y_json_completo`, 9 lineas):
```text
crates/tc2/tests/recursos.rs:378:     let n = |k: &str| v["claves"][k].as_str().unwrap().parse::<usize>().unwrap();
crates/tc2/tests/recursos.rs:379:     let malos = n("fallos") + n("avisos") + n("no-comprobado");
crates/tc2/tests/recursos.rs:380:     let todos = v["hallazgos"].as_array().unwrap();
crates/tc2/tests/recursos.rs:382:     assert_eq!(
crates/tc2/tests/recursos.rs:384:         malos + todos.iter().filter(|h| h["estado"] == "ok").count()
crates/tc2/tests/recursos.rs:388:     assert!(malos > 4, "el caso tiene que no caber: {sj}");
crates/tc2/tests/recursos.rs:389:     assert_eq!(lineas.len(), 5);
crates/tc2/tests/recursos.rs:390:     assert_eq!(
crates/tc2/tests/recursos.rs:392:         format!("y {} mas: tc2 doctor --recursos --json", malos - 3)
```

Diff normalizado: identicos salvo la orden de detalle (`tc2 doctor --json` frente a `tc2 doctor --recursos --json`) y el nombre de la variable del JSON (`json`/`sj`). El arranque de los dos tests tambien es paralelo: lanzar sin `--json`, `lineas.len() <= 5`, sin ANSI, lanzar con `--json`, mismo codigo y una linea (doctor.rs:947-957, recursos.rs:356-377).

##### Arreglo sugerido

Un ayudante `comprobar_salida_acotada(texto, json, detalle)` en `tests/comun/` que reciban los dos tests.

<a id="a5-09"></a>

#### A5-09 · D1 · IMPORTANTE · pendiente [A5]

_`doctor_no_escribe_nada` copia el cuerpo de `doctor_contra`_

- id: A5-09
- regla: D1 (Duplicacion)
- ubicacion: crates/tc2/tests/doctor.rs:1017 (y crates/tc2/tests/doctor.rs:519)
- modulo: tests de doctor (montar la plataforma simulada y llamar a `doctor::comprobar`)
- severidad: IMPORTANTE
- motivo_severidad: D1 real (IMPORTANTE como minimo): `doctor_no_escribe_nada` copia el cuerpo del ayudante `doctor_contra` del mismo fichero en vez de reutilizarlo. Sin divergencia de comportamiento, no tapa fallo de datos ni de seguridad. Es codigo de test. Un tercer uso del mismo cierre `conectar` esta en doctor.rs:1099-1104.
- estado: confirmado
- grupo_seguimiento: A5 — Seleccion, inventario, recursos y doctor

##### Evidencia

Fragmento 1 (ayudante `doctor_contra`, 10 lineas):
```text
crates/tc2/tests/doctor.rs:519:     let m = servir(prov, caso);
crates/tc2/tests/doctor.rs:520:     let base = m.base();
crates/tc2/tests/doctor.rs:521:     let conectar = move |p: &Personas| {
crates/tc2/tests/doctor.rs:522:         Ok(Conectada {
crates/tc2/tests/doctor.rs:523:             plataforma: adaptador(prov, &base, p),
crates/tc2/tests/doctor.rs:524:             rol: Rol::Persona,
crates/tc2/tests/doctor.rs:525:         })
crates/tc2/tests/doctor.rs:526:     };
crates/tc2/tests/doctor.rs:527:     let g = f.git();
crates/tc2/tests/doctor.rs:528:     let (hs, r) = hallazgos(&g, &clon, &conectar);
```

Fragmento 2 (test `doctor_no_escribe_nada`, 10 lineas):
```text
crates/tc2/tests/doctor.rs:1017:         let m = servir(prov, &Caso::bien(prov));
crates/tc2/tests/doctor.rs:1018:         let base = m.base();
crates/tc2/tests/doctor.rs:1019:         let conectar = move |p: &Personas| {
crates/tc2/tests/doctor.rs:1020:             Ok(Conectada {
crates/tc2/tests/doctor.rs:1021:                 plataforma: adaptador(prov, &base, p),
crates/tc2/tests/doctor.rs:1022:                 rol: Rol::Persona,
crates/tc2/tests/doctor.rs:1023:             })
crates/tc2/tests/doctor.rs:1024:         };
crates/tc2/tests/doctor.rs:1025:         let g = f.git();
crates/tc2/tests/doctor.rs:1026:         let _ = hallazgos(&g, &clon, &conectar);
```

Diff normalizado: identicos salvo `caso` frente a `&Caso::bien(prov)` y `let (hs, r) =` frente a `let _ =`. La copia existe porque `doctor_contra` crea su propio `Fixture` y clon (doctor.rs:517-518) y aqui hace falta el clon preparado y el `Git` con su registro.

##### Arreglo sugerido

Partir `doctor_contra` en `doctor_contra_en(f, clon, prov, caso) -> (hs, r, recibidas, g)` y que `doctor_contra` y `doctor_no_escribe_nada` lo llamen. (El `let _ =` de doctor.rs:1026 no es Disfraz: ahi solo importa el efecto, y `hallazgos` hace `panic!` si `comprobar` da `Err`, doctor.rs:70.)

<a id="a5-06"></a>

#### A5-06 · D4 · IMPORTANTE · pendiente [A5]

_`las_reglas_se_leen_de_la_tabla` convierte el `Err` en `default()` y comprueba justo lo que da el `default`_

- id: A5-06
- regla: D4 (Disfraz) — en test
- ubicacion: crates/tc2/src/seleccion/mod.rs:849 (test `las_reglas_se_leen_de_la_tabla`)
- modulo: seleccion (lectura de `tabla-seleccion` del ADR)
- severidad: IMPORTANTE
- motivo_severidad: D4 de test si el mutante sobrevive: el test que demuestra que "las reglas salen de la tabla" (F1-15 decision 1) convierte un `Err` en `Seleccion::default()` con `unwrap_or_default()` y luego solo comprueba que `agentes` esta vacio, que es justo lo que da el `default`. Sigue en verde si la copia del ADR deja de leerse. No es CRITICO: otros tests leen el ADR real y fallarian si este no se leyera.
- estado: confirmado
- grupo_seguimiento: A5 — Seleccion, inventario, recursos y doctor

##### Evidencia

```text
crates/tc2/src/seleccion/mod.rs:844:         let adr = tabla::ADR.replace(
crates/tc2/src/seleccion/mod.rs:845:             "| S12 | `revision` | `siempre` | `lanzador` | `general-purpose` |",
crates/tc2/src/seleccion/mod.rs:846:             "| S12 | `revision` | `carril=fix` | `lanzador` | `general-purpose` |",
crates/tc2/src/seleccion/mod.rs:847:         );
crates/tc2/src/seleccion/mod.rs:848:         assert_ne!(adr, tabla::ADR);
crates/tc2/src/seleccion/mod.rs:849:         let s = revision_con(&adr, &rutas, &c, &i, &ctx).unwrap_or_default();
crates/tc2/src/seleccion/mod.rs:850:         assert!(s.agentes.is_empty(), "{s:?}");
```

`Seleccion` deriva `Default` (mod.rs:95) con `agentes: Vec::new()`. El atomo `carril=fix` solo lo valida `tabla::atomo_valido`:
```text
crates/tc2/src/seleccion/tabla.rs:168:         || a.strip_prefix("carril=")
crates/tc2/src/seleccion/tabla.rs:169:             .or_else(|| a.strip_prefix("carril!="))
crates/tc2/src/seleccion/tabla.rs:170:             .is_some_and(|c| CARRILES.contains(&c))
```
`grep -rn "carril=fix" crates/tc2 docs/adr/F1-15*.md`: el unico `carril=fix` de una `tabla-seleccion` es el de este test (la del ADR real usa `carril=errata` y `carril=riesgo-alto`), asi que ningun otro test depende de que `carril=fix` se lea.

##### Mutante propuesto (para el agente de mutacion)

- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da
- Fichero y linea: crates/tc2/src/seleccion/tabla.rs:170
- Cambio: `            .is_some_and(|c| CARRILES.contains(&c))` -> `            .is_some_and(|c| CARRILES.contains(&c) && c != "fix")`
  (comportamiento roto de verdad: una fila valida del ADR con `carril=fix` deja de leerse).
- Orden: `cargo test -p tc2 --lib seleccion::tests::las_reglas_se_leen_de_la_tabla` y, para descartar que otro lo cace, `cargo test -p tc2 --lib seleccion::` y la suite `cargo test -p tc2`.
- Esperado si es D4: `las_reglas_se_leen_de_la_tabla` en verde con el mutante (`test result: ok. 1 passed`): `revision_con` da `Err("S12: condicion fuera del vocabulario: carril=fix")`, `unwrap_or_default` lo vuelve `Seleccion::default()` y `agentes` esta vacio.
- Rojo de control (prueba que el test corre): en crates/tc2/src/seleccion/mod.rs:556 cambiar `let filas = tabla::seleccion(adr)?;` por `let filas = tabla::seleccion(tabla::ADR)?;` (la funcion deja de leer la tabla que recibe): S12 queda `siempre`, `agentes = ["general-purpose"]` y el test tiene que fallar en mod.rs:850.
- Test que deberia ponerse rojo y no lo hace: `las_reglas_se_leen_de_la_tabla` (falta exigir `Ok`: `let s = revision_con(...).expect(..)` o `assert!(r.is_ok())` antes, o comprobar `s.reglas` con `M04`/`S08`, que el `default` no tiene).

##### Mutación

- Resultado: **confirmado** (el mutante sobrevive en verde y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` propio). Clon restaurado con `git checkout -- .` entre pasos. No se ejecutó la suite completa (33 min por pase); sí los binarios que nombran la selección (`seleccion`, `revisoras`).

##### Mutante

```diff
--- a/crates/tc2/src/seleccion/tabla.rs
+++ b/crates/tc2/src/seleccion/tabla.rs
@@ -170 +170 @@
-            .is_some_and(|c| CARRILES.contains(&c))
+            .is_some_and(|c| CARRILES.contains(&c) && c != "fix")
```

```text
cargo test -p tc2 --lib seleccion::tests::las_reglas_se_leen_de_la_tabla
test seleccion::tests::las_reglas_se_leen_de_la_tabla ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.01s

cargo test -p tc2 --lib seleccion::
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 384 filtered out; finished in 0.62s

cargo test -p tc2 --test seleccion
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.72s

cargo test -p tc2 --test revisoras
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.08s
```

##### Control (código roto de verdad)

```diff
--- a/crates/tc2/src/seleccion/mod.rs
+++ b/crates/tc2/src/seleccion/mod.rs
@@ -556 +556 @@
-    let filas = tabla::seleccion(adr)?;
+    let filas = tabla::seleccion(tabla::ADR)?;
```

Orden: `cargo test -p tc2 --lib seleccion::tests::las_reglas_se_leen_de_la_tabla`

```text
test seleccion::tests::las_reglas_se_leen_de_la_tabla ... FAILED
thread 'seleccion::tests::las_reglas_se_leen_de_la_tabla' (30392) panicked at crates\tc2\src\seleccion\mod.rs:850:9:
Seleccion { revisoras: ["taskcode-plugin:rust-wasm-reviewer"], agentes: ["general-purpose"], roles: [], skills: [], modelo: "sonnet", reglas: ["M04", "S08", "S12"], avisos: [], senales: [], rutas: {"taskcode-plugin:rust-wasm-reviewer": ["a.rs"]} }
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.01s
```

### A6 — Estimación, progreso, roadmap y tablero

<a id="a6-04"></a>

#### A6-04 · D1 · IMPORTANTE divergente · pendiente [A6]

_Juego de tareas de `tests/siguiente.rs` copiado en `tests/roadmap.rs` y ya distinto de lo que dice su comentario_

- id: A6-04
- regla: D1
- ubicacion: crates/tc2/tests/roadmap.rs:437-486 ~ crates/tc2/tests/siguiente.rs:53-94
- modulo: tests de roadmap y next
- severidad: IMPORTANTE (divergente)
- motivo_severidad: D1 real (minimo IMPORTANTE) con copias que ya divergen: el comentario de la copia de roadmap dice que publica «el juego de tareas de `tests/siguiente.rs`», pero anade `TASK-009` en revision y cambia los titulos; quien edite una no sabe que la otra ya no es la misma. Por la decision 5 del plan, `divergente` no sube a CRITICO: no tapa fallo de datos ni seguridad.
- evidencia:

  `crates/tc2/tests/roadmap.rs:437-445`
  ```rust
  /// Publica en `bare` el juego de tareas de `tests/siguiente.rs` (ilegible,
  /// descartada, H2 antes que H1) sin pasar por `tc2`.
  fn publicar_estado(f: &Fixture, bare: &Path) {
      let dir = f.raiz.join("estado");
      f.crudo(&f.raiz, &["init", "-q", "-b", RAMA, "estado"]);
      let tarea = |id: &str, sprint: u32, estado: &str, deps: &[&str]| {
          let mut t = Tarea::nueva(id, &format!("Titulo de {id}"), "2026-01-01");
  ```

  `crates/tc2/tests/siguiente.rs:57-60` (con `tarea` en 42-51: titulo `"t"`)
  ```rust
  fn publicar_estado(f: &Fixture, bare: &Path) -> String {
      let dir = f.raiz.join("estado");
      f.crudo(&f.raiz, &["init", "-q", "-b", RAMA, "estado"]);
      for (id, md) in [
  ```

  Lo que divergio:
  - `crates/tc2/tests/roadmap.rs:470`: `("TASK-009", tarea("TASK-009", 2, "en-revision", &[])),` — no esta en `siguiente.rs:60-82`.
  - `crates/tc2/tests/roadmap.rs:443`: titulo `format!("Titulo de {id}")` frente a `crates/tc2/tests/siguiente.rs:43`: `Tarea::nueva(id, "t", "2026-01-01")`.
  - `siguiente.rs:93` devuelve el commit (`rev-parse HEAD`); `roadmap.rs` no.
  El resto (init, las 9 tareas comunes con sus dependencias, el ROADMAP con `### H2 — Antes` / `### H1 — Despues`, add/commit/push: roadmap.rs:452-485 frente a siguiente.rs:60-92) es igual.
- estado: confirmado
- grupo_seguimiento: A6

<a id="a6-02"></a>

#### A6-02 · D1 · IMPORTANTE · pendiente [A6]

_`linea_esperada` (test) reimplementa `ids_de_la_linea` del crate_

- id: A6-02
- regla: D1
- ubicacion: crates/tc2/src/grafo.rs:642-656 ~ crates/tc2/src/roadmap/generar.rs:401-416
- modulo: grafo (tests) / roadmap::generar
- severidad: IMPORTANTE
- motivo_severidad: D1 real (minimo IMPORTANTE). El helper de test `linea_esperada` reimplementa la funcion publica `ids_de_la_linea` del mismo crate, linea a linea; `tests/roadmap.rs:642-651` ya usa la de produccion para la misma fixture. Ningun ADR lo impone. No tapa fallo de datos ni seguridad. No `divergente` en lo que se usa (con la linea presente dan lo mismo; sin ella, la copia da `vec![]` y la original `None`).
- evidencia:

  `crates/tc2/src/roadmap/generar.rs:401-415`
  ```rust
  pub fn ids_de_la_linea(roadmap: &str, titulo: &str) -> Option<Vec<String>> {
      let prefijo = format!("- **{titulo}:** ");
      let resto = roadmap
          .lines()
          .find_map(|l| l.trim_end_matches('\r').strip_prefix(prefijo.as_str()))?;
      if resto == "ninguna" {
          return Some(Vec::new());
      }
      Some(
          resto
  ```

  `crates/tc2/src/grafo.rs:642-651`
  ```rust
      fn linea_esperada(titulo: &str) -> Vec<String> {
          let prefijo = format!("- **{titulo}:** ");
          let resto = TW_ESPERADO
              .lines()
              .find_map(|l| l.trim_end_matches('\r').strip_prefix(prefijo.as_str()))
              .unwrap_or_default();
          if resto == "ninguna" {
              return Vec::new();
          }
          resto
  ```
  Y las dos siguen igual: `.split(", ").filter_map(|p| p.split(' ').next()).map(str::to_owned).collect()` (generar.rs:411-414, grafo.rs:652-655).

  Diff normalizado: `roadmap` -> `TW_ESPERADO` (parametro fijado), `?` -> `.unwrap_or_default()`, y `Some(...)` quitado. La version reducida: `crate::roadmap::generar::ids_de_la_linea(TW_ESPERADO, titulo).unwrap_or_default()`.
- estado: confirmado
- grupo_seguimiento: A6

<a id="a6-03"></a>

#### A6-03 · D1 · IMPORTANTE · pendiente [A6]

_Operación de prueba `Poner`, `fila` y `terminada` copiadas entre los tests de estimación y progreso_

- id: A6-03
- regla: D1
- ubicacion: crates/tc2/tests/estimacion.rs:278-310 ~ crates/tc2/tests/progreso.rs:92-124 (y `fila` estimacion.rs:319-324 ~ progreso.rs:129-134; `terminada` estimacion.rs:329-345 ~ progreso.rs:141-155)
- modulo: tests de estimacion y progreso
- severidad: IMPORTANTE
- motivo_severidad: D1 real (minimo IMPORTANTE): la operacion de prueba `Poner` con su `impl Operacion` y el `poner` que la lanza (33 lineas) estan copiados byte a byte en dos ficheros de test de A6, y con ellos `fila` y `terminada`; existe `tests/comun/` para esto. Ningun ADR lo impone. No tapa fallo de datos ni seguridad. `terminada` difiere solo en datos (estimacion pone carril, complejidad y dos criterios; progreso, uno), no en comportamiento: no `divergente`.
- evidencia:

  `crates/tc2/tests/estimacion.rs:278-287` (y `crates/tc2/tests/progreso.rs:92-101`, identico)
  ```rust
  struct Poner(Vec<(String, String)>);

  impl Operacion for Poner {
      fn nombre(&self) -> &str {
          "poner"
      }
      fn anotacion(&self) -> String {
          "poner".to_owned()
      }
      fn exige_red(&self) -> bool {
  ```

  `crates/tc2/tests/estimacion.rs:303-310` (y `crates/tc2/tests/progreso.rs:117-124`, identico)
  ```rust
  fn poner(f: &Fixture, dir: &Path, ficheros: &[(String, String)]) {
      let g = tc2::git::Git::nuevo().con_entorno(entorno_ssh(f));
      let i = estado::transaccion(
          &Contexto::nuevo(&g, dir, &sin_catalogo, None),
          &Poner(ficheros.to_vec()),
      );
      assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
  }
  ```

  `terminada`: `crates/tc2/tests/estimacion.rs:329-336` frente a `crates/tc2/tests/progreso.rs:141-146`
  ```rust
  // estimacion.rs
  let mut t = Tarea::nueva(id, &format!("Hecha {id}"), "2026-10-01");
  "terminada".clone_into(&mut t.estado);
  t.carril = Some("funcionalidad".to_owned());
  t.complejidad = Some("media".to_owned());
  t.plan_aprobado = true;
  let s = BASE + dia * 86_400;
  let r1 = s + imp * 60;
  // progreso.rs: las mismas lineas sin `carril` ni `complejidad`
  ```
  Diff normalizado de `Poner`+`poner`+`fila`: ninguno. De `terminada`: dos lineas de datos de mas y un criterio menos en el cuerpo.
- estado: confirmado
- grupo_seguimiento: A6 (los helpers que tambien se repiten fuera de A6 —`entorno_ssh`, `identificar`, `sin_catalogo`, `preparar`— van como pista a A8: pistas/A6.txt puntos 1-4)

<a id="a6-05"></a>

#### A6-05 · D1 · IMPORTANTE · pendiente [A6]

_Doble `EnMemoria` copiado en los tests de `siguiente` aunque el de `roadmap::tests` es `pub(crate)`_

- id: A6-05
- regla: D1
- ubicacion: crates/tc2/src/siguiente.rs:139-149 ~ crates/tc2/src/roadmap/mod.rs:578-588
- modulo: siguiente (tests) / roadmap (tests)
- severidad: IMPORTANTE
- motivo_severidad: D1 real (minimo IMPORTANTE): el doble `EnMemoria` de `Estado` esta copiado en los tests unitarios de `siguiente`, aunque el de `roadmap::tests` es `pub(crate)` en el mismo crate y `tablero::tests` ya reutiliza ese modulo (`use crate::roadmap::tests::{estado_de_referencia, personas};`, tablero.rs:830). Ningun ADR lo impone. No tapa fallo de datos ni seguridad; no divergen. (La tercera copia, `tests/roadmap.rs:570-580`, es de un test de integracion, otro crate: no puede usar `cfg(test)`; se descarta.)
- evidencia:

  `crates/tc2/src/roadmap/mod.rs:578-588`
  ```rust
      /// Estado en memoria.
      pub(crate) struct EnMemoria(pub(crate) BTreeMap<String, String>);

      impl Estado for EnMemoria {
          fn rutas(&self) -> Vec<String> {
              self.0.keys().cloned().collect()
          }
          fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo> {
              Ok(self.0.get(ruta).cloned())
          }
      }
  ```

  `crates/tc2/src/siguiente.rs:139-149`
  ```rust
      /// Estado en memoria.
      struct EnMemoria(BTreeMap<String, String>);

      impl Estado for EnMemoria {
          fn rutas(&self) -> Vec<String> {
              self.0.keys().cloned().collect()
          }
          fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo> {
              Ok(self.0.get(ruta).cloned())
          }
      }
  ```
  Diff normalizado: solo la visibilidad (`pub(crate)`). Reducido: `use crate::roadmap::tests::EnMemoria;`.
- estado: confirmado
- grupo_seguimiento: A6

<a id="a6-06"></a>

#### A6-06 · D1 · IMPORTANTE · pendiente [A6]

_`c5_sin_red_y_sin_origin_sin_claves` reescribe `start_de_luis` del mismo fichero_

- id: A6-06
- regla: D1
- ubicacion: crates/tc2/tests/solape.rs:514-529 ~ crates/tc2/tests/solape.rs:602-617
- modulo: tests de solape
- severidad: IMPORTANTE
- motivo_severidad: D1 real (minimo IMPORTANTE): `c5_sin_red_y_sin_origin_sin_claves` reescribe el cuerpo entero del helper `start_de_luis` del mismo fichero (16 lineas) en vez de llamarlo; `start_de_luis` ademas devuelve el registro que c5 vuelve a pedir con `g.registro()`. Ningun ADR lo impone. No tapa fallo de datos ni seguridad; no divergen.
- evidencia:

  `crates/tc2/tests/solape.rs:514-523` (helper `start_de_luis`)
  ```rust
      let g = tc2::git::Git::nuevo().con_entorno(entorno_ssh(&e.f));
      let luis = || -> Result<Quien, tc2::contrato::Informe> {
          Ok(Quien::Persona(Cuenta {
              proveedor: Proveedor::GitHub,
              id: 1002,
              login: "luis".to_owned(),
          }))
      };
      let entorno = tc2::toma::Entorno {
          dir: &e.b,
  ```

  `crates/tc2/tests/solape.rs:602-611` (dentro de `c5_sin_red_y_sin_origin_sin_claves`)
  ```rust
      let g = tc2::git::Git::nuevo().con_entorno(entorno_ssh(&e.f));
      let luis = || -> Result<Quien, tc2::contrato::Informe> {
          Ok(Quien::Persona(Cuenta {
              proveedor: Proveedor::GitHub,
              id: 1002,
              login: "luis".to_owned(),
          }))
      };
      let entorno = tc2::toma::Entorno {
          dir: &e.b,
  ```
  Las seis lineas siguientes (`git: &g, quien: &luis, catalogo: &sin_catalogo, ahora: tc2::identidad::ahora(), }; let (i, s) = tc2::toma::start_con_solape(&entorno, ...)`) tambien coinciden (524-529 frente a 612-617). Diff normalizado: solo el literal `"TASK-002"` frente al parametro `id`. Reducido: `let (i, s, r) = start_de_luis(&e, "TASK-002");`.
- estado: confirmado
- grupo_seguimiento: A6

<a id="a6-07"></a>

#### A6-07 · D4 · IMPORTANTE · pendiente [A6]

_`c7_fuera_de_un_repo` dice probar el respaldo a la carpeta del proceso y todas sus entradas dan `SIN_TAREA` igual_

- id: A6-07
- regla: D4
- ubicacion: crates/tc2/src/progreso/mod.rs:479-497 (test `c7_fuera_de_un_repo`), sobre el codigo de crates/tc2/src/progreso/mod.rs:205-210 (`orden_estado`)
- modulo: progreso
- severidad: IMPORTANTE
- motivo_severidad: D4 en test (minimo IMPORTANTE). El test dice probar que «el JSON que no vale usa la carpeta del proceso» (respaldo exigido por la adenda de TASK-084 de F1-17: «busca el repo subiendo desde `workspace.current_dir` del JSON (o `cwd`, o la carpeta del proceso)»), pero todas sus entradas dan `SIN_TAREA` se use o no la carpeta del proceso, asi que no lo comprueba. Ademas, si la carpeta temporal esta dentro de un repo, salta sus dos aserciones principales sin fallar, aunque el comentario dice «se comprueba». No tapa un fallo de datos ni de seguridad (la status line pinta «sin tarea»): no CRITICO.
- evidencia:

  El codigo cuyo respaldo se dice probar, `crates/tc2/src/progreso/mod.rs:205-210`:
  ```rust
  pub fn orden_estado(stdin: &str, cwd: Option<&Path>, ahora: u64) -> String {
      match carpeta_del_json(stdin).or_else(|| cwd.map(Path::to_path_buf)) {
          Some(d) => linea_en(&d, ahora),
          None => linea::SIN_TAREA.to_owned(),
      }
  }
  ```

  El test, `crates/tc2/src/progreso/mod.rs:479-497`:
  ```rust
      /// C7: fuera de un repo, sin tarea; el JSON que no vale usa la carpeta
      /// del proceso.
      fn c7_fuera_de_un_repo() {
          ...
          // test no probaria nada: se comprueba).
          if buscar(&d).is_none() {
              ...
              assert_eq!(orden_estado(&json, None, 0), linea::SIN_TAREA);
              assert_eq!(orden_estado("no es json", Some(&d), 0), linea::SIN_TAREA);
          }
          assert_eq!(orden_estado("", None, 0), linea::SIN_TAREA);
  ```
  La entrada que lo prueba: `orden_estado("no es json", Some(&d), 0)` con `d` fuera de un repo da `SIN_TAREA` tanto si usa `d` (linea_en de una carpeta sin `.git` -> `SIN_TAREA`, mod.rs:164-166) como si la ignora (`None` -> `SIN_TAREA`, mod.rs:208). Ningun otro test lo distingue: los E2E de `tests/progreso.rs` mandan siempre un JSON valido (`json_de`, 765-772) salvo en `c7_sin_tarea_o_fuera_de_un_repo` (776-794), donde tampoco hay tarea en curso y todo da `SIN_TAREA`. Contraste: el E2E `r1_01_...` (tests/progreso.rs:553-556) si falla si la carpeta esta dentro de un repo («el test no probaria nada»); el unitario lo salta en silencio.

- mutante propuesto (sin compilar ni ejecutar aqui: lo hace el agente de mutacion):
  - commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da
  - fichero:linea: crates/tc2/src/progreso/mod.rs:206
  - cambio (ignora la carpeta del proceso; compila sin avisos nuevos, `cwd` sigue usado):
    ```diff
    -    match carpeta_del_json(stdin).or_else(|| cwd.map(Path::to_path_buf)) {
    +    match carpeta_del_json(stdin).or_else(|| cwd.filter(|_| false).map(Path::to_path_buf)) {
    ```
  - test que deberia ponerse rojo y se espera que siga verde: `cargo test -p tc2 --lib progreso::tests::c7_fuera_de_un_repo`
  - comprobacion de suite (se espera verde tambien): `cargo test -p tc2 --lib progreso` y `cargo test -p tc2 --test progreso`
  - control en rojo (prueba que el test corre): en mod.rs:208 `None => linea::SIN_TAREA.to_owned(),` -> `None => String::new(),` debe poner rojo `c7_fuera_de_un_repo` por su linea 495 (`assert_eq!(orden_estado("", None, 0), linea::SIN_TAREA)`).
- estado: confirmado
- grupo_seguimiento: A6

##### Mutación

- Resultado: **confirmado** (el mutante sobrevive en verde y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` propio). Clon restaurado con `git checkout -- .` entre pasos.

##### Mutante

```diff
--- a/crates/tc2/src/progreso/mod.rs
+++ b/crates/tc2/src/progreso/mod.rs
@@ -206 +206 @@
-    match carpeta_del_json(stdin).or_else(|| cwd.map(Path::to_path_buf)) {
+    match carpeta_del_json(stdin).or_else(|| cwd.filter(|_| false).map(Path::to_path_buf)) {
```

```text
cargo test -p tc2 --lib progreso::tests::c7_fuera_de_un_repo
test progreso::tests::c7_fuera_de_un_repo ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.01s

cargo test -p tc2 --lib progreso
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 374 filtered out; finished in 1.03s

cargo test -p tc2 --test progreso
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 47.93s
```

##### Control (código roto de verdad)

```diff
--- a/crates/tc2/src/progreso/mod.rs
+++ b/crates/tc2/src/progreso/mod.rs
@@ -208 +208 @@
-        None => linea::SIN_TAREA.to_owned(),
+        None => String::new(),
```

Orden: `cargo test -p tc2 --lib progreso::tests::c7_fuera_de_un_repo`

```text
test progreso::tests::c7_fuera_de_un_repo ... FAILED
thread 'progreso::tests::c7_fuera_de_un_repo' (33724) panicked at crates\tc2\src\progreso\mod.rs:495:9:
assertion `left == right` failed
  left: ""
 right: "tc2 · sin tarea en curso"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.00s
```

### A7 — Config, init, desinstalar, i18n, confianza, ciclo y verificación

<a id="a7-04"></a>

#### A7-04 · D1 · IMPORTANTE divergente · pendiente [A7]

_Cinco ayudantes de test copiados entre los módulos de `ciclo`, dos ya divergentes_

- id: A7-04
- regla: D1
- ubicacion: crates/tc2/src/ciclo/liberar.rs:423-467 (tests: md, estado, motivo, texto, fila) ~ crates/tc2/src/ciclo/operacion.rs:286-325 (tests: md, estado, motivo, texto, fila)
- modulo: ciclo (tests unitarios)
- severidad: IMPORTANTE (divergente)
- motivo_severidad: D1 real en codigo de test: cinco helpers copiados entre los dos modulos de test de `ciclo` (unos 40 lineas por lado). Tres son identicos (`estado`, `motivo`, `texto`) y dos ya divergen (`md` de liberar anade `plan_aprobado` y una fila de transicion con `hace`; `fila` de liberar recibe `por`, la de operacion fija `agente`). Ningun ADR lo impone. No tapa un fallo de datos ni de seguridad.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A7

##### Evidencia

Identicos — `crates/tc2/src/ciclo/liberar.rs:439-460`
```rust
    fn estado(tareas: &[(&str, String)]) -> EnMemoria {
        EnMemoria(
            tareas
                .iter()
                .map(|(id, t)| (format!("{id}/tarea.md"), t.clone()))
                .collect::<BTreeMap<_, _>>(),
        )
    }
    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
        match r { Err(Fallo::Rechazada { motivo, .. }) => Some(motivo), _ => None }
```
`crates/tc2/src/ciclo/operacion.rs:297-318`
```rust
    fn estado(tareas: &[(&str, String)]) -> EnMemoria {
        EnMemoria(
            tareas
                .iter()
                .map(|(id, t)| (format!("{id}/tarea.md"), t.clone()))
                .collect::<BTreeMap<_, _>>(),
        )
    }
    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
        match r { Err(Fallo::Rechazada { motivo, .. }) => Some(motivo), _ => None }
```
`texto` identico en liberar.rs:455-460 y operacion.rs:313-318 (`match r.as_deref() { Ok([c]) => c.contenido.clone(), otro => format!("{otro:?}") }`).

Divergentes — `md`:
```rust
// liberar.rs:423-436
    fn md(id: &str, estado: &str, asignado: Option<Asignado>, hace: u64) -> String {
        let mut t = Tarea::nueva(id, &format!("Tarea {id}"), "2026-11-01");
        t.estado = estado.to_owned();
        if estado == DESCARTADA { t.motivo = Some("no".to_owned()); }
        t.plan_aprobado = true;
        t.asignado_a = asignado;
        t.rama = Some(crate::alta::rama_de(id, &t.tipo, &t.titulo));
        ...
// operacion.rs:286-295
    fn md(id: &str, estado: &str, asignado: Option<Asignado>) -> String {
        let mut t = Tarea::nueva(id, &format!("Tarea {id}"), "2026-11-01");
        t.estado = estado.to_owned();
        if estado == DESCARTADA { t.motivo = Some("el de entonces".to_owned()); }
        t.asignado_a = asignado;
        t.rama = Some(crate::alta::rama_de(id, &t.tipo, &t.titulo));
        t.escribir(&Cuerpo::plantilla())
```
y `fila` (liberar.rs:462-467 con `{por}`, operacion.rs:320-325 con `agente` fijo). Tambien `AHORA` (liberar.rs:397 ~ operacion.rs:276) y `ana()` duplicados. El pase 0 lo marco (liberar.rs:437-462 ~ operacion.rs:295-320, 172 tokens; liberar.rs:691-707 ~ operacion.rs:462-478, 135 tokens).

Arreglo: un `#[cfg(test)] mod soporte` en `ciclo/` con `estado`, `motivo`, `texto`, `md(id, estado, asignado, Opciones)` y `fila(fase, por)`.

<a id="a7-01"></a>

#### A7-01 · D1 · IMPORTANTE · pendiente [A7]

_Esqueleto «plan como dato» copiado a mano entre `init` y `desinstalar`_

- id: A7-01
- regla: D1
- ubicacion: crates/tc2/src/init/mod.rs:187-225, 288-294, 346-369, 401-482, 553-563 ~ crates/tc2/src/desinstalar/mod.rs:211-249, 323-329, 367-390, 416-483, 605-615
- modulo: init / desinstalar
- severidad: IMPORTANTE
- motivo_severidad: D1 real (regla 4D: Duplicacion real >= IMPORTANTE). Dos ordenes con el mismo esqueleto «plan como dato» copiado a mano: tipo `Accion` con `nueva`/`motivo`/`orden`, `Hecho::solo`, `t`, `leer`, `no_se_lee`, el bloque de rechazo que pasa las `crea`/`borra` a `pendiente motivo=no-aplicada`, el bucle de aplicacion que corta en el primer fallo y la lectura de `current_dir` en `orden`. El ADR que se invoca (F1-12 PL1, citado en init/mod.rs:3-7 y desinstalar/mod.rs:6) impone el patron, no el codigo: no lo justifica. Las copias se diferencian en lo que cada orden anade (init: `propuesta`, `diagnostico`, `faltan`, `arreglo`; desinstalar: `sha`, `bloquea`, `ambito`), no en un comportamiento contradictorio, asi que no lleva la marca `divergente`. No tapa un fallo de datos ni de seguridad.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A7

##### Evidencia

1) `leer` y `no_se_lee`: identicos salvo `t(ruta)` frente a `ruta.display().to_string()` (mismo resultado: `t` es `p.display().to_string()`, init/mod.rs:347-349).

`crates/tc2/src/init/mod.rs:352-369`
```rust
pub(crate) fn no_se_lee(ruta: &Path, e: &std::io::Error) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Error)
        .con("ruta", t(ruta))
        .diagnostico(
            Mensaje::nuevo(&txt::NO_SE_LEE)
                .con("ruta", t(ruta))
                .con("error", e.to_string()),
        )
}
pub(crate) fn leer(p: &Path) -> Result<Option<Vec<u8>>, Informe> {
    match std::fs::read(p) { Ok(b) => Ok(Some(b)), Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None), Err(e) => Err(no_se_lee(p, &e)) }
```
`crates/tc2/src/desinstalar/mod.rs:373-390`
```rust
pub(crate) fn leer(p: &Path) -> Result<Option<Vec<u8>>, Informe> {
    match std::fs::read(p) { Ok(b) => Ok(Some(b)), Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None), Err(e) => Err(no_se_lee(p, &e)) }
}
pub(crate) fn no_se_lee(ruta: &Path, e: &std::io::Error) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Error)
        .con("ruta", ruta.display().to_string())
        .diagnostico(
            Mensaje::nuevo(&txt::NO_SE_LEE)
                .con("ruta", ruta.display().to_string())
                .con("error", e.to_string()),
```
Hasta el texto del catalogo esta repetido: `crates/tc2/src/i18n/catalogo/init.rs:56-59` y `crates/tc2/src/i18n/catalogo/desinstalar.rs:40-43`, los dos `es: "no se puede leer {ruta}: {error}"`.

2) Constructor y builders de `Accion` (init/mod.rs:188-225 ~ desinstalar/mod.rs:213-249): mismos campos comunes, mismo `crea`/`borra` con `"taskcodetwo"` y la decision, mismos `motivo` y `orden` con `#[must_use]`. Diff normalizado: init inicializa `propuesta/diagnostico/faltan`, desinstalar `sha/bloquea`; `crea` ~ `borra`.
```rust
// init/mod.rs:209-213
    pub(crate) fn crea(ruta: impl Into<String>, clase: &'static str, efecto: Efecto) -> Self {
        let mut a = Self::nueva(ruta, clase, "taskcodetwo", Decision::Crea);
        a.efecto = efecto;
        a
    }
// desinstalar/mod.rs:233-237
    pub(crate) fn borra(ruta: impl Into<String>, clase: &'static str, efecto: Efecto) -> Self {
        let mut a = Self::nueva(ruta, clase, "taskcodetwo", Decision::Borra);
        a.efecto = efecto;
        a
    }
```

3) Bucle de aplicacion (init/mod.rs:455-481 ~ desinstalar/mod.rs:461-483). Diff normalizado: `Crea`~`Borra`; init guarda `(ruta, error)` y trata aparte `AplicarPuertas`; desinstalar arma el `Informe` en el sitio.
```rust
// init/mod.rs:455-463
        for a in acciones.iter_mut() {
            if a.decision != Decision::Crea {
                continue;
            }
            if fallo.is_some() {
                a.decision = Decision::Pendiente;
                a.motivo = Some("no-aplicada".to_owned());
                continue;
            }
// desinstalar/mod.rs:461-469
        for a in acciones.iter_mut() {
            if a.decision != Decision::Borra {
                continue;
            }
            if fallo.is_some() {
                a.decision = Decision::Pendiente;
                a.motivo = Some("no-aplicada".to_owned());
                continue;
            }
```
y el mismo cierre del fallo: init/mod.rs:477-480 (`a.decision = Decision::Pendiente; a.motivo = Some("error".to_owned());`) ~ desinstalar/mod.rs:470-472.

4) Bloque de rechazo previo (init/mod.rs:423-432 ~ desinstalar/mod.rs:427-436):
```rust
// init/mod.rs:429-432
        for a in acciones.iter_mut().filter(|a| a.decision == Decision::Crea) {
            a.decision = Decision::Pendiente;
            a.motivo = Some("no-aplicada".to_owned());
        }
// desinstalar/mod.rs:430-436
        for a in acciones
            .iter_mut()
            .filter(|a| a.decision == Decision::Borra)
        {
            a.decision = Decision::Pendiente;
            a.motivo = Some("no-aplicada".to_owned());
        }
```

5) `Hecho::solo` identico (init/mod.rs:289-294 ~ desinstalar/mod.rs:324-329) y la lectura de `current_dir` en `orden` identica (init/mod.rs:553-563 ~ desinstalar/mod.rs:605-615, mismo `Mensaje::nuevo(&crate::i18n::catalogo::cli::CARPETA_ACTUAL)`).

El pase 0 lo marco como «fn casi iguales» (Jaccard 0.83, desinstalar/mod.rs:213 ~ init/mod.rs:188) y 8 fragmentos repetidos (desinstalar/mod.rs:227-249 ~ init/mod.rs:203-225, 310-335 ~ 275-300, 369-382 ~ 359-372, 427-438 ~ 425-434, 460-469 ~ 454-463, 588-597 ~ 544-553, 604-617 ~ 552-565, 624-635 ~ 571-581).

Lo comun que hay que extraer: un modulo «plan como dato» con `Accion<D, E>` generica en la decision y el efecto, `leer`/`no_se_lee` parametrizados por la orden y el bucle `aplicar_en_orden`.

<a id="a7-03"></a>

#### A7-03 · D1 · IMPORTANTE · pendiente [A7]

_`estado` y `evaluar` de `confianza` repiten lectura, cierre, huella y `Evaluacion`_

- id: A7-03
- regla: D1
- ubicacion: crates/tc2/src/confianza/mod.rs:198-242 (estado) ~ crates/tc2/src/confianza/mod.rs:246-275 (confiar)
- modulo: confianza
- severidad: IMPORTANTE
- motivo_severidad: D1 real (>= 10 lineas y > 50 tokens de logica). Las dos funciones repiten la misma firma de seis parametros, la lectura del almacen, el calculo del cierre con su `map_err`, la huella, la insercion de la `Entrada` y la construccion de la `Evaluacion`; solo cambian en si pasan la cache y en el estado. Ningun ADR lo impone (F1-8 decision 5 fija la interfaz `estado`, no su codigo). No divergen hoy ni tapan un fallo de datos o de seguridad.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A7

##### Evidencia

Lectura y cierre: `crates/tc2/src/confianza/mod.rs:206-217`
```rust
    let ruta = ruta_almacen(almacen, repo);
    let mut a = leer_almacen(&ruta);
    let entrada = a.claves.get(clave).cloned();
    let c = cierre::calcular(
        &repo.raiz,
        &repo.indice,
        orden,
        incluye,
        entrada.as_ref().map(|e| e.ficheros.as_slice()),
    )
    .map_err(|e| error_de_cierre(op, &e))?;
    let huella = cierre::huella(orden, incluye, &c.ficheros);
```
`crates/tc2/src/confianza/mod.rs:254-259`
```rust
    let ruta = ruta_almacen(almacen, repo);
    let mut a = leer_almacen(&ruta);
    a.version = VERSION_ALMACEN;
    let c = cierre::calcular(&repo.raiz, &repo.indice, orden, incluye, None)
        .map_err(|e| error_de_cierre(op, &e))?;
    let huella = cierre::huella(orden, incluye, &c.ficheros);
```

Insercion y resultado: `crates/tc2/src/confianza/mod.rs:226-241`
```rust
        a.claves.insert(
            clave.to_owned(),
            Entrada {
                huella: huella.clone(),
                ficheros: c.ficheros.clone(),
            },
        );
    ...
    Ok(Evaluacion {
        estado,
        huella,
        ficheros: c.ficheros.len(),
        hasheados: c.hasheados,
        globs_vacios: c.globs_vacios.clone(),
    })
```
`crates/tc2/src/confianza/mod.rs:260-274`
```rust
    a.claves.insert(
        clave.to_owned(),
        Entrada {
            huella: huella.clone(),
            ficheros: c.ficheros.clone(),
        },
    );
    escribir_almacen(&ruta, &a, op)?;
    Ok(Evaluacion {
        estado: Estado::Confiado,
        huella,
        ficheros: c.ficheros.len(),
        hasheados: c.hasheados,
        globs_vacios: c.globs_vacios.clone(),
    })
```

Diff normalizado: `cache = Some(entrada.ficheros)` frente a `None`; `estado` calculado frente a `Estado::Confiado`; `a.version = VERSION_ALMACEN` solo en `confiar`; `let _ = escribir_almacen` (cache, documentado en 224-225) frente a `escribir_almacen(..)?`. El pase 0 lo marco como fragmento repetido de 66 tokens (confianza/mod.rs:193-207 ~ 242-255). Arreglo: una `fn evaluar(repo, almacen, clave, orden, incluye, op, cache: bool) -> Result<(Almacen, PathBuf, Cierre, String), Informe>` y un `Evaluacion::de(estado, huella, &c)`.

<a id="a7-05"></a>

#### A7-05 · D1 · IMPORTANTE · pendiente [A7]

_Los dos tests unitarios de `ciclo/mod.rs` montan el mismo escenario git línea a línea_

- id: A7-05
- regla: D1
- ubicacion: crates/tc2/src/ciclo/mod.rs:505-527 ~ crates/tc2/src/ciclo/mod.rs:570-590
- modulo: ciclo (tests unitarios)
- severidad: IMPORTANTE
- motivo_severidad: D1 real en codigo de test: los dos tests unitarios de `ciclo/mod.rs` montan el mismo escenario git (carpeta temporal, gitconfig, bare `remoto.git`, clon, commit, `remote add`, `push`, `fetch`) linea a linea, 20 lineas por lado. Solo cambia el fichero del primer commit. No tapa un fallo de datos ni de seguridad. Ningun ADR lo impone.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A7

##### Evidencia

`crates/tc2/src/ciclo/mod.rs:505-518`
```rust
        let raiz = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("ciclo-para-mi-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(&raiz)?;
        let _borrar = Carpeta(raiz.clone());
        std::fs::write(raiz.join("gitconfig"), CONFIG)?;
        sh(&raiz, &raiz, &["init", "-q", "--bare", "-b", "develop", "remoto.git"])?;
        sh(&raiz, &raiz, &["init", "-q", "-b", "develop", "clon"])?;
        let (bare, clon) = (raiz.join("remoto.git"), raiz.join("clon"));
```
`crates/tc2/src/ciclo/mod.rs:570-583`
```rust
        let raiz = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("ciclo-umbral-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(&raiz)?;
        let _borrar = Carpeta(raiz.clone());
        std::fs::write(raiz.join("gitconfig"), CONFIG)?;
        sh(&raiz, &raiz, &["init", "-q", "--bare", "-b", "develop", "remoto.git"])?;
        sh(&raiz, &raiz, &["init", "-q", "-b", "develop", "clon"])?;
        let (bare, clon) = (raiz.join("remoto.git"), raiz.join("clon"));
```
y el final, igual en los dos (mod.rs:524-527 ~ 587-590):
```rust
        let url = bare.to_str().ok_or("ruta no UTF-8")?;
        sh(&raiz, &clon, &["remote", "add", "origin", url])?;
        sh(&raiz, &clon, &["push", "-q", "origin", "develop" /* , "main" */])?;
        sh(&raiz, &clon, &["fetch", "-q", "origin"])?;
```

Diff normalizado: el nombre de la carpeta (`ciclo-para-mi` / `ciclo-umbral`), el primer commit (`personas.yml` + rama `main` / `README.md`) y el `push` de `main` solo en el primero. El pase 0 lo marco (mod.rs:497-519 ~ 563-584, 186 tokens). Ademas `CONFIG` (mod.rs:446) y `entorno`/`sh` repiten `Fixture` de `tests/comun/mod.rs` (de A8): va como pista.

Arreglo: un `fn repo_con_remoto(nombre, primer_commit: impl FnOnce(&Path)) -> R<(Carpeta, Git, PathBuf)>` en el modulo de tests.

<a id="a7-07"></a>

#### A7-07 · D2 · IMPORTANTE · pendiente [A7]

_Texto renderizado en español en el sitio y metido en `{error}`: con `TC2_IDIOMA=en` el diagnóstico mezcla idiomas (F2-8)_

- id: A7-07
- regla: D2
- ubicacion: crates/tc2/src/init/aplicar.rs:25-27, 82-84; crates/tc2/src/init/mod.rs:470; crates/tc2/src/desinstalar/proyecto.rs:373-376; crates/tc2/src/desinstalar/mod.rs:542
- modulo: init / desinstalar (i18n)
- severidad: IMPORTANTE
- normalizacion: consolidador TASK-138, 2026-11-22: sube de MENOR a IMPORTANTE: es D2 que cambia la salida (con `TC2_IDIOMA=en` el diagnostico mezcla un trozo en espanol, contra F2-8/RF-73). La regla del plan (decision 4) deja D2 en MENOR solo si no cambia el comportamiento.
- motivo_severidad: D2: el codigo se aparta de F2-8 (RF-73), que fija que el texto se genera al escribir la salida en el idioma que toque. Aqui se renderiza en espanol en el sitio (`Idioma::Es` fijo, o un literal) y ese texto entra como valor del hueco `{error}` de `FALLO_AL_APLICAR`: con `TC2_IDIOMA=en` el diagnostico sale en ingles con un trozo en espanol. Solo afecta al diagnostico de stderr; la linea de stdout, `motivo` y los codigos no cambian (ADR F1-4). Por eso no llega a CRITICO; es IMPORTANTE porque cambia la salida (ver `normalizacion`).
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A7

##### Evidencia

La regla, en la cabecera del modulo: `crates/tc2/src/i18n/mod.rs:3-6`
```rust
//! El idioma se decide en un solo sitio, el borde del proceso (`main.rs`,
//! con [`Idioma::de_entorno`]). El resto del codigo no escribe textos: arma
//! un [`Mensaje`] neutro (un [`Texto`] del catalogo mas sus argumentos) y el
//! texto se genera al escribir la salida, en el idioma que toque.
```
(Plan, `docs/plan/PLAN-TASKCODE-2.md:338`: «F2-8 | i18n de mensajes (es, en)».)

Codigo que se aparta:
```rust
// crates/tc2/src/init/aplicar.rs:25-27
fn texto(i: &Informe) -> String {
    i.diagnostico_en(Idioma::Es).unwrap_or_else(|| i.linea())
}
// crates/tc2/src/init/aplicar.rs:82-84
                Mensaje::nuevo(&txt::CAMBIO_DESDE_EL_PLAN)
                    .con("ruta", ruta.display().to_string())
                    .texto(Idioma::Es)
// crates/tc2/src/init/mod.rs:470
                        Err(i.diagnostico_en(Idioma::Es).unwrap_or_else(|| i.linea()))
// crates/tc2/src/desinstalar/proyecto.rs:373-376
fn texto(i: &Informe) -> String {
    i.diagnostico_en(crate::i18n::Idioma::Es)
        .unwrap_or_else(|| i.linea())
}
// crates/tc2/src/desinstalar/mod.rs:541-542
            if &ahora != antes {
                return Err("cambio desde el plan".to_owned());
```
A donde van: `crates/tc2/src/init/mod.rs:529-533` y `crates/tc2/src/desinstalar/mod.rs:477-479` los meten como `.con("error", e)` en `FALLO_AL_APLICAR`, cuya plantilla inglesa es `"could not create {ruta}: {error}. ..."` (`i18n/catalogo/init.rs:52-55`) y `"could not remove {ruta}: {error}. ..."` (`i18n/catalogo/desinstalar.rs:36-39`).

Entrada que lo muestra (derivada del codigo, no ejecutada): con `TC2_IDIOMA=en`, editar `~/.profile` entre el plan y la aplicacion de `tc2 desinstalar --maquina` da en stderr `could not remove <ruta>: cambio desde el plan. What the previous lines list ...`. Ademas el literal `"cambio desde el plan"` ni siquiera esta en el catalogo, y `init` si tiene la entrada traducida `CAMBIO_DESDE_EL_PLAN` (catalogo/init.rs:60-63). El test de guarda no lo ve: ver A7-10.

Arreglo: que `aplicar` devuelva `Mensaje` en vez de `String` y que el hueco se rellene con `con_mensaje`.

<a id="a7-09"></a>

#### A7-09 · D4 · IMPORTANTE · pendiente [A7]

_`del_arbol` da `motivo=sin-config` («crea la configuración con tc2 init») ante cualquier error de lectura, también no UTF-8 o sin permiso_

- id: A7-09
- regla: D4
- ubicacion: crates/tc2/src/config/mod.rs:534-543 (del_arbol)
- modulo: config (lo usan confianza/mod.rs:312 y verificacion/ejecutar.rs:213)
- severidad: IMPORTANTE
- motivo_severidad: D4 real en codigo (regla 4D: Disfraz real >= IMPORTANTE). `del_arbol` dice «no hay config» ante cualquier error de lectura: un fichero que existe pero no es UTF-8 o no se puede leer sale como `motivo=sin-config` con el arreglo «crea la configuracion con tc2 init». El diagnostico es falso y la orden que propone no lo arregla. Falla cerrado (no ejecuta nada), asi que no tapa un fallo de datos ni de seguridad: no sube a CRITICO.
- estado: confirmado (lectura contra d82ea9d; la salida de abajo se deriva del codigo, no se ejecuto)
- grupo_seguimiento: A7

##### Evidencia

`crates/tc2/src/config/mod.rs:531-543`
```rust
/// Lee la configuracion del arbol de trabajo de `raiz` (F1-8 decision 5:
/// la orden que se confia y se ejecuta es la del arbol, no la de la base).
/// `Err` lleva el informe `config rechazada`.
pub fn del_arbol(raiz: &Path) -> Result<Config, Informe> {
    match std::fs::read_to_string(raiz.join(RUTA)) {
        Ok(texto) => leer(&texto).map_err(|e| e.informe()),
        Err(_) => Err(Invalida::una(
            Motivo::SinConfig,
            "",
            Mensaje::nuevo(&t::NO_HAY_CONFIG_EN_ARBOL).con("ruta", RUTA),
        )
```
Los textos: `crates/tc2/src/i18n/catalogo/config.rs:83-86` (`"no hay {ruta} en el arbol de trabajo"`) y el arreglo de `Motivo::SinConfig` en `config/mod.rs:405` -> `catalogo/config.rs:17-19` (`"crea la configuracion con tc2 init"`). `Motivo::SinConfig` se documenta como «No hay `.taskcode/config.yml` en la base» (config/mod.rs:322-323).

Entrada con la que no hace su trabajo: `.taskcode/config.yml` existe y guarda `---\nformato: 2\n` en UTF-16LE con BOM (lo que deja el Bloc de notas de Windows con «Unicode»), o cualquier byte no UTF-8. `std::fs::read_to_string` devuelve `ErrorKind::InvalidData`, cae en `Err(_)` y `tc2 verificar --perfil rapida` (o `tc2 confiar verificacion.comando`) responde `config rechazada motivo=sin-config errores=1` (con el comando cambiado a `verificar`/`confiar` en ejecutar.rs:213-216 y confianza/mod.rs:312-315), diciendo que no hay fichero y que se cree con `tc2 init`. El mismo error de permisos (`PermissionDenied`) da el mismo mensaje.

Contraste en el mismo modulo: `init` y `desinstalar` leen ese fichero con `String::from_utf8_lossy` (init/inventario.rs:94, desinstalar/proyecto.rs:265) y distinguen «no existe» (`ErrorKind::NotFound`, init/mod.rs:366) de «no se lee» (`no_se_lee`). Arreglo: `Err(e) if e.kind() == NotFound => SinConfig`, y el resto con un mensaje «no se puede leer {ruta}: {error}» o `Motivo::Yaml`.

<a id="a7-10"></a>

#### A7-10 · D4 · IMPORTANTE · pendiente [A7]

_`ningun_literal_fuera_del_catalogo` solo busca cuatro formas textuales; ya se le escapa el literal de A7-07_

- id: A7-10
- regla: D4
- ubicacion: crates/tc2/src/i18n/guarda.rs:17-23, 53-83 (test `ningun_literal_fuera_del_catalogo`)
- modulo: i18n (test de guarda de F2-8)
- severidad: IMPORTANTE
- motivo_severidad: D4 en test (regla 4D: Disfraz real >= IMPORTANTE). El test se llama «ningun literal fuera del catalogo» y su cabecera dice que comprueba que «ningun texto dirigido a la persona se escribe fuera del catalogo», pero solo busca cuatro formas textuales. Hoy ya llega a la persona un literal en espanol sin pasar por el catalogo (A7-07: `Err("cambio desde el plan".to_owned())`, desinstalar/mod.rs:542, metido en `{error}`) y el test sigue en verde. No tapa un fallo de datos ni de seguridad.
- estado: confirmado
- grupo_seguimiento: A7

##### Evidencia

`crates/tc2/src/i18n/guarda.rs:1-3`
```rust
//! Test de guarda de F2-8: ningun texto dirigido a la persona se escribe
//! fuera del catalogo. Lee el codigo de `src/` (sin los modulos de tests) y
//! busca las formas de colar un literal en un diagnostico.
```
`crates/tc2/src/i18n/guarda.rs:18-23`
```rust
const PROHIBIDOS: &[&str] = &[
    ".diagnostico(\"",
    ".diagnostico(format!",
    "Mensaje::externo(\"",
    "Mensaje::externo(format!",
];
```
Lo que ya se escapa sin mutar nada: `crates/tc2/src/desinstalar/mod.rs:541-542` (`return Err("cambio desde el plan".to_owned());`), que acaba en `Mensaje::nuevo(&txt::FALLO_AL_APLICAR)...con("error", e)` (desinstalar/mod.rs:477-479); y `crates/tc2/src/init/aplicar.rs:57` (`Err("aplicar-puertas sin plataforma".to_owned())`), por el mismo camino (init/mod.rs:529-533).

##### Mutante propuesto (para el agente de mutacion)

- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da.
- Mutante (verde esperado): `crates/tc2/src/desinstalar/mod.rs:601`, sustituir
  `.diagnostico(Mensaje::nuevo(&txt::SIN_ENTORNO).con("error", e.to_string()))`
  por
  `.diagnostico(Mensaje::nuevo(&crate::i18n::catalogo::comun::EXTERNO).con("texto", "no se sabe donde esta este tc2"))`.
  Compila (`con` admite `&str`) y pone un literal en espanol en un diagnostico sin pasar por el catalogo.
- Orden: `cargo test -p tc2 --lib i18n::guarda::ningun_literal_fuera_del_catalogo`
- Esperado con el mutante: `test result: ok. 1 passed` (el test no lo detecta).
- Contraste en rojo (codigo roto de verdad, para probar que el test corre): en la misma linea, `.diagnostico(Mensaje::externo(e.to_string()))`. Cambia el recuento de `Mensaje::externo(` (`EXTERNOS` solo admite `estado/lectura.rs`) y el test debe salir `FAILED` con «usos de Mensaje::externo distintos de EXTERNOS».

##### Mutación

- Resultado: **confirmado** (el mutante sobrevive en verde y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` propio). Clon restaurado con `git checkout -- .` entre pasos.
- El mutante compila con un aviso (`unused variable: e`, desinstalar/mod.rs:599).

##### Mutante

```diff
--- a/crates/tc2/src/desinstalar/mod.rs
+++ b/crates/tc2/src/desinstalar/mod.rs
@@ -601 +601 @@
-                .diagnostico(Mensaje::nuevo(&txt::SIN_ENTORNO).con("error", e.to_string()))
+                .diagnostico(Mensaje::nuevo(&crate::i18n::catalogo::comun::EXTERNO).con("texto", "no se sabe donde esta este tc2"))
```

```text
cargo test -p tc2 --lib i18n::guarda::ningun_literal_fuera_del_catalogo
test i18n::guarda::ningun_literal_fuera_del_catalogo ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.05s

cargo test -p tc2 --lib i18n
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 395 filtered out; finished in 0.06s
```

##### Control (código roto de verdad)

```diff
--- a/crates/tc2/src/desinstalar/mod.rs
+++ b/crates/tc2/src/desinstalar/mod.rs
@@ -601 +601 @@
-                .diagnostico(Mensaje::nuevo(&txt::SIN_ENTORNO).con("error", e.to_string()))
+                .diagnostico(Mensaje::externo(e.to_string()))
```

Orden: `cargo test -p tc2 --lib i18n::guarda::ningun_literal_fuera_del_catalogo`

```text
test i18n::guarda::ningun_literal_fuera_del_catalogo ... FAILED
thread 'i18n::guarda::ningun_literal_fuera_del_catalogo' (192) panicked at crates\tc2\src\i18n\guarda.rs:82:5:
usos de Mensaje::externo distintos de EXTERNOS: hay {"desinstalar/mod.rs": 1, "estado/lectura.rs": 1}, se permiten {"estado/lectura.rs": 1}
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.05s
```

<a id="a7-11"></a>

#### A7-11 · D4 · IMPORTANTE · pendiente [A7]

_El test dice cubrir `develop` sin publicar y provoca `ramas-por-delante`; `ramas-sin-remoto` (TASK-118) no lo ejercita nadie_

- id: A7-11
- regla: D4
- ubicacion: crates/tc2/tests/init.rs:1693-1754 (test `aplicar_puertas_exige_personas_ramas_y_persona`) frente a crates/tc2/src/init/inventario.rs:347-352
- modulo: tests de init (TASK-118)
- severidad: IMPORTANTE
- motivo_severidad: D4 en test (Disfraz real >= IMPORTANTE). El docstring dice que el test cubre el rechazo «con `develop` sin publicar», pero lo que provoca es un commit local por delante (`ramas-por-delante`). La rama que rechaza `develop`/`main` ausentes de `origin` (`ramas-sin-remoto`, criterio de TASK-118) no la ejercita ningun test de la suite (grep de `ramas-sin-remoto` en `crates/tc2/tests`: 0 apariciones fuera del fixture de seleccion). Si esa comprobacion desaparece, `--aplicar-puertas` crearia las puertas antes de que las ramas esten en el remoto, que es justo lo que el mensaje dice evitar («ya no se podrian subir con un push»). No es un fallo de datos ni de seguridad de credenciales: queda IMPORTANTE.
- estado: confirmado
- grupo_seguimiento: A7

##### Evidencia

Lo que el test dice cubrir: `crates/tc2/tests/init.rs:1693-1695`
```rust
/// Criterio 2 (resto de la precondicion): sin `personas.yml`, con `develop`
/// sin publicar o con la persona fuera de la lista, la opcion se rechaza
/// con su motivo y sin escribir en la plataforma.
```
Lo que prueba en realidad para `develop`: `crates/tc2/tests/init.rs:1732-1752`
```rust
    // `develop` con un commit local que `origin` no tiene.
    ...
        &["commit", "-q", "--allow-empty", "-m", "local", "--no-verify"],
    ...
        Some("ramas-por-delante"),
```
La rama sin cubrir: `crates/tc2/src/init/inventario.rs:347-352`
```rust
    if !sin_remoto.is_empty() {
        let diag =
            Mensaje::nuevo(&txt::PUERTAS_RAMAS_SIN_REMOTO).con("ramas", sin_remoto.join(", "));
        v.extend(rechazo("ramas-sin-remoto".to_owned(), sin_remoto, diag));
        return Ok(());
    }
```
El criterio: `crates/tc2/tests/fixtures/seleccion-taskcodetwo/TASK-118/tarea.md:37` enumera `ramas-sin-remoto` entre los rechazos exigidos. `repo_listo` (init.rs:1229-1243) siempre deja `develop` y `main` publicadas, asi que ningun test llega a esa rama.

##### Mutante propuesto (para el agente de mutacion)

- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da.
- Mutante (verde esperado): `crates/tc2/src/init/inventario.rs:347`, `if !sin_remoto.is_empty() {` -> `if false && !sin_remoto.is_empty() {`.
- Ordenes:
  - `cargo test -p tc2 --test init aplicar_puertas_exige_personas_ramas_y_persona` -> esperado `test result: ok. 1 passed`.
  - Para ver que ningun otro test lo detecta: `cargo test -p tc2 --test init` -> esperado todo en verde.
- Contraste en rojo (para probar que el test corre): `crates/tc2/src/init/inventario.rs:356`, `"ramas-por-delante"` -> `"ramas-por-detras"`; el mismo test debe salir `FAILED` en la asercion de init.rs:1746-1752.

##### Mutación

- Resultado: **confirmado** (el mutante sobrevive en verde y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` propio). Clon restaurado con `git checkout -- .` entre pasos.

##### Mutante

```diff
--- a/crates/tc2/src/init/inventario.rs
+++ b/crates/tc2/src/init/inventario.rs
@@ -347 +347 @@
-    if !sin_remoto.is_empty() {
+    if false && !sin_remoto.is_empty() {
```

```text
cargo test -p tc2 --test init aplicar_puertas_exige_personas_ramas_y_persona
test aplicar_puertas_exige_personas_ramas_y_persona ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 15.16s

cargo test -p tc2 --test init
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 63.59s
```

##### Control (código roto de verdad)

```diff
--- a/crates/tc2/src/init/inventario.rs
+++ b/crates/tc2/src/init/inventario.rs
@@ -356 +356 @@
-        v.extend(rechazo("ramas-por-delante".to_owned(), por_delante, diag));
+        v.extend(rechazo("ramas-por-detras".to_owned(), por_delante, diag));
```

Orden: `cargo test -p tc2 --test init aplicar_puertas_exige_personas_ramas_y_persona`

```text
test aplicar_puertas_exige_personas_ramas_y_persona ... FAILED
thread 'aplicar_puertas_exige_personas_ramas_y_persona' (15864) panicked at crates\tc2\tests\init.rs:1747:5:
assertion `left == right` failed: accion ruta=.../repo\.taskcode/config.yml clase=config dueno=taskcodetwo decision=existe
(...)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 24 filtered out; finished in 17.70s
```

### A8 — Transversal: actualizar_cli, instalacion, contrato, dato y xtask

<a id="a8-13"></a>

#### A8-13 · D1 · IMPORTANTE · pendiente [A8]

_Comando de shell según el sistema copiado tres veces en los tests de `ejecutar_con_limite`_

- id: A8-13
- regla: D1
- ubicacion: crates/tc2/src/actualizar_cli.rs:788-797 ~ crates/tc2/src/actualizar_cli.rs:804-813 ~ crates/tc2/src/actualizar_cli.rs:829-838
- modulo: actualizar_cli (tests unitarios)
- severidad: IMPORTANTE
- motivo_severidad: D1 real (en el umbral: 10 lineas por copia): el mismo «comando de shell segun el sistema» (`cmd /c` en Windows, `sh -c` en el resto, mas `TC2_PRUEBA=1`) copiado tres veces en dos tests de `ejecutar_con_limite`; solo cambia el guion. Codigo de test: no tapa un fallo de datos ni de seguridad. No `divergente`.
- evidencia:

  `crates/tc2/src/actualizar_cli.rs:788-797`
  ```rust
          let mut cmd = if cfg!(windows) {
              let mut c = Command::new("cmd");
              c.args(["/c", "ping -n 30 127.0.0.1 >nul & echo fin"]);
              c
          } else {
              let mut c = Command::new("sh");
              c.args(["-c", "sleep 30; echo fin"]);
              c
          };
          cmd.env("TC2_PRUEBA", "1");
  ```

  `crates/tc2/src/actualizar_cli.rs:829-838`
  ```rust
          let mut cmd = if cfg!(windows) {
              let mut c = Command::new("cmd");
              c.args(["/c", "start /b ping -n 6 127.0.0.1 & echo hola"]);
              c
          } else {
              let mut c = Command::new("sh");
              c.args(["-c", "sleep 5 & echo hola"]);
              c
          };
          cmd.env("TC2_PRUEBA", "1");
  ```

  Diff normalizado: identicos salvo los dos literales del guion; la tercera copia (`eco`, 804-813) igual con `echo hola`. Pase 0: fragmento repetido de 113 tokens normalizados `actualizar_cli.rs:786-800 ~ 827-841`.

  Extraccion propuesta: `fn shell(windows: &str, unix: &str) -> Command` en el `mod tests`.
- estado: confirmado
- grupo_seguimiento: A8 — Transversal
- absorbe: (ninguno)

<a id="a9-03"></a>

#### A9-03 · D1 · IMPORTANTE · pendiente [A8]

_El cierre `runs_de` de un test reescribe `runs_del_job` del mismo módulo_

- id: A9-03
- regla: D1
- ubicacion: xtask/src/workflows.rs:2712-2721 (cierre `runs_de` de `el_workflow_del_revisor_pasa_el_lint`) ~ xtask/src/workflows.rs:2162-2173 (`runs_del_job`); copias mas cortas del mismo acceso en xtask/src/workflows.rs:2680-2686 (`valores`) y :2136-2139, y de «los pasos de un job» en :928-934 (`run_del_paso`), :1307-1313 (`aislamiento_antes_del_checkout`), :2102-2108 (`artefactos_subidos`), :1892-1895 y :1988-1991 (cierres `pasos`)
- modulo: xtask (lint de workflows, codigo de test: el modulo es `#[cfg(test)]`, xtask/src/main.rs:31-32)
- severidad: IMPORTANTE
- motivo_severidad: D1 real, en el umbral (10 lineas por copia, unos 85 tokens): el cierre `runs_de` reescribe dentro de un test la funcion `runs_del_job` que el mismo modulo ya tiene 540 lineas mas arriba; solo cambia que devuelve la lista en vez de unirla con `\n`, y sus dos usos (:2722-2726 `.any(|r| r.contains("tc2 hallazgos validar"))` y :2742-2746 `normalizar(&r).contains("needs.")`) funcionan igual sobre `runs_del_job`. Codigo de test: no tapa un fallo de datos ni de seguridad (no CRITICO). No `divergente`.
- evidencia:

  `xtask/src/workflows.rs:2712-2721`
  ```rust
      let runs_de = |n: &str| -> Vec<String> {
          let mut v = Vec::new();
          if let Some(j) = job(n) {
              j.escalares("", &mut v);
          }
          v.iter()
              .filter(|(k, _)| *k == "run")
              .map(|(_, x)| (*x).to_owned())
              .collect()
      };
  ```

  `xtask/src/workflows.rs:2162-2171` (la funcion acaba en :2172-2173 con `.join("\n")` y `}`)
  ```rust
  fn runs_del_job(raiz: &Nodo, job: &str) -> String {
      let mut valores = Vec::new();
      if let Some(j) = raiz.get("jobs").and_then(|j| j.get(job)) {
          j.escalares("", &mut valores);
      }
      valores
          .iter()
          .filter(|(k, _)| *k == "run")
          .map(|(_, v)| *v)
          .collect::<Vec<_>>()
  ```
  Diff normalizado: `job(n)` es el cierre de :2679 (`raiz.get("jobs").and_then(|j| j.get(n))`), igual que la expresion de :2164; `(*x).to_owned()` + `collect()` frente a `*v` + `collect::<Vec<_>>().join("\n")`. Lo demas, identico salvo nombres.

  Relacionadas, por debajo del umbral (no cuentan solas): «los pasos del job N» se escribe cinco veces como `let Some(Nodo::Lista(pasos)) = raiz.get("jobs").and_then(|j| j.get(job)).and_then(|j| j.get("steps")) else { ... }` (7 lineas en :928-934, :1307-1313 y :2102-2108) o como cierre `pasos` de 4 lineas (:1892-1895 = :1988-1991, identicos), sin una funcion `pasos_del_job`.

  Comparado contra todo el repo: `git grep -n 'escalares("", &mut\|get("steps")' d82ea9d -- '*.rs'` solo da `xtask/src/workflows.rs` (:593, :628, :931, :1310, :1892, :1988, :2105, :2138, :2165, :2683, :2715); no hay copia fuera de xtask (tc2 lee YAML con `serde_saphyr`, otro formato de datos).

  Extraccion propuesta: usar `runs_del_job(&raiz, n)` en el test y anadir `fn pasos_del_job(raiz: &Nodo, job: &str) -> &[Nodo]` para las cinco lecturas de pasos.
- estado: confirmado (lectura contra d82ea9d; no se ejecuto)
- grupo_seguimiento: A8 — Transversal
- absorbe: (ninguno)

<a id="a8-15"></a>

#### A8-15 · D4 · IMPORTANTE · pendiente [A8]

_El único test de `detectar_modo` no pasa por las ramas `opt/homebrew` ni `winget/links`_

- id: A8-15
- regla: D4
- ubicacion: crates/tc2/src/instalacion.rs:56 y :62 (`detectar_modo`, ramas `opt/homebrew` y `winget/links`), test que deberia cubrirlas: crates/tc2/src/instalacion.rs:280-332 (`detectar_modo_con_rutas_de_los_tres_sistemas`)
- modulo: instalacion
- severidad: IMPORTANTE
- normalizacion: consolidador TASK-138, 2026-11-22: quita «pendiente de confirmar por mutacion»: la seccion «Mutación» confirma los dos mutantes en verde y el control en rojo. La frase «Sin ejecutar: queda pendiente-mutacion» del motivo se sustituyo en la ronda 1 de revision.
- motivo_severidad: Disfraz en test, candidato: el unico test de `detectar_modo` no pasa por dos de las ramas que reconocen un gestor de paquetes. Si una se rompe, `tc2 actualizar --cli` trata un `tc2` de Homebrew (`/opt/homebrew/bin`) o de winget (`WinGet\Links`) como instalado a mano y lo sustituye por encima del gestor; `tc2 desinstalar` usa el mismo `Modo` (`instalacion::Entorno::modo`). No llega a CRITICO: no hay perdida de datos ni fallo de seguridad (el binario nuevo pasa por sha256 y attestation igual). Confirmado por mutacion (seccion «Mutación»: mutantes en verde y control en rojo).
- evidencia:

  Codigo, `crates/tc2/src/instalacion.rs:52-62`
  ```rust
      let homebrew = dir
          .iter()
          .position(|p| p == "cellar")
          .is_some_and(|i| i + 2 < dir.len())
          || dir.starts_with(&["opt".to_owned(), "homebrew".to_owned()])
          || seguidas("linuxbrew", ".linuxbrew");
      let gestor = if npm {
          Some("npm")
      } else if homebrew {
          Some("homebrew")
      } else if seguidas("winget", "packages") || seguidas("winget", "links") {
  ```

  Test, `crates/tc2/src/instalacion.rs:306` y `:307-310`: la unica ruta de Homebrew con `opt/homebrew` es `"/opt/homebrew/Cellar/tc2/2.0.0/bin/tc2"`, que ya entra por la rama de `cellar` (`cellar` en la posicion 2 de 6 carpetas); la unica de winget es `...\Microsoft\WinGet\Packages\x\tc2.exe` (rama `packages`). Ningun otro test llama a `detectar_modo` ni nombra homebrew o winget (`rtk proxy grep -rn -i "homebrew\|winget" crates/tc2/tests crates/tc2/src`, fuera de instalacion.rs: sin resultados).

  Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da.
  Mutante 1 (borra la rama `opt/homebrew`):
  ```diff
  --- a/crates/tc2/src/instalacion.rs
  +++ b/crates/tc2/src/instalacion.rs
  @@ -53,7 +53,6 @@
           .iter()
           .position(|p| p == "cellar")
           .is_some_and(|i| i + 2 < dir.len())
  -        || dir.starts_with(&["opt".to_owned(), "homebrew".to_owned()])
           || seguidas("linuxbrew", ".linuxbrew");
  ```
  Mutante 2 (borra la rama `winget/links`):
  ```diff
  -    } else if seguidas("winget", "packages") || seguidas("winget", "links") {
  +    } else if seguidas("winget", "packages") {
  ```
  Orden: `cargo test -p tc2 --lib instalacion::tests::detectar_modo_con_rutas_de_los_tres_sistemas`
  Esperado si es Disfraz: `test result: ok. 1 passed` con cada mutante puesto. Contraste en rojo: con `Modo::Gestor("homebrew")` cambiado a `Modo::Instalador` en la linea 61 el mismo test tiene que fallar.
  Arreglo del test: anadir `("/opt/homebrew/bin/tc2", "homebrew")` y `(r"C:\Users\ana\AppData\Local\Microsoft\WinGet\Links\tc2.exe", "winget")` a la tabla de gestores.
- estado: confirmado
- grupo_seguimiento: A8 — Transversal
- absorbe: (ninguno)

##### Mutación

- Resultado: **confirmado** (los dos mutantes sobreviven en verde y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` propio). Clon restaurado con `git checkout -- .` entre pasos.

##### Mutante 1 (rama `opt/homebrew`)

```diff
--- a/crates/tc2/src/instalacion.rs
+++ b/crates/tc2/src/instalacion.rs
@@ -56 +55,0 @@
-        || dir.starts_with(&["opt".to_owned(), "homebrew".to_owned()])
```

```text
cargo test -p tc2 --lib instalacion::tests::detectar_modo_con_rutas_de_los_tres_sistemas
test instalacion::tests::detectar_modo_con_rutas_de_los_tres_sistemas ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.00s

cargo test -p tc2 --lib instalacion
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 397 filtered out; finished in 0.00s
```

##### Mutante 2 (rama `winget/links`)

```diff
--- a/crates/tc2/src/instalacion.rs
+++ b/crates/tc2/src/instalacion.rs
@@ -62 +62 @@
-    } else if seguidas("winget", "packages") || seguidas("winget", "links") {
+    } else if seguidas("winget", "packages") {
```

```text
cargo test -p tc2 --lib instalacion::tests::detectar_modo_con_rutas_de_los_tres_sistemas
test instalacion::tests::detectar_modo_con_rutas_de_los_tres_sistemas ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.00s

cargo test -p tc2 --lib instalacion
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 397 filtered out; finished in 0.00s
```

##### Control (código roto de verdad; equivale a dar `Modo::Instalador` para homebrew)

```diff
--- a/crates/tc2/src/instalacion.rs
+++ b/crates/tc2/src/instalacion.rs
@@ -61 +61 @@
-        Some("homebrew")
+        None
```

Orden: `cargo test -p tc2 --lib instalacion::tests::detectar_modo_con_rutas_de_los_tres_sistemas`

```text
test instalacion::tests::detectar_modo_con_rutas_de_los_tres_sistemas ... FAILED
thread 'instalacion::tests::detectar_modo_con_rutas_de_los_tres_sistemas' (1924) panicked at crates\tc2\src\instalacion.rs:316:13:
assertion `left == right` failed: /opt/homebrew/Cellar/tc2/2.0.0/bin/tc2
  left: Instalador
 right: Gestor("homebrew")
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.00s
```

<a id="a9-02"></a>

#### A9-02 · D4 · IMPORTANTE · pendiente [A8]

_`comprobar_nuevo` (el binario nuevo arranca y dice versión, target y sha256) no tiene test que lo haga fallar_

- id: A9-02
- regla: D4
- ubicacion: crates/tc2/src/actualizar_cli.rs:406-415 (llamada a `comprobar_nuevo` y rechazo `binario-no-arranca`) y :710-745 (`comprobar_nuevo`); tests que deberian cubrirlo: crates/tc2/tests/actualizar_cli.rs (todos sus `#[test]`, 293-704) y crates/tc2/src/actualizar_cli.rs:779-1046 (`mod tests`)
- modulo: actualizar_cli
- severidad: IMPORTANTE
- normalizacion: consolidador TASK-138, 2026-11-22: quita «pendiente de confirmar por mutacion»: la seccion «Mutación» confirma los dos mutantes en verde y el control en rojo. La frase «Sin ejecutar: queda pendiente-mutacion» del motivo se sustituyo en la ronda 1 de revision.
- motivo_severidad: Disfraz en test, candidato: el paso que exige que el binario nuevo arranque y diga su version, su target, el sha256 que calculo este proceso y un protocolo antes de sustituir el `tc2` instalado (ADR F1-14, punto 4; cabecera de actualizar_cli.rs:18-22) no tiene ningun test que lo haga fallar. El unico binario que llega a ese paso en los tests es `tc2_falso`, construido para pasarlo (examples/tc2_falso.rs:6-8: responde con el sha256 de su propio fichero). Si `comprobar_nuevo` se rompe y devuelve siempre `Ok(())`, `tc2 actualizar --cli` sustituye el ejecutable por un fichero que no arranca, o que dice ser otra version o de otro target, y la persona se queda sin CLI. No llega a CRITICO: el sha256 contra `SHA256SUMS` y la attestation se comprueban antes (actualizar_cli.rs:376-402), asi que no entra un binario ajeno a la release. Confirmado por mutacion (seccion «Mutación»: mutantes en verde y control en rojo).
- evidencia:

  Codigo, `crates/tc2/src/actualizar_cli.rs:406-410`
  ```rust
      if let Err(detalle) = comprobar_nuevo(&temporal, &publicado.version, target.alias, &obtenido) {
          return Informe::nuevo(NOMBRE, Resultado::Rechazada)
              .con("motivo", "binario-no-arranca")
              .diagnostico(
                  Mensaje::nuevo(&txt::NO_ARRANCA)
  ```
  y `crates/tc2/src/actualizar_cli.rs:732-743` (las cuatro comprobaciones: version, target, sha256, protocolo).

  Ningun test llega a la rama: `git grep -n "no-arranca\|NO_ARRANCA\|comprobar_nuevo" d82ea9d -- '*.rs'` solo da actualizar_cli.rs:406, :408, :410, :710 y el catalogo `i18n/catalogo/actualizar.rs:81`; ningun fichero de `tests/`. En `crates/tc2/tests/actualizar_cli.rs` los tests que descargan un binario lo rechazan antes por sha256 (`sha256_que_no_casa_es_rechazada_y_no_toca_nada`, :450), attestation (`attestation_con_gh_ausente_valido_y_que_falla`, :482), redireccion (:640) o tamano (:673), o sirven `tc2_falso` con su sha256 correcto (`actualizar_sustituye_vx_por_vy_servida_en_127_0_0_1`, :413-447), que pasa las cuatro comprobaciones.

  Entrada con la que el codigo roto no se detectaria: una release de ensayo con `release(VERSION_Y, b"binario-y", None)` (sha256 correcto de un fichero que no es ejecutable) o con `release("9.9.8", &binario_vy(), None)` (`tc2_falso` dice 9.9.9).

  Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da.
  Mutante 1 (se salta la comprobacion entera):
  ```diff
  --- a/crates/tc2/src/actualizar_cli.rs
  +++ b/crates/tc2/src/actualizar_cli.rs
  @@ -406 +406 @@
  -    if let Err(detalle) = comprobar_nuevo(&temporal, &publicado.version, target.alias, &obtenido) {
  +    if let Err(detalle) = Ok::<(), String>(()) {
  ```
  Mutante 2 (quita solo la comprobacion del sha256 que da el binario nuevo):
  ```diff
  @@ -738 +738 @@
  -    if campo("sha256") != sha256 {
  +    if false {
  ```
  Los dos compilan (solo avisos `unused`; el workspace no niega avisos: `Cargo.toml`, `[workspace.lints.rust] unused = "warn"`).
  Ordenes: `cargo test -p tc2 --test actualizar_cli` y `cargo test -p tc2 --lib actualizar_cli::tests`
  Esperado si es Disfraz: `test result: ok.` en las dos, con todos los tests en verde, con cada mutante puesto.
  Contraste en rojo: con `return Err("roto".to_owned());` como primera linea de `comprobar_nuevo` (actualizar_cli.rs:711), `actualizar_sustituye_vx_por_vy_servida_en_127_0_0_1` tiene que fallar (codigo 3 en lugar de 0).
  Arreglo del test: un caso con `release(VERSION_Y, b"binario-y", None)` que exija `actualizar rechazada motivo=binario-no-arranca`, codigo 3, el ejecutable intacto (`sha256(&exe)` igual) y `restos(&dir, &exe)` vacio; y otro con `release("9.9.8", &binario_vy(), None)` para la version.
- estado: confirmado
- grupo_seguimiento: A8 — Transversal
- absorbe: (ninguno)

##### Mutación

- Resultado: **confirmado** (los dos mutantes sobreviven en verde y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` propio). Clon restaurado con `git checkout -- .` entre pasos.

##### Mutante 1 (se salta la comprobación entera)

```diff
--- a/crates/tc2/src/actualizar_cli.rs
+++ b/crates/tc2/src/actualizar_cli.rs
@@ -406 +406 @@
-    if let Err(detalle) = comprobar_nuevo(&temporal, &publicado.version, target.alias, &obtenido) {
+    if let Err(detalle) = Ok::<(), String>(()) {
```

```text
cargo test -p tc2 --test actualizar_cli
test actualizar_sin_cli_es_uso ... ok
test la_siguiente_ejecucion_borra_el_old ... ok
test actualizar_sustituye_vx_por_vy_servida_en_127_0_0_1 ... ok
test un_corte_en_la_descarga_del_binario_se_reintenta_una_vez ... ok
test redireccion_no_permitida_no_se_sigue_ni_reintenta ... ok
test binario_mayor_que_el_limite_se_rechaza ... ok
test el_tc2_de_la_cache_del_plugin_no_se_sustituye ... ok
test sha256_que_no_casa_es_rechazada_y_no_toca_nada ... ok
test attestation_con_gh_ausente_valido_y_que_falla ... ok
test comprobar_no_modifica_nada_y_sin_red_da_sinred_con_la_url ... ok
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.37s

cargo test -p tc2 --lib actualizar_cli::tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 391 filtered out; finished in 2.15s
```

##### Mutante 2 (quita solo la comprobación del sha256 que da el binario nuevo)

```diff
--- a/crates/tc2/src/actualizar_cli.rs
+++ b/crates/tc2/src/actualizar_cli.rs
@@ -738 +738 @@
-    if campo("sha256") != sha256 {
+    if false {
```

```text
cargo test -p tc2 --test actualizar_cli
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.41s

cargo test -p tc2 --lib actualizar_cli::tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 391 filtered out; finished in 2.14s
```

##### Control (código roto de verdad)

```diff
--- a/crates/tc2/src/actualizar_cli.rs
+++ b/crates/tc2/src/actualizar_cli.rs
@@ -710,0 +711 @@ fn comprobar_nuevo(nuevo: &Path, version: &str, alias: &str, sha256: &str) -> Result<(), String> {
+    return Err("roto".to_owned());
```

Orden: `cargo test -p tc2 --test actualizar_cli actualizar_sustituye_vx_por_vy_servida_en_127_0_0_1`

```text
test actualizar_sustituye_vx_por_vy_servida_en_127_0_0_1 ... FAILED
thread 'actualizar_sustituye_vx_por_vy_servida_en_127_0_0_1' (13656) panicked at crates\tc2\tests\actualizar_cli.rs:424:5:
assertion `left == right` failed: el binario descargado no responde como tc2 9.9.9 de win-x64: roto. No se ha cambiado nada
  left: Some(3)
 right: Some(0)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.33s
```

<a id="a11-02"></a>

#### A11-02 · D4 · IMPORTANTE · pendiente [A8]

_Cuatro de las cinco órdenes de `orden_del_gestor` solo se prueban «no vacías» y la deduplicación de `dirs_de_instalacion` no la ejercita ningún test_

- id: A11-02
- regla: D4
- ubicacion: crates/tc2/src/instalacion.rs:77-87 (`orden_del_gestor`, ramas npm, homebrew, winget y scoop de :79-82) y :223-234 (`dirs_de_instalacion`, el `if !v.contains(&d)` de :229); tests que deberian cubrirlo: crates/tc2/src/instalacion.rs:317 (`assert!(!orden_del_gestor(gestor).is_empty())` en `detectar_modo_con_rutas_de_los_tres_sistemas`), :334-374 (`rutas_del_entorno_por_sistema`) y crates/tc2/tests/desinstalar.rs:760-810 (`plugin_y_gestor_no_borran_el_binario`)
- modulo: instalacion
- severidad: IMPORTANTE
- normalizacion: consolidador TASK-138, 2026-11-22: quita «pendiente-mutacion»: la seccion «Mutación» confirma los mutantes en verde y el control en rojo. La frase «Sin ejecutar: queda pendiente-mutacion» del motivo se sustituyo en la ronda 1 de revision.
- motivo_severidad: Disfraz en test, candidato: el criterio C9 de TASK-078 pide que `tc2 desinstalar` escriba «la orden exacta» que quita un `tc2` de un gestor (desinstalar.rs:757-758), y de las cinco ordenes solo la de cargo tiene un test con su texto (desinstalar.rs:807); las otras cuatro solo se comprueban como «no vacias» (instalacion.rs:317). Si una se rompe (otro paquete, otro gestor), la persona recibe una orden que no quita nada o quita otra cosa, y la suite sigue en verde. Ademas, la deduplicacion de `dirs_de_instalacion` (la que evita listar dos veces la misma carpeta cuando `TC2_INSTALL_DIR` es la de por defecto) no la ejercita ningun test: el unico caso pone `TC2_INSTALL_DIR` vacio (:340, :345). No llega a CRITICO: no se pierde ni corrompe nada (con `Decision::Persona` la orden la lanza la persona) y el codigo de hoy es correcto. Confirmado por mutacion (seccion «Mutación»: mutantes en verde y control en rojo).
- evidencia:

  Codigo, `crates/tc2/src/instalacion.rs:77-86`
  ```rust
  pub fn orden_del_gestor(gestor: &str) -> String {
      match gestor {
          "npm" => "npm uninstall -g taskcodetwo",
          "homebrew" => "brew uninstall taskcodetwo",
          "winget" => "winget uninstall taskcodetwo",
          "scoop" => "scoop uninstall taskcodetwo",
          "cargo" => "cargo uninstall tc2",
          _ => "",
      }
      .to_owned()
  ```
  y `:225-232`
  ```rust
          for d in [self.install_dir.clone(), self.dir_por_defecto()]
              .into_iter()
              .flatten()
          {
              if !v.contains(&d) {
                  v.push(d);
              }
          }
  ```

  Tests: `crates/tc2/src/instalacion.rs:316-317`
  ```rust
              assert_eq!(detectar_modo(ruta), Modo::Gestor(gestor), "{ruta}");
              assert!(!orden_del_gestor(gestor).is_empty(), "{gestor}");
  ```
  y `crates/tc2/tests/desinstalar.rs:806-808`, el unico texto exacto: `"clase=binario dueno=gestor decision=persona motivo=cargo orden=cargo%20uninstall%20tc2"`. `git grep -n "uninstall -g\|brew uninstall\|winget uninstall\|scoop uninstall" d82ea9d -- crates` solo da instalacion.rs:79-82. `git grep -n "TC2_INSTALL_DIR\|install_dir" d82ea9d -- crates/tc2/tests/desinstalar.rs` da :70 (`None`) y :375 (`Some(propio)`, una carpeta distinta de la de por defecto). La duplicacion tendria efecto: `desinstalar/maquina.rs:77` recorre `e.dirs_de_instalacion()` y empuja una `Accion` por carpeta (:86-88 y siguientes), asi que la misma carpeta daria dos veces sus lineas (por lectura).

  Entrada con la que el codigo roto no se detectaria: `orden_del_gestor("npm")` con cualquier texto no vacio; `Entorno` de Windows con `TC2_INSTALL_DIR` = `%LOCALAPPDATA%\taskcodetwo\bin`.

  Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da.
  Mutante 1 (orden de npm equivocada):
  ```diff
  --- a/crates/tc2/src/instalacion.rs
  +++ b/crates/tc2/src/instalacion.rs
  @@ -79 +79 @@
  -        "npm" => "npm uninstall -g taskcodetwo",
  +        "npm" => "npm uninstall -g tc2",
  ```
  Mutante 2 (sin deduplicar):
  ```diff
  @@ -229,3 +229 @@
  -            if !v.contains(&d) {
  -                v.push(d);
  -            }
  +            v.push(d);
  ```
  Los dos compilan por lectura (el 2 deja `v` igual de tipado; ningun `unused`).
  Ordenes: `cargo test -p tc2 --lib instalacion::tests` y `cargo test -p tc2 --test desinstalar`
  Esperado si es Disfraz: `test result: ok.` en las dos, con cada mutante puesto.
  Contraste en rojo: con `"cargo" => "cargo uninstall x",` en :83, `cargo test -p tc2 --test desinstalar plugin_y_gestor_no_borran_el_binario` tiene que fallar (la asercion de desinstalar.rs:806-808).
  Arreglo del test: en el bucle de :301-318, `assert_eq!(orden_del_gestor(gestor), <orden esperada>)` por cada gestor (una columna mas en la tabla); y un caso en `rutas_del_entorno_por_sistema` con `TC2_INSTALL_DIR` igual a `C:/u/L/taskcodetwo/bin` que exija un solo elemento en `dirs_de_instalacion()`.
- estado: confirmado
- grupo_seguimiento: A8 — Transversal
- absorbe: (ninguno); relaciona A8-15 (mismo modulo y mismo test, otras ramas sin cubrir de `detectar_modo`)

##### Mutación

- Resultado: **confirmado** (los dos mutantes sobreviven en verde en las dos órdenes y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` en `task-138-mutantes/target`). Clon restaurado con `git checkout -- .` entre pasos; los mutantes se aplicaron de uno en uno.

##### Mutante 1 (orden de npm equivocada)

```diff
--- a/crates/tc2/src/instalacion.rs
+++ b/crates/tc2/src/instalacion.rs
@@ -79 +79 @@
-        "npm" => "npm uninstall -g taskcodetwo",
+        "npm" => "npm uninstall -g tc2",
```

```text
cargo test -p tc2 --lib instalacion::tests
test instalacion::tests::rutas_del_entorno_por_sistema ... ok
test instalacion::tests::detectar_modo_con_rutas_de_los_tres_sistemas ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 397 filtered out; finished in 0.00s

cargo test -p tc2 --test desinstalar
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.22s
```

##### Mutante 2 (sin deduplicar `dirs_de_instalacion`)

```diff
--- a/crates/tc2/src/instalacion.rs
+++ b/crates/tc2/src/instalacion.rs
@@ -229,3 +229 @@
-            if !v.contains(&d) {
-                v.push(d);
-            }
+            v.push(d);
```

```text
cargo test -p tc2 --lib instalacion::tests
test instalacion::tests::rutas_del_entorno_por_sistema ... ok
test instalacion::tests::detectar_modo_con_rutas_de_los_tres_sistemas ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 397 filtered out; finished in 0.00s

cargo test -p tc2 --test desinstalar
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.07s
```

##### Control (código roto de verdad)

```diff
--- a/crates/tc2/src/instalacion.rs
+++ b/crates/tc2/src/instalacion.rs
@@ -83 +83 @@
-        "cargo" => "cargo uninstall tc2",
+        "cargo" => "cargo uninstall x",
```

Orden: `cargo test -p tc2 --test desinstalar plugin_y_gestor_no_borran_el_binario`

```text
test plugin_y_gestor_no_borran_el_binario ... FAILED
thread 'plugin_y_gestor_no_borran_el_binario' (38116) panicked at crates\tc2\tests\desinstalar.rs:806:5:
assertion failed: stdout(&h).contains("clase=binario dueno=gestor decision=persona motivo=cargo orden=cargo%20uninstall%20tc2")
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.03s
```

<a id="a11-03"></a>

#### A11-03 · D4 · IMPORTANTE · pendiente [A8]

_Ningún test escapa `*`, `_`, `~` ni `&` en `plano`: quitar `&` dejaría pasar `&#x202E;` y la suite sigue en verde_

- id: A11-03
- regla: D4
- ubicacion: crates/tc2/src/dato.rs:73-97 (`plano`, escapes de `'*'` en :76, `'_'` en :77, `'~'` en :86 y `'&'` en :88); tests que deberian cubrirlo: crates/tc2/src/dato.rs:207-227 (`plano_deja_el_texto_inerte`), que es donde remite crates/tc2/src/hallazgos/mod.rs:2176 («Las reglas de `plano` se prueban en `dato`»)
- modulo: dato (A8; lo usan hallazgos y plan, F6-7)
- severidad: IMPORTANTE
- normalizacion: consolidador TASK-138, 2026-11-22: quita «pendiente-mutacion»: la seccion «Mutación» confirma los mutantes en verde y el control en rojo. La frase «Sin ejecutar: queda pendiente-mutacion» del motivo se sustituyo en la ronda 1 de revision.
- motivo_severidad: Disfraz en test, candidato: `plano` es la regla unica para meter texto ajeno (del PR, de la tarea o del modelo) en Markdown sin que cambie su forma (cabecera dato.rs:1-3, ADR F1-8 adenda F6-7), y su doc promete que no abre formato ni HTML (:48-53). De sus 17 caracteres escapados, al menos cuatro no aparecen escapados en ningun test del repo: `*` y `_` (enfasis), `~` (tachado) y `&` (entidades HTML). Quitar `'&'` de la lista dejaria pasar `&#x202E;`, que GitHub pinta como U+202E: justo el cambio de direccion que `plano` quita cuando llega como caracter (:55, test :209-216). La suite seguiria en verde. No llega a CRITICO: el codigo de hoy escapa los cuatro; el test no tapa un fallo existente. Confirmado por mutacion (seccion «Mutación»: mutantes en verde y control en rojo).
- evidencia:

  Codigo, `crates/tc2/src/dato.rs:73-77` y `:86-89`
  ```rust
          if matches!(
              c,
              '\\' | '`'
                  | '*'
                  | '_'
  ...
                  | '~'
                  | '#'
                  | '&'
                  | '@'
  ```

  Lo que si prueban los tests: `crates/tc2/src/dato.rs:212` (`\@`, `\://`, `www\.`, `\$`, `\:smile:`) y `:225` (`\#\#`); `crates/tc2/src/hallazgos/mod.rs:2173` (`\|`, `\!`, `\[`, `\]`, `\(`, `\)`, `\<`, `\>`, `\#`, `` \` ``); `crates/tc2/tests/inyeccion.rs:294` (los mismos mas `\@`). Busqueda en todo el crate (herramienta Grep, patron `\\&|\\~|\\_|\\\*`, glob `*.rs`, sobre `crates/tc2` con el arbol en d82ea9d): «No matches found».

  Entrada con la que el codigo roto no se detectaria: `plano("&#x202E;evil **negrita** _x_ ~~t~~", 400)`.

  Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da.
  Mutante 1 (no escapa `&`):
  ```diff
  --- a/crates/tc2/src/dato.rs
  +++ b/crates/tc2/src/dato.rs
  @@ -88 +87,0 @@
  -                | '&'
  ```
  Mutante 2 (no escapa `*`):
  ```diff
  @@ -76 +75,0 @@
  -                | '*'
  ```
  Los dos compilan por lectura (el `matches!` sigue bien formado).
  Ordenes: `cargo test -p tc2 --lib dato::tests`, `cargo test -p tc2 --lib hallazgos::` y `cargo test -p tc2 --test inyeccion`
  Esperado si es Disfraz: `test result: ok.` en las tres, con cada mutante puesto.
  Contraste en rojo: sin el `s.push('\\');` de :96, `cargo test -p tc2 --lib dato::tests::plano_deja_el_texto_inerte` tiene que fallar (el `\@user` de :212).
  Arreglo del test: anadir a `raro` (o a un caso nuevo) `&#x202E; **n** _x_ ~~t~~` con su salida exacta `\&\#x202E; \*\*n\*\* \_x\_ \~\~t\~\~`.
- estado: confirmado
- grupo_seguimiento: A8 — Transversal
- absorbe: (ninguno)

##### Mutación

- Resultado: **confirmado** (los dos mutantes sobreviven en verde en las tres órdenes y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` en `task-138-mutantes/target`). Clon restaurado con `git checkout -- .` entre pasos; los mutantes se aplicaron de uno en uno.

##### Mutante 1 (no escapa `&`)

```diff
--- a/crates/tc2/src/dato.rs
+++ b/crates/tc2/src/dato.rs
@@ -88 +87,0 @@
-                | '&'
```

```text
cargo test -p tc2 --lib dato::tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 397 filtered out; finished in 0.00s

cargo test -p tc2 --lib hallazgos::
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 376 filtered out; finished in 0.06s

cargo test -p tc2 --test inyeccion
test las_respuestas_hostiles_no_cambian_el_veredicto ... ok
test el_prompt_encierra_todo_dato_del_pr ... ok
test la_carpeta_de_la_tarea_y_la_peticion_de_plan_quedan_inertes ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.92s
```

##### Mutante 2 (no escapa `*`)

```diff
--- a/crates/tc2/src/dato.rs
+++ b/crates/tc2/src/dato.rs
@@ -76 +75,0 @@
-                | '*'
```

```text
cargo test -p tc2 --lib dato::tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 397 filtered out; finished in 0.00s

cargo test -p tc2 --lib hallazgos::
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 376 filtered out; finished in 0.17s

cargo test -p tc2 --test inyeccion
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.09s
```

##### Control (código roto de verdad)

```diff
--- a/crates/tc2/src/dato.rs
+++ b/crates/tc2/src/dato.rs
@@ -96 +95,0 @@
-            s.push('\\');
```

Orden: `cargo test -p tc2 --lib dato::tests::plano_deja_el_texto_inerte`

```text
test dato::tests::plano_deja_el_texto_inerte ... FAILED
thread 'dato::tests::plano_deja_el_texto_inerte' (15572) panicked at crates\tc2\src\dato.rs:209:9:
assertion `left == right` failed
  left: "evil zero @user http://x.y www.e.com $x$ :smile: ñ ab c"
 right: "evil zero \\@user http\\://x.y www\\.e.com \\$x\\$ \\:smile: ñ ab c"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.00s
```

<a id="a11-04"></a>

#### A11-04 · D4 · IMPORTANTE · pendiente [A8]

_El brazo de `cadena_json` que escapa los controles U+0000-U+001F no lo pasa ningún test: sin él, `--json` sale con ESC crudo y JSON inválido_

- id: A11-04
- regla: D4
- ubicacion: crates/tc2/src/contrato.rs:336 (`cadena_json`, brazo `c if (c as u32) < 0x20 => \u00XX`); tests que deberian cubrirlo: crates/tc2/src/contrato.rs:367-406 (`linea_de_contrato_y_json_con_las_mismas_claves`, `el_diagnostico_va_a_stderr_solo_si_falla`) y los tests `--json` de doctor y recursos (crates/tc2/src/doctor/mod.rs:524-528, :634-638; crates/tc2/tests/doctor.rs:952-957; crates/tc2/tests/recursos.rs:371-377)
- modulo: contrato (A8; lo usan contrato, doctor, inventario y version)
- severidad: IMPORTANTE
- normalizacion: consolidador TASK-138, 2026-11-22: quita «pendiente-mutacion»: la seccion «Mutación» confirma los mutantes en verde y el control en rojo. La frase «Sin ejecutar: queda pendiente-mutacion» del motivo se sustituyo en la ronda 1 de revision.
- motivo_severidad: Disfraz en test, candidato: el contrato (ADR F1-4, cabecera contrato.rs:3-6) promete stdout sin ANSI y, con `--json`, un objeto JSON en una linea; los controles U+0000-U+001F distintos de `\n`, `\r` y `\t` los escapa solo el brazo de :336, y ningun test pasa por el con un valor que lleve uno de esos controles. Los tests `--json` que miran «sin ESC» (`!contains('\u{1b}')`) lo hacen sobre entradas que no traen ESC. Si el brazo se quita, un valor ajeno con ESC o BEL (el nombre de una skill leido del disco, inventario.rs:118-125; una ruta de `desinstalar`, contrato.rs:285) sale crudo en `--json`: JSON invalido (RFC 8259 exige escapar U+0000-U+001F) y ANSI en stdout. No llega a CRITICO: el codigo de hoy escapa; el test no tapa un fallo existente. Confirmado por mutacion (seccion «Mutación»: mutantes en verde y control en rojo).
- evidencia:

  Codigo, `crates/tc2/src/contrato.rs:329-338`
  ```rust
      for c in t.chars() {
          match c {
              '"' => s.push_str("\\\""),
              '\\' => s.push_str("\\\\"),
              '\n' => s.push_str("\\n"),
              '\r' => s.push_str("\\r"),
              '\t' => s.push_str("\\t"),
              c if (c as u32) < 0x20 => s.push_str(&format!("\\u{:04x}", c as u32)),
              c => s.push(c),
          }
  ```

  Ningun test espera un `\u00XX` en una salida: busqueda en `crates/` (herramienta Grep, patron `u001b|u0007|u001B`) solo da dos ENTRADAS de la API simulada (`crates/tc2/tests/plataforma.rs:456`, `crates/tc2/tests/plataforma_gitlab.rs:458`), cuyo `message` no llega a las claves del informe (esperado: `"identidad error motivo=http estado=500"`, plataforma.rs:457). Las entradas con ESC/BEL de los tests (`\u{1b}`, `\u{7}`: solape.rs:701-703, tests/solape.rs:786, progreso/linea.rs:243-249, tablero.rs:940, ciclo.rs:311, hallazgos/mod.rs:2289) van a la linea de texto o a textos saneados, no a `cadena_json` (Grep `\\u\{1b\}|\\u\{7\}|...` en `crates/`). `cadena_json` lo llaman contrato.rs:172-195 y :285-292, doctor/mod.rs:163-174, inventario.rs:118-125 y version.rs:46-49 (`git grep -n cadena_json d82ea9d -- crates/tc2/src`).

  Entrada con la que el codigo roto no se detectaria: `Informe::nuevo("x", Resultado::Ok).con("k", "a\u{1b}[31mb").json()`.

  Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da.
  Mutante (los controles salen tal cual):
  ```diff
  --- a/crates/tc2/src/contrato.rs
  +++ b/crates/tc2/src/contrato.rs
  @@ -336 +335,0 @@
  -            c if (c as u32) < 0x20 => s.push_str(&format!("\\u{:04x}", c as u32)),
  ```
  Compila por lectura (el brazo `c => s.push(c)` cubre el resto).
  Orden: `cargo test -p tc2` (el crate entero: no hay un test concreto que deberia cogerlo, y los `--json` estan repartidos en lib, doctor, recursos y desinstalar).
  Esperado si es Disfraz: `test result: ok.` en todos los binarios de test, con el mutante puesto.
  Contraste en rojo: con `'"' => s.push('"'),` en :331, `cargo test -p tc2 --lib contrato::tests::el_diagnostico_va_a_stderr_solo_si_falla` tiene que fallar (exige `\"algo\"`, :404).
  Arreglo del test: en `linea_de_contrato_y_json_con_las_mismas_claves`, una clave con `"a\u{1b}[31m\u{7}b"` que exija `"a\u001b[31m\u0007b"` en el JSON y `serde_json::from_str` correcto.
- estado: confirmado
- grupo_seguimiento: A8 — Transversal
- absorbe: (ninguno)

##### Mutación

- Resultado: **confirmado** (el mutante sobrevive en verde en todo el crate y el control da rojo).
- Commit base: d82ea9dba1f77577d39b32a4f5caea2e140ae8da (clon en `pruebas-claude/taskcodetwo/task-138-mutantes/repo`, `CARGO_TARGET_DIR` en `task-138-mutantes/target`). Clon restaurado con `git checkout -- .` entre pasos.

##### Mutante (los controles salen tal cual)

```diff
--- a/crates/tc2/src/contrato.rs
+++ b/crates/tc2/src/contrato.rs
@@ -336 +335,0 @@
-            c if (c as u32) < 0x20 => s.push_str(&format!("\\u{:04x}", c as u32)),
```

Orden: `cargo test -p tc2` (47 binarios de test: lib, main, los 44 de `tests/` y doc-tests). Los 47 dan `test result: ok`: 763 pasan, 0 fallan, 13 ignorados (los `#[ignore]` de siempre). Extracto:

```text
Running unittests src\lib.rs
test result: ok. 397 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 13.22s
Running tests\desinstalar.rs
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.22s
Running tests\doctor.rs
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 157.47s
Running tests\recursos.rs
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.83s
Running tests\verificar.rs
test result: ok. 15 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 15.26s
Doc-tests tc2
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

##### Control (código roto de verdad)

```diff
--- a/crates/tc2/src/contrato.rs
+++ b/crates/tc2/src/contrato.rs
@@ -331 +331 @@
-            '"' => s.push_str("\\\""),
+            '"' => s.push('"'),
```

Orden: `cargo test -p tc2 --lib contrato::tests::el_diagnostico_va_a_stderr_solo_si_falla`

```text
test contrato::tests::el_diagnostico_va_a_stderr_solo_si_falla ... FAILED
thread 'contrato::tests::el_diagnostico_va_a_stderr_solo_si_falla' (40056) panicked at crates\tc2\src\contrato.rs:402:9:
assertion failed: j.stdout.ends_with(",\"diagnostico\":\"di \\\"algo\\\"\"}\n")
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 398 filtered out; finished in 0.00s
```

### A10 — Scripts de CI e instaladores

<a id="a10-01"></a>

#### A10-01 · D4 · IMPORTANTE · pendiente [A10]

_`! cmp -s` bajo `set -eu` nunca para: la guarda del ensayo de `actualizar --cli` no comprueba nada_

- id: A10-01
- regla: D4
- ubicacion: .github/scripts/probar-actualizar.sh:60 (y su `set -eu` de :19)
- modulo: scripts de CI (release.yml, job de ensayo de `tc2 actualizar --cli`, TASK-027)
- severidad: IMPORTANTE
- normalizacion: consolidador TASK-138, 2026-11-22: sube de MENOR a IMPORTANTE. Es Disfraz real: la guarda de :60 nunca para (reproducido con el arnes d4-cmp.sh), y la regla 4D del plan (decision 4) fija D4 real >= IMPORTANTE; que hoy no tape un fallo ya existente impide CRITICO, no baja de IMPORTANTE.
- motivo_severidad: Disfraz en script: la linea pretende abortar el ensayo si el `SHA256SUMS` alterado («malo») saliera igual que el bueno, pero en POSIX `set -e` no actua sobre una tuberia negada con `!`, asi que la guarda nunca para nada. Hoy no tapa un fallo real: el `awk` de :58 cambia siempre el primer caracter, y si ambos ficheros salieran vacios el paso 2 fallaria despues por otro motivo (no daria `motivo=sha256-no-casa`). Por eso no llega a CRITICO; es IMPORTANTE por ser D4 real (ver `normalizacion`).
- evidencia:

  `.github/scripts/probar-actualizar.sh:19` y `:57-60`
  ```sh
  set -eu
  ...
  cp "$tmp"/srv/y/tc2-* "$tmp/srv/malo/"
  awk '{ c = substr($1, 1, 1); print (c == "0" ? "1" : "0") substr($1, 2) "  " $2 }' \
    "$tmp/srv/y/SHA256SUMS" >"$tmp/srv/malo/SHA256SUMS"
  ! cmp -s "$tmp/srv/y/SHA256SUMS" "$tmp/srv/malo/SHA256SUMS"
  ```

  Entrada con la que no hace el trabajo: dos `SHA256SUMS` identicos. Arnes `tareas/03-en-revision/TASK-138/seguimiento/arneses/a10/d4-cmp.sh` (recibe como argumento el fichero de `git show d82ea9d:.github/scripts/probar-actualizar.sh`; copia la linea 60 tal cual, comprueba que es identica a `sed -n 60p` del commit auditado, y la ejecuta con `set -eu` sobre dos ficheros iguales), ejecutado con `sh` (GNU bash 5.3.15 en modo sh) en una carpeta temporal bajo task-138, sin red:
  ```
  linea 60: ! cmp -s "$tmp/srv/y/SHA256SUMS" "$tmp/srv/malo/SHA256SUMS"
  (la linea ejecutada es identica a la 60)
  SIGUE: los ficheros son iguales y set -e no paro (estado de la linea: 1)
  salida=0
  ```
  (Corregido en la ronda 1 de revision: la primera version del arnes rotulaba como «estado de la linea» el `$?` de la comprobacion `[ ... ] && echo` posterior, que daba 0. El arnes guarda ahora el `$?` justo despues de la linea 60: con dos ficheros iguales `! cmp -s` da 1 y `set -e` no para.)
  POSIX (`set -e`): no se aplica a una tuberia que empieza por `!`. El mismo script usa la forma buena en otra parte (`probar-instalador.sh:205`: `! grep -qF '/viejo' ... || rojo ...`).

  Version correcta (1 linea): `! cmp -s "$tmp/srv/y/SHA256SUMS" "$tmp/srv/malo/SHA256SUMS" || rojo "el SHA256SUMS alterado es igual al bueno"`.
- estado: confirmado (reproducido)
- grupo_seguimiento: scripts-de-ci (A10)
- absorbe: -

<a id="a10-02"></a>

#### A10-02 · D4 · IMPORTANTE · pendiente [A10]

_El modo `publico` de `probar-instalador.sh` imprime VERDE si el instalador sale 0, sin comprobar binario ni versión_

- id: A10-02
- regla: D4
- ubicacion: .github/scripts/probar-instalador.sh:268-287 (`sin_version`), con `linea_unix` en :45-51 (tuberia `curl ... | sh` sin `pipefail`); afirmacion que no cumple: :12-14
- modulo: scripts de CI (release.yml, job `verificar-instaladores`, modo `publico`, TASK-026)
- severidad: IMPORTANTE
- normalizacion: consolidador TASK-138, 2026-11-22: sube de MENOR a IMPORTANTE. Es Disfraz real: el modo publico imprime VERDE sin comprobar binario ni version (reproducido con el arnes d4-sin-version.sh); D4 real >= IMPORTANTE (plan, decision 4).
- motivo_severidad: Disfraz en script: la cabecera dice que el modo `publico` «comprueba que sin TC2_VERSION instala la ultima estable», pero si el instalador sale con 0 la funcion no comprueba nada (ni que haya binario, ni su version, ni que sea la ultima estable) y da VERDE. Ademas, `linea_unix` es `curl ... | sh` sin `pipefail`: si curl falla, `sh` lee una entrada vacia y sale 0. MENOR y no IMPORTANTE porque en `publico` corre antes `medir` con la misma URL (un `install.sh` inalcanzable ya falla alli) y porque `install.sh` comprueba `tc2 --version` antes de dejar el binario; lo que queda sin comprobar es «la ultima estable». La tabla de evidencia de TASK-026 muestra que la rama de exito nunca se ejercio («Sin TC2_VERSION, los 8 casos fallan»).
- evidencia:

  `.github/scripts/probar-instalador.sh:12-14`
  ```sh
  # - publico: la linea real contra la release publicada. Mide la instalacion y
  #   comprueba que sin TC2_VERSION instala la ultima estable o, si no la hay,
  #   falla diciendo que se use TC2_VERSION.
  ```
  `.github/scripts/probar-instalador.sh:277-286`
  ```sh
    codigo=$?
    set -e
    echo "$salida"
    if [ "$codigo" -ne 0 ]; then
      case "$salida" in
        *"no hay ninguna release estable"*TC2_VERSION*) ;;
        *) rojo "sin TC2_VERSION fallo sin decir que se use TC2_VERSION" ;;
      esac
    fi
    echo "probar: VERDE: sin TC2_VERSION, ultima estable o error con el arreglo"
  ```
  `.github/scripts/probar-instalador.sh:46-47`
  ```sh
    if command -v curl >/dev/null 2>&1; then
      curl -fsSL "$url/install.sh" | HOME="$1" SHELL=/bin/sh sh
  ```

  Entrada con la que no hace el trabajo: un instalador que sale 0 sin instalar nada (p. ej. curl falla y la tuberia da el 0 de `sh`). Arnes `tareas/03-en-revision/TASK-138/seguimiento/arneses/a10/d4-sin-version.sh` (recibe como argumento el fichero de `git show d82ea9d:.github/scripts/probar-instalador.sh`): carga las lineas 268-287 del commit auditado tal cual (`sed -n '268,287p'`), con `so=unix` y `linea_unix() { false | HOME="$1" SHELL=/bin/sh sh; }` (lo que da la linea 47 cuando curl falla). Ejecutado con `sh` en una carpeta temporal bajo task-138, sin red:
  ```

  probar: VERDE: sin TC2_VERSION, ultima estable o error con el arreglo
  binario instalado: NINGUNO
  salida=0
  ```

  Version que si comprueba (3 lineas tras el `fi`): con `codigo` 0, exigir `[ -x "$tmp/home-ultima/.local/bin/tc2" ]` (o `dest-ultima/tc2.exe`) y que su `--version` sea la del tag al que redirige `/releases/latest`.
- estado: confirmado (reproducido)
- grupo_seguimiento: scripts-de-ci (A10)
- absorbe: -

<a id="a10-05"></a>

#### A10-05 · D4 · IMPORTANTE · pendiente [A10]

_La rama `wget` de `install.sh` no limita la redirección a https como promete el comentario_

- id: A10-05
- regla: D4
- ubicacion: instalar/install.sh:47-56 (`descargar`, rama `wget` de :51-52)
- modulo: instalador de una linea de Unix (TASK-026, F2-11)
- severidad: IMPORTANTE
- verificado: no (lectura; conducta de wget sin reproducir)
- normalizacion: consolidador TASK-138, 2026-11-22: sube de MENOR a IMPORTANTE y queda NO VERIFICADO: la promesa del comentario (solo redirecciones a https) no se cumple en la rama wget por lectura, pero la conducta de wget no se reprodujo (no hay wget y no se descarga nada). La tarea de seguimiento lo reproduce primero; si wget no sigue la redireccion a http, el caso se descarta.
- motivo_severidad: Disfraz en codigo: el comentario de la funcion promete «solo se aceptan redirecciones a https» y la rama de curl lo cumple (`--proto-redir '=https'`), pero la rama de wget, la que usan los sistemas sin curl (alpine, que release.yml prueba asi: «alpine (sin curl, sin bash)», `.github/workflows/release.yml:174`), no pasa ninguna restriccion de protocolo. No llega a CRITICO, y es IMPORTANTE NO VERIFICADO (ver `normalizacion`): `SHA256SUMS` y el binario llegan por el mismo canal, asi que la restriccion protege poco frente a quien ya controla la primera respuesta https; y `tc2 actualizar --cli` si la cumple (`crates/tc2/src/descarga.rs:216-222`, `redireccion_permitida`: https y lista de hosts).
- evidencia:

  `instalar/install.sh:47-56`
  ```sh
  # descargar <url> <fichero>: solo se aceptan redirecciones a https.
  descargar() {
    if hay curl; then
      curl -fsSL --proto-redir '=https' --retry 3 -o "$2" "$1"
    elif hay wget; then
      wget -q -O "$2" "$1"
    else
      fallo "no hay curl ni wget" "instala curl (apt-get install curl, apk add curl...) y repite"
    fi
  }
  ```
  Entrada con la que no hace el trabajo: un sistema sin curl (alpine) y una URL cuya respuesta redirige a `http://`. La linea de wget no lleva opcion que limite el protocolo de la redireccion; la de curl si. La conducta por defecto de wget (seguir la redireccion) no se ha reproducido aqui: `command -v wget` no da nada en esta maquina y la regla de la auditoria prohibe descargar herramientas y usar la red. Queda para el revisor por pares o la tarea de seguimiento, con un servidor en 127.0.0.1 que responda 302 a otra URL http y `install.sh` con curl fuera del PATH.

  Version que cumple el comentario: en la rama de wget, rechazar la redireccion (p. ej. `wget --max-redirect=0` y seguir a mano solo `Location: https://...`) o exigir curl y decirlo en el arreglo.
- estado: confirmado por lectura; reproduccion pendiente
- grupo_seguimiento: instaladores (A10)
- absorbe: -

<a id="a11-01"></a>

#### A11-01 · D4 · IMPORTANTE · pendiente [A10]

_`! cmp -s` en mitad del paso «vY» de `release.yml` nunca para: la guarda de que el `sed` cambió la versión no comprueba nada (mismo patrón que A10-01)_

- id: A11-01
- regla: D4
- ubicacion: .github/workflows/release.yml:284 (guarda `! cmp -s Cargo.toml Cargo.toml.orig` del paso «vY, el mismo codigo con la version siguiente», :276-289; shell por defecto `bash` en :47-49)
- modulo: workflow de release (job `actualizar-ensayo`, TASK-027, F2-12)
- severidad: IMPORTANTE
- normalizacion: consolidador TASK-138, 2026-11-22: caso propio, relacionado con A10-01 (mismo patron `! cmp -s` en mitad del paso, otro fichero): va en el mismo grupo A10 y la misma tarea corrige los dos.
- motivo_severidad: Disfraz en CI, el mismo defecto que A10-01 en otro fichero: la linea pretende parar el ensayo si el `sed` de :283 no cambio la version de `Cargo.toml` (la «vY» saldria con la misma version que vX), pero un comando negado con `!` no dispara `-e` en bash, y como no es la ultima orden del paso (le siguen `rm`, `cargo update`, `cargo xtask empaquetar` y el `echo VY=`), el paso sigue y termina en verde. Hoy no tapa un fallo real: si el `sed` no casara, `VY` diria `x.y.(z+1)` pero el binario de `vy/` llevaria la version de vX, y `probar-actualizar.sh` fallaria despues con otro motivo, lejos de la causa. Por la regla del plan (decision 4, normalizacion del consolidador en A10-01), D4 real es IMPORTANTE como minimo; no CRITICO porque no deja pasar una release mala (falla despues).
- evidencia:

  `.github/workflows/release.yml:47-49` y `:280-285`
  ```yaml
  defaults:
    run:
      shell: bash
  ...
          run: |
            base="${VERSION%%-*}"
            vy="$(echo "$base" | awk -F. '{ print $1 "." $2 "." $3 + 1 }')"
            sed -i.orig "s/^version = \"$VERSION\"$/version = \"$vy\"/" Cargo.toml
            ! cmp -s Cargo.toml Cargo.toml.orig
            rm Cargo.toml.orig
  ```

  Entrada con la que no hace el trabajo: un `Cargo.toml` cuya linea `version = "..."` no casa con `VERSION` (el `sed` no cambia nada). Arnes `tareas/03-en-revision/TASK-138/seguimiento/arneses/a11/d4-release-284.sh`: toma las lineas 281-285 con `git show d82ea9d:.github/workflows/release.yml | sed -n 281,285p` (fichero `arneses/a11/release-281-285.txt`, junto al arnes; el contraste usa `release-208-214.txt`), les quita la sangria, anade un `echo SIGUE` y lo ejecuta con el shell que GitHub Actions usa para `shell: bash` (`bash --noprofile --norc -eo pipefail`), GNU bash 5.3.15, sin red y sin cargo, en una carpeta temporal bajo task-138:
  ```
  SIGUE: el sed no cambio Cargo.toml y la guarda de la linea 284 no paro
  salida=0
  Cargo.toml sin cambios
  ```
  Contraste en el mismo arnes: las lineas 208-214 del mismo workflow, donde `! cmp -s SHA256SUMS malo/SHA256SUMS` es la ULTIMA orden del paso, con dos ficheros iguales: `salida contraste=1` (el paso si falla, porque el estado del paso es el de su ultima orden). Por eso :214 no es hallazgo y :284 si.

  Version correcta (1 linea): `! cmp -s Cargo.toml Cargo.toml.orig || { echo "::error::el sed no cambio la version de Cargo.toml"; exit 1; }` (o `cmp -s ... && exit 1`).

  Comparado contra todo el repo (`git grep -n "! cmp -s" d82ea9d`): release.yml:214 (ultima orden: funciona), release.yml:284 (este caso) y .github/scripts/probar-actualizar.sh:60 (A10-01).
- estado: confirmado (reproducido con el arnes)
- grupo_seguimiento: scripts-de-ci (A10)
- absorbe: (ninguno); relaciona A10-01 (mismo patron en probar-actualizar.sh:60; la tarea de scripts-de-ci puede corregir los dos)

### Abstracción común — soporte de tests compartido (tests/comun y módulos `prueba`)

<a id="a8-02"></a>

#### A8-02 · D1 · IMPORTANTE divergente · pendiente [XT]

_Lanzador `CARGO_BIN_EXE_tc2` reescrito en 26 ficheros de test y divergente en qué tokens quita del entorno_

- id: A8-02
- regla: D1
- ubicacion: crates/tc2/tests/aprobar.rs:78-90 ~ inyeccion.rs:161-173 ~ plan.rs:48-60 (`lanzar`, variante con `TC2_IDIOMA`); crates/tc2/tests/ciclo.rs:58-69 ~ liberar.rs:59-70 ~ toma.rs:63-74 (`lanzar`, sin `TC2_IDIOMA`); crates/tc2/tests/alta.rs:29-38 ~ importar.rs:20-29 (`tc2`, sin quitar tokens); crates/tc2/tests/estimacion.rs:215-227, progreso.rs:51-64, solape.rs:53-65, siguiente.rs:18-28, roadmap.rs:22-37, git_binario.rs:15-27, tablero.rs:30-55, doctor.rs:140-163, recursos.rs:138-140, verificar.rs:149-170, cli.rs:7-22 (las demas variantes de `tc2`)
- modulo: tests de integracion de A2, A3, A4, A5, A6, A7 y A8 (cli.rs); destino comun: crates/tc2/tests/comun/mod.rs (A8)
- severidad: IMPORTANTE (divergente)
- motivo_severidad: D1 real: el lanzador del binario `Command::new(env!("CARGO_BIN_EXE_tc2"))` con el entorno aislado esta reescrito en 26 ficheros de test y las copias ya divergen en lo que aislan: unas quitan `TC2_TOKEN_PERSONA`/`TC2_TOKEN_AGENTE`, cuatro quitan ademas `TC2_TOKEN_REVISOR` y el resto no quita ninguno, de modo que un token de la maquina de quien corre los tests entra en unos ficheros y no en otros. Va primera en su grupo (`divergente`). Codigo de test: no tapa un fallo de datos ni de seguridad del producto (no CRITICO).
- evidencia:

  `crates/tc2/tests/aprobar.rs:78-90` (identica en inyeccion.rs:161-173 y plan.rs:48-60)
  ```rust
  fn lanzar(dir: &Path, entorno: Vec<(OsString, OsString)>, args: &[&str]) -> Output {
      Command::new(env!("CARGO_BIN_EXE_tc2"))
          .current_dir(dir)
          .args(args)
          .envs(entorno)
          .env("LC_ALL", "C")
          .env("GIT_TERMINAL_PROMPT", "0")
          .env("TC2_IDIOMA", "es")
          .env_remove("TC2_TOKEN_PERSONA")
          .env_remove("TC2_TOKEN_AGENTE")
  ```

  `crates/tc2/tests/alta.rs:29-38` (identica en importar.rs:20-29)
  ```rust
  fn tc2(f: &Fixture, dir: &Path, args: &[&str]) -> Output {
      Command::new(env!("CARGO_BIN_EXE_tc2"))
          .current_dir(dir)
          .args(args)
          .envs(f.entorno())
          .env("LC_ALL", "C")
          .env("GIT_TERMINAL_PROMPT", "0")
          .output()
          .expect("lanzar tc2")
  }
  ```

  `crates/tc2/tests/tablero.rs:44-47` (la variante que quita los tres tokens)
  ```rust
          .env_remove("TC2_TOKEN_AGENTE")
          .env_remove("TC2_TOKEN_PERSONA")
          .env_remove("TC2_TOKEN_REVISOR")
          .envs(env.iter().copied())
  ```

  Diff normalizado: el esqueleto `current_dir/args/envs/env("LC_ALL","C")/env("GIT_TERMINAL_PROMPT","0")/output` es comun; difieren en `TC2_IDIOMA` y en los `env_remove`. Recuento por fichero (grep en d82ea9d sobre `crates/tc2/tests/*.rs` que usan `CARGO_BIN_EXE_tc2`): 26 ficheros; 13 con `env_remove("TC2_TOKEN_PERSONA")` (aprobar, ciclo, doctor, estimacion, frescura, inyeccion, liberar, plan, progreso, recursos, solape, tablero, toma); 4 con `TC2_TOKEN_REVISOR` (doctor, frescura, recursos, tablero); sin ningun `env_remove` de tokens: actualizar_cli, alta, cli, convivencia_016, git_binario, hallazgos, importar, plataforma_e2e, revisoras, roadmap, seleccion, siguiente, verificar. Sin `TC2_IDIOMA`: alta, ciclo, convivencia_016, importar, liberar, toma. `tests/comun/mod.rs` no tiene ayudante para lanzar el binario (`crates/tc2/tests/comun/mod.rs:1-334`).

  Extraccion propuesta: `comun::Lanzador` (o `Fixture::tc2(dir, entorno, args)`) que fije `LC_ALL`, `GIT_TERMINAL_PROMPT`, `TC2_IDIOMA` y quite todas las `TC2_TOKEN_*` y `GIT_CEILING_DIRECTORIES` en un solo sitio; cada test anade lo suyo con `.env(..)`.
- estado: confirmado
- grupo_seguimiento: lanzador-tests (tests/comun)
- absorbe: A5-07, parte del lanzador de A3-06; relaciona pistas A2.6, A3.P3, A5.2, A6.4

<a id="a8-11"></a>

#### A8-11 · D1 · IMPORTANTE divergente · pendiente [XT]

_Dos servidores HTTP simulados con `Servidor::nuevo` idéntico y `atender` divergente, además de `comun::http_mock`_

- id: A8-11
- regla: D1
- ubicacion: crates/tc2/tests/identidad.rs:33-98 (`Servidor` y `atender`) ~ crates/tc2/tests/init.rs:31-105 (`Servidor` y `atender`) ~ crates/tc2/tests/comun/http_mock.rs:68-193 (`Mock` y `atender`); `base`/`peticiones` tambien en crates/tc2/tests/actualizar_cli.rs:121-127
- modulo: tests de identidad (A1), tests de init (A7), tests/comun (A8), tests de actualizar_cli (A8)
- severidad: IMPORTANTE (divergente)
- motivo_severidad: D1 real: dos servidores HTTP simulados de GitHub con `Servidor::nuevo` identico (19 lineas) y `atender` que ya diverge, ademas del doble comun `comun::http_mock::Mock`, que `tests/init.rs` usa a la vez en el mismo fichero. El motivo que da `identidad.rs` para tener uno propio (no mandar FIN tras la respuesta, TASK-027) ya lo cumple `http_mock`. Va primera en su grupo (`divergente`). Codigo de test: no tapa un fallo de datos ni de seguridad (no CRITICO).
- evidencia:

  `crates/tc2/tests/identidad.rs:39-48` (identico a `crates/tc2/tests/init.rs:37-46`; la funcion entera 39-57 ~ 37-55, hash 16568ea5)
  ```rust
      fn nuevo(rutas: &[(&str, &str)]) -> Self {
          let l = TcpListener::bind("127.0.0.1:0").unwrap();
          let puerto = l.local_addr().unwrap().port();
          let peticiones: Arc<Mutex<Vec<String>>> = Arc::default();
          let rutas: Arc<BTreeMap<String, String>> = Arc::new(
              rutas
                  .iter()
                  .map(|(r, c)| ((*r).to_owned(), (*c).to_owned()))
                  .collect(),
          );
  ```

  Divergencia en `atender`: `crates/tc2/tests/init.rs:78-87` lee `Content-Length` y el cuerpo, quita la consulta y solo responde a `GET`
  ```rust
          if let Some((k, v)) = l.split_once(':')
              && k.trim().eq_ignore_ascii_case("content-length")
          {
              largo = v.trim().parse().unwrap_or(0);
          }
      }
      let mut cuerpo = vec![0u8; largo];
      let _ = std::io::Read::read_exact(&mut lector, &mut cuerpo);
      let mut partes = primera.split_whitespace();
      let metodo = partes.next().unwrap_or_default().to_owned();
  ```
  mientras `crates/tc2/tests/identidad.rs:74-85` no lee el cuerpo y busca la ruta con su consulta (`rutas.get(&ruta)`): un `POST` con cuerpo o una ruta con `?` se comportan distinto en un doble y en el otro.

  El motivo de `crates/tc2/tests/identidad.rs:4-7` («no cierra la conexion tras una respuesta completa … ~19 s, TASK-027») es el mismo que ya cumple `crates/tc2/tests/comun/http_mock.rs:185-192` («el servidor no manda FIN detras de una respuesta completa … TASK-027, TASK-115»), que ademas registra metodo, cuerpo y cabeceras y responde por `METODO ruta`. `tests/init.rs:21` importa `Mock` y `:108-117` (`github()`, con `Servidor`) y `:1257-1299` (`github_mock()`, con `Mock`) sirven las mismas rutas `/user`, `/repos/o/r/rulesets` y `/repos/o/r/rules/branches/develop` (pista A7.4). `base()`/`peticiones()` identicos (hashes df48066a y 57226a4e) en identidad.rs:59-65, init.rs:57-63 y actualizar_cli.rs:121-127.

  Fuera del alcance de la extraccion: el servidor de `tests/actualizar_cli.rs:54-199` sirve bytes, cortes y redirecciones que `Mock` no sabe hacer, y el de los tests unitarios de `crates/tc2/src/descarga.rs:417-435` vive en otro crate; si `Mock` gana bytes y corte se podrian absorber, pero no es la copia.

  Extraccion propuesta: borrar los dos `Servidor` de identidad.rs e init.rs y usar `comun::http_mock::Mock` (con un `Mock::con_rutas_get(&[(ruta, json)])` para la forma corta).
- estado: confirmado
- grupo_seguimiento: servidor-http-test (tests/comun/http_mock)
- absorbe: (ninguno; pistas A1.1, A7.4)

<a id="a8-12"></a>

#### A8-12 · D1 · IMPORTANTE divergente · pendiente [XT]

_Seis recorridos de carpeta para fotografiar «lo que la orden no puede cambiar», ya distintos en lo que ven_

- id: A8-12
- regla: D1
- ubicacion: crates/tc2/tests/init.rs:211-232 ~ crates/tc2/tests/desinstalar.rs:37-62 ~ crates/tc2/tests/importar.rs:382-396 (`instantanea`); crates/tc2/tests/tablero.rs:303-330 ~ crates/tc2/tests/frescura.rs:317-347 ~ crates/tc2/tests/doctor.rs:971-1000 (`foto`)
- modulo: tests de init y desinstalar (A7), importar (A3), tablero y frescura (A6), doctor (A5)
- severidad: IMPORTANTE (divergente)
- motivo_severidad: D1 real entre cuatro duenos: «lo que la orden no puede cambiar» se fotografia con seis recorridos de carpeta escritos a mano, y ya difieren en lo que ven: la `instantanea` de importar no anota carpetas (una carpeta vacia creada no cambia la foto) y la `foto` de frescura solo lee el primer nivel de `.git/taskcodetwo/` (un fichero en una subcarpeta, como `progreso/`, no cambia la foto), mientras tablero y doctor recorren todo. Son las comprobaciones de «no toca nada» de esos tests: la copia mas debil prueba menos sin que se note. Va primera en su grupo (`divergente`). Codigo de test (no CRITICO).
- evidencia:

  `crates/tc2/tests/tablero.rs:311-321`
  ```rust
      let mut pila = vec![clon.join(".git").join("taskcodetwo")];
      let mut ficheros = Vec::new();
      while let Some(d) = pila.pop() {
          for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
              let p = e.path();
              if p.is_dir() {
                  pila.push(p);
              } else if p.file_name().and_then(|n| n.to_str())
                  != Some(tc2::huella::FICHERO_SINCRONIZACION)
              {
  ```

  `crates/tc2/tests/frescura.rs:332-341`
  ```rust
      let propia = clon.join(".git").join("taskcodetwo");
      let mut ficheros: Vec<PathBuf> = std::fs::read_dir(&propia)
          .into_iter()
          .flatten()
          .flatten()
          .map(|e| e.path())
          .filter(|p| {
              p.file_name().and_then(|n| n.to_str()) != Some(tc2::huella::FICHERO_SINCRONIZACION)
          })
          .collect();
  ```
  (sin recursion: las subcarpetas de `.git/taskcodetwo/` entran como rutas sin contenido y lo de dentro no se mira; `crate::huella::CARPETA_PROGRESO` es una subcarpeta, `crates/tc2/src/huella.rs:66-70`).

  `crates/tc2/tests/importar.rs:384-392` frente a `crates/tc2/tests/init.rs:212-226`: el mismo recorrido recursivo; importar inserta solo ficheros (`m.insert(p.clone(), std::fs::read(&p).unwrap())`), init anota tambien cada carpeta (`m.insert(format!("{rel}/"), Vec::new())`) y desinstalar la marca con `"dir"` y guarda `"<len> <sha256>"` en vez del contenido (A7-06). `crates/tc2/tests/doctor.rs:984-996` es el mismo `pila`/`read_dir` de tablero sin el filtro de `FICHERO_SINCRONIZACION`. `comun::Fixture::huella` (`crates/tc2/tests/comun/mod.rs:105-118`) hace la parte de refs, worktrees y status, pero no la de ficheros.

  Extraccion propuesta: `comun::instantanea(dir, Opciones { carpetas: bool, contenido: Contenido::Bytes|Hash, excluir: &[&str] })` y `Fixture::foto(clon, ..)` sobre ella y sobre `Fixture::huella`.
- estado: confirmado
- grupo_seguimiento: instantanea-de-carpeta (tests/comun)
- absorbe: A7-06; pista A6.7, A7.6

<a id="a8-16"></a>

#### A8-16 · D1 · IMPORTANTE divergente · pendiente [XT]

_Operación de prueba `Escribir`, `salida`, `con_fila` y `preparar` copiadas entre siete ficheros, ya divergentes_

- id: A8-16
- regla: D1
- ubicacion: crates/tc2/tests/ciclo.rs:198-222 ~ crates/tc2/tests/plan.rs:479-503 ~ crates/tc2/tests/solape.rs:131-155 (`struct Escribir` + `impl Operacion`, identicos), variantes ya separadas en crates/tc2/tests/toma.rs:187-211 y crates/tc2/tests/liberar.rs:221-245; crates/tc2/tests/ciclo.rs:81-91 ~ crates/tc2/tests/liberar.rs:82-92 (`salida`, identica; `start` de crates/tc2/tests/toma.rs:172-183 es la misma comprobacion); crates/tc2/tests/ciclo.rs:174-195 ~ crates/tc2/tests/liberar.rs:195-216 (`con_fila`); crates/tc2/tests/ciclo.rs:135-148 ~ crates/tc2/tests/liberar.rs:134-151 ~ crates/tc2/tests/toma.rs:121-136 ~ crates/tc2/tests/plan.rs:101-118 (`preparar`; y aprobar.rs:130, estimacion.rs:266, progreso.rs:79)
- modulo: tests de integracion de A3 (plan, aprobar), A4 (toma, liberar), A6 (solape, estimacion, progreso) y A7 (ciclo); destino comun: crates/tc2/tests/comun/mod.rs (A8)
- severidad: IMPORTANTE (divergente)
- motivo_severidad: D1 real entre cuatro duenos, el resto del escenario «remoto + personas + operacion por la transaccion» que A8-01 y A8-02 no cubren: la operacion de prueba `Escribir` (25 lineas) esta tres veces identica y dos mas ya separadas, `salida` y `con_fila` estan copiadas entre `ciclo` y `liberar`, y `preparar` repite los mismos pasos en siete ficheros. Las copias ya divergen (`con_fila` gano un parametro `por` en una sola; `Escribir` cambio de campos en `toma` y `liberar`), por eso `divergente`. Ningun ADR lo impone. Codigo de test: no tapa un fallo de datos ni de seguridad (no CRITICO).
- evidencia:

  `crates/tc2/tests/ciclo.rs:198-207` (sha1 167a625e del bloque 198-222, igual en plan.rs:479-503 y solape.rs:131-155)
  ```rust
  struct Escribir {
      ruta: String,
      de: &'static str,
      a: &'static str,
  }

  impl Operacion for Escribir {
      fn nombre(&self) -> &str {
          "escribir"
      }
  ```

  `crates/tc2/tests/ciclo.rs:81-90` (identica a liberar.rs:82-92: `diff` vacio)
  ```rust
  fn salida(o: &Output) -> (Option<i32>, String) {
      let out = texto(&o.stdout);
      assert_eq!(out.lines().count(), 1, "{out}");
      let err = texto(&o.stderr);
      if o.status.code() == Some(0) {
          assert!(err.is_empty(), "stderr en un exito: {err}");
      } else {
          assert_eq!(err.lines().count(), 1, "{err}");
      }
      (o.status.code(), out.trim_end().to_owned())
  ```

  `crates/tc2/tests/plan.rs:101-110`
  ```rust
  fn preparar(f: &Fixture, altas: &[&[&str]]) -> (PathBuf, PathBuf) {
      let (bare, prep) = f.remoto_y_clon();
      escribir(&prep, ".taskcode/personas.yml", PERSONAS);
      escribir(&prep, ".taskcode/config.yml", CONFIG);
      f.crudo(&prep, &["add", ".taskcode"]);
      f.crudo(&prep, &["commit", "-q", "-m", "personas y config"]);
      f.crudo(&prep, &["push", "-q", "origin", "develop"]);
      for args in altas {
          let mut v = vec!["new"];
          v.extend_from_slice(args);
  ```

  Diff normalizado:
  - `Escribir`: ninguno entre ciclo, plan y solape (mismo sha1). toma.rs:189 cambia `de`/`a` por `cambiar: Box<dyn Fn(Option<String>) -> String>`; liberar.rs:223-224 por `de: Option<String>, a: String`.
  - `salida`: ninguno. `toma.rs:172-183` (`start`) es la misma comprobacion con otro nombre (Jaccard 0.82 del pase 0).
  - `con_fila`: solo la firma (`por: &str` en liberar.rs:195) y `agente` fijo en ciclo.rs:184 frente a `{por}` en liberar.rs:205.
  - `preparar` (plan ~ toma): plan escribe ademas `config.yml` y hace el clon con `clone` + `remote set-url` a mano; toma usa su `otro_clon`. Los pasos `remoto_y_clon` → `personas.yml` → commit → push → `new` por alta → clon `a` → `identificar(&a, 1001, "ana")` son los mismos en los siete.
  - Por debajo del umbral, viaja con estos: el repo sin remoto de 5 lineas (`init -q -b develop` + README + add + commit) de estado_bordes.rs:46-53 (A4-08) esta inline en frescura.rs:481-485 y roadmap.rs:274-278 (A6).

  Extraccion propuesta: en `tests/comun/mod.rs`, `comun::ops::Escribir` (con un `Fn` para el cambio, que cubre las cinco variantes), `comun::salida_de_contrato(&Output)`, `comun::con_fila(antes, despues, fase, por)` y `Fixture::preparar(personas, config, altas)` que devuelva `(bare, clon_a)`; `Fixture::repo_sin_remoto(nombre)` junto a ellos.
- estado: confirmado
- grupo_seguimiento: lanzador-tests (tests/comun)
- absorbe: A4-09 (sus pares `Escribir`, `start`/`salida` y `preparar` de toma ~ liberar, ampliados a ciclo, plan y solape); relaciona A6-03 (`preparar` de estimacion ~ progreso), A3-06 (`preparar` de aprobar ~ plan) y A4-08; pistas A4.3, A6.4, A7.3

<a id="a8-20"></a>

#### A8-20 · D1 · IMPORTANTE divergente · pendiente [XT]

_Git de sistema para preparar escenarios escrito cuatro veces, divergente en qué aísla_

- id: A8-20
- regla: D1
- ubicacion: crates/tc2/tests/agente_git/mod.rs:40-55 (`Clon::git`, A4) ~ crates/tc2/tests/comun/mod.rs:60-76 (`Fixture::crudo`, A8); y en tests unitarios de src: crates/tc2/src/ciclo/mod.rs:473-485 (`sh`, A7) ~ crates/tc2/src/estado/montaje.rs:336-356 (`sh`, A4), con crates/tc2/src/git/estado.rs:554-566 (`repo`, A4) y la constante `CONFIG` de crates/tc2/src/ciclo/mod.rs:446 igual a crates/tc2/tests/comun/mod.rs:22
- modulo: tests de agente_git y unitarios de estado/git (A4), ciclo (A7), tests/comun (A8)
- severidad: IMPORTANTE (divergente)
- motivo_severidad: D1 real entre tres duenos (concepto «invocacion de git» de la pasada): el «git de sistema para preparar el escenario» (lanzar `git`, exigir exito con el stderr en el mensaje, devolver stdout) esta escrito cuatro veces, dos en integracion y dos en tests unitarios. Las copias ya divergen en lo que aislan: `ciclo` y `comun` aislan la configuracion del usuario (`GIT_CONFIG_NOSYSTEM`, `GIT_CONFIG_GLOBAL`) y fijan `LC_ALL=C`; `montaje` no aisla nada, pasa la identidad por `-c` y crea con `init.defaultBranch=main` (el resto de la suite usa `develop`); `git/estado.rs:558` lanza `git init` sin ningun aislamiento. Un `init.templateDir` o un hook en la configuracion global de quien corre los tests entra en unos y no en otros. Ningun ADR lo impone. Codigo de test (no CRITICO).
- evidencia:

  `crates/tc2/tests/comun/mod.rs:60-69`
  ```rust
      pub fn crudo(&self, dir: &Path, args: &[&str]) -> String {
          let o = Command::new("git")
              .current_dir(dir)
              .args(args)
              .envs(self.entorno())
              .env("LC_ALL", "C")
              .env("GIT_TERMINAL_PROMPT", "0")
              .output()
              .expect("lanzar git");
          assert!(
  ```

  `crates/tc2/tests/agente_git/mod.rs:40-49`
  ```rust
      fn git(&self, dir: &Path, args: &[&str]) -> String {
          let o = Command::new("git")
              .current_dir(dir)
              .args(args)
              .envs(self.env.iter().cloned())
              .env("LC_ALL", "C")
              .env("GIT_TERMINAL_PROMPT", "0")
              .output()
              .expect("lanzar git");
          assert!(
  ```

  `crates/tc2/src/estado/montaje.rs:336-345` frente a `crates/tc2/src/ciclo/mod.rs:473-480`
  ```rust
      fn sh(dir: &Path, args: &[&str]) -> R<String> {
          let s = Command::new("git")
              .current_dir(dir)
              .args([
                  "-c",
                  "user.name=Prueba",
                  "-c",
                  "user.email=prueba@example.com",
                  "-c",
                  "commit.gpgsign=false",
  ```

  Diff normalizado: `crudo` ~ `Clon::git` solo cambian el entorno (`self.entorno()` / `self.env`), el mensaje del assert (`en {dir}`) y que `Clon::git` hace `trim()`; Jaccard 0.86 en el pase 0. `ciclo::sh` ~ `montaje::sh` comparten la cola (`if !s.status.success() { return Err(format!("git {args:?}: {}", ..)) } Ok(String::from_utf8_lossy(&s.stdout).trim().to_owned())`, montaje.rs:352-355 = ciclo/mod.rs:481-484) y difieren en el aislamiento descrito (Jaccard 0.83). La cabecera de `comun/mod.rs:1-4` promete «la configuracion del usuario aislada»; los unitarios de src no pueden usar `tests/comun` (otro crate), y el par interno de `ciclo/mod.rs` ya es A7-05.

  Extraccion propuesta: `Fixture::crudo_con(dir, entorno, args)` en `tests/comun` (que absorbe `Clon::git` y el `git` con `entorno_ssh` de A8-01) y, para los unitarios, un `#[cfg(test)] pub(crate) mod prueba` en `git` con la misma `CONFIG`, `entorno(raiz)` y `sh(raiz, dir, args)` de `ciclo`, que usen `montaje` y `git/estado`.
- estado: confirmado
- grupo_seguimiento: soporte-tests-unitarios (git::prueba) y lanzador-tests (tests/comun)
- absorbe: relaciona A7-05 (escenario git de los unitarios de ciclo) y A8-01 (`git` con `entorno_ssh`); pista A7.5

<a id="a8-05"></a>

#### A8-05 · D1 · IMPORTANTE divergente · pendiente [XT]

_Cinco dobles de `trait Plataforma` reescriben los métodos que no usan y divergen al llamarlos (`sin_uso`, a mano, `panic!`)_

- id: A8-05
- regla: D1
- ubicacion: crates/tc2/src/identidad.rs:649-679 (`Falsa`, test unitario) ~ crates/tc2/src/revision/mod.rs:537-568 (`Puertas`, test unitario) ~ crates/tc2/src/tablero.rs:998-1025 (`Falsa`, test unitario) ~ crates/tc2/tests/revision.rs:68-130 (`Compartida`, integracion) ~ crates/tc2/tests/tablero.rs:119-146 (`Falsa`, integracion)
- modulo: identidad (A1), revision (A2), tablero (A6), tests/revision (A2), tests/tablero (A6)
- severidad: IMPORTANTE (divergente)
- motivo_severidad: D1 real: cinco dobles de `trait Plataforma` reescriben los mismos metodos que no usan (`abrir_pr`, `estado_puertas`, `merge_pr`, `leer_puertas`, `leer_permisos_ci`, `firma_commit`, `identidad`, `prs_abiertos`) y ya divergen en que hacen si se les llama: tres devuelven un `Informe` de error en silencio (`sin_uso`), otro lo construye a mano y `tests/tablero.rs` hace `panic!`. Una llamada inesperada hace fallar el test en uno y pasa desapercibida en los otros. F1-13 fija el trait, no que cada doble lo copie (el descarte de F1-13 cubre `github.rs`/`gitlab.rs`, no los dobles de test). Codigo de test: ni datos ni seguridad del producto (no CRITICO).
- evidencia:

  `crates/tc2/src/identidad.rs:649-666` (identico, salvo el metodo real, en `crates/tc2/src/revision/mod.rs:537-568` y `crates/tc2/tests/revision.rs:68-130`)
  ```rust
      fn sin_uso(op: &str) -> Informe {
          Informe::nuevo(op, Resultado::Error)
      }

      impl Plataforma for Falsa {
          fn prs_abiertos(&self, _: Rol) -> Paso<Vec<crate::plataforma::PrAbierto>> {
              Err(sin_uso("prs-abiertos"))
          }
          fn abrir_pr(&self, _: &str, _: &str, _: &str, _: &str) -> Informe {
              sin_uso("abrir-pr")
  ```

  `crates/tc2/src/tablero.rs:1007-1015`
  ```rust
          fn estado_puertas(&self, _: Rol, _: u64, _: &ModeloPuertas) -> Informe {
              Informe::nuevo("estado-puertas", Resultado::Error)
          }
          fn merge_pr(&self, _: u64, _: &str) -> Informe {
              Informe::nuevo("merge-pr", Resultado::Error)
          }
          fn leer_puertas(&self, _: Rol, _: &str) -> Paso<ReglasLeidas> {
              Err(Informe::nuevo("leer-puertas", Resultado::Error))
          }
  ```

  `crates/tc2/tests/tablero.rs:128-136`
  ```rust
      fn estado_puertas(&self, _: Rol, _: u64, _: &ModeloPuertas) -> Informe {
          panic!("board no lee puertas")
      }
      fn merge_pr(&self, _: u64, _: &str) -> Informe {
          panic!("board no mergea")
      }
      fn leer_puertas(&self, _: Rol, _: &str) -> Paso<ReglasLeidas> {
          panic!("board no lee puertas")
      }
  ```

  `sin_uso` identica en tres sitios (hash 5fec2db4: `identidad.rs:649-651`, `revision/mod.rs:537-539`, `tests/revision.rs:68-70`). Los `impl Plataforma for` del crate son 7 (grep en d82ea9d): `github.rs:401` y `gitlab.rs:527` (descartados, F1-13) y estos cinco dobles. Diff normalizado: cuerpos identicos salvo el metodo que cada doble usa; difieren en `sin_uso(..)` / `Informe::nuevo(..)` / `panic!(..)`.

  Extraccion propuesta: un `PlataformaNula` con el comportamiento elegido (`panic!` con el nombre de la operacion, para que una llamada inesperada se vea) bajo `#[cfg(test)]` en `plataforma` para los unitarios, y otro en `tests/comun` para integracion (dos crates de test distintos no pueden compartir uno), con delegacion del metodo que cada test necesita.
- estado: confirmado
- grupo_seguimiento: dobles-plataforma
- absorbe: A2-06; pistas A2.5, A6.6

<a id="a3-06"></a>

#### A3-06 · D1 · IMPORTANTE divergente · pendiente [XT]

_Resto tras la fusión: `valor` divergente (importar con `trim_end`, alta sin él) y `como_persona` ~ `hito_como_persona`_

- id: A3-06
- regla: D1
- ubicacion: crates/tc2/tests/alta.rs:29-42 y :178-183 ~ crates/tc2/tests/importar.rs:20-41; crates/tc2/tests/aprobar.rs:28-117 ~ crates/tc2/tests/plan.rs:23-85; crates/tc2/tests/aprobar.rs:190-202 ~ :384-396
- modulo: tests de alta, importar, aprobar y plan
- severidad: IMPORTANTE (divergente)
- absorbido_por: A8-01, A8-02 (en parte)
- normalizacion: consolidador TASK-138, 2026-11-22: absorbido en parte: el lanzador `tc2`/`texto` va en A8-02 y `entorno_ssh`/`valor` en A8-01. Queda como caso propio lo que esos dos no recogen: la copia divergente de `valor` (tests/importar.rs:36-41 con `trim_end` frente a tests/alta.rs:179-183) y `como_persona` ~ `hito_como_persona` (tests/aprobar.rs:191-202 ~ :385-396). Su tarea es la de soporte de tests compartido (grupo lanzador-tests).
- motivo_severidad: D1 real en las ayudas de los tests de integracion: el lanzador del binario, el entorno ssh, `texto` y `valor` estan copiados entre ficheros del reparto (y fuera de el, ver `pistas/A3.txt`) en vez de vivir en `tests/comun/`, que ya existe para eso. Una de las copias ya diverge: `valor` de `tests/importar.rs` recorta el final de linea y la de `tests/alta.rs` no, y por eso un test de alta lo hace a mano. No tapa ningun fallo de datos: es mantenimiento de tests.
- estado: confirmado (por lectura contra d82ea9d)
- grupo_seguimiento: A8 (tests/comun) — la abstraccion comun la decide el dueno de `tests/comun/`; A3 pone la evidencia de su reparto

##### Evidencia

1. `fn tc2` identica (10 lineas, 77 tokens; hash del pase 0 `53242c0843`), `crates/tc2/tests/alta.rs:29-38` y `crates/tc2/tests/importar.rs:20-29`:

```rust
fn tc2(f: &Fixture, dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tc2"))
        .current_dir(dir)
        .args(args)
        .envs(f.entorno())
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("lanzar tc2")
}
```

y `fn texto` igual en `alta.rs:40-42` e `importar.rs:31-33`.

2. `fn valor`, copias que ya divergen. `crates/tc2/tests/alta.rs:179-183`:

```rust
fn valor<'a>(linea: &'a str, clave: &str) -> Option<&'a str> {
    linea
        .split(' ')
        .find_map(|kv| kv.strip_prefix(clave)?.strip_prefix('='))
}
```

`crates/tc2/tests/importar.rs:36-41`:

```rust
fn valor<'a>(linea: &'a str, clave: &str) -> Option<&'a str> {
    linea
        .trim_end()
        .split(' ')
        .find_map(|kv| kv.strip_prefix(clave)?.strip_prefix('='))
}
```

Diff: `.trim_end()`. Con la copia de alta, la ultima clave de la linea sale con el `\n` pegado; `tests/alta.rs:393` lo compensa a mano (`valor(out.trim_end(), "motivo")`). `tests/aprobar.rs:113-117` y `tests/plan.rs:81-85` son la copia sin `trim_end` (alli `tc2` ya recorta la salida).

3. `entorno_ssh` (17 lineas) y `lanzar` (13 lineas), identicos en `crates/tc2/tests/aprobar.rs:42-58` / `:78-90` y `crates/tc2/tests/plan.rs:30-46` / `:48-60`, con las mismas constantes `ORIGIN`, `PERSONAS` y `CONFIG` (`aprobar.rs:29-32`, `plan.rs:24-28`) y el mismo arranque de `preparar` (`aprobar.rs:131-136` y `plan.rs:102-107`: `remoto_y_clon`, `escribir` de `personas.yml` y `config.yml`, `add`, `commit`, `push`). Fragmento de `aprobar.rs:43-56` (en `plan.rs:31-44` igual):

```rust
    let bare = f
        .raiz
        .join("remoto.git")
        .to_string_lossy()
        .replace('\\', "/");
    let mut e = f.entorno();
    e.push((
        "GIT_SSH_COMMAND".into(),
        format!(
            "sh -c 'case \"$2\" in *receive-pack*) exec git receive-pack \"$0\";; *) exec git upload-pack \"$0\";; esac' '{bare}'"
```

4. Dentro de `tests/aprobar.rs`: `como_persona` (`:191-202`) y `hito_como_persona` (`:385-396`) son la misma funcion de 11 lineas salvo la llamada final (`aprobar_en(&e, "TASK-001")` frente a `aprobar_hito_en(&e, hito)`).

Arreglo esperado: `lanzar_tc2`, `entorno_ssh`, `texto` y una sola `valor` (con `trim_end`) en `tests/comun/mod.rs`.

<a id="a8-01"></a>

#### A8-01 · D1 · IMPORTANTE · pendiente [XT]

_Escenario «remoto bare con ssh falso» (`entorno_ssh`, `identificar`, `git`...) copiado en 10 ficheros de test en vez de `tests/comun/`_

- id: A8-01
- regla: D1
- ubicacion: crates/tc2/tests/aprobar.rs:42-58 ~ crates/tc2/tests/ciclo.rs:29-45 ~ crates/tc2/tests/estimacion.rs:197-213 ~ crates/tc2/tests/frescura.rs:28-44 ~ crates/tc2/tests/inyeccion.rs:143-159 ~ crates/tc2/tests/liberar.rs:30-46 ~ crates/tc2/tests/plan.rs:30-46 ~ crates/tc2/tests/progreso.rs:31-47 ~ crates/tc2/tests/solape.rs:35-51 ~ crates/tc2/tests/toma.rs:33-49 (`entorno_ssh`); crates/tc2/tests/ciclo.rs:47-56 ~ frescura.rs:47-56 ~ liberar.rs:48-57 ~ toma.rs:52-61 (`entorno_sin_red`); crates/tc2/tests/ciclo.rs:122-131 ~ estimacion.rs:247-256 ~ plan.rs:87-96 ~ progreso.rs:66-75 ~ solape.rs:109-118 ~ toma.rs:107-116 (`identificar`); crates/tc2/tests/ciclo.rs:93-104 ~ solape.rs:72-83 ~ toma.rs:86-97 (`git` con el ssh falso); crates/tc2/tests/ciclo.rs:150-154 ~ liberar.rs:153-157 ~ solape.rs:120-124 ~ toma.rs:139-143 (`otro_clon`); crates/tc2/tests/ciclo.rs:229-238 ~ plan.rs:510-519 ~ solape.rs:162-171 (`editar`); `valor` (7 copias), `texto` (12) y `sin_catalogo` (10), listadas en la evidencia
- modulo: tests de integracion de A2 (inyeccion), A3 (aprobar, plan), A4 (toma, liberar), A6 (estimacion, frescura, progreso, solape) y A7 (ciclo); destino comun: crates/tc2/tests/comun/mod.rs (A8)
- severidad: IMPORTANTE
- motivo_severidad: D1 real entre cinco duenos: el escenario «remoto bare servido por un ssh falso» de los tests de integracion esta copiado byte a byte por fichero en vez de vivir en `tests/comun/`, que ya existe y ya tiene `Fixture::entorno`, `Fixture::crudo` y `Fixture::otro_clon`. Ningun ADR lo impone. Codigo de test: no tapa un fallo de datos ni de seguridad (no CRITICO). Estas copias no divergen todavia (cuerpos con el mismo hash); la familia del lanzador, que si diverge, es A8-02.
- evidencia:

  Agrupado por cuerpo exacto (texto identico, `scripts/a8_fns.py`, mismo sha1 por grupo) contra d82ea9d:
  - `entorno_ssh`: 10 copias identicas de 17 lineas (hash e306782f) en aprobar, ciclo, estimacion, frescura, inyeccion, liberar, plan, progreso, solape y toma.
  - `entorno_sin_red`: 4 copias identicas de 10 lineas (678d4646).
  - `identificar`: 6 copias identicas de 10 lineas (01a739bc); la de `liberar.rs:126-130` ya es otra (e2153157).
  - `git` (Command git con `entorno_ssh`): 3 copias identicas de 12 lineas (d522ead9).
  - `otro_clon` 4 (e99bdbd5), `editar` 3 (c76e2ec1), `valor` 7 (28fb620c: alta, aprobar, ciclo, liberar, plan, solape, toma), `texto` 12 (66559344), `sin_catalogo` 10 (ce74e8a9). Cada una por debajo del umbral por separado; se citan porque viajan con las anteriores.

  `crates/tc2/tests/ciclo.rs:29-38` (igual en las otras nueve)
  ```rust
  fn entorno_ssh(f: &Fixture) -> Vec<(OsString, OsString)> {
      let bare = f
          .raiz
          .join("remoto.git")
          .to_string_lossy()
          .replace('\\', "/");
      let mut e = f.entorno();
      e.push((
          "GIT_SSH_COMMAND".into(),
          format!(
  ```

  `crates/tc2/tests/ciclo.rs:122-131` (igual en estimacion, plan, progreso, solape y toma)
  ```rust
  fn identificar(dir: &Path, id: u64, login: &str) {
      let cuenta = Cuenta {
          proveedor: Proveedor::GitHub,
          id,
          login: login.to_owned(),
      };
      Cache::en(&dir.join(".git"))
          .guardar(&cuenta, tc2::identidad::ahora())
          .unwrap();
  }
  ```

  Diff normalizado: ninguno dentro de cada grupo (texto identico). La cabecera de `tests/ciclo.rs:5` lo reconoce («Como en `tests/toma.rs`»). `crates/tc2/tests/comun/mod.rs:44-52` ya da `Fixture::entorno` y `:60-76` `Fixture::crudo` (el `git` de arriba es `crudo` con otro entorno).

  Extraccion propuesta: `Fixture::entorno_ssh(&self)`, `Fixture::entorno_sin_red(&self)`, `Fixture::git_con(&self, dir, entorno, args)` e `identificar(dir, id, login)` en `tests/comun/mod.rs`, mas un `comun::contrato::{valor, texto}`.
- estado: confirmado
- grupo_seguimiento: lanzador-tests (tests/comun)
- absorbe: A3-06 (parte de `entorno_ssh`, `texto`, `valor`), A4-09; relaciona A6-03 (sus ayudantes compartidos fuera de A6) y pistas A2.6, A3.P3, A4.2, A6.1-4, A7.3

<a id="a8-18"></a>

#### A8-18 · D1 · IMPORTANTE · pendiente [XT]

_Ayudantes `estado` + `motivo` de los tests unitarios de `Operacion` copiados en varios módulos_

- id: A8-18
- regla: D1
- ubicacion: crates/tc2/src/ciclo/operacion.rs:297-311 ~ crates/tc2/src/ciclo/liberar.rs:439-453 (A7) ~ crates/tc2/src/toma/operacion.rs:320-334 (A4) (`estado` + `motivo` de los tests unitarios, identicos); `motivo` tambien en crates/tc2/src/plan/operacion.rs:559-564, crates/tc2/src/plan/aprobar.rs:600-605 (A3) y crates/tc2/src/revision/operacion.rs:230-235 (A2); `struct EnMemoria` + `impl Estado` en crates/tc2/src/siguiente.rs:140-149 ~ crates/tc2/src/roadmap/mod.rs:579-588 (A6)
- modulo: tests unitarios de ciclo (A7), toma (A4), plan (A3), revision (A2), siguiente y roadmap (A6)
- severidad: IMPORTANTE
- motivo_severidad: D1 real entre cuatro duenos en el soporte de los tests unitarios de `Operacion`: el par `estado` (montar un `EnMemoria`) + `motivo` (sacar el motivo de un `Fallo::Rechazada`) es texto identico en tres modulos (15 lineas, mismo sha1) y `motivo` solo, en seis. El propio doble `EnMemoria` ya es `pub(crate)` en `roadmap::tests` y aun asi `siguiente` define el suyo. Ningun ADR lo impone. Codigo de test: no tapa un fallo de datos ni de seguridad (no CRITICO); no `divergente`: las copias son iguales.
- evidencia:

  `crates/tc2/src/ciclo/operacion.rs:297-306` (sha1 c029e373 del bloque 297-311, el mismo en ciclo/liberar.rs:439-453 y toma/operacion.rs:320-334)
  ```rust
      fn estado(tareas: &[(&str, String)]) -> EnMemoria {
          EnMemoria(
              tareas
                  .iter()
                  .map(|(id, t)| (format!("{id}/tarea.md"), t.clone()))
                  .collect::<BTreeMap<_, _>>(),
          )
      }

      fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
  ```

  `crates/tc2/src/siguiente.rs:140-149` (igual a roadmap/mod.rs:579-588 salvo `pub(crate)`)
  ```rust
      struct EnMemoria(BTreeMap<String, String>);

      impl Estado for EnMemoria {
          fn rutas(&self) -> Vec<String> {
              self.0.keys().cloned().collect()
          }
          fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo> {
              Ok(self.0.get(ruta).cloned())
          }
      }
  ```

  Diff normalizado: `estado` + `motivo`, ninguno en las tres copias. `motivo` (6 lineas) tiene el mismo sha1 (8bf22db1) en plan/operacion.rs, revision/operacion.rs, plan/aprobar.rs y toma/operacion.rs. `plan/operacion.rs:544-551` es `estado` con la ruta ya dada (`(*r).to_owned()` en vez de `format!("{id}/tarea.md")`) y `revision/operacion.rs:226-228` monta una sola tarea. Los `md(...)` de cada modulo (ciclo/operacion.rs:286, toma/operacion.rs:308, ciclo/liberar.rs:423, revision/operacion.rs:218, plan/operacion.rs:536, plan/aprobar.rs:586) repiten `Tarea::nueva` → estado → `asignado_a` → `rama_de` → `escribir(&Cuerpo::plantilla())` con parametros distintos: se citan, no cuentan.

  Extraccion propuesta: un modulo `#[cfg(test)] pub(crate) mod prueba` en `estado` con `EnMemoria`, `estado(&[(&str, String)])` y `motivo(&Result<..>)`, que usen los seis modulos y `siguiente`.
- estado: confirmado
- grupo_seguimiento: soporte-tests-unitarios (estado::prueba)
- absorbe: (ninguno); relaciona el par `fn sh`/escenario git de A8-20

### Abstracción común — capa de órdenes y operaciones con dueño

<a id="a8-07"></a>

#### A8-07 · D1 · IMPORTANTE · pendiente [XO]

_Preámbulo de `Operacion::aplicar` con dueño copiado seis veces; `ciclo` ya lo tiene extraído_

- id: A8-07
- regla: D1
- ubicacion: crates/tc2/src/revision/guardar.rs:96-108 ~ crates/tc2/src/revision/operacion.rs:131-143 (A2) ~ crates/tc2/src/plan/guardar.rs:113-125 ~ crates/tc2/src/plan/operacion.rs:366-379 ~ crates/tc2/src/plan/aprobar.rs:520-533 (A3; y la variante sin `dueno` de `aprobable`, crates/tc2/src/plan/aprobar.rs:227-235) ~ crates/tc2/src/toma/operacion.rs:180-189 (A4) ~ crates/tc2/src/ciclo/operacion.rs:77-90 (A7, ya extraido como `fn tarea`)
- modulo: revision, plan, toma, ciclo (operaciones de la rama de estado con dueno)
- severidad: IMPORTANTE
- motivo_severidad: D1 real entre cuatro duenos: el preambulo de `Operacion::aplicar` de toda operacion «con dueno» (limpiar `dueno`, `tarea_de`, `de_otra_persona` y guardar el login de quien la lleva) esta copiado seis veces; `ciclo` ya lo tiene extraido en `fn tarea(&self, estado, orden)` pero es un metodo de su propio tipo y los demas no lo usan. F1-2 fija el punto unico de escritura, no este preambulo. No tapa un fallo de datos ni de seguridad. No `divergente`: cambian el texto del rechazo (`APROBAR_DE_OTRA_PERSONA`, `tomada`/`TOMADA`) y el estado exigido, a proposito.
- evidencia:

  `crates/tc2/src/revision/guardar.rs:97-108`
  ```rust
          *self.dueno.borrow_mut() = None;
          let (t, _) = tarea_de(estado, &self.id)?;
          if let Some((fallo, dueno)) = de_otra_persona(
              &self.id,
              t.asignado_a.as_ref(),
              &self.asignado,
              NOMBRE,
              ("de-otra-persona", &txt::DE_OTRA_PERSONA),
          ) {
              *self.dueno.borrow_mut() = dueno;
  ```

  `crates/tc2/src/ciclo/operacion.rs:77-86`
  ```rust
      fn tarea(&self, estado: &dyn Estado, orden: &str) -> Result<(Tarea, Cuerpo), Fallo> {
          *self.dueno.borrow_mut() = None;
          let (t, c) = tarea_de(estado, &self.id)?;
          if let Some((fallo, dueno)) = de_otra_persona(
              &self.id,
              t.asignado_a.as_ref(),
              &self.asignado,
              orden,
              ("de-otra-persona", &txt::DE_OTRA_PERSONA),
          ) {
  ```

  Diff normalizado: identicos en los seis `aplicar` salvo la constante de la orden (`NOMBRE` / `super::NOMBRE`), el par del rechazo (`plan/aprobar.rs:529` usa `&txt::APROBAR_DE_OTRA_PERSONA`; `toma/operacion.rs:185` usa `("tomada", &txt::TOMADA)`) y que `plan/operacion.rs:367` y `plan/aprobar.rs:521` limpian ademas su `resumen`/`ronda`. Detras, cinco de ellos siguen con el mismo `if t.estado != X { return Err(por_estado(..)) }`. Tambien se repiten los metodos `dueno()` (7 copias identicas de 3 lineas, hash 0cc89cd0) y `rechazo()` (4 copias, b6ff2593) de esos tipos, y el esqueleto `nombre/anotacion/exige_red` de `revision/guardar.rs:75-95` ~ `plan/guardar.rs:92-112` (pista A2.1: la cabecera de revision/guardar.rs dice que sigue el precedente de `plan guardar`).

  Extraccion propuesta: la `fn tarea` de `ciclo/operacion.rs` como funcion libre en `estado/tareas.rs` junto a `de_otra_persona` (`tarea_con_dueno(estado, id, asignado, orden, rechazo, &RefCell<Option<String>>)`), y un tipo `Dueno` con `dueno()` para no repetir el campo.
- estado: confirmado
- grupo_seguimiento: operacion-con-dueno
- absorbe: A2-01, A3-04; pistas A2.1, A2.2

<a id="a8-19"></a>

#### A8-19 · D1 · IMPORTANTE · pendiente [XO]

_Formato de la anotación con dueño escrito y parseado a mano en cada operación_

- id: A8-19
- regla: D1
- ubicacion: crates/tc2/src/revision/operacion.rs:185-196 (A2) ~ crates/tc2/src/toma/operacion.rs:266-282 (A4) ~ crates/tc2/src/ciclo/operacion.rs:241-250 y crates/tc2/src/ciclo/liberar.rs:351-357 (A7) (`desde_anotacion`: leer `nombre|id|cuenta|login|...`); escritura del mismo formato en `fn anotacion` de crates/tc2/src/revision/operacion.rs:116-124, crates/tc2/src/plan/aprobar.rs:506-513 (A3) y crates/tc2/src/toma/operacion.rs:145-152, entre otras
- modulo: revision (A2), plan (A3), toma (A4), ciclo (A7): operaciones con dueno de la rama de estado
- severidad: IMPORTANTE
- motivo_severidad: D1 real entre cuatro duenos: el formato de la anotacion de una operacion con dueno (`<orden>|<id>|<cuenta>|<login>[|<resto>]`) se escribe a mano en cada `anotacion()` (9 veces `self.asignado.login().unwrap_or_default()` en 7 ficheros) y se vuelve a parsear a mano en cada `desde_anotacion`, con el mismo `splitn` → `parse::<u64>` → `Asignado::persona` → `es_id`. Es el contrato que reconstruye la operacion pendiente desde el catalogo: si un modulo cambia el orden o el separador, solo se entera su propio test de ida y vuelta. Ningun ADR fija un parser por orden. No tapa un fallo de datos hoy (no CRITICO). No `divergente`: cambian a proposito los campos de cola (`ahora`, motivo, actividad).
- evidencia:

  `crates/tc2/src/revision/operacion.rs:185-194`
  ```rust
  pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
      let mut partes = a.splitn(5, '|');
      if partes.next()? != NOMBRE {
          return None;
      }
      let id = partes.next()?;
      let cuenta: u64 = partes.next()?.parse().ok()?;
      let asignado = Asignado::persona(cuenta, partes.next()?)?;
      let ahora: u64 = partes.next()?.parse().ok()?;
      crate::git::es_id(id)
  ```

  `crates/tc2/src/toma/operacion.rs:266-275`
  ```rust
  pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
      let mut partes = a.splitn(4, '|');
      if partes.next()? != NOMBRE {
          return None;
      }
      let id = partes.next()?;
      let cuenta: u64 = partes.next()?.parse().ok()?;
      let asignado = Asignado::persona(cuenta, partes.next()?)?;
      crate::git::es_id(id).then(|| {
          Box::new(Tomar::con(
  ```

  Diff normalizado: `splitn(5)` / `splitn(4)`; revision lee ademas `ahora` del quinto campo y toma lo saca del reloj (`crate::identidad::ahora()`); el resto (nombre, id, cuenta, `Asignado::persona`, `es_id(id).then(Box::new(X::con(id, asignado, "agente", ..)))`) es el mismo. `ciclo/operacion.rs:241-250` hace lo mismo con el nombre como variable (`match nombre`) y `ciclo/liberar.rs:351-357` con `split('|').collect()` e indices (`partes.get(2)?.parse()`), otra forma de la misma lectura. En la escritura, `plan/aprobar.rs:506-513` y `toma/operacion.rs:145-152` son el mismo `format!("{X}|{}|{}|{}", self.id, self.asignado.id().unwrap_or_default(), self.asignado.login().unwrap_or_default())` con otra constante. Los tests de ida y vuelta tambien se copian (`revision/operacion.rs:302-316` ~ `toma/operacion.rs:530-544`, Jaccard 0.86 en el pase 0).

  Extraccion propuesta: un tipo `AnotacionConDueno { orden, id, asignado, resto }` en `estado` (junto a `de_otra_persona`) con `escribir()` y `leer(a, orden) -> Option<(id, Asignado, &str)>`; cada `desde_anotacion` solo interpreta su `resto`.
- estado: confirmado
- grupo_seguimiento: operacion-con-dueno
- absorbe: (ninguno); relaciona A8-07 (mismo grupo: el preambulo de `aplicar`) y A3-01 (D4 del `anotacion_ida_y_vuelta` de `alta`)

<a id="a8-06"></a>

#### A8-06 · D1 · IMPORTANTE · pendiente [XO]

_Conversión `Fallo -> Informe` de la rama de estado escrita a mano en nueve sitios de cinco dueños_

- id: A8-06
- regla: D1
- ubicacion: crates/tc2/src/roadmap/mod.rs:303-313 ~ crates/tc2/src/estimacion/mod.rs:245-254 ~ crates/tc2/src/siguiente.rs:76-85 ~ crates/tc2/src/tablero.rs:443-451 (A6) ~ crates/tc2/src/seleccion/orden.rs:592-602 (A5) ~ crates/tc2/src/ciclo/mod.rs:245-254 (A7) ~ crates/tc2/src/importar/mod.rs:415-428 (A3) ~ crates/tc2/src/revision/mod.rs:233-245 y :249-253 (A2) ~ crates/tc2/src/toma/mod.rs:159-163 (A4)
- modulo: roadmap, estimacion, siguiente, tablero, seleccion, ciclo, importar, revision, toma
- severidad: IMPORTANTE
- motivo_severidad: D1 real entre cinco duenos: la conversion de un `Fallo` de la rama de estado en el `Informe` de la orden (`Rechazada {motivo, diagnostico}` → `rechazada motivo=…` con el diagnostico; `Error(d)` → `error` con el diagnostico) esta escrita a mano en nueve sitios. F1-2 define `Fallo`, no como se pinta. No tapa un fallo de datos ni de seguridad. No `divergente`: las variantes anaden claves a proposito (`id`, `simular`, `asignado`).
- evidencia:

  `crates/tc2/src/seleccion/orden.rs:592-602`
  ```rust
              crate::estado::tarea_de(&v, id).map_err(|f| match f {
                  crate::estado::Fallo::Rechazada {
                      motivo,
                      diagnostico,
                  } => Informe::nuevo(SELECCION, Resultado::Rechazada)
                      .con("motivo", motivo)
                      .diagnostico(diagnostico),
                  crate::estado::Fallo::Error(m) => {
                      Informe::nuevo(SELECCION, Resultado::Error).diagnostico(m)
                  }
  ```

  `crates/tc2/src/ciclo/mod.rs:245-254`
  ```rust
          Err(Fallo::Rechazada {
              motivo,
              diagnostico,
          }) => {
              let i = Informe::nuevo(nombre, Resultado::Rechazada)
                  .con("motivo", motivo)
                  .diagnostico(diagnostico);
              return Err(con_rechazo(i, op.dejar()));
          }
          Err(Fallo::Error(d)) => return Err(Informe::nuevo(nombre, Resultado::Error).diagnostico(d)),
  ```

  `crates/tc2/src/roadmap/mod.rs:303-313` (ya citado en A6-01): `fn de_fallo(f: Fallo) -> Informe { match f { Fallo::Rechazada { motivo, diagnostico, } => Informe::nuevo(NOMBRE, Resultado::Rechazada).con("motivo", motivo).diagnostico(diagnostico), Fallo::Error(d) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d), } }`.

  Diff normalizado: el `match` es el mismo en los nueve; cambian el nombre de la orden y lo que se anade encima (`.con("id", id)` en estimacion, `.con("simular", "si")` en importar, `con_rechazo` en ciclo, `rechazada(..)` + `asignado` en revision, `rechazo_de_toma` en toma). `estado/transaccion.rs:264-276` tiene otro `de_fallo` privado que convierte el rechazo en `error` (no es este concepto). A2-10 (D3, MENOR) marca el de revision por su forma; aqui cuenta como una copia mas.

  Extraccion propuesta: `impl Fallo { pub fn informe(self, comando: &str) -> Informe }` en `estado` (junto a `Fallo`); cada orden anade sus claves sobre el resultado.
- estado: confirmado
- grupo_seguimiento: fallo-a-informe
- absorbe: A6-01 (lo amplia de 4 a 9 sitios y 5 duenos); relaciona A2-10; pistas A2.3, A6.5

<a id="a8-08"></a>

#### A8-08 · D1 · IMPORTANTE · pendiente [XO]

_Adaptador «informe de la transacción -> informe de la orden» cuatro veces_

- id: A8-08
- regla: D1
- ubicacion: crates/tc2/src/revision/mod.rs:201-210 (`de_review`) ~ crates/tc2/src/toma/mod.rs:106-115 (`de_start`) ~ crates/tc2/src/ciclo/mod.rs:423-432 (`de_la_orden`) ~ crates/tc2/src/plan/mod.rs:542-551 (`de_la_orden`); y `rechazada` crates/tc2/src/revision/mod.rs:212-216 ~ crates/tc2/src/toma/mod.rs:117-121
- modulo: revision (A2), toma (A4), ciclo (A7), plan (A3)
- severidad: IMPORTANTE
- motivo_severidad: D1 real entre cuatro duenos: el adaptador «informe de la transaccion → informe de la orden» (renombrar el comando, quitar `operacion` e `id`, poner `id=` el primero y `pendiente=no` en un `sinred` sin `pendiente`) esta cuatro veces, dos de ellas con el mismo nombre y los argumentos en distinto orden. Ningun ADR lo impone (F1-4 fija la linea, no cuatro funciones). No tapa un fallo de datos ni de seguridad. No `divergente`: hoy dan lo mismo.
- evidencia:

  `crates/tc2/src/revision/mod.rs:201-210`
  ```rust
  fn de_review(mut i: Informe, id: &str) -> Informe {
      NOMBRE.clone_into(&mut i.comando);
      i.claves.retain(|(k, _)| k != "operacion" && k != "id");
      i.claves
          .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
      if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
          i = i.con("pendiente", "no");
      }
      i
  }
  ```

  `crates/tc2/src/plan/mod.rs:542-551`
  ```rust
  fn de_la_orden(nombre: &str, mut i: Informe, id: &str) -> Informe {
      nombre.clone_into(&mut i.comando);
      i.claves.retain(|(k, _)| k != "operacion" && k != "id");
      i.claves
          .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
      if i.resultado == Resultado::Sinred && i.valor("pendiente").is_none() {
          i = i.con("pendiente", "no");
      }
      i
  }
  ```

  Diff normalizado: `toma/mod.rs:106-115` (`de_start`) es identico a `de_review` salvo el nombre (pase 0: «fn igual salvo nombres y literales», 111 tokens); `ciclo/mod.rs:423-432` es identico a `plan/mod.rs:542-551` con la firma `(mut i, nombre, id)` en lugar de `(nombre, mut i, id)` (pase 0: `fn de_la_orden`, similitud 0.89). `rechazada(motivo, m)` de 5 lineas es identica en revision y toma (hash 67405710). Otras dos formas del mismo `retain` en `plan/mod.rs:274` (con `hito`) y `estimacion/mod.rs:214`.

  Extraccion propuesta: `Informe::como_orden(self, nombre, id) -> Informe` en `contrato` (o en `estado`, junto a la transaccion), y `Informe::rechazada(nombre, motivo, m)`.
- estado: confirmado
- grupo_seguimiento: envoltorio-de-orden
- absorbe: (ninguno)

<a id="a8-09"></a>

#### A8-09 · D1 · IMPORTANTE · pendiente [XO]

_`con_dueno` y `escribir` idénticos en `ciclo` y `plan`_

- id: A8-09
- regla: D1
- ubicacion: crates/tc2/src/ciclo/mod.rs:376-386 (`con_dueno`) ~ crates/tc2/src/plan/mod.rs:527-537 (`con_dueno`); crates/tc2/src/ciclo/mod.rs:367-372 (`escribir`) ~ crates/tc2/src/plan/mod.rs:487-492 (`escribir`)
- modulo: ciclo (A7), plan (A3)
- severidad: IMPORTANTE
- motivo_severidad: D1 real entre dos duenos: `con_dueno` (11 lineas) es texto identico en `ciclo` y `plan`, y justo encima `escribir` (6 lineas) tambien. Ningun ADR lo impone. No tapa un fallo de datos ni de seguridad. No `divergente`.
- evidencia:

  `crates/tc2/src/ciclo/mod.rs:376-386` (identica a `crates/tc2/src/plan/mod.rs:527-537`, hash 202108bb; solo cambia el comentario de doc)
  ```rust
  fn con_dueno(mut i: Informe, dueno: Option<String>) -> Informe {
      if i.resultado != Resultado::Rechazada {
          return i;
      }
      match (i.valor("motivo"), dueno) {
          (Some("de-otra-persona"), Some(l)) => i = i.con("asignado", l),
          (Some("formato-1"), _) => i = i.con("arreglo", "tc2 actualizar"),
          _ => {}
      }
      i
  }
  ```

  `crates/tc2/src/ciclo/mod.rs:367-372` (identica a `crates/tc2/src/plan/mod.rs:487-492`, hash 50e546a8)
  ```rust
  fn escribir(e: &Entorno<'_>, op: &dyn Operacion) -> Informe {
      let personas = crate::roadmap::personas(e.git, e.dir);
      let generar = crate::roadmap::generador(&personas);
      let ctx = estado::Contexto::nuevo(e.git, e.dir, e.catalogo, Some(&generar));
      estado::transaccion(&ctx, op)
  }
  ```

  Diff normalizado: ninguno. El patron de `escribir` (personas → generador → `Contexto::nuevo` → `transaccion`) esta ademas inline en `importar/mod.rs:346-349`, `revision/mod.rs:298-300`, `toma/mod.rs:211-213` y `lib.rs:738-741` (pista A3.P4; cada copia por debajo del umbral). La toma lo resuelve con `rechazo_de_toma` (`toma/mod.rs:125`) y revision inline (`revision/mod.rs:239-242`): la misma idea, otra forma.

  Extraccion propuesta: `estado::escribir_con_roadmap(git, dir, catalogo, op)` y `Informe::con_dueno(self, dueno)` en un sitio comun (`estado` o `contrato`), usados por las seis ordenes.
- estado: confirmado
- grupo_seguimiento: envoltorio-de-orden
- absorbe: (ninguno; pista A3.P4)

<a id="a8-10"></a>

#### A8-10 · D1 · IMPORTANTE · pendiente [XO]

_Política de lectura de la configuración de la base escrita dos veces (`ciclo`, `plan`)_

- id: A8-10
- regla: D1
- ubicacion: crates/tc2/src/ciclo/mod.rs:166-177 (`umbral_de_inactividad`) ~ crates/tc2/src/plan/mod.rs:437-448 (`config_de_base`)
- modulo: ciclo (A7), plan (A3)
- severidad: IMPORTANTE
- motivo_severidad: D1 real entre dos duenos: la misma politica de lectura de la configuracion de la base («sin base o sin config, el de por defecto; una que no vale falla con su informe a nombre de la orden») escrita dos veces; el doc de `plan` lo reconoce («como `ciclo::umbral_de_inactividad`»). Ningun ADR pide dos copias (F1-10 decision 3 fija la politica). No tapa un fallo de datos ni de seguridad. No `divergente`.
- evidencia:

  `crates/tc2/src/ciclo/mod.rs:166-176`
  ```rust
  pub fn umbral_de_inactividad(g: &Git, dir: &Path, op: &str) -> Result<u64, Informe> {
      match crate::config::de_base(g, dir) {
          Ok(c) => Ok(u64::from(c.dias_sin_actividad)),
          Err(i) if matches!(i.valor("motivo"), Some("sin-base" | "sin-config")) => {
              Ok(DIAS_POR_DEFECTO)
          }
          Err(mut i) => {
              op.clone_into(&mut i.comando);
              Err(i)
          }
  ```

  `crates/tc2/src/plan/mod.rs:437-447`
  ```rust
  pub fn config_de_base(g: &Git, dir: &Path) -> Result<Config, Informe> {
      match crate::config::de_base(g, dir) {
          Ok(c) => Ok(c),
          Err(i) if matches!(i.valor("motivo"), Some("sin-base" | "sin-config")) => {
              Ok(Config::suelo())
          }
          Err(mut i) => {
              NOMBRE.clone_into(&mut i.comando);
              Err(i)
          }
  ```

  Diff normalizado: mismo `match`; cambian el valor por defecto (`DIAS_POR_DEFECTO` frente a `Config::suelo()`, que ya trae `dias_sin_actividad` por defecto) y el nombre de la orden (parametro `op` frente a la constante). Pase 0: «fn casi iguales», Jaccard 0.81. Doc de plan, `crates/tc2/src/plan/mod.rs:434-436`: «una que no vale falla con su informe a nombre de `plan` (como `ciclo::umbral_de_inactividad`)».

  Extraccion propuesta: `config::de_base_o_suelo(g, dir, op) -> Result<Config, Informe>` en `config` (A7); `ciclo` toma `dias_sin_actividad` de ahi.
- estado: confirmado
- grupo_seguimiento: config-de-base
- absorbe: (ninguno)

### Abstracción común — utilidades de E/S y tiempo

<a id="a8-04"></a>

#### A8-04 · D1 · IMPORTANTE divergente · pendiente [XU]

_Escritura atómica tres veces (`huella`, `desinstalar`, `confianza`); solo `huella` reintenta el `rename` en Windows_

- id: A8-04
- regla: D1
- ubicacion: crates/tc2/src/huella.rs:89-116 (`escribir_local` / `escribir_local_con`) ~ crates/tc2/src/confianza/mod.rs:160-178 (`escribir_almacen`) ~ crates/tc2/src/desinstalar/mod.rs:571-588 (`escribir_atomico`)
- modulo: huella (A8), confianza (A7), desinstalar (A7)
- severidad: IMPORTANTE (divergente)
- motivo_severidad: D1 real: tres escrituras «temporal al lado + rename» y ya divergen en comportamiento. Solo `huella` repite el `rename` en Windows cuando un lector tiene el destino abierto sin `FILE_SHARE_DELETE`, y su propio doc da el motivo (Python, Node o un antivirus, L2 de F1-17 punto 7); `desinstalar` escribe el `settings.json` de Claude Code y el perfil del shell sin ese reintento, y `confianza` deja el temporal suelto si falla `write`. No llega a CRITICO: un fallo deja el destino como estaba (el rename es atomico) y lo dice con error; no se pierde ni se corrompe el dato.
- evidencia:

  `crates/tc2/src/huella.rs:101-115`
  ```rust
      let mut nombre = ruta.file_name().unwrap_or_default().to_os_string();
      nombre.push(format!(".tmp-{}", std::process::id()));
      let tmp = ruta.with_file_name(nombre);
      std::fs::write(&tmp, contenido)?;
      let mut r = std::fs::rename(&tmp, ruta);
      for e in esperas {
          if r.is_ok() {
              break;
          }
          std::thread::sleep(*e);
          r = std::fs::rename(&tmp, ruta);
  ```

  `crates/tc2/src/confianza/mod.rs:172-178`
  ```rust
      let tmp = ruta.with_extension(format!("json.{}.tmp", std::process::id()));
      std::fs::write(&tmp, texto).map_err(|e| error(&e))?;
      std::fs::rename(&tmp, ruta).map_err(|e| {
          let _ = std::fs::remove_file(&tmp);
          error(&e)
      })
  ```

  `crates/tc2/src/desinstalar/mod.rs:578-587`
  ```rust
      let mut nombre = destino.file_name().unwrap_or_default().to_os_string();
      nombre.push(format!(".tc2-tmp-{}", std::process::id()));
      let tmp = destino.with_file_name(nombre);
      let r = std::fs::write(&tmp, contenido)
          .and_then(|()| std::fs::set_permissions(&tmp, permisos))
          .and_then(|()| std::fs::rename(&tmp, &destino));
      if r.is_err() {
          let _ = std::fs::remove_file(&tmp);
      }
      r
  ```

  Diff normalizado (lo comun: nombre del temporal con el pid junto al destino, `write`, `rename`, borrar el temporal si falla). Diferencias: sufijo del temporal (`.tmp-<pid>` / `.json.<pid>.tmp` / `.tc2-tmp-<pid>`); reintentos del `rename` con `ESPERAS_RENAME` solo en huella (`crates/tc2/src/huella.rs:75-81`, doc en :72-74); `create_dir_all` en huella y confianza, no en desinstalar; desinstalar sigue enlaces y conserva permisos; confianza no borra el temporal si falla `write` (el `?` de la linea 173 sale antes).

  Extraccion propuesta: `huella::escribir_local` generalizado con opciones (`conservar_permisos`, `seguir_enlace`, `crear_carpeta`) y usado por los tres; el reintento de Windows queda para todos.
- estado: confirmado
- grupo_seguimiento: escritura-atomica
- absorbe: A7-02 (lo amplia con el tercer extremo, `huella`, de A8); pista A7.1

<a id="a8-17"></a>

#### A8-17 · D1 · IMPORTANTE divergente · pendiente [XU]

_«Leer con tope» escrito ocho veces y divergente en qué aceptan_

- id: A8-17
- regla: D1
- ubicacion: crates/tc2/src/identidad.rs:229-240 (`Cache::leer`, A1) ~ crates/tc2/src/estado/frescura.rs:171-181 (`Marca::leer_crudo`, A4) ~ crates/tc2/src/hallazgos/orden.rs:153-160 (`leer_con_tope`, A2) ~ crates/tc2/src/seleccion/orden.rs:280-287 (`leer_con_tope`, A5) ~ crates/tc2/src/progreso/fichero.rs:125-133 (`bytes`) y crates/tc2/src/progreso/mod.rs:51-59 (`texto_corto`, A6) ~ crates/tc2/src/plan/mod.rs:152-158 y crates/tc2/src/importar/mod.rs:257-263 (`leer_fichero`, A3)
- modulo: identidad (A1), hallazgos (A2), plan e importar (A3), estado (A4), seleccion (A5), progreso (A6)
- severidad: IMPORTANTE (divergente)
- motivo_severidad: D1 real entre seis duenos: «leer un fichero con tope (`take(tope + 1)` y rechazar si pasa)» esta escrito ocho veces en lugar de una funcion comun. El par identidad ~ frescura pasa del umbral (10 lineas, 84 tokens normalizados en el pase 0); las otras seis son la misma idea en 7-9 lineas. Las copias ya divergen en algo que se ve desde fuera: `hallazgos` y `seleccion` aceptan bytes que no son UTF-8 (`from_utf8_lossy`, los cambian por U+FFFD) mientras `identidad`, `frescura`, `progreso` y `plan`/`importar` los rechazan (`read_to_string` o `String::from_utf8`); y unas suman `tope + 1` y otras `tope.saturating_add(1)`. Ningun ADR fija una lectura por modulo. No tapa un fallo de datos ni de seguridad (todas acotan la memoria), por eso no CRITICO.
- evidencia:

  `crates/tc2/src/identidad.rs:229-240`
  ```rust
          use std::io::Read;
          // Con tope: un fichero mayor no es una cache de `tc2` (se trata como
          // corrupta, es decir, como si no hubiera).
          let mut texto = String::new();
          std::fs::File::open(&self.ruta)
              .ok()?
              .take(LIMITE_CACHE + 1)
              .read_to_string(&mut texto)
              .ok()?;
          if texto.len() as u64 > LIMITE_CACHE {
  ```

  `crates/tc2/src/estado/frescura.rs:172-181`
  ```rust
          use std::io::Read;
          let mut texto = String::new();
          std::fs::File::open(&self.ruta)
              .ok()?
              .take(LIMITE_MARCA + 1)
              .read_to_string(&mut texto)
              .ok()?;
          if texto.len() as u64 > LIMITE_MARCA {
              return None;
          }
  ```

  `crates/tc2/src/hallazgos/orden.rs:153-160` (identica byte a byte a seleccion/orden.rs:280-287, con la misma doc)
  ```rust
  fn leer_con_tope(p: &Path, tope: u64) -> std::io::Result<Option<String>> {
      let mut b = Vec::new();
      std::fs::File::open(p)?.take(tope + 1).read_to_end(&mut b)?;
      if b.len() as u64 > tope {
          return Ok(None);
      }
      Ok(Some(String::from_utf8_lossy(&b).into_owned()))
  }
  ```

  Diff normalizado: identidad ~ frescura solo cambian la constante (`LIMITE_CACHE` / `LIMITE_MARCA`) y el comentario. Frente a ellas, `leer_con_tope` usa `read_to_end` + `from_utf8_lossy` y devuelve `io::Result<Option<String>>`; `progreso/fichero.rs:125-133` devuelve bytes con `(v.len() as u64 <= LIMITE).then_some(v)`; `progreso/mod.rs:51-59` recorta el salto final; `plan/mod.rs:152-158` e `importar/mod.rs:257-263` (ya D1 entre si, A3-03) usan `saturating_add(1)` y convierten el error en `Informe`. Recuento: `git grep -n -E '\.take\([A-Za-z_]+ ?\+ ?1\)|\.take\(tope|take\(LIMITE'` en d82ea9d da estas ocho (mas dos lecturas de stdin, `progreso/mod.rs:371` y `verificacion/ejecutar.rs:141`, que no son de fichero).

  Extraccion propuesta: una `fn leer_con_tope(ruta, tope) -> io::Result<Option<Vec<u8>>>` en un modulo comun de la raiz (junto a `huella::escribir_local`, la otra mitad de los ficheros locales), y que cada llamador decida UTF-8 estricto o con perdida de forma explicita.
- estado: confirmado
- grupo_seguimiento: lectura-con-tope
- absorbe: relaciona A3-03 (su par interno plan ~ importar entra en la misma funcion); pistas A2.4, A5.1

<a id="a9-01"></a>

#### A9-01 · D1 · IMPORTANTE divergente · pendiente [XU]

_«Esperar a un hijo con plazo y matarlo» dos veces; solo `verificacion` mata el árbol entero_

- id: A9-01
- regla: D1
- ubicacion: crates/tc2/src/actualizar_cli.rs:574-592 (`ejecutar_con_limite`, bucle de espera con plazo) ~ crates/tc2/src/verificacion/ejecutar.rs:380-393 (`lanzar`, bucle de espera con plazo); doc en conflicto: crates/tc2/src/actualizar_cli.rs:516-518 frente a crates/tc2/src/verificacion/ejecutar.rs:394-416 (`matar_arbol`)
- modulo: actualizar_cli (A8); verificacion (A7). Lo usan tambien `desinstalar/maquina.rs:352` (A7) a traves de `ejecutar_con_limite`
- severidad: IMPORTANTE (divergente)
- motivo_severidad: D1 real entre dos duenos: «esperar a un hijo con plazo y matarlo si no acaba» esta escrito dos veces en la misma caja (sondeo con `try_wait`, `Instant::now() >= limite`, `kill` + `wait`, `sleep`). Las copias ya divergen en algo que se ve desde fuera: `verificacion::lanzar` mata el arbol entero (`matar_arbol`: `taskkill /T /F` en Windows, `kill -KILL -- -<pgid>` con `process_group(0)` en Unix) y `ejecutar_con_limite` solo mata al hijo y documenta que matar el arbol «no» se puede hacer de forma portable sin dependencias, cuando la otra copia lo hace sin dependencias. Consecuencia documentada en el propio `ejecutar_con_limite` (actualizar_cli.rs:510-516): con un nieto que hereda las tuberias (un `gh.bat`, un `tc2` que lanza otro proceso) la llamada tarda el plazo entero (60 s por llamada a `gh`, `ESPERA_GH`, actualizar_cli.rs:460) y el nieto queda vivo. Ningun ADR impone dos esperas distintas. No tapa un fallo de datos ni de seguridad (el binario nuevo sigue pasando por sha256 y attestation): no CRITICO, va primera en su grupo por `divergente`.
- evidencia:

  `crates/tc2/src/actualizar_cli.rs:574-583` (sigue en :586-591 con `if Instant::now() >= fin { let _ = hijo.kill(); let _ = hijo.wait(); break None; }` y `std::thread::sleep(Duration::from_millis(20));`)
  ```rust
      let fin = Instant::now() + limite;
      let status = loop {
          match hijo.try_wait() {
              Ok(Some(s)) => break Some(s),
              Ok(None) => {}
              Err(e) => {
                  // Sin poder saber si sigue vivo: se mata antes de salir.
                  let _ = hijo.kill();
                  let _ = hijo.wait();
                  return Err(e);
  ```

  `crates/tc2/src/verificacion/ejecutar.rs:381-390` (sigue en :392 con `std::thread::sleep(SONDEO);`, `SONDEO` = 10 ms en :49)
  ```rust
      let limite = Instant::now() + Duration::from_secs(u64::from(timeout));
      loop {
          if let Some(s) = hijo.try_wait()? {
              return Ok(Fin::Codigo(s.code().unwrap_or(-1)));
          }
          if Instant::now() >= limite {
              matar_arbol(hijo.id());
              let _ = hijo.kill();
              let _ = hijo.wait();
              return Ok(Fin::Timeout);
  ```

  Diff normalizado: mismo bucle (`Instant::now() + plazo`; `try_wait`; al cumplirse el plazo `kill` + `wait` y salida «sin terminar»; `sleep` fijo). Difieren en (1) `matar_arbol(hijo.id())` solo en verificacion, (2) el error de `try_wait`: `?` en verificacion frente a `kill` + `wait` + `return Err` en actualizar_cli, (3) `break Some(s)`/`break None` frente a `return Ok(Fin::...)` y (4) el sondeo (20 ms frente a 10 ms).

  La contradiccion de la documentacion, `crates/tc2/src/actualizar_cli.rs:516-518`:
  ```rust
  /// acaban solos cuando el nieto cierre las tuberias (o con el proceso). Al
  /// nieto no se le mata: no hay una forma portable de matar el arbol sin
  /// dependencias.
  ```
  frente a `crates/tc2/src/verificacion/ejecutar.rs:394-395` y :399-409:
  ```rust
  /// Mata el comando y lo que haya lanzado (la shell sola dejaria vivos a sus
  /// hijos).
  ...
          let mut c = Command::new("taskkill");
          c.args(["/T", "/F", "/PID", &pid]);
  ...
          let mut c = Command::new("kill");
          c.args(["-KILL", "--", &format!("-{pid}")]);
  ```

  Comparado contra todo el repo (`git grep -n "try_wait" d82ea9d -- '*.rs'`): solo estas dos esperas de produccion y un bucle de test en `crates/tc2/tests/progreso.rs:583-590` (espera sin matar, de un test; no cuenta).

  Extraccion propuesta: una espera comun (`esperar_con_plazo(hijo, limite) -> io::Result<Option<ExitStatus>>`) que al cumplirse el plazo llame a `matar_arbol`, con `process_group(0)` en Unix tambien para `ejecutar_con_limite`; `ejecutar_con_limite` conserva sus hilos lectores y `lanzar` su redireccion a fichero.
- estado: confirmado (lectura contra d82ea9d; no se ejecuto)
- grupo_seguimiento: espera-con-plazo (A7, A8)
- absorbe: (ninguno)

<a id="a8-03"></a>

#### A8-03 · D1 · IMPORTANTE · pendiente [XU]

_Formateo `AAAA-MM-DDTHH:MM:SSZ` dos veces en producción (`alta`, `toma`) y el algoritmo civil otra vez en un test y en xtask_

- id: A8-03
- regla: D1
- ubicacion: crates/tc2/src/alta.rs:673-683 (`instante_utc`) ~ crates/tc2/src/toma/operacion.rs:250-261 (`instante`); crates/tc2/src/alta.rs:687-698 (`fecha_de_dias`) ~ crates/tc2/tests/identidades.rs:366-384 (`iso`) ~ xtask/src/main.rs:430-448 (`fecha_utc`)
- modulo: alta (A3), toma (A4), tests de identidades (A1), xtask (A8)
- severidad: IMPORTANTE
- motivo_severidad: D1 real: el formateo `AAAA-MM-DDTHH:MM:SSZ` de unos segundos desde 1970 esta dos veces en produccion (`alta::instante_utc` y `toma::instante`, cuerpo identico de 11 lineas, los dos en uso) y el algoritmo civil_from_days de Hinnant esta pegado otra vez en un test (`iso`, porque `alta::fecha_de_dias` es `pub(crate)`) y en xtask (`fecha_utc`, en u64). Ningun ADR lo impone. No tapa un fallo de datos (las copias dan lo mismo para fechas desde 1970) ni de seguridad. No `divergente` en comportamiento observable.
- evidencia:

  `crates/tc2/src/alta.rs:673-683`
  ```rust
  pub(crate) fn instante_utc(segundos: u64) -> String {
      let dias = i64::try_from(segundos / 86_400).unwrap_or(0);
      let s = segundos % 86_400;
      format!(
          "{}T{:02}:{:02}:{:02}Z",
          fecha_de_dias(dias),
          s / 3600,
          s % 3600 / 60,
          s % 60
      )
  }
  ```

  `crates/tc2/src/toma/operacion.rs:251-261`
  ```rust
  pub fn instante(segundos: u64) -> String {
      let dias = i64::try_from(segundos / 86_400).unwrap_or(0);
      let resto = segundos % 86_400;
      format!(
          "{}T{:02}:{:02}:{:02}Z",
          crate::alta::fecha_de_dias(dias),
          resto / 3600,
          resto % 3600 / 60,
          resto % 60
      )
  }
  ```
  Diff normalizado: identicos salvo el nombre `s`/`resto` y la ruta `crate::alta::`.

  `crates/tc2/tests/identidades.rs:369-378` frente a `crates/tc2/src/alta.rs:688-697`
  ```rust
      let z = dias + 719_468;
      let era = z.div_euclid(146_097);
      let doe = z - era * 146_097;
      let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
      let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
      let mp = (5 * doy + 2) / 153;
      let dia = doy - (153 * mp + 2) / 5 + 1;
      let mes = if mp < 10 { mp + 3 } else { mp - 9 };
      let anio = yoe + era * 400 + i64::from(mes <= 2);
  ```
  Diff normalizado con alta.rs: `doe = z.rem_euclid(146_097)` frente a `z - era * 146_097` (igual para z >= 0) y nombres `d/m/a` frente a `dia/mes/anio`. `xtask/src/main.rs:434-443` es el mismo cuerpo en `u64` (`era = z / 146_097`, `u64::from(mes <= 2)`); pase 0 lo empareja con `iso` (Jaccard 0.84).

  Por que no se reutiliza: `identidades.rs` no ve `alta::fecha_de_dias` (`pub(crate)`) y xtask no depende de `tc2` (`xtask/Cargo.toml:15-16`: solo `distribucion`). La inversa (`dias_desde_epoca`, `crates/tc2/src/inventario.rs:592-600`) vive en el inventario y la usan estimacion, app_github y tarea (pista A5.3): el modulo de fechas esta repartido.

  Extraccion propuesta: un `fecha` en la caja hoja `distribucion` (ya compartida por tc2 y xtask) con `instante_utc(u64)`, `fecha_de_dias(i64)` y `dias_desde_epoca`; `toma::instante` pasa a ser una reexportacion.
- estado: confirmado
- grupo_seguimiento: fechas-utc
- absorbe: (ninguno; pistas A1.2, A3.P1-P2, A4.1, A5.3)

### MENOR sin tarea (solo documento)

<a id="a1-12"></a>

#### A1-12 · D3 · MENOR · — (MENOR: solo documento)

_`estado_puertas` reescribe en línea `solo_nivel` del mismo fichero_

- id: A1-12
- regla: D3
- ambito: codigo
- ubicacion: crates/tc2/src/plataforma/gitlab.rs:704
- modulo: plataforma (adaptador GitLab, `estado-puertas`, G01)
- severidad: MENOR
- motivo_severidad: D3 sin cambio de comportamiento: `estado_puertas` reescribe en linea la comprobacion «la lista de niveles es exactamente `nivel`, sin usuarios ni grupos» que el mismo fichero ya tiene como `solo_nivel` (y que usa `leer_para_aplicar`). Son dos copias de la misma regla de G01 que podrian separarse si una se toca.
- estado: confirmado
- grupo_seguimiento: A1 — Plataforma e identidad (MENOR: solo consta en el documento)
- evidencia: fragmentos literales con fichero:linea en la seccion «Evidencia» de abajo (contra d82ea9d)

##### Evidencia

Antes (7 lineas):
```text
crates/tc2/src/plataforma/gitlab.rs:704:         let g01 = protegida.is_some_and(|p| {
crates/tc2/src/plataforma/gitlab.rs:705:             !p.allow_force_push
crates/tc2/src/plataforma/gitlab.rs:706:                 && p.merge_access_levels.len() == 1
crates/tc2/src/plataforma/gitlab.rs:707:                 && p.merge_access_levels.iter().all(|n| {
crates/tc2/src/plataforma/gitlab.rs:708:                     n.access_level == MAINTAINER && n.user_id.is_none() && n.group_id.is_none()
crates/tc2/src/plataforma/gitlab.rs:709:                 })
crates/tc2/src/plataforma/gitlab.rs:710:         });
```

La funcion que ya existe en el mismo fichero:
```text
crates/tc2/src/plataforma/gitlab.rs:104: fn solo_nivel(v: &[NivelApi], nivel: u64) -> bool {
crates/tc2/src/plataforma/gitlab.rs:105:     v.len() == 1
crates/tc2/src/plataforma/gitlab.rs:106:         && v.iter()
crates/tc2/src/plataforma/gitlab.rs:107:             .all(|n| n.access_level == nivel && n.user_id.is_none() && n.group_id.is_none())
crates/tc2/src/plataforma/gitlab.rs:108: }
```

Despues (version reducida, 2 lineas, mismo comportamiento: `solo_nivel` exige `len() == 1` y que todos sean `nivel` sin `user_id` ni `group_id`):

```rust
let g01 = protegida
    .is_some_and(|p| !p.allow_force_push && solo_nivel(&p.merge_access_levels, MAINTAINER));
```

Los casos de G01 de plataforma_gitlab.rs (598-613, 654-664 y 685-692) cubren las tres condiciones, asi que el cambio se puede comprobar con `cargo test -p tc2 --test plataforma_gitlab protecciones_de_gitlab`.

<a id="a2-07"></a>

#### A2-07 · D2 · MENOR · — (MENOR: solo documento)

_La cabecera del módulo contradice la decisión revisada del ADR y la versión de contrato que el código usa_

- id: A2-07
- regla: D2
- ubicacion: crates/tc2/src/hallazgos/mod.rs:11 y :17-18 (y crates/tc2/src/hallazgos/orden.rs:27)
- modulo: hallazgos
- severidad: MENOR
- motivo_severidad: D2 de documentacion: la cabecera del modulo contradice la decision revisada del ADR y la version del contrato que el propio codigo usa. No cambia el comportamiento (el texto del prompt, `instrucciones()`, ya dice lo correcto: «Nadie ejecuta la evidencia», mod.rs:1091).
- evidencia:

  `crates/tc2/src/hallazgos/mod.rs:11` frente a `:49`
  ```rust
  //! Contrato (`crates/tc2/esquemas/hallazgos.schema.json`, version 2): la
  ...
  pub const VERSION: u32 = 3;
  ```
  (el esquema, `hallazgos.schema.json:3` y `:11`, es la version 3: `"const": 3`).

  `crates/tc2/src/hallazgos/mod.rs:17-18`
  ```rust
  //! declara y se ancla al diff, pero no se ejecuta: eso es de F6-6 (adenda
  //! F6-4 de F1-8). [`instrucciones`] es el texto que va al prompt, hecho con
  ```

  ADR `docs/adr/F1-8-seguridad-modelo-de-amenazas.md:849-851` (adenda F6-6):
  «**Decisión 3 (ejecución de la evidencia), revisada por la persona el 2026-11-22**: la adenda F6-4 decía «F6-6 la ejecuta»; **no entra en F6-6**.»

  `crates/tc2/src/hallazgos/orden.rs:27-28`: `//! veredicto=<rojo|verde|incompleta> conclusion=<success|failure>` — falta `tope`, que `veredicto` escribe desde F6-6 (`Dictamen::Tope`, mod.rs:1244 y :1254).
- estado: confirmado
- grupo_seguimiento: A2 (MENOR: solo consta en el documento)

<a id="a2-08"></a>

#### A2-08 · D2 · MENOR · — (MENOR: solo documento)

_El esquema exige `hallazgos` en la respuesta incremental y `validar_con` acepta solo `cierres`_

- id: A2-08
- regla: D2
- ubicacion: crates/tc2/src/hallazgos/mod.rs:924-933 frente a crates/tc2/esquemas/hallazgos.schema.json:37-48
- modulo: hallazgos
- severidad: MENOR
- motivo_severidad: D2: el validador se aparta del contrato que declara como suyo. El esquema exige `hallazgos` en la respuesta incremental y `validar_con` acepta un objeto solo con `cierres` (cambio deliberado de MEN-4 de TASK-052 que no se llevo al esquema). El esquema solo lo usan los tests (no es una puerta en produccion), asi que no cambia el comportamiento de `tc2`; MENOR.
- evidencia:

  Contrato declarado, `crates/tc2/src/hallazgos/mod.rs:11-13`:
  «Contrato (`crates/tc2/esquemas/hallazgos.schema.json`, ...): la respuesta trae exactamente un bloque `json` ... (`$defs/respuesta` del esquema)»; y `hallazgos.schema.json:4`: «$defs/respuesta es lo que la revisora devuelve en su unico bloque json (lo valida hallazgos::validar)».

  `crates/tc2/esquemas/hallazgos.schema.json:37-41`
  ```json
          {
            "type": "object",
            "additionalProperties": false,
            "required": ["hallazgos"],
            "properties": {
  ```

  `crates/tc2/src/hallazgos/mod.rs:924-933`
  ```rust
              // Una clave que falta es una lista vacia (las dos igual): sin
              // `hallazgos`, los cierres validos no se pierden.
              let lista = |k: &str| match o.get(k) {
                  Some(Value::Array(l)) => Some(l.clone()),
                  None => Some(Vec::new()),
                  _ => None,
              };
              let solo_esas = o.keys().all(|k| k == "hallazgos" || k == "cierres");
              match (lista("hallazgos"), lista("cierres")) {
                  (Some(h), Some(c)) if solo_esas => (h, c),
  ```

  Entrada que lo prueba (ya en el propio test, `crates/tc2/src/hallazgos/mod.rs:2429-2431`): la respuesta ```` ```json {"cierres": [{"id":"I1","fichero":"src/calculo.rs","linea":12,"motivo":"arreglado"}]} ``` ```` sale con `(v.cierres.len(), v.rechazados.len()) == (1, 0)`, y esa misma entrada no cumple `$defs/respuesta` por su `required: ["hallazgos"]` (leido del esquema; no se ha pasado por un validador, que esta auditoria no ejecuta). `el_esquema_y_el_codigo_dicen_lo_mismo` (mod.rs:2104-2162) no compara `$defs/respuesta`.
- estado: confirmado
- grupo_seguimiento: A2 (MENOR: solo consta en el documento)

<a id="a2-09"></a>

#### A2-09 · D3 · MENOR · — (MENOR: solo documento)

_Siete brazos de `match` idénticos salvo la subórden_

- id: A2-09
- regla: D3
- ubicacion: crates/tc2/src/hallazgos/orden.rs:200-220
- modulo: hallazgos
- severidad: MENOR
- motivo_severidad: D3: siete brazos de `match` identicos salvo el nombre de la suborden y la funcion; la version reducida hace lo mismo. No cambia el comportamiento.
- evidencia:

  Antes, `crates/tc2/src/hallazgos/orden.rs:200-209` (siguen igual `ronda`, `abiertos`, `desde` y `prompt` hasta `:220`, 21 lineas en total):
  ```rust
          [s, resto @ ..] if s == "validar" => match validar(resto) {
              Ok(i) | Err(i) => i.salida_en(json, idioma),
          },
          [s, resto @ ..] if s == "veredicto" => match veredicto(resto) {
              Ok(i) | Err(i) => i.salida_en(json, idioma),
          },
          [s, resto @ ..] if s == "anterior" => match anterior(resto) {
              Ok(i) | Err(i) => i.salida_en(json, idioma),
          },
  ```

  Despues (5 lineas, misma salida para cada suborden y `suborden-desconocida` para el resto):
  ```rust
  const SUBORDENES: [(&str, fn(&[String]) -> Result<Informe, Informe>); 7] = [("validar", validar), ("veredicto", veredicto), ("anterior", anterior), ("ronda", ronda), ("abiertos", abiertos), ("desde", desde), ("prompt", prompt)];
  [s, resto @ ..] => match SUBORDENES.iter().find(|(n, _)| n == s) {
      Some((_, f)) => match f(resto) { Ok(i) | Err(i) => i }.salida_en(json, idioma),
      None => uso("suborden-desconocida").salida_en(json, idioma),
  },
  ```
  (119 tokens normalizados repetidos entre `:199-209` y `:211-221`, pase 0.)
- estado: confirmado
- grupo_seguimiento: A2 (MENOR: solo consta en el documento)

<a id="a2-10"></a>

#### A2-10 · D3 · MENOR · — (MENOR: solo documento)

_`paso` pinta dos veces a mano la conversión `Fallo -> Informe`_

- id: A2-10
- regla: D3
- ubicacion: crates/tc2/src/revision/mod.rs:232-254
- modulo: revision
- severidad: MENOR
- normalizacion: consolidador TASK-138, 2026-11-22: se mantiene aparte (MENOR, D3) como pide `absorbe:` de A8-06, que solo lo «relaciona»: la tarea de A8-06 lo resuelve de paso.
- motivo_severidad: D3: `paso` (167 lineas, clippy `too_many_lines`) pinta dos veces a mano la conversion `Fallo -> Informe` con 23 lineas donde bastan 5 en el sitio de la llamada (mas un `de_fallo` de 6 lineas reutilizable); la version reducida da los mismos informes. No cambia el comportamiento. (El aviso de clippy por si solo no es el hallazgo: lo es esta reduccion concreta.)
- evidencia:

  Antes, `crates/tc2/src/revision/mod.rs:232-245` y `:247-254`:
  ```rust
      if let Err(f) = sondeo.aplicar(&vista) {
          return Err(match f {
              Fallo::Rechazada {
                  motivo,
                  diagnostico,
              } => {
                  let i = rechazada(&motivo, diagnostico);
                  match sondeo.dueno() {
                      Some(l) => i.con("asignado", l),
                      None => i,
                  }
              }
              Fallo::Error(d) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
          });
      }
      let (t, cuerpo) = match estado::tarea_de(&vista, id) {
          Ok(tc) => tc,
          Err(Fallo::Rechazada {
              motivo,
              diagnostico,
          }) => return Err(rechazada(&motivo, diagnostico)),
          Err(Fallo::Error(d)) => return Err(Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d)),
      };
  ```
  (literal de `:232-254`, 23 lineas.)

  Despues (con `fn de_fallo(f: Fallo) -> Informe` de 6 lineas junto a `rechazada`, como el `de_fallo` de `crates/tc2/src/roadmap/mod.rs:303-313`):
  ```rust
      sondeo.aplicar(&vista).map_err(|f| {
          let i = de_fallo(f);
          match sondeo.dueno() { Some(l) => i.con("asignado", l), None => i }
      })?;
      let (t, cuerpo) = estado::tarea_de(&vista, id).map_err(de_fallo)?;
  ```
  Mismo resultado: `Rechazada` da `rechazada(&motivo, diagnostico)` (con `asignado` si hay dueno en el sondeo, como hoy) y `Error` da `Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d)`. `dueno()` solo tiene valor tras un `de-otra-persona` (revision/operacion.rs:132 y :141), asi que anadirlo tambien en la rama `Error` no cambia nada.
- estado: confirmado
- grupo_seguimiento: A2 (MENOR; la conversion esta repetida tambien fuera de A2: A6-01 y pistas/A2.txt punto 3)

<a id="a3-07"></a>

#### A3-07 · D2 · MENOR · — (MENOR: solo documento)

_Comentarios que se apartan del código y del ADR que el código ya cumple (F4-6)_

- id: A3-07
- regla: D2
- ubicacion: crates/tc2/src/tarea/modelo.rs:65-67; crates/tc2/src/plan/heuristica.rs:286
- modulo: tarea (modelo), plan (heuristica)
- severidad: MENOR
- motivo_severidad: comentarios que se apartan del codigo que tienen debajo y del ADR que ese codigo ya cumple (F4-6 sobre F1-15). No cambian el comportamiento; engañan a quien lea el modulo.
- estado: confirmado (por lectura contra d82ea9d)
- grupo_seguimiento: A3 (solo documento: MENOR)

##### Evidencia

1. `crates/tc2/src/tarea/modelo.rs:65-67` dice que el modelo no tiene el `roles` ampliado de F1-15:

```rust
// Sin `zonas` (F1-17) ni `hito` (F1-18) ni el `roles` ampliado de F1-15:
// hasta que los recoja la tarea que los usa, `zonas` y `hito` se conservan
// como claves desconocidas.
```

pero el `Rol` del mismo fichero ya lo tiene desde F4-6, `crates/tc2/src/tarea/modelo.rs:283-299`:

```rust
    /// Agentes elegidos (F1-15 decision 10; F4-6).
    #[serde(default)]
    pub agentes: Vec<String>,
    /// Ids de las filas de las tablas de F1-15 que se aplicaron.
    #[serde(default)]
    pub reglas: Vec<String>,
```

(y `senales`, `avisos`, `inventario`, `catalogo`; se escriben en `ampliado`, `:479-499`, y los prueba `tests/tarea.rs:247-289` «el mapa `roles` ampliado (divergencia 1 de F1-15) se lee»).

2. `crates/tc2/src/plan/heuristica.rs:286-293` llama «minimo» a lo que el codigo aplica como maximo:

```rust
/// Roles que pide un nivel para un tipo: el tope de `hotfix` es un minimo.
fn con_tope(nivel: &str, tipo: &str) -> u8 {
    let n = por_nivel(nivel);
    if tipo == "hotfix" {
        n.min(ROLES_HOTFIX)
```

frente a la constante del mismo fichero, `:44-45`: «Roles de un `hotfix` como mucho (tope, no sustitucion)», y al test `resolucion` (`:392-393`: `critica` + `hotfix` da 1 rol).

Version corregida (1 linea cada una): en `modelo.rs:65`, «Sin `zonas` (F1-17) ni `hito` (F1-18): se conservan como claves desconocidas»; en `heuristica.rs:286`, «el tope de `hotfix` es un maximo».

<a id="a3-09"></a>

#### A3-09 · D3 · MENOR · — (MENOR: solo documento)

_Envoltorio `leidas` que solo reenvía a `estado::tareas`_

- id: A3-09
- regla: D3
- ubicacion: crates/tc2/src/alta.rs:503-509 (`leidas`)
- modulo: alta
- severidad: MENOR
- motivo_severidad: envoltorio `pub(crate)` que solo reenvia a `crate::estado::tareas` con los mismos argumentos; no cambia comportamiento.
- estado: confirmado (por lectura contra d82ea9d)
- grupo_seguimiento: A3 (solo documento: MENOR)

##### Evidencia

Antes, `crates/tc2/src/alta.rs:503-509` (7 lineas, 4 de ellas doc):

```rust
/// Todos los `tarea.md` del estado, de los dos formatos, en orden de ruta y
/// leidos en un solo proceso (F2-4b) con [`crate::estado::tareas`]: los
/// mismos sirven para las parecidas ([`de_formato2`]) y para el historico de
/// la estimacion (F3-13).
pub(crate) fn leidas(estado: &dyn Estado, rutas: &[String]) -> Result<Vec<Leida>, Fallo> {
    crate::estado::tareas(estado, rutas)
}
```

Usos (los dos unicos, `grep -rn "alta::leidas\|fn leidas" crates`): `crates/tc2/src/alta.rs:382` (`let leidas = leidas(estado, &rutas)?;`) y `crates/tc2/src/importar/mod.rs:122` (`let leidas = alta::leidas(estado, &rutas)?;`).

Despues (version reducida, 2 lineas, y se borra la funcion):

```rust
let leidas = crate::estado::tareas(estado, &rutas)?;        // alta.rs:382
let leidas = crate::estado::tareas(estado, &rutas)?;        // importar/mod.rs:122
```

<a id="a4-11"></a>

#### A4-11 · D3 · MENOR · — (MENOR: solo documento)

_Dos funciones de 4 líneas que repiten `renombrar(i, OP)`_

- id: A4-11
- regla: D3
- ubicacion: crates/tc2/src/estado/montaje.rs:63-66 y crates/tc2/src/estado/transaccion.rs:180-183 (renombrado) ~ crates/tc2/src/git/rama.rs:467-470 (renombrar)
- modulo: estado (montaje, transaccion)
- severidad: MENOR
- motivo_severidad: D3 que no cambia comportamiento: dos funciones de 4 lineas que repiten `renombrar(i, OP)`. Por debajo del umbral de D1.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A4 (MENOR: solo consta en el documento)

##### Evidencia

Antes — `crates/tc2/src/estado/montaje.rs:63-66` (y transaccion.rs:180-183, identica):

```rust
fn renombrado(mut i: Informe) -> Informe {
    OP.clone_into(&mut i.comando);
    i
}
```

que ya existe con el nombre de la operacion como parametro en `crates/tc2/src/git/rama.rs:467-470`:

```rust
pub(crate) fn renombrar(mut i: Informe, op: &str) -> Informe {
    op.clone_into(&mut i.comando);
    i
}
```

Despues (1 linea por uso, montaje.rs:161 y transaccion.rs:192; montaje.rs:268 igual):

```rust
paso(git, dir, politica, true).unwrap_or_else(|i| renombrar(i, OP))
```

<a id="a4-12"></a>

#### A4-12 · D3 · MENOR · — (MENOR: solo documento)

_`Git::con_entorno` solo reenvía a `con_entorno_extra`_

- id: A4-12
- regla: D3
- ubicacion: crates/tc2/src/git/puerto.rs:262-269 (Git::con_entorno)
- modulo: git (puerto)
- severidad: MENOR
- motivo_severidad: D3 que no cambia comportamiento: un metodo publico que solo reenvia a otro (`con_entorno_extra`), con su doc diciendo que son iguales. Dos nombres para lo mismo en el puerto.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A4 (MENOR: solo consta en el documento)

##### Evidencia

Antes — `crates/tc2/src/git/puerto.rs:262-269`:

```rust
    /// Anade variables de entorno a cada llamada (los tests aislan asi la
    /// configuracion del usuario sin tocar el entorno del proceso). Igual
    /// que [`Git::con_entorno_extra`]: acumula, no sustituye, asi que el
    /// orden de las dos no importa.
    #[must_use]
    pub fn con_entorno(self, vars: Vec<(OsString, OsString)>) -> Self {
        self.con_entorno_extra(vars)
    }
```

Despues (0 lineas): se quita `con_entorno` y quien lo llama usa `con_entorno_extra` (mismo efecto: puerto.rs:279-284). O, al reves, se queda un solo nombre. Las llamadas estan repartidas por `src/` y `tests/` (p. ej. git/estado.rs:676, tests/agente_git/mod.rs:119, tests/estado_casos.rs:529, tests/toma.rs:218, tests/liberar.rs:263).

<a id="a5-04"></a>

#### A5-04 · D2 · MENOR · — (MENOR: solo documento)

_`fuentes` filtra `Origen::Proyecto` y no elige agentes del proyecto; hoy ninguna fila los pide_

- id: A5-04
- regla: D2 (Desviacion)
- ubicacion: crates/tc2/src/seleccion/orden.rs:126 (`fuentes`, filtro de `Origen::Proyecto`)
- modulo: seleccion::orden (inventario y catalogo de `revisoras`, `seleccion` y `plan`)
- severidad: MENOR
- motivo_severidad: D2 sin cambio de comportamiento hoy: ninguna fila de `tabla-seleccion` elige un agente del proyecto (S02-S05 nombran agentes del plugin y S12 `general-purpose`). Solo se notaria si `recursos:` o una tarea nombrara un agente de `.claude/agents` del proyecto: saldria `no-instalado`.
- estado: confirmado (por lectura)
- grupo_seguimiento: A5 — Seleccion, inventario, recursos y doctor

##### ADR

```text
docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:418:2. **Fuentes: solo lo instalado.** El **inventario** (F3-9) es la única
docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:421:   `.claude/skills` y `.claude/agents` del proyecto, y los agentes integrados
docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md:424:   en CI, siempre el de la base del PR (`develop`), leído con `git show` de la
```

La adenda F4-6, divergencia 6 (ADR F1-15, «las skills del proyecto se leen de la base»), solo habla de skills; nada retira los agentes del proyecto del inventario.

##### Codigo que se aparta

```text
crates/tc2/src/seleccion/orden.rs:126:         entradas.extend(
crates/tc2/src/seleccion/orden.rs:127:             l.inventario
crates/tc2/src/seleccion/orden.rs:128:                 .entradas()
crates/tc2/src/seleccion/orden.rs:129:                 .iter()
crates/tc2/src/seleccion/orden.rs:130:                 .filter(|e| e.origen != Origen::Proyecto)
crates/tc2/src/seleccion/orden.rs:131:                 .cloned(),
crates/tc2/src/seleccion/catalogo.rs:317:             ".claude/skills",
```

El filtro quita del inventario de la maquina todo lo de origen proyecto (skills y agentes); lo de la base lo repone `catalogo::de_base`, que solo lista `.claude/skills` (catalogo.rs:310-327). Con `--plugin` (orden.rs:134-151) tampoco entra ningun agente del proyecto.

##### Entrada que lo prueba (lectura, sin ejecutar)

Un repo con `.claude/agents/mi-agente.md` en `develop` y `recursos: [{id: mi-agente, tipo: agente, rol: revisor, dominios: [puertas-seguridad]}]` y una tarea que dice «token»: el inventario de `tc2 revisoras` no contiene `mi-agente` (ni de la maquina, filtrado; ni de la base, no se lee), asi que S10 lo elige y S14 lo quita con `no-instalado:mi-agente` aunque este en la base. `tc2 doctor --recursos` (que usa `inventario::leer` sin filtro) si lo ve: los dos caminos discrepan.

##### Arreglo sugerido

Que `de_base` liste tambien `.claude/agents/*.md` de la base y los anada como `Tipo::Agente` de origen proyecto.

<a id="a5-05"></a>

#### A5-05 · D2 · MENOR · — (MENOR: solo documento)

_La cabecera de `recursos.rs` afirma que `init` y `start` ya paran por recursos_

- id: A5-05
- regla: D2 (Desviacion)
- ubicacion: crates/tc2/src/recursos.rs:13 (cabecera del modulo; tambien recursos.rs:40 y 144-149)
- modulo: recursos (manifiesto de recursos, F1-14 punto 8)
- severidad: MENOR
- motivo_severidad: D2 de documentacion que no cambia el comportamiento: el codigo de `evaluar_recursos` hace lo que dice para los tres momentos; lo falso es la afirmacion de quien lo usa. Induce a creer que `init` y `start` ya paran por recursos.
- estado: confirmado (por lectura y grep)
- grupo_seguimiento: A5 — Seleccion, inventario, recursos y doctor

##### ADR y estado declarado

```text
docs/adr/F1-14-distribucion-e-instalacion.md:574:   evaluar_recursos(inv: Inventario, m: Manifiesto, momento: Init | Doctor | Start(tarea)) -> InformeRecursos
docs/adr/F1-14-distribucion-e-instalacion.md:580:   | **obligatorio** | `git` con versión mínima, `tc2` con protocolo compatible, skills de fase del plugin, versión mínima del plugin | para con el arreglo en `init`, `doctor` y `start` |
```

La adenda F4-6 de F1-15 (divergencia 7) deja fuera "`start` que para por recursos" (F3-9/F1-14 y F4-4): hoy solo `doctor --recursos` lo dice. La firma con `Init | Doctor | Start` la impone F1-14 (no es Diluvio); lo que se aparta es la cabecera.

##### Codigo que se aparta

```text
crates/tc2/src/recursos.rs:13: //! No depende de `doctor`: `init`, `start` y `seleccionar` lo usan con su
crates/tc2/src/recursos.rs:14: //! propio adaptador. Sin E/S: lo que hay en la maquina entra ya leido
crates/tc2/src/recursos.rs:40:     /// Para con el arreglo en `init`, `doctor` y `start`.
```

`grep -rn "evaluar_recursos\|Momento" crates/tc2/src` fuera de recursos.rs da solo:
```text
crates/tc2/src/doctor/recursos.rs:261:    let informe = recursos::evaluar_recursos(&lectura.inventario, &m, &Momento::Doctor);
```
Ni `init`, ni `start`, ni `seleccionar` llaman a `evaluar_recursos`; `Momento::Init` no se construye en ningun sitio y `Momento::Start` solo en el test `lo_recomendado_se_cruza_con_el_inventario` (recursos.rs:488).

##### Arreglo sugerido

Cambiar la cabecera a "Lo usa `doctor --recursos`; `init` y `start` (pendientes, F4-6 divergencia 7) lo usaran con su adaptador".

<a id="a5-10"></a>

#### A5-10 · D3 · MENOR · — (MENOR: solo documento)

_Cierres y bloques repetidos entre `paso` y `paso_seleccion`_

- id: A5-10
- regla: D3 (Diluvio)
- ubicacion: crates/tc2/src/seleccion/orden.rs:491 (y orden.rs:416-422 frente a 573-579, 431-433 frente a 582-584 y 608-610, 441-444 frente a 617-620)
- modulo: seleccion::orden (`tc2 revisoras` y `tc2 seleccion`)
- severidad: MENOR
- motivo_severidad: D3 sin cambio de comportamiento: cierres y bloques repetidos entre `paso` y `paso_seleccion`, y un `tabla_ilegible` reescrito en linea. Contribuye a que `paso_seleccion` pase de 100 lineas (clippy `too_many_lines`, 115/100), pero el aviso solo no es el hallazgo.
- estado: confirmado
- grupo_seguimiento: A5 — Seleccion, inventario, recursos y doctor

##### Evidencia

`tabla_ilegible` existe (orden.rs:233-236) y `paso` lo reescribe en linea:
```text
crates/tc2/src/seleccion/orden.rs:233: fn tabla_ilegible(comando: &str, error: String) -> Informe {
crates/tc2/src/seleccion/orden.rs:234:     Informe::nuevo(comando, Resultado::Error)
crates/tc2/src/seleccion/orden.rs:235:         .diagnostico(Mensaje::nuevo(&txt::TABLA_ILEGIBLE).con("error", error))
crates/tc2/src/seleccion/orden.rs:491:     .map_err(|x| {
crates/tc2/src/seleccion/orden.rs:492:         Informe::nuevo(NOMBRE, Resultado::Error)
crates/tc2/src/seleccion/orden.rs:493:             .diagnostico(Mensaje::nuevo(&txt::TABLA_ILEGIBLE).con("error", x))
crates/tc2/src/seleccion/orden.rs:494:     })?;
```

Mismo cierre de error de lectura en las dos ordenes (solo cambia `NOMBRE`/`SELECCION`):
```text
crates/tc2/src/seleccion/orden.rs:416:     let fallo_lectura = |ruta: &str, error: String| {
crates/tc2/src/seleccion/orden.rs:417:         Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
crates/tc2/src/seleccion/orden.rs:418:             Mensaje::nuevo(&txt::NO_SE_LEE)
crates/tc2/src/seleccion/orden.rs:573:     let error = |ruta: &str, err: String| {
crates/tc2/src/seleccion/orden.rs:574:         Informe::nuevo(SELECCION, Resultado::Error).diagnostico(
crates/tc2/src/seleccion/orden.rs:575:             Mensaje::nuevo(&txt::NO_SE_LEE)
```
Leer un fichero con tope, tres veces (orden.rs:431-433, 582-584, 608-610):
```text
crates/tc2/src/seleccion/orden.rs:582:             let texto = leer_con_tope(Path::new(f), TOPE_RUTAS)
crates/tc2/src/seleccion/orden.rs:583:                 .map_err(|x| error(f, x.to_string()))?
crates/tc2/src/seleccion/orden.rs:584:                 .ok_or_else(|| error(f, format!("mas de {TOPE_RUTAS} bytes")))?;
```
Y la configuracion de la base con su aviso, dos veces (orden.rs:441-444 y 617-620), aunque `fuentes` ya la lee (orden.rs:153).

##### Version reducida (antes 4 + 7 + 7 + 9 lineas; despues 5)

```text
fn no_se_lee(cmd: &str, ruta: &str, e: String) -> Informe { Informe::nuevo(cmd, Resultado::Error).diagnostico(Mensaje::nuevo(&txt::NO_SE_LEE).con("ruta", ruta).con("error", e)) }
fn leer_fichero(cmd: &str, f: &str) -> Result<String, Informe> {
    leer_con_tope(Path::new(f), TOPE_RUTAS).map_err(|x| no_se_lee(cmd, f, x.to_string()))?
        .ok_or_else(|| no_se_lee(cmd, f, format!("mas de {TOPE_RUTAS} bytes"))) }
// orden.rs:491-494 -> .map_err(|x| tabla_ilegible(NOMBRE, x))?;
```

<a id="a5-11"></a>

#### A5-11 · D3 · MENOR · — (MENOR: solo documento)

_Dos enums con las mismas cuatro variantes y un `match` que traduce_

- id: A5-11
- regla: D3 (Diluvio)
- ubicacion: crates/tc2/src/doctor/recursos.rs:41 (y crates/tc2/src/recursos.rs:64, crates/tc2/src/doctor/mod.rs:55)
- modulo: doctor::recursos / recursos (estado de una comprobacion)
- severidad: MENOR
- motivo_severidad: D3 sin cambio de comportamiento: dos enums con las mismas cuatro variantes en el mismo orden y un `match` que traduce una en otra. No llega a D1 (son tipos de datos, sin logica; menos de 10 lineas de traduccion).
- estado: confirmado
- grupo_seguimiento: A5 — Seleccion, inventario, recursos y doctor

##### Evidencia

```text
crates/tc2/src/recursos.rs:64: pub enum Estado {
crates/tc2/src/recursos.rs:66:     Falla,
crates/tc2/src/recursos.rs:68:     Aviso,
crates/tc2/src/recursos.rs:70:     NoComprobado,
crates/tc2/src/recursos.rs:72:     Ok,
crates/tc2/src/doctor/mod.rs:55: pub enum Estado {
crates/tc2/src/doctor/mod.rs:57:     Falla,
crates/tc2/src/doctor/mod.rs:59:     Aviso,
crates/tc2/src/doctor/mod.rs:61:     NoComprobado,
crates/tc2/src/doctor/mod.rs:63:     Ok,
crates/tc2/src/doctor/recursos.rs:41:     let estado = match x.estado {
crates/tc2/src/doctor/recursos.rs:42:         recursos::Estado::Falla => Estado::Falla,
crates/tc2/src/doctor/recursos.rs:43:         recursos::Estado::Aviso => Estado::Aviso,
crates/tc2/src/doctor/recursos.rs:44:         recursos::Estado::NoComprobado => Estado::NoComprobado,
crates/tc2/src/doctor/recursos.rs:45:         recursos::Estado::Ok => Estado::Ok,
crates/tc2/src/doctor/recursos.rs:46:     };
```

La independencia que pide recursos.rs:13 ("No depende de `doctor`") se conserva si el tipo vive en `recursos` y `doctor` lo reutiliza; no hace falta un segundo enum.

##### Version reducida (antes 11 + 6 lineas; despues 2)

```text
// doctor/mod.rs: en lugar de `pub enum Estado {...}` (el `impl Estado { fn texto }` pasa a recursos.rs)
pub use crate::recursos::Estado;
// doctor/recursos.rs:41-46 -> let estado = x.estado;
```

<a id="a5-12"></a>

#### A5-12 · D3 · MENOR · — (MENOR: solo documento)

_Dos bucles de 13-14 líneas que solo cambian la lista y el `TipoRecurso`_

- id: A5-12
- regla: D3 (Diluvio)
- ubicacion: crates/tc2/src/doctor/recursos.rs:162 (y doctor/recursos.rs:177)
- modulo: doctor::recursos (`recomendados`, lo que piden `roles.*` de las tareas abiertas)
- severidad: MENOR
- motivo_severidad: D3 sin cambio de comportamiento: dos bucles de 13-14 lineas dentro de la misma funcion que solo cambian la lista recorrida y el `TipoRecurso`. Se deja en D3 y no en D1 porque no son dos funciones, sino el mismo bloque repetido en linea.
- estado: confirmado
- grupo_seguimiento: A5 — Seleccion, inventario, recursos y doctor

##### Evidencia

```text
crates/tc2/src/doctor/recursos.rs:162:         for s in r
crates/tc2/src/doctor/recursos.rs:163:             .diseno
crates/tc2/src/doctor/recursos.rs:164:             .skills
crates/tc2/src/doctor/recursos.rs:165:             .iter()
crates/tc2/src/doctor/recursos.rs:166:             .chain(&r.implementacion.skills)
crates/tc2/src/doctor/recursos.rs:167:             .chain(&r.revision.skills)
crates/tc2/src/doctor/recursos.rs:168:         {
crates/tc2/src/doctor/recursos.rs:169:             v.push(Recomendado {
crates/tc2/src/doctor/recursos.rs:170:                 id: s.clone(),
crates/tc2/src/doctor/recursos.rs:171:                 tipo: TipoRecurso::Skill,
crates/tc2/src/doctor/recursos.rs:177:         for a in r
crates/tc2/src/doctor/recursos.rs:178:             .diseno
crates/tc2/src/doctor/recursos.rs:179:             .agentes
crates/tc2/src/doctor/recursos.rs:180:             .iter()
crates/tc2/src/doctor/recursos.rs:181:             .chain(&r.implementacion.agentes)
crates/tc2/src/doctor/recursos.rs:182:             .chain(&r.revision.agentes)
crates/tc2/src/doctor/recursos.rs:183:             .chain(&r.revision.agente)
crates/tc2/src/doctor/recursos.rs:185:             v.push(Recomendado {
crates/tc2/src/doctor/recursos.rs:186:                 id: a.clone(),
crates/tc2/src/doctor/recursos.rs:187:                 tipo: TipoRecurso::Agente,
```

##### Version reducida (antes 29 lineas, 162-190; despues 5)

```text
let skills = r.diseno.skills.iter().chain(&r.implementacion.skills).chain(&r.revision.skills).map(|x| (x, TipoRecurso::Skill));
let agentes = r.diseno.agentes.iter().chain(&r.implementacion.agentes).chain(&r.revision.agentes)
    .chain(&r.revision.agente).map(|x| (x, TipoRecurso::Agente));
v.extend(skills.chain(agentes).map(|(id, tipo)| Recomendado { id: id.clone(), tipo, fuente: fuente.clone() }));
```

<a id="a5-13"></a>

#### A5-13 · D3 · MENOR · — (MENOR: solo documento)

_Vector `hash` que se rellena y nunca se lee en un test_

- id: A5-13
- regla: D3 (Diluvio)
- ubicacion: crates/tc2/src/seleccion/mod.rs:1066 (test `la_muestra_del_adr_se_reproduce`; tambien mod.rs:1105 y 1107)
- modulo: seleccion (tests de la muestra del ADR F1-15)
- severidad: MENOR
- motivo_severidad: D3 en test sin efecto en lo que se comprueba: un vector `hash` que se rellena y nunca se lee, y un comentario que promete volver a seleccionar. No es D4: el determinismo frente al orden del catalogo si queda comprobado por `assert_eq!(cat2, cat)` (si `Catalogo::nuevo` dejara de ordenar, el test se pondria rojo, y tambien `el_catalogo_no_depende_del_orden` en catalogo.rs:388).
- estado: confirmado
- grupo_seguimiento: A5 — Seleccion, inventario, recursos y doctor

##### Evidencia

```text
crates/tc2/src/seleccion/mod.rs:1066:         let mut hash = Vec::new();
crates/tc2/src/seleccion/mod.rs:1104:             assert_eq!(celdas(&s), esperado, "{}", t.tarea);
crates/tc2/src/seleccion/mod.rs:1105:             hash.push(celdas(&s));
crates/tc2/src/seleccion/mod.rs:1107:         // Determinista: otra vez, con el catalogo en otro orden, lo mismo.
crates/tc2/src/seleccion/mod.rs:1108:         let mut rs: Vec<Revisora> = cat.revisoras().to_vec();
crates/tc2/src/seleccion/mod.rs:1111:         let cat2 = Catalogo::nuevo(rs);
crates/tc2/src/seleccion/mod.rs:1112:         assert_eq!(cat2, cat);
```

`hash` no aparece en ninguna otra linea del test (mod.rs:1051-1113); el comentario dice «otra vez ... lo mismo», pero no se vuelve a llamar a `tres`/`seleccionar` con `cat2`.

##### Version reducida (antes 3 lineas; despues 1)

```text
// borrar mod.rs:1066 y mod.rs:1105; mod.rs:1107 -> // Determinista: el catalogo en otro orden es el mismo catalogo.
```

<a id="a5-14"></a>

#### A5-14 · D3 · MENOR · — (MENOR: solo documento)

_Tres tests escriben en línea el ayudante `incidencias`_

- id: A5-14
- regla: D3 (Diluvio)
- ubicacion: crates/tc2/src/inventario.rs:1160 (y inventario.rs:1176, 1286; ayudante en inventario.rs:1331)
- modulo: inventario (tests)
- severidad: MENOR
- motivo_severidad: D3 en test sin cambio de comportamiento: tres tests escriben en linea lo que ya hace el ayudante `incidencias` del mismo modulo de tests. Cada copia esta por debajo del umbral de D1 (3-4 lineas).
- estado: confirmado
- grupo_seguimiento: A5 — Seleccion, inventario, recursos y doctor

##### Evidencia

```text
crates/tc2/src/inventario.rs:1331:     fn incidencias(v: &[Incidencia]) -> Vec<String> {
crates/tc2/src/inventario.rs:1332:         v.iter().map(|i| format!("{} {}", i.id, i.motivo)).collect()
crates/tc2/src/inventario.rs:1161:             l.ilegibles
crates/tc2/src/inventario.rs:1162:                 .iter()
crates/tc2/src/inventario.rs:1163:                 .map(|i| format!("{} {}", i.id, i.motivo))
crates/tc2/src/inventario.rs:1164:                 .collect::<Vec<_>>(),
crates/tc2/src/inventario.rs:1177:             l.ilegibles
crates/tc2/src/inventario.rs:1178:                 .iter()
crates/tc2/src/inventario.rs:1179:                 .map(|i| format!("{} {}", i.id, i.motivo))
crates/tc2/src/inventario.rs:1180:                 .collect::<Vec<_>>(),
crates/tc2/src/inventario.rs:1286:             l.ilegibles
crates/tc2/src/inventario.rs:1287:                 .iter()
crates/tc2/src/inventario.rs:1288:                 .map(|i| format!("{} {}", i.id, i.motivo))
crates/tc2/src/inventario.rs:1289:                 .collect::<Vec<_>>(),
```
(La de los avisos, inventario.rs:1126-1129, es la misma forma sobre `l.avisos`.)

##### Version reducida (antes 4 lineas x 4; despues 1 x 4)

```text
incidencias(&l.ilegibles),   // en 1161-1164, 1177-1180 y 1286-1289
incidencias(&l.avisos),      // en 1126-1129
```

<a id="a6-08"></a>

#### A6-08 · D3 · MENOR · — (MENOR: solo documento)

_Función privada que solo reenvía a `tablero::sanear`_

- id: A6-08
- regla: D3
- ubicacion: crates/tc2/src/solape.rs:405-407
- modulo: solape
- severidad: MENOR
- motivo_severidad: D3 sin cambio de comportamiento (regla 4D: MENOR). Una funcion privada que solo reenvia a `crate::tablero::sanear` con los mismos argumentos; `progreso/linea.rs:18` ya importa la misma funcion con un `use`.
- evidencia:

  Antes, `crates/tc2/src/solape.rs:405-407`:
  ```rust
  fn sanear(t: &str, max: usize) -> String {
      crate::tablero::sanear(t, max)
  }
  ```
  Despues (1 linea, como `crates/tc2/src/progreso/linea.rs:18`: `use crate::tablero::sanear;`):
  ```rust
  use crate::tablero::sanear;
  ```
  Las llamadas (`solape.rs:356-357`, `371-372`, `416`) no cambian.
- estado: confirmado
- grupo_seguimiento: A6 (MENOR: solo consta en el documento)

<a id="a7-08"></a>

#### A7-08 · D3 · MENOR · — (MENOR: solo documento)

_`linea()` y `json()` repiten el literal `extra` y `..self.comun()`_

- id: A7-08
- regla: D3
- ubicacion: crates/tc2/src/init/mod.rs:227-265
- modulo: init
- severidad: MENOR
- motivo_severidad: D3 sin cambio de comportamiento: `linea()` y `json()` repiten el mismo literal `extra` y el mismo `..self.comun()`; la otra orden con el mismo tipo (`desinstalar`, mod.rs:253-274) ya lo resuelve con un `comun(f)` que recibe el formateador.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A7

##### Evidencia

Antes, `crates/tc2/src/init/mod.rs:238-258`:
```rust
    pub fn linea(&self) -> String {
        LineaAccion {
            extra: &[
                ("motivo", self.motivo.as_deref()),
                ("orden", self.orden.as_deref()),
            ],
            ..self.comun()
        }
        .texto()
    }
    pub fn json(&self) -> String {
        let mut s = LineaAccion {
            extra: &[
                ("motivo", self.motivo.as_deref()),
                ("orden", self.orden.as_deref()),
            ],
            ..self.comun()
        }
        .json();
```
mas `fn comun` con `extra: &[]` (mod.rs:227-235), que nunca se usa tal cual: las dos llamadas lo pisan.

Despues (5 lineas, como en desinstalar/mod.rs:262-273):
```rust
fn comun(&self, f: impl Fn(&LineaAccion<'_>) -> String) -> String {
    f(&LineaAccion { ruta: &self.ruta, clase: self.clase, dueno: self.dueno, decision: self.decision.texto(),
        extra: &[("motivo", self.motivo.as_deref()), ("orden", self.orden.as_deref())] })
}
pub fn linea(&self) -> String { self.comun(|l| l.texto()) }
```
y `json()` queda en `let mut s = self.comun(|l| l.json());` seguido del anadido de `propuesta`.

<a id="a8-14"></a>

#### A8-14 · D3 · MENOR · — (MENOR: solo documento)

_Siete bloques «carpeta actual o informe» en `lib.rs` donde basta `tablero::carpeta_actual`_

- id: A8-14
- regla: D3
- ubicacion: crates/tc2/src/lib.rs:355-361, :384-390, :420-427, :575-581, :606-612, :631-638, :714-720 (frente a `crates/tc2/src/tablero.rs:819-824`, `tablero::carpeta_actual`); fuera de A8, el mismo bloque en crates/tc2/src/importar/mod.rs:280-286, crates/tc2/src/roadmap/mod.rs:245-252 y crates/tc2/src/siguiente.rs:53-55
- modulo: lib (A8); importar (A3), roadmap y siguiente (A6)
- severidad: MENOR
- motivo_severidad: Diluvio: siete bloques de 6-7 lineas en `lib.rs` para «carpeta actual o informe de error» cuando ya existe la funcion publica de una linea que lo hace (`tablero::carpeta_actual`, que el mismo `lib.rs` usa en `orden_tablero` y `orden_doctor`). Cada bloque esta por debajo del umbral de D1 (10 lineas), por eso no cuenta como Duplicacion. No cambia comportamiento: MENOR, solo consta en el documento.
- evidencia:

  Antes, `crates/tc2/src/lib.rs:575-581` (igual, con otro nombre de orden, en los otros seis)
  ```rust
      let dir = match std::env::current_dir() {
          Ok(d) => d,
          Err(e) => {
              return Informe::nuevo(seleccion::orden::NOMBRE, Resultado::Error)
                  .diagnostico(Mensaje::nuevo(&txt::CARPETA_ACTUAL).con("error", e.to_string()));
          }
      };
  ```

  Despues (version reducida), con `crates/tc2/src/tablero.rs:819-824` ya existente:
  ```rust
      let dir = match tablero::carpeta_actual(seleccion::orden::NOMBRE) {
          Ok(d) => d,
          Err(i) => return i,
      };
  ```
  (o `carpeta_actual(op)?` donde la funcion devuelve `Result<_, Informe>`). Recuento: `rtk proxy grep -n CARPETA_ACTUAL` en d82ea9d da 7 usos en `lib.rs`, 1 en `importar/mod.rs`, 1 en `roadmap/mod.rs`, 1 en `siguiente.rs` y la propia `tablero.rs:822`. La funcion comun encajaria mejor fuera de `tablero` (p. ej. en `contrato` o en la raiz), pero eso es de la extraccion.
- estado: confirmado
- grupo_seguimiento: A8 — Transversal (MENOR: solo consta en el documento)
- absorbe: (ninguno; pista A3.P5)

<a id="a8-21"></a>

#### A8-21 · D3 · MENOR · — (MENOR: solo documento)

_`ejecutar_en` repite `con_json(resto)` 25 veces_

- id: A8-21
- regla: D3
- ubicacion: crates/tc2/src/lib.rs:134-262 (`ejecutar_en`: `let (json, resto) = con_json(resto);` repetido en 25 brazos, lineas 135, 141, 145, 149, 153, 157, 161, 165, 169, 173, 177, 187, 193, 200, 206, 212, 216, 222, 228, 232, 236, 240, 244, 248 y 260); `con_json` en crates/tc2/src/lib.rs:316-322
- modulo: lib (A8)
- severidad: MENOR
- motivo_severidad: Diluvio: `ejecutar_en` tiene 180 lineas (clippy `too_many_lines` 180/100 y `cognitive_complexity` 36/25 en el pase 0) y buena parte es el mismo `[orden, resto @ ..] if orden == X => { let (json, resto) = con_json(resto); ... }` 25 veces. Sacar `con_json` una vez antes del `match` quita 25 lineas y deja cada brazo en una. No cambia comportamiento si `guardar`/`actualizar-rama` y `new` (lineas 118-133), que leen `--json` solo en primera posicion con su propio `match`, siguen aparte: MENOR, solo consta en el documento. Un aviso de clippy por si solo no seria hallazgo; aqui la reduccion es concreta.
- evidencia:

  Antes, `crates/tc2/src/lib.rs:140-147`
  ```rust
          [orden, resto @ ..] if orden == siguiente::NOMBRE => {
              let (json, resto) = con_json(resto);
              siguiente::orden(&resto, json, idioma)
          }
          [orden, resto @ ..] if orden == roadmap::NOMBRE => {
              let (json, resto) = con_json(resto);
              roadmap::orden(&resto, json, idioma, &catalogo)
          }
  ```

  Despues (version reducida), tras los brazos de `progreso`, `--version`, `guardar` y `new`:
  ```rust
      let Some((orden, resto)) = args.split_first() else { return falta_la_orden(idioma) };
      let (json, resto) = con_json(resto);
      match orden.as_str() {
          o if o == siguiente::NOMBRE => siguiente::orden(&resto, json, idioma),
          o if o == roadmap::NOMBRE => roadmap::orden(&resto, json, idioma, &catalogo),
  ```
  Recuento: `git show d82ea9d:crates/tc2/src/lib.rs | grep -n 'con_json(resto)'` da 25 lineas, las listadas en la ubicacion.
- estado: confirmado
- grupo_seguimiento: A8 — Transversal (MENOR: solo consta en el documento)
- absorbe: (ninguno)

<a id="a9-04"></a>

#### A9-04 · D3 · MENOR · — (MENOR: solo documento)

_`clase_de_tag` repite el predicado de `version_valida` y un mensaje_

- id: A9-04
- regla: D3
- ubicacion: crates/distribucion/src/lib.rs:258-279 (`clase_de_tag`), frente a crates/distribucion/src/lib.rs:160-175 (`version_valida`)
- modulo: distribucion
- severidad: MENOR
- motivo_severidad: Diluvio: `clase_de_tag` repite el predicado del sufijo de prerelease que ya tiene `version_valida` en el mismo fichero (`!s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')`) y escribe dos veces el mismo mensaje de error (:262 y :275-277). Con `version_valida` la funcion baja de 22 a unas 10 lineas. Por debajo del umbral de D1 (5 lineas repetidas). No cambia comportamiento: MENOR, solo consta en el documento.
- evidencia:

  Antes, `crates/distribucion/src/lib.rs:263-278`
  ```rust
      if resto.is_empty() {
          return Ok(Clase::Estable);
      }
      match resto.strip_prefix('-') {
          Some(sufijo)
              if !sufijo.is_empty()
                  && sufijo
                      .bytes()
                      .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-') =>
          {
  ```
  (el mismo predicado esta en `version_valida`, lib.rs:170-174; el mensaje `format!("el tag «{tag}» no es v{version} ni v{version}-<sufijo>")` en :262 y :275-277).

  Despues (version reducida):
  ```rust
      let error = || format!("el tag «{tag}» no es v{version} ni v{version}-<sufijo>");
      match tag.strip_prefix('v').and_then(|r| r.strip_prefix(version)).ok_or_else(error)? {
          "" => Ok(Clase::Estable),
          r if r.starts_with('-') && version_valida(&format!("{version}{r}")) => Ok(Clase::Prerelease),
          _ => Err(error()),
      }
  ```
  Equivalente para la `version` del workspace (`X.Y.Z` sin guion): `version_valida` parte por el primer `-`, asi que con `r = "-s"` exige exactamente `s` no vacio de alfanumericos, `.` y `-`. Lo comprueba ya `tag_y_version_tienen_que_casar` (lib.rs:473-490: `v0.0.0-`, `v0.0.0-a b`, `v0.0.0-a/b` rechazados).
  Recuento: `git grep -n "is_ascii_alphanumeric() || b == b'.' || b == b'-'" d82ea9d -- '*.rs'` da lib.rs:173 y :271 en distribucion, y `plataforma/deteccion.rs:142` y `plataforma/http.rs:99` en tc2, que validan nombres de host (otra regla, no se cuentan).
- estado: confirmado
- grupo_seguimiento: A8 — Transversal (MENOR: solo consta en el documento)
- absorbe: (ninguno)

<a id="a10-03"></a>

#### A10-03 · D2 · MENOR · — (MENOR: solo documento)

_La plantilla de GitLab calcula la clave de reutilización sin las órdenes (contrato de TASK-042)_

- id: A10-03
- regla: D2
- ubicacion: plantillas/gitlab/verificacion.gitlab-ci.yml:103 (frente a .github/workflows/verificacion.yml:240-243)
- modulo: plantilla de verificacion de GitLab (TASK-041 F5-1, TASK-042 F5-2)
- severidad: MENOR
- motivo_severidad: Desviacion del contrato de la linea `arbol=` de F5-2: TASK-042 y la guia fijan `clave = sha256(perfil|runner|huella-arbol|version|ordenes)` y declaran para GitLab una sola divergencia (la version vacia, divergencia 17). La plantilla calcula la clave con 4 campos y sin las ordenes, asi que un verde de otro `comando`/argumentos/`huella-local` da la misma clave en GitLab. Hoy nadie lee la linea de GitLab (sin busqueda, degradacion declarada); es un contrato para F8-1. Por eso MENOR.
- evidencia:

  Criterio, `tareas/04-terminadas/TASK-042/tarea.md:63`: «`clave = sha256(perfil|runner|huella-arbol|version|ordenes)`. [...] escribe una linea por perfil `arbol= perfil= runner= clave= local= origen= ejecutada|reutilizada de <sha>` (contrato para F8-1). [...] GitLab: la linea va en un fichero del artefacto del informe, sin busqueda (D2).»
  Guia, `docs/ci/verificacion-reutilizable.md:254-258`: «`clave = sha256(perfil|runner|huella-arbol|versión del workflow|órdenes)` [...] y `órdenes = sha256` del JSON `[comando, argumentos-<perfil>, shell, huella-local]`». Divergencia declarada, `:688-691`: «la línea va en `.tc2/cache-verificacion-<perfil>.txt` [...] sin la versión del workflow en la clave» (solo la version).

  GitHub, `.github/workflows/verificacion.yml:240-243`
  ```sh
            # Las ordenes entran en la clave: un verde de otro comando,
            # argumentos, shell o huella local no se reutiliza.
            ordenes=$(printf '%s' "$ENTRADAS" | jq -c --arg p "$p" '[.comando, .["argumentos-" + $p], .shell, .["huella-local"]]' | sha256sum | cut -d' ' -f1)
            k=$(printf '%s|%s|%s|%s|%s' "$p" "$r" "$huella" "$VERSION" "$ordenes" | sha256sum | cut -d' ' -f1)
  ```
  GitLab, `plantillas/gitlab/verificacion.gitlab-ci.yml:103`
  ```sh
          clave=$(printf '%s|%s|%s|%s' "$TC2_PERFIL" "$runner" "$huella" "" | sha256sum | cut -d' ' -f1)
  ```
  Diff normalizado: GitLab tiene 4 campos (el cuarto, la version, vacio, como declara la divergencia 17) y le falta el quinto, `ordenes`. Origen: las ordenes entraron en la clave en la correccion de la ronda 1 de TASK-042 (`tarea.md:70`: «MEN-1 ordenes fuera de la clave [...] corregidos en 4f4a62d») y solo se aplicaron al workflow de GitHub. El test `workflow_y_plantilla_tienen_las_mismas_entradas` (xtask/src/workflows.rs) compara entradas, no la formula de la clave.
- estado: confirmado (por lectura)
- grupo_seguimiento: verificacion-reutilizable (A10)
- absorbe: -

<a id="a10-04"></a>

#### A10-04 · D3 · MENOR · — (MENOR: solo documento)

_`observado.json` y `observado-e2e.json` (518 líneas) no los lee nadie; los fixtures de GitLab citan un fichero que no existe_

- id: A10-04
- regla: D3
- ubicacion: crates/tc2/tests/fixtures/plataforma/github/observado.json:1-504 y crates/tc2/tests/fixtures/plataforma/gitlab/observado-e2e.json:1-14; referencias rotas en crates/tc2/tests/fixtures/plataforma/gitlab/{aprobacion-bot,esperando-checks,plan,sin-aprobacion,sin-cambios,verde}.json:4
- modulo: fixtures de plataforma (A10); sus tests, plataforma.rs y plataforma_gitlab.rs, son de A1
- severidad: MENOR
- motivo_severidad: Diluvio: 518 lineas en `tests/fixtures/` que ningun test ni fuente lee. En la PoC de F1-13 (Python) `observado.json` era la referencia con la que se comparaba cada GET real; al portar a Rust se copio el fichero pero no la comparacion, asi que hoy es documentacion con forma de fixture. No cambia comportamiento: MENOR.
- evidencia:

  Uso (contra d82ea9d): `git grep -n "observado\.json\|observado-e2e" d82ea9d -- . ':!tareas'` solo da el texto `"origen"` de los 12 fixtures de casos; ningun `.rs` (los lectores de fixtures, `tests/plataforma.rs:68-71` y `tests/plataforma_gitlab.rs:74-80`, abren `{caso}.json` con los 6 casos de la PoC M). Ningun test nombra los hechos (`git grep "E2E-0[1-5]\|E2E-GL-0" -- crates`: sin resultados en tests ni src).

  Papel que tenia, `docs/adr/F1-13-adaptador-de-plataforma.md:70-71`: «Lo estable de cada GET [...] se compara con `fixtures/observado.json`: si la plataforma cambia, la PoC falla.» Y TASK-021 (`tarea.md:68`): «Las formas reales observadas en el E2E se añadieron a `observado.json` (E2E-01 a 05)» — se anadieron a un fichero que nada lee.

  Antes: `github/observado.json` (504 lineas) y `gitlab/observado-e2e.json` (14) en `tests/fixtures/plataforma/`.
  Despues (reducida): moverlos a la evidencia de sus tareas (`tareas/04-terminadas/TASK-021/evidencia/`, `TASK-022/evidencia/`, donde ya estan `e2e.log` y `e2e-gitlab.log`) o, si deben seguir siendo referencia, un test que los lea; 0 lineas en `tests/fixtures`.

  Ademas, los 6 fixtures de GitLab copian el `origen` de GitHub y citan un fichero que en su carpeta no existe, `crates/tc2/tests/fixtures/plataforma/gitlab/verde.json:4`:
  ```json
   "origen": "fixture inventado con los nombres de campo observados (ver observado.json); lo no observado se marca en la PoC",
  ```
  (`ls crates/tc2/tests/fixtures/plataforma/gitlab/`: solo `observado-e2e.json`.)
- estado: confirmado (por grep)
- grupo_seguimiento: fixtures (A10)
- absorbe: -

<a id="a11-05"></a>

#### A11-05 · D2 · MENOR · — (MENOR: solo documento)

_`main.rs` escribe a mano los códigos de salida 2 y 1, contra «la tabla vive aquí y solo aquí» de `contrato.rs` (F1-4, F2-3)_

- id: A11-05
- regla: D2
- ubicacion: crates/tc2/src/main.rs:34 (`ExitCode::from(2)`) y :75 (`ExitCode::from(1)`) frente a crates/tc2/src/contrato.rs:8-10 (la tabla de codigos «vive aqui y solo aqui») y :59-71 (`Resultado::codigo`)
- modulo: contrato (promesa) / main (A8)
- severidad: MENOR
- motivo_severidad: Desviacion de la regla que el propio modulo declara (F1-4 decision 5, fijada en F2-3): la tabla de codigos de salida tiene que vivir solo en `contrato.rs`, y `main.rs` escribe dos de sus valores a mano. Hoy coinciden (uso = 2, error = 1), asi que no cambia el comportamiento: MENOR. Si la tabla cambiase, estas dos salidas (argumento que no es UTF-8 y stdout que no se puede escribir) se quedarian con el codigo viejo sin que nada lo avise.
- evidencia:

  La regla, `crates/tc2/src/contrato.rs:8-10`
  ```rust
  //! La tabla de codigos de salida vive aqui y solo aqui: la propuso F1-4 y la
  //! fija F2-3 (0 ok y sin-cambios, 1 error, 2 uso, 3 rechazada, 4 sinred,
  //! 5 agotada, 6 conflicto, 7 esperando, 8 denegada).
  ```
  y su unica fuente, `:61-63`: `Self::Ok | Self::SinCambios => 0,` / `Self::Error => 1,` / `Self::Uso => 2,`.

  La linea que se aparta, `crates/tc2/src/main.rs:31-34`
  ```rust
              Err(_) => {
                  let m = Mensaje::nuevo(&txt::ARGUMENTO_NO_UTF8);
                  let _ = writeln!(std::io::stderr(), "{}", m.texto(idioma));
                  return ExitCode::from(2);
  ```
  y `:72-75`
  ```rust
      if let Err(e) = escrito {
          let m = Mensaje::nuevo(&txt::NO_SE_PUDO_ESCRIBIR).con("error", e.to_string());
          let _ = writeln!(std::io::stderr(), "{}", m.texto(idioma));
          return ExitCode::from(1);
  ```
  `git grep -n "ExitCode::from([0-9]" d82ea9d -- crates/tc2/src` solo da estas dos (el resto del crate sale por `ExitCode::from(salida.codigo)`, main.rs:70 y :80). Arreglo: `ExitCode::from(Resultado::Uso.codigo())` y `ExitCode::from(Resultado::Error.codigo())`.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A8 — Transversal (MENOR: solo consta en el documento)
- absorbe: (ninguno)

<a id="a11-06"></a>

#### A11-06 · D2 · MENOR · — (MENOR: solo documento)

_La cabecera de `ci.yml` anuncia como futuro el reutilizable `verificacion` de F5-1, ya entregado_

- id: A11-06
- regla: D2
- ubicacion: .github/workflows/ci.yml:3-4 (cabecera: «La CI de verdad (check `verificacion`, cache por arbol) llega con F5-1») frente a .github/workflows/verificacion.yml:1-2 y CHANGELOG.md:18 y :20
- modulo: workflow ci (A8)
- severidad: MENOR
- motivo_severidad: Desviacion de documentacion: la cabecera del workflow sigue anunciando como futuro algo que ya esta entregado (F5-1, TASK-041, y la cache por arbol, F5-2, TASK-042, terminadas el 2026-11-19), y no dice que este repo no usa ese reutilizable para si mismo (sigue con `cargo xtask verificar` en la matriz de :58-59). Solo es texto; no cambia el comportamiento: MENOR.
- evidencia:

  `.github/workflows/ci.yml:1-4`
  ```yaml
  # CI de TaskCodeTwo. comprobar-repo (TASK-001, F1-0) valida tareas/ y los
  # enlaces de la documentacion; codigo (F2-1) verifica el workspace de Rust en
  # los tres sistemas con el mismo comando que en local. La CI de verdad (check
  # `verificacion`, cache por arbol) llega con F5-1.
  ```
  frente a `.github/workflows/verificacion.yml:1-2`
  ```yaml
  # Workflow reutilizable de verificacion (TASK-041, F5-1; TASK-042, F5-2;
  # ADR F1-7 y F1-8).
  ```
  y `CHANGELOG.md:18` («TASK-042 (feature) — F5-2 Cache por SHA del arbol compartida (2026-11-19)») y `:20` («TASK-041 (feature) — F5-1 Workflow reutilizable de GitHub Actions y plantilla de GitLab (2026-11-19)»). Ningun workflow del repo llama a `verificacion.yml` (`git grep -n "uses: .*verificacion.yml" d82ea9d -- .github/workflows` solo da el ejemplo comentado de verificacion.yml:29). Arreglo: decir en la cabecera que F5-1 entrego el reutilizable para los proyectos y que este repo lo adopta con el relevo (F4-5 del plan), o adoptarlo.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A8 — Transversal (MENOR: solo consta en el documento)
- absorbe: (ninguno)

<a id="a11-07"></a>

#### A11-07 · D2 · MENOR · — (MENOR: solo documento)

_El CHANGELOG nombra `taskctl actualizar --cli`, orden que no existe; la entregada es `tc2 actualizar --cli`_

- id: A11-07
- regla: D2
- ubicacion: CHANGELOG.md:49 («TASK-027 (feature) — F2-12 taskctl actualizar --cli y aviso de version nueva»), renderizado desde tareas/04-terminadas/TASK-027/tarea.md:3; orden real: crates/tc2/src/actualizar_cli.rs:37 (`tc2 actualizar --cli`)
- modulo: documentacion (CHANGELOG)
- severidad: MENOR
- motivo_severidad: El changelog nombra una orden que no existe: `taskctl actualizar --cli`. El `taskctl` que hay (0.16.1, el de la 0.16 con que se trabaja este repo) no tiene `actualizar`, y la orden que entrego TASK-027 es `tc2 actualizar --cli`. Las demas ordenes que nombra el changelog (`tc2 init`, `new`, `import`, `next`, `roadmap`, `board`, `status`, `desinstalar`, `doctor`, `descartar`, `pausa`, `start`, `review`, `approve`, `plan`) tienen su brazo en `crates/tc2/src/lib.rs:118-259`. Solo texto (y el titulo historico de la tarea del que sale): MENOR; se arregla en el titulo de TASK-027 y se regenera la linea.
- evidencia:

  `CHANGELOG.md:49`
  ```
  - TASK-027 (feature) — F2-12 taskctl actualizar --cli y aviso de version nueva (2026-11-16)
  ```
  `tareas/04-terminadas/TASK-027/tarea.md:3` (`git grep -n "^titulo:" d82ea9d -- tareas/04-terminadas/TASK-027*/tarea.md`): `titulo: "F2-12 taskctl actualizar --cli y aviso de version nueva"`. La orden real: `crates/tc2/src/actualizar_cli.rs:37` `pub const NOMBRE: &str = "actualizar";` dentro de `tc2` (y su arreglo en el catalogo, `i18n/catalogo/actualizar.rs:22`: `arreglo=\"tc2 actualizar --cli\"`).
  `taskctl --version` da `0.16.1` y `taskctl --help` lista «Comandos: new, import, board, roadmap, metricas, start, plan, approve, trocear, review, codex-review, veredicto, verificar-menores, verificar-lote, finish, siguiente, estado, doctor, doctor-deploy, pausa, registrar-coste, cadena» (ejecutado en esta maquina, solo lectura): sin `actualizar`.
  Cruce del changelog con las tareas: las 74 lineas `- TASK-NNN` de CHANGELOG.md son exactamente las 74 carpetas de `tareas/04-terminadas` en d82ea9d (`comm` de las dos listas, `task-138/a11/changelog.txt` y `terminadas.txt`): ninguna sobra ni falta.
- estado: confirmado (lectura y `taskctl --help`)
- grupo_seguimiento: A8 — Transversal (MENOR: solo consta en el documento)
- absorbe: (ninguno)

### Fichas fusionadas en otra

<a id="a2-01"></a>

#### A2-01 · D1 · IMPORTANTE · fusionado en A8-07

_Preámbulo de `aplicar` copiado en las dos operaciones de `revision`_

- id: A2-01
- regla: D1
- ubicacion: crates/tc2/src/revision/guardar.rs:96-108 ~ crates/tc2/src/revision/operacion.rs:131-143
- modulo: revision
- severidad: IMPORTANTE
- absorbido_por: A8-07
- normalizacion: consolidador TASK-138, 2026-11-22: fusionado en A8-07 (su campo `absorbe:`). No cuenta como caso aparte; su evidencia sigue valiendo y la tarea de A8-07 lo nombra.
- motivo_severidad: D1 real (regla 4D: minimo IMPORTANTE). El mismo preambulo de `Operacion::aplicar` (limpiar `dueno`, leer la tarea, rechazar si es de otra persona y guardar su login) esta copiado en las dos operaciones de `revision`; F1-2 fija el punto unico de escritura, no este preambulo. No tapa fallo de datos ni de seguridad. No `divergente`: hoy dicen lo mismo.
- evidencia:

  `crates/tc2/src/revision/guardar.rs:96-108`
  ```rust
      fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
          *self.dueno.borrow_mut() = None;
          let (t, _) = tarea_de(estado, &self.id)?;
          if let Some((fallo, dueno)) = de_otra_persona(
              &self.id,
              t.asignado_a.as_ref(),
              &self.asignado,
              NOMBRE,
              ("de-otra-persona", &txt::DE_OTRA_PERSONA),
          ) {
              *self.dueno.borrow_mut() = dueno;
              return Err(fallo);
          }
  ```

  `crates/tc2/src/revision/operacion.rs:131-143`
  ```rust
      fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
          *self.dueno.borrow_mut() = None;
          let (mut t, mut cuerpo) = tarea_de(estado, &self.id)?;
          if let Some((fallo, dueno)) = de_otra_persona(
              &self.id,
              t.asignado_a.as_ref(),
              &self.asignado,
              NOMBRE,
              ("de-otra-persona", &txt::DE_OTRA_PERSONA),
          ) {
              *self.dueno.borrow_mut() = dueno;
              return Err(fallo);
          }
  ```

  Diff normalizado: identicos salvo el patron de la tupla de `tarea_de` (`(t, _)` frente a `(mut t, mut cuerpo)`). 13 lineas, 76 tokens normalizados (pase 0). `crates/tc2/src/ciclo/operacion.rs:77-89` ya lo extrajo como `fn tarea(&self, estado, orden) -> Result<(Tarea, Cuerpo), Fallo>`: la abstraccion existe y `revision` no la usa.
- estado: confirmado
- grupo_seguimiento: A2 (el mismo bloque esta en plan/guardar.rs, plan/operacion.rs, plan/aprobar.rs, toma/operacion.rs y ciclo/operacion.rs: ver pistas/A2.txt punto 2; si A8 abre la abstraccion comun «preambulo de aplicar», este caso se absorbe alli)

<a id="a3-04"></a>

#### A3-04 · D1 · IMPORTANTE · fusionado en A8-07

_Preámbulo de las operaciones de diseño copiado entre `guardar` y `aprobar`_

- id: A3-04
- regla: D1
- ubicacion: crates/tc2/src/plan/guardar.rs:113-131 (`Guardar::aplicar`) ~ crates/tc2/src/plan/aprobar.rs:520-539 (`PedirCambios::aplicar`); la misma forma en crates/tc2/src/plan/operacion.rs:366-386 (`Planificar::aplicar`) y el envoltorio `dueno()`/`rechazo()` en guardar.rs:81-89 ~ aprobar.rs:490-498 ~ operacion.rs:208-216
- modulo: plan (guardar, aprobar, operacion)
- severidad: IMPORTANTE
- absorbido_por: A8-07
- normalizacion: consolidador TASK-138, 2026-11-22: fusionado en A8-07 (su campo `absorbe:`). No cuenta como caso aparte; su evidencia sigue valiendo y la tarea de A8-07 lo nombra.
- motivo_severidad: D1 real: el preambulo de las operaciones de diseno sobre una tarea (limpiar `dueno`, leer la tarea, rechazar la de otra persona guardando su login, exigir `en-diseno`, rechazar con el plan aprobado) esta copiado casi literal; solo cambian la constante de la orden y el texto del rechazo. Ningun ADR lo impone (F4-1 y F4-2 piden el mismo orden de comprobaciones, no tres copias). Hoy no divergen en comportamiento.
- estado: confirmado (por lectura contra d82ea9d)
- grupo_seguimiento: A3

##### Evidencia

Copia 1, `crates/tc2/src/plan/guardar.rs:114-131` (10 lineas utiles):

```rust
        *self.dueno.borrow_mut() = None;
        let (t, _) = tarea_de(estado, &self.id)?;
        if let Some((fallo, dueno)) = de_otra_persona(
            &self.id, t.asignado_a.as_ref(), &self.asignado, super::NOMBRE,
            ("de-otra-persona", &txt::DE_OTRA_PERSONA),
        ) {
            *self.dueno.borrow_mut() = dueno;
            return Err(fallo);
        }
        if t.estado != EN_DISENO {
            return Err(por_estado(&self.id, &t.estado, super::NOMBRE, &[EN_DISENO]));
        }
        if t.plan_aprobado {
            return Err(self.rechazo("plan-aprobado", Mensaje::nuevo(&txt::PLAN_APROBADO)));
```

Copia 2, `crates/tc2/src/plan/aprobar.rs:522-538`:

```rust
        *self.dueno.borrow_mut() = None;
        let (t, _) = tarea_de(estado, &self.id)?;
        if let Some((fallo, dueno)) = de_otra_persona(
            &self.id, t.asignado_a.as_ref(), &self.asignado, NOMBRE,
            ("de-otra-persona", &txt::APROBAR_DE_OTRA_PERSONA),
        ) {
            *self.dueno.borrow_mut() = dueno;
            return Err(fallo);
        }
        if t.estado != EN_DISENO {
            return Err(por_estado(&self.id, &t.estado, NOMBRE, &[EN_DISENO]));
        }
        if t.plan_aprobado {
            return Err(self.rechazo("plan-aprobado", Mensaje::nuevo(&txt::PLAN_APROBADO)));
```

(Los argumentos de `de_otra_persona` van en una linea aqui; en el fichero ocupan cinco, `guardar.rs:116-122` y `aprobar.rs:524-530`.)

Diff normalizado: `super::NOMBRE` (`plan`) frente a `NOMBRE` (`approve`) y `txt::DE_OTRA_PERSONA` frente a `txt::APROBAR_DE_OTRA_PERSONA`; el resto, igual token a token. `Planificar::aplicar` (`operacion.rs:367-386`) repite las dos primeras comprobaciones con los mismos argumentos que `Guardar` y cambia la de estado por `PLANIFICABLES`. Los metodos `dueno()` y `rechazo()` son identicos en los tres structs (`guardar.rs:83-89`, `aprobar.rs:492-498`, `operacion.rs:210-216`).

Arreglo esperado: una funcion `comprobar_diseno(estado, id, asignado, orden, texto, estados)` que devuelva la tarea o el rechazo con el dueno.

<a id="a2-06"></a>

#### A2-06 · D1 · IMPORTANTE · fusionado en A8-05

_Doble de `Plataforma` copiado entre el test unitario y el de integración de `revision`_

- id: A2-06
- regla: D1
- ubicacion: crates/tc2/src/revision/mod.rs:553-567 ~ crates/tc2/tests/revision.rs:115-129
- modulo: revision (tests)
- severidad: IMPORTANTE
- absorbido_por: A8-05
- normalizacion: consolidador TASK-138, 2026-11-22: fusionado en A8-05 (su campo `absorbe:`). No cuenta como caso aparte; su evidencia sigue valiendo y la tarea de A8-05 lo nombra.
- motivo_severidad: D1 real (regla 4D: minimo IMPORTANTE). El doble de `Plataforma` del test unitario de `revision` y el del test de integracion repiten, byte a byte, los cinco metodos que no usan (15 lineas). `trait Plataforma` (F1-13) obliga a implementar los metodos, no a copiarlos en cada doble: un doble nulo comun o metodos por defecto lo evitan. Codigo de test; no tapa fallo de datos ni de seguridad; no `divergente`.
- evidencia:

  `crates/tc2/src/revision/mod.rs:553-567`
  ```rust
          fn merge_pr(&self, _: u64, _: &str) -> Informe {
              sin_uso("merge-pr")
          }
          fn leer_puertas(&self, _: Rol, _: &str) -> Paso<ReglasLeidas> {
              Err(sin_uso("leer-puertas"))
          }
          fn leer_permisos_ci(&self) -> Informe {
              sin_uso("leer-permisos-ci")
          }
          fn firma_commit(&self, _: &str) -> Paso<Firma> {
  ```

  `crates/tc2/tests/revision.rs:115-124`
  ```rust
      fn merge_pr(&self, _: u64, _: &str) -> Informe {
          sin_uso("merge-pr")
      }
      fn leer_puertas(&self, _: Rol, _: &str) -> Paso<ReglasLeidas> {
          Err(sin_uso("leer-puertas"))
      }
      fn leer_permisos_ci(&self) -> Informe {
          sin_uso("leer-permisos-ci")
      }
      fn firma_commit(&self, _: &str) -> Paso<Firma> {
  ```

  Diff normalizado: identicos (solo cambia la sangria); siguen iguales `firma_commit` e `identidad` hasta `:567` / `:129`, y la funcion `sin_uso(op) -> Informe { Informe::nuevo(op, Resultado::Error) }` esta en los dos (`crates/tc2/src/revision/mod.rs:537-539`, `crates/tc2/tests/revision.rs:68-70`). 123 tokens normalizados (pase 0, ventana 551-570 ~ 113-132).
- estado: confirmado
- grupo_seguimiento: A8 (concepto «dobles de Plataforma» del plan, punto 3: hay mas dobles en identidad.rs, tablero.rs y tests/tablero.rs, ver pistas/A2.txt punto 5); si A8 no lo agrupa, A2

<a id="a4-09"></a>

#### A4-09 · D1 · IMPORTANTE · fusionado en A8-01, A8-16

_Ayudantes para lanzar `tc2` contra el remoto de la fixture copiados entre `toma` y `liberar`_

- id: A4-09
- regla: D1
- ubicacion: crates/tc2/tests/toma.rs:33-225 ~ crates/tc2/tests/liberar.rs:30-266
- modulo: tests de toma y liberar
- severidad: IMPORTANTE
- absorbido_por: A8-01 y A8-16
- normalizacion: consolidador TASK-138, 2026-11-22: fusionado en A8-01 y A8-16 (su campo `absorbe:`). No cuenta como caso aparte; su evidencia sigue valiendo y la tarea de A8-01 y A8-16 lo nombra.
- motivo_severidad: D1 real en tests: unas 100 lineas de ayudantes para lanzar `tc2` contra el remoto de la fixture por `GIT_SSH_COMMAND` copiadas entre los dos ficheros (y, fuera de A4, en otros nueve; ver pistas/A4.txt). Las firmas de `preparar` y de la operacion `Escribir` ya se separaron, pero el comportamiento de lo compartido es el mismo. No tapa fallo de datos.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A4 (coordinar con A8, dueno de tests/comun y de la D1 entre duenos)

##### Evidencia

`crates/tc2/tests/toma.rs:33-42`:

```rust
fn entorno_ssh(f: &Fixture) -> Vec<(OsString, OsString)> {
    let bare = f
        .raiz
        .join("remoto.git")
        .to_string_lossy()
        .replace('\\', "/");
    let mut e = f.entorno();
    e.push((
        "GIT_SSH_COMMAND".into(),
        format!(
```

`crates/tc2/tests/liberar.rs:30-39`: las mismas lineas (el pase 0 da 298 tokens normalizados iguales en liberar.rs:22-72 ~ toma.rs:23-77).

Pares identicos o casi (toma.rs ~ liberar.rs):
- `entorno_ssh` 33-49 ~ 30-46; `entorno_sin_red` 52-61 ~ 48-57; `lanzar` 63-74 ~ 59-70; `texto` 81-83 ~ 72-74; `valor` 100-104 ~ 112-116: identicos.
- `git` 86-97 ~ `git_con` 98-110: igual mas `.envs(env)`.
- `identificar` 107-116 ~ `cuenta` + `identificar` 118-130: misma `Cuenta` y mismo `Cache::guardar`.
- `preparar` 121-136 ~ 134-151: mismos pasos (`personas.yml`, commit, push, `tc2 new` por cada alta, clon `a` con la identidad de ana); liberar.rs anade `config`, toma.rs admite argumentos de `new`.
- `otro_clon` 139-143 ~ 153-157; `publicado` 146-148 ~ `tarea` 166-168; `punta_estado` 150-152 ~ `punta` 170-172; `worktree` 167-169 ~ 174-176: identicos con otro nombre.
- `start` 172-183 ~ `salida` 82-92: misma comprobacion de una linea en stdout y stderr vacio en exito.
- `Escribir` + `sin_catalogo` + `escribir_estado` 187-225 ~ `Escribir` + `sin_catalogo` + `transaccion` 221-266: misma operacion por la transaccion; cambia como se calcula el texto nuevo.

Diff normalizado: los nombres (`publicado`/`tarea`, `punta_estado`/`punta`, `start`/`salida`) y las dos extensiones citadas; el resto, igual.

<a id="a5-07"></a>

#### A5-07 · D1 · IMPORTANTE · fusionado en A8-02

_Lanzador de `tc2` con entorno aislado copiado en los tests de `recursos` y `doctor`_

- id: A5-07
- regla: D1 (Duplicacion)
- ubicacion: crates/tc2/tests/recursos.rs:143 (y crates/tc2/tests/doctor.rs:140; tercera copia parcial en crates/tc2/tests/seleccion.rs:29)
- modulo: tests de doctor, recursos y seleccion (lanzador del binario `tc2`)
- severidad: IMPORTANTE
- absorbido_por: A8-02
- normalizacion: consolidador TASK-138, 2026-11-22: fusionado en A8-02 (su campo `absorbe:`). No cuenta como caso aparte; su evidencia sigue valiendo y la tarea de A8-02 lo nombra.
- motivo_severidad: D1 real (IMPORTANTE como minimo): la misma funcion de 20+ lineas para lanzar `tc2` con el entorno aislado, copiada en dos ficheros de test de A5 (y en unos 20 ficheros de test de otros duenos, ver pista). No diverge en comportamiento (sin marca `divergente`), no tapa fallo de datos ni de seguridad. Es codigo de test: el riesgo es que una variable de aislamiento nueva (otro `TC2_TOKEN_*`) se anada en una copia y no en otra.
- estado: confirmado
- grupo_seguimiento: A5 — Seleccion, inventario, recursos y doctor (la familia completa entre duenos es de A8: abstraccion "lanzador tc2 en tests")

##### Evidencia

Fragmento 1 (tests/doctor.rs, 10 lineas):
```text
crates/tc2/tests/doctor.rs:141:     let o = Command::new(env!("CARGO_BIN_EXE_tc2"))
crates/tc2/tests/doctor.rs:142:         .current_dir(dir)
crates/tc2/tests/doctor.rs:143:         .args(args)
crates/tc2/tests/doctor.rs:144:         .envs(f.entorno())
crates/tc2/tests/doctor.rs:145:         .env("LC_ALL", "C")
crates/tc2/tests/doctor.rs:146:         .env("TC2_IDIOMA", "es")
crates/tc2/tests/doctor.rs:147:         .env("GIT_TERMINAL_PROMPT", "0")
crates/tc2/tests/doctor.rs:148:         .env("GIT_CEILING_DIRECTORIES", &f.raiz)
crates/tc2/tests/doctor.rs:152:         .env_remove("TC2_TOKEN_AGENTE")
crates/tc2/tests/doctor.rs:153:         .env_remove("TC2_TOKEN_PERSONA")
```

Fragmento 2 (tests/recursos.rs, 10 lineas):
```text
crates/tc2/tests/recursos.rs:154:     let o = c
crates/tc2/tests/recursos.rs:155:         .current_dir(dir)
crates/tc2/tests/recursos.rs:156:         .args(args)
crates/tc2/tests/recursos.rs:157:         .envs(f.entorno())
crates/tc2/tests/recursos.rs:158:         .env("LC_ALL", "C")
crates/tc2/tests/recursos.rs:159:         .env("TC2_IDIOMA", "es")
crates/tc2/tests/recursos.rs:160:         .env("GIT_TERMINAL_PROMPT", "0")
crates/tc2/tests/recursos.rs:161:         .env("GIT_CEILING_DIRECTORIES", &f.raiz)
crates/tc2/tests/recursos.rs:163:         .env_remove("TC2_TOKEN_AGENTE")
crates/tc2/tests/recursos.rs:164:         .env_remove("TC2_TOKEN_PERSONA")
```

Siguen iguales: `.env_remove("TC2_TOKEN_REVISOR")`, `.output().expect("lanzar tc2")` y el `(code.unwrap_or(-1), stdout lossy, stderr lossy)` (doctor.rs:154-162 y recursos.rs:165-172).

Diff normalizado: identicas salvo (a) `CLAUDE_CONFIG_DIR`: `f.raiz.join("claude")` en doctor.rs:151 frente al parametro `claude` en recursos.rs:162; (b) doctor.rs:155 anade `.envs(env)` y recursos.rs:151-153 un `PATH` opcional. tests/seleccion.rs:31-43 repite el mismo arranque (`Command::new(env!("CARGO_BIN_EXE_tc2"))`, `current_dir`, `envs(f.entorno())`, `LC_ALL`, `TC2_IDIOMA`, `GIT_CEILING_DIRECTORIES`, `CLAUDE_CONFIG_DIR`) sin quitar los `TC2_TOKEN_*`.

##### Arreglo sugerido

Un `comun::tc2(f, dir, args, claude, env)` en `tests/comun/mod.rs` (que ya tiene `Fixture::entorno`), usado por los tres.

<a id="a6-01"></a>

#### A6-01 · D1 · IMPORTANTE · fusionado en A8-06

_Conversión `Fallo -> Informe` escrita a mano en cuatro órdenes de A6_

- id: A6-01
- regla: D1
- ubicacion: crates/tc2/src/roadmap/mod.rs:303-313 ~ crates/tc2/src/estimacion/mod.rs:245-254 (y dos copias mas cortas en crates/tc2/src/siguiente.rs:76-85 y crates/tc2/src/tablero.rs:443-451)
- modulo: roadmap / estimacion / siguiente / tablero
- severidad: IMPORTANTE
- absorbido_por: A8-06
- normalizacion: consolidador TASK-138, 2026-11-22: fusionado en A8-06 (su campo `absorbe:`). No cuenta como caso aparte; su evidencia sigue valiendo y la tarea de A8-06 lo nombra.
- motivo_severidad: D1 real (regla 4D: minimo IMPORTANTE). Misma conversion `Fallo -> Informe` escrita a mano en cuatro ordenes de A6; ningun ADR la impone (F1-2 define `Fallo`, no como se pinta). No tapa un fallo de datos ni de seguridad. No `divergente`: la copia de estimacion anade `.con("id", id)` a proposito (la orden lleva ID).
- evidencia:

  `crates/tc2/src/roadmap/mod.rs:303-313`
  ```rust
  fn de_fallo(f: Fallo) -> Informe {
      match f {
          Fallo::Rechazada {
              motivo,
              diagnostico,
          } => Informe::nuevo(NOMBRE, Resultado::Rechazada)
              .con("motivo", motivo)
              .diagnostico(diagnostico),
          Fallo::Error(d) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
      }
  }
  ```

  `crates/tc2/src/estimacion/mod.rs:245-254`
  ```rust
      let rechazo = |f: Fallo| match f {
          Fallo::Rechazada {
              motivo,
              diagnostico,
          } => Informe::nuevo(NOMBRE, Resultado::Rechazada)
              .con("id", id)
              .con("motivo", motivo)
              .diagnostico(diagnostico),
          Fallo::Error(d) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
      };
  ```

  Diff normalizado (nombres -> comodin): identicos salvo `.con("id", id)` en estimacion y la forma (fn frente a closure). Las otras dos copias, mismo cuerpo con otro nombre de orden:
  - `crates/tc2/src/siguiente.rs:78-84`: `Err(Fallo::Rechazada { motivo, diagnostico, }) => Informe::nuevo(NOMBRE, Resultado::Rechazada).con("motivo", motivo).diagnostico(diagnostico), Err(Fallo::Error(d)) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),`
  - `crates/tc2/src/tablero.rs:443-451`: `situacion::calcular(&vista).map_err(|f| match f { Fallo::Rechazada { motivo, diagnostico, } => Informe::nuevo(op, Resultado::Rechazada).con("motivo", motivo).diagnostico(diagnostico), Fallo::Error(d) => Informe::nuevo(op, Resultado::Error).diagnostico(d), })?;`

  Extraccion propuesta: `fn informe_de_fallo(comando: &str, f: Fallo) -> Informe` junto a `Fallo` (estado), y estimacion anade su `id` encima.
- estado: confirmado
- grupo_seguimiento: A6 (la misma abstraccion tiene copias fuera de A6: ver pistas/A6.txt punto 5; A8 decide si la agrupa en su tarea de abstraccion comun)

<a id="a7-02"></a>

#### A7-02 · D1 · IMPORTANTE divergente · fusionado en A8-04

_Escritura atómica (temporal + rename) en `desinstalar` y `confianza`, ya divergente_

- id: A7-02
- regla: D1
- ubicacion: crates/tc2/src/desinstalar/mod.rs:571-588 (escribir_atomico) ~ crates/tc2/src/confianza/mod.rs:160-178 (escribir_almacen)
- modulo: desinstalar / confianza
- severidad: IMPORTANTE (divergente)
- absorbido_por: A8-04
- normalizacion: consolidador TASK-138, 2026-11-22: fusionado en A8-04 (su campo `absorbe:`). No cuenta como caso aparte; su evidencia sigue valiendo y la tarea de A8-04 lo nombra.
- motivo_severidad: D1 real: las dos escriben «temporal al lado + rename + borrar el temporal si falla». Ya divergen en comportamiento: el nombre del temporal (`<nombre>.tc2-tmp-<pid>` frente a `<ruta>.json.<pid>.tmp`), `escribir_atomico` conserva permisos y sigue enlaces simbolicos, y `escribir_almacen` crea la carpeta y solo borra el temporal si falla el `rename` (si falla `write`, el temporal a medias se queda). Ninguna reintenta el `rename`, cuando la escritura atomica comun del proyecto, `huella::escribir_local` (crates/tc2/src/huella.rs:89-116, fuera de A7), reintenta por un antivirus en Windows. No tapa un fallo de datos (el almacen de confianza falla cerrado: si no se lee, todo vuelve a `nunca`), asi que no sube a CRITICO. Ningun ADR impone dos implementaciones.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A7

##### Evidencia

`crates/tc2/src/desinstalar/mod.rs:578-587`
```rust
    let mut nombre = destino.file_name().unwrap_or_default().to_os_string();
    nombre.push(format!(".tc2-tmp-{}", std::process::id()));
    let tmp = destino.with_file_name(nombre);
    let r = std::fs::write(&tmp, contenido)
        .and_then(|()| std::fs::set_permissions(&tmp, permisos))
        .and_then(|()| std::fs::rename(&tmp, &destino));
    if r.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    r
```

`crates/tc2/src/confianza/mod.rs:168-177`
```rust
    if let Some(padre) = ruta.parent() {
        std::fs::create_dir_all(padre).map_err(|e| error(&e))?;
    }
    let texto = serde_json::to_string_pretty(a).map_err(|e| error(&e))?;
    let tmp = ruta.with_extension(format!("json.{}.tmp", std::process::id()));
    std::fs::write(&tmp, texto).map_err(|e| error(&e))?;
    std::fs::rename(&tmp, ruta).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        error(&e)
    })
```

Diff normalizado: mismo esqueleto `tmp = ruta + sufijo(pid)`; `write(tmp)`; `rename(tmp, ruta)`; `remove_file(tmp)` en error. Difieren en: sufijo del temporal, `set_permissions` y `canonicalize` del enlace (solo desinstalar), `create_dir_all` (solo confianza), y en que confianza no borra el temporal cuando falla `write` (`?` en la linea 173 sale antes del `remove_file`).

La tercera copia, `huella::escribir_local` (huella.rs:89-116, de A8), va como pista en `pistas/A7.txt`.

<a id="a7-06"></a>

#### A7-06 · D1 · IMPORTANTE divergente · fusionado en A8-12

_Instantánea de carpeta copiada entre los tests de `init` y `desinstalar`, ya divergente_

- id: A7-06
- regla: D1
- ubicacion: crates/tc2/tests/init.rs:210-232 (instantanea) ~ crates/tc2/tests/desinstalar.rs:35-62 (instantanea)
- modulo: tests de init y desinstalar
- severidad: IMPORTANTE (divergente)
- absorbido_por: A8-12
- normalizacion: consolidador TASK-138, 2026-11-22: fusionado en A8-12 (su campo `absorbe:`). No cuenta como caso aparte; su evidencia sigue valiendo y la tarea de A8-12 lo nombra.
- motivo_severidad: D1 real en codigo de test: la misma «instantanea de una carpeta» (recorrido recursivo, ruta relativa con `/`, marca de carpeta) copiada en los dos ficheros de test, 20 lineas por lado. Ya divergen: init guarda el contenido entero y marca las carpetas con la clave `rel/`; desinstalar guarda `"<len> <sha256>"` y marca la carpeta con el valor `"dir"`. Los dos tests la usan para lo mismo («no cambia ni un byte»). Tambien `stdout(h)` es identico (init.rs:197-199 ~ desinstalar.rs:92-94). No tapa un fallo de datos ni de seguridad; ningun ADR lo impone.
- estado: confirmado (lectura contra d82ea9d)
- grupo_seguimiento: A7

##### Evidencia

`crates/tc2/tests/init.rs:211-226`
```rust
fn instantanea(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn recorrer(base: &Path, d: &Path, m: &mut BTreeMap<String, Vec<u8>>) {
        let Ok(it) = std::fs::read_dir(d) else { return };
        for e in it.flatten() {
            let p = e.path();
            let rel = p.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/");
            if p.is_dir() {
                m.insert(format!("{rel}/"), Vec::new());
                recorrer(base, &p, m);
            } else {
                m.insert(rel, std::fs::read(&p).unwrap());
```
`crates/tc2/tests/desinstalar.rs:37-55`
```rust
fn instantanea(dir: &Path) -> BTreeMap<String, String> {
    fn recorrer(base: &Path, d: &Path, m: &mut BTreeMap<String, String>) {
        let Ok(it) = std::fs::read_dir(d) else { return };
        for e in it.flatten() {
            let p = e.path();
            let rel = p.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/");
            if p.is_dir() {
                m.insert(rel, "dir".to_owned());
                recorrer(base, &p, m);
            } else {
                let b = std::fs::read(&p).unwrap();
                m.insert(rel, format!("{} {}", b.len(), distribucion::sha256::hex(&b)));
```
Las dos terminan igual (init.rs:229-231 ~ desinstalar.rs:59-61: `let mut m = BTreeMap::new(); recorrer(dir, dir, &mut m); m`).

Diff normalizado: tipo del valor (`Vec<u8>` / `String`), marca de carpeta (clave `rel/` / valor `"dir"`) y contenido (bytes / `len sha256`). El pase 0 lo marco (desinstalar.rs:38-48 ~ init.rs:212-222, 75 tokens; `fn stdout` con similitud 1.00).

Arreglo: una sola `instantanea` en `tests/comun` (A8), que es donde ya esta `Fixture`.

## Cobertura por módulo

Hay una fila por módulo de `src`, por fichero de `tests` y por cada fichero o carpeta del resto, sacada de `git ls-tree -r d82ea9d` fuera de `tareas/` y `docs/` (283 ficheros). Cada caso distinto cuenta una sola vez, en el módulo de su primera ubicación, así que las columnas D suman 108. Un 0 es explícito. «Remite a» lista los casos de abstracción común con un extremo en el módulo y que cuentan en otro. «Lectura»: completa, mecánica (fixtures de datos y ficheros generados recorridos enteros por script y por uso, sin releer cada valor) o parcial (ver la sección siguiente).

| Módulo | Ficheros | Auditor | Lectura | D1 | D2 | D3 | D4 | Casos | Remite a |
|---|---|---|---|---|---|---|---|---|---|
| `.github/scripts/comprobar-repo.sh` | 1 | A10 | completa | 0 | 0 | 0 | 0 | — | — |
| `.github/scripts/humo.sh` | 1 | A10 | completa | 0 | 0 | 0 | 0 | — | — |
| `.github/scripts/probar-actualizar.sh` | 1 | A10 | completa | 0 | 0 | 0 | 1 | A10-01 | — |
| `.github/scripts/probar-instalador.sh` | 1 | A10 | completa | 0 | 0 | 0 | 1 | A10-02 | — |
| `.github/workflows/ci.yml` | 1 | A8, A11 | completa | 0 | 1 | 0 | 0 | A11-06 | — |
| `.github/workflows/release.yml` | 1 | A8, A11 | completa | 0 | 0 | 0 | 1 | A11-01 | — |
| `.github/workflows/revision-ia.yml` | 1 | A2, A8, A11 | completa | 0 | 0 | 0 | 0 | — | — |
| `.github/workflows/verificacion.yml` | 1 | A8, A10 | completa | 0 | 0 | 0 | 0 | — | — |
| `configuración y manifiestos` | 11 | A2, A8, A9, A10 | completa 10, mecánica (generado) 1 | 0 | 0 | 0 | 0 | — | — |
| `crates/distribucion/` | 2 | A8, A9, A10 | completa | 0 | 0 | 1 | 0 | A9-04 | — |
| `crates/tc2/esquemas/` | 3 | A2, A7 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/examples/` | 2 | A8, A9 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/src/actualizar_cli.rs` | 1 | A8, A9 | completa | 2 | 0 | 0 | 1 | A8-13, A9-01, A9-02 | — |
| `crates/tc2/src/alta.rs` | 1 | A3 | completa | 1 | 1 | 1 | 1 | A3-01, A3-08, A3-09, A8-03 | — |
| `crates/tc2/src/carril.rs` | 1 | A8, A11 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/src/ciclo/` | 3 | A7 | completa | 5 | 0 | 0 | 0 | A7-04, A7-05, A8-09, A8-10, A8-18 | A8-06, A8-07, A8-08, A8-19, A8-20 |
| `crates/tc2/src/confianza/` | 2 | A7 | completa | 1 | 0 | 0 | 0 | A7-03 | A8-04 |
| `crates/tc2/src/config/` | 3 | A7 | completa | 0 | 0 | 0 | 1 | A7-09 | — |
| `crates/tc2/src/contrato.rs` | 1 | A8, A11 | completa | 0 | 0 | 0 | 1 | A11-04 | — |
| `crates/tc2/src/dato.rs` | 1 | A8, A11 | completa | 0 | 0 | 0 | 1 | A11-03 | — |
| `crates/tc2/src/descarga.rs` | 1 | A8, A9, A10 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/src/desinstalar/` | 5 | A7 | completa | 0 | 0 | 0 | 0 | — | A8-04 |
| `crates/tc2/src/doctor/` | 6 | A5 | completa | 0 | 0 | 2 | 0 | A5-11, A5-12 | — |
| `crates/tc2/src/estado/` | 8 | A4 | completa | 1 | 0 | 1 | 0 | A4-03, A4-11 | A8-17, A8-20 |
| `crates/tc2/src/estimacion/` | 5 | A6 | completa | 0 | 0 | 0 | 0 | — | A8-06 |
| `crates/tc2/src/git/` | 9 | A4 | completa | 6 | 0 | 1 | 1 | A4-01, A4-02, A4-04, A4-05, A4-06, A4-07, A4-10, A4-12 | A8-20 |
| `crates/tc2/src/grafo.rs` | 1 | A6 | completa | 1 | 0 | 0 | 0 | A6-02 | — |
| `crates/tc2/src/hallazgos/` | 4 | A2 | completa | 4 | 2 | 1 | 3 | A2-02, A2-03, A2-04, A2-05, A2-07, A2-08, A2-09, A2-11, A2-12, A2-13 | A8-17 |
| `crates/tc2/src/huella.rs` | 1 | A8, A11 | completa | 1 | 0 | 0 | 0 | A8-04 | — |
| `crates/tc2/src/i18n/` | 27 | A7 | completa | 0 | 0 | 0 | 1 | A7-10 | — |
| `crates/tc2/src/identidad.rs` | 1 | A1 | completa | 2 | 0 | 0 | 0 | A8-05, A8-17 | — |
| `crates/tc2/src/importar/` | 2 | A3 | completa | 0 | 0 | 0 | 0 | — | A8-06, A8-17 |
| `crates/tc2/src/init/` | 3 | A7 | completa | 1 | 1 | 1 | 0 | A7-01, A7-07, A7-08 | — |
| `crates/tc2/src/instalacion.rs` | 1 | A8, A11 | completa | 0 | 0 | 0 | 2 | A8-15, A11-02 | — |
| `crates/tc2/src/inventario.rs` | 1 | A5 | completa | 0 | 0 | 1 | 0 | A5-14 | — |
| `crates/tc2/src/lib.rs` | 1 | A8, A9 | completa | 0 | 0 | 2 | 0 | A8-14, A8-21 | — |
| `crates/tc2/src/main.rs` | 1 | A8, A11 | completa | 0 | 1 | 0 | 0 | A11-05 | — |
| `crates/tc2/src/plan/` | 7 | A3 | completa | 2 | 0 | 0 | 1 | A3-02, A3-03, A3-05 | A8-07, A8-08, A8-09, A8-10, A8-17, A8-18, A8-19 |
| `crates/tc2/src/plataforma/` | 12 | A1, A10 | completa | 3 | 0 | 1 | 2 | A1-01, A1-02, A1-03, A1-07, A1-08, A1-12 | — |
| `crates/tc2/src/progreso/` | 4 | A6 | completa | 0 | 0 | 0 | 1 | A6-07 | A8-17 |
| `crates/tc2/src/recursos.rs` | 1 | A5 | completa | 0 | 1 | 0 | 0 | A5-05 | — |
| `crates/tc2/src/revision/` | 3 | A2 | completa | 3 | 0 | 1 | 0 | A2-10, A8-07, A8-08, A8-19 | A8-05, A8-06, A8-18 |
| `crates/tc2/src/roadmap/` | 4 | A6 | completa | 1 | 0 | 0 | 0 | A8-06 | A8-18 |
| `crates/tc2/src/seleccion/` | 5 | A5 | completa | 2 | 2 | 2 | 1 | A5-01, A5-02, A5-03, A5-04, A5-06, A5-10, A5-13 | A8-06, A8-17 |
| `crates/tc2/src/siguiente.rs` | 1 | A6 | completa | 1 | 0 | 0 | 0 | A6-05 | A8-06, A8-18 |
| `crates/tc2/src/situacion.rs` | 1 | A6 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/src/solape.rs` | 1 | A6 | completa | 0 | 0 | 1 | 0 | A6-08 | — |
| `crates/tc2/src/tablero.rs` | 1 | A6 | completa | 0 | 0 | 0 | 0 | — | A8-05, A8-06 |
| `crates/tc2/src/tarea/` | 5 | A3 | completa | 0 | 1 | 0 | 0 | A3-07 | — |
| `crates/tc2/src/toma/` | 2 | A4 | completa | 0 | 0 | 0 | 0 | — | A8-03, A8-06, A8-07, A8-08, A8-18, A8-19 |
| `crates/tc2/src/verificacion/` | 3 | A7, A9 | completa | 0 | 0 | 0 | 0 | — | A9-01 |
| `crates/tc2/src/version.rs` | 1 | A8, A11 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/src/yaml.rs` | 1 | A8 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/actualizar_cli.rs` | 1 | A8, A11 | completa | 0 | 0 | 0 | 0 | — | A8-11 |
| `crates/tc2/tests/agente_git/` | 1 | A4 | completa | 1 | 0 | 0 | 0 | A8-20 | — |
| `crates/tc2/tests/alta.rs` | 1 | A3, A9 | completa | 1 | 0 | 0 | 0 | A3-06 | A8-02 |
| `crates/tc2/tests/aprobar.rs` | 1 | A3 | completa | 2 | 0 | 0 | 0 | A8-01, A8-02 | A3-06 |
| `crates/tc2/tests/ciclo.rs` | 1 | A7 | completa | 1 | 0 | 0 | 0 | A8-16 | A8-01, A8-02 |
| `crates/tc2/tests/cli.rs` | 1 | A8 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/comun/` | 3 | A8, A9 | completa | 0 | 0 | 0 | 0 | — | A8-11, A8-20 |
| `crates/tc2/tests/config.rs` | 1 | A7 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/convivencia_016.rs` | 1 | A4 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/desinstalar.rs` | 1 | A7 | completa | 0 | 0 | 0 | 0 | — | A8-12 |
| `crates/tc2/tests/doctor.rs` | 1 | A5 | completa | 2 | 0 | 0 | 0 | A5-08, A5-09 | A8-12 |
| `crates/tc2/tests/estado_bordes.rs` | 1 | A4 | completa | 1 | 0 | 0 | 0 | A4-08 | — |
| `crates/tc2/tests/estado_casos.rs` | 1 | A4 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/estado_concurrencia.rs` | 1 | A4 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/estimacion.rs` | 1 | A6 | completa | 1 | 0 | 0 | 0 | A6-03 | A8-01, A8-02 |
| `crates/tc2/tests/fixtures/estimacion/` | 1 | A6, A10 | mecánica | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/fixtures/hallazgos/` | 7 | A2, A10 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/fixtures/informe-verificacion/` | 2 | A7, A10 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/fixtures/inyeccion/` | 1 | A2, A10 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/fixtures/muestra-f1-15/` | 1 | A5, A10 | mecánica | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/fixtures/plataforma/` | 14 | A10 | completa 1, mecánica 13 | 0 | 0 | 1 | 0 | A10-04 | — |
| `crates/tc2/tests/fixtures/revisoras/` | 5 | A2, A10 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/fixtures/roadmap/` | 1 | A6, A10 | mecánica | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/fixtures/seleccion-taskcodetwo/` | 22 | A10 | completa 2, mecánica 20 | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/fixtures/toolsaweb-1fc94425/` | 3 | A6, A10 | completa 2, mecánica 1 | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/frescura.rs` | 1 | A6 | completa | 0 | 0 | 0 | 0 | — | A8-01, A8-12 |
| `crates/tc2/tests/git_binario.rs` | 1 | A4 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/git_casos_limite.rs` | 1 | A4 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/git_ciclo.rs` | 1 | A4 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/git_conflicto.rs` | 1 | A4 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/git_push.rs` | 1 | A4 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/hallazgos.rs` | 1 | A2 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/identidad.rs` | 1 | A1 | completa | 1 | 0 | 0 | 0 | A8-11 | — |
| `crates/tc2/tests/identidades.rs` | 1 | A1 | completa | 0 | 0 | 0 | 0 | — | A8-03 |
| `crates/tc2/tests/importar.rs` | 1 | A3 | completa | 0 | 0 | 0 | 0 | — | A3-06, A8-12 |
| `crates/tc2/tests/init.rs` | 1 | A1, A7 | completa | 1 | 0 | 0 | 1 | A7-11, A8-12 | A8-11 |
| `crates/tc2/tests/inyeccion.rs` | 1 | A2 | completa | 0 | 0 | 0 | 0 | — | A8-01 |
| `crates/tc2/tests/liberar.rs` | 1 | A4 | completa | 0 | 0 | 0 | 0 | — | A8-01, A8-16 |
| `crates/tc2/tests/plan.rs` | 1 | A3 | completa | 0 | 0 | 0 | 0 | — | A3-06, A8-01, A8-16 |
| `crates/tc2/tests/plataforma.rs` | 1 | A1 | completa | 2 | 0 | 0 | 1 | A1-04, A1-05, A1-11 | — |
| `crates/tc2/tests/plataforma_e2e.rs` | 1 | A1 | completa | 1 | 0 | 0 | 0 | A1-06 | — |
| `crates/tc2/tests/plataforma_gitlab.rs` | 1 | A1 | completa | 0 | 0 | 0 | 1 | A1-10 | — |
| `crates/tc2/tests/plataforma_gitlab_e2e.rs` | 1 | A1 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/progreso.rs` | 1 | A6 | completa | 0 | 0 | 0 | 0 | — | A8-01 |
| `crates/tc2/tests/prs_abiertos.rs` | 1 | A1 | completa | 0 | 0 | 0 | 1 | A1-09 | — |
| `crates/tc2/tests/recursos.rs` | 1 | A5 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/revision.rs` | 1 | A2 | completa | 0 | 0 | 0 | 0 | — | A8-05 |
| `crates/tc2/tests/revisoras.rs` | 1 | A2 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/roadmap.rs` | 1 | A6 | completa | 1 | 0 | 0 | 0 | A6-04 | — |
| `crates/tc2/tests/seleccion.rs` | 1 | A5 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/siguiente.rs` | 1 | A6 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/solape.rs` | 1 | A6 | completa | 1 | 0 | 0 | 0 | A6-06 | A8-01, A8-16 |
| `crates/tc2/tests/tablero.rs` | 1 | A6 | completa | 0 | 0 | 0 | 0 | — | A8-05, A8-12 |
| `crates/tc2/tests/tarea.rs` | 1 | A3 | completa | 0 | 0 | 0 | 0 | — | — |
| `crates/tc2/tests/toma.rs` | 1 | A4 | completa | 0 | 0 | 0 | 0 | — | A8-01, A8-16 |
| `crates/tc2/tests/verificar.rs` | 1 | A7 | completa | 0 | 0 | 0 | 0 | — | — |
| `documentación de la raíz (README, CHANGELOG, CLAUDE.md, LICENSE)` | 4 | A11 | completa | 0 | 1 | 0 | 0 | A11-07 | — |
| `instalar/` | 2 | A10 | completa | 0 | 0 | 0 | 1 | A10-05 | — |
| `plantillas/` | 1 | A10 | completa | 0 | 1 | 0 | 0 | A10-03 | — |
| `xtask/` | 3 | A8, A9, A10 | completa | 1 | 0 | 0 | 0 | A9-03 | — |
| **Total** | **283** | | | **54** | **12** | **17** | **25** | **108** | |

La fila «configuración y manifiestos» agrupa estos 11 ficheros, todos con lectura completa salvo `Cargo.lock` (mecánica): `.cargo/config.toml`, `.gitattributes`, `.gitignore`, `.taskcode/config.yml`, `Cargo.lock`, `Cargo.toml`, `crates/distribucion/Cargo.toml`, `crates/tc2/Cargo.toml`, `rust-toolchain.toml`, `rustfmt.toml`, `xtask/Cargo.toml`. Los `Cargo.toml` de `crates/distribucion/` y de `xtask/` cuentan aquí y no en la fila de su carpeta.

Los E2E con tokens reales (`crates/tc2/tests/plataforma_e2e.rs`, `crates/tc2/tests/plataforma_gitlab_e2e.rs`) se leyeron enteros, pero son **no verificables por mutación**: van con `#[ignore]` y necesitan GitHub y GitLab reales. Su único caso (A1-06) es D1 de lectura.

## Ficheros sin lectura completa

Carlos pidió que no quedara ningún fichero sin mirar, así que aquí se dice sin rebajas qué no se leyó entero. La lista también está en `task-138/faltan.txt`, fuera del repo.

Cruce rehecho tras A11: las 226 líneas de `alcance.txt` y los 283 ficheros de `git ls-tree -r --name-only d82ea9d` fuera de `tareas/` y `docs/` están todos en alguna cobertura por ruta exacta, y todos tienen lectura completa salvo los de lectura mecánica declarada (38 ficheros):

| Ficheros | Lectura | Quién |
|---|---|---|
| `Cargo.lock` | mecánica: generado por cargo; se recorrió por script, no se relee cada versión | A8 |
| 37 fixtures de datos grandes de `crates/tc2/tests/fixtures/` (`estimacion/paridad*`, `muestra-f1-15/`, `plataforma/` salvo `observado-e2e.json`, `roadmap/referencia.md`, `seleccion-taskcodetwo/TASK-*`, `toolsaweb-1fc94425/tareas.tsv`) | mecánica: recorridas enteras por script (formato válido, uso y paridad entre pares), sin releer cada valor | A10 |

Antes de A11 había 14 ficheros sin lectura completa: los 4 documentos de la raíz sin auditor (`CHANGELOG.md`, `CLAUDE.md`, `LICENSE`, `README.md`) y 10 que A8 leyó por zonas (`crates/tc2/src/huella.rs`, `crates/tc2/src/instalacion.rs`, `crates/tc2/src/version.rs`, `crates/tc2/src/contrato.rs`, `crates/tc2/src/dato.rs`, `crates/tc2/src/carril.rs`, `crates/tc2/tests/actualizar_cli.rs`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `.github/workflows/revision-ia.yml`). A11 los leyó enteros (`task-138/cobertura/A11.md`) y de ahí salen A11-01 a A11-07.

## Descartes justificados por un ADR

- `trait Plataforma`, con `plataforma/github.rs` y `plataforma/gitlab.rs`: implementan el mismo trait por F1-13. `metodo_y_ruta`, `ep`, `exigir_persona` y los cuerpos de `estado_puertas`, `puertas_de_rama`, `firma_commit`, `leer_plan` y `aplicar_puertas` son API propia de cada plataforma (F1-13, decisiones 1 y 4); lo neutral ya está en `puertas::evaluar` y `aplicar_en_lote`. Solo cuenta la lógica que no depende de la API (A1-01, A1-02, A1-03).
- Fixtures y E2E paralelos de GitHub y GitLab: son la misma batería por plataforma (F2-9, PoC M de F1-13, RR-15). Solo es caso su arnés común (A1-04, A1-06).
- `instalar/install.sh` frente a `instalar/install.ps1`: hay dos lenguajes, uno por sistema, porque lo impone F1-14 (fila F2-11) y TASK-026. La tabla de alias que repiten la vigila `xtask/src/instaladores.rs`.
- `TC2_BASE_URL`: los instaladores aceptan `localhost` y `tc2` solo `127.0.0.1`. La divergencia está declarada en `docs/distribucion/instalar.md:126-127`.
- Plantilla de GitLab frente a `verificacion.yml`: van en paralelo por plataforma (F2-9, F1-13) y las divergencias de búsqueda, publicador y versión en la clave están declaradas (`docs/ci/verificacion-reutilizable.md`, divergencias 3 y 17). La única no declarada es A10-03.
- Campos repetidos de los structs serde de `hallazgos`: `#[serde(flatten)]` no admite `deny_unknown_fields`, y el contrato exige que una clave desconocida falle cerrado (F1-8, adenda F6-4).
- `roles.revision` deja `regla` como estaba: es una divergencia aceptada en F1-15 (adendas F6-3 y F4-6).
- `CrudoV2`/`CrudoV1` y `Tarea::leer`/`TareaV1::leer`: F1-9, punto 3, impone dos formatos de tarea (el 1 solo se lee, para migrar).
- `registro_permitido` (`tests/comun/mod.rs:215-281`) frente a `git::puerto::es_prohibido`: es el detector independiente por lista permitida de F1-4 B2b, y duplicar el criterio es justo lo que pide el ADR.
- `borrar_rama_de_estado` no pasa por `lanzar`, y la rama de estado no usa `reset` ni force: F1-2 (D1-D2), F1-4 (D3) y F2-14 C7.
- Secuencias largas `paso` (montaje), `intento` (transacción) y `actualizar` (remoto): son los pasos de F1-1 D2, F1-2 D1-D2 y F1-4. Su parte repetida es A4-03.
- `capacidades.rs` `TABLA` copia `tabla-equivalencias` de F1-13 porque así lo diseña el módulo, y un test la compara fila a fila con el ADR. La etiqueta «plegada» de `dominios.rs` la justifica la adenda F4-6 de F1-15.
- `estimacion/real.rs` `tabla` e `historico.rs` `troceo`: F1-17 (D6a) exige paridad con `poc-f1-17.py`. El `let _ = stdin` de `progreso/mod.rs:369` cumple la adenda de TASK-084 («siempre una línea y código 0»).
- Pasos repetidos en los workflows («Compilar tc2», «Toolchain y target»): compartirlos exigiría una acción compuesta, y el lint de xtask solo admite `checkout` y `upload-artifact` (F1-8). Además están por debajo del umbral.
- `tc2_falso` arma a mano el JSON de `version --json` para no enlazar la biblioteca (TASK-027). La `Carpeta` de `tests/actualizar_cli.rs` vive en `CARGO_TARGET_TMPDIR` a propósito, por el antivirus (TASK-027). `sha256.rs` es propio para no traer dependencias a la verificación.

Los demás descartes de cada auditor (por debajo del umbral, ruido del pase 0, avisos de clippy sin una versión reducida) se razonaron en sus ficheros de cobertura. No se trasladan aquí porque no son casos.

## Límites

- **E2E con tokens reales** (`plataforma_e2e.rs`, `plataforma_gitlab_e2e.rs`): no se mutaron ni se ejecutaron, porque un mutante contra GitHub o GitLab reales crea PR, aprobaciones o ramas que git no deshace. Son no verificables por mutación.
- **wget**: A10-05 queda NO VERIFICADO. No hay `wget` en la máquina y el plan prohíbe descargar. Su tarea lo reproduce primero, con un servidor en 127.0.0.1 que responda 302 a una URL http.
- **Modelo real**: no se ejecutó ningún flujo con un modelo de IA real (workflow `revision-ia`, `tc2 hallazgos` contra respuestas reales). Los D4 de `hallazgos` se probaron con mutantes sobre los tests y las fixtures del repo.
- **D4 de código sin ejecutar**: A1-07 y A7-09 dan una entrada concreta derivada del código y no se ejecutó `tc2` con ella. A4-10 ejecutó git real, pero no `tc2`.
- **Lectura parcial**: ya no queda ninguna. Los 10 ficheros que A8 leyó por zonas y los 4 documentos de la raíz sin auditor los leyó enteros A11; solo `Cargo.lock` y las fixtures de datos grandes tienen lectura mecánica declarada (ver «Ficheros sin lectura completa»).
- **Falsos negativos de D1**: un clon semántico (`match` en un sitio, `if let` en otro) se escapa del hash normalizado. Lo mitigan las pistas de los siete auditores y la pasada por conceptos de A8. Queda un residuo que se acepta. Sin `jscpd` no se puede contrastar.
- **D2 sin reabrir los ADR**: se midió contra la tabla `módulo → ADR → TASK` y las citas de cabecera. Una desviación de un ADR que el código no cita se puede escapar.
- **Fixtures grandes**: se recorrieron enteros por script (que el formato sea válido, uso, paridad entre pares), pero no se releyó cada valor.
- **La mutación es una muestra**: solo se mutaron los candidatos D4 que los auditores confirmaron leyendo, no todo el workspace. Un test débil que nadie marcó no aparece aquí.

## Normalización del consolidador

Cada cambio queda anotado en su ficha con una línea `normalizacion:` (visible en el detalle):

- A10-01 y A10-02 suben de MENOR a IMPORTANTE. Son Disfraz real en scripts de verificación: uno tiene una guarda que nunca para y el otro da verde sin comprobar.
- A10-05 sube de MENOR a IMPORTANTE con la marca **no verificado** (wget).
- A3-08 y A7-07 suben de MENOR a IMPORTANTE. Son D2 que cambian el comportamiento observable: el alta acepta lo que RF-01 manda rechazar, y un diagnóstico sale en dos idiomas.
- A8-15 y A9-02 pierden «pendiente de confirmar por mutación»: la mutación los confirmó.
- A4-10 queda en regla D4; se consideró CRITICO y se descartó (ver el resumen).
- A5-01 y A5-02 se marcan `divergente`: su propio motivo dice que las copias ya se comportan distinto.
- Fusiones según `absorbe:`: A2-01 y A3-04 → A8-07; A2-06 → A8-05; A4-09 → A8-01 y A8-16; A5-07 → A8-02; A6-01 → A8-06; A7-02 → A8-04; A7-06 → A8-12. A3-06 se fusiona solo en parte: su resto queda como caso del grupo XT. A2-10 queda aparte, porque A8-06 solo lo «relaciona».
- A11-02, A11-03 y A11-04 pierden «pendiente-mutacion»: la mutación los confirmó (sección «Mutación» de cada ficha). A11-01 (el mismo patrón de A10-01 en `release.yml:284`) queda como caso propio relacionado con A10-01, en el mismo grupo A10. A11-05, A11-06 y A11-07 son D2 que no cambian el comportamiento: MENOR, sin tarea.
- Los 18 D4 de test con mutación (A1-08, A1-09, A1-10, A1-11, A2-11, A2-12, A2-13, A3-01, A3-02, A5-06, A6-07, A7-10, A7-11, A8-15, A9-02, A11-02, A11-03, A11-04) tienen su sección «Mutación» con el resultado confirmado. Ningún D4 de test cita un test `#[ignore]` ni un mutante que no compile. Comprobación automática: los 18 traen `test result: ok` del test nombrado con el mutante puesto y un control en rojo.
- Comprobación automática de `fichero:línea`: la primera ubicación de cada ficha existe en el commit auditado y su línea está dentro del fichero (sin fallos).
