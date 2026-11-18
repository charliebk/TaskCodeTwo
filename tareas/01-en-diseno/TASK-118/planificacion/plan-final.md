# Plan — TASK-118: Aplicar las puertas (rulesets) desde tc2 init

(Lo consolida el agente unificador a partir de 2 roles de brainstorm
lanzados en paralelo: arquitectura, riesgos.
Los desacuerdos entre roles se senalan, no se promedian.)

## Enfoque propuesto

Duracion estimada: 150 min
(de la rama abierta al cierre con merge; senales: ~9 ficheros de codigo previstos, 3 capas puertas/adaptadores/init, 2 criterios; heuristica «simple» frente a «media» declarada: el enunciado infraestima, 2 adaptadores de escritura)
- Orden de arquitectura: (1) `Origenes` y `origenes(cfg)` puros en `puertas.rs`, clave `plataforma.revisor_id`, `ruleset_github(p, Option<&Origenes>)` sin `_nota`; (2) `aplicar_puerta` en el trait con defecto `rechazada motivo=no-soportada` y GitHub `POST {r}/rulesets`; (3) GitLab: ramas/tags protegidos, metodo de merge y aprobacion en Premium; (4) `init --aplicar-puertas`: precondicion, `Efecto::AplicarPuerta`, `Conexion` unica; (5) notas y adenda al ADR F1-5.
- Precondicion unica antes de cualquier escritura (riesgos D1 + D2 + D5, ver desacuerdos): `Rol::Persona` con TTY y `personas.yml` existente (riesgos 7, arquitectura suposicion 1), `origenes(cfg)` Ok, checks ya vistos publicados por ese origen en `develop`, `develop`/`main` en el remoto y local no por delante. Si falla: `init rechazada motivo=puertas-sin-origen faltan=...`.
- GitLab (riesgos 1): push=Developer y `allow_force_push=false` explicitos en `taskcodetwo/tareas` y ramas de tarea; nunca el valor por defecto de la API. Nunca `DELETE` de una proteccion (riesgos 2).

## Troceo

### Parte a: modelo puro y config (`Origenes`, `origenes(cfg)`, `revisor_id`, `ruleset_github` con origenes)
### Parte b: trait `aplicar_puerta` y adaptador GitHub (`POST /rulesets`, 403 de plan, 422 = otro llego antes)
### Parte c: adaptador GitLab (protecciones, merge, aprobacion Premium, deteccion por proteccion y no por nombre)
### Parte d: `init --aplicar-puertas` (precondicion, efecto, `Conexion` unica, linea `aplicadas=/faltan=`) y adenda ADR
No se trocea: Carlos pidio estas cuatro tareas del H0 sin trocear (2026-11-18).

## Desacuerdos entre roles, y como se resuelven

- D1 rechazo entero — riesgos D1(a): sin checks se rechaza la orden entera antes de cualquier escritura, tambien las tres puertas sin checks / arquitectura D2(a)+D3(a): un `Efecto` por puerta con corte en el primer fallo, precondicion local — gana riesgos (a) para la precondicion — fail-closed y es lo que dice el criterio 2; el `Efecto` por puerta de arquitectura se conserva para fallos de red, con `aplicadas=/faltan=` y repeticion idempotente (riesgos, estados intermedios).
- D2 origen «real» — riesgos D2(b): exigir un check con ese nombre ya publicado por ese origen en la cabeza de `develop` (`checks_publicados`) / arquitectura D3(b) descartada: huevo y gallina en repo nuevo y red en la precondicion — gana riesgos (b) — fail-closed: un id erroneo o de App no instalada bloquea `develop` para todos con `bypass_actors: []` (riesgos 3); el coste (la opcion no aplica hasta tras el primer PR con checks) es repetible y reversible, el bloqueo no.
- D3 ruleset `taskcodetwo-*` existente y distinto (GitHub) — riesgos D3(b): informar `difiere` y actualizar solo con confirmacion, guardando lo anterior / arquitectura D4(a): crear solo los que faltan, no tocar los existentes — gana arquitectura (a) — no escribe sobre nada existente (lo mas cerrado) y cabe en el alcance; la actualizacion con confirmacion de riesgos pasa a fuera de alcance. Ambos roles contradicen F1-5 l. 368 («distinto → los actualiza»): adenda al ADR.
- D4 GitLab rama ya protegida — riesgos D4(a): `PATCH` sobre la proteccion existente o rechazar; nunca `DELETE`+`POST` / arquitectura D4(a): no tocar lo existente — gana arquitectura (a), con la prohibicion de `DELETE` de riesgos como regla — no tocar es mas seguro que `PATCH`; consecuencia aceptada: en GitLab `main` (protegida al crear el proyecto) queda con la proteccion previa y se informa `difiere`.

