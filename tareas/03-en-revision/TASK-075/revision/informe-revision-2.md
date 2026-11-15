# Informe de revision — TASK-075 (ronda 2)

- Commit revisado: b516dee870028718a61296eec40038c470640855
- Revisor: code-quality-reviewer
- Veredicto: aprobada

Pase generico, verificacion acotada al diff de las correcciones (b516dee sobre 0229179). Alcance de la PoC reducido por decision de Carlos: un hueco declarado en «No verificado» no se cuenta como hallazgo.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-17-progreso-y-estimacion.md:226 (Control de fuga) y tareas/03-en-revision/TASK-075/poc/poc-f1-17.py:567 (`fugas`) y :1209 | implementacion |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-17-progreso-y-estimacion.md:335 y :486 (decision 12) y tareas/03-en-revision/TASK-075/poc/poc-f1-17.py:978 (`mergeada`) | plan |

Sin hallazgos nuevos. Los 6 MENOR de la ronda 1 (MEN-1..MEN-6) tambien los he mirado, aunque la peticion los reservaba a quien coordina, y estan corregidos (detalle abajo).

## Reproduccion

Carpeta de trabajo: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-075-r2` (borrada al terminar; la PoC usa ademas la suya, que borra ella). ToolsaWeb y OpenDockTrading solo lectura (`git log`). Mis guiones estan en el scratchpad de la sesion, no en el repo, e importan la PoC por ruta.

- **PoC completa** (`python -I -B poc-f1-17.py <TaskCodeTwo> <ToolsaWeb> <OpenDockTrading>`): 48,5 s, exit 0, `RESULTADO: todo como se espera`, `carpeta de trabajo borrada al terminar: si`. La salida es **identica** al `poc-f1-17.log` commiteado tras normalizar CRLF (`diff` vacio). `git status` limpio despues (no reescribe nada).
- **IMP-1 (control de fuga)**: corregido. `fugas()` cuenta, por cada estimacion, las muestras usadas que terminaron despues del `approve` de la tarea estimada o que son ella misma. Fuera de muestra da 0 en los dos repos; con la fuga a proposito (`disponibles_con_fuga`) da 2515 y 734 y el control falla si diera 0 (`comprueba("E", n_fuga_mut > 0, ...)`). Prueba propia, una fuga distinta de la de la PoC, sobre ToolsaWeb: (a) sin fuga, 0; (b) anadiendo en **un solo caso** (tarea de en medio) una muestra con `finish` posterior al `approve`, `fugas` = 1; (c) anadiendo en un solo caso la propia tarea, `fugas` = 1. Detecta de verdad y da 0 sin fuga. La frase del ADR ya no vende la estimacion dentro de muestra como prueba: dice que no prueba nada y da las cuatro metricas con las dos n (0,40 frente a 0,45; p80 1,16 frente a 0,71; p90 2,04 frente a 1,07; error medio 78,8 % frente a 61,6 %; n 154 frente a 149), todas verificadas contra el log (lineas 46 y 56).
- **IMP-2 (`start` por ascendencia)**: corregido. `mergeada` usa `git merge-base --is-ancestor <rama remota de la predecesora> origin/develop`, sin mirar el mensaje. S2 (rama empujada, sin integrar): niega, y no por vacio, porque la rama si existe en el remoto. S3 (merge de otra tarea con el trailer `Tarea: TASK-001` copiado, rama sin integrar): niega `predecesora-sin-merge`. S4 (merge `--no-ff` con el mensaje de la 0.16, sin trailer): arranca. Confirmado contra los repos reales: 0 de 50 merges del primer padre con `^Tarea: TASK` en ToolsaWeb@43ab5907e, 0 de 48 en OpenDockTrading@2b0064f2c y 0 en TaskCodeTwo `develop`, como dice el ADR. El ADR promete solo lo probado: el caso real de pilotos (ramas antiguas en el remoto) y el uso de `ultimo_commit_revisado` quedan en «No verificado», que lo declara.
- **`en_zona` frente a globs (MEN-3)**: corregido. 17 casos mios contra `en_zona` (`core/*.rs`, `src/a*/**`, `src/*.rs`, `**/*.rs`, `a/**/b`, `web/**`, `core/a/**`, `?`, zonas `None`): todos dan lo esperado, incluidos los tres que fallaban con `solapan` en la ronda 1 (`src/otro/x.rs` frente a `src/a*/**`, `src/deep/x.rs` frente a `src/*.rs`). `solapan` sigue dando `True` para esos pares, que es lo correcto como comparacion conservadora zona con zona, y el ADR lo dice. S9 lo prueba con `core/*.rs`.
- **MEN-1** (variantes de umbral): la tabla marca las filas que cambian el real o las muestras y el texto dice que no son comparables. **MEN-2** (rename): el ADR limita la causa a lectores sin `FILE_SHARE_DELETE` (Python, Node) y lo del lector de Rust queda en «No verificado». **MEN-4** (`restante` desbordado): el codigo (`desbordada` con menos de N_MIN vivas) y el punto 5 coinciden y L5 sale «faltan ≥31 min (fase en curso por encima de sus parecidas)» en el log. **MEN-5**: «17 son ADR y una, F1-0, es la creacion del repo» cuadra con el log (`errata 17, funcionalidad 1`). **MEN-6**: el punto «texto ajeno saneado» esta en la decision 8, con el caso de prueba remitido a F3-12.
- Cifras de la ronda: las tablas del ADR (variantes, start S1-S9, statusline L1-L5) se comprueban contra el log por la seccion G de la PoC, y el log reproducido es identico.

## Notas (fuera de la tabla; no abren ronda)

- `puede_empezar` en la PoC hace `fetch` solo de `develop` y de la rama de estado, mientras que la decision 12 del ADR dice «`fetch` de `develop`, de la rama de estado y de las ramas». Los casos S2-S4 no lo notan porque las ramas se empujan desde la misma maquina que arranca o se traen con otro fetch. En la PoC un fallo asi niega de mas, no de menos; es un detalle para F3-6.
- La prueba por ascendencia da un falso negativo si la rama de la predecesora recibio commits despues del merge, o si se integro por squash o rebase (otros SHA). El ADR parte de que el flujo es `--no-ff` y de que F1-3 conserva las ramas, y declara lo de las ramas ausentes; vale como esta para el alcance de la PoC.
- `fugas()` mide las muestras del nivel de cascada elegido, no las lineas base (`ingenua`, `complejidad`), que usan el mismo `disponibles`; no hay fuga posible por otro camino en el codigo leido.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-075/poc/poc-f1-17.py` y `poc-f1-17.log`: mi parte del diff (excluida del diff embebido); sin ellos no se puede reproducir.
- `tareas/03-en-revision/TASK-075/revision/informe-revision-1.md`: formato del informe y hallazgos a verificar.
- ToolsaWeb y OpenDockTrading, solo lectura con `git log --first-parent --merges`: para contar los trailers `Tarea:` (IMP-2).
