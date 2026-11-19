# Peticion de revision — TASK-118 (ronda 1)

- Tarea: TASK-118 — Aplicar las puertas (rulesets) desde tc2 init
- Rama revisada: feature/task-118-aplicar-las-puertas-rulesets-desde-tc2-i
- Rama base: develop
- Commit revisado (HEAD): 34a6b7b21259c24709fa626eb1939edb7a053ecc
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-118 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-118/tarea.md
- tareas/03-en-revision/TASK-118/planificacion/
- crates/tc2/src/config/claves.rs
- crates/tc2/src/config/mod.rs
- crates/tc2/src/i18n/catalogo/init.rs
- crates/tc2/src/i18n/catalogo/plataforma.rs
- crates/tc2/src/init/aplicar.rs
- crates/tc2/src/init/inventario.rs
- crates/tc2/src/init/mod.rs
- crates/tc2/src/plataforma/github.rs
- crates/tc2/src/plataforma/gitlab.rs
- crates/tc2/src/plataforma/mod.rs
- crates/tc2/src/plataforma/puertas.rs
- crates/tc2/tests/init.rs
- crates/tc2/tests/plataforma.rs
- crates/tc2/tests/plataforma_gitlab.rs

## Commits a revisar (git log develop..HEAD)

````
34a6b7b chore(TASK-118): autorrevision
a7f212e feat(TASK-118): adenda de aplicar-puertas en F1-5 y F1-13
ce027ac feat(TASK-118): aplicar las puertas desde tc2 init --aplicar-puertas
eef310d chore(TASK-118): tarea en curso
````

## Diff de tu dominio (14 fichero(s) de 17; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/config/claves.rs b/crates/tc2/src/config/claves.rs
index 86d0ff5..927f582 100644
--- a/crates/tc2/src/config/claves.rs
+++ b/crates/tc2/src/config/claves.rs
@@ -441,6 +441,11 @@ pub(crate) fn tabla() -> Vec<Campo> {
                     "plataforma del remoto",
                 ),
                 Campo::obligatorio("api", texto(Regla::Url), "URL de la API"),
+                Campo::opcional(
+                    "revisor_id",
+                    Nodo::Entero { min: 0, max: None },
+                    "origen de revision-ia: el app_id de la App del revisor en GitHub o su user.id en GitLab; 0 o ausente, no hay y tc2 init --aplicar-puertas se rechaza (TASK-118)",
+                ),
             ]),
             "plataforma de un host que no es github.com ni gitlab.com",
         )
diff --git a/crates/tc2/src/config/mod.rs b/crates/tc2/src/config/mod.rs
index 3013fe9..f4bcd89 100644
--- a/crates/tc2/src/config/mod.rs
+++ b/crates/tc2/src/config/mod.rs
@@ -195,6 +195,10 @@ pub struct PlataformaConfig {
     pub proveedor: Proveedor,
     /// URL de la API.
     pub api: String,
+    /// Origen de `revision-ia` (TASK-118): el `app_id` de la App del revisor
+    /// (GitHub) o su `user.id` (GitLab). 0 o ausente: no hay.
+    #[serde(default)]
+    pub revisor_id: Option<u64>,
 }
 
 fn de_proveedor<'de, D: Deserializer<'de>>(d: D) -> Result<Proveedor, D::Error> {
diff --git a/crates/tc2/src/i18n/catalogo/init.rs b/crates/tc2/src/i18n/catalogo/init.rs
index dcddac2..64f9f92 100644
--- a/crates/tc2/src/i18n/catalogo/init.rs
+++ b/crates/tc2/src/i18n/catalogo/init.rs
@@ -2,8 +2,28 @@
 
 crate::i18n::textos! {
     USO_INIT {
-        es: "uso: init [--simular] [--json]",
-        en: "usage: init [--simular] [--json]",
+        es: "uso: init [--simular] [--aplicar-puertas] [--json]",
+        en: "usage: init [--simular] [--aplicar-puertas] [--json]",
+    }
+    PUERTAS_SIN_PLATAFORMA {
+        es: "--aplicar-puertas necesita hablar con la plataforma como persona y no puede ({motivo}). No se ha tocado nada: resuelvelo (remoto origin, TC2_TOKEN_PERSONA en un terminal, la clave plataforma) y repite tc2 init --aplicar-puertas",
+        en: "--aplicar-puertas needs to talk to the platform as the person and cannot ({motivo}). Nothing was touched: fix it (origin remote, TC2_TOKEN_PERSONA in a terminal, the plataforma key) and run tc2 init --aplicar-puertas again",
+    }
+    PUERTAS_SIN_PERSONAS {
+        es: "--aplicar-puertas exige que {ruta} ya exista y este versionado con tu cuenta. No se ha tocado nada: ejecuta tc2 init sin la opcion, versiona {ruta} y repite tc2 init --aplicar-puertas",
+        en: "--aplicar-puertas requires {ruta} to exist already and be committed with your account. Nothing was touched: run tc2 init without the option, commit {ruta} and run tc2 init --aplicar-puertas again",
+    }
+    PUERTAS_RAMAS_SIN_REMOTO {
+        es: "--aplicar-puertas exige que {ramas} ya esten en origin: con las puertas puestas ya no se podrian subir con un push. No se ha tocado nada: publicalas (git push origin develop main) y repite tc2 init --aplicar-puertas",
+        en: "--aplicar-puertas requires {ramas} to be on origin already: once the gates are in place they could no longer be pushed. Nothing was touched: publish them (git push origin develop main) and run tc2 init --aplicar-puertas again",
+    }
+    PUERTAS_RAMAS_POR_DELANTE {
+        es: "{ramas} tienen en este clon commits que origin no tiene (o falta traerlos): con las puertas puestas ese push se rechazaria. No se ha tocado nada: ejecuta git fetch origin, sube lo que falte y repite tc2 init --aplicar-puertas",
+        en: "{ramas} have commits in this clone that origin does not have (or they still need fetching): once the gates are in place that push would be rejected. Nothing was touched: run git fetch origin, push what is missing and run tc2 init --aplicar-puertas again",
+    }
+    PUERTAS_SIN_REVISOR_ID {
+        es: "los checks {faltan} no tienen origen: sin el, cualquiera con permiso de escribir checks los daria por buenos. No se ha tocado nada: anade plataforma.revisor_id en {ruta} (el app_id de la App del revisor en GitHub o su user.id en GitLab) y repite tc2 init --aplicar-puertas",
+        en: "the checks {faltan} have no origin: without it, anyone allowed to write checks could pass them. Nothing was touched: add plataforma.revisor_id to {ruta} (the app_id of the reviewer App on GitHub or its user.id on GitLab) and run tc2 init --aplicar-puertas again",
     }
     PROYECTO_016 {
         es: "{ruta} es de TaskCode 0.16: init no lo pisa. No se ha tocado nada; para pasar el proyecto a la 2.0: {arreglo}",
diff --git a/crates/tc2/src/i18n/catalogo/plataforma.rs b/crates/tc2/src/i18n/catalogo/plataforma.rs
index 81720e8..142aa83 100644
--- a/crates/tc2/src/i18n/catalogo/plataforma.rs
+++ b/crates/tc2/src/i18n/catalogo/plataforma.rs
@@ -290,4 +290,14 @@ crate::i18n::textos! {
         es: "caracter no permitido en la consulta",
         en: "character not allowed in the query",
     }
+
+    // --- aplicar-puertas (TASK-118) ---
+    APLICAR_NO_SOPORTADA {
+        es: "esta plataforma no admite aplicar las puertas desde tc2: aplicalas a mano con la propuesta de tc2 init --json",
+        en: "this platform does not support applying the gates from tc2: apply them by hand with the proposal from tc2 init --json",
+    }
+    CHECKS_SIN_ORIGEN_VISTO {
+        es: "no se han visto publicados por su origen en la cabeza de develop los checks {faltan}: exigirlos ahora bloquearia develop para todos. No se ha escrito nada en la plataforma; mergea en develop un PR en el que la CI y el revisor publiquen esos checks y repite tc2 init --aplicar-puertas",
+        en: "the checks {faltan} have not been seen published by their origin on the head of develop: requiring them now would block develop for everyone. Nothing was written on the platform; merge into develop a PR where the CI and the reviewer publish those checks and run tc2 init --aplicar-puertas again",
+    }
 }
diff --git a/crates/tc2/src/init/aplicar.rs b/crates/tc2/src/init/aplicar.rs
index f922147..c0ef56e 100644
--- a/crates/tc2/src/init/aplicar.rs
+++ b/crates/tc2/src/init/aplicar.rs
@@ -1,10 +1,12 @@
 //! Aplicacion de las acciones `crea` del plan de `tc2 init`. Cada efecto
 //! vuelve a comprobar lo que lee: si algo cambio desde el plan, falla sin
-//! pisarlo.
+//! pisarlo (las puertas, con `aplicar-puertas`, igual: solo crean lo que
+//! falta en ese momento).
 
 use std::io::Write;
 use std::path::Path;
 
+use super::inventario::Conexion;
 use super::{Contexto, Efecto, NOMBRE};
 use crate::contrato::{Informe, Resultado};
 use crate::git::estado::construir_commit;
@@ -14,6 +16,7 @@ use crate::huella::LINEAS_EXCLUDE;
 use crate::i18n::Idioma;
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::init as txt;
+use crate::plataforma::puertas::{APLICAR_PUERTAS, Origenes, modelo};
 
 /// Mensaje del commit raiz de `main` y `develop`.
 const MENSAJE_RAIZ: &str = "chore: inicia TaskCodeTwo (tc2 init)";
@@ -50,6 +53,17 @@ pub(super) fn aplicar(e: &Efecto, c: &Contexto<'_>) -> Result<(), String> {
                 Err(texto(&i))
             }
         }
+        // Necesita la plataforma: lo aplica [`puertas`] desde `init`.
+        Efecto::AplicarPuertas { .. } => Err("aplicar-puertas sin plataforma".to_owned()),
+    }
+}
+
+/// `aplicar-puertas` con el adaptador del plan: el adaptador vuelve a
+/// comprobar todo antes de escribir y solo crea lo que falta.
+pub(super) fn puertas(conexion: &Conexion, origenes: &Origenes) -> Informe {
+    match conexion {
+        Ok((p, _)) => p.aplicar_puertas(&modelo(), origenes),
+        Err(m) => Informe::nuevo(APLICAR_PUERTAS, Resultado::Rechazada).con("motivo", m.clone()),
     }
 }
 
