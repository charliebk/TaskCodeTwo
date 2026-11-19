# Plan — TASK-036: F4-1 plan con brainstorm de roles

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 210 min
Senales: un comando nuevo con su operacion de la rama de estado (precedente: `ciclo` y `toma`), una tabla de roles y plantillas portadas de la 0.16, una heuristica portada, un subcomando para guardar el plan y unos 6 tests, la mitad pesados (repo bare mas clon, como en `ciclo/mod.rs`).

`tc2 plan TASK-NNN` es una orden mas del flujo y escribe solo con una `Operacion` que pasa por `estado::transaccion`, el unico punto de escritura. Lo que decide es puro: el carril y la complejidad dan cuantos roles se lanzan, y las etiquetas dan los condicionales. Publica en la carpeta fija de la tarea, en la rama de estado (F1-9 punto 2):
- `TASK-NNN/planificacion/plan-final.md`: el esqueleto de F1-9 punto 7, solo si no existe todavia.
- `TASK-NNN/planificacion/brainstorm/peticion-<rol>-<ronda>.md` mas `peticion-unificador-<ronda>.md`, o `peticion-plan-<ronda>.md` cuando hay un solo rol (mismos nombres que `plan-brainstorm.ts` de la 0.16).
- `tarea.md`: pasa a `en-diseno`, `carril:` queda congelado, fila en `## Transiciones` con fase `plan` y el modo congelado (F1-10 D5).

El CLI no llama a ningun modelo (como la 0.16, TASK-013/016).

Que se crea y que se extiende, en este orden:
1. **`carril.rs` se extiende** (su comentario: «F4 y F6 amplian este modulo en vez de copiar la tabla»): `Carril::declarado(tipo)` (`fix` para el tipo `fix`, `funcionalidad` para el resto, F1-10 C) y `Carril::roles_diseno(self, nivel, tipo, config) -> u8`: errata 0, fix 1, funcionalidad segun la complejidad con minimo `roles_min()`, riesgo alto 4; despues `max` con `politica_carriles.<c>.roles_diseno`, y el tope de 1 de hotfix solo fuera de riesgo alto.
2. **Se crea `plan/roles.rs`**: la tabla constante de los 5 roles mas el id del unificador, con `id`, `pregunta`, `mira`, `noMira` y `entregables` copiados literalmente de `roles-brainstorm.ts` de la 0.16.2, el orden de la escalera y `ROLES_CONDICIONALES` (rendimiento por las etiquetas `rendimiento/wasm/workers/3d` y solo si se lanzan 2 o mas roles), y `seleccionar(n, etiquetas)`.
3. **Se crea `plan/heuristica.rs`**: la heuristica de `heuristica-complejidad.yml` como constantes (pesos, cortes 0/3/5/7/8, roles 0..4, las 14 palabras de riesgo), plegando con `tarea::texto::plegar`. Aqui tambien la validacion que bloquea: objetivo vacio, ningun criterio, un criterio vacio, mas de 12 criterios.
4. **Se crea `plan/plantillas.rs`**: funciones puras de texto (peticion de rol, del unificador, de redaccion y esqueleto de `plan-final.md`), con la cabecera, el bloque de complejidad y el enunciado de la 0.16; `taskctl approve` pasa a `tc2 approve`.
5. **Se crea `plan/operacion.rs` con `Planificar`**: implementa `Operacion`, `exige_red() = true`. Reutiliza `tarea_de`, `por_estado` (origenes `planificada` y `en-diseno`), `de_otra_persona` y `Cuerpo::anadir_transicion`. La ronda sale de `estado.rutas()`; repetir sobre una ronda abierta no cambia nada.
6. **Se crea `plan/mod.rs` con la orden** (precedente `ciclo::orden_descartar`): `Entorno` de `toma`, configuracion de la base con `config::de_base` (politica y `modo_flujo`), linea `plan ok id= carril= modo= ronda= roles= forma=redaccion|unificador peticiones=`. Se engancha en `lib.rs` con `de_una_tarea`, mas el catalogo de anotaciones e `i18n/catalogo/plan.rs`.
7. **Se crea `plan guardar TASK-NNN <destino> <fichero>`** (D3): publica en `planificacion/` un fichero de fuera del repo; destinos solo `plan-final.md` o `brainstorm/salida-<rol>-<n>.md`; solo con la tarea `en-diseno` y sin aprobar. Precedente `importar/fichero.rs`. Motivo (comprobado en `estado/transaccion.rs:392`): con `tareas/` sucio toda transaccion acaba `rechazada motivo=tareas-con-cambios`, asi que el agente no puede escribir sus salidas directamente en `tareas/` como en la 0.16.

