---
id: TASK-145
titulo: "Auditoria 4D: A7 config, init, desinstalar, i18n, confianza y ciclo (8 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: fix/task-145-auditoria-4d-a7-config-init-desinstalar
asignado_a: null
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-23
dependencias: []
---
## Objetivo

Corregir los 8 casos IMPORTANTE del grupo A7 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 4 Duplicaciones (una `divergente`), 1 Desviación de F2-8 que cambia la salida y 3 Disfraces (uno en código y dos de test con mutante confirmado). A7-02 y A7-06 van en las tareas de abstracción común (A8-04 y A8-12). No se pide el MENOR (A7-08). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [ ] A7-04 (`divergente`, va primero): los ayudantes de test `md`, `estado`, `motivo`, `texto` y `fila` de `ciclo/liberar.rs:423-467` y `ciclo/operacion.rs:286-325` quedan en un solo módulo de prueba de `ciclo`, con los parámetros que hoy separan `md` y `fila`.
- [ ] A7-01 y A7-03: el esqueleto «plan como dato» de `init/mod.rs` y `desinstalar/mod.rs` (rangos de la ficha A7-01) se comparte, y `estado` y `confiar` de `confianza/mod.rs:198-242` ~ `:246-275` comparten la lectura, el cierre, la huella y la `Evaluacion`.
- [ ] A7-05: los dos tests unitarios de `ciclo/mod.rs:505-527` ~ `:570-590` montan el escenario git con un solo ayudante.
- [ ] A7-07 y A7-10: ningún texto se renderiza en español en el sitio para meterlo en `{error}` (`init/aplicar.rs:25-27`, `:82-84`, `init/mod.rs:470`, `desinstalar/proyecto.rs:373-376`, `desinstalar/mod.rs:542`). `ningun_literal_fuera_del_catalogo` (`i18n/guarda.rs:17-23`, `:53-83`) caza la forma `Err("...".to_owned())`. Con el mutante de la ficha A7-10 aplicado, `cargo test -p tc2 --lib i18n::guarda::ningun_literal_fuera_del_catalogo` sale en rojo, y con `TC2_IDIOMA=en` el diagnóstico sale entero en inglés.
- [ ] A7-09: `config::del_arbol` (`config/mod.rs:534-543`) da `sin-config` solo con `ErrorKind::NotFound`. Un `.taskcode/config.yml` en UTF-16 o sin permiso de lectura da otro motivo con la causa, y no «crea la configuración con tc2 init». Lo prueba un test con un fichero UTF-16LE con BOM.
- [ ] A7-11: un test ejercita el rechazo `ramas-sin-remoto` con `develop` sin publicar (`init/inventario.rs:347-352`, criterio de TASK-118), y el docstring de `aplicar_puertas_exige_personas_ramas_y_persona` (`tests/init.rs:1693-1754`) dice lo que provoca. Con el mutante de la ficha aplicado, `cargo test -p tc2 --test init` sale en rojo.
- [ ] `cargo xtask verificar` en verde, y el `## Resultado` lista los 8 IDs con su commit.