diff --git a/crates/tc2/src/init/inventario.rs b/crates/tc2/src/init/inventario.rs
index 61c3fdf..944c6e7 100644
--- a/crates/tc2/src/init/inventario.rs
+++ b/crates/tc2/src/init/inventario.rs
@@ -18,12 +18,12 @@ use crate::i18n::Mensaje;
 use crate::i18n::catalogo::init as txt;
 use crate::plataforma::personas::{self, Personas};
 use crate::plataforma::puertas::{modelo, ruleset_github};
-use crate::plataforma::{GitHub, GitLab, Plataforma, Proveedor, Rol, detectar};
+use crate::plataforma::{EstadoPuerta, GitHub, GitLab, Plataforma, Proveedor, Rol, detectar};
 
 /// El adaptador de la plataforma y su proveedor, o el motivo por el que no
 /// lo hay (`sin-remoto`, `sin-token` o el motivo de `detectar`, como
 /// `plataforma-contradice`).
-type Conexion = Result<(Box<dyn Plataforma>, Proveedor), String>;
+pub(super) type Conexion = Result<(Box<dyn Plataforma>, Proveedor), String>;
 
 /// Ramas de codigo que `init` deja (F1-3).
 const RAMAS: [&str; 2] = ["main", "develop"];
@@ -72,7 +72,7 @@ fn denegada(f: FalloRemoto, detalle: &str) -> Informe {
         )
 }
 
