# Salida del rol brainstorm-riesgos — TASK-075 (ronda 1)

(Volcada tal cual por quien orquesta. Por la caída de conexión el rol no leyó F1-2, F1-3, F1-9, F1-14, F1-15 ni F1-16; sus referencias a ellos salen de la salida de riesgos de TASK-074.)

## Riesgos, de mayor a menor daño

- **Falsa garantía de «no se pisan».** El reparto `--personas N` y el aviso de solape solo ven las zonas declaradas.
  - Lo dispara una tarea sin zona, una zona mal declarada o una dependencia implícita. Ya pasó en F1-16: B usó código de A sin declararlo.
  - Consecuencia: dos personas trabajan en paralelo sobre el mismo módulo con el visto bueno del CLI, que es peor que no tener aviso.
  - Mitigación: una tarea sin zona declarada choca con todas y se reparte en serie. El texto dice «aviso», nunca «garantía». En `review` se compara el diff real con la zona declarada.
- **`start` decide sobre un estado viejo.**
  - Lo dispara una predecesora terminada en otra máquina que aún no se ha subido, o la caché local de la rama de estado.
  - Consecuencia: bloquea sin motivo o, peor, deja empezar sobre una base sin integrar.
  - Mitigación: `start` hace fetch de la rama de estado y de `develop` y comprueba que la predecesora tiene su merge en `develop`, no solo su `estado:`. Sin red se niega y explica por qué; nunca permite a ciegas.
- **Aviso de cambios ajenos que llega tarde o no llega.**
  - Lo dispara un compañero que empuja a la rama, o la propia persona que edita en el worktree mientras el agente trabaja.
  - Consecuencia: se revisa X y se integra Y, como ya pasó con la 0.16.
  - Mitigación: el mismo CAS de F1-16, con el head y el `status --porcelain` esperados en cada transición. La status line solo enseña el aviso; la decisión no depende de ella.
- **La métrica de entrenamiento está sucia.** Lo verifiqué en dos repos:
  - En ODT, la línea «Real (… sin pausas)» de TASK-096 da 414 min, pero sus Transiciones muestran `start` a las 01:51 y `review` a las 08:15, con una noche dentro.
  - En ToolsaWeb, TASK-296 tiene una `pausa` sin reanudar seguida de `finish`, y dos `plan` (se replanificó).
  - Consecuencia: las colas falsas dominan la media de error.
  - Mitigación: recalcular lo real desde Transiciones, cortar los huecos mayores de un umbral (por ejemplo, 45 min sin commit ni coste) y declarar la regla en el ADR.
- **No hay datos por fase para lo que pide F1-17.** Lo verifiqué: Transiciones solo tiene `plan`, `approve`, `start`, `review`×n y `finish`.
  - Falta el inicio del diseño (`creado` es solo una fecha), y `plan`→`approve` es espera humana, no trabajo.
  - Consecuencia: la estimación de la fase de diseño no se puede validar con el histórico.
  - Mitigación: el ADR define las fases medibles (implementación = start→1.ª review; rondas = review→review; cierre = última review→finish) y deja el diseño «sin calibrar» hasta tener datos de la 2.0.
- **Pocas muestras con estimación previa, y en formato libre.** Lo verifiqué:
  - ToolsaWeb tiene 27 de 163 terminadas con `## Duracion`; ODT, 38 de 48.
  - En ODT, 6 dicen «— (no se entiende: 240 min …)» y 1 dice «el plan no la dice».
  - En los 31 pares legibles de ODT, la razón real/estimada tiene mediana ≈0,53, va de 0,14 a 4,14, y 26 de 31 quedan por debajo de 1.
  - Consecuencia: la 0.16 sobreestima ≈2×, y un modelo entrenado sobre esto hereda el sesgo o se ajusta a 30 puntos.
  - Mitigación: la línea base no es la estimación de la 0.16, sino lo real de tareas parecidas sobre las 163+48 con Transiciones, con validación fuera de muestra (abajo).
