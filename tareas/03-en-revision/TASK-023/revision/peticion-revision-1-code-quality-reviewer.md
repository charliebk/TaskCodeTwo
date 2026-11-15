# Peticion de revision — TASK-023 (ronda 1)

- Tarea: TASK-023 — F2-7 Configuracion minima con esquema validado y valores por defecto
- Rama revisada: feature/task-023-f2-7-configuracion-minima-con-esquema-va
- Rama base: develop
- Commit revisado (HEAD): 25e7d2ce796319aac3b8ab5b4d26518c6c197f78
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-023 (criterios de aceptacion y plan)
- Alcance: completa (motivo: primera ronda de revision)

## Instrucciones para el agente revisor

Eres un revisor INDEPENDIENTE del agente que implemento. Tu trabajo es
reproducir empiricamente, no leer el diff y opinar: clona el repo a un
directorio temporal, corre la suite tu mismo (salvo que esta
peticion ya traiga la verificacion de la ronda en verde para el commit revisado;
entonces vale ese resultado) y construye el caso que
rompe el codigo antes de reportarlo. Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. Vuelca tu salida en el
informe de esta ronda (informe-revision-1-code-quality-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-023/tarea.md
- tareas/03-en-revision/TASK-023/planificacion/
- .gitattributes
- crates/tc2/esquemas/config.schema.json

## Commits a revisar (git log develop..HEAD)

````
25e7d2c chore(TASK-023): autorrevision
29cf97f feat(TASK-023): configuracion minima con esquema validado y valores por defecto
cb57d49 chore(TASK-023): tarea en curso
````

## Diff de tu dominio (2 fichero(s) de 11; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/.gitattributes b/.gitattributes
index 85de2bc..5389f20 100644
--- a/.gitattributes
+++ b/.gitattributes
@@ -5,3 +5,6 @@
 *.rs text eol=lf
 *.toml text eol=lf
 Cargo.lock text eol=lf
+# Esquema publicado de la configuracion (F2-7): se compara byte a byte con
+# el que genera el codigo.
+crates/tc2/esquemas/*.json text eol=lf
diff --git a/crates/tc2/esquemas/config.schema.json b/crates/tc2/esquemas/config.schema.json
new file mode 100644
index 0000000..b35ff50
--- /dev/null
+++ b/crates/tc2/esquemas/config.schema.json
@@ -0,0 +1,816 @@
+{
+  "$schema": "https://json-schema.org/draft/2020-12/schema",
+  "additionalProperties": false,
+  "description": "Generado por tc2 (config::esquema_json). La primera linea del fichero es ---. Una clave o un valor desconocido falla cerrado.",
+  "properties": {
+    "confianza_incluye": {
+      "default": [],
+      "description": "ficheros que suma la huella de confianza [F1-11 decision 1; F1-8 decision 5]",
+      "items": {
+        "description": "glob relativo: sin negacion (!), rutas absolutas ni .., y nunca todo el arbol (**, *)",
+        "minLength": 1,
+        "type": "string"
+      },
+      "type": "array"
+    },
+    "derivados": {
+      "default": [],
+      "description": "comandos de las transiciones que el check estado repite [F1-11 decision 1]",
+      "items": {
+        "additionalProperties": false,
+        "properties": {
+          "comando": {
+            "description": "que ejecuta. orden no vacia y sin caracteres de control (rutas de Windows entre comillas simples o con /)",
+            "minLength": 1,
+            "type": "string"
+          },
+          "escribe": {
+            "description": "lo unico que un genera puede escribir; obligatoria en genera, prohibida en comprueba",
+            "items": {
+              "description": "ruta normalizada bajo tareas/, sin \\, :, comodines, caracteres de control, nombres reservados de Windows ni .git; nunca tareas/ entera, tareas/ROADMAP.md ni tareas/TASK-NNN/ (sin distinguir mayusculas)",
+              "minLength": 1,
+              "type": "string"
+            },
+            "minItems": 1,
+            "type": "array"
+          },
+          "fases": {
+            "default": [
+              "new",
+              "plan",
+              "approve",
+              "start",
+              "review",
+              "finish",
+              "descartar",
+              "pausa"
+            ],
+            "description": "en que transiciones corre (por defecto, en todas)",
+            "items": {
+              "enum": [
+                "new",
+                "plan",
+                "approve",
+                "start",
+                "review",
+                "finish",
+                "descartar",
+                "pausa"
+              ],
+              "type": "string"
+            },
+            "minItems": 1,
+            "type": "array"
+          },
+          "timeout": {
+            "default": 60,
+            "description": "segundos",
+            "maximum": 600,
+            "minimum": 1,
+            "type": "integer"
+          },
+          "tipo": {
+            "description": "si escribe ficheros o solo comprueba",
+            "enum": [
+              "genera",
+              "comprueba"
+            ],
+            "type": "string"
+          }
+        },
+        "required": [
+          "comando",
+          "tipo"
+        ],
+        "type": "object"
+      },
+      "type": "array"
+    },
+    "formato": {
+      "description": "version del formato del proyecto; con 2 manda la 2.0 [F1-12 D4a, decision 6]",
+      "maximum": 2,
+      "minimum": 2,
+      "type": "integer"
+    },
+    "lote": {
+      "additionalProperties": false,
+      "description": "presupuesto por defecto de los lotes; sin cifras inventadas [F1-16, consecuencias]",
+      "properties": {
+        "presupuesto_horas": {
+          "description": "presupuesto del lote en horas",
+          "minimum": 1,
+          "type": "integer"
+        },
+        "presupuesto_tokens": {
+          "description": "presupuesto del lote en tokens",
+          "minimum": 1,
+          "type": "integer"
+        },
+        "tope_tokens_tarea": {
+          "description": "tope de tokens por tarea",
+          "minimum": 1,
+          "type": "integer"
+        }
+      },
+      "type": "object"
+    },
+    "modo_flujo": {
+      "default": "manual",
+      "description": "modo del flujo; lo lee tambien la 0.16 [F1-12 decision 1 (convivencia); RF-21]",
+      "enum": [
+        "manual",
+        "semiautomatico",
+        "automatico"
+      ],
+      "type": "string"
+    },
+    "plataforma": {
+      "additionalProperties": false,
+      "description": "plataforma de un host que no es github.com ni gitlab.com [F1-13 decision 7]",
+      "properties": {
+        "api": {
+          "description": "URL de la API. URL https de la API (o http://127.0.0.1:<puerto> en pruebas)",
+          "minLength": 1,
+          "type": "string"
+        },
+        "proveedor": {
+          "description": "plataforma del remoto",
+          "enum": [
+            "github",
+            "gitlab"
+          ],
+          "type": "string"
+        }
+      },
+      "required": [
+        "proveedor",
+        "api"
+      ],
+      "type": "object"
+    },
+    "politica_carriles": {
+      "additionalProperties": false,
+      "default": {},
+      "description": "endurece la politica por carril; solo los cuatro carriles [F1-10 decision 2]",
+      "properties": {
+        "errata": {
+          "additionalProperties": false,
+          "description": "endurece la fila del carril",
+          "properties": {
+            "lote": {
+              "const": false,
+              "description": "solo se puede sacar del lote"
+            },
+            "paradas_extra": {
+              "default": [],
+              "description": "paradas del automatico que se anaden",
+              "items": {
+                "enum": [
+                  "start",
+                  "review"
+                ],
+                "type": "string"
+              },
+              "type": "array"
+            },
+            "perfil": {
+              "description": "perfil de verificacion; no baja del suelo",
+              "enum": [
+                "completa",
+                "completa y pesada"
+              ],
+              "type": "string"
+            },
+            "plan_aprobado": {
+              "const": true,
+              "description": "solo se puede exigir, no quitar"
+            },
+            "revisoras_extra": {
+              "default": [],
+              "description": "revisoras que se anaden",
+              "items": {
+                "description": "texto no vacio, sin caracteres de control ni espacios en los extremos",
+                "minLength": 1,
+                "type": "string"
+              },
+              "type": "array"
+            },
+            "roles_diseno": {
+              "description": "roles de diseno minimos; no baja del suelo",
+              "maximum": 5,
+              "minimum": 0,
+              "type": "integer"
+            }
+          },
+          "type": "object"
+        },
+        "fix": {
+          "additionalProperties": false,
+          "description": "endurece la fila del carril",
+          "properties": {
+            "lote": {
+              "const": false,
+              "description": "solo se puede sacar del lote"
+            },
+            "paradas_extra": {
+              "default": [],
+              "description": "paradas del automatico que se anaden",
+              "items": {
+                "enum": [
+                  "start",
+                  "review"
+                ],
+                "type": "string"
+              },
+              "type": "array"
+            },
+            "perfil": {
+              "description": "perfil de verificacion; no baja del suelo",
+              "enum": [
+                "completa",
+                "completa y pesada"
+              ],
+              "type": "string"
+            },
+            "plan_aprobado": {
+              "const": true,
+              "description": "solo se puede exigir, no quitar"
+            },
+            "revisoras_extra": {
+              "default": [],
+              "description": "revisoras que se anaden",
+              "items": {
+                "description": "texto no vacio, sin caracteres de control ni espacios en los extremos",
+                "minLength": 1,
+                "type": "string"
+              },
+              "type": "array"
+            },
+            "roles_diseno": {
+              "description": "roles de diseno minimos; no baja del suelo",
+              "maximum": 5,
+              "minimum": 1,
+              "type": "integer"
+            }
+          },
+          "type": "object"
+        },
+        "funcionalidad": {
+          "additionalProperties": false,
+          "description": "endurece la fila del carril",
+          "properties": {
+            "lote": {
+              "const": false,
+              "description": "solo se puede sacar del lote"
+            },
+            "paradas_extra": {
+              "default": [],
+              "description": "paradas del automatico que se anaden",
+              "items": {
+                "enum": [
+                  "start",
+                  "review"
+                ],
+                "type": "string"
+              },
+              "type": "array"
+            },
+            "perfil": {
+              "description": "perfil de verificacion; no baja del suelo",
+              "enum": [
+                "completa",
+                "completa y pesada"
+              ],
+              "type": "string"
+            },
+            "plan_aprobado": {
+              "const": true,
+              "description": "solo se puede exigir, no quitar"
+            },
+            "revisoras_extra": {
+              "default": [],
+              "description": "revisoras que se anaden",
+              "items": {
+                "description": "texto no vacio, sin caracteres de control ni espacios en los extremos",
+                "minLength": 1,
+                "type": "string"
+              },
+              "type": "array"
+            },
+            "roles_diseno": {
+              "description": "roles de diseno minimos; no baja del suelo",
+              "maximum": 5,
+              "minimum": 1,
+              "type": "integer"
+            }
+          },
+          "type": "object"
+        },
+        "riesgo-alto": {
+          "additionalProperties": false,
+          "description": "endurece la fila del carril",
+          "properties": {
+            "lote": {
+              "const": false,
+              "description": "solo se puede sacar del lote"
+            },
+            "paradas_extra": {
+              "default": [],
+              "description": "paradas del automatico que se anaden",
+              "items": {
+                "enum": [
+                  "start",
+                  "review"
+                ],
+                "type": "string"
+              },
+              "type": "array"
+            },
+            "perfil": {
+              "description": "perfil de verificacion; no baja del suelo",
+              "enum": [
+                "completa",
+                "completa y pesada"
+              ],
+              "type": "string"
+            },
+            "plan_aprobado": {
+              "const": true,
+              "description": "solo se puede exigir, no quitar"
+            },
+            "revisoras_extra": {
+              "default": [],
+              "description": "revisoras que se anaden",
+              "items": {
+                "description": "texto no vacio, sin caracteres de control ni espacios en los extremos",
+                "minLength": 1,
+                "type": "string"
+              },
+              "type": "array"
+            },
+            "roles_diseno": {
+              "description": "roles de diseno minimos; no baja del suelo",
+              "maximum": 5,
+              "minimum": 4,
+              "type": "integer"
+            }
+          },
+          "type": "object"
+        }
+      },
+      "type": "object"
+    },
+    "recursos": {
+      "default": [],
+      "description": "recursos necesarios que el proyecto anade; solo anade [F1-14 D6a; F1-15 decision 6]",
+      "items": {
+        "additionalProperties": false,
+        "properties": {
+          "dominios": {
+            "default": [],
+            "description": "dominios de F1-15 que cubre",
+            "items": {
+              "enum": [
+                "puertas-seguridad",
+                "rendimiento"
+              ],
+              "type": "string"
+            },
+            "minItems": 1,
+            "type": "array"
+          },
+          "id": {
+            "description": "nombre de la skill, el agente o el ejecutable. texto no vacio, sin caracteres de control ni espacios en los extremos",
+            "minLength": 1,
+            "type": "string"
+          },
+          "rol": {
+            "description": "rol que cubre",
+            "enum": [
+              "revisor"
+            ],
+            "type": "string"
+          },
+          "tipo": {
+            "description": "que es",
+            "enum": [
+              "skill",
+              "agente",
+              "ejecutable"
+            ],
+            "type": "string"
+          }
+        },
+        "required": [
+          "id",
+          "tipo"
+        ],
+        "type": "object"
+      },
+      "type": "array"
+    },
+    "rutas_carril": {
+      "additionalProperties": false,
+      "default": {},
+      "description": "rutas que suben el carril minimo; solo fix y riesgo-alto [F1-10 decision 2]",
+      "properties": {
+        "fix": {
+          "description": "globs que suben a este carril",
+          "items": {
+            "description": "glob relativo: sin negacion (!), rutas absolutas ni ..",
+            "minLength": 1,
+            "type": "string"
+          },
+          "minItems": 1,
+          "type": "array"
+        },
+        "riesgo-alto": {
+          "description": "globs que suben a este carril",
+          "items": {
+            "description": "glob relativo: sin negacion (!), rutas absolutas ni ..",
+            "minLength": 1,
+            "type": "string"
+          },
+          "minItems": 1,
+          "type": "array"
+        }
+      },
+      "type": "object"
+    },
+    "seleccion": {
+      "additionalProperties": false,
+      "default": {
+        "dominios_extra": {},
+        "modelos_permitidos": [
+          "haiku",
+          "sonnet",
+          "opus"
+        ]
+      },
+      "description": "seleccion de skills, agentes y modelo [F1-15 decision 3 y S15]",
+      "properties": {
+        "dominios_extra": {
+          "additionalProperties": false,
+          "default": {},
+          "description": "solo anade senales a un dominio de la lista",
+          "properties": {
+            "puertas-seguridad": {
+              "additionalProperties": false,
+              "description": "senales que se anaden",
+              "properties": {
+                "etiquetas": {
+                  "default": [],
+                  "description": "etiquetas que se anaden",
+                  "items": {
+                    "description": "texto no vacio, sin caracteres de control ni espacios en los extremos",
+                    "minLength": 1,
+                    "type": "string"
+                  },
+                  "type": "array"
+                },
+                "palabras": {
+                  "default": [],
+                  "description": "palabras que se anaden",
+                  "items": {
+                    "description": "texto no vacio, sin caracteres de control ni espacios en los extremos",
+                    "minLength": 1,
+                    "type": "string"
+                  },
+                  "type": "array"
+                },
+                "rutas": {
+                  "default": [],
+                  "description": "globs que se anaden",
+                  "items": {
+                    "description": "glob relativo: sin negacion (!), rutas absolutas ni ..",
+                    "minLength": 1,
+                    "type": "string"
+                  },
+                  "type": "array"
+                }
+              },
+              "type": "object"
+            },
+            "rendimiento": {
+              "additionalProperties": false,
+              "description": "senales que se anaden",
+              "properties": {
+                "etiquetas": {
+                  "default": [],
+                  "description": "etiquetas que se anaden",
+                  "items": {
+                    "description": "texto no vacio, sin caracteres de control ni espacios en los extremos",
+                    "minLength": 1,
+                    "type": "string"
+                  },
+                  "type": "array"
+                },
+                "palabras": {
+                  "default": [],
+                  "description": "palabras que se anaden",
+                  "items": {
+                    "description": "texto no vacio, sin caracteres de control ni espacios en los extremos",
+                    "minLength": 1,
+                    "type": "string"
+                  },
+                  "type": "array"
+                },
+                "rutas": {
+                  "default": [],
+                  "description": "globs que se anaden",
+                  "items": {
+                    "description": "glob relativo: sin negacion (!), rutas absolutas ni ..",
+                    "minLength": 1,
+                    "type": "string"
+                  },
+                  "type": "array"
+                }
+              },
+              "type": "object"
+            }
+          },
+          "type": "object"
+        },
+        "modelos_permitidos": {
+          "default": [
+            "haiku",
+            "sonnet",
+            "opus"
+          ],
+          "description": "solo restringe la escala",
+          "items": {
+            "enum": [
+              "haiku",
+              "sonnet",
+              "opus"
+            ],
+            "type": "string"
+          },
+          "minItems": 1,
+          "type": "array"
+        }
+      },
+      "type": "object"
+    },
+    "tope_rondas": {
+      "default": 2,
+      "description": "rondas de revision sin preguntar [F1-11 PoC B; F1-16 D6a]",
+      "maximum": 10,
+      "minimum": 1,
+      "type": "integer"
+    },
+    "verificacion": {
+      "additionalProperties": false,
+      "description": "contrato de verificacion de F1-7 [F1-11 decision 1]",
+      "properties": {
+        "comando": {
+          "description": "comando del proyecto; se llama con --perfil <p>. orden no vacia y sin caracteres de control (rutas de Windows entre comillas simples o con /)",
+          "minLength": 1,
+          "type": "string"
+        },
+        "huella_arbol": {
+          "default": [],
+          "description": "ficheros versionados de la clave de cache (F1-7 dec. 2)",
+          "items": {
+            "description": "glob relativo: sin negacion (!), rutas absolutas ni ..",
+            "minLength": 1,
+            "type": "string"
+          },
+          "type": "array"
+        },
+        "huella_local": {
+          "description": "imprime lo que la suite lee fuera del arbol. orden no vacia y sin caracteres de control (rutas de Windows entre comillas simples o con /)",
+          "minLength": 1,
+          "type": "string"
+        },
+        "perfiles": {
+          "additionalProperties": false,
+          "description": "perfiles de F1-7; completa obligatoria",
+          "properties": {
+            "completa": {
+              "additionalProperties": false,
+              "description": "perfil de F1-7",
+              "properties": {
+                "runner": {
+                  "anyOf": [
+                    {
+                      "additionalProperties": false,
+                      "properties": {
+                        "github": {
+                          "description": "runner alojado de GitHub",
+                          "enum": [
+                            "windows",
+                            "linux"
+                          ],
+                          "type": "string"
+                        }
+                      },
+                      "required": [
+                        "github"
+                      ],
+                      "type": "object"
+                    },
+                    {
+                      "items": {
+                        "description": "texto no vacio, sin caracteres de control ni espacios en los extremos",
+                        "minLength": 1,
+                        "type": "string"
+                      },
+                      "minItems": 1,
+                      "prefixItems": [
+                        {
+                          "const": "self-hosted"
+                        }
+                      ],
+                      "type": "array"
+                    }
+                  ],
+                  "description": "donde corre en la CI"
+                },
+                "timeout": {
+                  "description": "segundos; en rapida es un presupuesto que no bloquea, en las demas agotarlo es rojo",
+                  "maximum": 21600,
+                  "minimum": 1,
+                  "type": "integer"
+                }
+              },
+              "required": [
+                "timeout",
+                "runner"
+              ],
+              "type": "object"
+            },
+            "impacto": {
+              "additionalProperties": false,
+              "description": "perfil de F1-7",
+              "properties": {
+                "timeout": {
+                  "description": "segundos; en rapida es un presupuesto que no bloquea, en las demas agotarlo es rojo",
+                  "maximum": 21600,
+                  "minimum": 1,
+                  "type": "integer"
+                }
+              },
+              "required": [
+                "timeout"
+              ],
+              "type": "object"
+            },
+            "pesada": {
+              "additionalProperties": false,
+              "description": "perfil de F1-7",
+              "properties": {
+                "runner": {
+                  "anyOf": [
+                    {
+                      "additionalProperties": false,
+                      "properties": {
+                        "github": {
+                          "description": "runner alojado de GitHub",
+                          "enum": [
+                            "windows",
+                            "linux"
+                          ],
+                          "type": "string"
+                        }
+                      },
+                      "required": [
+                        "github"
+                      ],
+                      "type": "object"
+                    },
+                    {
+                      "items": {
+                        "description": "texto no vacio, sin caracteres de control ni espacios en los extremos",
+                        "minLength": 1,
+                        "type": "string"
+                      },
+                      "minItems": 1,
+                      "prefixItems": [
+                        {
+                          "const": "self-hosted"
+                        }
+                      ],
+                      "type": "array"
+                    }
+                  ],
+                  "description": "donde corre en la CI"
+                },
+                "timeout": {
+                  "description": "segundos; en rapida es un presupuesto que no bloquea, en las demas agotarlo es rojo",
+                  "maximum": 21600,
+                  "minimum": 1,
+                  "type": "integer"
+                }
+              },
+              "required": [
+                "timeout",
+                "runner"
+              ],
+              "type": "object"
+            },
+            "rapida": {
+              "additionalProperties": false,
+              "description": "perfil de F1-7",
+              "properties": {
+                "timeout": {
+                  "description": "segundos; en rapida es un presupuesto que no bloquea, en las demas agotarlo es rojo",
+                  "maximum": 21600,
+                  "minimum": 1,
+                  "type": "integer"
+                }
+              },
+              "required": [
+                "timeout"
+              ],
+              "type": "object"
+            }
+          },
+          "required": [
+            "completa"
+          ],
+          "type": "object"
+        },
+        "programada": {
+          "additionalProperties": false,
+          "description": "job programado fuera del check cacheado",
+          "properties": {
+            "comando": {
+              "description": "pasos que dependen de la red. orden no vacia y sin caracteres de control (rutas de Windows entre comillas simples o con /)",
+              "minLength": 1,
+              "type": "string"
+            },
+            "cron": {
+              "description": "cuando corre. expresion cron de 5 campos",
+              "minLength": 1,
+              "type": "string"
+            },
+            "runner": {
+              "anyOf": [
+                {
+                  "additionalProperties": false,
+                  "properties": {
+                    "github": {
+                      "description": "runner alojado de GitHub",
+                      "enum": [
+                        "windows",
+                        "linux"
+                      ],
+                      "type": "string"
+                    }
+                  },
+                  "required": [
+                    "github"
+                  ],
+                  "type": "object"
+                },
+                {
+                  "items": {
+                    "description": "texto no vacio, sin caracteres de control ni espacios en los extremos",
+                    "minLength": 1,
+                    "type": "string"
+                  },
+                  "minItems": 1,
+                  "prefixItems": [
+                    {
+                      "const": "self-hosted"
+                    }
+                  ],
+                  "type": "array"
+                }
+              ],
+              "description": "donde corre"
+            }
+          },
+          "required": [
+            "comando",
+            "cron",
+            "runner"
+          ],
+          "type": "object"
+        },
+        "shell": {
+          "description": "shell con la que se lanza en todas partes (H16)",
+          "enum": [
+            "cmd",
+            "bash",
+            "pwsh",
+            "sh"
+          ],
+          "type": "string"
+        }
+      },
+      "required": [
+        "comando",
+        "shell",
+        "perfiles"
+      ],
+      "type": "object"
+    }
+  },
+  "required": [
+    "formato"
+  ],
+  "title": "Configuracion de TaskCode 2 (.taskcode/config.yml)",
+  "type": "object"
+}
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-023/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-023/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-023/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````
