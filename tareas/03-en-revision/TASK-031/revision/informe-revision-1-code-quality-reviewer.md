# Informe de revision — TASK-031 (ronda 1)

- Commit revisado: 8a54049c717b71525d0a180c8e00676686271db0
- Revisor: code-quality-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-031 aprobada | aprobada-con-correcciones | cambios-solicitados)

Propuesta del revisor: **aprobada** (sin CRITICO ni IMPORTANTE; 2 MENOR nuevos y los declarados por la tarea, confirmados).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/roadmap/mod.rs:262 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/roadmap/mod.rs:250 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/roadmap/zona.rs:20 | implementacion |
| MEN-4 | MENOR | abierto | crates/tc2/src/roadmap/mod.rs:296 | implementacion |
| MEN-5 | MENOR | abierto | crates/tc2/src/roadmap/mod.rs:71 | implementacion |
| MEN-6 | MENOR | abierto | crates/tc2/src/roadmap/mod.rs:62 | implementacion |
| MEN-7 | MENOR | abierto | crates/tc2/src/roadmap/mod.rs:262 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

MEN-1 y MEN-3 son nuevos (reproducidos por mi). MEN-2 y MEN-4 a MEN-7 son los MENOR que la
tarea declara sin corregir (reproducidos o confirmados por mi); los registro para que queden en
el `## Resultado`, tambien los que no se corrigen.

## Metodo y puerta

- Clon en `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-031-r1-calidad`,
  rama `feature/task-031-f3-4-roadmap-la-vista-para-todos`, HEAD 9401218 (8a54049 mas la peticion).
  Compilado con `cargo` en el propio clon (target propio).
- Tests por criterio ejecutados por mi: `--lib roadmap` (13 ok), `--test roadmap` (8 ok),
  `--test alta` (12 ok), `--test estado_concurrencia` (5 ok), `--test siguiente` (6 ok).
- Puerta completa del workspace (fmt, clippy `-D warnings`, `cargo test --workspace --locked`):
  ver la linea «Puerta completa» al final.

## Criterio a criterio (existe, prueba lo que dice, pasa, y mutacion)

- **C1** `tests/alta.rs::dos_clones_crean_a_la_vez_con_ids_distintos` y
  `tests/estado_concurrencia.rs::dos_clones_dan_de_alta_a_la_vez_y_el_roadmap_queda_al_dia`:
  existen, pasan, prueban lo pedido (dos clones a la vez, todos `ok`, el ROADMAP tiene todas las
  tareas, `rev-list --merges` vacio, `comparar_en` al dia). Mutacion: `lib.rs:313`
  `Some(&generar)` -> `None` (el alta deja de regenerar): `dos_clones_crean_a_la_vez...` se pone
  ROJO. Observacion: el test del binario usa el `rev-list --merges` del bare y `comparar_en` (la
  funcion), no la orden `tc2 roadmap --comparar` como pide la letra del criterio; el efecto es el
  mismo y la orden se ejercita con binario en `tests/roadmap.rs`. No lo cuento como hallazgo.
- **C2** `roadmap/zona.rs::extrae_byte_a_byte_con_lf_y_crlf` y
  `tests/roadmap.rs::zona_manual_editada_sobrevive_a_new_import_y_escribir`: existen, pasan, y
  comprueban la zona con CRLF, NBSP, tabulador, espacios finales y `|` tras `--escribir`, `new`,
  `import` y otro `--escribir`, ademas de `status` limpio y la carpeta igual a lo publicado.
  Mutacion: `generar.rs` normaliza CRLF de la zona a LF -> ROJOS `salida_de_referencia` y
  `zona_manual_editada_sobrevive...`. (El test unitario de `zona.rs` solo cubre `extraer`, no el
  generador: la red real esta en los otros dos.)
- **C3** `tests/roadmap.rs::solo_cambia_la_rama_de_estado`: existe, pasa, compara `for-each-ref`
  del remoto y del clon, `status --porcelain` y el arbol de `HEAD` antes y despues de cada orden,
  y exige que `roadmap` y `--comparar` no muevan ninguna ref. Mutacion: `comparar_en` crea una
  rama `zz-mutante` -> ROJO («roadmap --comparar movio ["clon:refs/heads/zz-mutante"]»).
- **C4** `tests/roadmap.rs::toolsaweb_listas_y_en_marcha_como_next_y_su_roadmap` y
  `roadmap_y_next_coinciden_y_escribir_es_idempotente`: existen, pasan; ToolsaWeb (404 tareas,
  54 listas) y repo de prueba contra `tc2 next`. Mutacion: «Listas para empezar» en orden inverso
  -> ROJOS los dos.
- **C5** `roadmap::tests::salida_de_referencia` (fixture `referencia.md`) y
  `roadmap_y_next_coinciden_y_escribir_es_idempotente`: existen, pasan; la referencia cubre
  con quien, revision, hito con dependencias y Mermaid, terminadas y descartadas, y el segundo
  `--escribir` da `sin-cambios`. No mutado (pedido solo C1 a C4); cubierto de forma indirecta por
  las mutaciones de C2 y C4, que tumban `salida_de_referencia`.
