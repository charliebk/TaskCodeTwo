# Clasificación de los IMPORTANTES de la Fase 1 (TASK-073, F1-15)

Una fila por IMPORTANTE de los informes de revisión de las 15 tareas de la
Fase 1 (TASK-002 a TASK-015 y TASK-094), todas las rondas. Un hallazgo que
una ronda reabre o vuelve a listar con el mismo ID cuenta una vez, en la ronda
en que aparece por primera vez. La clasifica el implementador de TASK-073; la
cita (copiada del informe de esa ronda, menos de 15 palabras) permite
contrastarla. `poc-f1-15.py` (sección R) comprueba que las filas son
exactamente los IDs de las tablas de los informes, que cada cita está en el
informe de su ronda y que la clase es de la lista cerrada.

## Lista cerrada de clases y precedencia

Cada hallazgo lleva una sola clase. Si encaja en dos, gana la primera de esta
lista (la que pide el rol que lo habría visto antes):

1. **puerta-seguridad**: trata de una puerta, un permiso, una identidad, una
   firma o una frontera de confianza: que se pueda saltar o bajar, o que el
   ADR diga mal qué permite o qué exige. Incluye una afirmación falsa sobre
   una garantía de puerta.
2. **contrato**: contradice una regla del plan aprobado o el contrato de otro
   ADR (códigos, patrones, nombres) sin ser una puerta.
3. **correccion**: el comportamiento diseñado está mal (el flujo no funciona o
   hace otra cosa), sin ser una puerta ni un contrato.
4. **fidelidad-evidencia**: el ADR afirma como medido o probado algo que su
   evidencia no sostiene.
5. **reproducibilidad**: la PoC no se puede volver a ejecutar o da otro
   resultado.

## Tabla

