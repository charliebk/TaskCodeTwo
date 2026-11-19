# Informe de revisión — TASK-044 (ronda 1)

- Commit revisado: c9a85505931de5de8ec1231c552f61d2462bf4c1
- Revisor: code-quality-reviewer
- Veredicto: aprobada

Alcance: completa, pase genérico (el diff cae en Rust, YAML de CI y documentación; sin revisor de dominio único). Profundidad: a fondo en el contrato esquema-lector-productor, el lint de workflows y el workflow; por encima el texto de la guía.

## Verificación

- Puerta: se cita la verificación completa en verde de la ronda para el commit revisado (la petición la trae); no se repitió la suite entera.
- `cargo test -p xtask`: 25 pasan, 0 fallan (incluye `el_informe_es_el_fixture_de_tc2` y `el_informe_se_sube_siempre_y_solo_desde_ejecutar`).
- `cargo test -p tc2 --lib verificacion::`: 2 pasan, 0 fallan.
- Prueba a mano de `TC2_INFORME=<ruta> cargo xtask verificar --perfil rapida`: el perfil completo (`cargo test --workspace`, con los tests de integración que lanzan `tc2.exe`) no terminó en más de 20 minutos en esta máquina y se abortó (no es un fallo del cambio: la verificación completa ya vino en verde). Para ejercitar el comando real se forzó el camino en rojo con `RUSTFLAGS=-Dwarnings`, que corta en `guarda_rustflags` antes de cualquier paso: el binario escribió el informe, imprimió la ruta y salió con ROJO; el JSON es válido (`json.load` correcto), con `commit` real de 40 hex, `inicio`/`fin` en forma, `resultado: rojo` y `pasos: []` (con una línea en blanco dentro del array, solo estética; el lector lo acepta por construcción: rojo con cero pasos no viola ninguna regla). El camino verde con pasos reales no se ejecutó a mano; lo cubre la prueba de fixture byte a byte.
- Mutación sobre el lint: `workflows.rs:505-509` (subida solo en clase ejecuta) y la comprobación de SHA (`:499`) tienen su prueba negativa (`actions/upload-artifact en clase privilegio` y `sin fijar por SHA`); el SHA `ea165f8d…` de `upload-artifact` corresponde a v4.6.2.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MENOR-1 | MENOR | aceptado (seguimiento en TASK-049: commit del merge frente a head.sha, ya en la divergencia 8) | xtask/src/main.rs:415 y crates/tc2/src/verificacion/informe.rs:178 | plan |
| MENOR-2 | MENOR | aceptado (seguimiento: alinear tests null y reglas descriptivas entre esquema y lector) | crates/tc2/src/verificacion/informe.rs:69 | implementacion |

### MENOR-1 — El `commit` del informe puede no ser el del check
- Donde: xtask/src/main.rs:415-418 (`git rev-parse HEAD`) y crates/tc2/src/verificacion/informe.rs:178 (la regla habla del check «en el head del PR»).
- Que pasa: el publicador firma el check sobre `github.event.pull_request.head.sha`, pero en `pull_request` `actions/checkout` deja el commit de merge (`refs/pull/N/merge`), así que `HEAD` en el runner es otro SHA. El informe dice `commit X` y el check está en `head.sha`: no casan. La guía lo describe como «commit verificado (`git rev-parse HEAD`)».
- Reproduccion: no verificado (no hay ejecución en GitHub; es TASK-049, divergencia 8 ya anotada). Caso a ejecutar: abrir un PR, bajar el artefacto `informe-verificacion-completa` y comparar su `commit` con `head.sha` del PR; esperado, distintos.
- Impacto: el revisor de F6-3 no puede comprobar que el informe es del mismo commit que el check; con un informe obsoleto o de otro commit solo cuenta el check, que es lo que la regla ya dice.
- Sugerencia: en TASK-049, no exigir igualdad de SHA entre informe y check, o que el workflow pase el head real (p. ej. `TC2_COMMIT`); documentarlo en la guía junto a la divergencia 8.

### MENOR-2 — Lector más laxo que el esquema (y el esquema promete más de lo que valida)
- Donde: crates/tc2/src/verificacion/informe.rs:69 (`#[serde(default)] tests: Option<Tests>`).
- Que pasa: `"tests": null` se lee como ausente, pero el esquema (`type: object`) lo rechaza. Además, «tras el primero en rojo no hay más pasos» y «`verde` exige todos los pasos en verde» están en las descripciones del esquema, no son validación de esquema, y el lector solo hace cumplir la segunda; la primera no la aplica nadie. Reproducción por lectura de serde, no ejecutada.
- Impacto: nulo hoy (el único productor, xtask, nunca emite `tests` ni pasos tras un rojo); solo importa si un tercer productor se guía por el esquema.
- Sugerencia: no corregir ahora; aceptar con este motivo, o bien un caso en `lo_que_el_contrato_no_admite_es_error` si se quiere fijar el comportamiento.

## Criterios de aceptación

- Criterio 1 («lo cita y no repite tests»): la mitad «lo cita» está cubierta (`seccion_revisor` con artefacto, perfil, pasos, regla de no relanzar y subordinación al check; prueba de contenido). «No repite tests» no es demostrable hasta el revisor de F6-3: divergencia 9 de la guía y plan lo declaran, queda para TASK-049. Aceptable como divergencia explícita.
- Criterio 2 (cumple F5-4 y lo que cita): artefacto por perfil y `informe=` en el resumen en GitHub, `artifacts: when: always` en GitLab, contrato versionado con lector y productor sincronizados por fixture; E2E real pendiente y anotado (divergencia 8).

## Seguridad del workflow

Sin hallazgos: `upload-artifact` fijada por SHA, solo en `ejecutar` (`permissions: contents: read`, sin secretos), ruta fija bajo `runner.temp`, `if-no-files-found: ignore`; el publicador no abre el artefacto, solo concatena nombres derivados de `PERFILES`, que `buscar` ya restringe a los cuatro perfiles conocidos. El lint rechaza la acción en clase privilegio y sin SHA (probado). La divergencia 7 (segunda acción en la lista blanca de F1-8) está documentada.

## Lecturas fuera del contexto permitido

- xtask/src/main.rs y xtask/src/workflows.rs: el productor y el lint son el núcleo del contrato esquema-lector-productor.
- crates/tc2/src/verificacion/informe.rs: el lector.
- /c/Users/nullcad2025/.claude/plugins/cache/taskcode-marketplace/taskcode-plugin/0.16.2/skills/task-workflow/revision.md: plantilla del informe.
