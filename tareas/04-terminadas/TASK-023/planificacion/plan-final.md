# Plan — TASK-023: F2-7 Configuracion minima con esquema validado y valores por defecto

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 240 min

Señales: unos 6 ficheros de código nuevos o tocados (`yaml`, `carril`, `config/` con 3 ficheros, `lib.rs`), un esquema publicado, 2 ficheros de tests y 4 criterios.

**Qué es.** Un módulo de biblioteca `config` que lee `.taskcode/config.yml` y comprueba que cumple un esquema cerrado. Devuelve una de dos cosas:
- una `Config` tipada con los valores por defecto ya aplicados;
- o la **lista completa** de errores, cada uno con la ruta de su clave (`politica_carriles.fix.aprobaciones: clave no admitida`).

El esquema JSON publicado se **genera a partir de la misma tabla de claves** que usa el validador. Un test falla si el fichero publicado no coincide con lo que genera el código.

No hay subcomando nuevo. Es el mismo precedente que F2-6 y F2-9: solo biblioteca. Quien la consume es `doctor` (F3-8), `init` (F3-1) o `carril` (F6).

**Lo que ya fijan los ADR y no se reabre:**
- **Fichero y formato:** `.taskcode/config.yml`, con `---` como primera línea y `formato: 2`, los dos obligatorios (F1-12 D4a, punto 6).
- **Origen:** se lee de `origin/develop:.taskcode/config.yml`, nunca del worktree de la tarea (F1-10 dec. 3; F1-11 dec. 1).
- **Claves o valores desconocidos:** fallan cerrado; son un error, no un aviso (F1-10 dec. 2; F1-11 dec. 1; F1-15 S15). En `tarea.md` la regla es la contraria (F1-9 D4a).
- **Tope:** el límite de 12 cuenta las claves de primer nivel (F1-11, PoC B y dec. 1).
- **Personas:** la lista de personas es otro fichero de `.taskcode/` (F1-6 D6a) y no cuenta para el tope.

**Inventario de claves.** Quedan 11 de 12, con la agrupación de D1.

| # | Clave | Origen | Tipo | Por defecto | Validación (falla cerrado) |
|---|---|---|---|---|---|
| 1 | `formato` | F1-12 D4a, punto 6 | entero | obligatoria, sin defecto | vale `2`; además, la primera línea del fichero es `---` |
| 2 | `politica_carriles` | F1-10 dec. 2 | mapa de carril a mapa | `{}` (el suelo del binario) | ver la nota 1 |
| 3 | `rutas_carril` | F1-10 dec. 2 | mapa de {`fix`, `riesgo-alto`} a globs | `{}` | lista no vacía; sin `!`, sin `/` inicial y sin `..` |
| 4 | `verificacion` | F1-11 dec. 1 | mapa | ausente (D6) | la tabla `esquema:inicio/fin` de F1-11 |
| 5 | `derivados` | F1-11 dec. 1 | lista de mapas | `[]` | tabla de F1-11; `fases` dentro de las fases de `tc2` (D7); `timeout` de 1 a 600, 60 si falta |
| 6 | `confianza_incluye` | F1-11 dec. 1; F1-8 dec. 5 | globs | `[]` | sin negación, rutas absolutas ni `..`; nunca `**` ni `*` |
| 7 | `plataforma` | F1-13 dec. 7 | mapa {`proveedor`, `api`} | ausente: se detecta por el host | `proveedor` es `github` o `gitlab`; `api` pasa por `plataforma::http` |
| 8 | `recursos` | F1-14 D6a; F1-15 dec. 6 | lista de mapas | `[]` | `id`, `tipo` (`skill`, `agente` o `ejecutable`), `rol` (opcional) y `dominios` (opcional, lista cerrada) (D8) |
| 9 | `seleccion` | F1-15 dec. 3 y S15 | mapa | `modelos_permitidos: [haiku, sonnet, opus]`; `dominios_extra: {}` | ver la nota 2 |
| 10 | `flujo` | 0.16 `modo_flujo` y `tope_rondas`; RF-21, RF-33; F1-16 D6a | mapa {`modo`, `tope_rondas`} | `modo: automatico`; `tope_rondas: 2` | `modo` es `manual`, `semiautomatico` o `automatico`; `tope_rondas` entero de 1 a 10 |
| 11 | `lote` | F1-16, consecuencias | mapa {`presupuesto_tokens`, `presupuesto_horas`, `tope_tokens_tarea`} | ausente, sin cifra inventada (D5) | enteros > 0 |

