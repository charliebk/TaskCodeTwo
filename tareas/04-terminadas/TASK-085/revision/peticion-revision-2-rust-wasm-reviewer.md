# Peticion de revision — TASK-085 (ronda 2)

- Tarea: TASK-085 — F4-6 plan propone skills, agentes y modelo por rol
- Rama revisada: feature/task-085-f4-6-plan-propone-skills-agentes-y-model
- Rama base: develop
- Commit revisado (HEAD): 1fc787b2571f855dd609564c17b18d55c7713123
- Fecha: 2026-11-22
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-085 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 0689c2cec9429cd0695cd68458dcdad880695989 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (9 hallazgo(s)))

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

- tareas/03-en-revision/TASK-085/tarea.md
- tareas/03-en-revision/TASK-085/planificacion/
- crates/tc2/src/plan/operacion.rs
- crates/tc2/src/revision/mod.rs
- crates/tc2/src/revision/operacion.rs
- crates/tc2/src/seleccion/catalogo.rs
- crates/tc2/src/seleccion/mod.rs
- crates/tc2/src/seleccion/orden.rs
- crates/tc2/src/tarea/mod.rs
- crates/tc2/src/tarea/modelo.rs
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/estratos.txt
- crates/tc2/tests/fixtures/seleccion-taskcodetwo/tabla.md
- crates/tc2/tests/seleccion.rs
- crates/tc2/tests/tarea.rs
- docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 2 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/seleccion/catalogo.rs:89 | informe-revision-1-rust-wasm-reviewer.md |
| IMP-2 | IMPORTANTE | corregido | crates/tc2/src/seleccion/mod.rs:270 | informe-revision-1-rust-wasm-reviewer.md |
| IMP-3 | IMPORTANTE | corregido | crates/tc2/src/tarea/modelo.rs:301 | informe-revision-1-rust-wasm-reviewer.md |

## Commits a revisar (git log 0689c2cec9429cd0695cd68458dcdad880695989..HEAD)

````
1fc787b chore(TASK-085): estado de los hallazgos de la ronda 1
2e2d33d fix(TASK-085): tabla dorada de las 10 tareas, regla de muestra escrita y aviso de Implementacion (MEN-1, MEN-2, MEN-4)
73818be fix(TASK-085): dominios en un solo sitio, error de tabla visible y heuristica una vez (MEN-1, MEN-2, MEN-3)
3e949ae fix(TASK-085): un solo tipo Rol para los tres roles (IMP-3)
1443600 fix(TASK-085): test de S06 y del orden de S07 con empate por id (IMP-2)
9d43951 fix(TASK-085): huella del catalogo sin la ruta de lectura (IMP-1)
b9fc601 chore(TASK-085): informes de la ronda 1 con cambios solicitados
e2713b5 chore(TASK-085): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 0689c2cec9429cd0695cd68458dcdad880695989..HEAD)

````diff
diff --git a/crates/tc2/src/plan/operacion.rs b/crates/tc2/src/plan/operacion.rs
index 471b3c5..8f65a4d 100644
--- a/crates/tc2/src/plan/operacion.rs
+++ b/crates/tc2/src/plan/operacion.rs
@@ -52,7 +52,7 @@ use crate::i18n::catalogo::plan as txt;
 use crate::i18n::{Mensaje, Texto};
 use crate::identidad::Quien;
 use crate::inventario::Inventario;
-use crate::seleccion::{self, Catalogo, dominios};
+use crate::seleccion::{self, Catalogo};
 use crate::tarea::{Asignado, Cuerpo, Transicion};
 use crate::toma::instante;
 
@@ -157,7 +157,8 @@ impl Planificar {
         self
     }
 
