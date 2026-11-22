## Revision de `src/calculo.rs`

La media nueva tiene la condicion al reves y el tope no tiene test. Hay un
hallazgo sin evidencia y otro que cita una linea que no esta en el diff:
los dos tienen que salir rechazados.

```json
[
  {
    "id": "H1",
    "severidad": "CRITICO",
    "clase": "correccion",
    "fichero": "src/calculo.rs",
    "linea": 5,
    "descripcion": "La condicion esta al reves: con dos o mas valores devuelve None.",
    "evidencia": {
      "tipo": "borde",
      "entrada": "media(&[2.0, 4.0])",
      "esperado": "Some(3.0)",
      "obtenido": "None"
    }
  },
  {
    "id": "H2",
    "severidad": "IMPORTANTE",
    "clase": "contrato",
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
    "id": "H3",
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
  },
  {
    "id": "H4",
    "severidad": "MENOR",
    "clase": "correccion",
    "fichero": "src/calculo.rs",
    "linea": 3,
    "descripcion": "El comentario podria ser mas claro."
  },
  {
    "id": "H5",
    "severidad": "IMPORTANTE",
    "clase": "contrato",
    "fichero": "src/calculo.rs",
    "linea": 40,
    "descripcion": "Linea que la revisora no ha visto.",
    "evidencia": {
      "tipo": "borde",
      "entrada": "0",
      "esperado": "1",
      "obtenido": "2"
    }
  }
]
```
