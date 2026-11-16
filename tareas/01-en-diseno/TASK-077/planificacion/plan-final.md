# Plan — TASK-077: F2-13 Identidad de la persona desde la plataforma

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 240 min

F2-13 no inventa una identidad nueva: une lo que ya existe (`Plataforma::identidad(Rol)` con `GET /user` en GitHub y GitLab y cache de proceso en `conexion.rs:195`; `Usuario {login, id, tipo}`; `Personas`, que `plataforma/mod.rs:64-67` dice que «la lee F2-13 del fichero de `.taskcode/`») y anade lo que falta: el lector y validador de `.taskcode/personas.yml`, un tipo `Asignado` para `asignado_a`, un modulo `identidad.rs` que resuelve quien ejecuta (persona o agente) y la orden `tc2 identidad`. `start` es de F3-6 y aun no existe: F2-13 entrega la pieza de biblioteca y F3-6 la conecta.

Definiciones:
- Se guarda el `id` numerico como clave de comparacion (F1-6 decision 3) y el `login` al lado como copia para mostrar; `personas.yml` es la autoridad que refresca un login renombrado.
- Origen: la API con el token por el puerto `Plataforma` (sin `gh`/`glab`, F1-14).
- Cache local en `<git-common-dir>/tc2/identidad.yml` (`{proveedor, id, login, verificado}`), no commiteada y compartida por los worktrees; solo la escribe `tc2 identidad` con TTY; la lee el agente sin TTY; TTL de 7 dias desde `verificado`.
- Sin plataforma no hay modo (RN-06 es «una persona con agentes», no «sin plataforma»).
- Sin red, `start` ya falla con `SinRed` (F1-2 D2b); la cache solo sirve para transiciones locales de una tarea que la persona ya lleva.
- Migracion: campo opcional `correos:` por persona en `personas.yml`; lo que no case queda `Heredado`, `doctor` avisa y F3-10 (`reasignar`) lo resuelve.

Piezas:
- `plataforma/personas.rs` (nuevo, texto → `Personas`, sin E/S): se mueve aqui `Personas` y gana `login`/`nombre`/`correos`.
- `tarea::modelo`: `asignado_a: Option<Asignado>`, `Asignado = Persona{id: u64, login: String} | Heredado(String)`, tipos planos sin depender de `plataforma`.
- `identidad.rs` (nuevo, primer nivel): `quien_ejecuta(&dyn Plataforma, &Personas, cache, tty) -> Paso<Quien>` y la cache local.
- `lib.rs` e `i18n/catalogo/identidad.rs`: orden `identidad [--json]` (precedente de `new`).
- `github.rs`, `gitlab.rs`, `conexion.rs`: no se tocan.
- Nota en ADR F1-9 (linea 464) con la forma decidida de `asignado_a`.

Orden de construccion:
1. `personas.rs` con validacion de unicidad (C3, C4).
2. `Asignado` en el modelo con ida y vuelta y lectura heredada (C5, C9); ajustar `alta.rs` y `tests/tarea.rs`.
3. `identidad.rs`: `Quien`, cache local y TTL (C2, C6, C7, C8).
4. Orden `tc2 identidad` y su catalogo (C1).
5. Conversion con `correos` (C9).

Alternativas descartadas: identidad con `gh api user` o `glab` (F1-14 eligio la API HTTP; segundo camino; no sirve con hosts propios de GitLab); solo el `id` en `asignado_a` (el roadmap tendria que leer la lista o la API por tarea).

## Lo que el rol no cubrio

- Sin roles de riesgos, testing ni dominio: suplantacion via cache, login renombrado y TTL no tienen revision independiente.
- Un repo que pase de GitHub a GitLab: los espacios de `id` no se comparan entre plataformas; queda sin cubrir.
- La integracion en `start` es de F3-6.
- Suposiciones sin verificar: un helper reutilizable del directorio comun de git (`estado/montaje.rs`, `cerrojo.rs`); que en GitLab `Usuario.login` es el `username` (`gitlab.rs:122-130`); que el lector 0.16 (`modelo.rs:533-687`) puede devolver `Heredado` sin romper `tests/tarea.rs`; que nadie mas consume `asignado_a` como `String`.

## Riesgos aceptados y que los contiene

- El agente podria editar la cache local y hacerse pasar por otra persona de la lista: se acepta porque `asignado_a` no aprueba nada; la aprobacion sigue siendo la firma de F1-6/F4-2.
- Cambio de forma de `asignado_a` (texto → objeto): F1-9 deja a F2-13 decidir el valor; se anota alli sin reabrir el ADR.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):
- `tests/identidad.rs` con mock en 127.0.0.1 (patron de `tests/plataforma.rs:521`): C1, C2, C7, C8, una vez con GitHub y otra con GitLab, contando peticiones.
- Unitarios en `personas.rs` (C3, C4) y en `identidad.rs` (C6).
- Ida y vuelta en `tests/tarea.rs` (C5, C9).

## Lo que necesita decision de una persona

Carlos acepta las recomendaciones (consigna del 2026-11-16):
- D1: `asignado_a` como objeto `{id, login}`.
- D2: cache local en el directorio comun de git con TTL de 7 dias.
- D3: el agente puede tomar sin TTY en nombre de la persona de la cache, con `decidido_por: agente` (RN-06).
- D4: no hay modo sin plataforma.
- D5: migracion con alias `correos:` en `personas.yml` y `Heredado` para lo que no case.
- D6: criterios sustituidos por C1..C9 antes de `approve`.