-    /// La seleccion de diseno e implementacion de la tarea (F4-6).
+    /// La seleccion de diseno e implementacion de la tarea (F4-6). `Err` si
+    /// las tablas del ADR no se leen.
     fn seleccion(
         &self,
         t: &crate::tarea::Tarea,
@@ -165,14 +166,8 @@ impl Planificar {
         carril: Carril,
         resolucion: &heuristica::Resolucion,
         n: u8,
-    ) -> (seleccion::Seleccion, seleccion::Seleccion) {
-        let tabla = seleccion::tabla::dominios(seleccion::tabla::ADR).unwrap_or_default();
-        let texto = dominios::Texto {
-            objetivo: &enunciado.objetivo,
-            criterios: &enunciado.criterios,
-            etiquetas: &t.etiquetas,
-        };
-        let doms = dominios::dominios(&texto, &[], &tabla, &self.config.seleccion.dominios_extra);
+    ) -> Result<(seleccion::Seleccion, seleccion::Seleccion), String> {
+        let doms = seleccion::orden::dominios_vivos(Some((t, enunciado)), &[], &self.config)?;
         let entrada = seleccion::Entrada {
             rutas: &[],
             dominios: &doms,
@@ -185,11 +180,11 @@ impl Planificar {
             modelos_permitidos: &self.config.seleccion.modelos_permitidos,
             ronda: 1,
         };
-        let s = |rol| {
-            seleccion::seleccionar(rol, &entrada, &self.catalogo, &self.inventario, &ctx)
-                .unwrap_or_default()
-        };
-        (s(seleccion::Rol::Diseno), s(seleccion::Rol::Implementacion))
+        let s = |rol| seleccion::seleccionar(rol, &entrada, &self.catalogo, &self.inventario, &ctx);
+        Ok((
+            s(seleccion::Rol::Diseno)?,
+            s(seleccion::Rol::Implementacion)?,
+        ))
     }
 
     /// `roles.<rol>` con la seleccion `s` (la `regla` de la 0.16 se
@@ -312,7 +307,7 @@ fn peticiones(d: &Datos<'_>, roles: &[Rol], forma: Forma) -> Vec<(String, String
 }
 
 /// Los dominios de unas senales `<dominio>:<senal>:<valor>`, sin repetir.
-pub fn dominios_de(senales: &[String]) -> Vec<String> {
+pub fn dominios_de_senales(senales: &[String]) -> Vec<String> {
     let mut v: Vec<String> = Vec::new();
     for s in senales {
         let d = s.split(':').next().unwrap_or_default().to_owned();
@@ -327,12 +322,8 @@ pub fn dominios_de(senales: &[String]) -> Vec<String> {
 /// seleccion (por el sufijo del id: `taskcode-plugin:brainstorm-riesgos`
 /// es `brainstorm-riesgos`), sin el unificador, que decide la forma. Con
 /// `n` por encima de la escalera (`politica_carriles`, hasta 5) se suman los
-/// condicionales que falten, como [`roles::seleccionar`]. Si la tabla no se
-/// pudo leer (sin filas aplicadas), los de antes de F4-6.
-fn roles_de(s: &seleccion::Seleccion, n: u8, etiquetas: &[String]) -> Vec<Rol> {
-    if s.reglas.is_empty() {
-        return roles::seleccionar(n, etiquetas);
-    }
+/// condicionales que falten, como [`roles::seleccionar`].
+fn roles_de(s: &seleccion::Seleccion, n: u8) -> Vec<Rol> {
     let mut v: Vec<Rol> = s
         .roles
         .iter()
@@ -427,8 +418,15 @@ impl Operacion for Planificar {
             &enunciado,
         );
         let n = carril.roles_diseno(resolucion.roles, &t.tipo, &self.config);
-        let (sd, si) = self.seleccion(&t, &enunciado, carril, &resolucion, n);
-        let roles = roles_de(&sd, n, &t.etiquetas);
+        let (sd, si) = self
+            .seleccion(&t, &enunciado, carril, &resolucion, n)
+            .map_err(|x| {
+                Fallo::Error(
+                    Mensaje::nuevo(&crate::i18n::catalogo::seleccion::TABLA_ILEGIBLE)
+                        .con("error", x),
+                )
+            })?;
+        let roles = roles_de(&sd, n);
         let forma = if roles.len() == 1 {
             Forma::Redaccion
         } else {
@@ -482,7 +480,7 @@ impl Operacion for Planificar {
             roles: roles.iter().map(|r| r.titulo).collect(),
             forma,
             peticiones: nombres(ronda),
-            dominios: dominios_de(&sd.senales),
+            dominios: dominios_de_senales(&sd.senales),
             modelos: (sd.modelo.clone(), si.modelo.clone()),
             avisos: sd.avisos.iter().chain(&si.avisos).cloned().collect(),
         });
diff --git a/crates/tc2/src/revision/mod.rs b/crates/tc2/src/revision/mod.rs
index 8b2a486..9951003 100644
--- a/crates/tc2/src/revision/mod.rs
+++ b/crates/tc2/src/revision/mod.rs
@@ -489,19 +489,19 @@ pub fn seleccion_de(
         });
     let f = seleccion::orden::fuentes(g, principal, claude, &[]);
     avisos.extend(f.avisos.iter().cloned());
-    let (carril, cx) = seleccion::orden::de_tarea(t, cuerpo);
+    let (carril, r) = seleccion::orden::de_tarea(t, cuerpo);
     let cfg = config::de_base(g, principal).unwrap_or_else(|_| {
         avisos.push("config-de-base-no-leida".to_owned());
         config::Config::suelo()
     });
     let ctx = seleccion::Contexto {
         carril,
-        complejidad: Some(cx),
+        complejidad: Some(seleccion::orden::complejidad(&r)),
         modelos_permitidos: &cfg.seleccion.modelos_permitidos,
         ronda: 1,
     };
     // F4-6: con los dominios (texto vivo, rutas del diff y lo congelado).
-    let doms = seleccion::orden::dominios_de(Some((t, cuerpo)), &rutas, &cfg);
+    let doms = seleccion::orden::dominios_de(Some((t, cuerpo)), &rutas, &cfg).unwrap_or_default();
     let entrada = seleccion::Entrada {
         rutas: &rutas,
         dominios: &doms,
diff --git a/crates/tc2/src/revision/operacion.rs b/crates/tc2/src/revision/operacion.rs
index 45f52e3..b3d92a0 100644
--- a/crates/tc2/src/revision/operacion.rs
+++ b/crates/tc2/src/revision/operacion.rs
@@ -89,7 +89,7 @@ impl Revisar {
     }
 
     /// `roles.revision` de `t` con la seleccion; `None` si no hay seleccion.
-    fn anotar(&self, t: &crate::tarea::Tarea) -> Option<crate::tarea::RolRevision> {
+    fn anotar(&self, t: &crate::tarea::Tarea) -> Option<crate::tarea::Rol> {
         let s = self.seleccion.as_ref()?;
         let mut rv = t.roles.revision.clone();
         rv.agente.clone_from(&s.agente);
diff --git a/crates/tc2/src/seleccion/catalogo.rs b/crates/tc2/src/seleccion/catalogo.rs
index 53608c5..c67fb59 100644
--- a/crates/tc2/src/seleccion/catalogo.rs
+++ b/crates/tc2/src/seleccion/catalogo.rs
@@ -85,9 +85,27 @@ impl Catalogo {
     }
 
     /// Huella del catalogo (12 hex del sha256 de su forma canonica, como
-    /// [`Inventario::huella`]).
+    /// [`Inventario::huella`]): lo que cada entrada declara, sin la `ruta`
+    /// de la que se leyo (la misma skill en otra maquina da la misma huella).
     pub fn huella(&self) -> String {
-        let s = format!("{:?}{:?}", self.revisoras, self.otros);
+        let canonica: Vec<_> = self
+            .revisoras
+            .iter()
+            .chain(&self.otros)
+            .map(|r| {
+                (
+                    &r.id,
+                    &r.rol,
+                    &r.etiquetas,
+                    &r.dominios,
+                    r.prioridad,
+                    &r.patrones,
+                    r.fallback,
+                    r.umbral_dominios,
+                )
+            })
+            .collect();
+        let s = format!("{canonica:?}");
         crate::contrato::corto(&distribucion::sha256::hex(s.as_bytes()))
     }
 
@@ -383,5 +401,22 @@ mod tests {
         let b = Catalogo::nuevo(vec![r("a"), r("b")]);
         assert_eq!(a, b);
         assert_eq!(a.revisoras().len(), 2);
+        // La huella no depende de la ruta de lectura, si de lo declarado.
+        let en = |ruta: &str| Revisora {
+            ruta: ruta.to_owned(),
+            ..r("x")
+        };
+        let huella = |r: Revisora| Catalogo::nuevo(vec![r]).huella();
+        assert_eq!(
+            huella(en("C:/u1/.claude/x/SKILL.md")),
+            huella(en("/home/u2/.claude/x/SKILL.md"))
+        );
+        assert_ne!(
+            huella(en("x")),
+            huella(Revisora {
+                prioridad: 1,
+                ..en("x")
+            })
+        );
     }
 }
diff --git a/crates/tc2/src/seleccion/mod.rs b/crates/tc2/src/seleccion/mod.rs
index f17618f..bc0958a 100644
--- a/crates/tc2/src/seleccion/mod.rs
+++ b/crates/tc2/src/seleccion/mod.rs
@@ -1239,6 +1239,94 @@ mod tests {
         }
     }
 
+    /// S06 y S07 (decision 8): las skills de diseno de un dominio detectado;
+    /// la de ejecucion por mayor solape (etiquetas y dominios), despues mayor
+    /// prioridad, despues menor id, con aviso `empate-por-id`.
+    #[test]
+    fn skills_de_diseno_y_ejecucion_por_dominio_solape_prioridad_e_id() {
+        let (cat, inv) = catalogo_muestra(&[]);
+        let extra: Vec<crate::config::Recurso> = [
+            ("d-rend", "diseno", "", "rendimiento", 0),
+            ("d-puertas", "diseno", "", "puertas-seguridad", 0),
+            ("e-solape2", "ejecucion", "k1,k2", "", 0),
+            ("e-prio1", "ejecucion", "k1", "", 1),
+            ("e-prio5", "ejecucion", "k1", "", 5),
+            ("e-m", "ejecucion", "k3", "", 2),
+            ("e-n", "ejecucion", "k3", "", 2),
+            ("e-dom", "ejecucion", "", "rendimiento", 0),
+        ]
+        .iter()
+        .map(|(id, rol, etq, dom, prioridad)| {
+            let lista = |s: &str| {
+                s.split(',')
+                    .filter(|x| !x.is_empty())
+                    .map(str::to_owned)
+                    .collect()
+            };
+            crate::config::Recurso {
+                id: (*id).to_owned(),
+                tipo: "skill".to_owned(),
+                rol: Some((*rol).to_owned()),
+                dominios: lista(dom),
+                etiquetas: lista(etq),
+                prioridad: Some(*prioridad),
+            }
+        })
+        .collect();
+        let mut rs: Vec<Revisora> = cat.revisoras().iter().chain(cat.otros()).cloned().collect();
+        rs.extend(catalogo::de_config(&extra));
+        let mut es: Vec<Entrada> = inv.entradas().to_vec();
+        es.extend(extra.iter().map(|r| Entrada {
+            tipo: Tipo::Skill,
+            id: r.id.clone(),
+            origen: Origen::Proyecto,
+        }));
+        let (cat, inv) = (Catalogo::nuevo(rs), Inventario::nuevo(es, Vec::new()));
+        let criterios = vec!["criterio".to_owned()];
+        let con = |etq: &[&str]| {
+            let etq: Vec<String> = etq.iter().map(|e| (*e).to_owned()).collect();
+            let texto = dominios::Texto {
+                objetivo: "Cambio",
+                criterios: &criterios,
+                etiquetas: &etq,
+            };
+            let t = tres(
+                "feature",
+                Carril::Funcionalidad,
+                "simple",
+                &texto,
+                &[],
+                &cat,
+                &inv,
+                &todos(),
+                1,
+            );
+            (t.d, t.i)
+        };
+        // S06: solo la de diseno del dominio detectado.
+        let (d, i) = con(&["wasm"]);
+        assert_eq!(d.skills, ["d-rend"]);
+        assert!(d.reglas.contains(&"S06".to_owned()), "{d:?}");
+        // S07 por dominio (solape 1 con rendimiento).
+        assert_eq!(i.skills, ["e-dom"]);
+        let (d, _) = con(&["k1"]);
+        assert!(
+            d.skills.is_empty() && !d.reglas.contains(&"S06".to_owned()),
+            "{d:?}"
+        );
+        // Mayor solape gana a mayor prioridad.
+        let (_, i) = con(&["k1", "k2"]);
+        assert_eq!((i.skills, i.avisos), (vec!["e-solape2".to_owned()], vec![]));
+        // Mismo solape: mayor prioridad, aunque su id sea mayor.
+        let (_, i) = con(&["k1"]);
+        assert!(i.reglas.contains(&"S07".to_owned()), "{i:?}");
+        assert_eq!((i.skills, i.avisos), (vec!["e-prio5".to_owned()], vec![]));
+        // Mismo solape y prioridad: menor id, con aviso.
+        let (_, i) = con(&["k3"]);
+        assert_eq!(i.skills, ["e-m"]);
+        assert_eq!(i.avisos, ["empate-por-id:e-m,e-n"]);
+    }
+
     /// Criterio 3 (RR-17): el nucleo de `seleccion` no nombra procesos, red,
     /// ficheros ni clientes de modelos fuera de los tests; y un agente de
     /// diseno quitado del inventario sale con `no-instalado` (S14), sin
diff --git a/crates/tc2/src/seleccion/orden.rs b/crates/tc2/src/seleccion/orden.rs
index a1d01d9..521045f 100644
--- a/crates/tc2/src/seleccion/orden.rs
+++ b/crates/tc2/src/seleccion/orden.rs
@@ -160,10 +160,10 @@ pub fn fuentes(
     }
 }
 
-/// Carril efectivo y complejidad de una tarea: el carril mayor entre el
-/// declarado por el tipo y el congelado; la complejidad mayor entre la
-/// declarada y la heuristica (F1-15 decision 1).
-pub fn de_tarea(t: &Tarea, c: &Cuerpo) -> (Carril, &'static str) {
+/// Carril efectivo y resolucion de la heuristica de una tarea: el carril
+/// mayor entre el declarado por el tipo y el congelado; de la resolucion
+/// salen la complejidad ([`complejidad`]) y los roles de diseno.
+pub fn de_tarea(t: &Tarea, c: &Cuerpo) -> (Carril, heuristica::Resolucion) {
     let declarado = Carril::declarado(&t.tipo);
     let carril = t
         .carril
@@ -178,43 +178,63 @@ pub fn de_tarea(t: &Tarea, c: &Cuerpo) -> (Carril, &'static str) {
         &t.dependencias,
         &e,
     );
-    (carril, complejidad(&r))
+    (carril, r)
 }
 
-/// Los dominios de una tarea para la revision (F4-6; F1-15 decision 3):
-/// los del texto vivo y las `rutas` del diff con `dominios_extra` de la
-/// configuracion de la base, mas las senales congeladas con el plan
-/// aprobado (`roles.diseno.senales`): reescribir la tarea no quita un
-/// dominio. Sin tarea, solo las rutas.
+/// Los dominios del texto vivo de una tarea (objetivo y criterios de `e`,
+/// etiquetas de `t`) y de las `rutas`, con `dominios_extra` de la
+/// configuracion de la base (F1-15 decision 3). Sin tarea, solo las rutas.
+/// `Err` si la tabla del ADR no se lee.
+pub fn dominios_vivos(
+    tarea: Option<(&Tarea, &heuristica::Enunciado)>,
+    rutas: &[String],
+    cfg: &crate::config::Config,
+) -> Result<super::dominios::Dominios, String> {
+    let tabla = super::tabla::dominios(super::tabla::ADR)?;
+    let texto = match tarea {
+        Some((t, e)) => super::dominios::Texto {
+            objetivo: &e.objetivo,
+            criterios: &e.criterios,
+            etiquetas: &t.etiquetas,
+        },
+        None => super::dominios::Texto {
+            objetivo: "",
+            criterios: &[],
+            etiquetas: &[],
+        },
+    };
+    Ok(super::dominios::dominios(
+        &texto,
+        rutas,
+        &tabla,
+        &cfg.seleccion.dominios_extra,
+    ))
+}
+
+/// Los dominios de una tarea para la revision (F4-6): los de
+/// [`dominios_vivos`] mas las senales congeladas con el plan aprobado
+/// (`roles.diseno.senales`): reescribir la tarea no quita un dominio.
 pub fn dominios_de(
     tarea: Option<(&Tarea, &Cuerpo)>,
     rutas: &[String],
     cfg: &crate::config::Config,
-) -> super::dominios::Dominios {
-    let tabla = super::tabla::dominios(super::tabla::ADR).unwrap_or_default();
-    let extra = &cfg.seleccion.dominios_extra;
+) -> Result<super::dominios::Dominios, String> {
     match tarea {
         Some((t, c)) => {
             let e = heuristica::Enunciado::de(c);
-            let texto = super::dominios::Texto {
-                objetivo: &e.objetivo,
-                criterios: &e.criterios,
-                etiquetas: &t.etiquetas,
-            };
-            let d = super::dominios::dominios(&texto, rutas, &tabla, extra);
-            super::dominios::union(&d, &t.roles.diseno.senales)
-        }
-        None => {
-            let texto = super::dominios::Texto {
-                objetivo: "",
-                criterios: &[],
-                etiquetas: &[],
-            };
-            super::dominios::dominios(&texto, rutas, &tabla, extra)
+            let d = dominios_vivos(Some((t, &e)), rutas, cfg)?;
+            Ok(super::dominios::union(&d, &t.roles.diseno.senales))
         }
+        None => dominios_vivos(None, rutas, cfg),
     }
 }
 
+/// El informe de error de `comando` si las tablas del ADR no se leen.
+fn tabla_ilegible(comando: &str, error: String) -> Informe {
+    Informe::nuevo(comando, Resultado::Error)
+        .diagnostico(Mensaje::nuevo(&txt::TABLA_ILEGIBLE).con("error", error))
+}
+
 /// La complejidad de la seleccion: la mayor entre la declarada y la
 /// heuristica (F1-15 decision 1).
 pub fn complejidad(r: &heuristica::Resolucion) -> &'static str {
@@ -422,7 +442,7 @@ fn paso(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
         avisos.push("config-de-base-no-leida".to_owned());
         crate::config::Config::suelo()
     });
-    let mut doms = dominios_de(None, &rutas, &config);
+    let mut doms = dominios_de(None, &rutas, &config).map_err(|x| tabla_ilegible(NOMBRE, x))?;
     if let Some(id) = &a.tarea {
         let leida = crate::estado::leer(e.git, e.dir)
             .map_err(|i| i.valor("motivo").unwrap_or("estado").to_owned())
@@ -434,10 +454,11 @@ fn paso(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
             });
         match leida {
             Ok((t, c)) => {
-                let (k, x) = de_tarea(&t, &c);
+                let (k, r) = de_tarea(&t, &c);
                 carril = k;
-                cx = Some(x.to_owned());
-                doms = dominios_de(Some((&t, &c)), &rutas, &config);
+                cx = Some(complejidad(&r).to_owned());
+                doms = dominios_de(Some((&t, &c)), &rutas, &config)
+                    .map_err(|x| tabla_ilegible(NOMBRE, x))?;
             }
             Err(m) => avisos.push(format!("tarea-no-leida:{m}")),
         }
@@ -597,15 +618,7 @@ fn paso_seleccion(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
         avisos.push("config-de-base-no-leida".to_owned());
         crate::config::Config::suelo()
     });
-    let (carril, _) = de_tarea(&t, &c);
-    let enunciado = heuristica::Enunciado::de(&c);
-    let r = heuristica::resolver(
-        t.complejidad.as_deref(),
-        &t.etiquetas,
-        &t.tipo,
-        &t.dependencias,
-        &enunciado,
-    );
+    let (carril, r) = de_tarea(&t, &c);
     let cx = complejidad(&r);
     let n = carril.roles_diseno(r.roles, &t.tipo, &config);
     let ctx = Contexto {
@@ -614,8 +627,10 @@ fn paso_seleccion(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
         modelos_permitidos: &config.seleccion.modelos_permitidos,
         ronda: 1,
     };
-    let doms_plan = dominios_de(Some((&t, &c)), &[], &config);
-    let doms_rev = dominios_de(Some((&t, &c)), &rutas, &config);
+    let doms_plan =
+        dominios_de(Some((&t, &c)), &[], &config).map_err(|x| tabla_ilegible(SELECCION, x))?;
+    let doms_rev =
+        dominios_de(Some((&t, &c)), &rutas, &config).map_err(|x| tabla_ilegible(SELECCION, x))?;
     let sel = |rol, doms: &super::dominios::Dominios| {
         let entrada = super::Entrada {
             rutas: &rutas,
@@ -623,10 +638,8 @@ fn paso_seleccion(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
             etiquetas: &t.etiquetas,
             roles_base: n,
         };
-        super::seleccionar(rol, &entrada, &f.catalogo, &f.inventario, &ctx).map_err(|x| {
-            Informe::nuevo(SELECCION, Resultado::Error)
-                .diagnostico(Mensaje::nuevo(&txt::TABLA_ILEGIBLE).con("error", x))
-        })
+        super::seleccionar(rol, &entrada, &f.catalogo, &f.inventario, &ctx)
+            .map_err(|x| tabla_ilegible(SELECCION, x))
     };
     let d = sel(super::Rol::Diseno, &doms_plan)?;
     let im = sel(super::Rol::Implementacion, &doms_plan)?;
@@ -636,19 +649,9 @@ fn paso_seleccion(a: &[String], e: &Entorno<'_>) -> Result<Informe, Informe> {
     }
     let nombres = |d: &super::dominios::Dominios| {
         let v: Vec<String> = d.iter().map(|(x, _)| x.clone()).collect();
-        if v.is_empty() {
-            "ninguno".to_owned()
-        } else {
-            v.join(",")
-        }
-    };
-    let modelo = |s: &Seleccion| {
-        if s.modelo.is_empty() {
-            "-".to_owned()
-        } else {
-            s.modelo.clone()
-        }
+        crate::plan::lista_o_ninguno(&v)
     };
+    let modelo = |s: &Seleccion| crate::plan::o_guion(&s.modelo).to_owned();
     Ok(Informe::nuevo(SELECCION, Resultado::Ok)
         .con("id", t.id.as_str())
         .con("carril", carril.nombre())
diff --git a/crates/tc2/src/tarea/mod.rs b/crates/tc2/src/tarea/mod.rs
index fb22564..1f18919 100644
--- a/crates/tc2/src/tarea/mod.rs
+++ b/crates/tc2/src/tarea/mod.rs
@@ -19,8 +19,8 @@ use std::fmt;
 
 pub use cuerpo::{Cuerpo, PAUSA, REANUDAR, Seccion, Transicion};
 pub use modelo::{
-    Asignado, COMPLEJIDADES, ESTADOS, Heredado, LOGIN_MAX, REGLAS, Rol, RolRevision, Roles, TIPOS,
-    Tarea, TareaV1, id_valido, login_valido, titulo_valido,
+    Asignado, COMPLEJIDADES, ESTADOS, Heredado, LOGIN_MAX, REGLAS, Rol, Roles, TIPOS, Tarea,
+    TareaV1, id_valido, login_valido, titulo_valido,
 };
 
 /// Un `tarea.md` leido.
diff --git a/crates/tc2/src/tarea/modelo.rs b/crates/tc2/src/tarea/modelo.rs
index 785479d..576e978 100644
--- a/crates/tc2/src/tarea/modelo.rs
+++ b/crates/tc2/src/tarea/modelo.rs
@@ -266,10 +266,14 @@ fn asignado(v: Option<serde_json::Value>) -> Result<Option<Asignado>, ErrorTarea
     }
 }
 
-/// Un rol de `roles` (diseno, implementacion).
+/// Un rol de `roles` (diseno, implementacion o revision). `agente` y
+/// `ronda` son solo de revision: en diseno e implementacion se rechazan al
+/// leer y no se escriben.
 #[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
 #[serde(deny_unknown_fields)]
 pub struct Rol {
+    /// Agente que revisa (solo revision).
+    pub agente: Option<String>,
     /// Modelo del rol (contenido de F1-15).
     pub modelo: Option<String>,
     /// Skills del rol.
@@ -293,38 +297,7 @@ pub struct Rol {
     pub inventario: Option<String>,
     /// Huella del catalogo con el que se calculo.
     pub catalogo: Option<String>,
-}
-
-/// El rol de revision, que ademas nombra el agente.
-#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
-#[serde(deny_unknown_fields)]
-pub struct RolRevision {
-    /// Agente que revisa.
-    pub agente: Option<String>,
-    /// Modelo del revisor.
-    pub modelo: Option<String>,
-    /// Skills del revisor.
-    pub skills: Vec<String>,
-    /// Regla que eligio la skill (vocabulario de la 0.16).
-    pub regla: Option<String>,
-    /// Agentes elegidos (F1-15 decision 10; F4-6).
-    #[serde(default)]
-    pub agentes: Vec<String>,
-    /// Ids de las filas de las tablas de F1-15 que se aplicaron.
-    #[serde(default)]
-    pub reglas: Vec<String>,
-    /// Senales de dominio (`<dominio>:<senal>:<valor>`); en diseno son las
-    /// congeladas con el plan aprobado.
-    #[serde(default)]
-    pub senales: Vec<String>,
-    /// Avisos de la seleccion.
-    #[serde(default)]
-    pub avisos: Vec<String>,
-    /// Huella del inventario con el que se calculo.
-    pub inventario: Option<String>,
-    /// Huella del catalogo con el que se calculo.
-    pub catalogo: Option<String>,
-    /// Ronda de revision en la que se calculo.
+    /// Ronda de revision en la que se calculo (solo revision).
     pub ronda: Option<u32>,
 }
 
@@ -338,7 +311,7 @@ pub struct Roles {
     /// Implementacion.
     pub implementacion: Rol,
     /// Revision.
-    pub revision: RolRevision,
+    pub revision: Rol,
 }
 
 /// Tarea en formato 2 (F1-9, punto 3).
@@ -481,8 +454,15 @@ fn fecha(clave: &str, s: String) -> Result<String, ErrorTarea> {
     }
 }
 
-fn rol(clave: &str, r: Rol) -> Result<Rol, ErrorTarea> {
+/// `roles.<rol>` validado; `revision` dice si admite `agente` y `ronda`.
+fn rol(clave: &str, r: Rol, revision: bool) -> Result<Rol, ErrorTarea> {
+    for (k, hay) in [("agente", r.agente.is_some()), ("ronda", r.ronda.is_some())] {
+        if hay && !revision {
+            return Err(ErrorTarea::en(clave, format!("clave «{k}» desconocida")));
+        }
+    }
     Ok(Rol {
+        agente: texto_opcional(clave, r.agente)?,
         modelo: texto_opcional(clave, r.modelo)?,
         skills: textos(clave, r.skills)?,
         regla: uno_de_opcional(clave, r.regla, &REGLAS)?,
@@ -492,30 +472,25 @@ fn rol(clave: &str, r: Rol) -> Result<Rol, ErrorTarea> {
         avisos: textos(clave, r.avisos)?,
         inventario: texto_opcional(clave, r.inventario)?,
         catalogo: texto_opcional(clave, r.catalogo)?,
+        ronda: r.ronda,
     })
 }
 
 /// Las claves ampliadas de F4-6, solo las que tienen valor (una tarea de
 /// antes se reescribe byte a byte igual).
-fn ampliado(
-    agentes: &[String],
-    reglas: &[String],
-    senales: &[String],
-    avisos: &[String],
-    huellas: (Option<&str>, Option<&str>),
-) -> String {
+fn ampliado(r: &Rol) -> String {
     let mut s = String::new();
     for (k, v) in [
-        ("agentes", agentes),
-        ("reglas", reglas),
-        ("senales", senales),
-        ("avisos", avisos),
+        ("agentes", &r.agentes),
+        ("reglas", &r.reglas),
+        ("senales", &r.senales),
+        ("avisos", &r.avisos),
     ] {
         if !v.is_empty() {
             s.push_str(&format!(", {k}: {}", yaml::lista(v)));
         }
     }
-    for (k, v) in [("inventario", huellas.0), ("catalogo", huellas.1)] {
+    for (k, v) in [("inventario", &r.inventario), ("catalogo", &r.catalogo)] {
         if let Some(v) = v {
             s.push_str(&format!(", {k}: {}", yaml::cadena(v)));
         }
@@ -564,9 +539,9 @@ impl Tarea {
                     modelo: Some("sonnet".to_owned()),
                     ..Rol::default()
                 },
-                revision: RolRevision {
+                revision: Rol {
                     agente: Some("general-purpose".to_owned()),
-                    ..RolRevision::default()
+                    ..Rol::default()
                 },
             },
             estimacion: None,
@@ -638,7 +613,6 @@ impl Tarea {
             ));
         }
         let roles = exigir("roles", c.roles)?;
-        let revision = roles.revision;
         let estimacion = c.estimacion.and_then(|_| {
             partes
                 .entradas
@@ -660,21 +634,9 @@ impl Tarea {
             rama: texto_opcional("rama", c.rama)?,
             asignado_a: asignado(c.asignado_a)?,
             roles: Roles {
-                diseno: rol("roles.diseno", roles.diseno)?,
-                implementacion: rol("roles.implementacion", roles.implementacion)?,
-                revision: RolRevision {
-                    agente: texto_opcional("roles.revision", revision.agente)?,
-                    modelo: texto_opcional("roles.revision", revision.modelo)?,
-                    skills: textos("roles.revision", revision.skills)?,
-                    regla: uno_de_opcional("roles.revision", revision.regla, &REGLAS)?,
-                    agentes: textos("roles.revision", revision.agentes)?,
-                    reglas: textos("roles.revision", revision.reglas)?,
-                    senales: textos("roles.revision", revision.senales)?,
-                    avisos: textos("roles.revision", revision.avisos)?,
-                    inventario: texto_opcional("roles.revision", revision.inventario)?,
-                    catalogo: texto_opcional("roles.revision", revision.catalogo)?,
-                    ronda: revision.ronda,
-                },
+                diseno: rol("roles.diseno", roles.diseno, false)?,
+                implementacion: rol("roles.implementacion", roles.implementacion, false)?,
+                revision: rol("roles.revision", roles.revision, true)?,
             },
             estimacion,
             tokens_diseno: c.tokens_diseno,
@@ -699,19 +661,20 @@ impl Tarea {
     /// con `\n`; el cuerpo, byte a byte como se leyo).
     pub fn escribir(&self, cuerpo: &Cuerpo) -> String {
         let r = &self.roles;
-        let rol = |x: &Rol| {
+        // Revision lleva `agente` delante; `ronda` solo sale con valor.
+        let rol = |x: &Rol, revision: bool| {
+            let agente = if revision {
+                format!("agente: {}, ", yaml::opcional(x.agente.as_deref()))
+            } else {
+                String::new()
+            };
+            let ronda = x.ronda.map(|n| format!(", ronda: {n}")).unwrap_or_default();
             format!(
-                "{{modelo: {}, skills: {}, regla: {}{}}}",
+                "{{{agente}modelo: {}, skills: {}, regla: {}{}{ronda}}}",
                 yaml::opcional(x.modelo.as_deref()),
                 yaml::lista(&x.skills),
                 yaml::opcional(x.regla.as_deref()),
-                ampliado(
-                    &x.agentes,
-                    &x.reglas,
-                    &x.senales,
-                    &x.avisos,
-                    (x.inventario.as_deref(), x.catalogo.as_deref())
-                )
+                ampliado(x)
             )
         };
         let mut s = String::from("---\nformato: 2\n");
@@ -738,29 +701,11 @@ impl Tarea {
                 .as_ref()
                 .map_or_else(|| "null".to_owned(), Asignado::yaml),
         );
-        let rv = &r.revision;
-        let ronda = rv
-            .ronda
-            .map(|n| format!(", ronda: {n}"))
-            .unwrap_or_default();
-        let revision = format!(
-            "{{agente: {}, modelo: {}, skills: {}, regla: {}{}{ronda}}}",
-            yaml::opcional(rv.agente.as_deref()),
-            yaml::opcional(rv.modelo.as_deref()),
-            yaml::lista(&rv.skills),
-            yaml::opcional(rv.regla.as_deref()),
-            ampliado(
-                &rv.agentes,
-                &rv.reglas,
-                &rv.senales,
-                &rv.avisos,
-                (rv.inventario.as_deref(), rv.catalogo.as_deref())
-            )
-        );
         let roles = format!(
-            "\n  diseno: {}\n  implementacion: {}\n  revision: {revision}",
-            rol(&r.diseno),
-            rol(&r.implementacion)
+            "\n  diseno: {}\n  implementacion: {}\n  revision: {}",
+            rol(&r.diseno, false),
+            rol(&r.implementacion, false),
+            rol(&r.revision, true)
         );
         s.push_str("roles:");
         s.push_str(&roles);
diff --git a/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md b/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
index e065676..4ba75a8 100644
--- a/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
+++ b/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
@@ -967,9 +967,16 @@ ni modelo y S14 en diseño; tabla adversaria del detector), `tests/tarea.rs`
 carril y complejidad, agente y modelo en cada petición), `tests/aprobar.rs`
 (`plan`, `approve`, revisión con el texto reescrito y `doctor --recursos`) y
 `tests/seleccion.rs` (10 tareas terminadas de TaskCodeTwo por regla fija,
-deterministas).
+deterministas, comparadas con la tabla dorada `tabla.md` de su fixture).
+S06 y el orden de S07 (solape, prioridad, id y `empate-por-id`) los cubre
+el unitario `skills_de_diseno_y_ejecucion_por_dominio_solape_prioridad_e_id`.
 
 **No verificado**: que las 10 tareas de TaskCodeTwo coincidan con lo que
-haría una persona (lo marca Carlos tras el cierre, D10a); el orquestador
+haría una persona (lo marca Carlos tras el cierre, D10a). En esa tabla la
+columna «Implementación» sale «—» en las 10 y no hay skills de diseño: el
+inventario del test no tiene ninguna skill con `rol: ejecucion` ni
+`rol: diseno` en su bloque yaml, y sin metadatos S06 y S07 no eligen nada.
+La comparación juzga, por tanto, los agentes de diseño, la revisión y los
+modelos, no las skills de implementación ni de diseño; el orquestador
 (F4-4) consumiendo la selección; el revisor en CI con dominios en un PR real
 (E2E).
````

## Excluido del diff (8 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 0689c2cec9429cd0695cd68458dcdad880695989..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-085/evidencias/tabla-10-tareas-taskcodetwo.md   |    4 +-
 tareas/{02-en-curso => 03-en-revision}/TASK-085/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-085/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-085/revision/informe-revision-1-code-quality-reviewer.md         |   65 +++
 tareas/03-en-revision/TASK-085/revision/informe-revision-1-rust-wasm-reviewer.md            |   65 +++
 tareas/03-en-revision/TASK-085/revision/peticion-revision-1-code-quality-reviewer.md        |  284 ++++++++++
 tareas/03-en-revision/TASK-085/revision/peticion-revision-1-rust-wasm-reviewer.md           | 2935 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-085/tarea.md                                    |   11 +-
 8 files changed, 3360 insertions(+), 4 deletions(-)
````

## Tests del diff (4 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 0689c2cec9429cd0695cd68458dcdad880695989..HEAD -- ":(literal)crates/tc2/tests/fixtures/seleccion-taskcodetwo/estratos.txt" ":(literal)crates/tc2/tests/fixtures/seleccion-taskcodetwo/tabla.md" ":(literal)crates/tc2/tests/seleccion.rs" ":(literal)crates/tc2/tests/tarea.rs"
````

````
crates/tc2/tests/fixtures/seleccion-taskcodetwo/estratos.txt | 15 +++++++++++++++
 crates/tc2/tests/fixtures/seleccion-taskcodetwo/tabla.md     | 12 ++++++++++++
 crates/tc2/tests/seleccion.rs                                | 25 +++++++++++++++++--------
 crates/tc2/tests/tarea.rs                                    |  9 +++++++++
 4 files changed, 53 insertions(+), 8 deletions(-)
````
