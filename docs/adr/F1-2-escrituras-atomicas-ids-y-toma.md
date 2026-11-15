# ADR F1-2 — Escrituras atómicas en la rama de estado, reserva de IDs y toma de tareas

- Estado: **aceptada** (2026-11-14)
- Tarea: TASK-003 · Plan: [F1-2 en PLAN-TASKCODE-2](../plan/PLAN-TASKCODE-2.md) (§4 P2 y tabla de la Fase 1)
- Requisitos: RR-01, RR-09, RR-12, RN-05, RN-06 de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); H3 de la [auditoría externa](../auditoria/AUDITORIA-EXTERNA-2026-11-13.md); se apoya en RF-02 y RF-07
- Depende de: [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (carpeta fija `TASK-NNN/` en la rama huérfana `taskcode/tareas`, montada en `tareas/`)
- Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2b, D3a, D4a del plan de TASK-003)

## Contexto

F1-1 dejó el estado de las tareas en una rama propia, `taskcode/tareas`, que
todos los clones escriben. Faltaba decidir **cómo se escribe** sin pisarse:

- **H3** (auditoría, CRÍTICO): en la 0.16 el ID se calcula en local; dos
  clones generan el mismo `TASK-NNN` y el `pull` da `CONFLICT (add/add)` en
  `tarea.md` y en `ROADMAP.md`.
- **RR-09 / RR-12**: varias personas y agentes a la vez; IDs y ramas únicos
  aunque se creen a la vez; una tarea la lleva una sola persona; el estado que
  se ve es el más reciente.
- **RN-05**: el estado no se pierde por operaciones de Git; **RN-06**: un
  proyecto de una persona con agentes funciona igual de bien que un equipo
  (también sin remoto).
- **RR-01**: toda operación de Git la hace un comando del CLI, idempotente y
  con un código de salida que dice qué hacer.
- **P2** del plan: cada cambio es un commit pequeño con push inmediato; si el
  push se rechaza, el CLI rehace el cambio sobre lo nuevo y reintenta; el ID se
  reserva con ese push; `start` escribe quién la lleva antes de crear la rama.

Preguntas que responde: qué hace el CLI cuando su push se rechaza, cómo se
reserva el ID, cómo se toma una tarea sin que dos la tomen, y qué pasa sin red.

## Alternativas

| | Descripción | Veredicto |
|---|---|---|
| **D1a** | Tras un push rechazado, descartar el commit local, `fetch` y **re-ejecutar la operación** sobre el estado nuevo (recalcula ID y precondiciones). El rechazo non-fast-forward hace de compare-and-swap. | **Elegida** |
| D1b | Tras un push rechazado, `pull --rebase` del commit local. | Descartada: con el mismo ID da `AA TASK-NNN/tarea.md` y `UU ROADMAP.md` (PoC 3a); aunque no chocara, no recalcula el ID ni ve que otro tomó la tarea. |
| **D2b** | Sin red: las operaciones que reservan algo compartido (`new`, `import`, `start`, `liberar`, `reasignar`) fallan con `SinRed`; las de una tarea que ya lleva quien las lanza se commitean en local y se publican re-ejecutándolas en la siguiente sincronización. | **Elegida** |
| D2a | Sin red, todo falla. | Descartada: impide trabajar en la tarea propia en un tren o con la VPN caída, sin ganar nada (nadie más puede escribir esa tarea). |
| D2c | Sin red, IDs provisionales que se renumeran al publicar. | Descartada: la rama de código y las referencias salen del ID; renumerar rompe su unicidad (RR-12). |
| **D3a** | Tope de 5 intentos con espera exponencial y jitter; después `Agotada` con su código. Parámetro con valor por defecto. | **Elegida**, con la base de la espera ajustada por la PoC (ver «Divergencia con el plan») |
| **D4a** | ID `TASK-NNN` consecutivo = máximo de las carpetas + 1, reservado con el push. | **Elegida** |
| D4b | ULID o prefijo por persona (lo que sugería H3). | Descartada: no necesita reserva, pero rompe la numeración que usan los tres repos y no es legible; con D1a la reserva funciona (PoC 1 y 3b). |

