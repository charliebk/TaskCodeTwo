# Peticion de revision — TASK-118 (ronda 2)

- Tarea: TASK-118 — Aplicar las puertas (rulesets) desde tc2 init
- Rama revisada: feature/task-118-aplicar-las-puertas-rulesets-desde-tc2-i
- Rama base: develop
- Commit revisado (HEAD): 29d24a7abacf092fb86a6d23d91ddc0b2cc75977
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-118 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 34a6b7b21259c24709fa626eb1939edb7a053ecc (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (4 hallazgo(s)))

## Instrucciones para el agente revisor (verificacion acotada)

Eres un revisor INDEPENDIENTE del agente que implemento. La rama ya paso una
revision completa: esta ronda verifica las correcciones de la ronda anterior,
no vuelve a revisar la rama entera ni los MENORES de la ronda anterior (tu
tabla solo trae CRITICOS e IMPORTANTES). Haz esto, y nada mas:

1. Comprueba, uno por uno, que cada hallazgo pendiente de tu tabla de abajo
   queda resuelto por el diff de esta ronda (los marcados `corregido` tambien:
   verificalo, no lo des por bueno). Para un CRITICO o IMPORTANTE, reproduce el
   fallo original y comprueba que ya no ocurre; puedes leer el repo entero si la
   causa no esta en el diff.
2. Ejecuta la suite (o el comando de verificacion del proyecto) sobre el commit
   revisado y di el resultado: verificar sin ejecutar no vale. Si esta peticion
   ya trae la verificacion de la ronda en verde para ese mismo commit, no la
   repitas: cita ese resultado y corre solo los tests de lo que verificas.
3. Busca en el diff de esta ronda el fallo que la correccion abre: es justo
   donde se cuelan los fallos nuevos. Si encuentras uno, construye el caso que
   lo rompe antes de reportarlo.

Solo abre un hallazgo NUEVO si es CRITICO o si lo introduce la propia correccion
(algo que el diff de esta ronda rompe). Lo demas que veas fuera de eso va como
nota al final del informe, no en la tabla: no abre otra ronda.

Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. En la tabla, cada hallazgo pendiente que ya este
resuelto va como `corregido`; el que no, `abierto`. Vuelca tu salida en el
informe de esta ronda (informe-revision-2-rust-wasm-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-118/tarea.md
- tareas/03-en-revision/TASK-118/planificacion/
- crates/tc2/src/init/mod.rs
- crates/tc2/src/plataforma/github.rs
- crates/tc2/src/plataforma/gitlab.rs
- crates/tc2/src/plataforma/mod.rs
- crates/tc2/tests/init.rs
- crates/tc2/tests/plataforma_gitlab.rs
- docs/adr/F1-13-adaptador-de-plataforma.md
- docs/adr/F1-5-puertas-en-github.md

## Hallazgos de la ronda 1 pendientes de verificar

Tus 1 hallazgo(s) MENOR de la ronda 1 no se reverifican (los comprueba
quien coordina): recibes esta ronda porque la correccion
toca ficheros de tu dominio. Revisa en el diff de esta ronda lo que la correccion
cambia en tu terreno y si abre un fallo.

## Commits a revisar (git log 34a6b7b21259c24709fa626eb1939edb7a053ecc..HEAD)

````
29d24a7 chore(TASK-118): autorrevision ronda 2
ad859f7 fix(TASK-118): la segunda ronda de GitLab en init parte de los ajustes ya puestos
b9f6621 fix(TASK-118): GitLab completa una puerta a medias y precisa las ADR
1cdd6cd chore(TASK-118): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 34a6b7b21259c24709fa626eb1939edb7a053ecc..HEAD)

