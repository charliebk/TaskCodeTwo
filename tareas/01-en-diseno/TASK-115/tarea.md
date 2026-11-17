---
id: TASK-115
titulo: "Seguimiento de F2-12: menores de actualizar --cli y mock HTTP estable"
tipo: fix
sprint: 0
etiquetas: [distribucion, tests]
complejidad: simple
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: true
rama: fix/task-115-seguimiento-de-f2-12-menores-de-actualiz
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-16
actualizado: 2026-11-17
dependencias: []
---
## Objetivo

Cerrar los MENORES aceptados en TASK-027 y estabilizar tests/comun/http_mock.rs, que da sinred intermitente en esta maquina.

## Criterios de aceptacion
- [ ] ejecutar_con_limite devuelve la salida ya leida cuando un nieto retiene stdout y su doc dice lo que hace (test con nieto, MEN-7 r3)
- [ ] Un fallo de TLS no se reintenta y el timeout global no dobla la espera (tests en crates/tc2/src/descarga.rs, MEN-8 y MEN-9 r3)
- [ ] Tests de integracion de redireccion no permitida y de limite de tamano en crates/tc2/tests/actualizar_cli.rs (MEN-5 r1)
- [ ] tests/comun/http_mock.rs no manda FIN tras una respuesta completa (test determinista en tests/comun: tras leer el Content-Length la lectura siguiente agota el plazo y no da Ok(0)) y plataforma y plataforma_gitlab pasan 10 de 10 seguidas (evidencia del script en ## Resultado)
- [ ] El doc de bajar_con_un_reintento esta sobre su funcion y sin lineas duplicadas (test unitario con include_str! en crates/tc2/src/actualizar_cli.rs)

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T03:47:05Z | plan | automatico | persona | 0.16.1 |
| 2026-11-17T03:48:28Z | approve | automatico | persona | 0.16.1 |
