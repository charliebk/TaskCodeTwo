# Brainstorm — TASK-118, rol riesgos (ronda 1)

- Rol: `brainstorm-riesgos`
- Agente: taskcode-plugin:brainstorm-riesgos

## Modos de fallo, ordenados por gravedad, con el escenario concreto de cada uno

1. **GitLab: el agente se queda sin poder empujar ni el estado ni las ramas de tarea.** En GitLab, «sin borrado ni force-push» solo se consigue protegiendo la rama, y la API de `protected_branches` pone `push_access_level` a Maintainer si no se le da otro valor. Si `taskcodetwo/tareas` y `feature/*`/`fix/*`/`hotfix/*` se protegen con ese valor, el agente, que es Developer (F1-13 D6), deja de poder hacer push del estado y de las ramas de tarea, y tc2 se para entero. Mitigacion: fijar push=Developer y `allow_force_push=false` en esas puertas, nunca el valor por defecto. Lo de Developer esta leido en el ADR; el valor por defecto de la API es suposicion.
2. **GitLab: la rama queda sin proteger si se cae a mitad de un «borrar y crear».** GitLab protege `main` al crear el proyecto. Si ya existe, `POST /protected_branches` da 409 (suposicion). La salida facil es `DELETE` + `POST`, y si el proceso muere, se corta la red o el `POST` da 4xx entre las dos llamadas, `main` o `develop` quedan **menos protegidas que antes**. Mitigacion: actualizar con `PATCH` sobre la proteccion que ya existe; si no se puede, rechazar con el arreglo y no tocar nada.
3. **GitHub: un check que nunca llega bloquea `develop` para todo el mundo, persona incluida.** `ruleset_github` lleva `bypass_actors: []` (`crates/tc2/src/plataforma/puertas.rs:261`). Basta un `integration_id` que no es cero pero es de una App no instalada, una errata en el numero, o `verificacion`/`carril` exigidos sin el workflow de F5-1: ningun PR a `develop` o `main` se puede mergear. Se sale borrando el ruleset desde la UI. Mitigacion: aceptar el origen de un check solo si ya se ha visto un check con ese nombre publicado por ese origen (decision D2).
4. **Se pisan reglas que la persona puso a mano.** F1-5 dice «distinto → los actualiza» (`docs/adr/F1-5-puertas-en-github.md:368`). Un `PUT` sustituye el ruleset entero y se pierde, por ejemplo, el actor con bypass que la persona anadio para no quedarse fuera. En GitLab pasa lo mismo con los ajustes de merge del proyecto y con los niveles de acceso que ya tenian las ramas protegidas. Mitigacion: decision D3.
5. **Nunca converge y vuelve a escribir en cada ejecucion.** En GitLab, `ReglasLeidas.rulesets` solo lleva los nombres de las reglas de aprobacion (`gitlab.rs:1011`), y el inventario mira `r.rulesets.contains(&nombre)` (`init/inventario.rs:684`). `taskcodetwo-ramas-de-tarea`, `-tareas` y `-tags-version` son ramas o tags protegidos, no reglas de aprobacion, asi que nunca salen como `existe` (verificado leyendo el codigo). En GitHub, si se compara el JSON tal como vuelve (con `id`, `source`, `_links`, valores por defecto anadidos y otro orden), siempre sale «distinto» y se hace `PUT` cada vez. Mitigacion: comparar solo los campos del modelo, ya normalizados, y en GitLab reconocer lo aplicado por rama o tag protegido, no por nombre.
6. **Se aplican las puertas antes de que lo que escribe `init` haya llegado al remoto.** `init` no empuja nada (no hay efecto de push en `init/aplicar.rs`). Si en la misma ejecucion se aplica `taskcodetwo-develop`, el push con el que la persona sube la configuracion y `personas.yml` a `develop` queda rechazado, y no tiene alternativa por PR porque faltan los checks. Mitigacion: exigir que `develop` y `main` existan en el remoto y que la cabeza local no vaya por delante de la remota; si no, rechazar.
7. **El agente aplica las puertas.** F1-6 detecta el caso `agente-con-administration`. Si `--aplicar-puertas` no obliga a usar `Rol::Persona` con TTY (H6), un agente con un token demasiado amplio las aplica, o las cambia. Mitigacion: fallar cerrado sin la identidad de la persona y sin TTY.

## Estados intermedios y fallos parciales

- Son cinco puertas, y en GitLab unas 10 llamadas o mas (proteccion, tags, ajustes del proyecto, aprobacion). Si se pierde la red, llega un 429 o un 5xx en la llamada 3 de 5, quedan unas creadas y otras no, y no hay transaccion. Lo aceptable es que repetir la orden lleve al mismo estado final, y que la linea diga que se aplico y que no (`aplicadas=…`, `faltan=…`).
- Orden: las tres puertas sin checks son inocuas si quedan solas. Lo peligroso a medias es solo el riesgo 2 (GitLab). En GitLab Free, `approval_rules` da 200 con la lista vacia (`mod.rs:128`): el plan parece dar reglas y el `POST` posterior falla con 403 cuando las ramas ya estan protegidas. Hay que leer el tier antes de escribir nada.
- Concurrencia: si dos personas, o dos clones, ejecutan la orden a la vez, las dos leen «no existe» y las dos crean. La segunda recibe 422 en GitHub (nombre unico) o 409 en GitLab (suposicion), y eso debe tratarse como «otro llego antes: vuelvo a leer y comparo», no como error.
- Sin checks ni `integration_id`, el rechazo tiene que llegar **antes de cualquier escritura**, tambien de las tres puertas sin checks (decision D1).