No se toca: `toma::MODO` (F4-4), `roles:` del frontmatter (F4-6, TASK-085), `approve` (TASK-037), `tc2 buscar` (H10 descartado).

Diferencias con la 0.16, a proposito: no se publican esqueletos vacios de `salida-*` (los escribe `guardar`); no hay migracion del `plan-final.md` antiguo de la raiz; no se avisa de palabras vagas ni de varios frentes.

## Lo que el rol no cubrio

- Criterio 1: la forma se copia de la tabla; que los agentes `brainstorm-*` esten instalados no se comprueba (inventario de TASK-080). tc2 no trae agentes propios.
- Criterio 2: el significado exacto de «todos» y de «uno» va en D1 y D2; como llega una tarea a riesgo alto antes de tener diff, en D4.
- Criterio 3 (RR-02): la revision por dominio es de F6. Sin roles de riesgos ni testing: los casos limite de la deduccion de ronda no estan mirados (en la 0.16 costaron 5 rondas).
- El plugin instalado es el 0.16.2 (no hay 0.16.1 en la cache): se portan `roles-brainstorm.ts`, `plan-brainstorm.ts`, `heuristica-complejidad.yml` y `commands/plan.ts`.

## Riesgos aceptados y que los contiene

- Que el texto copiado de los roles se separe del de la 0.16: un test fija ids y preguntas, citando la version de origen (0.16.2).
- Pesos de la heuristica en codigo y no en YAML: la configuracion de la base solo puede subir roles (`politica_carriles`); hacerla configurable queda fuera.
- La ronda se deduce de los ficheros de la rama de estado: se decide en `aplicar` sobre el estado fresco; dos `plan` a la vez reintentan en vez de pisar.
- `plan` sin red falla en lugar de quedar pendiente: el orquestador necesita las peticiones en ese momento.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio):
- Criterio 1: 1 test, ligero: ids de la escalera y del condicional en orden, sus preguntas literales y el unificador `brainstorm-unificador`.
- Criterio 2: 2 tests. Ligero, con tabla: `roles_diseno` y `seleccionar` sobre carriles x complejidad declarada o heuristica x hotfix x etiqueta rendimiento x `politica_carriles`. **Pesado** (bare mas clon): `plan` sobre una tarea fix publica solo `peticion-plan-1.md`; con `--carril riesgo-alto`, 4 peticiones de rol mas la del unificador.
- Criterio 3: 3 tests, **pesados**: `plan` publica en `TASK-NNN/planificacion/`, deja la tarea `en-diseno` con carril y modo congelados y la fila de transicion, no pisa un `plan-final.md` existente y repetir da `sin-cambios`; objetivo vacio o sin criterios da `rechazada` sin escribir; `plan guardar` publica `plan-final.md` y rechaza un destino fuera de la lista.

## Lo que necesita decision de una persona

Carlos dejo dicho el 2026-11-18 que el H4 se hace igual que las tareas del H0: opcion recomendada y sin trocear; se aplican:

- D1 «todos» en riesgo alto: (a) los 4 roles de base, rendimiento por etiqueta (como `carril.rs` `roles_min() = 4` y la PoC de F1-10). **Elegida.**
- D2 roles en fix: (a) exactamente 1; solo `politica_carriles.fix.roles_diseno` puede subirlo. **Elegida.**
- D3 publicar salidas y plan final: (a) `tc2 plan guardar` en esta tarea, desde un fichero de fuera del repo. **Elegida.**
- D4 llegar a riesgo alto: (a) `tc2 plan TASK-NNN --carril <c>`, que solo sube por encima de `Carril::declarado(tipo)` y queda congelado; el check `carril` de F6 vuelve a comprobar el minimo por el diff. **Elegida.**
- D5 complejidad no declarada: (a) portar la heuristica de la 0.16 y usar el mayor de declarada y heuristica. **Elegida.**

## Troceo

### Parte a: seleccion de roles y plantillas (puntos 1 a 4, puros, sin git)
### Parte b: operacion `Planificar` y la orden `plan` (puntos 5 y 6)
### Parte c: `plan guardar` (punto 7)

No se trocea: Carlos pidio el H4 sin trocear (2026-11-18).
