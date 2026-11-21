## Revision de `src/calculo.rs`

El tope no tiene test (IMPORTANTE, con evidencia). El CRITICO de la media
no trae evidencia: se rechaza y no puede poner el check en rojo.

```json
[
  {
    "id": "I1",
    "severidad": "IMPORTANTE",
    "clase": "fidelidad-evidencia",
    "fichero": "src/calculo.rs",
    "linea": 12,
    "descripcion": "Ningun test fija el tope de 100.",
    "evidencia": {
      "tipo": "mutante",
      "linea_original": "x.min(100)",
      "mutacion": "x.min(101)",
      "test": "tope_corta_en_100"
    }
  },
  {
    "id": "C1",
    "severidad": "CRITICO",
    "clase": "correccion",
    "fichero": "src/calculo.rs",
    "linea": 5,
    "descripcion": "La condicion esta al reves."
  }
]
```
