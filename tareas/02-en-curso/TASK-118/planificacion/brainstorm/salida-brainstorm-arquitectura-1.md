# Brainstorm — TASK-118, rol arquitectura (ronda 1)

- Rol: `brainstorm-arquitectura`
- Agente: taskcode-plugin:brainstorm-arquitectura

## Enfoque propuesto, con rutas y nombres concretos

Anadir una operacion de escritura al puerto `Plataforma` (`crates/tc2/src/plataforma/mod.rs`), con el modelo de puertas que ya existe en `puertas::modelo()` y, por check, su origen (que hoy no existe en el codigo). En `init`, la opcion `--aplicar-puertas`: las lineas `ruleset` que hoy salen `persona` pasan a `crea` con un efecto nuevo que aplica `aplicar.rs`, con el patron de «plan como dato» de `init/mod.rs`.

Orden de construccion:
1. `Origenes` y `origenes(cfg)` en `puertas.rs`, la clave de config (D1) y `ruleset_github` con origenes. Todo puro.
2. `aplicar_puerta` en el trait, con implementacion por defecto, y el adaptador de GitHub con `POST /rulesets`.
3. El adaptador de GitLab: ramas y tags protegidos, ajustes de merge y la regla de aprobacion en Premium.
4. `init`: la opcion, la precondicion (D3), el `Efecto::AplicarPuerta` y la `Conexion` unica (D5). Se carga `personas.yml` en el adaptador.
5. Notas: `NOTA_SIN_ORIGEN`, el comentario del trait (l. 285, «son de F3-1»), la cabecera de `init/mod.rs` y la adenda al ADR (D4).

## Que se extiende y que se crea

- `trait Plataforma` (`plataforma/mod.rs:286`): se extiende con implementacion por defecto `rechazada motivo=no-soportada`, como `leer_plan`. **D2 (a) recomendada**: una llamada por puerta, `fn aplicar_puerta(&self, p: &Puerta, o: &Origenes) -> Informe`, que encaja con un `Efecto` por `Accion` y con el corte en el primer fallo de `init()`. (b) Por lote, `-> Vec<Informe>`.
- `puertas.rs`: `struct Origenes { checks: Vec<(&str, u64)> }` y `fn origenes(cfg) -> Result<Origenes, Vec<&str>>`, pura, que devuelve los checks sin origen como motivo de rechazo. `ruleset_github(p, Option<&Origenes>)` pone el `integration_id` por check y quita `_nota` cuando recibe origenes: un solo traductor para la propuesta y para la escritura. El mismo `Origenes` lo usara `estado_puertas` (F6-2).
- `github.rs`: `Endpoint::CrearRuleset` (`POST {r}/rulesets`) y `exigir_persona`. El 403 de plan da `rechazada motivo=plan` (F1-13 D1). `RulesetApi` privado.
- `gitlab.rs`: traduccion privada en el adaptador. Una puerta de GitLab son varias llamadas (rama o tag protegido, en Premium una regla de aprobacion `taskcodetwo-*`, y el metodo de merge del proyecto en las puertas con `solo_pr`). Endpoints POST/PUT en la lista cerrada.
- `config` (`config/mod.rs` `PlataformaConfig`, `config/claves.rs`): **D1 (a) recomendada**, clave `plataforma.revisor_id` (u64: `app_id` de la App en GitHub, `user.id` en GitLab), con 15368 (Actions, F1-5 PoC A) como constante en codigo para `verificacion` y `carril`; con 0 o sin clave, rechazo. (b) Guardar un slug y leer el id al aplicar (`GET /apps/{slug}`).
- `init/mod.rs`, `init/inventario.rs`, `init/aplicar.rs`: `Opciones.aplicar_puertas` y `Efecto::AplicarPuerta { nombre: &'static str }` (dato, conserva Clone y Eq). **D5 (a) recomendada**: la `Conexion` se crea una vez en `init()` y se pasa al inventario y a `aplicar`. (b) `aplicar.rs` reconstruye el adaptador desde `c.credenciales` (repite `detectar` y `GET /user`).

## Limites que cruza

- Contrato publico del trait `Plataforma` (nueva operacion de escritura) y su lista cerrada de endpoints.
- Esquema de configuracion: clave nueva `plataforma.revisor_id` (adelanta un contrato de F6-1).
- Contrato de linea de `init` (accion `crea` para `ruleset`, motivo `puertas-sin-origen`).
- ADR F1-5 (tabla de la l. 368, «distinto → los actualiza»): divergencia por D4.

Alternativas descartadas:
- **D3 (b)**: exigir los checks ya publicados en la cabeza de `develop` (`checks_publicados`): en un repo nuevo no hay ninguno antes del primer PR (huevo y gallina) y anade red a una precondicion. **D3 (a) recomendada**: precondicion local y determinista: el workflow existe (`RUTA_WORKFLOW`) y `origenes(cfg)` da `Ok`; si no, `init rechazada motivo=puertas-sin-origen faltan=...`.
- **D4 (b)**: `PUT /rulesets/{id}` de las que difieren (F1-5): obliga a guardar el id y comparar contra el modelo, e `init` «nunca pisa». **D4 (a) recomendada**: crear solo las que faltan; las que existen no se tocan; divergencia en adenda; la actualizacion queda para `doctor --arreglar` u otra tarea.

Desacuerdos previstos: con dominio sobre D1 (adelanta F6-1; se define aqui porque sin fuente no se cumple el criterio 2); con dominio y riesgos sobre D4; con riesgos sobre D2 (a) en GitLab (el metodo de merge se reaplica de forma idempotente en cada puerta con `solo_pr`).

Suposiciones no verificadas:
- `init` construye el adaptador con `Personas::default()` (`inventario.rs:594`, `:601`): `exigir_persona` siempre rechazaria. Confirmar que `--aplicar-puertas` exige `personas.yml` ya existente (F1-6 H6).
- En GitLab, `ReglasLeidas.rulesets` solo tiene reglas de aprobacion (`mod.rs:132`); en Free `existe` por nombre nunca acierta: el adaptador decide `sin-cambios` leyendo las protecciones.
- No hay ruta de CI de GitLab en `huella.rs`; para D3 en GitLab, mirar la plantilla de F5-1.
- F5-1 y F6-1 no implementadas: la opcion se rechazara en la practica hasta F6-1.
- Que `POST /rulesets` acepte el JSON de la PoC C con `integration_id`: F1-5 no hizo escrituras (l. 267-270).

## La decision de diseño que mas te preocupa (UNA sola)

D1: de donde sale el `integration_id` real. Ninguna parte del codigo lo tiene hoy: `Config` no tiene clave y nadie construye `ModeloPuertas`.
