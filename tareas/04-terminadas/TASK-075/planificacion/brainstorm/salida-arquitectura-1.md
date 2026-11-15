# Salida del rol brainstorm-arquitectura — TASK-075 (ronda 1)

(Volcada tal cual por quien orquesta. Por la regla de F1-15 se lanzan arquitectura y riesgos más unificador; esto es la aportación de arquitectura, no el plan final.)

## Enfoque

Un solo núcleo puro `estimacion` en `tc2`. Lo alimenta un único lector del histórico, que saca las muestras de los `tarea.md` terminados (`## Transiciones`, `## Duracion`, `## Coste`). De ese núcleo beben la status line, `status --seguir`, `tc2 resumen`, `next --personas N`, la hora de fin del lote (F4-9 y F1-16) y la calibración de F8-7.

El CLI recalcula solo en cada transición y escribe el resultado en el campo `estimacion` de F1-9 dentro del mismo commit de estado (F1-2). La status line solo lee y resta del reloj: no recorre el histórico y no gasta tokens.

Duracion estimada: 300 min (F1-16 estimó 180 y le costó 594; esta tarea tiene la misma forma, un ADR con PoC sobre dos repos y más consumidores).

## Piezas y límites

- **Lector `historico(tareas) -> Vec<Muestra>`**: se crea, encima del parser de F1-9. Cada muestra lleva repo, carril, complejidad, tamaño, minutos por fase (sacados de las marcas plan→approve→start→review*→finish), rondas y tokens. Lo usan todos los consumidores. Es el único sitio que interpreta esas secciones, para que no acaben existiendo dos maneras de calcular la duración real (hoy la 0.16 tiene una y el prototipo en Python de la sesión otra).
- **Núcleo `estimar(tarea, &[Muestra]) -> Estimacion` y `restante(est, transiciones, ahora) -> Restante`**: se crea. Mediana y p75 por fase entre las tareas parecidas, en cascada: carril+complejidad+tamaño, luego carril, luego el repo, y al final la semilla de TaskCodeTwo (F1-18). Guarda cuántas muestras usó y de qué nivel. El carril efectivo y el tamaño del diff se piden a la función `carril` de F1-10 y no se recalculan.
- **Campo `estimacion` de F1-9**: se extiende el hueco que ya existe. Forma: `{total_min, por_fase:{diseno,implementacion,revision,cierre}, p75_min, base:"n=12 carril+complejidad", calculada}`. `## Duracion` gana una fila por fase al cerrar (F8-7). La línea `Duracion estimada: N min` del plan se queda como estimación del agente y se compara con la del CLI (D1).
- **Vistas**: se extienden `status`, `status --seguir` y la status line (F3-5, F3-12), y se crea `tc2 resumen`. Coste de cada vista:
  - La status line hace O(1): un `tarea.md` más el reloj, que es el presupuesto en caliente de F1-14.
  - `resumen` hace O(N) y solo se ejecuta cuando alguien lo pide. Muestra una tabla de terminadas y pendientes, los minutos y tokens por tarea y en total, y lo que falta con la mediana y el p75 por tarea y en total.
- **`repartir(dag, duraciones, zonas, n) -> Reparto`**: se crea, para `next --personas N` (F3-3). Asignación por listas con prioridad al trabajo restante (camino crítico) y sin juntar zonas solapadas en paralelo. Devuelve una lista seguida por persona, el total con N personas y el camino crítico, que con N=1 vale como comprobación.
- **`zonas` declaradas y `solapan(a, b)`**: se crean. Hace falta un campo nuevo en F1-9, y esa divergencia se nombra. `start` usa la misma función para avisar del solape (RR-12, F3-11). La negativa a arrancar con una predecesora sin terminar es una precondición de `start` (F3-6). Este ADR fija la regla, no su código.

## Orden de construcción