- **Lectura y escritura concurrentes de `tarea.md`.**
  - Lo disparan la status line, que lee mientras `taskctl` reescribe, y dos terminales que añaden `## Coste` a la vez.
  - Consecuencia: la línea sale vacía o rota, y se pierde una fila de coste, con lo que los tokens quedan infravalorados.
  - Mitigación:
    - Escritura atómica: fichero temporal + rename, con reintento en Windows si el lector tiene el fichero abierto.
    - La status line lee un `estado.json` derivado. Si falla el parseo, enseña el último valor bueno con su antigüedad.
    - El coste se añade por proceso, con CAS o con un fichero por fuente.

## Cómo medir el error con honestidad (lo que pido al ADR)

- **Validación temporal:** entrenar con las tareas anteriores a una fecha T y medir con las posteriores (rolling origin), nunca dentro de la muestra. Por repo y cruzado: entrenar con ToolsaWeb y medir con ODT.
- **Métrica:** error relativo mediano y p80/p90 de real/estimado, por fase y en total, más el porcentaje dentro de ±50 %. Si el criterio exige «error medio», se da también, pero no como cifra principal.
- **Líneas base obligatorias:** la «Estimada» de la 0.16 y la mediana ingenua por complejidad. El modelo vale si gana a las dos fuera de muestra.
- **Tareas troceadas** (la Fase 1 encadenada, remates como TASK-404 sobre TASK-212): se agrupan para que no cuenten como muestras independientes.
- **Hora de fin del lote:** sumar medianas sale optimista porque las colas van a la derecha; hay que dar un intervalo con p50 y p80.

## Puntos sin retorno

- **Reescribir `## Duracion` o Transiciones de tareas ya terminadas al migrar:** se pierde la muestra original de calibración. Lo que haga falta se deriva aparte y nunca se tocan.
- **Publicar en el informe por versión una calibración con una métrica dentro de muestra:** la confianza que crea no se retira aunque luego se corrija.

## Descartado a propósito

- **Desfase de relojes entre máquinas:** todas las marcas que he comprobado van en UTC (`Z`). Basta con tratar las duraciones negativas como dato inválido.
- **Coste de recalcular:** hay menos de 300 `tarea.md`. Recalcularlo todo en cada transición es trivial y evita una caché que se quede vieja.
- **Formatos distintos entre ODT y ToolsaWeb:** tienen las mismas secciones y columnas (conviven `version` 0.14.0 y 0.16.1). Basta un parser tolerante que cuente los descartes.

## Desacuerdos previstos

- **Con la heurística «simple, 1 rol»:** no lo es. Toca `start` (bloquear), interferencia y trabajo en grupo, y es carril de puertas y seguridad. Además, los requisitos (2) y (3) de Carlos se solapan con F3-11 y F9-4. El ADR debe decir qué decide aquí y qué deja a esas tareas.
- **Con arquitectura, si propone estimar el diseño por fase o validar con «error medio» dentro de muestra:** en contra, por las dos razones de arriba.
- **Con cualquier enfoque que trate la zona declarada como fuente de verdad del solape:** es un aviso, no una garantía.

## Suposiciones no verificadas

- **«Sin pausas» en la 0.16:** por TASK-096 parece que no descuenta los huecos. Hay que leer el código de `taskctl finish`.
- **Status line de la 0.16:** que lee un fichero y no lanza `taskctl`. Hay que mirar la configuración `statusLine` del plugin.
- **Rename atómico en Windows:** que falla si la status line tiene el fichero abierto. Hay que probarlo en la PoC.
- **`pausa` sin `reanudar`:** que significa «pausa hasta el finish». Hay que revisar las otras tareas con `pausa` en ToolsaWeb.
- **Copia de `tarea.md` que lee el dashboard:** la de la rama de estado o la del worktree, según F1-1 y F1-2.

## Fuentes

- La petición del plan.
- PLAN-TASKCODE-2.md: RR-19, F1-17, F3-11/12/13, F4-9 y F8-7.
- AMPLIACION-2026-11-14.md §5 quater.
- F1-10: rutas de la PoC.
- ToolsaWeb `tareas/04-terminadas`: TASK-404, TASK-296 y los recuentos.
- OpenDockTrading `tareas/04-terminadas`: las 38 `## Duracion` y TASK-096.

No modifiqué ningún fichero.
