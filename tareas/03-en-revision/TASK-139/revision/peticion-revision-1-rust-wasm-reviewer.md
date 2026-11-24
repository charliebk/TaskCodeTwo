# Peticion de revision — TASK-139 (ronda 1)

- Tarea: TASK-139 — Auditoria 4D: A1 plataforma e identidad (11 IMPORTANTE)
- Rama revisada: fix/task-139-auditoria-4d-a1-plataforma-e-identidad-1
- Rama base: develop
- Commit revisado (HEAD): ff05668b1bee0f89850a7d1ffb3ca0ed3cf3d2b0
- Fecha: 2026-11-24
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-139 (criterios de aceptacion y plan)
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
informe de esta ronda (informe-revision-1-rust-wasm-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-139/tarea.md
- tareas/03-en-revision/TASK-139/planificacion/
- crates/tc2/src/i18n/catalogo/plataforma.rs
- crates/tc2/src/plataforma/github.rs
- crates/tc2/src/plataforma/gitlab.rs
- crates/tc2/src/plataforma/http.rs
- crates/tc2/src/plataforma/mod.rs
- crates/tc2/src/plataforma/operaciones.rs
- crates/tc2/src/plataforma/puertas.rs
- crates/tc2/tests/arnes_e2e/mod.rs
- crates/tc2/tests/comun/adaptadores.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/plataforma.rs
- crates/tc2/tests/plataforma_e2e.rs
- crates/tc2/tests/plataforma_gitlab.rs
- crates/tc2/tests/plataforma_gitlab_e2e.rs
- crates/tc2/tests/prs_abiertos.rs

## Commits a revisar (git log develop..HEAD)

````
ff05668 docs(TASK-139): resultado, criterios y autorrevision
c65d986 fix(TASK-139): A1-06 cliente HTTP del arnes E2E en tests/arnes_e2e
26b6928 fix(TASK-139): A1-04 tabla de errores HTTP y servido de fixtures comunes
26ad763 fix(TASK-139): A1-05 un solo constructor de adaptadores en tests/comun/adaptadores
13a43ba fix(TASK-139): A1-07 leer-puertas pagina los rulesets
c8a7acf fix(TASK-139): A1-01 leer-puertas en lote en operaciones::leer_en_lote
1786fca fix(TASK-139): A1-03 reglas de prs-abiertos en operaciones::prs_abiertos
82a0514 fix(TASK-139): A1-02 abrir-pr y merge-pr neutrales en plataforma/operaciones
84fbaa9 fix(TASK-139): A1-11 caso del 403 con retry-after sin remaining
c2c33f4 fix(TASK-139): A1-08 bases con host que empieza por punto o guion
b5cd0a9 fix(TASK-139): A1-10 el revisor no lee puertas por el rol, con su token
1dc5af3 fix(TASK-139): A1-09 prs-abiertos prueba las reglas en GitHub y GitLab
30596e1 chore(TASK-139): tarea en curso
````

## Diff de tu dominio (15 fichero(s) de 15; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/i18n/catalogo/plataforma.rs b/crates/tc2/src/i18n/catalogo/plataforma.rs
index a4dc504..e8864ce 100644
--- a/crates/tc2/src/i18n/catalogo/plataforma.rs
+++ b/crates/tc2/src/i18n/catalogo/plataforma.rs
@@ -188,6 +188,10 @@ crate::i18n::textos! {
         es: "el scope api y el rol Maintainer en el proyecto",
         en: "the api scope and the Maintainer role in the project",
     }
+    PR_EXISTENTE {
+        es: "GitHub dice que ya hay un PR para la rama pero no se encuentra abierto: reintenta (GitHub: «{api}»)",
+        en: "GitHub says there is already a PR for the branch but no open one is found: try again (GitHub: «{api}»)",
+    }
     MR_EXISTENTE {
         es: "GitLab dice que ya hay un MR para la rama pero no se encuentra abierto: reintenta (GitLab: «{api}»)",
         en: "GitLab says there is already an MR for the branch but no open one is found: try again (GitLab: «{api}»)",
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
index a5ac3c7..4112845 100644
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -15,24 +15,24 @@ use super::conexion::{Conexion, head_cambiado, rama_valida, rol_no_valido, sha_v
 use super::credenciales::{Credenciales, Rol};
 use super::errores::{self, es_aviso_de_plan};
 use super::http::{Base, Metodo, codificar};
+use super::operaciones::{
+    self, ABRIR_PR, AbrePr, Creado, LEER_PUERTAS, MERGE_PR, PRS_ABIERTOS, PrLeido, entrada_merge,
+    merge_fallido, segun_estado,
+};
 use super::puertas::{self, EstadoCheck, Lecturas, Origenes, Puerta};
 use super::{
     Creada, EstadoPuerta, FalloLectura, Firma, LEER_REVISION, ModeloPuertas, Paso, Personas,
     PlanObservado, Plataforma, PrAbierto, Proveedor, PuertasLeidas, ReglasLeidas,
     RevisionPublicada, Usuario, aplicar_en_lote,
 };
-use crate::contrato::{Informe, Resultado, Valor, corto};
+use crate::contrato::{Informe, Resultado, corto};
 use crate::hallazgos::ronda;
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::plataforma as t;
 
-const ABRIR_PR: &str = "abrir-pr";
-const MERGE_PR: &str = "merge-pr";
-const LEER_PUERTAS: &str = "leer-puertas";
 const LEER_PERMISOS_CI: &str = "leer-permisos-ci";
 const FIRMA_COMMIT: &str = "firma-commit";
 const IDENTIDAD: &str = "identidad";
-const PRS_ABIERTOS: &str = "prs-abiertos";
 const LEER_PLAN: &str = "leer-plan";
 
 /// Permiso de `leer-puertas` (lo nombra `doctor` en `permiso=`).
@@ -376,7 +376,13 @@ impl GitHub {
         self.con.exigir_persona(op, self.identidad(Rol::Persona))
     }
 
-    fn pr_abierto(&self, rama: &str, base: &str) -> Paso<Option<u64>> {
+    fn dueno(&self) -> &str {
+        self.repo.split('/').next().unwrap_or_default()
+    }
+}
+
+impl AbrePr for GitHub {
+    fn abierto(&self, rama: &str, base: &str) -> Paso<Option<u64>> {
         let cabeza = format!("{}:{rama}", self.dueno());
         let (_, ruta) = self.ep(&Endpoint::PrsAbiertos {
             cabeza: &cabeza,
@@ -393,57 +399,7 @@ impl GitHub {
             .map(|p| p.number))
     }
 
-    fn dueno(&self) -> &str {
-        self.repo.split('/').next().unwrap_or_default()
-    }
-}
-
-impl Plataforma for GitHub {
-    fn prs_abiertos(&self, rol: Rol) -> Paso<Vec<PrAbierto>> {
-        if rol == Rol::Revisor {
-            return Err(rol_no_valido(PRS_ABIERTOS, rol));
-        }
-        let (_, ruta) = self.ep(&Endpoint::TodosLosPrsAbiertos);
-        let prs: Vec<PrListaApi> =
-            self.con
-                .leer_paginas(PRS_ABIERTOS, rol, "pull_requests:read", ruta, |c| {
-                    self.con.parsear(PRS_ABIERTOS, c)
-                })?;
-        let mut v = Vec::new();
-        for p in prs {
-            let propio = matches!((&p.head.repo, &p.base.repo), (Some(h), Some(b)) if h.id == b.id);
-            // Una rama que tc2 no habria creado no casa con ninguna tarea.
-            if !propio || !rama_valida(&p.head.nombre) {
-                continue;
-            }
-            if !sha_valido(&p.head.sha) {
-                return Err(self.con.invalida_por(PRS_ABIERTOS, &t::RAMA_BASE_O_SHA));
-            }
-            v.push(PrAbierto {
-                numero: p.number,
-                rama: p.head.nombre,
-                sha: p.head.sha.to_ascii_lowercase(),
-                borrador: p.draft,
-            });
-        }
-        v.sort_by_key(|p| p.numero);
-        Ok(v)
-    }
-
-    fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe {
-        if !rama_valida(rama) || !rama_valida(base) {
-            return uso(ABRIR_PR, "rama-invalida", &t::RAMA_NO_VALIDA);
-        }
-        if titulo.trim().is_empty() || titulo.chars().any(char::is_control) {
-            return uso(ABRIR_PR, "titulo-invalido", &t::TITULO_NO_VALIDO);
-        }
-        match self.pr_abierto(rama, base) {
-            Ok(Some(n)) => {
-                return Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string());
-            }
-            Ok(None) => {}
-            Err(i) => return i,
-        }
+    fn crear(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Creado {
         let datos = serde_json::json!({
             "title": titulo, "head": rama, "base": base, "body": cuerpo,
         })
@@ -455,30 +411,56 @@ impl Plataforma for GitHub {
             Some(&datos),
         ) {
             Ok(r) => r,
-            Err(i) => return i,
+            Err(i) => return Creado::Fallo(i),
         };
         if r.estado == 422 && r.cuerpo.contains("already exists") {
-            // Otro proceso lo abrio entre la lectura y la escritura.
-            return match self.pr_abierto(rama, base) {
-                Ok(Some(n)) => {
-                    Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string())
-                }
-                Ok(None) => self
-                    .con
-                    .traducir(ABRIR_PR, Rol::Agente, "pull_requests:write", &r),
-                Err(i) => i,
-            };
+            return Creado::YaExiste(
+                Mensaje::nuevo(&t::PR_EXISTENTE).con("api", errores::mensaje_api(&r)),
+            );
         }
         if !r.exito() {
-            return self
-                .con
-                .traducir(ABRIR_PR, Rol::Agente, "pull_requests:write", &r);
+            return Creado::Fallo(self.con.traducir(
+                ABRIR_PR,
+                Rol::Agente,
+                "pull_requests:write",
+                &r,
+            ));
         }
         match self.con.parsear::<PrApi>(ABRIR_PR, &r.cuerpo) {
-            Ok(p) => Informe::nuevo(ABRIR_PR, Resultado::Ok).con("pr", p.number.to_string()),
-            Err(i) => i,
+            Ok(p) => Creado::Pr(p.number),
+            Err(i) => Creado::Fallo(i),
         }
     }
+}
+
+impl Plataforma for GitHub {
+    fn prs_abiertos(&self, rol: Rol) -> Paso<Vec<PrAbierto>> {
+        let leer = || {
+            let (_, ruta) = self.ep(&Endpoint::TodosLosPrsAbiertos);
+            let prs: Vec<PrListaApi> =
+                self.con
+                    .leer_paginas(PRS_ABIERTOS, rol, "pull_requests:read", ruta, |c| {
+                        self.con.parsear(PRS_ABIERTOS, c)
+                    })?;
+            Ok(prs
+                .into_iter()
+                .map(|p| PrLeido {
+                    propio: matches!((&p.head.repo, &p.base.repo), (Some(h), Some(b)) if h.id == b.id),
+                    numero: p.number,
+                    rama: p.head.nombre,
+                    sha: p.head.sha,
+                    borrador: p.draft,
+                })
+                .collect())
+        };
+        operaciones::prs_abiertos(rol, leer, || {
+            self.con.invalida_por(PRS_ABIERTOS, &t::RAMA_BASE_O_SHA)
+        })
+    }
+
+    fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe {
+        operaciones::abrir_pr(self, rama, base, titulo, cuerpo)
+    }
 
     fn estado_puertas(&self, rol: Rol, pr: u64, modelo: &ModeloPuertas) -> Informe {
         let op = puertas::ESTADO_PUERTAS;
@@ -496,11 +478,8 @@ impl Plataforma for GitHub {
             Ok(p) => p,
             Err(i) => return i,
         };
-        if p.merged {
-            return puertas::mergeado(pr);
-        }
-        if p.state != "open" {
-            return puertas::pr_cerrado(pr);
+        if let Err(i) = segun_estado(op, pr, p.merged, p.state == "open", &t::PR_CERRADO_REABRE) {
+            return i;
         }
         if !rama_valida(&p.base.nombre) || !sha_valido(&p.head.sha) {
             return self.con.invalida_por(op, &t::RAMA_BASE_O_SHA);
@@ -671,10 +650,7 @@ impl Plataforma for GitHub {
     }
 
     fn merge_pr(&self, pr: u64, sha: &str) -> Informe {
-        if !sha_valido(sha) {
-            return uso(MERGE_PR, "sha-invalido", &t::SHA_ESPERADO_COMPLETO);
-        }
-        if let Err(i) = self.exigir_persona(MERGE_PR) {
+        if let Err(i) = entrada_merge(sha, || self.exigir_persona(MERGE_PR)) {
             return i;
         }
         let permiso = "contents:write";
@@ -686,16 +662,8 @@ impl Plataforma for GitHub {
                 Ok(p) => p,
                 Err(i) => return i,
             };
-        if p.merged {
-            return Informe::nuevo(MERGE_PR, Resultado::SinCambios)
-                .con("pr", pr.to_string())
-                .con("estado", "mergeado");
-        }
-        if p.state != "open" {
-            return Informe::nuevo(MERGE_PR, Resultado::Rechazada)
-                .con("pr", pr.to_string())
-                .con("motivo", "pr-cerrado")
-                .diagnostico(&t::PR_CERRADO);
+        if let Err(i) = segun_estado(MERGE_PR, pr, p.merged, p.state == "open", &t::PR_CERRADO) {
+            return i;
         }
         if !p.head.sha.eq_ignore_ascii_case(sha) {
             return head_cambiado(pr);
@@ -711,10 +679,7 @@ impl Plataforma for GitHub {
             Err(i) => return i,
         };
         if !r.exito() {
-            let mut i = self.con.traducir(MERGE_PR, Rol::Persona, permiso, &r);
-            i.claves
-                .insert(0, ("pr".to_owned(), Valor::Texto(pr.to_string())));
-            return i;
+            return merge_fallido(pr, self.con.traducir(MERGE_PR, Rol::Persona, permiso, &r));
         }
         match self.con.parsear::<MergeApi>(MERGE_PR, &r.cuerpo) {
             Ok(MergeApi {
@@ -732,11 +697,12 @@ impl Plataforma for GitHub {
     }
 
     fn leer_puertas(&self, rol: Rol, rama: &str) -> Paso<ReglasLeidas> {
-        validar_puertas(rol, rama)?;
-        match self.rulesets(rol)? {
-            Some(rulesets) => self.puertas_de_rama(rol, rama, &rulesets, true),
-            None => Ok(sin_reglas_por_plan()),
-        }
+        operaciones::leer_puertas(
+            rol,
+            rama,
+            || self.rulesets(rol),
+            |rs, rama, checks| self.puertas_de_rama(rol, rama, rs.as_deref(), checks),
+        )
     }
 
     fn leer_puertas_de(
@@ -745,27 +711,14 @@ impl Plataforma for GitHub {
         ramas: &[&str],
         checks_de: &str,
     ) -> Vec<Paso<ReglasLeidas>> {
-        // Los rulesets son del repo: se leen una vez (solo si alguna rama
-        // es valida) y valen para todas.
-        let validas: Vec<Paso<()>> = ramas.iter().map(|r| validar_puertas(rol, r)).collect();
-        let comun = if validas.iter().any(Result::is_ok) {
-            Some(self.rulesets(rol))
-        } else {
-            None
-        };
-        ramas
-            .iter()
-            .zip(validas)
-            .map(|(rama, v)| {
-                v?;
-                match comun.as_ref() {
-                    Some(Ok(Some(rs))) => self.puertas_de_rama(rol, rama, rs, *rama == checks_de),
-                    Some(Ok(None)) => Ok(sin_reglas_por_plan()),
-                    Some(Err(i)) => Err(i.clone()),
-                    None => Err(rol_no_valido(LEER_PUERTAS, rol)),
-                }
-            })
-            .collect()
+        // Los rulesets son del repo: valen para todas las ramas.
+        operaciones::leer_en_lote(
+            rol,
+            ramas,
+            checks_de,
+            || self.rulesets(rol),
+            |rs, rama, checks| self.puertas_de_rama(rol, rama, rs.as_deref(), checks),
+        )
     }
 
     fn leer_permisos_ci(&self) -> Informe {
@@ -936,41 +889,18 @@ impl Plataforma for GitHub {
     }
 }
 
-/// Rama y rol de `leer-puertas`.
-fn validar_puertas(rol: Rol, rama: &str) -> Paso<()> {
-    if !rama_valida(rama) {
-        return Err(uso(LEER_PUERTAS, "rama-invalida", &t::RAMA_NO_VALIDA));
-    }
-    if rol == Rol::Revisor {
-        return Err(rol_no_valido(LEER_PUERTAS, rol));
-    }
-    Ok(())
-}
-
-/// Lo leido cuando el plan no da los rulesets (403 con aviso de plan).
-fn sin_reglas_por_plan() -> ReglasLeidas {
-    ReglasLeidas {
-        plan_da_reglas: false,
-        ..ReglasLeidas::default()
-    }
-}
-
 impl GitHub {
-    /// Los nombres de los rulesets del repo; `None` si el plan no los da.
+    /// Los nombres de todos los rulesets del repo para `leer-puertas`,
+    /// paginados ([`GitHub::rulesets_de`]); `None` si el plan no los da (el
+    /// 403 de plan, que en `leer-puertas` no es un fallo). Un 403 que ademas
+    /// es limite de peticiones es `agotada`.
     fn rulesets(&self, rol: Rol) -> Paso<Option<Vec<String>>> {
-        let r = self
-            .con
-            .pedir(LEER_PUERTAS, rol, self.ep(&Endpoint::Rulesets), None)?;
-        if es_aviso_de_plan(&r) {
-            return Ok(None);
-        }
-        if !r.exito() {
-            return Err(self
-                .con
-                .traducir(LEER_PUERTAS, rol, PERMISO_LEER_PUERTAS, &r));
+        match self.rulesets_de(LEER_PUERTAS, rol, PERMISO_LEER_PUERTAS) {
+            Err(i) if i.resultado == Resultado::Rechazada && i.valor("motivo") == Some("plan") => {
+                Ok(None)
+            }
+            r => r.map(Some),
         }
-        let rulesets: Vec<RulesetApi> = self.con.parsear(LEER_PUERTAS, &r.cuerpo)?;
-        Ok(Some(rulesets.into_iter().map(|r| r.name).collect()))
     }
 
     /// Los nombres de todos los rulesets del repo para `op`, paginados; el
@@ -983,15 +913,22 @@ impl GitHub {
         Ok(rulesets.into_iter().map(|r| r.name).collect())
     }
 
-    /// Las reglas de `rama` con los `rulesets` ya leidos; los check-runs de
-    /// su cabeza solo `con_checks`.
+    /// Las reglas de `rama` con los `rulesets` ya leidos (`None`: el plan no
+    /// los da, 403 con aviso, y no se sigue leyendo); los check-runs de su
+    /// cabeza solo `con_checks`.
     fn puertas_de_rama(
         &self,
         rol: Rol,
         rama: &str,
-        rulesets: &[String],
+        rulesets: Option<&[String]>,
         con_checks: bool,
     ) -> Paso<ReglasLeidas> {
+        let Some(rulesets) = rulesets else {
+            return Ok(ReglasLeidas {
+                plan_da_reglas: false,
+                ..ReglasLeidas::default()
+            });
+        };
         let reglas: Vec<ReglaApi> = self.con.leer(
             LEER_PUERTAS,
             rol,
diff --git a/crates/tc2/src/plataforma/gitlab.rs b/crates/tc2/src/plataforma/gitlab.rs
index 7060ff3..484c4bc 100644
--- a/crates/tc2/src/plataforma/gitlab.rs
+++ b/crates/tc2/src/plataforma/gitlab.rs
@@ -16,23 +16,23 @@ use super::conexion::{Conexion, head_cambiado, rama_valida, rol_no_valido, sha_v
 use super::credenciales::{Credenciales, Rol};
 use super::errores;
 use super::http::{Base, Metodo, codificar};
+use super::operaciones::{
+    self, ABRIR_PR, AbrePr, Creado, LEER_PUERTAS, MERGE_PR, PRS_ABIERTOS, PrLeido, entrada_merge,
+    merge_fallido, segun_estado,
+};
 use super::puertas::{self, EstadoCheck, Lecturas, Objetivo, Origenes, Puerta};
 use super::{
     Clave, Creada, EstadoPuerta, FalloLectura, Firma, ModeloPuertas, Paso, Personas, PlanObservado,
     Plataforma, PrAbierto, Proveedor, PuertasLeidas, ReglasLeidas, TierGitLab, Usuario,
     aplicar_en_lote,
 };
-use crate::contrato::{Informe, Resultado, Valor, corto};
+use crate::contrato::{Informe, Resultado, corto};
 use crate::i18n::catalogo::plataforma as t;
 use crate::i18n::{Mensaje, Texto};
 
-const ABRIR_PR: &str = "abrir-pr";
-const MERGE_PR: &str = "merge-pr";
-const LEER_PUERTAS: &str = "leer-puertas";
 const LEER_PERMISOS_CI: &str = "leer-permisos-ci";
 const FIRMA_COMMIT: &str = "firma-commit";
 const IDENTIDAD: &str = "identidad";
-const PRS_ABIERTOS: &str = "prs-abiertos";
 const LEER_PLAN: &str = "leer-plan";
 
 /// El scope de lectura en una clave (`doctor`, `permiso=`); el texto
@@ -423,19 +423,6 @@ impl GitLab {
         self.con.exigir_persona(op, self.identidad(Rol::Persona))
     }
 
-    fn mr_abierto(&self, rama: &str, base: &str) -> Paso<Option<u64>> {
-        let (_, ruta) = self.ep(&Endpoint::MrsAbiertos { rama, base });
-        let mrs: Vec<MrApi> =
-            self.con
-                .leer_paginas(ABRIR_PR, Rol::Agente, PERMISO_LECTURA, ruta, |c| {
-                    self.con.parsear(ABRIR_PR, c)
-                })?;
-        Ok(mrs
-            .into_iter()
-            .find(|m| m.source_branch == rama && m.target_branch == base && m.state == "opened")
-            .map(|m| m.iid))
-    }
-
     /// Checks requeridos del proyecto (F3-8): GitLab no exige checks por
     /// nombre; con «Pipelines must succeed» cuenta el pipeline entero
     /// (`todos`). `None` si el proyecto no se pudo leer o no trae el campo
@@ -479,16 +466,70 @@ impl GitLab {
             .map_err(fallo)?;
         Ok(estados.into_iter().map(|e| e.name).collect())
     }
+}
 
-    /// Si `rama` tiene commits que no estan en `base`.
-    fn tiene_commits(&self, rama: &str, base: &str) -> Paso<bool> {
+impl AbrePr for GitLab {
+    fn abierto(&self, rama: &str, base: &str) -> Paso<Option<u64>> {
+        let (_, ruta) = self.ep(&Endpoint::MrsAbiertos { rama, base });
+        let mrs: Vec<MrApi> =
+            self.con
+                .leer_paginas(ABRIR_PR, Rol::Agente, PERMISO_LECTURA, ruta, |c| {
+                    self.con.parsear(ABRIR_PR, c)
+                })?;
+        Ok(mrs
+            .into_iter()
+            .find(|m| m.source_branch == rama && m.target_branch == base && m.state == "opened")
+            .map(|m| m.iid))
+    }
+
+    /// GitLab abre un MR sin commits: se comprueba antes (que `rama` tenga
+    /// commits que no estan en `base`) para dar la misma linea que GitHub.
+    fn antes_de_crear(&self, rama: &str, base: &str) -> Paso<()> {
         let commits: Vec<CommitApi> = self.con.leer(
             ABRIR_PR,
             Rol::Agente,
             PERMISO_LECTURA,
             self.ep(&Endpoint::CommitsEntre { base, rama }),
         )?;
-        Ok(commits.iter().any(|c| !c.id.is_empty()))
+        if commits.iter().any(|c| !c.id.is_empty()) {
+            Ok(())
+        } else {
+            Err(Informe::nuevo(ABRIR_PR, Resultado::Rechazada)
+                .con("motivo", "sin-commits")
+                .diagnostico(Mensaje::nuevo(&t::SIN_COMMITS).con("cita", "")))
+        }
+    }
+
+    fn crear(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Creado {
+        let datos = serde_json::json!({
+            "source_branch": rama, "target_branch": base,
+            "title": titulo, "description": cuerpo,
+        })
+        .to_string();
+        let r = match self.con.pedir(
+            ABRIR_PR,
+            Rol::Agente,
+            self.ep(&Endpoint::CrearMr),
+            Some(&datos),
+        ) {
+            Ok(r) => r,
+            Err(i) => return Creado::Fallo(i),
+        };
+        if r.estado == 409 {
+            return Creado::YaExiste(
+                Mensaje::nuevo(&t::MR_EXISTENTE).con("api", errores::mensaje_api(&r)),
+            );
+        }
+        if !r.exito() {
+            return Creado::Fallo(
+                self.con
+                    .traducir(ABRIR_PR, Rol::Agente, PERMISO_ESCRITURA, &r),
+            );
+        }
+        match self.con.parsear::<MrApi>(ABRIR_PR, &r.cuerpo) {
+            Ok(m) => Creado::Pr(m.iid),
+            Err(i) => Creado::Fallo(i),
+        }
     }
 }
 
@@ -526,99 +567,32 @@ fn estado_check(s: &str) -> EstadoCheck {
 
 impl Plataforma for GitLab {
     fn prs_abiertos(&self, rol: Rol) -> Paso<Vec<PrAbierto>> {
-        if rol == Rol::Revisor {
-            return Err(rol_no_valido(PRS_ABIERTOS, rol));
-        }
-        let (_, ruta) = self.ep(&Endpoint::TodosLosMrsAbiertos);
-        let mrs: Vec<MrListaApi> =
-            self.con
-                .leer_paginas(PRS_ABIERTOS, rol, PERMISO_LECTURA, ruta, |c| {
-                    self.con.parsear(PRS_ABIERTOS, c)
-                })?;
-        let mut v = Vec::new();
-        for m in mrs {
-            // Los MR desde un fork no cuentan; una rama que tc2 no habria
-            // creado no casa con ninguna tarea.
-            if m.source_project_id != m.target_project_id || !rama_valida(&m.source_branch) {
-                continue;
-            }
-            let sha = m.sha.unwrap_or_default();
-            if !sha_valido(&sha) {
-                return Err(self.con.invalida_por(PRS_ABIERTOS, &t::RAMA_DESTINO_O_SHA));
-            }
-            v.push(PrAbierto {
-                numero: m.iid,
-                rama: m.source_branch,
-                sha: sha.to_ascii_lowercase(),
-                borrador: m.draft || m.work_in_progress,
-            });
-        }
-        v.sort_by_key(|p| p.numero);
-        Ok(v)
+        let leer = || {
+            let (_, ruta) = self.ep(&Endpoint::TodosLosMrsAbiertos);
+            let mrs: Vec<MrListaApi> =
+                self.con
+                    .leer_paginas(PRS_ABIERTOS, rol, PERMISO_LECTURA, ruta, |c| {
+                        self.con.parsear(PRS_ABIERTOS, c)
+                    })?;
+            // Un MR desde un fork tiene otro proyecto de origen.
+            Ok(mrs
+                .into_iter()
+                .map(|m| PrLeido {
+                    propio: m.source_project_id == m.target_project_id,
+                    numero: m.iid,
+                    rama: m.source_branch,
+                    sha: m.sha.unwrap_or_default(),
+                    borrador: m.draft || m.work_in_progress,
+                })
+                .collect())
+        };
+        operaciones::prs_abiertos(rol, leer, || {
+            self.con.invalida_por(PRS_ABIERTOS, &t::RAMA_DESTINO_O_SHA)
+        })
     }
 
     fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe {
-        if !rama_valida(rama) || !rama_valida(base) {
-            return uso(ABRIR_PR, "rama-invalida", &t::RAMA_NO_VALIDA);
-        }
-        if titulo.trim().is_empty() || titulo.chars().any(char::is_control) {
-            return uso(ABRIR_PR, "titulo-invalido", &t::TITULO_NO_VALIDO);
-        }
-        match self.mr_abierto(rama, base) {
-            Ok(Some(n)) => {
-                return Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string());
-            }
-            Ok(None) => {}
-            Err(i) => return i,
-        }
-        // GitLab abre un MR sin commits: se comprueba antes para dar la
-        // misma linea que GitHub.
-        match self.tiene_commits(rama, base) {
-            Ok(true) => {}
-            Ok(false) => {
-                return Informe::nuevo(ABRIR_PR, Resultado::Rechazada)
-                    .con("motivo", "sin-commits")
-                    .diagnostico(Mensaje::nuevo(&t::SIN_COMMITS).con("cita", ""));
-            }
-            Err(i) => return i,
-        }
-        let datos = serde_json::json!({
-            "source_branch": rama, "target_branch": base,
-            "title": titulo, "description": cuerpo,
-        })
-        .to_string();
-        let r = match self.con.pedir(
-            ABRIR_PR,
-            Rol::Agente,
-            self.ep(&Endpoint::CrearMr),
-            Some(&datos),
-        ) {
-            Ok(r) => r,
-            Err(i) => return i,
-        };
-        if r.estado == 409 {
-            // Otro proceso lo abrio entre la lectura y la escritura.
-            return match self.mr_abierto(rama, base) {
-                Ok(Some(n)) => {
-                    Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string())
-                }
-                Ok(None) => Informe::nuevo(ABRIR_PR, Resultado::Conflicto)
-                    .con("motivo", "mr-existente")
-                    .diagnostico(
-                        Mensaje::nuevo(&t::MR_EXISTENTE).con("api", errores::mensaje_api(&r)),
-                    ),
-                Err(i) => i,
-            };
-        }
-        if !r.exito() {
-            return self
-                .con
-                .traducir(ABRIR_PR, Rol::Agente, PERMISO_ESCRITURA, &r);
-        }
-        match self.con.parsear::<MrApi>(ABRIR_PR, &r.cuerpo) {
-            Ok(m) => Informe::nuevo(ABRIR_PR, Resultado::Ok).con("pr", m.iid.to_string()),
-            Err(i) => i,
-        }
+        operaciones::abrir_pr(self, rama, base, titulo, cuerpo)
     }
 
     fn estado_puertas(&self, rol: Rol, pr: u64, modelo: &ModeloPuertas) -> Informe {
@@ -640,11 +614,14 @@ impl Plataforma for GitLab {
             Ok(m) => m,
             Err(i) => return i,
         };
-        if m.state == "merged" {
-            return puertas::mergeado(pr);
-        }
-        if m.state != "opened" {
-            return puertas::pr_cerrado(pr);
+        if let Err(i) = segun_estado(
+            op,
+            pr,
+            m.state == "merged",
+            m.state == "opened",
+            &t::PR_CERRADO_REABRE,
+        ) {
+            return i;
         }
         if !rama_valida(&m.target_branch) || !sha_valido(&m.sha) {
             return self.con.invalida_por(op, &t::RAMA_DESTINO_O_SHA);
@@ -774,10 +751,7 @@ impl Plataforma for GitLab {
     }
 
     fn merge_pr(&self, pr: u64, sha: &str) -> Informe {
-        if !sha_valido(sha) {
-            return uso(MERGE_PR, "sha-invalido", &t::SHA_ESPERADO_COMPLETO);
-        }
-        if let Err(i) = self.exigir_persona(MERGE_PR) {
+        if let Err(i) = entrada_merge(sha, || self.exigir_persona(MERGE_PR)) {
             return i;
         }
         let m: MrApi = match self.con.leer(
@@ -789,16 +763,14 @@ impl Plataforma for GitLab {
             Ok(m) => m,
             Err(i) => return i,
         };
-        if m.state == "merged" {
-            return Informe::nuevo(MERGE_PR, Resultado::SinCambios)
-                .con("pr", pr.to_string())
-                .con("estado", "mergeado");
-        }
-        if m.state != "opened" {
-            return Informe::nuevo(MERGE_PR, Resultado::Rechazada)
-                .con("pr", pr.to_string())
-                .con("motivo", "pr-cerrado")
-                .diagnostico(&t::MR_CERRADO);
+        if let Err(i) = segun_estado(
+            MERGE_PR,
+            pr,
+            m.state == "merged",
+            m.state == "opened",
+            &t::MR_CERRADO,
+        ) {
+            return i;
         }
         if !m
             .sha
@@ -834,10 +806,10 @@ impl Plataforma for GitLab {
                 );
         }
         if !r.exito() {
-            let mut i = self.con.traducir(MERGE_PR, Rol::Persona, PERMISO_MERGE, &r);
-            i.claves
-                .insert(0, ("pr".to_owned(), Valor::Texto(pr.to_string())));
-            return i;
+            return merge_fallido(
+                pr,
+                self.con.traducir(MERGE_PR, Rol::Persona, PERMISO_MERGE, &r),
+            );
         }
         match self.con.parsear::<MergeApi>(MERGE_PR, &r.cuerpo) {
             Ok(MergeApi {
@@ -856,9 +828,12 @@ impl Plataforma for GitLab {
     }
 
     fn leer_puertas(&self, rol: Rol, rama: &str) -> Paso<ReglasLeidas> {
-        validar_puertas(rol, rama)?;
-        let comun = self.puertas_comunes(rol)?;
-        Ok(self.puertas_de_rama(rol, rama, &comun, true))
+        operaciones::leer_puertas(
+            rol,
+            rama,
+            || self.puertas_comunes(rol),
+            |c, rama, checks| Ok(self.puertas_de_rama(rol, rama, c, checks)),
+        )
     }
 
     fn leer_puertas_de(
@@ -868,25 +843,14 @@ impl Plataforma for GitLab {
         checks_de: &str,
     ) -> Vec<Paso<ReglasLeidas>> {
         // Ramas protegidas, reglas de aprobacion y proyecto son del
-        // proyecto: se leen una vez (R1-06 de TASK-035).
-        let validas: Vec<Paso<()>> = ramas.iter().map(|r| validar_puertas(rol, r)).collect();
-        let comun = if validas.iter().any(Result::is_ok) {
-            Some(self.puertas_comunes(rol))
-        } else {
-            None
-        };
-        ramas
-            .iter()
-            .zip(validas)
-            .map(|(rama, v)| {
-                v?;
-                match comun.as_ref() {
-                    Some(Ok(c)) => Ok(self.puertas_de_rama(rol, rama, c, *rama == checks_de)),
-                    Some(Err(i)) => Err(i.clone()),
-                    None => Err(rol_no_valido(LEER_PUERTAS, rol)),
-                }
-            })
-            .collect()
+        // proyecto: valen para todas las ramas.
+        operaciones::leer_en_lote(
+            rol,
+            ramas,
+            checks_de,
+            || self.puertas_comunes(rol),
+            |c, rama, checks| Ok(self.puertas_de_rama(rol, rama, c, checks)),
+        )
     }
 
     fn leer_permisos_ci(&self) -> Informe {
@@ -1243,17 +1207,6 @@ impl GitLab {
     }
 }
 
-/// Rama y rol de `leer-puertas`.
-fn validar_puertas(rol: Rol, rama: &str) -> Paso<()> {
-    if !rama_valida(rama) {
-        return Err(uso(LEER_PUERTAS, "rama-invalida", &t::RAMA_NO_VALIDA));
-    }
-    if rol == Rol::Revisor {
-        return Err(rol_no_valido(LEER_PUERTAS, rol));
-    }
-    Ok(())
-}
-
 /// Lo que `leer-puertas` lee del proyecto, comun a todas las ramas.
 struct PuertasComunes {
     ramas: Vec<RamaProtegidaApi>,
diff --git a/crates/tc2/src/plataforma/http.rs b/crates/tc2/src/plataforma/http.rs
index eed2af5..0b2b56e 100644
--- a/crates/tc2/src/plataforma/http.rs
+++ b/crates/tc2/src/plataforma/http.rs
@@ -490,6 +490,9 @@ mod tests {
             "https://host:99999",
             "https://host:8a",
             "http://127.0.0.1:70000",
+            // Host que empieza por `.` o `-`.
+            "https://.evil.example",
+            "https://-x.example",
         ] {
             assert!(Base::nueva(b).is_err(), "{b}");
         }
diff --git a/crates/tc2/src/plataforma/mod.rs b/crates/tc2/src/plataforma/mod.rs
index 9e9d0d6..7c6d2c9 100644
--- a/crates/tc2/src/plataforma/mod.rs
+++ b/crates/tc2/src/plataforma/mod.rs
@@ -5,8 +5,9 @@
 //! API HTTP y sin `gh` (F1-14). Sus operaciones forman una lista cerrada y
 //! devuelven el contrato de salida de [`crate::contrato`]; cada una corre con
 //! la identidad que le da F1-6 y falla cerrada sin ella. Los dos adaptadores
-//! comparten el transporte ([`conexion`]) y la evaluacion ([`puertas`]), y
-//! dan la misma linea byte a byte (RR-15).
+//! comparten el transporte ([`conexion`]), el flujo neutral de cada
+//! operacion ([`operaciones`]) y la evaluacion ([`puertas`]), y dan la misma
+//! linea byte a byte (RR-15).
 //!
 //! Se usa desde la biblioteca: el binario no gana subcomandos hasta que los
 //! comandos de fase (F6-2 `review`, F7 `finish`, F3-8 `doctor`) lo llamen.
@@ -20,6 +21,7 @@ pub mod errores;
 pub mod github;
 pub mod gitlab;
 pub mod http;
+pub(crate) mod operaciones;
 pub mod personas;
 #[cfg(test)]
 pub(crate) mod prueba;
diff --git a/crates/tc2/src/plataforma/operaciones.rs b/crates/tc2/src/plataforma/operaciones.rs
new file mode 100644
index 0000000..828cefe
--- /dev/null
+++ b/crates/tc2/src/plataforma/operaciones.rs
@@ -0,0 +1,235 @@
+//! Flujo neutral de las operaciones del puerto (TASK-139, grupo A1 de la
+//! auditoria 4D): lo que no depende de la API de la plataforma (validar la
+//! entrada, la idempotencia, la carrera de «ya existe» y las lineas de
+//! resultado). Cada adaptador solo lee y escribe su API y traduce lo leido a
+//! los tipos de aqui (F1-13, D1a), y asi los dos dan la misma linea byte a
+//! byte (RR-15). El transporte es de [`super::conexion`] y la evaluacion de
+//! las puertas, de [`super::puertas`].
+
+use std::cell::OnceCell;
+
+use super::conexion::{rama_valida, rol_no_valido, sha_valido, uso};
+use super::credenciales::Rol;
+use super::{Paso, PrAbierto, ReglasLeidas, puertas};
+use crate::contrato::{Informe, Resultado, Valor};
+use crate::i18n::catalogo::plataforma as t;
+use crate::i18n::{Mensaje, Texto};
+
+/// Nombre de la operacion `abrir-pr`.
+pub(crate) const ABRIR_PR: &str = "abrir-pr";
+
+/// Nombre de la operacion `merge-pr`.
+pub(crate) const MERGE_PR: &str = "merge-pr";
+
+/// Nombre de la operacion `prs-abiertos`.
+pub(crate) const PRS_ABIERTOS: &str = "prs-abiertos";
+
+/// Nombre de la operacion `leer-puertas`.
+pub(crate) const LEER_PUERTAS: &str = "leer-puertas";
+
+/// Lo que dio la escritura que crea el PR.
+pub(crate) enum Creado {
+    /// Creado, con su numero.
+    Pr(u64),
+    /// La plataforma dice que ya existe (GitHub: 422 `already exists`;
+    /// GitLab: 409). Lleva el diagnostico de la plataforma por si al releer
+    /// no se encuentra abierto.
+    YaExiste(Mensaje),
+    /// Cualquier otro fallo, ya traducido.
+    Fallo(Informe),
+}
+
+/// Las lecturas y la escritura de `abrir-pr` de un adaptador.
+pub(crate) trait AbrePr {
+    /// El PR abierto de `rama` contra `base`, si lo hay.
+    fn abierto(&self, rama: &str, base: &str) -> Paso<Option<u64>>;
+
+    /// Comprobacion antes de crear (GitLab abre un MR sin commits: lo
+    /// comprueba aqui para dar la misma linea que GitHub). Por defecto,
+    /// ninguna.
+    fn antes_de_crear(&self, _rama: &str, _base: &str) -> Paso<()> {
+        Ok(())
+    }
+
+    /// Crea el PR de `rama` contra `base`.
+    fn crear(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Creado;
+}
+
+/// `abrir-pr`: valida la entrada, devuelve el PR que ya este abierto
+/// (`sin-cambios`) o lo crea. Si al crear la plataforma dice que ya existe
+/// (otro proceso lo abrio entre la lectura y la escritura), se relee: el
+/// abierto es `sin-cambios`; si sigue sin verse, `conflicto
+/// motivo=pr-existente`, la misma linea en las dos plataformas.
+pub(crate) fn abrir_pr(
+    a: &impl AbrePr,
+    rama: &str,
+    base: &str,
+    titulo: &str,
+    cuerpo: &str,
+) -> Informe {
+    if !rama_valida(rama) || !rama_valida(base) {
+        return uso(ABRIR_PR, "rama-invalida", &t::RAMA_NO_VALIDA);
+    }
+    if titulo.trim().is_empty() || titulo.chars().any(char::is_control) {
+        return uso(ABRIR_PR, "titulo-invalido", &t::TITULO_NO_VALIDO);
+    }
+    let sin_cambios =
+        |n: u64| Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string());
+    match a.abierto(rama, base) {
+        Ok(Some(n)) => return sin_cambios(n),
+        Ok(None) => {}
+        Err(i) => return i,
+    }
+    if let Err(i) = a.antes_de_crear(rama, base) {
+        return i;
+    }
+    match a.crear(rama, base, titulo, cuerpo) {
+        Creado::Pr(n) => Informe::nuevo(ABRIR_PR, Resultado::Ok).con("pr", n.to_string()),
+        Creado::YaExiste(diag) => match a.abierto(rama, base) {
+            Ok(Some(n)) => sin_cambios(n),
+            Ok(None) => Informe::nuevo(ABRIR_PR, Resultado::Conflicto)
+                .con("motivo", "pr-existente")
+                .diagnostico(diag),
+            Err(i) => i,
+        },
+        Creado::Fallo(i) => i,
+    }
+}
+
+/// Entrada de `merge-pr`: el sha completo (sin llamar si no vale) y despues
+/// la persona de la lista.
+pub(crate) fn entrada_merge(sha: &str, exigir_persona: impl FnOnce() -> Paso<()>) -> Paso<()> {
+    if !sha_valido(sha) {
+        return Err(uso(MERGE_PR, "sha-invalido", &t::SHA_ESPERADO_COMPLETO));
+    }
+    exigir_persona()
+}
+
+/// El estado del PR leido antes de evaluarlo (`estado-puertas`) o
+/// mergearlo (`merge-pr`): mergeado es `sin-cambios estado=mergeado`, y
+/// cerrado sin mergear, `rechazada motivo=pr-cerrado` con el diagnostico
+/// `cerrado`.
+pub(crate) fn segun_estado(
+    op: &str,
+    pr: u64,
+    mergeado: bool,
+    abierto: bool,
+    cerrado: &'static Texto,
+) -> Paso<()> {
+    if mergeado {
+        return Err(puertas::mergeado_en(op, pr));
+    }
+    if !abierto {
+        return Err(puertas::pr_cerrado_en(op, pr, cerrado));
+    }
+    Ok(())
+}
+
+/// Un PR (o MR) de la lista de abiertos, tal como lo lee un adaptador.
+pub(crate) struct PrLeido {
+    /// Numero del PR (en GitLab, el `iid` del MR).
+    pub(crate) numero: u64,
+    /// Rama de origen.
+    pub(crate) rama: String,
+    /// Sha de la cabeza, sin validar (vacio si la API no lo da).
+    pub(crate) sha: String,
+    /// Si es borrador.
+    pub(crate) borrador: bool,
+    /// Si la rama de origen es del propio repo (no de un fork).
+    pub(crate) propio: bool,
+}
+
+/// `prs-abiertos` (adenda de TASK-032 a F1-4 y F1-13): el revisor no lista
+/// PR; con lo que da `leer`, los de forks y las ramas que tc2 no habria
+/// creado (no casan con ninguna tarea) se saltan, y un sha que no es de 40
+/// hex es `invalida` (falla cerrado). El sha va en minusculas y la lista,
+/// por numero.
+pub(crate) fn prs_abiertos(
+    rol: Rol,
+    leer: impl FnOnce() -> Paso<Vec<PrLeido>>,
+    invalida: impl Fn() -> Informe,
+) -> Paso<Vec<PrAbierto>> {
+    if rol == Rol::Revisor {
+        return Err(rol_no_valido(PRS_ABIERTOS, rol));
+    }
+    let mut v = Vec::new();
+    for p in leer()? {
+        if !p.propio || !rama_valida(&p.rama) {
+            continue;
+        }
+        if !sha_valido(&p.sha) {
+            return Err(invalida());
+        }
+        v.push(PrAbierto {
+            numero: p.numero,
+            rama: p.rama,
+            sha: p.sha.to_ascii_lowercase(),
+            borrador: p.borrador,
+        });
+    }
+    v.sort_by_key(|p| p.numero);
+    Ok(v)
+}
+
+/// `leer-puertas` de una rama: valida la rama y el rol, lee lo comun al
+/// repo con `comun` y las reglas de la rama, con sus checks publicados,
+/// con `por_rama(comun, rama, con_checks)`.
+pub(crate) fn leer_puertas<C>(
+    rol: Rol,
+    rama: &str,
+    comun: impl Fn() -> Paso<C>,
+    por_rama: impl Fn(&C, &str, bool) -> Paso<ReglasLeidas>,
+) -> Paso<ReglasLeidas> {
+    leer_rama(rol, rama, true, &OnceCell::new(), &comun, &por_rama)
+}
+
+/// `leer-puertas` de varias `ramas` (R1-06 de TASK-035): lo comun al repo
+/// se lee una sola vez, al llegar a la primera rama valida (si ninguna lo
+/// es, no se lee), y los checks publicados solo en `checks_de`. Un
+/// resultado por rama, en el mismo orden.
+pub(crate) fn leer_en_lote<C>(
+    rol: Rol,
+    ramas: &[&str],
+    checks_de: &str,
+    comun: impl Fn() -> Paso<C>,
+    por_rama: impl Fn(&C, &str, bool) -> Paso<ReglasLeidas>,
+) -> Vec<Paso<ReglasLeidas>> {
+    let leido = OnceCell::new();
+    ramas
+        .iter()
+        .map(|rama| leer_rama(rol, rama, *rama == checks_de, &leido, &comun, &por_rama))
+        .collect()
+}
+
+fn leer_rama<C>(
+    rol: Rol,
+    rama: &str,
+    con_checks: bool,
+    leido: &OnceCell<Paso<C>>,
+    comun: &impl Fn() -> Paso<C>,
+    por_rama: &impl Fn(&C, &str, bool) -> Paso<ReglasLeidas>,
+) -> Paso<ReglasLeidas> {
+    validar_puertas(rol, rama)?;
+    match leido.get_or_init(comun) {
+        Ok(c) => por_rama(c, rama, con_checks),
+        Err(i) => Err(i.clone()),
+    }
+}
+
+/// Rama y rol de `leer-puertas` (agente o persona).
+fn validar_puertas(rol: Rol, rama: &str) -> Paso<()> {
+    if !rama_valida(rama) {
+        return Err(uso(LEER_PUERTAS, "rama-invalida", &t::RAMA_NO_VALIDA));
+    }
+    if rol == Rol::Revisor {
+        return Err(rol_no_valido(LEER_PUERTAS, rol));
+    }
+    Ok(())
+}
+
+/// Un fallo de la escritura del merge, con `pr=` delante.
+pub(crate) fn merge_fallido(pr: u64, mut i: Informe) -> Informe {
+    i.claves
+        .insert(0, ("pr".to_owned(), Valor::Texto(pr.to_string())));
+    i
+}
diff --git a/crates/tc2/src/plataforma/puertas.rs b/crates/tc2/src/plataforma/puertas.rs
index b16f2c0..93f6af4 100644
--- a/crates/tc2/src/plataforma/puertas.rs
+++ b/crates/tc2/src/plataforma/puertas.rs
@@ -119,17 +119,27 @@ pub fn evaluar(l: &Lecturas) -> Informe {
 
 /// PR ya mergeado: nada que evaluar.
 pub fn mergeado(pr: u64) -> Informe {
-    Informe::nuevo(ESTADO_PUERTAS, Resultado::SinCambios)
-        .con("pr", pr.to_string())
-        .con("estado", "mergeado")
+    mergeado_en(ESTADO_PUERTAS, pr)
 }
 
 /// PR cerrado sin mergear.
 pub fn pr_cerrado(pr: u64) -> Informe {
-    Informe::nuevo(ESTADO_PUERTAS, Resultado::Rechazada)
+    pr_cerrado_en(ESTADO_PUERTAS, pr, &t::PR_CERRADO_REABRE)
+}
+
+/// `op sin-cambios pr=N estado=mergeado` (`estado-puertas` y `merge-pr`).
+pub(crate) fn mergeado_en(op: &str, pr: u64) -> Informe {
+    Informe::nuevo(op, Resultado::SinCambios)
+        .con("pr", pr.to_string())
+        .con("estado", "mergeado")
+}
+
+/// `op rechazada pr=N motivo=pr-cerrado`, con el diagnostico `diag`.
+pub(crate) fn pr_cerrado_en(op: &str, pr: u64, diag: &'static Texto) -> Informe {
+    Informe::nuevo(op, Resultado::Rechazada)
         .con("pr", pr.to_string())
         .con("motivo", "pr-cerrado")
-        .diagnostico(&t::PR_CERRADO_REABRE)
+        .diagnostico(diag)
 }
 
 /// La politica exige impuesta una garantia que el plan no da.
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-139/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-139/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-139/tarea.md                                    | 56 +++++++++++++++++++++++++++++++++++++++++++++++---------
 3 files changed, 47 insertions(+), 9 deletions(-)
````

## Tests del diff (8 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/arnes_e2e/mod.rs" ":(literal)crates/tc2/tests/comun/adaptadores.rs" ":(literal)crates/tc2/tests/comun/mod.rs" ":(literal)crates/tc2/tests/plataforma.rs" ":(literal)crates/tc2/tests/plataforma_e2e.rs" ":(literal)crates/tc2/tests/plataforma_gitlab.rs" ":(literal)crates/tc2/tests/plataforma_gitlab_e2e.rs" ":(literal)crates/tc2/tests/prs_abiertos.rs"
````

````
crates/tc2/tests/arnes_e2e/mod.rs         |  79 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/comun/adaptadores.rs     |  92 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/comun/mod.rs             |   1 +
 crates/tc2/tests/plataforma.rs            | 148 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--------------------------------------------------------------------
 crates/tc2/tests/plataforma_e2e.rs        |  47 +++++------------------------------------------
 crates/tc2/tests/plataforma_gitlab.rs     | 121 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-----------------------------------------------------------
 crates/tc2/tests/plataforma_gitlab_e2e.rs |  79 ++++++++++++++-----------------------------------------------------------------
 crates/tc2/tests/prs_abiertos.rs          | 134 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-------------------------------------------------------------
 8 files changed, 406 insertions(+), 295 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

### Posible duplicacion con lo que ya existia

Definiciones nuevas con nombre igual o parecido a otra de la rama base. Mira si hacen lo mismo con pequenas diferencias (duplicacion real = IMPORTANTE).

- `dueno` (crates/tc2/src/plataforma/github.rs:379) ~ `dueno` (crates/tc2/src/ciclo/liberar.rs:105), `dueno` (crates/tc2/src/ciclo/operacion.rs:146), `dueno` (crates/tc2/src/ciclo/operacion.rs:198) y 7 mas
- `abierto` (crates/tc2/src/plataforma/github.rs:385) ~ `abierto` (crates/tc2/src/hallazgos/mod.rs:1334), `abierto` (crates/tc2/src/hallazgos/mod.rs:2342), `abiertos` (crates/tc2/src/hallazgos/orden.rs:716)
- `crear` (crates/tc2/src/plataforma/github.rs:402) ~ `crear` (crates/tc2/src/git/rama.rs:318), `crear` (crates/tc2/src/toma/mod.rs:357)
- `prs_abiertos` (crates/tc2/src/plataforma/github.rs:437) ~ `prs_abiertos` (crates/tc2/src/plataforma/gitlab.rs:528), `prs_abiertos` (crates/tc2/src/plataforma/mod.rs:420), `prs_abiertos` (crates/tc2/src/plataforma/prueba.rs:21) y 6 mas
- `abrir_pr` (crates/tc2/src/plataforma/github.rs:461) ~ `abrir_pr` (crates/tc2/src/plataforma/gitlab.rs:560), `abrir_pr` (crates/tc2/src/plataforma/mod.rs:424), `abrir_pr` (crates/tc2/src/plataforma/prueba.rs:24) y 4 mas
- `abierto` (crates/tc2/src/plataforma/gitlab.rs:472) ~ `abierto` (crates/tc2/src/hallazgos/mod.rs:1334), `abierto` (crates/tc2/src/hallazgos/mod.rs:2342), `abiertos` (crates/tc2/src/hallazgos/orden.rs:716)
- `crear` (crates/tc2/src/plataforma/gitlab.rs:503) ~ `crear` (crates/tc2/src/git/rama.rs:318), `crear` (crates/tc2/src/toma/mod.rs:357)
- `Creado` (crates/tc2/src/plataforma/operaciones.rs:31) ~ `Creada` (crates/tc2/src/plataforma/mod.rs:348)
- `abierto` (crates/tc2/src/plataforma/operaciones.rs:45) ~ `abierto` (crates/tc2/src/hallazgos/mod.rs:1334), `abierto` (crates/tc2/src/hallazgos/mod.rs:2342), `abiertos` (crates/tc2/src/hallazgos/orden.rs:716)
- `crear` (crates/tc2/src/plataforma/operaciones.rs:55) ~ `crear` (crates/tc2/src/git/rama.rs:318), `crear` (crates/tc2/src/toma/mod.rs:357)
- `abrir_pr` (crates/tc2/src/plataforma/operaciones.rs:63) ~ `abrir_pr` (crates/tc2/src/plataforma/github.rs:433), `abrir_pr` (crates/tc2/src/plataforma/gitlab.rs:560), `abrir_pr` (crates/tc2/src/plataforma/mod.rs:424) y 5 mas
- `prs_abiertos` (crates/tc2/src/plataforma/operaciones.rs:147) ~ `prs_abiertos` (crates/tc2/src/plataforma/github.rs:402), `prs_abiertos` (crates/tc2/src/plataforma/gitlab.rs:528), `prs_abiertos` (crates/tc2/src/plataforma/mod.rs:420) y 7 mas
- `leer_puertas` (crates/tc2/src/plataforma/operaciones.rs:177) ~ `leer_puertas` (crates/tc2/src/plataforma/github.rs:734), `leer_puertas` (crates/tc2/src/plataforma/gitlab.rs:858), `leer_puertas` (crates/tc2/src/plataforma/mod.rs:467) y 7 mas
- `validar_puertas` (crates/tc2/src/plataforma/operaciones.rs:220) ~ `validar_puertas` (crates/tc2/src/plataforma/github.rs:940), `validar_puertas` (crates/tc2/src/plataforma/gitlab.rs:1247)

### Posible duplicacion dentro de la tarea

Definiciones nuevas parecidas entre si: la misma tarea puede estar escribiendo dos veces lo mismo.

- `abierto` (crates/tc2/src/plataforma/github.rs:385) ~ `abierto` (crates/tc2/src/plataforma/gitlab.rs:472)
- `abierto` (crates/tc2/src/plataforma/github.rs:385) ~ `abierto` (crates/tc2/src/plataforma/operaciones.rs:45)
- `crear` (crates/tc2/src/plataforma/github.rs:402) ~ `crear` (crates/tc2/src/plataforma/gitlab.rs:503)
- `crear` (crates/tc2/src/plataforma/github.rs:402) ~ `crear` (crates/tc2/src/plataforma/operaciones.rs:55)
- `prs_abiertos` (crates/tc2/src/plataforma/github.rs:437) ~ `prs_abiertos` (crates/tc2/src/plataforma/operaciones.rs:147)
- `abrir_pr` (crates/tc2/src/plataforma/github.rs:461) ~ `abrir_pr` (crates/tc2/src/plataforma/operaciones.rs:63)
- `abierto` (crates/tc2/src/plataforma/gitlab.rs:472) ~ `abierto` (crates/tc2/src/plataforma/operaciones.rs:45)
- `antes_de_crear` (crates/tc2/src/plataforma/gitlab.rs:487) ~ `antes_de_crear` (crates/tc2/src/plataforma/operaciones.rs:50)
- `crear` (crates/tc2/src/plataforma/gitlab.rs:503) ~ `crear` (crates/tc2/src/plataforma/operaciones.rs:55)