````diff
diff --git a/crates/tc2/src/init/mod.rs b/crates/tc2/src/init/mod.rs
index 1177e40..86d759f 100644
--- a/crates/tc2/src/init/mod.rs
+++ b/crates/tc2/src/init/mod.rs
@@ -484,9 +484,10 @@ pub fn init(op: Opciones, c: &Contexto<'_>) -> Hecho {
         if let Some(i) = &puertas {
             // Cada linea `ruleset` dice lo que de verdad paso: creada,
             // existente (otro llego antes) o sin aplicar.
-            let (aplicadas, existentes, faltan) = (
+            let (aplicadas, existentes, a_medias, faltan) = (
                 lista(i, "aplicadas"),
                 lista(i, "existentes"),
+                lista(i, "a-medias"),
                 lista(i, "faltan"),
             );
             for a in acciones.iter_mut().filter(|a| a.clase == "ruleset") {
@@ -497,14 +498,26 @@ pub fn init(op: Opciones, c: &Contexto<'_>) -> Hecho {
                     a.decision = Decision::Existe;
                     a.motivo = None;
                 } else if faltan.contains(&a.ruta) {
+                    // La que fallo dice si quedo escrita en parte; las de
+                    // detras no se intentaron.
                     a.decision = Decision::Pendiente;
-                    if a.motivo.as_deref() != Some("error") {
-                        a.motivo = Some("no-aplicada".to_owned());
-                    }
+                    a.motivo = Some(
+                        if a_medias.contains(&a.ruta) {
+                            "a-medias"
+                        } else if faltan.first() == Some(&a.ruta) {
+                            "error"
+                        } else {
+                            "no-aplicada"
+                        }
+                        .to_owned(),
+                    );
                 }
             }
             if !i.resultado.es_exito() {
                 cabeza.push(("aplicadas", Valor::Lista(aplicadas)));
+                if !a_medias.is_empty() {
+                    cabeza.push(("a-medias", Valor::Lista(a_medias)));
+                }
                 cabeza.push(("faltan", Valor::Lista(faltan.clone())));
                 if let (Some(f), Some(primera)) = (fallo.as_mut(), faltan.first()) {
                     f.0.clone_from(primera);
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
index cbc8c16..3710747 100644
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -796,7 +796,7 @@ impl Plataforma for GitHub {
             Ok(l) => l,
             Err(i) => return i,
         };
-        aplicar_en_lote(ps, &leidas, |p| {
+        aplicar_en_lote(ps, &leidas, |p, _escrito| {
             let cuerpo = puertas::ruleset_github(p, Some(origenes));
             if cuerpo.get("_nota").is_some() {
                 // Nunca se envia un check sin su `integration_id`.
diff --git a/crates/tc2/src/plataforma/gitlab.rs b/crates/tc2/src/plataforma/gitlab.rs
index ba5c656..f55836f 100644
--- a/crates/tc2/src/plataforma/gitlab.rs
+++ b/crates/tc2/src/plataforma/gitlab.rs
@@ -1010,7 +1010,7 @@ impl Plataforma for GitLab {
                 Err(self.con.traducir(op, rol, PERMISO_MERGE, &r))
             }
         };
-        aplicar_en_lote(ps, &leidas, |p| {
+        aplicar_en_lote(ps, &leidas, |p, escrito| {
             let mut nueva = false;
             match &p.objetivo {
                 Objetivo::Ramas(r) => {
@@ -1028,6 +1028,7 @@ impl Plataforma for GitLab {
                                 "allow_force_push": false,
                             }),
                         )?;
+                        *escrito |= nueva;
                     }
                 }
                 Objetivo::Tags(t) => {
@@ -1042,6 +1043,7 @@ impl Plataforma for GitLab {
                                 "create_access_level": DEVELOPER,
                             }),
                         )?;
+                        *escrito |= nueva;
                     }
                 }
             }
@@ -1064,6 +1066,7 @@ impl Plataforma for GitLab {
                     }
                     ajustes = true;
                     nueva = true;
+                    *escrito = true;
                 }
                 if pa.premium && !aprobacion {
                     nueva |= crear(
@@ -1074,6 +1077,7 @@ impl Plataforma for GitLab {
                             "rule_type": "any_approver",
                         }),
                     )?;
+                    *escrito |= nueva;
                     aprobacion = true;
                 }
             }
@@ -1184,11 +1188,17 @@ impl GitLab {
                             Some(_) => {}
                         }
                     }
+                    // Proteccion como la del modelo pero sin los ajustes de
+                    // merge del proyecto o sin la regla de aprobacion
+                    // (Premium): falta completarla, p. ej. tras un corte
+                    // entre proteger la rama y el `PUT`. Esas escrituras no
+                    // tocan la proteccion; `Difiere` queda para una
+                    // proteccion distinta del modelo, que no se toca.
                     if e == EstadoPuerta::Existe
                         && p.solo_pr.is_some()
                         && (!ajustes_bien || (premium && !con_aprobacion))
                     {
-                        e = EstadoPuerta::Difiere;
+                        e = EstadoPuerta::Falta;
                     }
                     e
                 }
diff --git a/crates/tc2/src/plataforma/mod.rs b/crates/tc2/src/plataforma/mod.rs
index 1550868..d9029e2 100644
--- a/crates/tc2/src/plataforma/mod.rs
+++ b/crates/tc2/src/plataforma/mod.rs
@@ -323,13 +323,17 @@ pub(crate) enum Creada {
     YaExistia,
 }
 
-/// Escribe las puertas que faltan de `leidas`, una a una, con `crear`. Un
-/// fallo corta: la linea dice `aplicadas=` y `faltan=`, y repetirla
-/// converge (lo creado sale como existente). Nunca reescribe ni borra.
+/// Escribe las puertas que faltan de `leidas`, una a una, con `crear`, que
+/// marca en su `bool` si ya escribio algo de esa puerta (en GitLab una
+/// puerta son varias escrituras). Un fallo corta: la linea dice
+/// `aplicadas=` (las completas), `a-medias=` (la que quedo escrita en
+/// parte, si la hay) y `faltan=` (las que no estan completas, con esa), y
+/// repetirla converge: lo creado sale como existente y lo que le falta a
+/// una puerta a medias se completa. Nunca reescribe ni borra.
 pub(crate) fn aplicar_en_lote(
     puertas: &[puertas::Puerta],
     leidas: &PuertasLeidas,
-    mut crear: impl FnMut(&puertas::Puerta) -> Paso<Creada>,
+    mut crear: impl FnMut(&puertas::Puerta, &mut bool) -> Paso<Creada>,
 ) -> Informe {
     let mut aplicadas = Vec::new();
     let mut existentes = leidas.con_estado(EstadoPuerta::Existe);
@@ -344,15 +348,18 @@ pub(crate) fn aplicar_en_lote(
         })
         .collect();
     for (k, p) in faltan.iter().enumerate() {
-        match crear(p) {
+        let mut escrito = false;
+        match crear(p, &mut escrito) {
             Ok(Creada::Nueva) => aplicadas.push(p.nombre_completo()),
             Ok(Creada::YaExistia) => existentes.push(p.nombre_completo()),
             Err(mut i) => {
                 puertas::APLICAR_PUERTAS.clone_into(&mut i.comando);
                 let resto = faltan[k..].iter().map(|p| p.nombre_completo()).collect();
-                return i
-                    .con_lista("aplicadas", aplicadas)
-                    .con_lista("faltan", resto);
+                i = i.con_lista("aplicadas", aplicadas);
+                if escrito {
+                    i = i.con_lista("a-medias", vec![p.nombre_completo()]);
+                }
+                return i.con_lista("faltan", resto);
             }
         }
     }
diff --git a/docs/adr/F1-13-adaptador-de-plataforma.md b/docs/adr/F1-13-adaptador-de-plataforma.md
index 7a4ae1c..2f693d6 100644
--- a/docs/adr/F1-13-adaptador-de-plataforma.md
+++ b/docs/adr/F1-13-adaptador-de-plataforma.md
@@ -375,7 +375,7 @@ no se ha configurado nada en ninguna plataforma):
    | `abrir-pr` | `GET /pulls?head=` y, si no hay, `POST /pulls` | `GET /merge_requests?source_branch=` y, si no hay, `POST /merge_requests` |
    | `estado-puertas` | `GET /pulls/{n}` (con `mergeable_state`), `GET /rules/branches/{base}`, `GET /commits/{sha}/check-runs`, `GET /pulls/{n}/reviews` | `GET /merge_requests/{iid}?include_diverged_commits_count=true`, `GET /approval_rules`, `GET /protected_branches/{destino}`, `GET /repository/commits/{sha}/statuses`, `GET /merge_requests/{iid}/approvals` |
    | `merge-pr` | `PUT /pulls/{n}/merge` con `merge_method=merge`; con cola, la cola | `PUT /merge_requests/{iid}/merge` (el método lo fija el proyecto); Premium con tren: el tren |
-   | `aplicar-puertas` (adenda de TASK-118) | `GET /rulesets`, `GET /commits/heads/develop/check-runs` y, de las que faltan, `POST /rulesets`; nunca `PUT` ni `DELETE` | `GET /protected_branches`, `GET /protected_tags`, `GET /repository/branches/develop` y sus estados, `GET /projects/:id` y el namespace; de lo que falta, `POST /protected_branches`, `POST /protected_tags`, `PUT /projects/:id` (ajustes de merge) y en Premium `POST /approval_rules`; nunca `DELETE` ni `PATCH` |
+   | `aplicar-puertas` (adenda de TASK-118) | `GET /user` (persona de la lista), `GET /rulesets`, `GET /commits/heads/develop/check-runs` y, de las que faltan, `POST /rulesets`; nunca `PUT` ni `DELETE` | `GET /user` (persona de la lista), `GET /protected_branches`, `GET /protected_tags`, `GET /repository/branches/develop` y sus estados, `GET /projects/:id`, el namespace y en Premium `GET /approval_rules`; de lo que falta, `POST /protected_branches`, `POST /protected_tags`, `PUT /projects/:id` (ajustes de merge) y en Premium `POST /approval_rules`; nunca `DELETE` ni `PATCH` |
    | `leer-puertas` | `GET /rulesets`, `GET /rules/branches/{rama}` | `GET /protected_branches`, `GET /protected_tags`, `GET /projects/:id` con sesión, `GET /approval_rules`, `GET /approvals` |
    | `leer-permisos-ci` | `GET /actions/permissions/workflow` | ninguna: el job token es fijo (G16); informa `fuente=plataforma` |
    | firma del `approve` (`carril`) | `GET /commits/{sha}` | `GET /repository/commits/{sha}/signature` |
diff --git a/docs/adr/F1-5-puertas-en-github.md b/docs/adr/F1-5-puertas-en-github.md
index 88221ca..e5c1a3d 100644
--- a/docs/adr/F1-5-puertas-en-github.md
+++ b/docs/adr/F1-5-puertas-en-github.md
@@ -625,9 +625,15 @@ lecturas) y `aplicar_puertas` (en lote), con implementación por defecto
 `aplicar-puertas rechazada motivo=no-soportada`. `aplicar_puertas` primero
 repite todas las comprobaciones de `preparar_puertas` y solo después
 escribe; crea las puertas que faltan, una a una, y si un fallo corta a mitad
-la línea dice `aplicadas=` y `faltan=`. Repetirla converge: lo ya creado
-sale como existente. Un 422 de GitHub o un 409 de GitLab al crear es «otro
-llegó antes» y cuenta como existente. GitHub: `POST /repos/{r}/rulesets`
+la línea dice `aplicadas=` (las completas), `a-medias=` (la que quedó
+escrita en parte, p. ej. en GitLab una rama ya protegida sin los ajustes de
+merge) y `faltan=`. Repetirla converge: lo ya creado sale como existente y
+a una puerta de GitLab cuya protección ya es la del modelo se le completan
+los ajustes de merge y la regla de aprobación (escrituras que no tocan la
+protección). Un 422 de GitHub al crear solo cuenta como «otro llegó antes»
+(existente) si la relectura de `GET /rulesets` ya trae ese nombre; si no, es
+un error. Un 409 de GitLab al proteger una rama o un tag cuenta como
+existente. GitHub: `POST /repos/{r}/rulesets`
 con el JSON de la PoC C, `integration_id` por check y sin `_nota`; el 403 de
 plan da `rechazada motivo=plan`. GitLab: `POST /protected_branches` y
 `POST /protected_tags` con niveles explícitos (nunca el valor por defecto
````

## Excluido del diff (11 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 34a6b7b21259c24709fa626eb1939edb7a053ecc..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-118/planificacion/brainstorm/peticion-brainstorm-arquitectura-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-118/planificacion/brainstorm/peticion-brainstorm-riesgos-1.md      |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-118/planificacion/brainstorm/peticion-unificador-1.md              |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-118/planificacion/brainstorm/salida-brainstorm-arquitectura-1.md   |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-118/planificacion/brainstorm/salida-brainstorm-riesgos-1.md        |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-118/planificacion/plan-final.md                                    |    0
 tareas/03-en-revision/TASK-118/revision/informe-revision-1-code-quality-reviewer.md                            |   21 +
 tareas/03-en-revision/TASK-118/revision/informe-revision-1-rust-wasm-reviewer.md                               |   21 +
 tareas/03-en-revision/TASK-118/revision/peticion-revision-1-code-quality-reviewer.md                           |  167 ++++++++
 tareas/03-en-revision/TASK-118/revision/peticion-revision-1-rust-wasm-reviewer.md                              | 1737 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-118/tarea.md                                                       |   15 +-
 11 files changed, 1957 insertions(+), 4 deletions(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 34a6b7b21259c24709fa626eb1939edb7a053ecc..HEAD -- ":(literal)crates/tc2/tests/init.rs" ":(literal)crates/tc2/tests/plataforma_gitlab.rs"
````

````
crates/tc2/tests/init.rs              |  29 +++++++++++++++++++----------
 crates/tc2/tests/plataforma_gitlab.rs | 126 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--
 2 files changed, 143 insertions(+), 12 deletions(-)
````