1. Contratos: `Muestra`, la forma de `estimacion`, `estimar` y `restante`, y la regla de cuándo se recalcula: en approve, en cada transición y en finish (en finish se registra el error).
2. PoC 1 sobre copias de ToolsaWeb y OpenDockTrading (los originales solo se leen), en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-075\`. Para cada tarea terminada se estima usando solo las que terminaron antes que ella. El error medio (en minutos, y el porcentual con mediana y p75) se compara con dos referencias: la «Estimada» de la 0.16 y la mediana global.
3. PoC 2: `resumen` y `repartir` sobre las pendientes de TaskCodeTwo, más dos zonas que se solapan. Tiene que reproducir el prototipo: camino crítico de 16,6 h y 77,1/38,6/25,8/19,3 h con 1 a 4 personas.
4. Divergencias con nombre:
   - F1-9: forma de `estimacion` y campo `zonas`.
   - F3-3: `--personas`.
   - F1-16: la «mediana de una ronda» del presupuesto pasa a salir de `estimar`.
   - F1-14: presupuesto de la status line.
5. ADR con la tabla requisito → evidencia (RR-19, RR-12 y los requisitos 1 a 3 de Carlos) y su fila en el índice.

## Alternativas descartadas

- **Recalcular en la status line en cada refresco**: recorre todo el histórico (404 tareas en ToolsaWeb) en el camino caliente de F1-14, y cuesta O(N) en cada refresco cuando el valor solo cambia en las transiciones.
- **Que estime el agente (como la línea del plan de la 0.16) o que se estime con hooks**: gasta tokens, va contra «comandos sobre hooks» y no se puede calibrar. Se mide como referencia, no como modelo.

## Desacuerdos previstos

- **Riesgos: el tiempo real sacado de las transiciones incluye pausas y noches.** Posición: el real de referencia es la línea «Real … sin pausas» de `## Duracion` cuando existe. Por fase solo se usan las marcas y se dice que es una cota alta, sin filtrar a ojo.
- **Riesgos: alcance. Reparto y solape pueden verse como trabajo de F3-3 y F3-11.** Posición: se quedan en este ADR como contrato, porque dependen de la estimación. El código va en F3. Lo decide Carlos (D3).
- **Riesgos: si `zonas` lo escribe el agente, se puede manipular para saltarse el aviso.** Posición: el aviso además cruza con los ficheros reales de la rama (F3-11), y la zona declarada solo sirve para planificar.

## Suposiciones no verificadas

- **Tareas con «Estimada» y «Real»:** unas 26 terminadas en ToolsaWeb y unas 38 en OpenDockTrading. Lo he contado con grep sobre `tareas/04-terminadas`; falta confirmarlo con el parser de F1-9, porque en OpenDockTrading una de las coincidencias está en un fichero de brainstorm.
- **Transiciones:** 162 terminadas en ToolsaWeb y 44 en OpenDockTrading (F1-9, PoC A).
- **Complejidad declarada:** está en todas las de ToolsaWeb y en ninguna de OpenDockTrading.
- **Tamaño del diff y carril:** el tamaño saldría del merge `--no-ff` de cada tarea (`M^1..M^2`) en las copias, y el carril, de la función de la PoC de F1-10.
- **Status line y `estado --seguir` de la 0.16.1:** no he podido ejecutarlos, así que no sé cómo pinta hoy la status line ni qué fases enseña `taskctl estado --seguir`. Habría que mirar `taskctl --help` y la configuración de la status line.
- **Semilla:** no está claro que baste una semilla con pocas muestras (3 ADR de OpenDockTrading); F1-18 ya dijo que no. Hay que ver qué nivel de la cascada da menos error.
- **No cubierto:** casos límite (rol de riesgos), estrategia de pruebas más allá de la PoC (rol de testing) y reglas de negocio (rol de dominio).

## Decisiones para Carlos

- **D1:**
  - D1a (recomendada): el CLI estima y la línea del plan se queda como estimación del agente; se miden las dos.
  - D1b: solo la del CLI.
  - D1c: solo la del plan.
- **D2:**
  - D2a (recomendada): enseñar la mediana y el p75 («≈95 min, hasta 140»).
  - D2b: solo la mediana.
- **D3:**
  - D3a (recomendada): reparto, zonas y precondiciones de `start` como contratos en F1-17, con el código en F3-3, F3-6 y F3-11.
  - D3b: un ADR aparte después de F1-17.
- **D4:**
  - D4a (recomendada): `zonas` como campo declarado en el plan, además del cruce con los ficheros reales.
  - D4b: zonas solo deducidas de la rama.

Rutas:
- `C:\Users\nullcad2025\Documents\sources\Personal\ToolsaWeb\tareas\04-terminadas`
- `C:\Users\nullcad2025\Documents\sources\Personal\OpenDockTrading\tareas\04-terminadas`
- `C:\Users\nullcad2025\Documents\sources\Personal\TaskCodeTwo\docs\adr\F1-9-formato-de-tarea-plan-y-especificacion.md` (campo `estimacion`, `## Duracion`)
