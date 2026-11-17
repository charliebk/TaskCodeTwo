# Plan — TASK-031: F3-4 tc2 roadmap: la vista para todos

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 300 min

El ROADMAP se genera con el mismo calculo que `next`: `siguiente::calcular` sale a una pieza comun `situacion` y un generador puro da forma a lo que ella calcula. Se regenera dentro de cada transaccion de estado con el gancho existente `Contexto.roadmap: Option<Roadmap>` (`estado/transaccion.rs:72-74` y `:348-355`), hoy a `None`; nunca se mergea y el push sin force hace de compare-and-swap (ADR F1-2 D2.4). `tc2 roadmap` sin opciones solo lee; `--comparar` dice si lo publicado esta al dia; `--escribir` lo publica como una transaccion mas.

Piezas:
- `situacion.rs` (nueva, sacada de `siguiente::calcular`): `Situacion {docs, nodos, ilegibles, hitos, orden, listas, en_marcha}` desde `&dyn Estado`; `next` y `roadmap` la consumen y coinciden por construccion.
- `roadmap/zona.rs` (nueva, hoja pura): extraccion estricta de la zona manual (error con inicio sin fin o fin suelto), conservada byte a byte; `grafo::Hitos::del_roadmap` pasa a usarla.
- `roadmap/generar.rs` y `roadmap/mermaid.rs` (nuevas, puras): `generar(s, zona) -> String`, LF y sin fecha (mismos datos, mismos bytes → `sin-cambios`).
- `roadmap/mod.rs` (nueva): `de_estado(e) -> Result<String, Fallo>` con la forma del tipo `Roadmap`, y la orden `roadmap [--comparar|--escribir] [--json]`; `--escribir` es una `Operacion` `Regenerar` con `exige_red() = true`.
- `estado::Contexto::nuevo` recibe el generador como argumento explicito; lo pasan alta (`lib.rs`) e `importar`; `Inicializar` (montaje) sigue sin el. Dependencias `roadmap → situacion → grafo/estado`, sin ciclos.
- «Con quien» como `@login` (`Asignado::login()`); un `Heredado` se convierte con `identidad::convertir_heredado` cuando se pueda y si no se muestra tal cual.

Formato del ROADMAP.md 2.0:
- `# Roadmap de tareas`, marca `<!-- generado por tc2; formato 2; no editar fuera de la zona manual -->`, zona manual byte a byte (o la de la 0.16 por defecto).
- `## Estado del trabajo` con `- **En marcha:**` y `- **Listas para empezar:**` (las mismas cadenas que `next`); `- **Atascadas (ciclo):**` y avisos solo si los hay.
- `## En curso` (en-diseno y en-curso) y `## En revision`: tabla `Tarea | Titulo | Con quien | Hito | Depende de`.
- `## Hn — titulo` por hito no cerrado, en el orden de la zona manual: tabla `Paso | Tarea | Titulo | Depende de | Estado | Con quien` y bloque Mermaid `flowchart TD` (nodos `T031["TASK-031 · titulo"]` con titulo escapado, aristas `T018 --> T031`, dependencias de otro hito como nodo de borde, `classDef` por estado).
- `## Terminadas` y `## Descartadas`: recuento y lista plegada en `<details>` (ADR F1-1 decision 3). El grafo completo no se commitea.
- Coste: un `cat-file --batch` y un Kahn O((n+e) log n) por intento.

Orden de construccion:
1. `situacion.rs` y `roadmap/zona.rs`; `next` las usa sin cambiar su salida (tests de `siguiente` y de ToolsaWeb en verde).
2. `generar` y `mermaid` puros con salida de referencia en fixture.
3. `de_estado` y `Contexto::nuevo` con el generador en alta e import.
4. Orden `tc2 roadmap` con `--comparar`/`--escribir`, despacho e i18n.
5. Tests de integracion de C1..C6.

Alternativas descartadas: que solo `--escribir` regenere (se queda desfasado y compite con las demas escrituras; contra F1-2 D2.4); parchear solo filas (arrastra estado previo; F1-1 dice «se regenera, nunca se mergea»).

## Lo que el rol no cubrio

- Riesgos (solo zona rota y Mermaid grande), estrategia de pruebas y reglas de negocio (que es «en curso», como se muestran las identidades) sin especialista.
- Suposiciones sin verificar: limites de Mermaid en GitHub/GitLab (`maxEdges` 500, `maxTextSize` 50 000); que el camino sin red no regenere no importa porque la re-ejecucion con red lo hace; que `Documento` da titulo y `asignado_a` de los dos formatos; que la rama de estado migrada de la 0.16 trae la zona manual (F1-12, F9).

## Riesgos aceptados y que los contiene

- Zona manual rota dentro de una transaccion: no se bloquea la operacion; se deja el ROADMAP anterior tal cual y `--comparar`/`next` avisan con `roadmap-roto` (D3).
- Mermaid grande: acotado a cada hito abierto (C6); revision manual en GitHub anotada en `## Resultado` (D7).

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):
- C1: concurrencia de dos `new` en `tests/alta.rs` y `tests/estado_concurrencia.rs`.
- C2: `roadmap/zona.rs` y `tests/roadmap.rs` (LF y CRLF).
- C3: huella de refs, `status` y arbol antes y despues.
- C4: ToolsaWeb y repo de prueba contra `next`.
- C5: fixture de referencia y doble `--escribir`.
- C6: tamano con la instantanea de ToolsaWeb.

## Lo que necesita decision de una persona

Carlos acepta las recomendaciones (consigna del 2026-11-16, sin trocear):
- D1: se regenera dentro de cada transaccion de estado (ADR F1-2).
- D2: sin opciones imprime; `--comparar` y `--escribir`; sin `--simular`.
- D3: zona manual rota no bloquea; se deja el anterior y se avisa `roadmap-roto`.
- D4: la zona manual se edita en `tareas/ROADMAP.md` y se publica con `tc2 roadmap --escribir`; nada de commits a mano en la rama de estado.
- D5: `--grafo` con el grafo completo (RF-04) va a una tarea nueva.
- D6: `asignado_a` heredado se convierte con `identidad::convertir_heredado` cuando se pueda.
- D7: comprobacion manual del Mermaid en GitHub en un repo de prueba, anotada en `## Resultado`.