- **C6** `tests/roadmap.rs::toolsaweb_cabe` y `mermaid::tests::grafo_demasiado_grande_es_una_nota`:
  existen y pasan (90 548 bytes, 14 hitos abiertos, un bloque por hito, cada uno bajo los
  limites; el test falla si algun bloque se convirtiera en nota porque exige un bloque por hito
  abierto). Margen sobre los 100 KB: 10 %, y el test no lleva tareas asignadas (cada `@login` y
  cada titulo real suma bytes); es el margen del criterio, no un fallo.

## Prueba a mano (repo de prueba con remoto bare, binario real)

Repo en `...\pruebas-claude\taskcodetwo\task-031-r1-manual` (borrado al terminar).

- `tc2 roadmap` sin rama de estado: `rechazada motivo=sin-rama-de-estado` y exit 3; con estado,
  imprime el ROADMAP sin tocar nada (`tareas=sin-montar` tras dos `new`).
- `--comparar`: `ok` al dia; tras un commit ajeno en la rama de estado, `rechazada
  motivo=desfasado` con exit 3, mensaje en es y en en («the published ROADMAP.md is out of date:
  run tc2 roadmap --escribir»), tambien con `--json`.
- `--escribir`: sin cambios da `sin-cambios`; con la zona editada en `tareas/ROADMAP.md` (CRLF,
  `ñ`, `—`) publica y deja `tareas=al-dia`, con 6 a 7 CR intactos en lo publicado y en la
  carpeta; repetido da `sin-cambios`. Tambien con otro clon que publica entre medias (el commit
  ajeno se integra, la zona editada se publica) y con `core.autocrlf=true` (la zona CRLF
  sobrevive byte a byte, `status` limpio, `--comparar` ok tras un `new`).
- `new` y `import` con la zona editada y sin publicar: terminan `ok` con `tareas=con-cambios`, no
  pisan la edicion, y un `--escribir` posterior la publica.
- Sin red: `roadmap denegada motivo=remoto-no-existe` (exit 8), nada publicado; recuperado con red.
- Idioma: uso y diagnosticos en es y en (`usage: roadmap [--comparar | --escribir] [--json]`). El
  ROADMAP.md es siempre en español (es un fichero compartido del equipo, no una salida de la CLI):
  lo veo coherente, solo lo anoto.
- Nota del entorno: `sed -i` de Git Bash en Windows quita los CR; no es del codigo (mis primeras
  pruebas con CRLF perdian la zona por eso; rehechas con `printf` y `perl`, correctas).

Valoracion del ROADMAP generado para una persona: la estructura sirve. Arriba la zona manual, luego
`Estado del trabajo` con en marcha y listas (las mismas cadenas que `next`) y los avisos solo si
existen, tablas `En curso` y `En revision` con `Con quien` (`@login`, o el texto heredado), un
apartado por hito abierto con paso, dependencias y estado legible («espera a TASK-002»), un
Mermaid por hito con colores por estado (nodos de borde para dependencias de otro hito) y
terminadas y descartadas plegadas en `<details>` con recuento. Los escapados funcionan (`|` en tablas,
`<` y comillas en Mermaid). Dos peros menores: en un repo recien creado aparece «Hitos sin titulo:
H0» porque la zona por defecto no declara H0 (MEN-3), y «En marcha» del encabezado no dice con
quien (esta en la tabla de debajo). El pintado real de Mermaid en GitHub/GitLab no lo he
verificado: es la D7 pendiente de Carlos, la dejo NO VERIFICADA.

## Reproduccion de los hallazgos nuevos

**MEN-1 — una edicion fuera de la zona manual no cambia nada publicado y se queda preparada
(`M `) para siempre, sin aviso** (`crates/tc2/src/roadmap/mod.rs:262`, `editado_en`/`escribir_en`).
Reproduccion: `tc2 roadmap --escribir` al dia; en `tareas/ROADMAP.md` cambiar solo
`## En curso` por `## En curso EDIT` (perl, que no toca los CR); `tc2 roadmap --escribir`.
Resultado: `roadmap sin-cambios`, exit 0, y `git -C tareas status --short` da `M  ROADMAP.md` con
la edicion intacta; `--comparar` dice `ok`. Si la edicion se hace a la vez que una de la zona,
se pierde en silencio al sobrescribirse la carpeta con lo publicado (el marcador del fichero dice
«no editar fuera de la zona manual», asi que es un uso fuera de contrato). Efecto: ninguna perdida
de trabajo en el caso normal; la persona queda con una carpeta sucia sin explicacion. Sugerencia: en
`sin-cambios` con edicion fuera de zona, o avisar, o restaurar la carpeta a lo publicado (como ya
hace tras un `ok`).

