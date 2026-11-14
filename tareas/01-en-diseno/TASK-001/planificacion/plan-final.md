
# Plan — TASK-001: F1-0 Crear TaskCodeTwo

(Esta tarea no lanza brainstorm: su complejidad resuelve cero roles.
El plan se redacta directamente a partir del enunciado.)

## Estado comprobado antes de planificar (2026-11-13)

| Pide F1-0 | Estado | Evidencia |
|---|---|---|
| Repo en GitHub, público, MIT | hecho | `gh repo view charliebk/TaskCodeTwo` → `PUBLIC`, licencia `mit` |
| `develop` y `main` | hecho | `git ls-remote --heads origin` lista las dos; rama por defecto `develop` |
| TaskCode 0.16 instalado | hecho | `taskctl --version` → `0.16.1`; `doctor` → `version-plugin` ok |
| `tareas/` y config | hecho | cinco carpetas de estado; `.taskcode/config.yml` válida (`modo_flujo: automatico`) |
| Plan importado con dependencias | hecho | 72 tareas (TASK-001..072 = F1-0..F9-9), sprint = fase; las dependencias coinciden con la columna «Dep.» del plan, comodines incluidos |
| CI mínima | **falta** | no existe `.github/` |
| `taskctl doctor` en verde | **aviso** | `workspace`: `TaskCodeTwo.iml` (fichero del IDE) sin ignorar |

## Enfoque propuesto

Duracion estimada: 35 min

Señales: de la rama abierta al cierre con merge; 2 ficheros nuevos de CI (un
workflow y un script de shell), 1 línea en `.gitignore`, 1 capa, 3 criterios;
sin código de la 2.0.

1. `.gitignore`: añadir `*.iml` (ficheros de proyecto de IntelliJ), junto a
   `.idea/`, que ya está. Así `doctor` queda sin el aviso de workspace.
2. **CI mínima** que no anticipa decisiones de las ADR (la CI de verdad es
   F1-7/F5-1 y la de código, F2-1):
   - `.github/workflows/ci.yml`: en `push` a `develop`/`main` y en
     `pull_request` contra ellas; un job `docs` en `ubuntu-latest`,
     `permissions: contents: read`, `actions/checkout` fijado por SHA,
     sin secretos ni acciones de terceros más allá de checkout.
   - `.github/scripts/comprobar-repo.sh` (bash, ejecutable también en local
     con el bash de Git): comprueba
     a) que cada enlace relativo de los `.md` de `docs/`, `README.md` y
        `CLAUDE.md` apunta a un fichero que existe;
     b) que cada `tareas/*/TASK-NNN/tarea.md` abre con frontmatter, su `id`
        coincide con la carpeta y su `estado` con la carpeta de estado;
     c) que cada dependencia citada existe como tarea.
     Sale con código ≠ 0 y una línea por fallo.
   - Mientras `tareas/` siga en las ramas de código (0.16), las comprobaciones
     b) y c) tienen sentido aquí; cuando la 2.0 lleve las tareas a su rama de
     estado (F1-1), se moverán o retirarán.
3. Subir la rama y comprobar que el workflow pasa en GitHub (`gh run list`).
4. `develop` local va 3 commits por delante de `origin/develop` (ROADMAP y
   dependencias de la sesión anterior): se publican con el cierre.

## Riesgos aceptados y que los contiene

- La CI puede quedar en rojo por un enlace roto ya existente en `docs/`:
  se arregla el enlace en esta misma tarea (es parte de «CI verde»).
- El SHA de `actions/checkout` se toma de la API de GitHub (`gh api`), no de
  memoria.
- El script es bash: aceptable para la 0.16 (que ya exige bash); la 2.0
  quitará bash (P12) y su CI la define F2-1.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- `taskctl doctor` en verde → 1 comprobación: `taskctl doctor` con 0 errores
  y 0 avisos tras el cambio (sin la exclusión local de `.git/info/exclude`).
- Tareas F1..F9 planificadas con dependencias → 1: `comprobar-repo.sh`
  pasa (todas las dependencias existen) y la comparación con el plan
  hecha arriba queda en el `## Resultado`.
- CI mínima → 2: el script falla con un enlace roto y con una dependencia
  inexistente provocados a propósito en una copia temporal (prueba negativa),
  y el workflow sale verde en GitHub sobre la rama.
- Ninguno pesado.

## Lo que necesita decision de una persona

Nada: el plan ya fija repo público y MIT (Carlos, 2026-11-13); la CI mínima
no decide nada que pertenezca a una ADR.