## Pruebas de concepto

Guion `poc-f1-2.sh` y su salida completa `poc-f1-2.log`, en la carpeta
`poc/` de TASK-003 (sin enlace: la carpeta de la tarea cambia de estado al
cerrarse), ejecutados en Git Bash (Git 2.56.0 en Windows) bajo
`%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-003\poc\` (borrado al
terminar). Solo remotos bare locales con rutas absolutas, con
`receive.denyNonFastForwards=true` (lo que F1-5 pedirá en la plataforma), sin
GitHub ni red externa.

El guion implementa en bash el punto único de escritura (`tx`) con el
protocolo de la decisión: cerrojo local, `fetch`, operación sobre
`origin/taskcode/tareas`, regenerar `ROADMAP.md`, commit, `push` sin force;
si se rechaza, descarta el commit y re-ejecuta. La concurrencia es real
(procesos en segundo plano que arrancan a la vez con una barrera de fichero),
pero en una sola máquina.

```bash
bash poc-f1-2.sh "$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-003/poc" > poc-f1-2.log 2>&1
```

### 1 — Altas concurrentes: 3 clones × 4 altas a la vez

Valores por defecto de la decisión (5 intentos, base de la espera 2 s).

```text
[c1] intento 1: push rechazado (non-fast-forward); commit descartado, re-ejecucion tras 1.36s
[c1] intento 2: push rechazado (non-fast-forward); commit descartado, re-ejecucion tras 3.42s
[c1] intento 3: push rechazado (non-fast-forward); commit descartado, re-ejecucion tras 4.06s
[c1] alta|alta 1 de c1 -> Publicada (intentos=4) TASK-005
...
[c2] intento 4: push rechazado (non-fast-forward); commit descartado, re-ejecucion tras 10.94s
[c2] alta|alta 1 de c2 -> Publicada (intentos=5) TASK-009
...
[c3] alta|alta 1 de c3 -> Publicada (intentos=1) TASK-001
carpetas TASK-* en origin: 12 ; titulos distintos: 12 ; entradas en ROADMAP.md: 12
IDs consecutivos desde TASK-001: si
altas publicadas: 12 ; agotadas: 0 ; pushes rechazados y reintentados: 7 ; maximo de intentos de una alta publicada: 5 ; duracion: 58s
commits locales sin publicar en los 3 clones: 0 (una Agotada no deja nada escrito)
commits en origin/taskcode/tareas: 13 (1 inicial + 12) ; merges: 0
remoto con receive.denyNonFastForwards=true; el guion no usa --force
```

Lectura: 12 IDs distintos y consecutivos, ninguna alta pisada (12 títulos
distintos), `ROADMAP.md` regenerado con las 12, historia lineal sin merges y
sin force. Hubo 7 pushes rechazados; todos se resolvieron re-ejecutando.

### 1b — Base de la espera (3 rondas de la prueba 1 por valor)

```text
duracion media de una transaccion sin contencion: 2642 ms
base 0.1s ronda 1: publicadas 11/12, agotadas 1, reintentos 14, max intentos 5, IDs consecutivos si, commits locales 0, 65s
base 0.1s ronda 2: publicadas 11/12, agotadas 1, reintentos 16, max intentos 4, IDs consecutivos si, commits locales 0, 69s
base 0.1s ronda 3: publicadas 11/12, agotadas 1, reintentos 15, max intentos 4, IDs consecutivos si, commits locales 0, 56s
base 0.5s ronda 1: publicadas 11/12, agotadas 1, reintentos 11, max intentos 5, IDs consecutivos si, commits locales 0, 46s
base 0.5s ronda 2: publicadas 11/12, agotadas 1, reintentos 11, max intentos 5, IDs consecutivos si, commits locales 0, 53s
base 0.5s ronda 3: publicadas 11/12, agotadas 1, reintentos 11, max intentos 5, IDs consecutivos si, commits locales 0, 49s
base 2s ronda 1: publicadas 12/12, agotadas 0, reintentos 7, max intentos 5, IDs consecutivos si, commits locales 0, 52s
base 2s ronda 2: publicadas 12/12, agotadas 0, reintentos 6, max intentos 5, IDs consecutivos si, commits locales 0, 48s
base 2s ronda 3: publicadas 11/12, agotadas 1, reintentos 8, max intentos 4, IDs consecutivos si, commits locales 0, 56s
RESULTADO 1b: base 0.1s -> 3 agotadas de 36 altas, 45 reintentos en 3 rondas
RESULTADO 1b: base 0.5s -> 3 agotadas de 36 altas, 33 reintentos en 3 rondas
RESULTADO 1b: base 2s -> 1 agotadas de 36 altas, 21 reintentos en 3 rondas
```

Lectura: en **todas** las rondas los IDs publicados son consecutivos y no
queda ningún commit local: una `Agotada` no deja nada escrito. Lo que cambia
es cuántas altas se agotan. El patrón es siempre el mismo: el clon que gana
el primer push encadena sus 4 altas (cada transacción dura ~2,6 s en este
guion) mientras un perdedor gasta sus 5 intentos en esperas más cortas que
esa ráfaga.

### 2 — Toma concurrente de la misma tarea

```text
[ana] start|TASK-001|ana -> Publicada (intentos=1) TASK-001 para ana
[ana] start: rama feature/task-001 y worktree creados
[beto] intento 1: push rechazado (non-fast-forward); commit descartado, re-ejecucion tras 1.42s
[beto] start|TASK-001|beto -> Rechazada (la lleva ana) en el intento 2
[beto] start: Rechazada (codigo 3); no se crea rama ni worktree
asignado en origin: ana
ramas feature/* en el clon de ana: feature/task-001
ramas feature/* en el clon de beto:
-- idempotencia: ana repite start
[ana] start|TASK-001|ana -> Publicada (sin cambios: ya la llevaba ana)
commits en origin tras repetir: 3 (1 inicial + alta + toma)
```

Lectura: los dos escriben `asignado` a la vez; el push de beto se rechaza, al
re-ejecutar ve que la lleva ana y termina con `Rechazada` **sin crear rama ni
worktree**. Repetir `start` no escribe estado (idempotente).

### 3 — Rebase frente a re-ejecución (evidencia de D1)

```text
=== 3a. Rebase ingenuo ===
c1 calcula en local: TASK-001
c2 calcula en local: TASK-001
c1 push OK
c2 push:  ! [rejected]        taskcode/tareas -> taskcode/tareas (fetch first)
  UU ROADMAP.md
  AA TASK-001/tarea.md
pull --rebase: conflicto en: ROADMAP.md TASK-001/tarea.md
=== 3b. Re-ejecucion ===
c2 tiene listo en local, sin publicar: alta de c2 (re-ejecucion)
[c1] alta|de c1 (re-ejecucion) -> Publicada (intentos=1) TASK-002
[c2] intento 1: push rechazado (non-fast-forward); commit descartado, re-ejecucion tras 1.02s
[c2] alta|de c2 (re-ejecucion) -> Publicada (intentos=2) TASK-003
```

Lectura: 3a reproduce H3 en la rama de estado (el mismo conflicto add/add que
describe la auditoría). En 3b, c2 tenía su alta calculada con el ID que luego
publicó c1; al re-ejecutar recalcula y publica la siguiente, sin conflicto.

### 4 — Sin red: operaciones compartidas

Remoto cambiado a una ruta inexistente.

```text
[ana] alta|alta sin red -> SinRed (exige red; nada escrito)
[ana] start|TASK-001|ana -> SinRed (exige red; nada escrito)
[ana] start: SinRed (codigo 4); no se crea rama ni worktree
HEAD de tareas/ igual que antes: si ; git status de tareas/: '' (vacio = limpio)
ramas feature/*: '' (vacio = ninguna)
```

### 5 — Sin red: transición de la tarea propia

```text
[ana] transicion|TASK-001|en-revision|ana -> SinRed (pendiente de publicar: commit local)
codigo: 4 (SinRed) ; commits pendientes de publicar: 1 (81e6526 transicion TASK-001 en-revision ana)
[beto] transicion|TASK-001|terminada|beto -> Rechazada (la lleva ana, no beto)
[beto] alta|tarea de beto -> Publicada (intentos=1) TASK-002
-- vuelve la red de ana; su siguiente operacion publica lo pendiente
[ana] pendiente re-ejecutada: transicion|TASK-001|en-revision|ana
[ana] alta|ana tras volver la red -> Publicada (intentos=1) TASK-003
estado de TASK-001 en origin: en-revision
el commit pendiente 81e6526 esta tal cual en origin: no (no = se re-ejecuto sobre lo nuevo, no se rebaso)
pendientes tras publicar: 0
```

Lectura: la transición propia queda como commit local con la operación
anotada (`Operacion: transicion|TASK-001|en-revision|ana`); al volver la red,
la siguiente operación la re-ejecuta sobre el estado nuevo (que ya tenía la
alta de beto) y la publica por el mismo camino. Una transición de una tarea
ajena se rechaza también sin red.

### 6 — Repo sin remoto (RN-06)

```text
[solo] alta|primera -> Publicada (repo sin remoto: commit local) TASK-001
[solo] import|2|lote -> Publicada (repo sin remoto: commit local) TASK-002 TASK-003
[solo] start|TASK-001|carlos -> Publicada (repo sin remoto: commit local) TASK-001 para carlos
[solo] start|TASK-001|agente -> Rechazada (la lleva carlos)
commits en taskcode/tareas: 4 ; ramas feature/*: feature/task-001
```

Lectura: sin remoto no hay «sin red»: el commit local es la transacción. El
`import` de N tareas es una sola operación (un commit, IDs consecutivos).

### Divergencia con el plan

El plan esperaba de la prueba 1 «12 IDs distintos y consecutivos» con D3a
(5 intentos, espera exponencial con jitter, valores sin medir). Con la base
de espera de partida (0,1 s, el orden de magnitud que suponía el plan) **no
se cumplió**: en las 3 rondas de 1b una alta de 12 terminó en `Agotada`
(3 de 36), y lo mismo con 0,5 s. La primera ejecución del guion, con 0,1 s,
dio también 11 publicadas y 1 agotada.

Ajuste, dentro de D3a (los dos valores eran parámetros): se mantienen los
**5 intentos** y la base de la espera pasa a **2 s**, del orden de la
duración de una transacción. Con 2 s: 12/12 en la prueba 1 y 1 agotada de 36
en 1b, con la mitad de reintentos que con 0,1 s. **No elimina** las
`Agotada` bajo una ráfaga sostenida de un mismo clon; lo que la decisión
garantiza es que una `Agotada` no escribe nada y tiene su código (5), así que
quien la recibe reintenta. En el uso real, una ráfaga de altas de un clon es
un `import` (una sola operación), no N transacciones. El valor por defecto se
revisa con el CLI real (F2-3) y contra la plataforma (F9-4).

La muestra es pequeña (3 rondas por valor, 36 altas): la diferencia entre
0,5 s y 2 s no es concluyente (en la repetición de la revisión, 2 s dio 0
agotadas, 0,5 s 3 y 0,1 s 4), y está calibrada contra el guion (~2,5 s por
transacción), no contra el CLI. Coste de esperar tanto: con 5 intentos y
base 2 s, la espera acumulada antes de `Agotada` llega a
2·(1+2+4+8) = 30 s, más la duración de las transacciones.

**D1a también sustituye el rebase de P2 y de F1-1.** P2 dice que el roadmap
«se regenera tras el rebase» y el ADR F1-1 lo repite (Decisión 3 y su PoC
A.5, que termina con `pull --rebase`). Con D1a no hay rebase: tras un
rechazo se descarta el commit local y se re-ejecuta la operación, que
regenera el `ROADMAP.md` sobre el estado nuevo. Lo de F1-1 sigue valiendo
para la forma (el ROADMAP se regenera y no se mergea), no para el mecanismo.

### No verificado

- **Pushes simultáneos contra GitHub o GitLab.** El bare local rechaza el
  non-fast-forward igual que GitHub (log de F1-1, A.5), pero que la
  plataforma serialice igual los pushes simultáneos, y con qué latencia, no se
  ha probado: pasa a **F9-4** (tres personas a la vez).
- **Concurrencia entre máquinas distintas**: la PoC usa procesos de una sola
  máquina.
- **Duración de una transacción en el CLI real**: la medida (2,6 s de media
  sin contención) es la del guion bash en Windows, que lanza muchos procesos;
  el valor por defecto de la espera se revisa con el CLI de F2-3 y en F9-4.
- **Cerrojo local con varios procesos del mismo clon**: implementado
  (`.git/taskcode-estado.lock`, creación exclusiva) pero sin prueba propia;
  pasa al test de F2-3.

## Cómo se ve en GitHub

No aplica: el protocolo solo usa `fetch` y `push` sin force, igual en GitHub y
GitLab (RR-15); las reglas de la rama (sin force-push ni borrado) son de F1-5.

## Decisión

1. **Un único punto de escritura** sobre `taskcode/tareas` (módulo de la rama
   de estado, F2-3). Todos los comandos que cambian estado pasan por él:
   `new`/`import` (F2-4), `start` (F3-6), transiciones, `liberar`/`reasignar`
   (F3-10). Es una transacción optimista (firmas orientativas, no código):

   ```text
   operacion(estado_actual) -> cambios | Rechazo(motivo)
   transaccion(operacion)   -> Publicada | Rechazada(motivo) | SinRed(pendiente) | Agotada
   ```

2. **Protocolo de push con reintento**:
   1. Cerrojo local del clon `.git/taskcode-estado.lock` (persona y agente del
      mismo clon no se pisan el worktree `tareas/`).
   2. `fetch` de `taskcode/tareas` y worktree en `origin/taskcode/tareas`.
   3. Ejecutar la operación sobre ese estado fresco; puede rechazar (p. ej. la
      tarea ya tiene `asignado`).
   4. Regenerar `ROADMAP.md`, commit pequeño (con la operación anotada en el
      mensaje) y `push` **sin force**. El rechazo non-fast-forward es el
      compare-and-swap.
   5. Si se rechaza: descartar el commit local, esperar
      `base · 2^(n-1) · U(0,5; 1)` y volver al paso 2 **re-ejecutando la
      operación** (nunca `rebase`). Por defecto **5 intentos** y
      **base 2 s** (ver «Divergencia con el plan»); los dos son parámetros.
      Tras el último, `Agotada`, sin dejar nada escrito.
   6. Soltar el cerrojo. `ROADMAP.md` no se mergea nunca: se regenera (F1-1).
3. **Reserva de IDs (H3)**: ID = máximo de las carpetas `TASK-*` + 1,
   calculado dentro de la operación, formato `TASK-NNN`. Las carpetas no se
   borran nunca (F1-1) y las descartadas conservan su ID (RF-07): ninguno se
   reutiliza. Un `import` de N tareas (RF-02) es **una** operación: un commit,
   un push, N IDs consecutivos. La rama de código sale del ID y hereda su
   unicidad; F2-4b añade la comprobación contra el remoto.
4. **Toma de tareas**: `start` exige `asignado` vacío o igual a quien la
   lanza; escribe `asignado` y estado, y **solo con el push publicado** crea
   rama y worktree. Si al re-ejecutar ve que otro la tomó:
   `Rechazada("la lleva X")`, sin crear nada. Idempotente: si ya la lleva la
   misma persona no escribe estado y solo crea lo que falte. El «quién» lo
   llena F2-13.
5. **Sin red** (D2b): `new`, `import`, `start`, `liberar` y `reasignar`
   exigen red; sin ella devuelven `SinRed` sin commit local. Las operaciones
   de una tarea que ya lleva quien las lanza (transiciones, plan, informes) se
   commitean en local como «pendientes de publicar» con la operación anotada;
   la siguiente transacción con red las **re-ejecuta** sobre el estado nuevo y
   las publica por el mismo camino (no hay segundo mecanismo). `status` dice
   cuántas hay y la hora de la última sincronización (F3-5b). **Un repo sin
   remoto no está «sin red»**: el commit local es la transacción (RN-06).
6. **Frescura (RR-12)**: toda escritura empieza con `fetch`; la lectura
   fresca (`status`, `roadmap`) es de F3-5b.
7. **Códigos de salida** (RR-01; valores propuestos, los fija F2-3 junto al
   resto de códigos del CLI):

   | Resultado | Código | Qué hace quien lo recibe |
   |---|---|---|
   | `Publicada` | 0 | sigue (también si no había nada que cambiar) |
   | `Rechazada(motivo)` | 3 | no reintenta: la operación no procede (la lleva otro, no existe) |
   | `SinRed(pendiente)` | 4 | si `pendiente=no`, reintenta con red; si `pendiente=sí`, sigue: se publicará |
   | `Agotada` | 5 | reintenta más tarde; no se ha escrito nada |

## Requisitos y evidencia

| Requisito | Decisión | Evidencia |
|---|---|---|
| H3 IDs sin colisión entre clones | D4a + D1a: ID calculado dentro de la operación y reservado con el push | PoC 1 (IDs distintos y consecutivos), 3a (rebase reproduce H3), 3b (re-ejecución lo evita) |
| RR-09 trabajo en grupo sin conflictos de estado ni IDs | protocolo con reintento; `ROADMAP.md` regenerado | PoC 1 (ninguna alta pisada, historia lineal, sin merges), 3b |
| RR-12 IDs y ramas únicos aunque se creen a la vez | rama derivada del ID publicado | PoC 1, 3b; comprobación contra el remoto: F2-4b (no verificado aquí) |
| RR-12 una tarea la lleva una sola persona | toma con `asignado` publicado antes de crear la rama | PoC 2 (rama solo en el ganador; perdedor `Rechazada`) |
| RR-12 estado más reciente | toda escritura parte de `fetch`; pendientes se re-ejecutan sobre lo nuevo | PoC 3b, 5; lectura fresca: F3-5b |
| RN-05 el estado no se pierde por Git | nunca force ni rebase; `Agotada` y `SinRed` no dejan escrituras a medias | PoC 1 (`commits locales sin publicar: 0`), 4 (HEAD igual, worktree limpio), 5 |
| RN-06 una persona con agentes igual que un equipo | sin remoto, el commit local es la transacción; cerrojo local para persona y agente del mismo clon | PoC 6; cerrojo: no verificado (F2-3) |
| RR-01 código de salida que dice qué hacer, idempotente | cuatro resultados con código; `start` idempotente | PoC 2 (códigos 0 y 3, repetir `start`), 4 (código 4), 1b (código 5 en las `Agotada`) |

## Consecuencias

- **F2-3** implementa el punto único de escritura (cerrojo, protocolo,
  pendientes, códigos) y su test de dos procesos a la vez; también el test
  propio del cerrojo local.
- **F2-4** reserva de IDs con este protocolo; **F2-4b** comprueba la rama
  contra el remoto y avisa de títulos duplicados.
- **F3-6** `start` con toma atómica e idempotente; **F3-10** `liberar` y
  `reasignar` como operaciones compartidas (exigen red).
- **F3-5b** `status`/`roadmap` frescos, con pendientes de publicar y hora de
  la última sincronización.
- **F2-13** llena el «quién» de `asignado` con la identidad de la plataforma.
- **F1-5** prohíbe force-push y borrado en `taskcode/tareas` (en la PoC,
  `receive.denyNonFastForwards`).
- **F9-4** valida contra la plataforma real con tres personas: pushes
  simultáneos, valores de intentos y espera.
- **Este repo no cambia** (RR-14): sigue con la 0.16 hasta el relevo.

## Fuera de alcance

- Lectura fresca y presentación de `status`/`roadmap`: F3-5b.
- Identidad de la persona y del agente: F2-13, F1-6.
- Reglas de la rama de estado en la plataforma: F1-5.
- Flujo de ramas y worktrees de tarea: F1-3.
- Formato de `tarea.md`: F1-9.

## Preguntas abiertas

Este ADR se planificó con un solo rol (arquitectura); quedan sin cubrir y se
resuelven en F2-3 y F3-10 salvo que se diga otra cosa:

- **Error de autenticación confundido con «sin red»**: la PoC trata cualquier
  fallo de `fetch` como `SinRed`. F2-3 tiene que distinguir red caída,
  credenciales y remoto inexistente, y dar un código propio al segundo.
- **Hook o regla de la plataforma que rechaza el push** por otro motivo que
  non-fast-forward: hoy la PoC lo cuenta como contención (su `grep` casa
  `rejected`) y acaba en `Agotada`, que mandaría reintentar sin fin ante un
  rechazo permanente; F2-3 tiene que separar non-fast-forward de cualquier
  otro rechazo del remoto, con un código propio.
- **Proceso que muere entre el push de `start` y la creación de la rama**:
  solo lo cubre la idempotencia (repetir `start` crea lo que falte).
- **Cerrojo local huérfano** (proceso muerto con el cerrojo tomado): falta la
  regla de caducidad (PID vivo, antigüedad).
- **Worktree `tareas/` desmontado o borrado** (`git clean -ffdx`): abierto
  desde F1-1.
- **Pendiente sin red frente a una reasignación hecha por otro**: al
  re-ejecutarla se rechaza (la lleva otro) y se pierde ese cambio; hay que
  decir qué ve la persona y dónde queda lo que había escrito.
- **Dominio**: qué es «lista para empezar», si un agente puede tomar una
  tarea en nombre de una persona, y si las transiciones propias sin red
  rompen lo que el equipo espera ver (RR-12).
- **Rendimiento**: contención medida solo con 3 clones en una máquina; la
  ráfaga de altas seguidas de un mismo clon es el peor caso (en uso real
  sería un `import`, una sola operación).

## Adenda (F2-3, TASK-018) — preguntas abiertas resueltas

El punto único de escritura está en `crates/tc2/src/estado/` y se prueba en
`crates/tc2/tests/estado_concurrencia.rs` y `estado_casos.rs`.

| Pregunta abierta | Respuesta de F2-3 | Evidencia |
|---|---|---|
| Error de autenticación confundido con «sin red» | `clasificar_fallo_remoto` separa red caída (`sinred`, 4), credenciales y remoto inexistente (`denegada`, 8) y rama inexistente; lo no reconocido es `sinred` | test unitario con textos de Git y de GitHub; F2-2 `fetch` con una ruta que no es un repo |
| Hook o regla que rechaza el push | `denegada` (8) en el primer intento, sin reintento | test con `pre-receive` que rechaza: un solo push |
| Cerrojo local huérfano | cerrojo del sistema operativo sobre `.git/taskcode-estado.lock` (`File::try_lock`): si el proceso muere, el sistema lo suelta; espera máxima 60 s y después `agotada motivo=cerrojo` | test de dos procesos del mismo clon sin remoto (solo los separa el cerrojo) |
| Worktree `tareas/` desmontado o borrado | la transacción no depende de `tareas/` (lee y escribe commits); tras publicar adelanta la rama local y dice `tareas=sin-montar`; `montar` la vuelve a montar | test de montaje |
| Pendiente sin red frente a un cambio de otro | al re-ejecutarla con red, si ya no procede, su ref pasa a `refs/taskcode/rechazadas/NNNNNN` y la línea dice `rechazadas=N`; el contenido sigue en un objeto de git | test sin red: `crear-unico` que otro hizo antes |
| Cerrojo con varios procesos del mismo clon (No verificado de la PoC) | verificado | mismo test del cerrojo |

Cómo quedan las pendientes (D2b): una operación con `exige_red()=false` sin
red se guarda como commit en `refs/taskcode/pendientes/NNNNNN`, con la
operación anotada en el mensaje (`Operacion: …`), y devuelve `sinred
pendiente=si`. La siguiente transacción con red no publica ese commit: la
reconstruye con el catálogo de operaciones y la **re-ejecuta** sobre el estado
nuevo, en el mismo commit que la operación nueva. Si la operación nueva se
rechaza, las pendientes no se publican y siguen ahí.

**Divergencia con la decisión 2**: no hay commit local que descartar tras un
rechazo (el commit sale de `commit-tree`, F1-4 divergencia 2), y el cerrojo se
suelta durante la espera para que otro proceso del clon avance.

**Valores por defecto**: se mantienen 5 intentos y base 2 s (`Politica`). Los
tests usan base corta. No se ha medido la duración de una transacción del CLI
contra la plataforma: sigue en F9-4.

**Contención en el remoto** (nuevo): dos pushes simultáneos al mismo remoto
local pueden dar `[remote rejected] (cannot lock ref …)` en vez de
non-fast-forward; se trata como contención (ver la adenda de F1-4).

### Tras la revisión de la ronda 1 de TASK-018

- **Remoto añadido después** (RN-06 → equipo): si el remoto aún no tiene la
  rama de estado y existe la local, la transacción parte de la local y
  publica su historia; antes empezaba de cero y la local divergía para
  siempre. Test `remoto_anadido_despues_publica_la_historia_local`.
- **`montar` toma el mismo cerrojo** que la transacción: cuatro `montar` a
  la vez en el mismo clon dan un `ok` y tres `sin-cambios`, no `error`.
- **Una pendiente que falla al re-ejecutarse** (error interno, no solo
  rechazo) pasa a `rechazadas`: no bloquea las escrituras siguientes.
- **Rutas**: además de relativas y sin `..`, portables entre sistemas (sin
  `<>:"|?*`, sin nombres reservados de Windows, sin final `.` o espacio); una
  ruta que solo difiere en mayúsculas de otra de la rama da `rechazada
  motivo=ruta-duplicada`.
- **`tareas=con-cambios` y `tareas=diverge` salen con `ok`** (aceptado): lo
  escrito está publicado y correcto; lo que no se pudo es adelantar la
  carpeta de la persona, y la línea lo dice. El contrato de F1-4 no muestra
  diagnóstico en un éxito; avisar de forma visible es de `status` (F3-5b).
  Sin remoto el mismo choque es `rechazada motivo=tareas-con-cambios`,
  porque no hay otro sitio donde quede escrito.
- **Límite aceptado: contenido UTF-8.** La vista lee los ficheros como texto
  (el puerto convierte la salida de `git` con sustitución); un fichero que no
  sea UTF-8 se corrompería al leerlo y reescribirlo. El formato de tarea
  (F1-9) es UTF-8; lo que escriba la persona a mano en otra codificación no
  está cubierto.
- **Límite aceptado: contención por texto.** `[remote rejected]` con «cannot
  lock ref» o «failed to update ref» se trata siempre como contención; si un
  servidor lo diera de forma permanente, acabaría en `agotada` tras los
  intentos (acotado), no en `denegada`. No visto en ningún servidor real.

### Tras la revisión de la ronda 2 de TASK-018

- **Fallo local frente a remoto**: un `permission denied` sin las marcas del
  remoto (`publickey`, «could not read from remote repository») es del
  sistema de ficheros del clon y da `error` (1) con `motivo=error-local`; no
  `sinred` (que mandaría reintentar) ni `credenciales`. `access denied`
  sigue siendo de credenciales.
- **Choque por mayúsculas por etapas**: una pendiente que al re-ejecutarse
  chocaría con lo ya publicado va a `rechazadas`; solo la operación actual
  puede devolver `rechazada motivo=ruta-duplicada`. Antes, la pendiente
  bloqueaba todas las escrituras del clon.
