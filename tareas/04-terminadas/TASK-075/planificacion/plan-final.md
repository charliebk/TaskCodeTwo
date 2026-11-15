# Plan — TASK-075: F1-17 ADR progreso en vivo y estimacion de tiempo

(Plan del unificador. Se lanzaron dos roles, arquitectura y riesgos: es la regla de F1-15 para tareas de puertas y seguridad, aunque la peticion generada por la 0.16 diga «heuristica simple, 1 rol, sin unificador». [rie] discrepa expresamente de esa heuristica: la tarea toca `start` (bloquear), interferencia y trabajo en grupo. Testing y dominio no se lanzaron. Cada afirmacion va firmada con el rol del que sale: [arq] o [rie]. Las marcadas [unif] son contrastes del unificador entre las dos salidas o contra el enunciado, no un punto de vista nuevo. Aviso: por una caida de conexion [rie] no leyo F1-2, F1-3, F1-9, F1-14, F1-15 ni F1-16; sus referencias a ellos salen de la salida de riesgos de TASK-074.)

## Enfoque propuesto

Duracion estimada: 300 min

Cifra de [arq]: F1-16 estimo 180 y le costo 594, y esta tarea tiene la misma forma (ADR con PoC sobre dos repos) y mas consumidores. [rie] no estimo. [unif] Contraste: [rie] midio que en OpenDockTrading la 0.16 sobreestima (mediana real/estimada ≈0,53), mientras que en los ADR de TaskCodeTwo el error va al reves (F1-16, ≈3,3×); la cifra se deja como la dio [arq]. **No se trocea.**

### Decisiones ya tomadas por la persona (Carlos, 2026-11-14)

1. **RR-19 imprescindible**: fases que faltan y tiempo estimado por tarea, visibles en el terminal de trabajo (status line y `status --seguir`), escritos por el CLI sin tokens; en un lote, tareas que faltan y hora estimada de fin; al cerrar, estimado frente a real por fase, tokens, rondas y hallazgos, con calibracion en el informe por version.
2. **Requisito (1) de Carlos**: comando de resumen tipo dashboard (terminadas/pendientes, tiempo y tokens parcial y total, estimacion para terminar parcial y total), determinista, que solo lee `tarea.md`, sin hooks ni tokens.
3. **Requisito (2)**: tareas en paralelo en terminales distintas: `start` se niega si una predecesora no esta terminada y avisa si otra tarea en curso toca los mismos ficheros; aviso de cambios ajenos.
4. **Requisito (3)**: reparto por camino critico con `--personas N` y zonas de ficheros declaradas.
5. **Seguridad y trabajo en grupo imprescindibles**; comandos deterministas sobre hooks; nombres `tc2`/`taskcodetwo`; CLI en Rust (F1-14); fidelidad sobre coste; sin trocear.

### Forma (la de [arq], con las mitigaciones de [rie] dentro)

