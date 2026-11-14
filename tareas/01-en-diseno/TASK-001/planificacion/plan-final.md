# Plan — TASK-001: F1-0 Crear TaskCodeTwo

(Esta tarea no lanza brainstorm: su complejidad resuelve cero roles.
El plan se redacta directamente a partir del enunciado.)

## Estado comprobado antes de planificar (2026-11-13)

| Pide F1-0 | Estado | Evidencia |
|---|---|---|
| Repo en GitHub, público, MIT | hecho | `gh repo view charliebk/TaskCodeTwo` → `PUBLIC`; `LICENSE` MIT |
| `develop` y `main` | hecho | `git branch -a` lista las dos en local y en `origin`; rama por defecto `develop` |
| TaskCode 0.16 instalado | hecho | `taskctl --version` → `0.16.1`; `doctor` → `version-plugin` ok |
| `tareas/` y config | hecho | cinco carpetas de estado; `.taskcode/config.yml` válida (`modo_flujo: automatico`) |
| Plan importado con dependencias | hecho | 72 tareas (TASK-001..072 = F1-0..F9-9). Un script de comparación (fuera del repo) cruza la columna «Dep.» del plan con `dependencias:` de cada tarea: 0 diferencias reales; las 3 que marca son comodines (`F1-*`, `F3-*`, `F7-*`) que el script no expande y que en las tareas están bien expandidos |
| `taskctl doctor` en verde | hecho | 0 errores, 0 avisos, 15 ok, 4 omitidas. El `TaskCodeTwo.iml` del IDE ya se ignora (`*.iml` en `.gitignore`, commit `b23f213` en `develop`) |
| CI mínima | **falta** | no existe `.github/` |

## Historial de esta ronda

Dos terminales abrieron el diseño a la vez (`3481d36` y `c117c4b`, ronda 2).
Una aprobación hecha desde el otro terminal (`df23f4f`) no correspondía a la
ronda vigente y se revirtió con un commit nuevo (`5cded15`), sin reescribir el
historial. Este plan parte del que redactó aquel terminal, actualizado.

## Enfoque propuesto

Duracion estimada: 30 min

Señales: de la rama abierta al cierre con merge; 2 ficheros nuevos de CI (un
workflow y un script de shell), 1 capa, 3 criterios; sin código de la 2.0.

1. **CI mínima** que valida `tareas/` y la documentación, decidida por Carlos
   el 2026-11-13 (ver «Decisiones»). No anticipa nada de las ADR: la CI de
   verdad es F1-7/F5-1 y la de código, F2-1.
   - `.github/workflows/ci.yml`: en `push` a `develop`/`main` y en
     `pull_request` contra ellas; un job en `ubuntu-latest`,
     `permissions: contents: read`, `actions/checkout` fijado por SHA, sin
     secretos ni acciones de terceros más allá de checkout.
   - `.github/scripts/comprobar-repo.sh` (bash, ejecutable también en local
     con el bash de Git). Comprueba:
     a) que hay las cinco carpetas `tareas/00-planificadas .. 04-terminadas`;
     b) que cada `tareas/*/TASK-NNN/tarea.md` abre con frontmatter, su `id`
        coincide con la carpeta y es único, y su `estado` corresponde a la
        carpeta de estado;
     c) que cada dependencia citada existe como tarea;
     d) que cada enlace relativo de los `.md` de `docs/`, `README.md` y
        `CLAUDE.md` apunta a un fichero que existe.
     Sale con código ≠ 0 y escribe una línea por fallo.
   - Mientras `tareas/` siga en las ramas de código (0.16), las comprobaciones
     a) a c) tienen sentido aquí. Cuando la 2.0 lleve las tareas a su rama de
     estado (F1-1), se moverán o se retirarán.
2. Subir la rama y comprobar que el workflow pasa en GitHub (`gh run list`).
3. `develop` local va por delante de `origin/develop` (ROADMAP, dependencias,
   `.iml`, commits de esta tarea): se publica con el cierre.

## Riesgos aceptados y que los contiene

- La CI puede quedar en rojo por un enlace roto que ya exista en `docs/`:
  se arregla en esta misma tarea (forma parte de «CI verde»).
- El SHA de `actions/checkout` se toma de la API de GitHub (`gh api`), no de
  memoria.
- El script es bash: aceptable para la 0.16 (que ya exige bash); la 2.0
  quitará bash (P12) y su CI la define F2-1.
- El parseo del frontmatter en bash es por líneas (`clave: valor`), no un
  parser YAML completo: basta para los campos que se comprueban; la
  validación YAML estricta es F2-5.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- `taskctl doctor` en verde → 1 comprobación: `taskctl doctor` con 0 errores
  y 0 avisos al cerrar.
- Tareas F1..F9 planificadas con dependencias → 1: `comprobar-repo.sh` pasa
  sobre el repo real (72 tareas, todas las dependencias existen); la
  comparación con el plan queda en el `## Resultado`.
- CI mínima → 2: prueba negativa en una copia temporal (scratchpad, fuera del
  repo), donde el script tiene que fallar con un enlace roto, una dependencia
  inexistente y una tarea en una carpeta que no es la de su estado; y el
  workflow tiene que salir verde en GitHub sobre la rama.
- Ninguno pesado.

## Lo que necesita decision de una persona

Decidido por Carlos el 2026-11-13:

- Repo público, licencia MIT (ya lo fijaba el plan, §7).
- Alcance de la «CI mínima»: **validar `tareas/`** (carpetas, frontmatter,
  IDs, estado y dependencias), más los enlaces de la documentación. Se
  descartaron el workflow vacío y dejar F1-0 sin CI.

No queda nada abierto.
