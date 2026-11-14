# TaskCodeTwo — contexto para Claude Code

TaskCode 2, construido con TaskCode 0.16 (requisito rector RR-14). Antes de
tocar nada, lee [`docs/plan/PLAN-TASKCODE-2.md`](docs/plan/PLAN-TASKCODE-2.md)
y [`docs/plan/REQUISITOS-FUNCIONALES.md`](docs/plan/REQUISITOS-FUNCIONALES.md):
los requisitos rectores (§0 y §2) están por encima de cualquier otra decisión.

## Idioma

Código, documentación, mensajes de commit y respuestas en **español**. Los
mensajes de commit sin tildes (los scripts de Git-Flow de TaskCode 0.16 los
procesan).

## Cómo se trabaja aquí (con TaskCode 0.16)

1. **Cada cambio es una tarea de `tareas/`** y sigue su ciclo con `taskctl`:
   `plan` → `approve` → `start` → `review` → `finish`. Rama por tarea, merge en
   `develop` con `--no-ff`; las ramas no se borran.
2. **Revisión por pares obligatoria** por un agente independiente que reproduce
   empíricamente. Hallazgos CRÍTICO / IMPORTANTE / MENOR documentados en el
   `## Resultado` de la tarea, también los que no se corrigen.
3. **Evidencia, no suposición.** Si no se ha ejecutado, no se afirma.
4. **Las tareas de la Fase 1 son decisiones (ADR)**: cada una deja un documento
   en `docs/adr/` con alternativas, decisión y motivo; no se escribe código de
   la 2.0 hasta cerrar la Fase 1.
5. Los ficheros que se pasan a `taskctl import` viven **fuera del repo**.

## Comandos

```bash
taskctl doctor
taskctl board
taskctl roadmap
```
