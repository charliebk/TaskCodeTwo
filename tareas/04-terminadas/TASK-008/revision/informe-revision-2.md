# Informe de revision — TASK-008 (ronda 2)

- Commit revisado: 78a62a2d0b376ed31451a0409ad556c0c661e11b
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md:355 | plan |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md:323 | plan |

Nuevos: sin hallazgos nuevos. Ningun CRITICO, y la correccion no abre ningun fallo que se pueda reproducir (ver verificacion por hallazgo).

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### Verificacion por hallazgo

**IMP-1 — corregido (verificado).** Antes, el acierto de `completa` de `develop` daba verde el check de la release sin correr la pesada. Ahora la busqueda es por perfil («Un acierto solo exime de su perfil», linea 355) y la tabla de eventos exige la pesada en la release con su propia clave. Reproducido con la PoC B: el caso negativo «PR de release (completa + pesada): la completa acierta, la pesada no corrio nunca» sale `fallo` (la pesada se ejecuta), y el caso «re-ejecucion del PR de release» sale `acierto` solo cuando la pesada tiene su propio check sobre el mismo arbol. Salida real: `RESULTADO B casos=11/11 ; controles negativos=12/12`.

**IMP-2 — corregido (verificado en la PoC; la parte de GitHub Actions, no verificada).** La clave ahora es `sha256(perfil | runner | huella del arbol | version)` (linea 323 y siguientes) y cada linea del resumen lleva `local=`. `buscar` solo filtra por la parte del arbol; `verificacion` corre siempre en el runner del perfil y acierta solo si su huella local coincide con la anotada en un candidato. Reproducido en la PoC B: «misma huella del arbol, otra huella local en el runner» → `fallo`; «huella local sin calcular en el runner del perfil» → `fallo`; «verde de otro runner (self-hosted) para la completa declarada en runner de GitHub» → `fallo`. Todos los controles negativos salen como se esperaba (12/12). Que la huella del arbol se pueda calcular en `buscar` (Linux) y que el job del runner la calcule en GitHub Actions sigue como «No verificado» en el ADR (lineas 273-278), como corresponde.

**Revision de la correccion (fallo que abre el diff).** Comprobado lo que pediste:

1. Release y completa reutilizada. La busqueda a dos niveles desde el merge ref de la release llega al check de la tarea (`f1`/`f_n`) porque `V -> head de develop -> segundo padre (head del PR)` es el nivel 2. Caso reproducido en la PoC A/B: el caso «PR de release, solo el perfil completa» da `acierto` con `completa reutilizada`. Eso es correcto: el arbol es el mismo, el perfil, el runner y la clave son los mismos, y el check es verde y de la App 15368.
2. Ausencia de check en `push` a `develop`/`main` (fila nueva de la tabla de eventos, linea 373). Recorrido: el `push` de la release a `main` (`R`) tiene como padre `m_n` (head de develop) que si lleva check de la release, y el caso «consulta tras el push del merge de la release en main» da `acierto`. Un `push` directo a `develop` sin PR no tiene check en el arbol ni en sus padres, y la busqueda devuelve vacio: la release ejecuta la completa. Es seguro (no hay falso acierto); cuesta una ejecucion, y eso es lo que el ADR dice.
3. No se abre falso acierto. Todos los candidatos exigen: `conclusion == success`, `app_id == 15368`, `name == verificacion`, `arbol` igual al del merge ref de la release, `perfil`, `runner` y `clave` iguales, y `local` igual al de la huella calculada en el runner. Los controles negativos (otro perfil, otra huella del arbol, otra huella local, sin huella local, otro runner, otra version, rojo, otra App, otro nombre, sin `arbol=`, tres niveles) salen como fallo. No se encuentra un camino en el que un check de otro arbol o de otro perfil se reutilice.

### Nota (no es hallazgo: no abre ronda)

- **Precondicion de la reutilizacion de `completa` en la release, no verificada.** La decision 1 (linea 316) dice que `completa` y `pesada` no pueden variar con `RAMA_BASE`, y el ADR reconoce en «No verificado» (linea 279) que la `completa` de ToolsaWeb planifica por grupos con un clasificador que lee la base. La clave no incluye la base. Por tanto, el acierto de la `completa` de `develop` en el PR de la release es correcto solo si F1-11 y F5-1 cumplen esa precondicion antes de cachear; la PoC B no lo simula (ningun caso con el mismo arbol y otra base). Sugerencia: enlazar esa precondicion desde la fila de la release de la tabla de eventos, o anadir a la PoC B un caso «mismo arbol, distinta base» que espere `fallo` mientras no se cumpla. Lo dejo como nota porque el ADR ya lo declara pendiente.

## Verificacion realizada

- Commit revisado: 78a62a2 (la peticion lo fija; el HEAD de la rama es e84be4a, que solo anade la peticion de la ronda 2). Diff revisado: `fee7038..78a62a2`, ficheros `docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md` y la PoC `poc-f1-7.sh` (cambiada en el commit `fee7038`).
- Suite del proyecto (no hay suite de codigo: el repo es documental). Puerta: `bash .github/scripts/comprobar-repo.sh` → `Tareas: 93; enlaces relativos: 55; fallos: 0`, salida 0.
- PoC ejecutada por mi en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-008-r2` (log en el mismo arbol, borrada al terminar). Salida 0, sin fallos:
  - A: `casos=10/10 como se esperaba`.
  - B: `casos=11/11 ; controles negativos=12/12`.
  - C1: `comprobaciones=9/9`.
  - C2: `errores ADR real=0 ; mutaciones detectadas=8/8 ; no aplicadas=0`. Mutaciones M6 (E4 por la medida en caliente de la misma linea) y M7 (cifra de la lectura) detectadas por su error esperado; M8 (E3 por el promedio anterior) detectada por el contexto de E3.
  - Llamadas: `gh api con GET=1 ; otro metodo=0 ; descargas GET=3`.
- No ejecutado: `taskctl doctor` (no pedido en esta ronda). No se escribio en GitHub ni en los repos externos de ToolsaWeb y OpenDockTrading.
- Los MENOR de la ronda 1 (MEN-1, MEN-2, MEN-3) no se han reverificado, como pide la peticion.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-008/poc/poc-f1-7.sh`: para reproducir B y C2 y comprobar los casos negativos (motivo: la peticion lo ejecuta, el contexto permitido no lo lista).
- `tareas/03-en-revision/TASK-008/revision/informe-revision-1.md`: para ver el hallazgo original y su reproduccion (motivo: contexto de IMP-1 e IMP-2).
- `C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.1\skills\task-workflow\revision.md`: para la plantilla del informe y la linea del veredicto (motivo: forma del informe).
- `tareas/03-en-revision/TASK-008/revision/peticion-revision-2.md`: es la peticion, leida por completo en el contexto permitido.