Notas a la tabla:
1. **`politica_carriles`.** Admite solo los carriles `errata`, `fix`, `funcionalidad` y `riesgo-alto`; se rechaza cualquier otro, también `release` y `hotfix`. Subclaves admitidas:
   - `roles_diseno`: al menos el suelo y como máximo 5;
   - `plan_aprobado`: solo `true`;
   - `paradas_extra`: dentro de {`start`, `review`};
   - `revisoras_extra`;
   - `lote`: solo `false`;
   - `perfil`: al menos el suelo.
2. **`seleccion`.** `modelos_permitidos` es un subconjunto no vacío de la escala. `dominios_extra` admite solo `puertas-seguridad` y `rendimiento`, cada uno con `etiquetas`, `palabras` y `rutas`.

**Piezas, en este orden:**
1. **Costura YAML estricta compartida (se extiende).**
   - `opciones()` sale de `tarea::yaml` (privado, `crates/tc2/src/tarea/yaml.rs:154`) a un módulo `crate::yaml` (`pub(crate)`), y `tarea::yaml` pasa a usarlo.
   - Mantiene lo que ya hace: rechaza claves duplicadas, fusiones, anclas y alias.
   - No cambia el comportamiento: los tests de `tarea` siguen en verde.
2. **`carril` mínimo (se crea),** en `crates/tc2/src/carril.rs`.
   - Contiene el enum de los cuatro carriles y, de la tabla de suelo de F1-10 dec. 1, solo las columnas que valida la configuración: `roles_min` y el perfil.
   - La dependencia va de `config` a `carril`, nunca al revés.
3. **`config/` (se crea),** con tres ficheros:
   - `mod.rs`: la API pública.
   - `claves.rs`: la tabla declarativa; cada clave con su ruta, origen, tipo, valor por defecto y rango.
   - `validar.rs`: decodifica a un árbol genérico (`serde_json::Value`, con las opciones estrictas) y lo recorre con la ruta de cada nodo, juntando los errores. Las reglas que cruzan campos se escriben a mano.
4. **Esquema publicado (se crea).**
   - `config::esquema_json()` genera JSON Schema 2020-12 a partir de la tabla, con `additionalProperties: false` en cada objeto y el ADR de origen en `description`.
   - El fichero es `crates/tc2/esquemas/config.schema.json`.
   - Un test lo compara con lo generado; con `TC2_ESCRIBIR_ESQUEMA=1`, lo reescribe.
5. **`config::de_base` (se crea).** Es una capa fina sobre `git::estado::leer_blob` (`crates/tc2/src/git/estado.rs:104`) para leer `origin/develop:.taskcode/config.yml`.

Firma propuesta:
```rust
pub fn leer(texto: &str) -> Result<Config, Invalida>          // pura, sin E/S
pub fn de_base(git: &Git, dir: &Path) -> Result<Config, Informe>
impl Config { pub fn suelo() -> Self }                         // formato 2 y todo por defecto = sin endurecer
pub struct Invalida { pub motivo: Motivo, pub errores: Vec<ErrorConfig> }  // ErrorConfig { ruta, mensaje }
impl Invalida { pub fn informe(&self) -> Informe }  pub fn esquema_json() -> String
```

**Contrato de salida.** Todo fallo de la configuración sale como `config rechazada`, con código 3, y no se reintenta.
- Línea: `config rechazada motivo=<m> errores=<n> claves=<ruta1>,<ruta2>…`.
- El diagnóstico va a stderr, con el primer error y su arreglo.

Motivos:
- `sin-config`: falta el fichero. Arreglo: `tc2 init`.
- `formato`: falta `---` o `formato: 2`, o tiene otro valor (un proyecto de la 0.16). Arreglo: `tc2 actualizar`.
- `yaml`: sintaxis, clave duplicada o alias.
- `config-invalida`: no cumple el esquema. Si la clave es una clave plana de la 0.16, el arreglo nombra `tc2 actualizar`.

Si falla la lectura en Git, sale tal cual el `Informe` de la capa git.

**Caso abierto de F1-10.** `de_base` distingue «no se pudo leer» de «inválida». Además se da `Config::suelo()`, que es la política sin endurecer. F2-7 entrega solo la forma; la regla la aplica F6 (D9).

**Coste.** Un `git show` y un recorrido lineal. Quien consume la configuración carga la `Config` una vez y pasa `&Config`; no hay `static`.

**Dependencias.** Ninguna nueva: ya están `serde`, `serde-saphyr` y `serde_json`.

## Troceo

La estimación pasa de 60 min. **No se trocea, por decisión de Carlos para el H2**: «tareas consecutivas en orden, no quebramos ninguna tarea». Las piezas 1 a 5 son una sola entrega y el esquema depende de la tabla del validador.