## Riesgos aceptados y que los contiene

- Agente sin push en GitLab por push=Maintainer por defecto — riesgos 1 — niveles explicitos en el adaptador y test que los comprueba.
- Bloqueo de `develop` por check que nunca llega — riesgos 3 — D2(b) y precondicion; vuelta atras manual desde la UI (riesgos).
- Estado parcial por red/429/5xx entre puertas (GitLab ~10 llamadas) — riesgos — repeticion idempotente y linea `aplicadas=/faltan=`; leer el tier antes de escribir.
- No converge: GitLab `rulesets.contains` nunca acierta (riesgos 5, verificado leyendo codigo; arquitectura suposicion 2) — deteccion por proteccion leida; comparar solo campos del modelo normalizados.
- La opcion se rechazara en la practica hasta F5-1/F6-1 — arquitectura — es el comportamiento del criterio 2.
- Concurrencia de dos clones (422/409) — riesgos — tratar como «otro llego antes: releo y comparo».

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; todos contra doble HTTP `comun/http_mock`):
- Criterio 1 (crea rulesets en GitHub y GitLab): 3 tests.
  - GitHub: `init --aplicar-puertas` hace `POST /rulesets` de las cinco `taskcodetwo-*` con `integration_id` y sin `_nota`; las existentes no se tocan — arquitectura/riesgos — **pesado** (init de punta a punta).
  - GitLab: protecciones con push=Developer y `allow_force_push=false`, ninguna peticion `DELETE`, rama ya protegida sale `difiere` — riesgos 1, 2 — **pesado**.
  - Segunda ejecucion: `sin-cambios` y cero escrituras en ambas plataformas — riesgos 5 — ligero (adaptador).
- Criterio 2 (rechazo con el motivo): 2 tests.
  - `origenes(cfg)` sin `revisor_id` o con 0 devuelve los checks que faltan — arquitectura — ligero (puro).
  - init sin checks publicados por ese origen (o sin persona/TTY, o `develop` sin remoto) sale `rechazada motivo=puertas-sin-origen faltan=...` y el doble registra cero `POST/PUT/PATCH/DELETE` — riesgos D1, D2, D5, 7 — **pesado**.

Fuera de alcance (no cabe en los dos criterios): actualizar rulesets distintos con confirmacion (riesgos D3 b), `PUT` de F1-5 y `doctor --arreglar` (arquitectura D4); `PATCH` de protecciones GitLab existentes (riesgos D4 a); volcado de valores previos (riesgos, vuelta atras); `tc2 desinstalar` de rulesets y ramas protegidas (riesgos); rulesets heredados de organizacion (riesgos).
Salidas que faltaron: ninguna; no se lanzo rol de dominio (arquitectura le anticipa desacuerdos en D1 y D4 que nadie defiende aqui).
Suposiciones no verificadas: que `POST /rulesets` acepta el JSON de la PoC C con `integration_id` (arquitectura; F1-5 no escribio); defaults y 409 de `protected_branches` y `feature/*` con `/` (riesgos); 422 por nombre repetido y heredados en `GET /rulesets` (riesgos); que `checks_publicados` da el origen del check (riesgos; mirar `leer_puertas`); ruta de CI de GitLab para la precondicion (arquitectura; plantilla de F5-1).

## Lo que necesita decision de una persona

Carlos dejo dicho el 2026-11-18 que en estas cuatro tareas del H0 se acepta la opcion recomendada y sin trocear. Donde los roles discrepan, la recomendada es la mas segura (fail-closed).
- Rechazo sin checks: (a) orden entera rechazada sin escribir nada (recomendada, riesgos D1) / (b) aplicar las tres sin checks. **Elegida.** (a): no deja estado mixto.
- Origen real: (a) valor distinto de cero en config (arquitectura) / (b) ademas check ya visto publicado por ese origen (recomendada, riesgos D2). **Elegida.** (b): fail-closed frente al bloqueo de `develop`.
- Ruleset/proteccion existente y distinta: (a) no tocar, informar `difiere`, adenda a F1-5 (recomendada, arquitectura D4) / (b) actualizar (`PUT` F1-5, `PATCH` o confirmacion, riesgos D3/D4). **Elegida.** (a): nunca pisa; nunca `DELETE`.
- Fuente del id: (a) clave `plataforma.revisor_id` u64 + 15368 para Actions (recomendada, arquitectura D1) / (b) slug y `GET /apps/{slug}`. **Elegida.** (a): adelanta un contrato de F6-1; lo caro de deshacer es la clave de config.
- Ramas en el remoto: (a) rechazar si `develop`/`main` no estan o el local va por delante (recomendada, riesgos D5) / (b) aplicar y avisar. **Elegida.** (a).
El plan no vale hasta que Carlos lo apruebe con `taskctl approve`.