1. **Lector `historico(tareas) -> Vec<Muestra>`** [arq], unico sitio que interpreta `## Transiciones`, `## Duracion` y `## Coste`, encima del parser de F1-9; parser tolerante que cuenta los descartes (conviven 0.14.0 y 0.16.1) [rie]. **El real se recalcula desde Transiciones con corte de huecos** mayores de un umbral sin commit ni coste, regla declarada en el ADR [rie] (desacuerdo 2). Duraciones negativas = dato invalido [rie]. Las tareas troceadas (Fase 1 encadenada, remates como TASK-404 sobre TASK-212) se agrupan y no cuentan como muestras independientes [rie]. Nunca se reescriben `## Duracion` ni Transiciones de tareas terminadas; lo derivado va aparte [rie].
2. **Nucleo puro `estimar(tarea, &[Muestra]) -> Estimacion` y `restante(est, transiciones, ahora)`** [arq]: mediana y percentil alto por fase en cascada carril+complejidad+tamano → carril → repo → semilla (F1-18), guardando n y nivel usado [arq]. Carril y tamano se piden a `carril` de F1-10 [arq]. Fases medibles: implementacion = start→1.a review, rondas = review→review, cierre = ultima review→finish; **diseno «sin calibrar»** hasta tener datos de la 2.0 [rie] (desacuerdo 3).
3. **Cuando se recalcula** [arq]: en approve, en cada transicion y en finish (en finish se registra el error). Recalcular todo en cada transicion es trivial con menos de 300 `tarea.md` [rie]. El resultado va al campo `estimacion` de F1-9 en el mismo commit de estado (F1-2) [arq].
4. **Vistas** [arq]: `status`, `status --seguir`, status line (F3-5, F3-12) y `tc2 resumen` nuevo (O(N), solo a peticion). **Status line O(1)**: lee un valor derivado, no recorre el historico, y si el parseo falla ensena el ultimo valor bueno con su antiguedad [rie] (desacuerdo 6, D5). Hora de fin del lote como intervalo p50/p80, no suma de medianas [rie].
5. **Escritura atomica y coste concurrente** [rie]: temporal + rename con reintento en Windows si el lector tiene el fichero abierto; `## Coste` se anade por proceso con CAS o un fichero por fuente. [arq] no trato la concurrencia.
6. **`repartir(dag, duraciones, zonas, n) -> Reparto`** para `next --personas N` (F3-3) [arq]: listas con prioridad al trabajo restante, sin juntar zonas solapadas; con N=1 devuelve el camino critico como comprobacion [arq]. **Zonas como aviso, nunca garantia** [rie]: tarea sin zona = choca con todas y va en serie [rie]; zona declarada en el plan solo para planificar, cruzada con el diff real de la rama en `start`/`review` (F3-11) [arq/rie] (desacuerdo 4, D4).
7. **Precondiciones de `start`** (regla en el ADR, codigo en F3-6) [arq]: `start` hace fetch de la rama de estado y de `develop` y exige que la predecesora tenga su merge en `develop`, no solo su `estado:`; sin red se niega y explica por que, nunca deja pasar a ciegas [rie] (desacuerdo 5). Aviso de cambios ajenos con el CAS de F1-16 (head y `status --porcelain` esperados en cada transicion); la status line solo ensena el aviso, la decision no depende de ella [rie].

**Orden** [arq]: contratos → PoC 1 → PoC 2 → divergencias con nombre (F1-9: forma de `estimacion` y campo `zonas`; F3-3: `--personas`; F1-16: la «mediana de una ronda» sale de `estimar`; F1-14: presupuesto de la status line) → ADR con tabla requisito → evidencia y fila del indice. **Se crea**: `docs/adr/F1-17-...md`, su fila en el indice y las PoC con su log. **No se toca**: otros ADR, el plan, los pilotos ni la 0.16.

**Descartado** [arq]: recalcular en la status line en cada refresco (O(N) en el camino caliente; 404 tareas en ToolsaWeb); que estime el agente o un hook (tokens, no calibrable): se mide como referencia, no como modelo.

### Desacuerdos entre roles (convertidos en D#)

1. **Medicion del error**: [arq] estima cada tarea con las terminadas antes que ella y da el error medio en minutos y porcentual con mediana y p75, contra la «Estimada» 0.16 y la mediana global / [rie] pide rolling origin por repo **y cruzado** (entrenar en ToolsaWeb, medir en ODT), error relativo mediano y p80/p90 por fase y total, % dentro de ±50 %, y el «error medio» solo como cifra secundaria; lineas base obligatorias la 0.16 y la mediana ingenua por complejidad, y el modelo vale solo si gana a las dos fuera de muestra. Gana [rie] (D8a), dando tambien el error medio que pide el criterio literal: las colas medidas por [rie] (razon 0,14–4,14) dominan una media, y publicar una calibracion dentro de muestra es punto sin retorno [rie].
2. **Real de referencia**: [arq] usa la linea «Real … sin pausas» de `## Duracion` cuando existe, sin filtrar a ojo / [rie] verifico que esa linea esta sucia (ODT TASK-096: 414 min con una noche entre `start` 01:51 y `review` 08:15; ToolsaWeb TASK-296: `pausa` sin reanudar y dos `plan`) y pide recalcular desde Transiciones con corte de huecos y regla declarada. Gana [rie]: trae evidencia en dos repos; [arq] no comprobo la linea. Umbral en D6.
3. **Fase de diseno**: [arq] incluye `diseno` en `por_fase` / [rie] verifico que no hay inicio de diseno (`creado` es solo fecha) y que plan→approve es espera humana. Gana [rie] (D7a): no se puede validar con el historico.
4. **Zonas**: [arq] zona declarada para planificar mas cruce con los ficheros reales / [rie] anade que sin zona = choca con todas y que el texto diga «aviso», nunca «garantia» (F1-16: B uso codigo de A sin declararlo). Compatibles salvo la regla de la tarea sin zona, que [arq] no da; gana la union (D4a).
5. **`start` sobre estado viejo**: [arq] solo fija «negarse si la predecesora no esta terminada» / [rie] exige fetch y merge en `develop`, y negarse sin red. Gana [rie]: con solo `estado:` se arranca sobre una base sin integrar. Va en D3a.
6. **Fuente de la status line**: [arq] lee el campo `estimacion` del `tarea.md` / [rie] lee un `estado.json` derivado con ultimo valor bueno, por la lectura concurrente. Ambos O(1). Recomendada la de [rie] (D5a), pero anade un fichero que [arq] no contempla en F1-9/F1-2 y [rie] no leyo esos ADR: decide Carlos.
7. **Percentil mostrado**: [arq] mediana y p75 / [rie] p80 (y p90 en la medicion) por las colas a la derecha. Recomendada [rie] (D2a) por los datos medidos.

