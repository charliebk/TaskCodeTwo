# Plan — TASK-074: F1-16 ADR lotes desatendidos

(Plan del unificador. Se lanzaron dos roles, arquitectura y riesgos: es la regla de F1-15 para tareas de puertas y seguridad, aunque la peticion generada por la 0.16 diga «1 rol, sin unificador». Testing y dominio no se lanzaron. Cada afirmacion va firmada con el rol del que sale: [arq] o [rie]. Las marcadas [unif] son contrastes del unificador contra el enunciado, no un punto de vista nuevo.)

## Enfoque propuesto

Duracion estimada: 180 min

Entre F1-15 (150) y F1-14 (210): un ADR con un nucleo puro, cinco piezas, siete divergencias con nombre y una PoC con un lote real de ToolsaWeb y nueve inyecciones. **No se trocea.**

### Decisiones ya tomadas por la persona (Carlos, 2026-11-14)

1. **RR-18 imprescindible**: un comando determinista del CLI (no el agente), rama `lote/<nombre>` desde `develop`, solo carriles permitidos, apartar la tarea en rojo o con CRITICO/IMPORTANTE sin resolver junto con sus dependientes, nunca integrar en rojo, presupuesto de tiempo y tokens, `--reanudar` sin repetir, informe de la manana y su equivalente en GitLab.
2. **Seguridad y trabajo en grupo imprescindibles**, con el caso real de Carlos: escribir en el prompt mientras una tarea trabajaba cambio el repo y provoco rondas extra. Los cambios ajenos se impiden y se detectan antes de revisar y antes de cerrar o integrar.
3. **Comandos deterministas sobre hooks**; nombres `tc2`, `taskcodetwo` y rama de estado `taskcodetwo/tareas`; CLI en Rust (F1-14).
4. **Sin trocear.** PoC sobre una copia de ToolsaWeb (el original solo se lee) en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-074\`, que se borra al terminar; GitHub solo lectura; sin instalar toolchains.

### Forma (la de [arq], con las mitigaciones de [rie] dentro)

1. **`tc2 lote` y su nucleo puro** [arq]: `planificar(tareas, deps, politica) -> Orden` (orden por dependencias, empate por ID, solo lo que admite la columna «Lote» de F1-10) y `siguiente_paso(...) -> Paso`. Reutiliza `politica`/`accion` (F1-10), `seleccionar` (F1-15) y las operaciones de F1-4; no decide carriles por su cuenta.
2. **Manifiesto fijado al lanzar** [rie]: lista, hash de cada `tarea.md`, carril y presupuesto. Si un hash cambia, la tarea se aparta; si el carril efectivo del diff sube a riesgo alto (`.taskcode/`, workflows), tambien. Reserva del lote en la rama de estado con el CAS de F1-2 [rie]; cada tarea tomada se marca `asignado` para que un companero no haga `start` sobre ella [arq].
3. **Estado** [arq/rie, D5]: diario `lotes/<nombre>.yml` (extiende `commit-estado` de F1-2) con SHA, resultado, tokens y tiempo [arq]. La fuente de verdad de `--reanudar` es Git: la cadena de primer padre de `lote/*` con el trailer `Tarea:`; cada paso tiene una precondicion comprobable y nunca se guarda un «paso actual» [rie].
4. **Operacion `integrar-en-lote`** (nueva en la lista cerrada de F1-4) [arq]: PR de la tarea contra `lote/<nombre>`, que la App del agente mergea por API solo con `estado-puertas` en verde. **CAS** [rie]: se mergea solo si el head de la tarea es igual a `ultimo_commit_revisado` y `lote/*` sigue en el SHA anotado al empezar. `merge-pr` sigue siendo solo de la persona, una vez, en el PR del lote contra `develop`.
5. **Impedir y detectar** (D4): agente lanzado sin stdin sobre `.taskcode/worktrees/`, nunca sobre el principal [arq]; ruleset `taskcodetwo-lote` sobre `refs/heads/lote/**` sin bypass (F1-5) y en GitLab rama protegida «push: nadie, merge: bot, pipelines must succeed» [arq]. La guarda `comprobar_procedencia` corre antes de `review` y antes de integrar, con `status --porcelain` limpio y HEAD esperado [arq/rie]. Lo ajeno en la rama de tarea la aparta como «interferida», sin re-revisar en bucle [rie]; lo ajeno en `lote/*` para el lote [arq]. Antes de abrir el PR final, el CLI exige que el primer padre de `lote/*` sean solo sus merges, con trailer y SHA verificado [rie, D3].
6. **Fallos y presupuesto** [rie]: «el entorno no responde» (limite de uso, red) para limpio y reanudable sin apartar, y una ronda cortada se repite entera, nunca da verde y sus tokens cuentan. Lo rojo se aparta con su cierre transitivo sobre las dependencias declaradas (tambien las de fuera del lote, y las parciales cuentan); la rama apartada se marca con su base de lote. El presupuesto, por lote y por tarea, lo mide el proceso del CLI; no se empieza una ronda si lo que queda es menos que la mediana de una ronda. Tope de rondas: D6.
7. **Final** [rie]: de noche no se resuelve ningun conflicto; `merge-tree` contra `origin/develop`, los conflictos van al informe y la verificacion completa del arbol final es obligatoria. **`informe(diario) -> Markdown`**, funcion pura [arq]: terminadas, apartadas con motivo y dependientes, interferidas, hallazgos sin corregir, tokens y tiempo; va en la descripcion del PR del lote.

**Orden** [arq]: contratos (diario, nucleo, regla de lote sobre las paradas de F1-10), divergencias con nombre (F1-6 decision 4, F1-3, F1-4, F1-5, F1-7, F1-13, F1-8 A16), PoC con inyecciones, informe de la PoC y tabla requisito → evidencia. **Se crea**: `docs/adr/F1-16-lotes-desatendidos.md`, su fila en el indice y la PoC con su log. **No se toca**: otros ADR, el plan, los pilotos ni la 0.16.

**Descartado** [arq]: merge local `--no-ff` sin PR (F1-3 retiro los merges locales; ninguna puerta de plataforma), orquestar desde el agente o con hooks, integrar cada tarea en `develop`. Coste: un CI por tarea y uno final, en serie; paralelizar es de F9-4.

### Desacuerdos entre roles (convertidos en D#)

- **PR final**: [arq]/P13 proponen un PR unico que se aprueba entero / [rie] pide nombrar el rechazo parcial como punto sin retorno, porque las tareas siguientes ya vieron el codigo de la rechazada. Gana [rie] (D1): no contradice P13 y deja escrito un procedimiento que, si falta, se improvisa.
- **App del agente mergeando en `lote/*`**: [arq] dice que `lote/*` no es la puerta (la puerta es el PR final) y que A5 queda igual que fuera del lote / [rie] dice que la mitigacion de F1-8 A16 depende de A5 y que el lote nocturno no se habilita mientras A5 siga abierto, salvo una sesion sin el token de la persona. Gana [rie] (D2): de noche no mira nadie y A5 es un hueco abierto, no una suposicion.
- **Quien impide lo rojo en `lote/*`**: [arq] confia en el ruleset sin bypass / [rie] avisa de que en repos privados gratuitos (ToolsaWeb, OpenDockTrading) el ruleset no se impone (F1-5 D2a, 403), y en GitLab Free tampoco. Gana [rie] (D3): la comprobacion del primer padre por el CLI es obligatoria siempre; el ruleset se queda donde la plataforma lo imponga.
- **Interferencia**: [arq] detecta y lanza sin terminal, sin bloquear el sistema de ficheros («en Windows no hay bloqueo portable») / [rie] anade el CAS en la integracion, que si impide. Gana la union que pidio Carlos, impedir y detectar (D4); nadie propone bloquear el sistema de ficheros.
- **Fuente de verdad de `--reanudar`**: [arq] usa el diario contrastado con las ramas del remoto / [rie] dice que manda Git, nunca un estado guardado. Gana [rie] (D5): el corte entre el merge y la escritura del diario es justo el caso que se rompe.
- **Tope de rondas**: [arq] hace rondas extra automaticas hasta `tope_rondas` y despues aparta, sin preguntar, y deja que lo decida Carlos / [rie] pone un tope fijo sin ronda extra automatica; con datos de TASK-006, 009, 011, 012 y 013, la ronda extra la decidio siempre una persona, y TASK-011 gasto ~21 M tokens. Recomendada la de [rie] (D6), pero choca con «fidelidad sobre coste»: es decision de Carlos.

## Lo que el rol no cubrio

- **Testing no se lanzo**: nadie discutio la cobertura del plan de pruebas; las inyecciones salen de la lista de [rie] y del orden de [arq].
- **Dominio no se lanzo**: que carriles entran en un lote se toma de la columna «Lote» de F1-10 sin revisarla; nadie reviso las reglas de negocio de apartar ni de presupuesto.
- [unif] §5 bis dice que una tarea con un `[DECIDE CARLOS]` abierto «no entra, o para el lote». Ningun rol lo trato: el ADR elige una de las dos.
- [unif] «Aceptando las recomendaciones en todas las preguntas»: [arq] exige un `approve` firmado antes de lanzar, pero nadie dice quien responde las D# de un plan dentro del lote. Hueco abierto: queda en D7.
- [unif] El progreso en vivo del lote (F4-9, F1-17) queda fuera; el informe es solo final.
- Salidas que faltaron: ninguna (llegaron las dos que se lanzaron). Hubo desacuerdo real, asi que no salta la alarma de consenso.

## Riesgos aceptados y que los contiene

- Rechazar una tarea dentro del PR final obliga a revertirla en `lote/*` y a volver a verificar todo el lote [rie]. Lo contiene el procedimiento de D1.
- Escribir en la rama de estado no tiene vuelta atras (F1-8 A4); un estado falso por inyeccion solo se corrige hacia delante [rie]. Lo contienen el manifiesto con hashes y que Git mande en `--reanudar`.
- Apartados en cascada en cadenas largas y dependencias implicitas no declaradas [rie]. Lo contienen el cierre transitivo y la medida del porcentaje en la PoC.
- Sin ruleset impuesto en privados gratuitos ni en GitLab Free [rie]. Lo contiene la comprobacion de procedencia antes del PR final (D3).
- Coste en serie: N × (agente + CI) [arq]. Lo contienen el presupuesto por tarea y no empezar una ronda sin margen [rie].
- [unif] La PoC usa un remoto bare local y una plataforma simulada [arq], asi que ni el ruleset ni el merge por API con SHA esperado se prueban contra GitHub. Lo contiene dejarlo como suposicion para F2-6.

## Plan de pruebas

PoC en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-074\` sobre una copia `git clone --no-hardlinks` de ToolsaWeb, con un remoto bare local [arq]. Lo que hace el agente se simula repitiendo el diff real de cada merge (`M^1..M^2`) y las rondas salen de su `tarea.md` [arq]. Log determinista y tiempo de reloj a stderr.

- **Criterio 1 → (a) lote limpio**: serie de documentacion TASK-173..TASK-191 de la PoC A de F1-10 [arq], mas dos o tres tareas de codigo con dependencias. Se demuestra que el arbol final de `lote/*` es igual al de `develop` real tras la serie [arq], que el primer padre solo tiene merges con trailer [rie] y que el informe sale del diario [arq].
- **Criterio 1 → (b) inyecciones de fallo**, cada una con su resultado esperado en el log:
  - Verificacion en rojo: la tarea y sus dependientes se apartan y el resto sigue [arq/rie].
  - Corte en cada frontera entre pasos: `--reanudar` no repite nada [rie].
  - Presupuesto agotado: para limpio [arq].
  - Limite de uso: para sin apartar y repite la ronda entera [rie].
  - Develop que avanza: `merge-tree` lleva los conflictos al informe [rie].
  - Se mide ademas el porcentaje apartado en cascada [rie].
- **Criterio 1 → (c) inyecciones de interferencia** (el caso de Carlos):
  - Commit ajeno en el worktree, antes de `review`: se aparta como interferida [arq/rie].
  - Commit ajeno en `lote/*`: el lote para [arq].
  - Push directo a `lote/*`: lo detecta la comprobacion de primer padre [rie].
  - Hash de `tarea.md` cambiado: la tarea se aparta [rie].
  - Head movido entre la revision y la integracion: el CAS rechaza el merge [rie].
  - Son mas de tres pruebas porque cada inyeccion es una frontera distinta que pidio [rie].
- **Criterio 2 → tabla requisito → decision → evidencia**: cubre RR-18, RF-16, cada parte de F1-16 (rama, carriles, que para y que aparta, presupuesto, reanudacion, informe, GitLab) y las siete divergencias con nombre. La fila del indice resuelve, `taskctl doctor` sale sin errores y los temporales quedan borrados.

## Lo que necesita decision de una persona

Este plan no vale hasta que Carlos lo apruebe con `taskctl approve`. La recomendada va marcada; es la mas fiel a los datos y a la seguridad.

Respuestas (Carlos, 2026-11-14): **D1a..D8a**. D2a, D6a y D7a se le preguntaron expresamente por cambiar su forma de trabajar (lote nocturno sin activar mientras A5 siga abierto; sin ronda extra automatica; solo tareas con plan aprobado y firmado) y respondio «Apruebo».

- **D1. PR final**: **(a, recomendada)** PR unico, con el rechazo parcial escrito como riesgo aceptado y su procedimiento (revertir en `lote/*` y verificar el lote completo) [rie]; (b) todo o nada sin procedimiento [arq/P13].
- **D2. Merge en `lote/*` por la App del agente**: **(a, recomendada)** `integrar-en-lote` con CAS, y el lote nocturno sin habilitar mientras F1-8 A5 siga abierto, salvo una sesion sin el token de la persona [rie]; (b) habilitado ya, con A5 igual que fuera del lote [arq].
- **D3. Nunca rojo en `lote/*`**: **(a, recomendada)** comprobacion del CLI sobre el primer padre, siempre, y ruleset donde se imponga [rie]; (b) solo ruleset y rama protegida [arq].
- **D4. Interferencia**: **(a, recomendada)** impedir lo portable (sin stdin, worktree aislado, CAS, ruleset, `asignado`) y detectar el resto antes de `review` y de integrar; lo ajeno en la rama de tarea la aparta, lo ajeno en `lote/*` para el lote [arq+rie]; (b) solo detectar [arq]; (c) bloquear el sistema de ficheros (nadie la propone: no es portable).
- **D5. `--reanudar`**: **(a, recomendada)** manda Git (primer padre con trailer) y el diario es el registro [rie]; (b) manda el diario contrastado con el remoto [arq].
- **D6. Tope de rondas**: **(a, recomendada por datos)** tope fijo, sin ronda extra automatica; el mismo hallazgo reaparecido aparta la tarea y la ronda extra la decide Carlos por la manana [rie]; (b) ronda extra automatica hasta `tope_rondas` y despues apartar [arq]. Choca con «fidelidad sobre coste»; con (a) el IMPORTANTE se sigue corrigiendo, pero por decision de una persona.
- **D7. Preguntas del plan dentro del lote** (hueco [unif]): **(a, recomendada)** solo entran tareas con el plan ya aprobado y firmado [arq]; (b) el CLI responde la recomendada de cada D# (lo que hoy se hace por instruccion). Ningun rol la defendio entera.
- **D8. Lote de la PoC**: **(a, recomendada)** TASK-173..TASK-191 mas dos o tres de codigo [arq], confirmado con `git log` en la copia; (b) otra serie que elija Carlos.

## Suposiciones no verificadas

- Que el agente sin terminal (`claude -p --output-format json`) devuelve tokens por ejecucion [arq], y que se pueden medir los de un agente muerto a mitad de ronda [rie]. Mirar el origen de `tokens_*` en la 0.16.1.
- Que un ruleset con `pull_request` de 0 aprobaciones deja mergear por API a la App, y lo mismo en GitLab [arq]. Sin probar; es de F2-6.
- Que `verificacion` y `revision-ia` se disparan en PR con base `lote/*` [rie]. Mirar el filtro `branches` de F5-1.
- Que ToolsaWeb es privado y gratuito [rie]. Confirmar con un GET de rulesets (solo lectura).
- Que la serie TASK-173..TASK-191 existe en el commit `93c9618ac` [arq], y que la cache por arbol de F1-7 se reutiliza en el PR final si `develop` no se movio [arq].
- Que en Windows no hay bloqueo portable del sistema de ficheros [arq]. Ningun rol lo midio.