**MEN-2 — `--escribir` falla y deja `tareas/ROADMAP.md` preparado** (`mod.rs:250`). Reproduccion:
editar la zona y poner el `origin` a una ruta inexistente: `roadmap denegada
motivo=remoto-no-existe` (exit 8) y `M  ROADMAP.md`. No se pierde nada y el siguiente intento con
red lo publica (probado); es el MENOR que declara la tarea, lo confirmo reproducido.

**MEN-3 — la zona por defecto no declara ningun hito y una instalacion nueva avisa «Hitos sin
titulo: H0»** (`zona.rs:20`). Reproduccion: repo nuevo, dos `new`, `tc2 roadmap`: la linea
`- **Hitos sin título:** H0` aparece de entrada. Es ruido en el primer ROADMAP que ve la persona.
No afecta a la correccion. La zona por defecto «es la de la 0.16 con tc2», como declara la tarea; lo
unico que sugiero es no avisar de H0 cuando la zona sigue siendo la de por defecto, o dar a la zona
un `### H0` de ejemplo.

## Valoracion de las desviaciones declaradas

- **`--comparar` sale con codigo 3** (`rechazada`): aceptada. Es lo que usa el contrato para
  «rechazada», sirve a la CI, y los tests y la ayuda lo dicen. `ok` sigue en 0.
- **`--escribir` hace `git add` de la zona y adelanta la carpeta**: aceptada. Sin esto el
  fast-forward de `tareas/` falla con el fichero cambiado (reproducido: `new`/`import` con la zona
  editada dejan `tareas=con-cambios` sin perder la edicion). El `git add` toca el indice del worktree
  de estado, no ninguna ref (C3 lo verifica); el coste es MEN-1/MEN-2.
- **Nodos de borde sin titulo**: aceptada. Salen con titulo en su propio hito o en las plegadas, y
  es lo que hace que ToolsaWeb quepa en 100 KB (C6); el ID basta para navegar.
- **Zona por defecto con «tc2» en lugar de «taskctl»**: aceptada (MEN-3 es solo el aviso de H0).
- **Cambio en `.gitattributes`** (`crates/tc2/tests/fixtures/roadmap/** -text`): aceptado y
  necesario. La referencia mezcla CRLF (zona) y LF (lo generado) y se compara byte a byte; sin
  `-text` un checkout con `autocrlf` la cambiaria. El patron se limita a la carpeta del fixture.
  Comprobado: `salida_de_referencia` pasa con los bytes del clon en Windows.
- **Los MENOR declarados sin corregir**: MEN-2 y MEN-4 a MEN-7, abajo; ninguno es IMPORTANTE segun el
  criterio de proporcionalidad (ninguno pierde trabajo en un caso realista ni cambia el resultado
  normal).

### MENOR declarados por la tarea (confirmados, no corregidos)

- **MEN-4** (`mod.rs:296`, `adelantar_editada`): tras publicar, la carpeta se sobrescribe con lo
  publicado; si la persona edita el fichero en ese instante, esa ultima edicion se pierde. Ventana
  de milisegundos entre el commit y el `fs::write`; carrera real pero no realista.
- **MEN-5** (`mod.rs:71`, `de_estado`): dos clones con distinto `origin/develop` convierten
  distinto un heredado y `--comparar` ve desfasado (la lista de personas sale de la rama base).
  Convergen con el siguiente `--escribir`.
- **MEN-6** (`mod.rs:62`, `personas`): una lista de personas ilegible se trata como vacia sin
  avisar (`unwrap_or_default`); el heredado se muestra tal cual. Un aviso costaria poco.
- **MEN-7** (`mod.rs:262`): `core.autocrlf=true` con finales mixtos, que la tarea marca «no
  verificado». Lo verifique para una zona CRLF con el resto LF (el caso del test): sobrevive byte a
  byte y sin `status` sucio (ver «Prueba a mano»). Un fichero con CRLF puro o `` sueltos queda
  sin probar; sigue como MENOR.

(MEN-2, el fallo de `--escribir` que deja la zona preparada, es el quinto declarado y esta
reproducido arriba.)

## Puerta completa

Sobre el clon en 9401218 (el codigo es el de 8a54049; solo suma la peticion): `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` limpio; `cargo test --workspace --locked`: 321 pasados, 0 fallidos, 6 ignorados (suma de las 30 lineas «test result»). La tarea decia 299 pasados y 5 ignorados sobre 3a1dd0f; no se por que la cifra difiere (mi suma incluye todos los binarios de test del workspace; no hay fallos en ninguna), lo anoto sin darle mas peso.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/roadmap/{mod,generar,mermaid,zona}.rs`, `crates/tc2/tests/roadmap.rs`,
  `crates/tc2/tests/alta.rs`, `crates/tc2/tests/estado_concurrencia.rs`,
  `crates/tc2/src/i18n/catalogo/roadmap.rs`, `crates/tc2/tests/comun/mod.rs`: hacen falta para
  comprobar cada criterio, mutar y reproducir (la skill pide reproducir, no leer solo el diff).
- `tareas/03-en-revision/TASK-031/tarea.md` y `planificacion/plan-final.md`: contexto permitido.
