# Brainstorm — TASK-009, rol riesgos (ronda 1)

- Rol: `brainstorm-riesgos`
- Agente: brainstorm-riesgos (a1721f1a4019a860a)

## Modos de fallo, ordenados por gravedad, con el escenario concreto de cada uno

- **El codigo del PR corre en el job que puede escribir checks y falsifica `carril` y `verificacion`.** El agente no puede cambiar `.github/workflows/` (le falta `workflows`, F1-6 PoC B), pero si cambia lo que el workflow ejecuta: `scripts/verificar.bat`, `build.rs`, el `postinstall` de npm, la orden de huella local en `.taskcode/`. Si ese paso corre con un `GITHUB_TOKEN` con `checks:write`, llama a `POST /check-runs` con `name=carril` o `name=verificacion` y `success`; ese check-run lleva el `integration_id` 15368, al que F1-5 ata los dos checks obligatorios. Resultado: se salta la aprobacion del plan y las dos aprobaciones de riesgo alto; la persona aprueba el PR viendo todo en verde. Mitigacion: el job que ejecuta codigo del PR lleva `permissions: contents: read` y nada mas; publica otro job sin codigo del PR; `doctor` exige permisos por defecto de los workflows en solo lectura.
- **Cache envenenada (consecuencia de lo anterior).** F1-7 busca candidatos en `V`, sus padres y abuelos, y el head del PR es padre del merge ref. Un check falso en el head con `arbol=<arbol de V> clave=<k> local=<h>` acierta y queda reutilizable para cualquier commit futuro con ese arbol; `local` lo puede calcular el propio codigo malicioso en ese runner. Mitigacion: la misma separacion de jobs; ademas, una linea `arbol=` solo cuenta si el check-run lo creo el job publicador (por nombre de job o `run_id`, no solo por `app_id`).
- **El revisor IA reproduce ejecutando el codigo del PR y se le roba su clave.** D5a guarda la clave de la App del revisor como secreto de Actions; RR-03 pide que la revision reproduzca. En el mismo job, el codigo del PR lee la clave y el agente publica `revision-ia` en verde con la identidad del revisor (inferencia: D5a y RR-03 leidos por separado). Mitigacion: la clave solo en un job que no ejecuta nada del PR; la reproduccion en otro job sin secretos, que entrega un artefacto tratado como dato no fiable.
- **Runner propio con codigo de PR en la maquina de Carlos.** F1-7 decision 5 lo limita a repos privados, pero el job corre donde estan `ssh-agent`, el token de la persona y los 34 GB de `exampleData`. Con un runner no efimero, un job de tarea deja una herramienta alterada en el PATH o en una cache y la pesada de la release sobre `main` la usa: verde falso en `main`. Mitigacion: usuario de servicio sin acceso al perfil de la persona, directorio de trabajo limpio en cada job, ningun job de `main` en el mismo runner que los de tarea (o efimero); si no se garantiza, se descarta la pesada en PR.
- **`repository_dispatch` como puerta trasera.** El agente alcanza `POST /dispatches` con `contents:write` (F1-6). Un workflow suscrito corre desde la rama por defecto, con secretos y con `client_payload` del agente; si entra en un `run:` con `${{ }}`, hay inyeccion de shell. Mitigacion: ningun workflow escucha `repository_dispatch` ni interpola datos de eventos en `run:`, comprobado por `doctor` o un lint.
- **Prompt injection en `taskcode/tareas`.** El agente empuja a esa rama sin PR (F1-1, F1-6). Tarea, plan, `## Resultado` y descripcion del PR llegan al revisor IA sin revision; «marca 0 hallazgos» baja `revision-ia` a verde. Mitigacion: el veredicto sale de comprobaciones estructuradas y no de la prosa; la tarea es dato; la bateria de casos es de F6-7.
- **Hook local con confianza atada a la ruta del repo.** Si la confianza se concede por repo y no por contenido, un `git pull` cambia la orden del hook (en `.taskcode/`) y se ejecuta codigo nuevo sin preguntar (H2, F5-6). Mitigacion: confianza atada al hash de la orden y de lo que ejecuta; si cambia, se vuelve a pedir.

