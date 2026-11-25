# MENORES pendientes — TASK-147

MENORES abiertos de la ronda 1 que no se corrigieron en la verificacion ligera (`taskctl verificar-menores`). Son backlog de la tarea: no abren otra ronda ni impiden `finish`.

| ID | Fichero | Motivo |
|---|---|---|
| MEN-2 | .github/scripts/probar-instalador.sh:48-51 y 124-125 (ramas `wget` de `bajar` y `ultima_estable`) | MEN-2 necesita un wget real y no hay wget ni Docker en la maquina (decision de Carlos); MEN-3 aceptado: la linea da un mensaje mas claro |