## Compatibilidad hacia atras

- **Rulesets que ya existen y no llevan el prefijo** (de la persona, de la organizacion o los `taskcode-*` del texto de F1-5): no se tocan nunca. Los rulesets de GitHub se suman, asi que el resultado efectivo puede ser mas estricto de lo que dice el modelo.
- **Rulesets heredados de la organizacion:** `GET /rulesets` los incluye. Uno de organizacion llamado `taskcodetwo-develop` saldria como `existe`, y un `PUT` por id al endpoint del repo fallaria (suposicion).
- **`ruleset_github` anade `_nota`** a `develop` y `main` (`puertas.rs:256`). Si `aplicar` reutiliza esa funcion, la clave se envia. F1-5 dejo sin verificar si GitHub rechaza o ignora claves desconocidas (divergencia 3).
- **Repos que siguen con la 0.16**, que mergea en `develop` con push directo: segun el comentario de `init/mod.rs:19`, `init` los rechaza; un companero con un clon que aun usa los scripts de la 0.16 se queda bloqueado. No verificado mas alla de ese comentario.
- **`tc2 desinstalar`** solo lista los rulesets como `persona motivo=si-existe` (`desinstalar/proyecto.rs:314`). Para GitLab no hay forma de saber que ramas protegidas puso tc2.

## Vuelta atras

- GitHub: un admin puede borrar o editar los rulesets aunque no haya bypass. Reversible a mano desde la UI o la API; tc2 no tiene una orden para quitarlos.
- GitLab: un Maintainer puede desproteger. Reversible a mano.
- **Punto sin retorno:** los valores anteriores que se sobrescriben (un ruleset de GitHub editado a mano, los ajustes de merge del proyecto en GitLab, los niveles de acceso de una rama ya protegida) se pierden, porque nadie los guarda. Mitigacion: volcarlos en la salida o en un fichero fuera del repo antes de escribir.
- Si una puerta deja fuera al agente (riesgos 1 y 3), el flujo queda parado hasta que la persona intervenga a mano.

## El riesgo que mas te preocupa (UNO solo)

El riesgo 2: en GitLab, actualizar la proteccion de `main` o `develop` borrandola y volviendola a crear. Un fallo a mitad deja la rama **mas desprotegida que antes de ejecutar tc2**, nadie se entera, y el proximo `init` sin la opcion lo da por bueno.

## Decisiones abiertas (opciones)

- **D1. Rechazo por falta de checks.** (a) Se rechaza la orden entera sin escribir nada. (b) Se aplican las tres puertas sin checks y se rechazan solo `develop` y `main`. **Recomendada: (a).** Es lo que dice el criterio, no deja estado mixto y hoy siempre rechaza: no hay ninguna fuente de `integration_id` en el codigo (grep).
- **D2. Que es un `integration_id` «real».** (a) Basta un valor distinto de cero en la configuracion. (b) Hace falta ademas haber visto un check con ese nombre publicado por ese origen en la cabeza de `develop`, que `leer_puertas` ya lee (`checks_publicados`). **Recomendada: (b).**
- **D3. Ruleset `taskcodetwo-*` que ya existe y es distinto.** (a) Se actualiza sin mas, como dice F1-5. (b) Se informa `difiere` con el diff y solo se actualiza con una confirmacion explicita, guardando antes lo anterior. **Recomendada: (b).** Contradice F1-5: desacuerdo con ese ADR.
- **D4. GitLab, rama ya protegida.** (a) `PATCH` sobre la proteccion que existe y, si no se puede, rechazar. (b) `DELETE` + `POST`. **Recomendada: (a).**
- **D5. Requisito previo de las ramas en el remoto.** (a) Se rechaza si `develop`/`main` no existen en el remoto o si el local va por delante. (b) Se aplica igualmente y se avisa. **Recomendada: (a).**

## Suposiciones no verificadas

- Que la API de GitLab pone push=Maintainer por defecto, que da 409 si la rama ya esta protegida, y que en `feature/*` el `*` abarca nombres con `/`.
- Que en GitHub un ruleset con `pull_request` en `refs/heads/develop` impide crear `develop` con un push.
- Que GitHub da 422 al crear un ruleset con un nombre que ya existe, y que `GET /rulesets` incluye los heredados de la organizacion.
- Que en GitLab Free o Premium no se puede exigir un estado concreto por su autor en la propia rama protegida («los checks» de GitLab se reducen a «el pipeline tiene que pasar»).
- Que la persona puede aprobar los PR de release `develop`→`main` con `require_last_push_approval` en un repo con una sola persona.
