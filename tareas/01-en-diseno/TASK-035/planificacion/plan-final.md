# Plan — TASK-035: F3-8 doctor: entorno, rama de estado, permisos, rulesets y checks reales

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 450 min (300 del plan del rol + 150 por D1b, `capacidades` dentro de esta tarea)

Modulo nuevo `crates/tc2/src/doctor/`, orden de solo lectura como `board` (`tablero.rs`), que junta entorno, rama de estado, instalacion y plataforma en una lista de `Hallazgo { area, id, estado: ok|aviso|falla|no-comprobado, motivo, arreglo }`. Las lecturas de plataforma usan el puerto existente (`leer_puertas`, `leer_permisos_ci`, `identidad`) y se comparan con `plataforma::puertas::modelo()` (`plataforma/mod.rs:122-123` ya asigna esa comparacion a F3-8). Lo local sale de `estado::leer`, `config::de_base` y del inventario de `init` en simulacion.

Salida (F1-4 D4 y F1-13 seccion C): primera linea `doctor <ok|rechazada> fallos=N avisos=N no-comprobado=N plataforma=<ok|no-configurada|sin-credenciales|sin-red|error> commit=…` y hasta 4 hallazgos, los peores primero, mas `y N mas: tc2 doctor --json`. `--json` con todos.

Piezas:
- `doctor/mod.rs` (nuevo): opciones, `Hallazgo`, agregador puro `informe(&[Hallazgo]) -> Informe` y salida (patron «plan como dato» de `init/mod.rs`, con 5 lineas como mucho).
- `doctor/entorno.rs` (nuevo): version de `git` por el puerto `Git` (hoy no hay ninguna comprobacion de version), `tc2` y su protocolo (`version.rs`), estar en un repo, config de la base valida (`config::de_base`).
- `doctor/estado.rs` (nuevo): rama de estado existe (`motivo=sin-rama-de-estado`, `estado/lectura.rs:155`) y es legible (ilegibles de `situacion`, como `board`). Instalacion: inventario de `init` en simulacion con credenciales vacias (rulesets `pendiente motivo=sin-token` sin API); cada accion que no salga `existe` es un hallazgo, asi `doctor` e `init` no discrepan. Requiere `pub(crate) fn plan(&Contexto) -> Result<Vec<Accion>, Informe>` en `init/mod.rs`.
- `doctor/plataforma.rs` (nuevo): proteccion por cada puerta de `modelo()` con `solo_pr` (develop, main) via `leer_puertas` (`falla motivo=proteccion-ausente arreglo=tc2-init` si falta el ruleset `taskcodetwo-<nombre>` o la regla `pull_request`; GitLab `push=0`); permisos con `identidad` por rol y `leer_permisos_ci` (`Denegada` con `permiso=` → `falla motivo=permiso-insuficiente`); modo degradado como `Consulta::de_fallo` (`tablero.rs:327`).
- `doctor/capacidades.rs` (nuevo, D1b): el contrato de F1-13 decision 3 `capacidades(plataforma, plan_observado, puertas) -> Vec<Garantia>` con la tabla de equivalencias G01..G20 compilada en el binario como dato (F1-13 decision 2) y el plan observado de la plataforma; `doctor` imprime las garantias sustituidas o ausentes con su motivo y arreglo (F1-13 seccion C), dentro del limite de 5 lineas.
- `identidad.rs` (se extiende): recibe `Consulta`, `Conectada`, `Conectar` y `conectar_produccion` desde `tablero.rs:297-384`; `tablero` los reexporta (evita acoplar `doctor` a otra orden y un ciclo con `plataforma/`).
- `plataforma/github.rs`, `gitlab.rs` y `ReglasLeidas` (se extienden): `checks_requeridos` (de `parameters.required_status_checks[].context` que ya trae `GET /rules/branches/{rama}`; hoy `ReglaApi` solo lee `type`, `github.rs:218`) y `checks_publicados: Option<Vec<String>>` (una lectura mas: `GET /commits/{rama}/check-runs` en GitHub, `GET /repository/commits/{rama}/statuses` en GitLab). El plan observado para `capacidades` se lee del repositorio/proyecto (`GET /repos/{o}/{r}` en GitHub, proyecto en GitLab) como lectura del puerto.
- `i18n/catalogo/doctor.rs` (nuevo) y despacho en `lib.rs`.

