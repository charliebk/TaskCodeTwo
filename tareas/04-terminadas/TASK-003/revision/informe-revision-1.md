# Informe de revision — TASK-003 (ronda 1)

- Commit revisado: 98746546804291bc7e1a18cc2fc2616a84dc8cd1
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

(HEAD real de la rama: c4d757e, que solo anade la peticion de revision. Alcance completo. Pase generico: ADR + guion de PoC.)

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md:350 | implementacion |
| MEN-2 | MENOR | abierto | docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md:267 | implementacion |
| MEN-3 | MENOR | abierto | docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md:216 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

No hay CRITICO ni IMPORTANTE.

### MEN-1 — La pregunta abierta sobre hooks describe un comportamiento que la PoC no tiene
- Donde: ADR:350; en `poc-f1-2.sh`, el bucle de `tx` detecta el rechazo con `grep -qE 'rejected|fetch first|non-fast-forward'`.
- Que pasa: el ADR dice que un hook o regla que rechaza el push por otro motivo «hoy cae en sin red». En la PoC cae en contencion: 5 reintentos y `Agotada` (codigo 5).
- Reproduccion: bare con `pre-receive` que hace `exit 1`; `git push` → `! [remote rejected] ... (pre-receive hook declined)`; el grep coincide por `rejected`; no pasa por `sin_red`.
- Impacto: `Agotada` significa «reintenta mas tarde»; un rechazo permanente (rama protegida, hook) haria reintentar sin fin. F2-3 debe separar non-fast-forward de cualquier otro rechazo con codigo propio. MENOR: la pregunta ya esta abierta y asignada a F2-3.

### MEN-2 — El ADR sustituye el «rebase» de P2 y de F1-1 sin decirlo
- Donde: ADR:267 («nunca `rebase`»).
- Que pasa: P2 dice «el roadmap se regenera tras el rebase»; F1-1 (linea 133 y PoC A.5) tambien. F1-2 decide re-ejecutar (D1a), justificado con la PoC 3a/3b, pero no dice que matiza P2 ni F1-1.
- Reproduccion: lectura cruzada de PLAN §P2 (lineas 91-96), ADR F1-1 (lineas 65 y 133) y ADR F1-2 (linea 267).
- Impacto: quien implemente F2-3 leyendo F1-1 puede hacer rebase. Correccion: anadirlo a «Divergencia con el plan».

### MEN-3 — La eleccion de 2 s se apoya en una muestra pequena y no declara la espera maxima
- Donde: ADR:216.
- Que pasa: 3 rondas por valor (36 altas); la diferencia entre 0,5 s (3 agotadas) y 2 s (1) esta dentro del ruido. En la repeticion del revisor: 2 s → 0, 0,5 s → 3, 0,1 s → 4. Calibrado contra el guion (~2,5 s por transaccion), no el CLI real. No dice la espera acumulada maxima: 2·(1+2+4+8) = 30 s mas las transacciones.
- Impacto: ninguno hoy (el valor es revisable en F2-3 y F9-4). Correccion: anotar n=3, diferencia no concluyente, y ~30 s de espera maxima.

## Verificacion realizada

- Puerta: `bash .github/scripts/comprobar-repo.sh` → «Tareas: 93; enlaces relativos: 21; fallos: 0».
- PoC reproducida: `poc-f1-2.sh` completo (`ESPERA_BASE=2`), Git 2.56.0.windows.4, en `$LOCALAPPDATA/pruebas-claude/taskcodetwo/task-003-r1`, ~8 min, «FIN PoC».
- Garantias (no varian): prueba 1 12/12, IDs consecutivos, ROADMAP con 12, 0 commits locales sin publicar, sin merges, `denyNonFastForwards=true`; 1b IDs consecutivos y estado limpio en las 9 rondas; prueba 2 gana ana, beto `Rechazada (la lleva ana)` codigo 3, idempotente; 3a reproduce H3 (`UU ROADMAP.md`, `AA TASK-001/tarea.md`), 3b sin conflicto; prueba 4 `SinRed` codigo 4, HEAD igual, worktree limpio; prueba 5 propia pendiente, ajena rechazada, re-ejecutada al volver la red; prueba 6 sin remoto funciona y el segundo `start` se rechaza.
- Lo que varia (esperado): reintentos prueba 1 10 (ADR 7); 1b 0,1 s 4 agotadas (ADR 3), 0,5 s 3 (3), 2 s 0 (1); transaccion sin contencion 2477 ms (2642).
- Citas del ADR contra `poc-f1-2.log`: coinciden. Lo no probado esta marcado «no verificado» (GitHub/GitLab, varias maquinas, CLI real, cerrojo).
- Divergencia de la base de espera: documentada y justificada (ver MEN-3).
- IDs citados: RR-01, RR-09, RR-12, RR-14, RR-15, RN-05, RN-06, RF-02, RF-07 en REQUISITOS; H3 en docs/auditoria/AUDITORIA-EXTERNA-2026-11-13.md:23; tareas F1-1, F1-3, F1-5, F1-6, F1-9, F2-3, F2-4, F2-4b, F2-13, F3-5b, F3-6, F3-10, F9-4 en la tabla del plan.
- Criterios: los tres cumplidos. Plan: D1a, D2b, D3a, D4a y la plantilla seguidos; no toca F1-1, .github, .taskcode ni la 0.16.
- Mutacion: no aplica (sin tests con aserciones); las comprobaciones discriminan (3a falla, 3b pasa).
- Repo sin cambios; temporales borrados.

## Lecturas fuera del contexto permitido

- La peticion y la plantilla de este informe; `revision.md` del plugin.
- docs/plan/PLAN-TASKCODE-2.md y REQUISITOS-FUNCIONALES.md; docs/auditoria/AUDITORIA-EXTERNA-2026-11-13.md (H3); docs/adr/F1-1 (coherencia).
- `poc/poc-f1-2.sh` y `poc-f1-2.log`; `.github/scripts/comprobar-repo.sh` (ejecutado).
