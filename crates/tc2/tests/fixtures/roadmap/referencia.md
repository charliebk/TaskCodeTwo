# Roadmap de tareas

<!-- generado por tc2; formato 2; no editar fuera de la zona manual -->

<!-- manual:inicio (a mano) -->
## Hitos

### H2 — Dos
El motivo.
### H1 — Uno
### H3 — Tres
<!-- manual:fin -->

## Estado del trabajo

Orden en que se hacen las tareas: ninguna va antes que aquellas de las que depende y, a igualdad, va primero la del hito anterior (en el orden de los `### Hn — título` de la zona manual) y luego la de número menor. tc2 lo regenera en cada cambio de la rama de estado; solo la zona manual se edita a mano.

- **En marcha:** TASK-008 (en diseño), TASK-002 (en curso), TASK-003 (en revisión)
- **Listas para empezar:** TASK-005
- **Atascadas (ciclo):** TASK-011, TASK-012
- **Ilegibles:** TASK-010
- **Dependencias inexistentes:** TASK-007>TASK-099
- **Dependen de una descartada:** TASK-007>TASK-006

## En curso

| Tarea | Título | Con quién | Hito | Depende de |
|---|---|---|---|---|
| TASK-008 | Con a \| b &lt;!-- y "comillas" #1 | alguien de fuera | H2 | — |
| TASK-002 | Motor | @ana | H1 | TASK-001 |

## En revisión

| Tarea | Título | Con quién | Hito | Depende de |
|---|---|---|---|---|
| TASK-003 | Revisar | @ana-x | H1 | — |

## H2 — Dos

0 de 6 terminadas.

| Paso | Tarea | Título | Depende de | Estado | Con quién |
|---|---|---|---|---|---|
| 1 | TASK-007 | Tras la descartada | TASK-006, TASK-099 | espera a TASK-006 (descartada) | — |
| 2 | TASK-008 | Con a \| b &lt;!-- y "comillas" #1 | — | en diseño | alguien de fuera |
| 4 | TASK-005 | Lista | TASK-001 | lista para empezar | — |
| 6 | TASK-004 | Usa el motor | TASK-002 | espera a TASK-002 | — |
| — | TASK-011 | Ciclo a | TASK-012 | atascada (ciclo) | — |
| — | TASK-012 | Ciclo b | TASK-011, TASK-010 | atascada (ciclo) | — |

```mermaid
flowchart TD
  T007["TASK-007 · Tras la descartada"]:::espera
  T008["TASK-008 · Con a | b #lt;!-- y #quot;comillas#quot; #35;1"]:::marcha
  T005["TASK-005 · Lista"]:::lista
  T004["TASK-004 · Usa el motor"]:::espera
  T011["TASK-011 · Ciclo a"]:::atascada
  T012["TASK-012 · Ciclo b"]:::atascada
  T001["TASK-001"]:::terminada
  T002["TASK-002"]:::borde
  T006["TASK-006"]:::descartada
  T010["TASK-010 · (ilegible)"]:::borde
  T006 --> T007
  T001 --> T005
  T002 --> T004
  T012 --> T011
  T011 --> T012
  T010 --> T012
  classDef lista fill:#d4f7d4,stroke:#2e7d32
  classDef marcha fill:#d6e9ff,stroke:#1565c0
  classDef espera fill:#f3f3f3,stroke:#888888
  classDef atascada fill:#ffd6d6,stroke:#c62828
  classDef terminada fill:#e8e8e8,stroke:#9e9e9e,color:#555555
  classDef descartada fill:#ffffff,stroke:#bbbbbb,stroke-dasharray:4 3,color:#888888
  classDef borde fill:#ffffff,stroke:#999999,stroke-dasharray:2 2
```

## H1 — Uno

1 de 3 terminadas.

| Paso | Tarea | Título | Depende de | Estado | Con quién |
|---|---|---|---|---|---|
| 5 | TASK-002 | Motor | TASK-001 | en curso | @ana |
| 7 | TASK-003 | Revisar | — | en revisión | @ana-x |

```mermaid
flowchart TD
  T002["TASK-002 · Motor"]:::marcha
  T003["TASK-003 · Revisar"]:::marcha
  T001["TASK-001"]:::terminada
  T001 --> T002
  classDef marcha fill:#d6e9ff,stroke:#1565c0
  classDef terminada fill:#e8e8e8,stroke:#9e9e9e,color:#555555
```

## Terminadas

Total: 2.

<details>
<summary>Ver la lista</summary>

- TASK-001 — Base (H1)
- TASK-009 — Cerrada (H3)

</details>

## Descartadas

Total: 1.

<details>
<summary>Ver la lista</summary>

- TASK-006 — Ya no (H2) — motivo: ya no hace falta

</details>
