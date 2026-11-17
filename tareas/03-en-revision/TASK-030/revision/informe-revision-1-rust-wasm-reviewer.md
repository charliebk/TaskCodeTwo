# Informe de revision — TASK-030 (ronda 1)

- Commit revisado: fc1f1887ff81feae206dda283f77e319fdfb12de
- Revisor: rust-wasm-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-030 aprobada | aprobada-con-correcciones | cambios-solicitados)

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/grafo.rs:133 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/estado/tareas.rs:44 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/siguiente.rs:112 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — `Hitos::del_roadmap` es fragil ante marcas repetidas o desordenadas
`del_roadmap` hace `break` en la primera linea que empieza por `<!-- manual:fin`, aunque `dentro`
sea falso, y solo lee la primera zona. Reproducido con dos tests temporales (ya revertidos):
- `"<!-- manual:fin -->\n<!-- manual:inicio -->\n### H1 — Uno\n<!-- manual:fin -->"` da cero hitos.
- Dos zonas manuales seguidas: solo cuenta la primera (H2 no aparece).
Efecto: un ROADMAP con una marca de fin suelta antes de la zona deja a `next` sin orden de hitos
(cae al orden por numero y avisa `hitos-sin-titulo=`). No afecta al ROADMAP generado por la 0.16
(una zona, inicio antes de fin). Sin panic. Sugerencia: el `break` solo si `dentro`, y decidir si
varias zonas suman.

### MEN-2 — Un `tarea.md` cuya lectura devuelve `None` desaparece sin aviso en `next`
`estado::tareas` hace `filter_map(|(r, t)| t.map(...))`: si `leer_varios` da `None` para una ruta
que si esta en `rutas()` (blob ilegible o desaparecido), esa tarea no sale ni como nodo ni en
`ilegibles=`, y no bloquea a sus dependientes. Contradice la promesa de C4 («no se saltan en
silencio»). Solo se da con un fallo de `cat-file` por blob concreto, no con el flujo normal; en el
alta ya era asi. Ademas, `zip` trunca en silencio si `leer_varios` devolviera menos elementos que
rutas (no ocurre con `Vista` ni con el valor por defecto).

### MEN-3 — ID de una carpeta ilegible en minusculas no casa con las dependencias
`es_tarea_md` admite `task-013/` sin distinguir mayusculas (como el alta), y `calcular` usa el
nombre de la carpeta tal cual como ID del nodo ilegible (`task-013`). Una dependencia `TASK-013` no
encuentra ese nodo, se avisa como `dependencias-inexistentes=` y no bloquea, cuando deberia bloquear
por ilegible. Caso de borde (carpeta en minusculas Y ilegible).

## Alcance
- Ficheros revisados: crates/tc2/src/grafo.rs, siguiente.rs, estado/tareas.rs, estado/mod.rs,
  alta.rs, lib.rs, i18n/catalogo/{cli,mod,siguiente}.rs, tests/siguiente.rs (y fixtures leidas).
- Contexto adicional pedido: estado/lectura.rs (contrato de `leer_varios` y `Vista::leer_varios`) y
  la funcion `numero` de alta.rs, para comparar el filtro viejo con el nuevo.
- Limite JS-WASM: sin cambios de firma exportada (no hay `#[wasm_bindgen]` en el diff).

## Reproduccion
- Clon: C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-030-r1-rust, rama
  feature/task-030-f3-3-next-lo-listo-para-empezar-por-depe (HEAD 0fdce38, codigo = fc1f188).
- Build y suite: `cargo test -p tc2 --locked`: todo en verde (lib 136 passed, 1 ignored por
  necesitar clon de ToolsaWeb; tests de integracion todos ok).
- Formato y lints: `cargo clippy --all-targets --locked -- -D warnings`: sin avisos, exit 0.
- Casos construidos: dos tests temporales de zona manual (MEN-1), revertidos.
- Mutacion (sobre grafo.rs, cada una revertida con git checkout):
  - Quitar el numero de la clave de desempate: rojo (`ids_con_sufijo_y_de_cuatro_cifras`).
  - Quitar el sufijo de la clave: VERDE, pero es un mutante equivalente: el ID completo es el ultimo
    componente de la clave y ordena igual `012 < 012a < 012b`. No es hallazgo.
  - Hito por su numero en vez de por posicion del titulo: rojo (`hito_sin_titulo_va_detras_y_avisa`,
    `toolsaweb_listas_y_en_marcha_como_su_roadmap`).
  - Descartada deja de bloquear en `listas`: rojo (`dependencia_descartada_bloquea_y_avisa`).
  - Ilegible deja de bloquear: rojo (`dependencia_ilegible_bloquea`).
  - Hito sin titulo sin el desplazamiento de 1_000_000: rojo (`hito_sin_titulo_va_detras_y_avisa`).
- Medidas: no aplican (sin limite WASM; el Kahn es O((n+e) log n) con `BinaryHeap`, la fixture de
  404 tareas corre en milisegundos).

## Revisado sin hallazgos
- Kahn de `grafo.rs`: contadores `faltan` por dependencias vivas deduplicadas; dependencias
  inexistentes no bloquean, descartadas e ilegibles no cuentan en el Kahn pero bloquean en `listas`;
  autodependencia y ciclos terminan en `atascadas` (probado en `ciclo_deja_atascadas`); duplicados
  avisan y vale el primero; `saturating_sub` evita desbordes; el desempate (hito por posicion de
  titulo, numero como `u64` y no como texto, ID) es total y determinista. Sin quadratico oculto.
- `hito_de_linea`: exige `### H<cifras> — <titulo>`; cifras que desbordan `u32` devuelven `None`,
  sin panic; `\r` tratado.
- `estado::tareas` y `es_tarea_md`: `get(..)` evita cortar en limites de caracter (probado con `TÁSK`);
  `alta::tareas` filtra primero con su `es_tarea_md` (mas estricto: sin sufijo), asi que su
  resultado, orden y descarte de formato 1/ilegibles no cambia.
- Produccion sin `unwrap`, `expect`, `panic!`, `unsafe` ni indexado por `[]` en grafo.rs,
  siguiente.rs y estado/tareas.rs (solo en tests, con `allow` explicito en tests/siguiente.rs).
- `numero_y_sufijo`: un ID con numero desbordado cae a `u64::MAX`, sin panic.
