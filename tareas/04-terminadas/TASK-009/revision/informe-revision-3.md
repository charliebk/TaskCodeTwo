# Informe de revision — TASK-009 (ronda 3)

- Commit revisado: 15920490783032be788e78412e2adf495fc805ca
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:75 | plan |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:381 | implementacion |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:417 | implementacion |
| IMP-4 | IMPORTANTE | corregido | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:417 | implementacion |
| MEN-7 | MENOR | abierto | docs/adr/F1-8-seguridad-modelo-de-amenazas.md:81 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Sin hallazgos nuevos: la correccion no abre ningun CRITICO ni IMPORTANTE nuevo, y no hay
ninguno abierto. Veredicto propuesto por el revisor: **aprobada** (el veredicto de la
cabecera se deja como esta, segun la peticion; lo registra quien coordina).

### Verificacion de los pendientes

**IMP-4 (corregido).** La celda de la fila «Agente en local» (ADR:417) ya no declara sin
salvedad «tocar workflows»: la frase final de la celda dice «Todo lo de esta celda queda
condicionado a resolver A5», y «tocar workflows» aparece con su causa real (la App no tiene
`workflows`, pero el token de la persona en esta maquina si). Comprobado con
`gh auth status`, solo la linea de scopes: `Token scopes: 'gist', 'read:org', 'repo', 'workflow'`
(el token no se ha mostrado). Esto confirma la afirmacion de la celda y de la nueva entrada de
«No verificado» (ADR:651-653). La salvedad cubre toda la celda: aprobar (A5), integrar (A5),
firmar un `approve` (A4), tocar workflows (A5) y publicar checks (A5, ver nota 2).
Coherente con el estado de la decision (ADR:420-425), con H9 (ADR:565), RR-08 (ADR:567) y la
divergencia 4 (ADR:550-553), que usan la misma salvedad.

**IMP-1 (corregido).** Sigue presente: la fila A7 (ADR:75) dice «mitigacion **parcial**» y la
regla de `buscar` para PR que tocan `.github/workflows/**` o `.taskcode/**`. Sin cambios en esta
ronda.

**IMP-2 (corregido).** Sigue presente: ADR:381 declara que el lint de la PoC es heuristico.
Sin cambios en esta ronda.

**IMP-3 (corregido).** Sigue presente la salvedad «salvo A4/A5» en la celda (ADR:417), y en
las demas referencias de la ronda 2. Sin cambios en esta ronda.

**Diff de la correccion.** `git diff 18370f1 HEAD -- docs/` esta vacio: desde el commit de la
correccion no ha cambiado ningun fichero de `docs/`. El diff de la correccion (ADR:417, ADR:651-653)
es el que describe la peticion.

**Fallo que abre la correccion.** Buscado: la celda nueva no contradice H9, RR-08, la
divergencia 4 ni el estado de la decision (ver arriba). No se ha encontrado ningun fallo nuevo
introducido por el diff.

**Verificacion ejecutada en esta ronda.**
- `bash .github/scripts/comprobar-repo.sh`: «Tareas: 93; enlaces relativos: 62; fallos: 0», salida 0.
- PoC `tareas/03-en-revision/TASK-009/poc/poc-f1-8.sh` con el ADR de esta rama, en una carpeta
  temporal de `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-009-r3`, salida 0. Resultados
  identicos a la ronda 2: A1 7/7 (controles negativos 8/8, limite 1/1); B1 positivos 5/5 y
  negativos 20/20; B2 decision 7 10/10 (falsos que aceptaria F1-7: 4; solo job publicador: 2);
  C errores ADR real 0, activos 16, mutaciones 14/14, no aplicadas 0; D 16/16; llamadas `gh api`
  con GET 15, con otro metodo 0.
- Carpeta temporal borrada al terminar. No se ha hecho ningun commit.
- No hay suite de tests de codigo en esta rama (solo documentacion y tareas); la verificacion
  es la PoC y `comprobar-repo.sh`, no `taskctl doctor` (no ejecutado).

## Notas (fuera de la tabla)

Estas notas no son hallazgos de la ronda: ninguna es CRITICO ni la introduce la correccion. Se dejan
para que quien coordine decida si se corrigen.

1. **Dos afirmaciones previas contradicen la celda nueva (ADR:81 y ADR:395-396).** La fila A13
   dice «la persona (el agente no tiene `workflows`, F1-6)», y el texto de A8 dice «lo contienen
   CODEOWNERS en `/.github/` y que el agente no tenga `workflows`». El token de la persona tiene el
   scope `workflow`, asi que ambas afirman sin salvedad algo que la celda 417 ya desmiente. No cambian
   el veredicto porque A5 ya es riesgo abierto CRITICO y el ADR lo declara en su celda, pero conviene
   aplicarles la misma salvedad A4/A5. La ronda 2 no las listo.
2. **«Publicar checks con su identidad (sin `checks:write`)» (ADR:417).** La causa que se da es un
   permiso de App, y el token observado es clasico (`gist`, `read:org`, `repo`, `workflow`), sin
   ningun scope de checks. Es plausible, pero no esta probado para el token de la persona. La
   salvedad de la celda lo cubre; sugerencia: marcarlo como «no verificado», como proponia IMP-4.
3. **Columna «No puede» con un elemento que hoy si puede (ADR:417).** «Tocar workflows» figura bajo
   «No puede (y quien lo impide)» y a continuacion se dice que el token si puede. El texto es
   coherente, pero quien escanee la columna lee lo contrario. Sugerencia: redactarlo como «hoy
   puede con el token (A5)».
4. **Cabecera de commit.** La peticion pone HEAD `1592049…` como commit revisado, y el HEAD actual
   es `49ec1c2` (solo anade la peticion de la ronda 3). Entre ambos no cambia ningun fichero de
   `docs/`, asi que la revision es valida. La cabecera se deja como viene.
5. **No comprobado:** la afirmacion de la celda «la App no tiene `workflows`, F1-6» se toma del ADR
   y no se ha leido F1-6 (fuera del contexto permitido).

## Lecturas fuera del contexto permitido

- `docs/adr/F1-8-seguridad-modelo-de-amenazas.md` (lineas 12-16, 60-90, 392-430, 540-570, 645-670, y
  grep de `workflows`, `scope`, `A5`): es el fichero de la peticion, pero se leyeron lineas fuera de
  los apartados que la peticion nombra, para comprobar que el resto del ADR no contradice la celda.
- `tareas/03-en-revision/TASK-009/revision/informe-revision-2.md` y `peticion-revision-3.md`: los
  hallazgos a verificar y las instrucciones de la ronda, que la peticion pide leer.
- `C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.1\skills\task-workflow\revision.md`:
  reglas comunes de la revision por pares (plantilla del informe, rondas, linea del veredicto),
  que la skill revisora exige leer antes de empezar.
- `tareas/03-en-revision/TASK-009/poc/poc-f1-8.sh`: solo ejecutado, no leido (la peticion lo pide).
- `gh auth status`: solo la linea de scopes (`Token scopes`), sin mostrar el token.

### MEN-7 — Nota de la revision 3 registrada como MENOR por quien orquesta

ADR:81 (A13) y ADR:395-396 (A8) siguen afirmando sin salvedad que el agente no tiene `workflows`, en contradiccion con la celda de ADR:417 (salvo A4/A5: el token de la persona tiene el scope `workflow`). Correccion: aplicarles la salvedad A5.
