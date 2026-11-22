## Ronda 2 de `src/calculo.rs`

El tope ya tiene test: cierro I1. Aparece un detalle nuevo.

```json
{
  "hallazgos": [
    {
      "id": "N1",
      "severidad": "MENOR",
      "clase": "correccion",
      "fichero": "src/calculo.rs",
      "linea": 5,
      "descripcion": "La condicion se lee mejor con is_empty.",
      "evidencia": {
        "tipo": "borde",
        "entrada": "media(&[])",
        "esperado": "None",
        "obtenido": "Some(NaN)"
      }
    }
  ],
  "cierres": [
    {
      "id": "I1",
      "fichero": "src/calculo.rs",
      "linea": 12,
      "motivo": "El tope de 100 ya tiene su test."
    }
  ]
}
```
