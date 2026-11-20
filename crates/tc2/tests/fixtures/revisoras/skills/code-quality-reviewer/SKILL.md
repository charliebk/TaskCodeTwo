---
name: code-quality-reviewer
description: Fixture de F6-3 con el bloque yaml de la revisora del plugin taskcode 0.16.2 (skills/code-quality-reviewer/SKILL.md), copiado tal cual.
---

# Revisor generico de calidad (fixture)

```yaml
rol: revisor
patrones_archivo: []       # ninguno: no reclama ficheros por extension
fallback: true             # se dispara cuando ningun otro patron casa
umbral_dominios: 3         # y tambien cuando casan mas de 3 dominios distintos
```