## Lo que los roles no cubrieron

- **Testing no se lanzo**: nadie diseno pruebas mas alla de las PoC; las mutaciones de control del plan de pruebas son [unif] por peticion de quien orquesta.
- **Dominio no se lanzo**: nadie reviso que es una «tarea parecida» ni las fases que el programador espera ver.
- [unif] **Hallazgos** al cerrar (RR-19 pide estimado/real, tokens, rondas y hallazgos): [arq] mete rondas y tokens en `Muestra`; ningun rol trato los hallazgos. El ADR los anade a las metricas finales.
- [unif] ODT no tiene complejidad declarada [arq], asi que la linea base «mediana ingenua por complejidad» de [rie] no se puede calcular alli. El ADR dice que nivel la sustituye en ODT.
- [unif] La salida de [rie] se baso en lo que TASK-074 dijo de F1-2/F1-9/F1-14/F1-16; sus mitigaciones sobre esos contratos no estan contrastadas contra los ADR.
- [unif] Nadie vio como pinta hoy la status line ni `taskctl estado --seguir` de la 0.16.1 [arq/rie], que el enunciado pone como base.
- Salidas que faltaron: ninguna de las dos lanzadas. Hubo desacuerdo real, asi que no salta la alarma de consenso.

## Riesgos aceptados y que los contiene

- Falsa garantia de «no se pisan» por zonas mal declaradas o dependencias implicitas [rie]: «es peor que no tener aviso». Lo contienen sin zona = choca con todas, el cruce con el diff real y el texto «aviso».
- `start` que bloquea sin red [rie]. Lo contiene que explique el motivo; nunca deja pasar a ciegas.
- Aviso de cambios ajenos que llega tarde («se revisa X y se integra Y») [rie]. Lo contiene el CAS en cada transicion, no la status line.
- Pocas muestras con estimacion previa y en formato libre (ToolsaWeb 27 de 163 con `## Duracion`; ODT 38 de 48, 7 ilegibles) [rie]. Lo contiene usar como base lo real de las 163+48 con Transiciones.
- Semilla insuficiente para TaskCodeTwo (3 ADR de ODT; F1-18 ya dijo que no) [arq]. Lo contiene medir que nivel de la cascada da menos error.
- Puntos sin retorno [rie]: reescribir historico terminado al migrar y publicar una calibracion dentro de muestra. Los contienen no tocar nunca esas secciones y D8a.

## Plan de pruebas