<!-- clasificacion:inicio -->
| Tarea | Hallazgo | Ronda | Clase | Cita del informe | Motivo de la clase |
|---|---|---|---|---|---|
| TASK-004 | IMP-1 | 1 | correccion | «`start` «idempotente» contradice la comprobacion del ID» | el flujo de `start` no funciona en su recuperación |
| TASK-005 | IMP-1 | 1 | contrato | «La excepcion `reset --hard` de `commit-estado` no es necesaria» | rompe una prohibición del plan aprobado (RN-05) |
| TASK-005 | IMP-2 | 1 | correccion | «`tag-anotado` publica el commit equivocado» | la operación hace otra cosa |
| TASK-005 | IMP-3 | 1 | fidelidad-evidencia | «El ADR dice «comprobada en B1/B2» para caminos que ninguna prueba ha ejecutado» | afirma una prueba que no se ejecutó |
| TASK-006 | IMP-1 | 1 | puerta-seguridad | «una rama que el agente escribe sin puerta» | el check `carril` se alimenta de lo que escribe el agente |
| TASK-006 | IMP-2 | 1 | correccion | «`strict` en `main` con releases por merge commit deja `develop` desactualizada» | efecto operativo de un ajuste; no debilita la puerta |
| TASK-006 | IMP-3 | 2 | puerta-seguridad | «La aprobacion del plan no queda atada al contenido aprobado» | la aprobación se puede reutilizar con otro plan |
| TASK-006 | IMP-4 | 2 | puerta-seguridad | «La regla de derivacion del carril minimo trata cualquier `.md` como errata» | baja el carril y con él las puertas |
| TASK-006 | IMP-5 | 2 | contrato | «La rama de errata `fix/errata-<slug>` no cumple el patron ni la unicidad de F1-3» | contradice el patrón de ramas de F1-3 |
| TASK-007 | IMP-1 | 1 | puerta-seguridad | «`estado-puertas` figura como `cualquiera`, pero el agente no puede leer checks ni statuses» | dice mal qué permiso exige una operación |
| TASK-007 | IMP-2 | 1 | puerta-seguridad | «figuran como `cualquiera`, pero exigen `administration:read`» | dice mal qué permiso exige una operación |
| TASK-007 | IMP-3 | 1 | fidelidad-evidencia | «La PoC no lee la tabla del ADR» | la evidencia dice comprobar lo que no comprueba |
| TASK-008 | IMP-1 | 1 | puerta-seguridad | «puede dar por bueno el check de la release sin correr la pesada» | el check obligatorio se da por bueno sin correr |
| TASK-008 | IMP-2 | 1 | correccion | «La huella (y el runner) no pueden calcularse donde corre `buscar`» | el diseño de la clave no funciona donde se usa |
| TASK-009 | IMP-1 | 1 | puerta-seguridad | «La caché `arbol=` sigue falsificable si el PR modifica el workflow del publicador» | se falsifica la caché del check |
| TASK-009 | IMP-2 | 1 | puerta-seguridad | «Las mitigaciones A6/A13 se apoyan en un lint que no detecta variantes clásicas» | la mitigación de una amenaza no la cubre |
| TASK-009 | IMP-3 | 1 | puerta-seguridad | «afirma como imposible lo que A4/A5 dicen que es posible» | dice mal una capacidad frente a una amenaza |
| TASK-009 | IMP-4 | 2 | puerta-seguridad | «declara sin salvedad lo que depende del token de la persona» | dice mal qué permite una identidad |
| TASK-011 | IMP-1 | 1 | puerta-seguridad | «`tipo` (texto de `tarea.md`, no confiable) elige la fila y puede bajar la puerta» | un dato no confiable baja la puerta |
| TASK-011 | IMP-2 | 3 | puerta-seguridad | «deja fuera la politica de configuracion del carril» | la unión de puertas pierde lo que la config endurece |
| TASK-012 | IMP-1 | 1 | correccion | «Las traducciones de los pilotos no son funcionales con las reglas del propio ADR» | lo diseñado no funciona en los pilotos |
| TASK-012 | IMP-2 | 1 | puerta-seguridad | «se salta con rutas triviales y con ficheros ya modificados» | un derivado cambia estado o aprobación sin aviso |
| TASK-012 | IMP-3 | 1 | fidelidad-evidencia | «`taskctl tareas --json` no sostienen lo que los pilotos necesitan» | la cifra no sostiene la conclusión |
| TASK-012 | IMP-4 | 1 | fidelidad-evidencia | «La ventaja en claves y en prácticas mide anidación, no esfuerzo ni cobertura real» | la medida no mide lo que dice |
| TASK-012 | IMP-5 | 2 | puerta-seguridad | «puede dejarse pasar desde el propio PR» | el PR deja verde un check obligatorio |
| TASK-013 | IMP-1 | 1 | reproducibilidad | «La PoC no es reproducible» | la PoC sale con 1 al reejecutarla |
| TASK-013 | IMP-2 | 2 | fidelidad-evidencia | «el ADR da por aprobados los nombres `tc2` / `taskcodetwo` por Carlos» | afirma una aprobación sin constancia |
| TASK-014 | IMP-1 | 1 | reproducibilidad | «El guion no se puede ejecutar fuera de `02-en-curso`» | la PoC no se ejecuta tras mover la tarea |
| TASK-015 | IMP-1 | 1 | puerta-seguridad | «G09: Ultimate se vende como «origen del check» y la API oficial lo desmiente» | afirmación falsa sobre una garantía de puerta |
| TASK-015 | IMP-2 | 1 | fidelidad-evidencia | «Filas marcadas «empirica» cuya unica llamada es un 401/403 anonimo» | evidencia que no prueba la fila |
| TASK-094 | IMP-1 | 1 | puerta-seguridad | ««cerrado» y la lista del registro se pueden manipular sin firma de persona» | se salta la firma de la persona |
| TASK-094 | IMP-2 | 1 | contrato | «el codigo de precondicion «4, el de F1-12» no es el de F1-12» | contradice el contrato de códigos de F1-2 y F1-4 |
| TASK-094 | IMP-3 | 1 | correccion | «nadie crea `hito-decisiones.yml` en `actualizar`; con `ausente` la puerta niega» | falta un paso del flujo; la puerta falla cerrada, no se salta |
<!-- clasificacion:fin -->

Las tareas TASK-002, TASK-003 y TASK-010 no tienen ningún IMPORTANTE.