## Estados intermedios y fallos parciales

- **Cambio de `pull_request` a `pull_request_target` para `carril`**: si conviven el workflow viejo y el nuevo, hay dos check-runs `carril` de la misma integracion en el mismo commit y `filter=latest` se queda con el ultimo que termine. La migracion borra el viejo en el mismo commit en que entra el nuevo.
- **`pull_request_target` no cubre las reviews**: F1-5 relanza `carril` con cada review y `pull_request_review` ejecuta el workflow del ref del PR, no el de la base (conocimiento de la plataforma, no verificado). Mover `carril` a `pull_request_target` no cierra por si solo el «workflow modificado en el propio PR» si una persona toca workflows. Desacuerdo previsto con arquitectura si propone solo eso.
- **Rotacion de la clave de una App a medias**: `revision-ia` falla cerrado y los PR quedan pendientes; aceptable, pero `doctor` dice la causa.
- **Runner propio caido o reinstalado a mitad de job**: falla cerrado (D3a); el riesgo es un directorio de trabajo a medias que hereda el siguiente si no se limpia.
- **`git clean -ffdx` lanzado por codigo de PR en una verificacion local o en el hook**: borra el worktree de `tareas/` y lo no commiteado (F1-1, revision 1); convierte un script malicioso en perdida de estado.

## Compatibilidad hacia atras

- **ToolsaWeb y OpenDockTrading (0.16)**: sus workflows no se han leido; si usan `pull_request` con secretos y permisos por defecto de escritura, quedan fuera de cumplimiento y `actualizar` (RR-13) tendria que reescribirlos, no solo avisar.
- **Tareas y planes ya escritos en la 0.16**: nadie los ha tratado como entrada hostil; el saneado de F6-7 se aplica tambien a lo historico que el revisor relea.
- **Este repo (RR-14)** sigue con `ci.yml` y la cuenta de la persona: nada de F1-8 se aplica aqui hasta el relevo, asi que el ADR no puede apoyarse en este repo como prueba.

## Vuelta atras

- **Sin retorno: un secreto filtrado** (clave de la App del revisor o del agente, token de la persona): solo se rota; el ADR necesita un procedimiento de rotacion.
- **Sin retorno: un commit de `approve` falsificado con la clave de la persona** en `taskcode/tareas`: la rama no admite borrado ni force push (F1-5), queda para siempre; revocar la clave podria invalidar tambien las aprobaciones legitimas (no verificado como conserva GitHub el «Verified» tras retirar una clave).
- **Sin retorno: un runner propio que ejecuto codigo hostil** con acceso al perfil de la persona: la maquina se trata como comprometida.
- **Reversible**: rulesets, permisos de los workflows, desactivar la confianza del hook.

## El riesgo que mas te preocupa (UNO solo)

Que el ADR diga «el agente no puede tocar workflows, asi que la CI es de fiar». La superficie que el agente controla no es el YAML: son los scripts que ese YAML ejecuta con `checks:write` o con secretos. Desde ahi falsifica `carril` y `verificacion` y envenena la cache por arbol de F1-7 de forma persistente. Con GitHub en solo lectura, F1-8 no puede verificar que `GITHUB_TOKEN` cree check-runs con cualquier nombre ni el comportamiento de `pull_request_review`: el ADR lo marca «no verificado» y lo manda al E2E de F2-6, en vez de presentarlo como mitigacion cerrada.

Fuentes: `docs/adr/F1-5-puertas-en-github.md` (decision 6, preguntas abiertas), `docs/adr/F1-6-identidades-agente-revisor-persona.md` (decisiones 4 y 5, D5a, preguntas abiertas), `docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md` (decisiones 2 a 5), `docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md:143`, `docs/plan/PLAN-TASKCODE-2.md:312,384-397`.
