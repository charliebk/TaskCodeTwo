# Peticion de redaccion del plan — TASK-145 (ronda 1)

- Tarea: TASK-145 — Auditoria 4D: A7 config, init, desinstalar, i18n, confianza y ciclo (8 IMPORTANTE)
- Tipo: fix · Complejidad declarada: no declarada (decide la heuristica)
- Ronda: 1
- Fecha: 2026-11-25
- Rol: `brainstorm-arquitectura` — lanzalo con el agente de ese mismo nombre
- Quien orquesta vuelca tu respuesta en: `../plan-final.md`

**Esta tarea se planifica con un solo rol y no hay unificador**: tu respuesta ES el plan final. Redactala con las secciones de abajo; quien orquesta la vuelca tal cual en `plan-final.md` (tu no escribes ficheros), sin salida intermedia que consolidar.

## Tu pregunta

> Dado lo que ya existe, ¿donde encaja este cambio y que forma tiene?

## Que miras

- Los modulos y ficheros que ya resuelven algo parecido, para extenderlos en vez de duplicarlos.
- Que se crea nuevo, que se extiende y en que orden se construye.
- Los limites que el cambio cruza: contratos publicos, formatos de fichero, esquemas.
- El precedente interno mas cercano: como se resolvio la ultima vez un problema de esta forma.

## Que NO miras

Que lo mires solo tu no te autoriza a cubrirlo todo: lo que queda fuera de tu pregunta no se mira, pero se DICE. Anota en el plan lo que no cubres, para que quien lo apruebe sepa que ese lado no se ha revisado:

- Como se prueba (es del rol de testing).
- Por donde se rompe (es del rol de riesgos).
- Las reglas de negocio (son del rol de dominio).

## Complejidad: declarada frente a heuristica

- Declarada en la tarea: **no declarada (decide la heuristica)**
- Heuristica (1 puntos): **simple**
- Senales encontradas:
  - criterios_aceptacion: 7 criterios (umbral: 5) (+1)

Los dos niveles coinciden. Se lanzan 1 rol.

## Que tiene que traer el plan final

- Enfoque propuesto, concreto: que se crea, que se extiende, en que orden.
- Lo que el rol no cubrio, contrastado contra los criterios de aceptacion.
- Reutilizacion (Regla 4D): lo que ya existe en el repo y se va a usar (helpers, funciones, fixtures, tests), buscado ANTES de proponer nada nuevo; y lo que se crea nuevo, con el motivo de no reutilizar lo que hay. Duplicar un helper que ya existe es un hallazgo IMPORTANTE.
- Riesgos aceptados y que los contiene.
- Plan de pruebas.
- Lo que necesita decision de una persona antes de implementar. **El checkpoint humano es obligatorio**: este plan no vale hasta que alguien lo apruebe con `taskctl approve`.

## Reglas

- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es el plan.
- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.

## Enunciado de la tarea

### Objetivo

````
Corregir los 8 casos IMPORTANTE del grupo A7 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 4 Duplicaciones (una `divergente`), 1 Desviación de F2-8 que cambia la salida y 3 Disfraces (uno en código y dos de test con mutante confirmado). A7-02 y A7-06 van en las tareas de abstracción común (A8-04 y A8-12). No se pide el MENOR (A7-08). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.
````

### Criterios de aceptacion

````
A7-04 (`divergente`, va primero): los ayudantes de test `md`, `estado`, `motivo`, `texto` y `fila` de `ciclo/liberar.rs:423-467` y `ciclo/operacion.rs:286-325` quedan en un solo módulo de prueba de `ciclo`, con los parámetros que hoy separan `md` y `fila`.
A7-01 y A7-03: el esqueleto «plan como dato» de `init/mod.rs` y `desinstalar/mod.rs` (rangos de la ficha A7-01) se comparte, y `estado` y `confiar` de `confianza/mod.rs:198-242` ~ `:246-275` comparten la lectura, el cierre, la huella y la `Evaluacion`.
A7-05: los dos tests unitarios de `ciclo/mod.rs:505-527` ~ `:570-590` montan el escenario git con un solo ayudante.
A7-07 y A7-10: ningún texto se renderiza en español en el sitio para meterlo en `{error}` (`init/aplicar.rs:25-27`, `:82-84`, `init/mod.rs:470`, `desinstalar/proyecto.rs:373-376`, `desinstalar/mod.rs:542`). `ningun_literal_fuera_del_catalogo` (`i18n/guarda.rs:17-23`, `:53-83`) caza la forma `Err("...".to_owned())`. Con el mutante de la ficha A7-10 aplicado, `cargo test -p tc2 --lib i18n::guarda::ningun_literal_fuera_del_catalogo` sale en rojo, y con `TC2_IDIOMA=en` el diagnóstico sale entero en inglés.
A7-09: `config::del_arbol` (`config/mod.rs:534-543`) da `sin-config` solo con `ErrorKind::NotFound`. Un `.taskcode/config.yml` en UTF-16 o sin permiso de lectura da otro motivo con la causa, y no «crea la configuración con tc2 init». Lo prueba un test con un fichero UTF-16LE con BOM.
A7-11: un test ejercita el rechazo `ramas-sin-remoto` con `develop` sin publicar (`init/inventario.rs:347-352`, criterio de TASK-118), y el docstring de `aplicar_puertas_exige_personas_ramas_y_persona` (`tests/init.rs:1693-1754`) dice lo que provoca. Con el mutante de la ficha aplicado, `cargo test -p tc2 --test init` sale en rojo.
`cargo xtask verificar` en verde, y el `## Resultado` lista los 8 IDs con su commit.
````
