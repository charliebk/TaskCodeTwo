# Plan — TASK-045: F5-5 Runner self-hosted Windows por perfil

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Lo que ya existe cubre el «donde corre»: la entrada `runner-<perfil>` y `runs-on: fromJSON(...)` en `ejecutar`. Falta el «cuando puede correr ahi», que son las condiciones D3b de F1-8 (decision 6). Se anaden:
1. una **guarda en `buscar`** (Linux, sin codigo del PR), que falla cerrada;
2. una **comprobacion de aislamiento** al empezar `ejecutar`, solo en un runner propio;
3. una **guia de alta** con los pasos de Carlos.

No se tocan `tc2 doctor` ni la plantilla de GitLab, y no hay entradas nuevas en el workflow.

Duracion estimada: 180 min

Senales: 1 workflow con 2 pasos nuevos, tests y una regla nueva en el lint, guia y adenda, mas un E2E negativo y otro con runner real (o su alternativa).

### Piezas y limites

- **Paso «Runner propio» en `buscar`** (`.github/workflows/verificacion.yml`). Mira el `runner-<perfil>` de cada perfil del evento. Si alguno lleva `self-hosted`, exige tres cosas: repo privado (`github.event.repository.private`), un `pull_request` de `develop` a `main`, y que el head sea del mismo repo (`head.repo.full_name == github.repository`). Si falta alguna, sale con `exit 2` y su motivo. Los datos del evento le llegan por `env:` (lint A13). Va en un paso aparte para que tenga una sola responsabilidad.
- **Paso «aislamiento del runner propio» al empezar `ejecutar`.** Lleva `if: runner.environment == 'self-hosted'` y `shell: powershell`, y va antes del checkout. Falla si la cuenta es administradora o `SYSTEM`, o si el runner no es Windows. Escribe el usuario y el `bash` que va a usar.
- **Lint en `xtask/src/workflows.rs`:** error si `actions/checkout` lleva `clean: false`. El `clean: true` por defecto da el directorio limpio por job de D3b(b).
- **Guia `docs/ci/verificacion-reutilizable.md`:** seccion «Runner propio (F5-5)» con el alta minima, las etiquetas, la regla de uso y la retirada. Se corrige la divergencia 6: una instancia por maquina y repo.
- **Adenda F5-5 en F1-8, decision 6:** D3b(c) se cumple por construccion, porque en el runner propio solo corre la release. Esto limita F1-7 (decision 5) y el ejemplo de F1-11, que ponia `completa` en `self-hosted`; se citan.
- **Fuera:**
  - `tc2 doctor`: leer los runners pide `administration`; queda en F3-8.
  - GitLab: degradacion declarada.

### Orden de construccion

1. Paso «Runner propio» y su prueba, que ejecuta con bash el `run:` real (se reutilizan `run_del_paso` y `ruta_bash`).
2. Paso de aislamiento y regla `clean: false` en el lint, con mutaciones.
3. Guia y adenda F5-5 en F1-8.
4. E2E negativo en `charliebk/taskcodetwo-e2e-github` (publico): no necesita a Carlos.
5. E2E positivo con un runner real en un repo de pruebas privado, o la alternativa de D2.

## Troceo

Pasa de 60 min. **No se trocea**: decision de Carlos del 2026-11-20 de cerrar H5 sin trocear.

## Lo que el rol no cubrio

- **Riesgos sin revisar:** un fork con una rama `develop` (la guarda lo cubre por `head.repo`), la credencial `.credentials` del runner en disco, la actualizacion automatica del runner, la espera en cola de hasta 24 h, `ejecutar` sin `timeout-minutes` y codigo hostil ya integrado en `develop` que corre en la release.
- **Testing:** no se ha valorado la cobertura de los casos.
- **Dominio:** si la pesada de ToolsaWeb depende de `RAMA_BASE` o de `exampleData` es cosa de F1-11 y F9.
- **Criterio 1:** queda parcial con D2b, o mientras ToolsaWeb no migre (F9). Se dice asi, sin rebajarlo.

## Riesgos aceptados y que los contiene

- **D3b(a)** (usuario de servicio, datos de solo lectura) depende del alta manual. Lo contienen el paso de aislamiento, que falla cerrado, y la guia.
- **Herramientas fuera del espacio de trabajo** del usuario de servicio. Lo contiene D1a: en ese runner solo entra codigo que ya paso por `develop`.
- **Sin comprobacion en `doctor`.** La contiene la guarda en tiempo de ejecucion, y queda en F3-8.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- **Criterio 1, la pesada corre en la maquina de Carlos** (1 prueba E2E): en el repo privado de pruebas, un PR `develop`->`main` con `runner-pesada: '["self-hosted","windows","tc2-e2e"]'` acaba con `verificacion` en verde. El log de aislamiento muestra `tc2-runner` sin administracion, y el stub lee los datos de solo lectura y no puede escribir en ellos. Con D2b, la pesada se queda en cola y se escribe «no demostrado».
- **Criterio 2, cumple F5-5, RN-02, F1-7 decision 5 y F1-8 D3b** (3 pruebas):
  1. `el_runner_propio_solo_en_la_release_de_un_privado` ejecuta con bash el `run:` real. Pasa la release de un repo privado del mismo repo. Fallan con `exit 2` el repo publico, el PR a `develop`, `workflow_dispatch`, el head de un fork y el hotfix. Un perfil en un runner de GitHub no se ve afectado.
  2. Mutaciones: `clean: false` da error; quitar el paso de aislamiento, o ponerlo despues del checkout, falla.
  3. E2E negativo en el repo publico: un `workflow_dispatch` con `perfiles: pesada` y runner `self-hosted` falla en `buscar` con «repo publico», sin encolar nada en el runner.

Ninguno es pesado.

## Lo que necesita decision de una persona

Carlos acepto el 2026-11-20 las opciones recomendadas:
- **D1** — el runner propio solo se usa en la release `develop`->`main` del mismo repo.
- **D3** — `clean: true` vigilado por el lint.
- **D4** — etiquetas `["self-hosted","windows","<proyecto>"]`.
- **D2** — recomendada (a): Carlos registra un runner en su maquina, unos 15 min en una consola de administrador, en un repo privado de pruebas. Crear ese repo y registrar el runner son acciones que se le confirman al llegar al E2E, con los pasos concretos de la guia. Si no lo hace, queda D2b (pesada en cola y criterio 1 «no demostrado»).

## Suposiciones no verificadas

- Que existe `runner.environment` (`self-hosted` o `github-hosted`).
- Que `github.event.repository.private` y `pull_request.head.repo.full_name` llegan al reutilizable con el evento del llamador.
- Las opciones `--runasservice` y `--windowslogonaccount` de `config.cmd`.
