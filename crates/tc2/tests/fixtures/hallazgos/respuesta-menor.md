## Revision de `src/calculo.rs`

Solo un detalle menor.

```json
[
  {
    "id": "M1",
    "severidad": "MENOR",
    "clase": "correccion",
    "fichero": "src/calculo.rs",
    "linea": 8,
    "descripcion": "Con la lista vacia divide entre cero.",
    "evidencia": {
      "tipo": "reproduccion",
      "pasos": ["llamar a media(&[])"],
      "fallo_esperado": "devuelve Some(NaN) en vez de None"
    }
  }
]
```