Orden: (1) esqueleto, agregador, salida texto/JSON, despacho, catalogo y `entorno.rs`; (2) `estado.rs` y `plan` de `init`; (3) mover `conectar_produccion` a `identidad.rs` sin cambio de comportamiento (tests de `board` en verde) y `plataforma.rs`; (4) checks requeridos y publicados (`check-sin-publicar`, `check-no-obligatorio`); (5) `capacidades` con la tabla G01..G20 y el plan observado; (6) adendas: F1-13 decision 4 (`leer-puertas` gana una llamada; lectura del plan) y F1-4 (salida de `doctor`).

Descartado: operacion nueva `checks-de-rama` en el puerto (amplia el contrato por la misma lectura de puertas); copiar en `doctor` la construccion del adaptador de `init/inventario.rs:541-574` (tercera copia; esa deuda de `init` se anota).

## Lo que el rol no cubrio

- Sin roles de riesgos, testing ni dominio.
- Fuera de alcance, contrastado con el criterio «cumple F3-8»: `doctor --recursos` (D2a: lo hace F3-9, TASK-080, que trae el inventario; la adenda de F1-14 ya lo liga a ese inventario); validez de la politica y tags `v*` sin firmar (F1-10); aviso de claves desconocidas (F1-9 D4, F1-11); ajustes de Actions (F1-8 A14); indice desfasado (F10-7, descartada). El origen de los checks (`integration_id`) espera a F6-1: solo se comprueba el nombre.
- Suposiciones a verificar: que el inventario de `init` en simulacion no escribe nada; que `GET /commits/{rama}/check-runs` acepta un nombre de rama; que responde GitLab a los `statuses` de una rama; como se observa el plan en cada plataforma.

## Riesgos aceptados y que los contiene

- `GET /rules/branches` no ve la proteccion clasica de GitHub: una rama protegida a la antigua sale «ausente»; el motivo nombra el ruleset `taskcodetwo-*` (modelo de F1-5).
- `doctor` no hace `fetch`: lee la ultima copia traida, como `board`; la salida lleva `commit=`; la frescura es de F3-5b.
- El inventario de `init` falla entero sin red: la instalacion queda `no-comprobado`; las comprobaciones locales siguen.
- Una cabeza de `develop` sin checks publicados (repo recien creado) da `aviso` mientras F5-1 no instale el workflow (D4a).

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- Unitarias puras: agregador (eleccion de las 5 lineas, codigo, JSON), comparacion de `ReglasLeidas` con `modelo()` una por motivo, `capacidades` sobre la tabla con planes distintos.
- Integracion con `http_mock`: criterios 1, 2, 4, 5, 9 y 11 en GitHub y en GitLab con las mismas aserciones sobre la linea (RR-15).
- Repos temporales: criterios 3, 6 y 10 (captura de refs, worktrees y ficheros antes y despues).
- Regresion: `board`, `init` y `plataforma` sin cambios tras mover `conectar_produccion` y extender `ReglasLeidas`.

## Lo que necesita decision de una persona

Carlos dejo dicho que en el H3 se acepta la opcion recomendada **y que no se trocea ninguna tarea**; donde la recomendada era trocear, manda no trocear:

- D1 `capacidades` G01..G20 por plan: el rol recomendaba (a) tarea nueva F3-8b; se elige **(b) incluirla aqui**, porque trocear esta excluido y F1-13 decision 3 asigna el contrato de `capacidades` a F3-8.
- D2 `doctor --recursos`: (a) lo hace F3-9 (TASK-080), que trae el inventario; no es trocear, es la tarea del plan que ya une `doctor` e inventario. **Elegida.**
- D3 version minima de git: (a) `aviso` por debajo de 2.56 y `falla` solo sin git. **Elegida.**
- D4 check requerido sin publicar: (a) `falla` si la rama tiene checks publicados y falta ese; `aviso` si no hay ninguno. **Elegida.**
- D5 codigo con problemas: (a) 3 (`rechazada`) con alguna `falla`, 0 con avisos. **Elegida.**