## Lo que el rol no cubrio

Contrastado con los criterios:
- **≤ 12 claves.** Cubierto: son 11. Sin revisión de dominio de los valores por defecto ni del esquema de `recursos`.
- **Esquema publicado.** Cubierto. No se ha comprobado que un editor lo cargue.
- **Tests.** El plan de pruebas no lo ha diseñado un rol de testing.
- **Cumple F2-7.** Cubierto lo que los ADR piden a F2-7.
- **Riesgos no mirados:**
  - globs patológicos y ficheros enormes;
  - BOM y `\r\n` en la primera línea `---`;
  - mayúsculas en las rutas de `escribe`;
  - una configuración válida que deja de serlo con un binario nuevo.
- **Reglas de negocio no verificadas:**
  - `paradas_extra`;
  - el valor numérico de «todos» en el suelo de `roles_diseno` para riesgo alto;
  - la forma de `lote`.

## Riesgos aceptados y que los contiene

- **Queda una sola clave libre (11 de 12).** Lo contiene el test del tope, que falla en cuanto se añade la número 13.
- **Se renombran claves sueltas de F1-15 y de la 0.16** para agruparlas. Lo contienen la tabla y el esquema, que dan el origen de cada una. La traducción es trabajo de `actualizar` (F1-12).
- **`carril` nace con solo dos columnas.** Lo contiene que está documentado como el sitio de la tabla de F1-10.
- **Este repo usa la configuración de la 0.16 (RR-14).** Leída con `config` saldría `motivo=formato`, y es lo esperado. Nada del repo la lee: los tests usan textos y repos temporales.
- **Sacar `opciones()` de `tarea::yaml`.** No cambia el comportamiento; lo contiene la suite de `tarea`, que sigue en verde.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **C1, ≤ 12 claves**
  1. `claves_de_primer_nivel_no_pasan_de_12_y_cada_una_cita_su_adr`.
- **C2, esquema publicado**
  1. `esquema_publicado_es_el_generado`.
  2. `esquema_cerrado_y_con_las_claves_de_la_tabla`: las `properties` coinciden con la tabla, `required` es solo `formato` y cada objeto lleva `additionalProperties: false`.
- **C3, validador y valores por defecto**
  1. `config_minima_da_los_valores_por_defecto`: `"---\nformato: 2\n"` da `Config::suelo()`.
  2. `configs_hostiles_rechazadas_con_la_ruta`: una tabla con los casos de las PoC de F1-10 y F1-11. Cada caso da su motivo y su ruta; uno con dos errores exige los dos.
  3. `traduccion_de_toolsaweb_de_f1_11_es_valida`.
- **C4, cumple F2-7 y los ADR**
  1. `tabla_de_f1_11_igual_que_el_esquema`.
  2. `de_base_lee_origin_develop_y_no_el_worktree` (integración, con un repo temporal).
  3. `sin_config_o_de_la_016_rechazada_con_el_contrato`.

## Lo que necesita decision de una persona

El checkpoint humano es obligatorio. Carlos delega en la opción recomendada («aceptamos todas las preguntas con la respuesta recomendada»), así que estas decisiones quedan tomadas:

- **D1, cómo caber en el tope.** Se agrupan `seleccion` y `flujo`: quedan 11 de 12.
- **D2, cómo se mantiene el esquema.** Se genera desde la tabla del código, con un test de sincronía. Se descartan escribirlo a mano y el crate `schemars`.
- **D3, subcomando o biblioteca.** Solo biblioteca. `doctor` (F3-8) dirá qué clave falla.
- **D4, dónde se publica el esquema.** En `crates/tc2/esquemas/config.schema.json`: es un contrato, no documentación, y queda al alcance de `include_str!`.
- **D5, presupuesto del lote.** `lote` no lleva cifras por defecto: no se inventan.
- **D6, `verificacion`.** Es opcional en el esquema; quien la necesita falla con el arreglo.
- **D7, fases de `derivados`.** Son `new`, `plan`, `approve`, `start`, `review`, `finish`, `descartar` y `pausa`; si falta, todas.
- **D8, `recursos`.** Esquema mínimo cerrado: `id`, `tipo`, `rol` y `dominios`.
- **D9, configuración inválida en la base.** F2-7 entrega `Invalida` y `Config::suelo()`; la regla la aplica F6.
- **D10, valores por defecto de `flujo`.** `modo: automatico` y `tope_rondas: 2`. Durante la implementación se comprueba cuál es el valor por defecto de `modo_flujo` en la 0.16.