Forma de las PoC del H1: copias `git clone --no-hardlinks` de ToolsaWeb y OpenDockTrading (originales solo lectura) en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-075\` [arq], log determinista con tiempo de reloj a stderr, tablas del ADR comprobadas contra el log y mutaciones de control. La carpeta se borra al terminar.

- **Criterio 1 → PoC 1, estimacion fuera de muestra**: rolling origin por repo y cruzado [rie] (cada tarea con solo las anteriores [arq]); mediana, p80, p90 y % dentro de ±50 % por fase y total, mas el error medio del criterio [rie]; contra la «Estimada» 0.16 y la mediana ingenua [rie] y la mediana global [arq]. Real recalculado con corte de huecos, con TASK-096 y TASK-296 como casos de control [rie]; troceadas agrupadas [rie]; descartes contados.
- **Criterio 2 → PoC 2, `resumen` y `repartir`** sobre las pendientes de TaskCodeTwo mas dos zonas solapadas [arq]: reproduce el prototipo, camino critico 16,6 h y 77,1 / 38,6 / 25,8 / 19,3 h con 1 a 4 personas [arq]; una tarea sin zona va en serie [rie].
- **Status line y concurrencia** [rie]: rename atomico en Windows con el fichero abierto por un lector; derivado corrupto → ultimo valor bueno con antiguedad; dos procesos anadiendo coste sin perder filas.
- **Mutaciones de control** [unif, por peticion; cada una sobre un riesgo de [rie]]: quitar el corte de huecos (TASK-096 debe cambiar), estimar dentro de muestra (el error debe bajar: detecta fuga), quitar «sin zona = choca» (el reparto debe paralelizar mal), alterar una duracion del camino critico (16,6 h debe moverse), predecesora sin merge en `develop` (`start` debe negarse). Cada mutacion tiene que hacer fallar su comprobacion.
- **Tabla requisito → decision → evidencia**: RR-19, RR-12, requisitos (1)-(3) de Carlos, cada parte de F1-17 y las divergencias con nombre; fila del indice, `taskctl doctor` sin errores, temporales borrados.

## Lo que necesita decision de una persona

Este plan no vale hasta que Carlos lo apruebe con `taskctl approve`. La recomendada va marcada como (a).

Respuestas (Carlos, 2026-11-15): **D1a..D8a**. D3a (start se niega sin red) y D4a (zonas declaradas, sin zona = choca con todas, siempre aviso) se le preguntaron expresamente por cambiar su forma de trabajar y respondio «Apruebo las dos».

- **D1. Quien estima**: **(a)** el CLI estima y la linea del plan queda como estimacion del agente; se miden las dos [arq]; (b) solo el CLI; (c) solo el plan.
- **D2. Que se muestra**: **(a)** mediana y p80 [rie: colas medidas]; (b) mediana y p75 [arq]; (c) solo mediana [arq].
- **D3. Alcance**: **(a)** reparto, zonas y precondiciones de `start` (fetch, merge en `develop`, negarse sin red [rie]) como contratos en F1-17, codigo en F3-3, F3-6 y F3-11 [arq]; (b) ADR aparte despues de F1-17 [arq]. [rie] pide que el ADR diga que decide aqui y que deja a F3-11 y F9-4.
- **D4. Zonas**: **(a)** campo declarado en el plan + cruce con el diff real + sin zona = choca con todas, siempre «aviso» [arq+rie]; (b) solo deducidas de la rama [arq].
- **D5. Fuente de la status line**: **(a)** `estado.json` derivado con ultimo valor bueno [rie]; (b) campo `estimacion` del `tarea.md` [arq]. (a) anade un fichero no previsto en F1-9/F1-2.
- **D6. Corte de huecos**: **(a)** recalcular desde Transiciones cortando huecos de mas de 45 min sin commit ni coste (cifra «por ejemplo» de [rie]), declarado en el ADR; (b) la linea «sin pausas» de la 0.16 [arq].
- **D7. Fase de diseno**: **(a)** «sin calibrar» hasta tener datos de la 2.0 [rie]; (b) estimarla con plan→approve como cota alta [arq].
- **D8. Cifra principal del error**: **(a)** mediana/p80/p90 fuera de muestra y el error medio tambien, como secundario [rie]; (b) el error medio como cifra principal, lectura literal del criterio [arq].

## Suposiciones no verificadas

- Recuentos: ~26 ToolsaWeb y ~38 ODT con «Estimada» y «Real» por grep [arq] frente a 27 de 163 y 38 de 48 [rie]; confirmar con el parser de F1-9 (una coincidencia de ODT esta en un brainstorm [arq]).
- Complejidad declarada en todas las de ToolsaWeb y en ninguna de ODT [arq].
- Tamano desde el merge `--no-ff` (`M^1..M^2`) y carril desde la PoC de F1-10 [arq].
- Que «sin pausas» de la 0.16 no descuenta huecos [rie]: leer `taskctl finish`. Que `pausa` sin `reanudar` = pausa hasta el finish [rie].
- Que la status line de la 0.16 lee un fichero y no lanza `taskctl` [rie]; nadie la ejecuto [arq].
- Que el rename atomico falla en Windows con el lector abierto [rie]: se prueba en la PoC.
- Que copia de `tarea.md` lee el dashboard, la de la rama de estado o la del worktree (F1-1, F1-2) [rie].
- Que todas las marcas van en UTC (`Z`) [rie, comprobado en las que miro].