-pub(super) fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
+pub(super) fn plan(c: &Contexto<'_>, aplicar: bool) -> Result<(Vec<Accion>, Conexion), Informe> {
     let g = c.git;
     let principal = localizar_principal(g, c.dir).map_err(|i| renombrar(i, NOMBRE))?;
     let mut v = Vec::new();
@@ -80,6 +80,8 @@ pub(super) fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
     // Configuracion: la 0.16 o una que no vale no se pisan (C4).
     let ruta_config = principal.join(config::RUTA);
     let mut clave: Option<(&'static str, String)> = None;
+    // `plataforma.revisor_id` (TASK-118): el origen de `revision-ia`.
+    let mut revisor: Option<u64> = None;
     match leer(&ruta_config)? {
         None => v.push(Accion::crea(
             t(&ruta_config),
@@ -91,6 +93,7 @@ pub(super) fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
         )),
         Some(bytes) => match config::leer(&String::from_utf8_lossy(&bytes)) {
             Ok(cfg) => {
+                revisor = cfg.plataforma.as_ref().and_then(|p| p.revisor_id);
                 clave = cfg.plataforma.map(|p| (p.proveedor.nombre(), p.api));
                 v.push(Accion::nueva(
                     t(&ruta_config),
@@ -234,9 +237,184 @@ pub(super) fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
     // `init` nunca lee ni escribe un `settings.json`.
     v.push(status_line(&principal));
 
-    // Rulesets: solo propuesta (D1); los aplica TASK-118.
-    puertas(&plataforma, &mut v);
-    Ok(v)
+    // Rulesets: solo propuesta (D1); con `--aplicar-puertas`, los que
+    // faltan se crean (TASK-118), tras comprobar todo lo que exigen.
+    if aplicar {
+        let previo = Previo {
+            principal: &principal,
+            remotas: &remotas,
+            revisor,
+        };
+        puertas_a_aplicar(c, &previo, &plataforma, &mut v)?;
+    } else {
+        puertas(&plataforma, &mut v);
+    }
+    Ok((v, plataforma))
+}
+
+/// Lo que `--aplicar-puertas` comprueba antes de leer la plataforma.
+struct Previo<'a> {
+    principal: &'a Path,
+    /// Ramas de `origin`, `(nombre, sha)`.
+    remotas: &'a [(String, String)],
+    /// `plataforma.revisor_id`.
+    revisor: Option<u64>,
+}
+
+/// Motivo de la linea `init rechazada` cuando lo que bloquea es la persona.
+const SIN_PERSONA: &str = "sin-persona";
+
+/// Las lineas `ruleset` con `--aplicar-puertas` (TASK-118). Antes de
+/// escribir nada, ni en el disco ni en la plataforma, se exige: la
+/// plataforma con el token de la persona, `personas.yml` ya existente,
+/// `develop` y `main` en `origin` sin commits locales por delante (D5), el
+/// origen de cada check (D-id) y, en la plataforma, la persona de la lista
+/// y cada check ya visto publicado por su origen en `develop` (D2). Si algo
+/// falla, las cinco lineas salen `rechaza` con el motivo (D1: se rechaza la
+/// orden entera). Si pasa: `crea` las que faltan (con la propuesta en
+/// JSON), `existe` las que ya estan y `existe motivo=difiere` las que estan
+/// distintas, que no se tocan (D3/D4).
+fn puertas_a_aplicar(
+    c: &Contexto<'_>,
+    previo: &Previo<'_>,
+    plataforma: &Conexion,
+    v: &mut Vec<Accion>,
+) -> Result<(), Informe> {
+    let modelo = modelo();
+    let lineas = |decision: Decision| -> Vec<Accion> {
+        modelo
+            .iter()
+            .map(|p| Accion::nueva(p.nombre_completo(), "ruleset", "taskcodetwo", decision))
+            .collect()
+    };
+    // Rechazo de las cinco lineas; la primera lleva el diagnostico.
+    let rechazo = |motivo: String, faltan: Vec<String>, diag: Mensaje| -> Vec<Accion> {
+        let mut ls = lineas(Decision::Rechaza);
+        for a in &mut ls {
+            a.motivo = Some(motivo.clone());
+        }
+        if let Some(a) = ls.first_mut() {
+            a.faltan = faltan;
+            a.diagnostico = Some(diag);
+        }
+        ls
+    };
+    let (p, prov) = match plataforma {
+        Ok((p, prov)) => (p, *prov),
+        Err(m) => {
+            v.extend(rechazo(
+                m.clone(),
+                Vec::new(),
+                Mensaje::nuevo(&txt::PUERTAS_SIN_PLATAFORMA).con("motivo", m.clone()),
+            ));
+            return Ok(());
+        }
+    };
+    let personas_existe = v
+        .iter()
+        .any(|a| a.clase == "personas" && a.decision == Decision::Existe);
+    if !personas_existe {
+        v.extend(rechazo(
+            "sin-personas".to_owned(),
+            Vec::new(),
+            Mensaje::nuevo(&txt::PUERTAS_SIN_PERSONAS).con("ruta", personas::RUTA),
+        ));
+        return Ok(());
+    }
+    // D5: `develop` y `main` en `origin`, y nada local por delante.
+    let mut sin_remoto = Vec::new();
+    let mut por_delante = Vec::new();
+    for r in RAMAS {
+        let Some((_, remota)) = previo.remotas.iter().find(|(n, _)| n == r) else {
+            sin_remoto.push(r.to_owned());
+            continue;
+        };
+        if let Some(local) = commit_de(c.git, previo.principal, NOMBRE, &format!("refs/heads/{r}"))?
+            && !local.eq_ignore_ascii_case(remota)
+        {
+            let s = correr(
+                c.git,
+                previo.principal,
+                NOMBRE,
+                Subcomando::MergeBase,
+                &["--is-ancestor", &local, remota],
+            )?;
+            if !s.exito() {
+                por_delante.push(r.to_owned());
+            }
+        }
+    }
+    if !sin_remoto.is_empty() {
+        let diag =
+            Mensaje::nuevo(&txt::PUERTAS_RAMAS_SIN_REMOTO).con("ramas", sin_remoto.join(", "));
+        v.extend(rechazo("ramas-sin-remoto".to_owned(), sin_remoto, diag));
+        return Ok(());
+    }
+    if !por_delante.is_empty() {
+        let diag =
+            Mensaje::nuevo(&txt::PUERTAS_RAMAS_POR_DELANTE).con("ramas", por_delante.join(", "));
+        v.extend(rechazo("ramas-por-delante".to_owned(), por_delante, diag));
+        return Ok(());
+    }
+    // D-id: el origen de cada check, de la configuracion.
+    let origenes = match crate::plataforma::puertas::origenes(prov, previo.revisor) {
+        Ok(o) => o,
+        Err(faltan) => {
+            let faltan: Vec<String> = faltan.into_iter().map(str::to_owned).collect();
+            let diag = Mensaje::nuevo(&txt::PUERTAS_SIN_REVISOR_ID)
+                .con("faltan", faltan.join(", "))
+                .con("ruta", config::RUTA);
+            v.extend(rechazo("puertas-sin-origen".to_owned(), faltan, diag));
+            return Ok(());
+        }
+    };
+    // En la plataforma: la persona de la lista, el plan y D2.
+    let leidas = match p.preparar_puertas(&modelo, &origenes) {
+        Ok(l) => l,
+        Err(i) => {
+            let motivo = match motivo_de(&i).as_str() {
+                "identidad" => SIN_PERSONA.to_owned(),
+                m => m.to_owned(),
+            };
+            let faltan = super::lista(&i, "faltan");
+            let diag = i.diagnostico.clone().unwrap_or_else(|| {
+                Mensaje::nuevo(&txt::PUERTAS_SIN_PLATAFORMA).con("motivo", motivo.clone())
+            });
+            v.extend(rechazo(motivo, faltan, diag));
+            return Ok(());
+        }
+    };
+    let mut efecto = Some(Efecto::AplicarPuertas {
+        origenes: origenes.clone(),
+        notas: leidas.notas.clone(),
+    });
+    for puerta in &modelo {
+        let nombre = puerta.nombre_completo();
+        let estado = leidas
+            .puertas
+            .iter()
+            .find(|(n, _)| *n == nombre)
+            .map_or(EstadoPuerta::Falta, |(_, e)| *e);
+        let nueva = |d: Decision| Accion::nueva(nombre.clone(), "ruleset", "taskcodetwo", d);
+        v.push(match estado {
+            EstadoPuerta::Existe => nueva(Decision::Existe),
+            EstadoPuerta::Difiere => nueva(Decision::Existe).motivo("difiere"),
+            EstadoPuerta::Falta => {
+                // El efecto en lote va en la primera que se crea; las demas
+                // las hace esa misma linea.
+                let mut a = Accion::crea(
+                    nombre.clone(),
+                    "ruleset",
+                    efecto.take().unwrap_or(Efecto::Ninguno),
+                );
+                if prov == Proveedor::GitHub {
+                    a.propuesta = Some(ruleset_github(puerta, Some(&origenes)));
+                }
+                a
+            }
+        });
+    }
+    Ok(())
 }
 
 /// Donde se recomienda pegar la status line: el `settings.local.json` del
@@ -586,21 +764,17 @@ fn plataforma(
     if credenciales.token(Rol::Persona).is_none() {
         return Ok(Err("sin-token".to_owned()));
     }
+    // La lista de personas, si ya existe y vale: `aplicar-puertas` exige
+    // que la identidad de persona este en ella (TASK-118). Sin ella, vacia
+    // (las lecturas de `init` no la miran).
+    let lista = leer(&principal.join(personas::RUTA))?
+        .and_then(|b| Personas::leer(&String::from_utf8_lossy(&b)).ok())
+        .unwrap_or_default();
     let p: Result<Box<dyn Plataforma>, Informe> = match destino.proveedor {
-        Proveedor::GitHub => GitHub::nuevo(
-            destino.api,
-            &destino.proyecto,
-            credenciales,
-            Personas::default(),
-        )
-        .map(|x| Box::new(x) as Box<dyn Plataforma>),
-        Proveedor::GitLab => GitLab::nuevo(
-            destino.api,
-            &destino.proyecto,
-            credenciales,
-            Personas::default(),
-        )
-        .map(|x| Box::new(x) as Box<dyn Plataforma>),
+        Proveedor::GitHub => GitHub::nuevo(destino.api, &destino.proyecto, credenciales, lista)
+            .map(|x| Box::new(x) as Box<dyn Plataforma>),
+        Proveedor::GitLab => GitLab::nuevo(destino.api, &destino.proyecto, credenciales, lista)
+            .map(|x| Box::new(x) as Box<dyn Plataforma>),
     };
     Ok(p.map(|p| (p, destino.proveedor)).map_err(|i| motivo_de(&i)))
 }
@@ -687,7 +861,7 @@ fn puertas(plataforma: &Conexion, v: &mut Vec<Accion>) {
                     .motivo("aplicar-con-task-118")
                     .orden(ORDEN_PUERTAS);
                 if *prov == Proveedor::GitHub {
-                    a.propuesta = Some(ruleset_github(&puerta));
+                    a.propuesta = Some(ruleset_github(&puerta, None));
                 }
                 a
             }
diff --git a/crates/tc2/src/init/mod.rs b/crates/tc2/src/init/mod.rs
index a733980..1177e40 100644
--- a/crates/tc2/src/init/mod.rs
+++ b/crates/tc2/src/init/mod.rs
@@ -11,7 +11,14 @@
 //! (solo en un repo sin commits; con historia lo hace la persona), la rama
 //! de estado montada en `tareas/` y `personas.yml` con la persona del token.
 //! El workflow de CI queda como ranura `pendiente` hasta F5-1 y los rulesets
-//! `taskcodetwo-*` solo se proponen (`persona`): aplicarlos es de TASK-118.
+//! `taskcodetwo-*` solo se proponen (`persona`); con `--aplicar-puertas`
+//! (TASK-118) los que faltan se crean en la plataforma (`crea`), y lo que
+//! ya esta no se toca. La opcion exige antes de escribir nada, ni en el
+//! disco ni en la plataforma: el token y la lista de la persona
+//! (`personas.yml` ya existente), `develop` y `main` en `origin` sin nada
+//! local por delante, el origen de cada check (`plataforma.revisor_id`) y
+//! que cada check ya se haya visto publicado por su origen en `develop`; si
+//! algo falta, `init rechazada motivo=... [faltan=...]`.
 //! La status line de Claude Code (F3-12) tambien se propone, como
 //! `pendiente motivo=pegar-a-mano` con la orden exacta: `init` nunca lee ni
 //! escribe un `settings.json`.
@@ -29,16 +36,17 @@ mod inventario;
 use std::path::{Path, PathBuf};
 
 use crate::Salida;
-use crate::contrato::{Informe, LineaAccion, Resultado};
+use crate::contrato::{Informe, LineaAccion, Resultado, Valor};
 use crate::git::Git;
 use crate::i18n::catalogo::init as txt;
 use crate::i18n::{Idioma, Mensaje};
 use crate::plataforma::Credenciales;
+use crate::plataforma::puertas::Origenes;
 
 /// Nombre de la orden.
 pub const NOMBRE: &str = "init";
 
-/// Orden que aplicara las puertas (TASK-118).
+/// Orden que aplica las puertas (TASK-118).
 pub const ORDEN_PUERTAS: &str = "tc2 init --aplicar-puertas";
 
 /// Arreglo para un proyecto de la 0.16.
@@ -49,6 +57,9 @@ pub const ARREGLO_016: &str = "tc2 actualizar --simular";
 pub struct Opciones {
     /// Solo imprimir el plan.
     pub simular: bool,
+    /// Crear en la plataforma los rulesets `taskcodetwo-*` que falten
+    /// (TASK-118).
+    pub aplicar_puertas: bool,
 }
 
 /// Lee las opciones (sin `--json`).
@@ -57,6 +68,7 @@ pub fn leer_opciones(args: &[String]) -> Result<Opciones, Informe> {
     for a in args {
         match a.as_str() {
             "--simular" if !op.simular => op.simular = true,
+            "--aplicar-puertas" if !op.aplicar_puertas => op.aplicar_puertas = true,
             _ => {
                 return Err(Informe::nuevo(NOMBRE, Resultado::Uso)
                     .con("motivo", "argumentos")
@@ -137,6 +149,14 @@ pub(crate) enum Efecto {
         /// Worktree principal.
         principal: PathBuf,
     },
+    /// `aplicar-puertas` en la plataforma (TASK-118): todas las puertas que
+    /// falten, en lote; va en la primera linea `ruleset` que se crea.
+    AplicarPuertas {
+        /// El origen de cada check.
+        origenes: Origenes,
+        /// Claves que se anaden a la linea final (`sin-aprobaciones=si`).
+        notas: Vec<(&'static str, String)>,
+    },
 }
 
 /// Una linea del plan.
@@ -158,6 +178,8 @@ pub struct Accion {
     pub propuesta: Option<serde_json::Value>,
     /// Que decir a la persona si bloquea.
     pub(crate) diagnostico: Option<Mensaje>,
+    /// Si bloquea, lo que falta (`faltan=` de la linea final).
+    pub(crate) faltan: Vec<String>,
     /// Lo que se hace al aplicar.
     pub(crate) efecto: Efecto,
 }
@@ -178,6 +200,7 @@ impl Accion {
             orden: None,
             propuesta: None,
             diagnostico: None,
+            faltan: Vec::new(),
             efecto: Efecto::Ninguno,
         }
     }
@@ -345,31 +368,58 @@ pub(crate) fn leer(p: &Path) -> Result<Option<Vec<u8>>, Informe> {
     }
 }
 
-/// El plan, sin aplicar nada.
+/// El plan, sin aplicar nada (sin `--aplicar-puertas`).
 pub fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
-    inventario::plan(c)
+    inventario::plan(c, false).map(|(v, _)| v)
+}
+
+/// Valores de la clave de lista `k` de un informe.
+fn lista(i: &Informe, k: &str) -> Vec<String> {
+    i.claves
+        .iter()
+        .find_map(|(x, v)| match v {
+            Valor::Lista(l) if x == k => Some(l.clone()),
+            _ => None,
+        })
+        .unwrap_or_default()
 }
 
 /// `init` con las opciones ya leidas.
 pub fn init(op: Opciones, c: &Contexto<'_>) -> Hecho {
-    let mut acciones = match plan(c) {
+    let (mut acciones, conexion) = match inventario::plan(c, op.aplicar_puertas) {
         Ok(v) => v,
         Err(i) => return Hecho::solo(i),
     };
-    let cuenta = |r: Resultado, acciones: &[Accion], cabeza: &[(&str, String)]| {
+    // Claves de las puertas para la linea final (`sin-aprobaciones=si`).
+    let notas: Vec<(&'static str, String)> = acciones
+        .iter()
+        .find_map(|a| match &a.efecto {
+            Efecto::AplicarPuertas { notas, .. } => Some(notas.clone()),
+            _ => None,
+        })
+        .unwrap_or_default();
+    let cuenta = |r: Resultado, acciones: &[Accion], cabeza: &[(&str, Valor)]| {
         let n = |d: Decision| acciones.iter().filter(|a| a.decision == d).count();
         let mut i = Informe::nuevo(NOMBRE, r);
         for (k, v) in cabeza {
-            i = i.con(k, v.clone());
+            i.claves.push(((*k).to_owned(), v.clone()));
         }
         if op.simular {
             i = i.con("simular", "si");
         }
-        i.con("crea", n(Decision::Crea).to_string())
+        let mut i = i
+            .con("crea", n(Decision::Crea).to_string())
             .con("existe", n(Decision::Existe).to_string())
             .con("persona", n(Decision::Persona).to_string())
-            .con("pendiente", n(Decision::Pendiente).to_string())
+            .con("pendiente", n(Decision::Pendiente).to_string());
+        if r.es_exito() {
+            for (k, v) in &notas {
+                i = i.con(k, v.clone());
+            }
+        }
+        i
     };
+    let texto = |s: &str| Valor::Texto(s.to_owned());
     if let Some(b) = acciones
         .iter()
         .find(|a| a.decision == Decision::Rechaza)
@@ -381,11 +431,14 @@ pub fn init(op: Opciones, c: &Contexto<'_>) -> Hecho {
             a.motivo = Some("no-aplicada".to_owned());
         }
         let motivo = b.motivo.clone().unwrap_or_default();
-        let mut cabeza = vec![("motivo", motivo.clone())];
+        let mut cabeza = vec![("motivo", texto(&motivo))];
         if let Some(o) = &b.orden {
-            cabeza.push(("arreglo", o.clone()));
+            cabeza.push(("arreglo", texto(o)));
         }
-        cabeza.push(("ruta", b.ruta.clone()));
+        if !b.faltan.is_empty() {
+            cabeza.push(("faltan", Valor::Lista(b.faltan.clone())));
+        }
+        cabeza.push(("ruta", texto(&b.ruta)));
         let diag = b.diagnostico.clone().unwrap_or_else(|| {
             Mensaje::nuevo(&txt::BLOQUEADA)
                 .con("ruta", b.ruta.clone())
@@ -397,6 +450,8 @@ pub fn init(op: Opciones, c: &Contexto<'_>) -> Hecho {
     if !op.simular {
         // (ruta, error) del primer fallo; lo que venia detras no se aplica.
         let mut fallo: Option<(String, String)> = None;
+        // El informe de `aplicar-puertas`, si se lanzo.
+        let mut puertas: Option<Informe> = None;
         for a in acciones.iter_mut() {
             if a.decision != Decision::Crea {
                 continue;
@@ -406,19 +461,63 @@ pub fn init(op: Opciones, c: &Contexto<'_>) -> Hecho {
                 a.motivo = Some("no-aplicada".to_owned());
                 continue;
             }
-            if let Err(e) = aplicar::aplicar(&a.efecto, c) {
+            let r = match &a.efecto {
+                Efecto::AplicarPuertas { origenes, .. } => {
+                    let i = aplicar::puertas(&conexion, origenes);
+                    let r = if i.resultado.es_exito() {
+                        Ok(())
+                    } else {
+                        Err(i.diagnostico_en(Idioma::Es).unwrap_or_else(|| i.linea()))
+                    };
+                    puertas = Some(i);
+                    r
+                }
+                e => aplicar::aplicar(e, c),
+            };
+            if let Err(e) = r {
                 a.decision = Decision::Pendiente;
                 a.motivo = Some("error".to_owned());
                 fallo = Some((a.ruta.clone(), e));
             }
         }
+        let mut cabeza = Vec::new();
+        if let Some(i) = &puertas {
+            // Cada linea `ruleset` dice lo que de verdad paso: creada,
+            // existente (otro llego antes) o sin aplicar.
+            let (aplicadas, existentes, faltan) = (
+                lista(i, "aplicadas"),
+                lista(i, "existentes"),
+                lista(i, "faltan"),
+            );
+            for a in acciones.iter_mut().filter(|a| a.clase == "ruleset") {
+                if aplicadas.contains(&a.ruta) {
+                    a.decision = Decision::Crea;
+                    a.motivo = None;
+                } else if existentes.contains(&a.ruta) && a.decision != Decision::Existe {
+                    a.decision = Decision::Existe;
+                    a.motivo = None;
+                } else if faltan.contains(&a.ruta) {
+                    a.decision = Decision::Pendiente;
+                    if a.motivo.as_deref() != Some("error") {
+                        a.motivo = Some("no-aplicada".to_owned());
+                    }
+                }
+            }
+            if !i.resultado.es_exito() {
+                cabeza.push(("aplicadas", Valor::Lista(aplicadas)));
+                cabeza.push(("faltan", Valor::Lista(faltan.clone())));
+                if let (Some(f), Some(primera)) = (fallo.as_mut(), faltan.first()) {
+                    f.0.clone_from(primera);
+                }
+            }
+        }
         if let Some((ruta, e)) = fallo {
-            let informe = cuenta(Resultado::Error, &acciones, &[("ruta", ruta.clone())])
-                .diagnostico(
-                    Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
-                        .con("ruta", ruta)
-                        .con("error", e),
-                );
+            cabeza.insert(0, ("ruta", texto(&ruta)));
+            let informe = cuenta(Resultado::Error, &acciones, &cabeza).diagnostico(
+                Mensaje::nuevo(&txt::FALLO_AL_APLICAR)
+                    .con("ruta", ruta)
+                    .con("error", e),
+            );
             return Hecho { acciones, informe };
         }
     }
@@ -467,12 +566,27 @@ mod tests {
     #[test]
     fn opciones() {
         let v = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
-        assert_eq!(leer_opciones(&v(&[])), Ok(Opciones { simular: false }));
+        assert_eq!(leer_opciones(&v(&[])), Ok(Opciones::default()));
         assert_eq!(
             leer_opciones(&v(&["--simular"])),
-            Ok(Opciones { simular: true })
+            Ok(Opciones {
+                simular: true,
+                aplicar_puertas: false
+            })
+        );
+        assert_eq!(
+            leer_opciones(&v(&["--aplicar-puertas", "--simular"])),
+            Ok(Opciones {
+                simular: true,
+                aplicar_puertas: true
+            })
         );
-        for args in [v(&["--simular", "--simular"]), v(&["--dry-run"]), v(&["x"])] {
+        for args in [
+            v(&["--simular", "--simular"]),
+            v(&["--aplicar-puertas", "--aplicar-puertas"]),
+            v(&["--dry-run"]),
+            v(&["x"]),
+        ] {
             let i = leer_opciones(&args).err();
             assert_eq!(
                 i.as_ref().map(|i| (i.resultado, i.valor("motivo"))),
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
index fbc12ce..cbc8c16 100644
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -15,10 +15,10 @@ use super::conexion::{Conexion, head_cambiado, rama_valida, rol_no_valido, sha_v
 use super::credenciales::{Credenciales, Rol};
 use super::errores::{self, es_aviso_de_plan};
 use super::http::{Base, Metodo, codificar};
-use super::puertas::{self, EstadoCheck, Lecturas};
+use super::puertas::{self, EstadoCheck, Lecturas, Origenes, Puerta};
 use super::{
-    FalloLectura, Firma, ModeloPuertas, Paso, Personas, PlanObservado, Plataforma, PrAbierto,
-    Proveedor, ReglasLeidas, Usuario,
+    Creada, EstadoPuerta, FalloLectura, Firma, ModeloPuertas, Paso, Personas, PlanObservado,
+    Plataforma, PrAbierto, Proveedor, PuertasLeidas, ReglasLeidas, Usuario, aplicar_en_lote,
 };
 use crate::contrato::{Informe, Resultado, Valor, corto};
 use crate::i18n::catalogo::plataforma as t;
@@ -41,6 +41,10 @@ pub const PERMISO_LEER_PUERTAS: &str = "metadata:read";
 /// 2026-11-17).
 pub const PERMISO_CHECKS: &str = "checks:read";
 
+/// Permiso de `aplicar-puertas`: leer y crear rulesets (GitHub App:
+/// «Administration» write).
+pub const PERMISO_APLICAR_PUERTAS: &str = "administration:write";
+
 /// Permiso de `leer-permisos-ci`.
 pub const PERMISO_PERMISOS_CI: &str = "administration:read";
 
@@ -61,6 +65,7 @@ enum Endpoint<'a> {
     Merge(u64),
     ReglasRama(&'a str),
     Rulesets,
+    CrearRuleset,
     CheckRuns(&'a str),
     CheckRunsDeRama(&'a str),
     Reviews(u64),
@@ -90,6 +95,7 @@ impl Endpoint<'_> {
             Self::Merge(n) => (Metodo::Put, format!("{r}/pulls/{n}/merge")),
             Self::ReglasRama(rama) => (Metodo::Get, format!("{r}/rules/branches/{rama}")),
             Self::Rulesets => (Metodo::Get, format!("{r}/rulesets?per_page=100")),
+            Self::CrearRuleset => (Metodo::Post, format!("{r}/rulesets")),
             Self::CheckRuns(sha) => (
                 Metodo::Get,
                 format!("{r}/commits/{sha}/check-runs?per_page=100"),
@@ -742,6 +748,88 @@ impl Plataforma for GitHub {
             organizacion: r.owner.tipo == "Organization",
         })
     }
+
+    fn preparar_puertas(&self, ps: &[Puerta], origenes: &Origenes) -> Paso<PuertasLeidas> {
+        let op = puertas::APLICAR_PUERTAS;
+        self.exigir_persona(op)?;
+        // Sin rulesets en el plan (403 con aviso), `rechazada motivo=plan`.
+        let existentes = self.rulesets_de(op, Rol::Persona, PERMISO_APLICAR_PUERTAS)?;
+        // D2: cada check, ya publicado por su App en la cabeza de `develop`.
+        let (_, ruta) = self.ep(&Endpoint::CheckRunsDeRama("develop"));
+        let runs: Vec<CheckRunApi> =
+            self.con
+                .leer_paginas(op, Rol::Persona, PERMISO_CHECKS, ruta, |c| {
+                    self.con
+                        .parsear::<CheckRunsApi>(op, c)
+                        .map(|x| x.check_runs)
+                })?;
+        let vistos: Vec<(String, u64)> = runs
+            .into_iter()
+            .filter_map(|r| r.app.map(|a| (r.name, a.id)))
+            .collect();
+        let faltan = puertas::no_vistos(origenes, &vistos);
+        if !faltan.is_empty() {
+            return Err(puertas::sin_origen_visto(faltan));
+        }
+        // Lo existente se reconoce por el nombre y no se compara: no se
+        // toca (D3), asi que da igual si difiere.
+        Ok(PuertasLeidas {
+            puertas: ps
+                .iter()
+                .map(|p| {
+                    let n = p.nombre_completo();
+                    let e = if existentes.contains(&n) {
+                        EstadoPuerta::Existe
+                    } else {
+                        EstadoPuerta::Falta
+                    };
+                    (n, e)
+                })
+                .collect(),
+            notas: Vec::new(),
+        })
+    }
+
+    fn aplicar_puertas(&self, ps: &[Puerta], origenes: &Origenes) -> Informe {
+        let op = puertas::APLICAR_PUERTAS;
+        let leidas = match self.preparar_puertas(ps, origenes) {
+            Ok(l) => l,
+            Err(i) => return i,
+        };
+        aplicar_en_lote(ps, &leidas, |p| {
+            let cuerpo = puertas::ruleset_github(p, Some(origenes));
+            if cuerpo.get("_nota").is_some() {
+                // Nunca se envia un check sin su `integration_id`.
+                return Err(puertas::sin_origen_visto(
+                    p.checks
+                        .iter()
+                        .filter(|c| origenes.de(c).is_none())
+                        .map(|c| (*c).to_owned())
+                        .collect(),
+                ));
+            }
+            let r = self.con.pedir(
+                op,
+                Rol::Persona,
+                self.ep(&Endpoint::CrearRuleset),
+                Some(&cuerpo.to_string()),
+            )?;
+            if r.exito() {
+                return Ok(Creada::Nueva);
+            }
+            // 422: el nombre ya existe si otro llego antes; se relee.
+            if r.estado == 422
+                && self
+                    .rulesets_de(op, Rol::Persona, PERMISO_APLICAR_PUERTAS)
+                    .is_ok_and(|v| v.contains(&p.nombre_completo()))
+            {
+                return Ok(Creada::YaExistia);
+            }
+            Err(self
+                .con
+                .traducir(op, Rol::Persona, PERMISO_APLICAR_PUERTAS, &r))
+        })
+    }
 }
 
 /// Rama y rol de `leer-puertas`.
@@ -781,6 +869,16 @@ impl GitHub {
         Ok(Some(rulesets.into_iter().map(|r| r.name).collect()))
     }
 
+    /// Los nombres de todos los rulesets del repo para `op`, paginados; el
+    /// 403 de plan es su `rechazada motivo=plan` (`errores::traducir`).
+    fn rulesets_de(&self, op: &str, rol: Rol, permiso: &'static str) -> Paso<Vec<String>> {
+        let (_, ruta) = self.ep(&Endpoint::Rulesets);
+        let rulesets: Vec<RulesetApi> = self
+            .con
+            .leer_paginas(op, rol, permiso, ruta, |c| self.con.parsear(op, c))?;
+        Ok(rulesets.into_iter().map(|r| r.name).collect())
+    }
+
     /// Las reglas de `rama` con los `rulesets` ya leidos; los check-runs de
     /// su cabeza solo `con_checks`.
     fn puertas_de_rama(
diff --git a/crates/tc2/src/plataforma/gitlab.rs b/crates/tc2/src/plataforma/gitlab.rs
index f74bf22..ba5c656 100644
--- a/crates/tc2/src/plataforma/gitlab.rs
+++ b/crates/tc2/src/plataforma/gitlab.rs
@@ -16,10 +16,11 @@ use super::conexion::{Conexion, head_cambiado, rama_valida, rol_no_valido, sha_v
 use super::credenciales::{Credenciales, Rol};
 use super::errores;
 use super::http::{Base, Metodo, codificar};
-use super::puertas::{self, EstadoCheck, Lecturas};
+use super::puertas::{self, EstadoCheck, Lecturas, Objetivo, Origenes, Puerta};
 use super::{
-    Clave, FalloLectura, Firma, ModeloPuertas, Paso, Personas, PlanObservado, Plataforma,
-    PrAbierto, Proveedor, ReglasLeidas, TierGitLab, Usuario,
+    Clave, Creada, EstadoPuerta, FalloLectura, Firma, ModeloPuertas, Paso, Personas, PlanObservado,
+    Plataforma, PrAbierto, Proveedor, PuertasLeidas, ReglasLeidas, TierGitLab, Usuario,
+    aplicar_en_lote,
 };
 use crate::contrato::{Informe, Resultado, Valor, corto};
 use crate::i18n::catalogo::plataforma as t;
@@ -55,6 +56,57 @@ const GARANTIAS_PREMIUM: [&str; 6] = ["G03", "G04", "G05", "G06", "G10", "G12"];
 /// Nivel de acceso de Maintainer.
 const MAINTAINER: u64 = 40;
 
+/// Nivel de acceso de Developer (el agente y el revisor, F1-13 D6).
+const DEVELOPER: u64 = 30;
+
+/// Nivel «nadie» (`No one`).
+const NADIE: u64 = 0;
+
+/// Regla de aprobacion que crea `aplicar-puertas` en Premium.
+const REGLA_APROBACION: &str = "taskcodetwo-aprobacion";
+
+/// Niveles que `aplicar-puertas` pone a una rama protegida, siempre
+/// explicitos (la API pone Maintainer si no se dan, y el agente, Developer,
+/// se quedaria sin push; riesgo 1 del plan de TASK-118).
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+struct Niveles {
+    push: u64,
+    merge: u64,
+}
+
+/// Los niveles de una puerta: `develop` y `main` (solo por MR) sin push y
+/// con merge solo de Maintainer (G01: el merge es de la persona); las ramas
+/// de tarea y la de estado, push y merge de Developer. Nunca force-push.
+fn niveles_de(p: &Puerta) -> Niveles {
+    if p.solo_pr.is_some() {
+        Niveles {
+            push: NADIE,
+            merge: MAINTAINER,
+        }
+    } else {
+        Niveles {
+            push: DEVELOPER,
+            merge: DEVELOPER,
+        }
+    }
+}
+
+/// El nombre de GitLab de un patron del modelo: sin `refs/heads/` ni
+/// `refs/tags/`, y `**` como `*` (el comodin de GitLab).
+fn nombre_gitlab(patron: &str) -> String {
+    patron
+        .trim_start_matches("refs/heads/")
+        .trim_start_matches("refs/tags/")
+        .replace("**", "*")
+}
+
+/// Si una lista de niveles es exactamente `nivel`, sin usuarios ni grupos.
+fn solo_nivel(v: &[NivelApi], nivel: u64) -> bool {
+    v.len() == 1
+        && v.iter()
+            .all(|n| n.access_level == nivel && n.user_id.is_none() && n.group_id.is_none())
+}
+
 /// Endpoints de la API que usa el adaptador (lista cerrada).
 enum Endpoint<'a> {
     Usuario,
@@ -75,6 +127,11 @@ enum Endpoint<'a> {
     Proyecto,
     Rama(&'a str),
     Namespace(u64),
+    CrearRamaProtegida,
+    TagsProtegidos,
+    CrearTagProtegido,
+    AjustesProyecto,
+    CrearReglaAprobacion,
 }
 
 impl Endpoint<'_> {
@@ -132,6 +189,14 @@ impl Endpoint<'_> {
                 Metodo::Get,
                 format!("{p}/repository/branches/{}", codificar(rama)),
             ),
+            // Escrituras de `aplicar-puertas` (TASK-118): solo crean; los
+            // ajustes de merge del proyecto, con `PUT` idempotente. Ningun
+            // `DELETE` ni `PATCH`.
+            Self::CrearRamaProtegida => (Metodo::Post, format!("{p}/protected_branches")),
+            Self::TagsProtegidos => (Metodo::Get, format!("{p}/protected_tags?per_page=100")),
+            Self::CrearTagProtegido => (Metodo::Post, format!("{p}/protected_tags")),
+            Self::AjustesProyecto => (Metodo::Put, p),
+            Self::CrearReglaAprobacion => (Metodo::Post, format!("{p}/approval_rules")),
         }
     }
 }
@@ -279,6 +344,9 @@ struct CommitApi {
 struct ProyectoApi {
     #[serde(default)]
     only_allow_merge_if_pipeline_succeeds: Option<bool>,
+    /// `merge`, `rebase_merge` o `ff` (solo con sesion).
+    #[serde(default)]
+    merge_method: Option<String>,
     #[serde(default)]
     namespace: Option<NamespaceRefApi>,
 }
@@ -299,6 +367,13 @@ struct RamaApi {
     commit: CommitApi,
 }
 
+#[derive(Deserialize)]
+struct TagProtegidoApi {
+    name: String,
+    #[serde(default, deserialize_with = "super::nulo_por_defecto")]
+    create_access_levels: Vec<NivelApi>,
+}
+
 /// Adaptador de GitLab para un proyecto.
 pub struct GitLab {
     con: Conexion,
@@ -908,6 +983,247 @@ impl Plataforma for GitLab {
             n.plan.as_deref().and_then(TierGitLab::de_plan),
         ))
     }
+
+    fn preparar_puertas(&self, ps: &[Puerta], origenes: &Origenes) -> Paso<PuertasLeidas> {
+        self.leer_para_aplicar(ps, origenes).map(|(l, _)| l)
+    }
+
+    fn aplicar_puertas(&self, ps: &[Puerta], origenes: &Origenes) -> Informe {
+        let op = puertas::APLICAR_PUERTAS;
+        let rol = Rol::Persona;
+        let (leidas, pa) = match self.leer_para_aplicar(ps, origenes) {
+            Ok(x) => x,
+            Err(i) => return i,
+        };
+        let mut ajustes = pa.ajustes_bien;
+        let mut aprobacion = pa.con_aprobacion;
+        // Una escritura; 409 es «ya existe» (otro llego antes).
+        let crear = |ep: Endpoint<'_>, datos: serde_json::Value| -> Paso<bool> {
+            let r = self
+                .con
+                .pedir(op, rol, self.ep(&ep), Some(&datos.to_string()))?;
+            if r.exito() {
+                Ok(true)
+            } else if r.estado == 409 {
+                Ok(false)
+            } else {
+                Err(self.con.traducir(op, rol, PERMISO_MERGE, &r))
+            }
+        };
+        aplicar_en_lote(ps, &leidas, |p| {
+            let mut nueva = false;
+            match &p.objetivo {
+                Objetivo::Ramas(r) => {
+                    let n = niveles_de(p);
+                    for nombre in r.iter().map(|x| nombre_gitlab(x)) {
+                        if pa.ramas.contains(&nombre) {
+                            continue;
+                        }
+                        nueva |= crear(
+                            Endpoint::CrearRamaProtegida,
+                            serde_json::json!({
+                                "name": nombre,
+                                "push_access_level": n.push,
+                                "merge_access_level": n.merge,
+                                "allow_force_push": false,
+                            }),
+                        )?;
+                    }
+                }
+                Objetivo::Tags(t) => {
+                    for nombre in t.iter().map(|x| nombre_gitlab(x)) {
+                        if pa.tags.contains(&nombre) {
+                            continue;
+                        }
+                        nueva |= crear(
+                            Endpoint::CrearTagProtegido,
+                            serde_json::json!({
+                                "name": nombre,
+                                "create_access_level": DEVELOPER,
+                            }),
+                        )?;
+                    }
+                }
+            }
+            if let Some(pr) = p.solo_pr {
+                if !ajustes {
+                    let r = self.con.pedir(
+                        op,
+                        rol,
+                        self.ep(&Endpoint::AjustesProyecto),
+                        Some(
+                            &serde_json::json!({
+                                "merge_method": "merge",
+                                "only_allow_merge_if_pipeline_succeeds": true,
+                            })
+                            .to_string(),
+                        ),
+                    )?;
+                    if !r.exito() {
+                        return Err(self.con.traducir(op, rol, PERMISO_MERGE, &r));
+                    }
+                    ajustes = true;
+                    nueva = true;
+                }
+                if pa.premium && !aprobacion {
+                    nueva |= crear(
+                        Endpoint::CrearReglaAprobacion,
+                        serde_json::json!({
+                            "name": REGLA_APROBACION,
+                            "approvals_required": pr.aprobaciones,
+                            "rule_type": "any_approver",
+                        }),
+                    )?;
+                    aprobacion = true;
+                }
+            }
+            Ok(if nueva {
+                Creada::Nueva
+            } else {
+                Creada::YaExistia
+            })
+        })
+    }
+}
+
+/// Lo leido por `aplicar-puertas` que la escritura necesita.
+struct ParaAplicar {
+    /// Nombres de las ramas protegidas.
+    ramas: Vec<String>,
+    /// Nombres de los tags protegidos.
+    tags: Vec<String>,
+    /// El proyecto ya mergea con merge commit y exige el pipeline.
+    ajustes_bien: bool,
+    /// Premium o Ultimate: hay reglas de aprobacion.
+    premium: bool,
+    /// Ya hay una regla de aprobacion que pide al menos una.
+    con_aprobacion: bool,
+}
+
+impl GitLab {
+    /// Las lecturas de `aplicar-puertas`: la persona de la lista, lo que ya
+    /// esta protegido, D2 (cada check ya publicado en la cabeza de
+    /// `develop`: `revision-ia` por el revisor, los del pipeline por
+    /// cualquiera), los ajustes del proyecto y el tier. Lo existente se
+    /// reconoce por la rama o el tag protegido, no por el nombre de una
+    /// regla de aprobacion.
+    fn leer_para_aplicar(
+        &self,
+        ps: &[Puerta],
+        origenes: &Origenes,
+    ) -> Paso<(PuertasLeidas, ParaAplicar)> {
+        let op = puertas::APLICAR_PUERTAS;
+        let rol = Rol::Persona;
+        self.exigir_persona(op)?;
+        let (_, ruta) = self.ep(&Endpoint::RamasProtegidas);
+        let ramas: Vec<RamaProtegidaApi> =
+            self.con
+                .leer_paginas(op, rol, PERMISO_MERGE, ruta, |c| self.con.parsear(op, c))?;
+        let (_, ruta) = self.ep(&Endpoint::TagsProtegidos);
+        let tags: Vec<TagProtegidoApi> =
+            self.con
+                .leer_paginas(op, rol, PERMISO_MERGE, ruta, |c| self.con.parsear(op, c))?;
+        let develop: RamaApi = self.con.leer(
+            op,
+            rol,
+            PERMISO_LECTURA,
+            self.ep(&Endpoint::Rama("develop")),
+        )?;
+        if !sha_valido(&develop.commit.id) {
+            return Err(self.con.invalida_por(op, &t::RAMA_DESTINO_O_SHA));
+        }
+        let (_, ruta) = self.ep(&Endpoint::Estados(&develop.commit.id));
+        let estados: Vec<EstadoApi> =
+            self.con
+                .leer_paginas(op, rol, PERMISO_LECTURA, ruta, |c| self.con.parsear(op, c))?;
+        let vistos: Vec<(String, u64)> =
+            estados.into_iter().map(|e| (e.name, e.author.id)).collect();
+        let faltan = puertas::no_vistos(origenes, &vistos);
+        if !faltan.is_empty() {
+            return Err(puertas::sin_origen_visto(faltan));
+        }
+        let proyecto: ProyectoApi =
+            self.con
+                .leer(op, rol, PERMISO_LECTURA, self.ep(&Endpoint::Proyecto))?;
+        let ajustes_bien = proyecto.merge_method.as_deref() == Some("merge")
+            && proyecto.only_allow_merge_if_pipeline_succeeds == Some(true);
+        let premium = matches!(
+            self.leer_plan(rol).map_err(|mut i| {
+                op.clone_into(&mut i.comando);
+                i
+            })?,
+            PlanObservado::GitLab(Some(t)) if t >= TierGitLab::Premium
+        );
+        let con_aprobacion = premium && {
+            let reglas: Vec<ReglaAprobacionApi> = self.con.leer(
+                op,
+                rol,
+                PERMISO_LECTURA,
+                self.ep(&Endpoint::ReglasAprobacion),
+            )?;
+            reglas.iter().any(|r| r.approvals_required >= 1)
+        };
+        let mut leidas = PuertasLeidas::default();
+        for p in ps {
+            let estado = match &p.objetivo {
+                Objetivo::Ramas(r) => {
+                    let n = niveles_de(p);
+                    let mut e = EstadoPuerta::Existe;
+                    for nombre in r.iter().map(|x| nombre_gitlab(x)) {
+                        match ramas.iter().find(|x| x.name == nombre) {
+                            None => e = EstadoPuerta::Falta,
+                            Some(x)
+                                if x.allow_force_push
+                                    || !solo_nivel(&x.push_access_levels, n.push)
+                                    || !solo_nivel(&x.merge_access_levels, n.merge) =>
+                            {
+                                if e != EstadoPuerta::Falta {
+                                    e = EstadoPuerta::Difiere;
+                                }
+                            }
+                            Some(_) => {}
+                        }
+                    }
+                    if e == EstadoPuerta::Existe
+                        && p.solo_pr.is_some()
+                        && (!ajustes_bien || (premium && !con_aprobacion))
+                    {
+                        e = EstadoPuerta::Difiere;
+                    }
+                    e
+                }
+                Objetivo::Tags(t) => {
+                    let mut e = EstadoPuerta::Existe;
+                    for nombre in t.iter().map(|x| nombre_gitlab(x)) {
+                        match tags.iter().find(|x| x.name == nombre) {
+                            None => e = EstadoPuerta::Falta,
+                            Some(x) if !solo_nivel(&x.create_access_levels, DEVELOPER) => {
+                                if e != EstadoPuerta::Falta {
+                                    e = EstadoPuerta::Difiere;
+                                }
+                            }
+                            Some(_) => {}
+                        }
+                    }
+                    e
+                }
+            };
+            leidas.puertas.push((p.nombre_completo(), estado));
+        }
+        if !premium {
+            leidas.notas.push(("sin-aprobaciones", "si".to_owned()));
+        }
+        Ok((
+            leidas,
+            ParaAplicar {
+                ramas: ramas.into_iter().map(|r| r.name).collect(),
+                tags: tags.into_iter().map(|t| t.name).collect(),
+                ajustes_bien,
+                premium,
+                con_aprobacion,
+            },
+        ))
+    }
 }
 
 /// Rama y rol de `leer-puertas`.
diff --git a/crates/tc2/src/plataforma/mod.rs b/crates/tc2/src/plataforma/mod.rs
index 549748b..1550868 100644
--- a/crates/tc2/src/plataforma/mod.rs
+++ b/crates/tc2/src/plataforma/mod.rs
@@ -281,8 +281,101 @@ pub struct PrAbierto {
     pub borrador: bool,
 }
 
+/// Como esta una puerta del modelo en la plataforma (`aplicar-puertas`).
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum EstadoPuerta {
+    /// No esta: se crea.
+    Falta,
+    /// Ya esta: no se toca.
+    Existe,
+    /// Ya esta, distinta del modelo: no se toca y se informa (TASK-118 D3/D4:
+    /// lo existente nunca se reescribe ni se borra).
+    Difiere,
+}
+
+/// Lo leido antes de aplicar las puertas: el estado de cada una, en el orden
+/// del modelo, y notas para la linea (`sin-aprobaciones=si`...).
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct PuertasLeidas {
+    /// `(nombre completo, estado)`.
+    pub puertas: Vec<(String, EstadoPuerta)>,
+    /// Claves que se anaden a la linea.
+    pub notas: Vec<(&'static str, String)>,
+}
+
+impl PuertasLeidas {
+    /// Nombres con ese estado.
+    pub fn con_estado(&self, e: EstadoPuerta) -> Vec<String> {
+        self.puertas
+            .iter()
+            .filter(|(_, x)| *x == e)
+            .map(|(n, _)| n.clone())
+            .collect()
+    }
+}
+
+/// Lo que dio la creacion de una puerta.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub(crate) enum Creada {
+    /// Creada ahora.
+    Nueva,
+    /// Otro llego antes (422 de GitHub, 409 de GitLab): ya existe.
+    YaExistia,
+}
+
+/// Escribe las puertas que faltan de `leidas`, una a una, con `crear`. Un
+/// fallo corta: la linea dice `aplicadas=` y `faltan=`, y repetirla
+/// converge (lo creado sale como existente). Nunca reescribe ni borra.
+pub(crate) fn aplicar_en_lote(
+    puertas: &[puertas::Puerta],
+    leidas: &PuertasLeidas,
+    mut crear: impl FnMut(&puertas::Puerta) -> Paso<Creada>,
+) -> Informe {
+    let mut aplicadas = Vec::new();
+    let mut existentes = leidas.con_estado(EstadoPuerta::Existe);
+    let difieren = leidas.con_estado(EstadoPuerta::Difiere);
+    let faltan: Vec<&puertas::Puerta> = puertas
+        .iter()
+        .filter(|p| {
+            leidas
+                .puertas
+                .iter()
+                .any(|(n, e)| *n == p.nombre_completo() && *e == EstadoPuerta::Falta)
+        })
+        .collect();
+    for (k, p) in faltan.iter().enumerate() {
+        match crear(p) {
+            Ok(Creada::Nueva) => aplicadas.push(p.nombre_completo()),
+            Ok(Creada::YaExistia) => existentes.push(p.nombre_completo()),
+            Err(mut i) => {
+                puertas::APLICAR_PUERTAS.clone_into(&mut i.comando);
+                let resto = faltan[k..].iter().map(|p| p.nombre_completo()).collect();
+                return i
+                    .con_lista("aplicadas", aplicadas)
+                    .con_lista("faltan", resto);
+            }
+        }
+    }
+    let r = if aplicadas.is_empty() {
+        crate::contrato::Resultado::SinCambios
+    } else {
+        crate::contrato::Resultado::Ok
+    };
+    let mut i = Informe::nuevo(puertas::APLICAR_PUERTAS, r)
+        .con_lista("aplicadas", aplicadas)
+        .con_lista("existentes", existentes);
+    if !difieren.is_empty() {
+        i = i.con_lista("difieren", difieren);
+    }
+    for (k, v) in &leidas.notas {
+        i = i.con(k, v.clone());
+    }
+    i
+}
+
 /// Puerto `Plataforma`: lista cerrada de operaciones (F1-4, F1-5, F1-6,
-/// F1-13). Las escrituras de puertas (`aplicar-puertas`) son de F3-1.
+/// F1-13). La unica escritura de puertas es `aplicar-puertas` (TASK-118),
+/// que lanza la persona con `tc2 init --aplicar-puertas`.
 pub trait Plataforma {
     /// `prs-abiertos` (agente o persona; adenda de TASK-032 a F1-4 y F1-13):
     /// los PR abiertos cuya rama de origen es del propio repo (los de forks
@@ -342,4 +435,39 @@ pub trait Plataforma {
                 .con("motivo", "no-soportada"),
         )
     }
+
+    /// Lecturas de `aplicar-puertas` (persona; TASK-118): comprueba todo lo
+    /// que la escritura exige (la persona de la lista, el plan y que cada
+    /// check del modelo ya se haya visto publicado por su origen en la
+    /// cabeza de `develop`) y dice como esta cada puerta. No escribe nada.
+    /// Por defecto, no soportada.
+    fn preparar_puertas(
+        &self,
+        _puertas: &[puertas::Puerta],
+        _origenes: &puertas::Origenes,
+    ) -> Paso<PuertasLeidas> {
+        Err(no_soportada())
+    }
+
+    /// `aplicar-puertas` (persona; F1-5, TASK-118): primero
+    /// [`Plataforma::preparar_puertas`] y, solo si pasa, crea las puertas
+    /// que faltan. Lo que ya existe no se toca (nunca `PUT` de un ruleset
+    /// ni `DELETE`). Por defecto, `rechazada motivo=no-soportada`.
+    fn aplicar_puertas(
+        &self,
+        _puertas: &[puertas::Puerta],
+        _origenes: &puertas::Origenes,
+    ) -> Informe {
+        no_soportada()
+    }
+}
+
+/// `aplicar-puertas rechazada motivo=no-soportada` (adaptadores de prueba).
+fn no_soportada() -> Informe {
+    Informe::nuevo(
+        puertas::APLICAR_PUERTAS,
+        crate::contrato::Resultado::Rechazada,
+    )
+    .con("motivo", "no-soportada")
+    .diagnostico(&crate::i18n::catalogo::plataforma::APLICAR_NO_SOPORTADA)
 }
diff --git a/crates/tc2/src/plataforma/puertas.rs b/crates/tc2/src/plataforma/puertas.rs
index ee5c27b..b16f2c0 100644
--- a/crates/tc2/src/plataforma/puertas.rs
+++ b/crates/tc2/src/plataforma/puertas.rs
@@ -9,6 +9,35 @@ use crate::i18n::{Mensaje, Texto};
 /// Nombre de la operacion.
 pub const ESTADO_PUERTAS: &str = "estado-puertas";
 
+/// Nombre de la operacion de escritura de puertas (TASK-118).
+pub const APLICAR_PUERTAS: &str = "aplicar-puertas";
+
+/// `aplicar-puertas` rechazada porque los checks `faltan` no se han visto
+/// publicados por su origen en la cabeza de `develop` (TASK-118 D2):
+/// exigirlos bloquearia `develop` para todos.
+pub fn sin_origen_visto(faltan: Vec<String>) -> Informe {
+    let diag = Mensaje::nuevo(&t::CHECKS_SIN_ORIGEN_VISTO).con("faltan", faltan.join(", "));
+    Informe::nuevo(APLICAR_PUERTAS, Resultado::Rechazada)
+        .con("motivo", "puertas-sin-origen")
+        .con_lista("faltan", faltan)
+        .diagnostico(diag)
+}
+
+/// Los checks de `o` que no estan en `vistos` (`(nombre, origen)` de los
+/// publicados en la cabeza de `develop`): los de origen fijo tienen que
+/// estar publicados por ese origen; los del pipeline, por cualquiera. En el
+/// orden del modelo.
+pub fn no_vistos(o: &Origenes, vistos: &[(String, u64)]) -> Vec<String> {
+    checks_del_modelo()
+        .into_iter()
+        .filter(|c| match o.de(c) {
+            Some(id) => !vistos.iter().any(|(n, x)| n == c && *x == id),
+            None => !vistos.iter().any(|(n, _)| n == c),
+        })
+        .map(str::to_owned)
+        .collect()
+}
+
 /// Estado de un check obligatorio, contado solo si lo publico su origen.
 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
 pub enum EstadoCheck {
@@ -154,7 +183,7 @@ pub struct Puerta {
     pub solo_pr: Option<SoloPr>,
     /// Rama al dia con la base antes del merge (`strict`).
     pub al_dia: bool,
-    /// Checks obligatorios. Su origen (`integration_id`) llega con F6-1.
+    /// Checks obligatorios. Su origen, en [`Origenes`].
     pub checks: Vec<&'static str>,
 }
 
@@ -209,10 +238,87 @@ pub fn modelo() -> Vec<Puerta> {
     ]
 }
 
+/// `integration_id` de GitHub Actions (F1-5, PoC A): el origen de
+/// `verificacion` y `carril` en GitHub.
+pub const ID_ACTIONS: u64 = 15368;
+
+/// Checks del modelo que publica el revisor IA (su origen es
+/// `plataforma.revisor_id`).
+const CHECKS_DEL_REVISOR: [&str; 1] = ["revision-ia"];
+
+/// De donde sale cada check obligatorio del modelo (TASK-118, D-id del
+/// plan): sin el, una puerta con checks no se aplica.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Origenes {
+    /// `(nombre, origen)`: en GitHub, el `integration_id` de la App que lo
+    /// publica (`app.id` del check-run); en GitLab, el `author.id` del
+    /// estado del commit (F1-13, decision 6).
+    pub checks: Vec<(String, u64)>,
+    /// Checks sin origen fijo (GitLab: los jobs del pipeline, que impone
+    /// «Pipelines must succeed» entero; su autor es quien lanzo el
+    /// pipeline, F1-13 G09). Cuentan con que se hayan visto publicados.
+    pub del_pipeline: Vec<String>,
+}
+
+impl Origenes {
+    /// El origen de `check`, si tiene uno fijo.
+    pub fn de(&self, check: &str) -> Option<u64> {
+        self.checks
+            .iter()
+            .find(|(n, _)| n == check)
+            .map(|(_, id)| *id)
+    }
+}
+
+/// Los checks obligatorios del modelo, sin repetir, en orden de aparicion.
+pub fn checks_del_modelo() -> Vec<&'static str> {
+    let mut v: Vec<&'static str> = Vec::new();
+    for p in modelo() {
+        for c in p.checks {
+            if !v.contains(&c) {
+                v.push(c);
+            }
+        }
+    }
+    v
+}
+
+/// Los origenes de los checks del modelo en `proveedor`, con `revisor_id`
+/// de la configuracion (`plataforma.revisor_id`; 0 cuenta como ausente).
+/// Pura. `Err` con los checks que se quedan sin origen.
+///
+/// GitHub: `verificacion` y `carril` los publica Actions ([`ID_ACTIONS`]) y
+/// `revision-ia` la App del revisor. GitLab: `revision-ia` es un estado del
+/// revisor (`author.id`) y `verificacion` y `carril` son del pipeline.
+pub fn origenes(
+    proveedor: super::Proveedor,
+    revisor_id: Option<u64>,
+) -> Result<Origenes, Vec<&'static str>> {
+    let revisor = revisor_id.filter(|id| *id != 0);
+    let mut o = Origenes::default();
+    let mut faltan = Vec::new();
+    for c in checks_del_modelo() {
+        let del_revisor = CHECKS_DEL_REVISOR.contains(&c);
+        match (del_revisor, proveedor, revisor) {
+            (true, _, Some(id)) => o.checks.push((c.to_owned(), id)),
+            (true, _, None) => faltan.push(c),
+            (false, super::Proveedor::GitHub, _) => o.checks.push((c.to_owned(), ID_ACTIONS)),
+            (false, super::Proveedor::GitLab, _) => o.del_pipeline.push(c.to_owned()),
+        }
+    }
+    if faltan.is_empty() {
+        Ok(o)
+    } else {
+        Err(faltan)
+    }
+}
+
 /// La puerta traducida a un ruleset de GitHub (la forma de los JSON de la
-/// PoC C de F1-5). Solo genera la propuesta: aplicarla es de TASK-118, y
-/// los checks van sin `integration_id` hasta que F6-1 de el real.
-pub fn ruleset_github(p: &Puerta) -> serde_json::Value {
+/// PoC C de F1-5). Sin `origenes` es la propuesta de `init` (checks sin
+/// `integration_id` y la clave `_nota`, que GitHub no admite); con ellos,
+/// el cuerpo que `aplicar-puertas` envia (TASK-118), con el
+/// `integration_id` de cada check.
+pub fn ruleset_github(p: &Puerta, origenes: Option<&Origenes>) -> serde_json::Value {
     use serde_json::json;
     let (target, include) = match &p.objetivo {
         Objetivo::Ramas(r) => ("branch", r),
@@ -239,9 +345,20 @@ pub fn ruleset_github(p: &Puerta) -> serde_json::Value {
             }
         }));
     }
+    // Checks sin origen conocido: con origenes no deberia quedar ninguno.
+    let mut sin_origen = false;
     if !p.checks.is_empty() {
-        let checks: Vec<serde_json::Value> =
-            p.checks.iter().map(|c| json!({"context": c})).collect();
+        let checks: Vec<serde_json::Value> = p
+            .checks
+            .iter()
+            .map(|c| match origenes.and_then(|o| o.de(c)) {
+                Some(id) => json!({"context": c, "integration_id": id}),
+                None => {
+                    sin_origen = true;
+                    json!({"context": c})
+                }
+            })
+            .collect();
         reglas.push(json!({
             "type": "required_status_checks",
             "parameters": {
@@ -253,7 +370,7 @@ pub fn ruleset_github(p: &Puerta) -> serde_json::Value {
     // Sin `integration_id` cualquier identidad con permiso de escribir checks
     // los daria por buenos (F1-5 lo descarta): la propuesta lleva una clave
     // que GitHub no admite, asi que no se puede aplicar tal cual.
-    let nota = (!p.checks.is_empty()).then_some(NOTA_SIN_ORIGEN);
+    let nota = sin_origen.then_some(NOTA_SIN_ORIGEN);
     let mut v = json!({
         "name": p.nombre_completo(),
         "target": target,
@@ -269,8 +386,7 @@ pub fn ruleset_github(p: &Puerta) -> serde_json::Value {
 }
 
 /// Nota de una propuesta con checks sin origen: no es aplicable.
-pub const NOTA_SIN_ORIGEN: &str =
-    "no aplicable: checks sin integration_id hasta F6-1; la aplica TASK-118";
+pub const NOTA_SIN_ORIGEN: &str = "no aplicable: checks sin integration_id; la aplica tc2 init --aplicar-puertas con plataforma.revisor_id y los checks ya publicados en develop";
 
 #[cfg(test)]
 mod tests {
@@ -302,7 +418,7 @@ mod tests {
                 })
                 .unwrap_or_default()
         };
-        let develop = ruleset_github(&m[0]);
+        let develop = ruleset_github(&m[0], None);
         assert_eq!(develop["name"], "taskcodetwo-develop");
         assert_eq!(develop["target"], "branch");
         assert_eq!(
@@ -318,7 +434,7 @@ mod tests {
             develop["rules"][3]["parameters"]["strict_required_status_checks_policy"],
             true
         );
-        let main = ruleset_github(&m[1]);
+        let main = ruleset_github(&m[1], None);
         assert_eq!(
             main["rules"][3]["parameters"]["strict_required_status_checks_policy"],
             false
@@ -329,18 +445,72 @@ mod tests {
             develop["rules"][3]["parameters"]["required_status_checks"][0]["integration_id"]
                 .is_null()
         );
-        let tareas = ruleset_github(&m[3]);
+        let tareas = ruleset_github(&m[3], None);
         assert!(tareas.get("_nota").is_none());
         assert_eq!(
             tareas["conditions"]["ref_name"]["include"][0],
             "refs/heads/taskcodetwo/tareas"
         );
         assert_eq!(tipos(&tareas), ["deletion", "non_fast_forward"]);
-        let tags = ruleset_github(&m[4]);
+        let tags = ruleset_github(&m[4], None);
         assert_eq!(tags["target"], "tag");
         assert_eq!(tipos(&tags), ["deletion", "update"]);
     }
 
+    /// TASK-118, criterio 2 (unitario): sin `revisor_id` (o con 0),
+    /// `revision-ia` se queda sin origen en las dos plataformas; con el, en
+    /// GitHub `verificacion` y `carril` son de Actions y en GitLab del
+    /// pipeline.
+    #[test]
+    fn origenes_de_los_checks() {
+        use super::super::Proveedor;
+        for p in [Proveedor::GitHub, Proveedor::GitLab] {
+            assert_eq!(origenes(p, None), Err(vec!["revision-ia"]), "{p:?}");
+            assert_eq!(origenes(p, Some(0)), Err(vec!["revision-ia"]), "{p:?}");
+        }
+        let gh = origenes(Proveedor::GitHub, Some(99001)).unwrap_or_default();
+        assert_eq!(
+            gh.checks,
+            [
+                ("verificacion".to_owned(), ID_ACTIONS),
+                ("revision-ia".to_owned(), 99001),
+                ("carril".to_owned(), ID_ACTIONS)
+            ]
+        );
+        assert!(gh.del_pipeline.is_empty());
+        let gl = origenes(Proveedor::GitLab, Some(2002)).unwrap_or_default();
+        assert_eq!(gl.checks, [("revision-ia".to_owned(), 2002)]);
+        assert_eq!(gl.del_pipeline, ["verificacion", "carril"]);
+    }
+
+    /// Con origenes, el ruleset lleva el `integration_id` de cada check y no
+    /// la `_nota`: es el cuerpo que se envia.
+    #[test]
+    fn ruleset_con_origenes_es_aplicable() {
+        let o = origenes(super::super::Proveedor::GitHub, Some(99001)).unwrap_or_default();
+        let m = modelo();
+        let develop = ruleset_github(&m[0], Some(&o));
+        assert!(develop.get("_nota").is_none(), "{develop}");
+        let checks = &develop["rules"][3]["parameters"]["required_status_checks"];
+        assert_eq!(
+            checks,
+            &serde_json::json!([
+                {"context": "verificacion", "integration_id": ID_ACTIONS},
+                {"context": "revision-ia", "integration_id": 99001},
+                {"context": "carril", "integration_id": ID_ACTIONS}
+            ])
+        );
+        // Un origen que falta deja la nota: nunca se envia sin el.
+        let parcial = Origenes {
+            checks: vec![("verificacion".into(), ID_ACTIONS)],
+            del_pipeline: Vec::new(),
+        };
+        assert_eq!(
+            ruleset_github(&m[1], Some(&parcial))["_nota"],
+            NOTA_SIN_ORIGEN
+        );
+    }
+
     fn l(checks: &[(&str, EstadoCheck)], aprob: usize, al_dia: bool) -> Lecturas {
         Lecturas {
             pr: 7,
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-118/planificacion/brainstorm/peticion-brainstorm-arquitectura-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-118/planificacion/brainstorm/peticion-brainstorm-riesgos-1.md      | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-118/planificacion/brainstorm/peticion-unificador-1.md              | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-118/planificacion/brainstorm/salida-brainstorm-arquitectura-1.md   | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-118/planificacion/brainstorm/salida-brainstorm-riesgos-1.md        | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-118/planificacion/plan-final.md                                    | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-118/tarea.md                                                       | 8 +++++++-
 7 files changed, 7 insertions(+), 1 deletion(-)
````

## Tests del diff (3 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/init.rs" ":(literal)crates/tc2/tests/plataforma.rs" ":(literal)crates/tc2/tests/plataforma_gitlab.rs"
````

````
crates/tc2/tests/init.rs              | 565 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-
 crates/tc2/tests/plataforma.rs        |  87 ++++++++++++++++++++++++
 crates/tc2/tests/plataforma_gitlab.rs | 116 ++++++++++++++++++++++++++++++++
 3 files changed, 767 insertions(+), 1 deletion(-)
````
