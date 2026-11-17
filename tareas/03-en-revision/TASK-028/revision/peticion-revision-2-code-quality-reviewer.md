# Peticion de revision — TASK-028 (ronda 2)

- Tarea: TASK-028 — F3-1 tc2 init
- Rama revisada: feature/task-028-f3-1-taskctl-init-setup
- Rama base: develop
- Commit revisado (HEAD): 2da874858cbdd8149b00e8c3fa4fe864bd3a560a
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-028 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 2c3706b7cb68b2e455377ec33ab0766aae68db39 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (13 hallazgo(s)))

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
informe de esta ronda (informe-revision-2-code-quality-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-028/tarea.md
- tareas/03-en-revision/TASK-028/planificacion/
- crates/tc2/src/init/aplicar.rs
- crates/tc2/src/init/inventario.rs
- crates/tc2/src/init/mod.rs
- crates/tc2/src/plataforma/puertas.rs
- crates/tc2/tests/init.rs
- docs/adr/F1-14-distribucion-e-instalacion.md
- docs/adr/F1-18-hito-de-decisiones.md
- docs/adr/F1-5-puertas-en-github.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 7 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | crates/tc2/src/init/inventario.rs:302 | informe-revision-1-code-quality-reviewer.md |

## Commits a revisar (git log 2c3706b7cb68b2e455377ec33ab0766aae68db39..HEAD)

````
2da8748 docs(TASK-028): adendas D4 en F1-18 y F1-14, F1-5 coherente con la adenda y C1 y C2 como se prueban
4a25582 fix(TASK-028): repo vacio solo si origin no tiene ramas, ordenes entrecomilladas, motivo real de detectar, propuesta no aplicable y linea para versionar la config
a3b807a chore(TASK-028): veredictos y coste de la revision ronda 1
d8d9854 chore(TASK-028): informes de revision ronda 1
9fff220 chore(TASK-028): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 2c3706b7cb68b2e455377ec33ab0766aae68db39..HEAD)

````diff
diff --git a/crates/tc2/src/init/aplicar.rs b/crates/tc2/src/init/aplicar.rs
index 4a5abb7..f922147 100644
--- a/crates/tc2/src/init/aplicar.rs
+++ b/crates/tc2/src/init/aplicar.rs
@@ -90,7 +90,7 @@ fn anadir(ruta: &Path, lineas: &[String]) -> Result<(), String> {
     };
     let mut nuevo = actual.clone();
     for l in lineas {
-        if actual.lines().any(|x| x.trim() == l) {
+        if actual.lines().any(|x| super::inventario::misma_linea(x, l)) {
             continue;
         }
         if !nuevo.is_empty() && !nuevo.ends_with('\n') {
diff --git a/crates/tc2/src/init/inventario.rs b/crates/tc2/src/init/inventario.rs
index a21e763..9bafe8c 100644
--- a/crates/tc2/src/init/inventario.rs
+++ b/crates/tc2/src/init/inventario.rs
@@ -21,21 +21,35 @@ use crate::plataforma::puertas::{modelo, ruleset_github};
 use crate::plataforma::{GitHub, GitLab, Plataforma, Proveedor, Rol, detectar};
 
 /// El adaptador de la plataforma y su proveedor, o el motivo por el que no
-/// lo hay (`sin-remoto`, `sin-plataforma`, `sin-token`).
-type Conexion = Result<(Box<dyn Plataforma>, Proveedor), &'static str>;
+/// lo hay (`sin-remoto`, `sin-token` o el motivo de `detectar`, como
+/// `plataforma-contradice`).
+type Conexion = Result<(Box<dyn Plataforma>, Proveedor), String>;
 
 /// Ramas de codigo que `init` deja (F1-3).
 const RAMAS: [&str; 2] = ["main", "develop"];
 
-/// Lineas de `lineas` que no estan en `texto` (comparadas sin espacios).
-fn faltan(texto: &str, lineas: &[&str]) -> Vec<String> {
+/// Lineas de `lineas` que no estan en `texto`. Se comparan sin espacios y
+/// sin la barra final: `/tareas` ya ignora lo mismo que `/tareas/`.
+pub(super) fn faltan(texto: &str, lineas: &[&str]) -> Vec<String> {
     lineas
         .iter()
-        .filter(|l| !texto.lines().any(|x| x.trim() == **l))
+        .filter(|l| !texto.lines().any(|x| misma_linea(x, l)))
         .map(|l| (*l).to_owned())
         .collect()
 }
 
+/// Si dos lineas de `.gitignore` o `exclude` son la misma regla.
+pub(super) fn misma_linea(a: &str, b: &str) -> bool {
+    let norm = |s: &str| s.trim().trim_end_matches('/').to_owned();
+    norm(a) == norm(b)
+}
+
+/// Orden entrecomillada para un shell POSIX (y PowerShell, salvo `'`):
+/// una ruta con espacios o `;` no parte la orden ni ejecuta nada.
+pub(super) fn comillas(s: &str) -> String {
+    format!("'{}'", s.replace('\'', "'\\''"))
+}
+
 fn sinred(detalle: &str) -> Informe {
     Informe::nuevo(NOMBRE, Resultado::Sinred)
         .con("pendiente", "no")
@@ -129,7 +143,11 @@ pub(super) fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
     };
 
     // `main` y `develop` (F1-3): con un commit raiz solo en un repo vacio.
-    ramas(c, &principal, remoto, &remotas, &mut v)?;
+    if !ramas(c, &principal, remoto, &remotas, &mut v)?
+        && let Some(a) = versionar(c, &principal, &v)?
+    {
+        v.push(a);
+    }
 
     // `info/exclude`.
     let comun = dir_comun(g, &principal, NOMBRE)?;
@@ -173,7 +191,11 @@ pub(super) fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
             Decision::Rechaza,
         )
         .motivo("montada-en-otra-ruta")
-        .orden(format!("git worktree move {} {}", t(&r), CARPETA)),
+        .orden(format!(
+            "git worktree move {} {}",
+            comillas(&t(&r)),
+            CARPETA
+        )),
         Previsto::SinRed(_) | Previsto::Denegada(..) => {
             // Ya devuelto arriba.
             Accion::nueva(
@@ -189,7 +211,7 @@ pub(super) fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
     let plataforma = if remoto {
         plataforma(c, &principal, clave.as_ref())?
     } else {
-        Err("sin-remoto")
+        Err("sin-remoto".to_owned())
     };
     v.push(personas_yml(&principal, &plataforma)?);
 
@@ -220,12 +242,16 @@ fn rechazo_016(ruta: String, clase: &'static str) -> Accion {
     a
 }
 
-/// Ramas de codigo que tiene `origin` (de [`RAMAS`]). Sin red, `sinred`.
+/// Todas las ramas que tiene `origin` (para saber si tiene historia, no solo
+/// `main` y `develop`). Sin red, `sinred`.
 fn ramas_remotas(c: &Contexto<'_>, principal: &Path) -> Result<Vec<String>, Informe> {
-    let mut args = vec!["--heads".to_owned(), REMOTO.to_owned()];
-    args.extend(RAMAS.iter().map(|r| format!("refs/heads/{r}")));
-    let args: Vec<&str> = args.iter().map(String::as_str).collect();
-    let s = correr(c.git, principal, NOMBRE, Subcomando::LsRemote, &args)?;
+    let s = correr(
+        c.git,
+        principal,
+        NOMBRE,
+        Subcomando::LsRemote,
+        &["--heads", REMOTO],
+    )?;
     if !s.exito() {
         return Err(match clasificar_fallo_remoto(&s.stderr) {
             FalloRemoto::SinRed | FalloRemoto::RamaInexistente => sinred(&s.primera_linea_error()),
@@ -241,16 +267,20 @@ fn ramas_remotas(c: &Contexto<'_>, principal: &Path) -> Result<Vec<String>, Info
         .collect())
 }
 
-/// Lineas de `main` y `develop` (D3): en un repo sin commits, un commit raiz
-/// con la configuracion y `.gitignore` y las dos ramas en el (publicadas si
-/// hay remoto); con historia, lo hace la persona.
+/// Lineas de `main` y `develop` (D3). El repo cuenta como vacio solo si no
+/// tiene commits y `origin` no tiene ninguna rama (la de estado no cuenta:
+/// es huerfana). Vacio: un commit raiz con la configuracion y `.gitignore` y
+/// las dos ramas en el (publicadas si hay remoto). Con historia, local o en
+/// `origin`, la rama que falta la crea la persona desde la que ya hay: un
+/// commit raiz propio dejaria en el remoto dos historias sin relacion.
+/// Devuelve si el repo estaba vacio.
 fn ramas(
     c: &Contexto<'_>,
     principal: &Path,
     remoto: bool,
     remotas: &[String],
     v: &mut Vec<Accion>,
-) -> Result<(), Informe> {
+) -> Result<bool, Informe> {
     let g = c.git;
     let hay_commits = !exigir(
         g,
@@ -270,12 +300,19 @@ fn ramas(
             locales.push(r);
         }
     }
+    // Ramas de `origin` con historia del proyecto (sin la de estado).
+    let historia_remota: Vec<&str> = remotas
+        .iter()
+        .map(String::as_str)
+        .filter(|r| *r != crate::huella::RAMA_ESTADO)
+        .collect();
+    let vacio = !hay_commits && historia_remota.is_empty();
     let mut nuevas: Vec<String> = Vec::new();
     let mut lineas = Vec::new();
     for r in RAMAS {
         let ruta = format!("refs/heads/{r}");
         let local = locales.contains(&r);
-        let en_remoto = remotas.iter().any(|x| x == r);
+        let en_remoto = historia_remota.contains(&r);
         let a = Accion::nueva(ruta, "rama", "proyecto", Decision::Existe);
         lineas.push(if local {
             if remoto && !en_remoto {
@@ -289,15 +326,30 @@ fn ramas(
                 "en-el-remoto",
                 format!("git fetch {REMOTO} {r} && git branch {r} {REMOTO}/{r}"),
             )
-        } else if hay_commits {
-            // Desde la otra rama si existe (HEAD puede estar sin nacer en
-            // esta, tras un `init` cortado a medias); si no, desde HEAD.
-            let desde = locales.first().copied().unwrap_or("HEAD");
-            let mut o = format!("git branch {r} {desde}");
+        } else if !vacio {
+            // Desde la otra rama local si existe (HEAD puede estar sin nacer
+            // en esta, tras un `init` cortado a medias); si no, desde la otra
+            // de `origin`, desde HEAD o desde la primera rama de `origin`.
+            let otra_remota = RAMAS
+                .iter()
+                .find(|x| historia_remota.contains(x))
+                .or_else(|| historia_remota.first());
+            let mut o = match (locales.first(), otra_remota) {
+                (Some(l), _) => format!("git branch {r} {l}"),
+                (None, Some(b)) if !hay_commits => {
+                    format!("git fetch {REMOTO} {b} && git branch {r} {REMOTO}/{b}")
+                }
+                _ => format!("git branch {r} HEAD"),
+            };
             if remoto {
                 o.push_str(&format!(" && git push {REMOTO} {r}"));
             }
-            persona(a, "con-historia", o)
+            let motivo = if hay_commits {
+                "con-historia"
+            } else {
+                "remoto-con-historia"
+            };
+            persona(a, motivo, o)
         } else {
             nuevas.push(r.to_owned());
             let mut a = a;
@@ -328,7 +380,37 @@ fn ramas(
             "git symbolic-ref HEAD refs/heads/develop && git reset -q".to_owned(),
         ));
     }
-    Ok(())
+    Ok(vacio)
+}
+
+/// Con historia, `init` deja la configuracion y `.gitignore` en el disco: la
+/// persona tiene que versionarlos (por PR) o quien clone no los tendra. Se
+/// dice si `init` los crea o cambia, o si siguen sin versionar.
+fn versionar(c: &Contexto<'_>, principal: &Path, v: &[Accion]) -> Result<Option<Accion>, Informe> {
+    const FICHEROS: [&str; 2] = [config::RUTA, ".gitignore"];
+    let los_toca = v
+        .iter()
+        .any(|a| matches!(a.clase, "config" | "gitignore") && a.decision == Decision::Crea);
+    let mut args = vec!["--porcelain", "--"];
+    args.extend(FICHEROS);
+    let sin_versionar = !exigir(c.git, principal, NOMBRE, Subcomando::Status, &args)?
+        .trim()
+        .is_empty();
+    Ok((los_toca || sin_versionar).then(|| {
+        persona(
+            Accion::nueva(
+                FICHEROS.join(","),
+                "versionar",
+                "proyecto",
+                Decision::Persona,
+            ),
+            "sin-versionar",
+            format!(
+                "git add -- {} && git commit -m \"chore: configura TaskCodeTwo\"",
+                FICHEROS.join(" ")
+            ),
+        )
+    }))
 }
 
 fn persona(mut a: Accion, motivo: &str, orden: String) -> Accion {
@@ -344,14 +426,15 @@ fn plataforma(
     clave: Option<&(&'static str, String)>,
 ) -> Result<Conexion, Informe> {
     let Some(url) = url_origin(c.git, principal, NOMBRE)? else {
-        return Ok(Err("sin-remoto"));
+        return Ok(Err("sin-remoto".to_owned()));
     };
-    let Ok(destino) = detectar(&url, clave.map(|(p, a)| (*p, a.as_str()))) else {
-        return Ok(Err("sin-plataforma"));
+    let destino = match detectar(&url, clave.map(|(p, a)| (*p, a.as_str()))) {
+        Ok(d) => d,
+        Err(i) => return Ok(Err(motivo_de_deteccion(&i))),
     };
     let credenciales = (c.credenciales)();
     if credenciales.token(Rol::Persona).is_none() {
-        return Ok(Err("sin-token"));
+        return Ok(Err("sin-token".to_owned()));
     }
     let p: Result<Box<dyn Plataforma>, Informe> = match destino.proveedor {
         Proveedor::GitHub => GitHub::nuevo(
@@ -369,8 +452,21 @@ fn plataforma(
         )
         .map(|x| Box::new(x) as Box<dyn Plataforma>),
     };
-    Ok(p.map(|p| (p, destino.proveedor))
-        .map_err(|_| "sin-plataforma"))
+    Ok(p.map(|p| (p, destino.proveedor)).map_err(|i| motivo_de(&i)))
+}
+
+/// Motivo concreto de un rechazo de `detectar` (su linea solo dice
+/// `motivo=plataforma` u `origin`; el porque esta en el mensaje): la clave
+/// del mensaje en minusculas y con guiones, p. ej. `plataforma-contradice`
+/// o `api-no-cambiable`. No lleva datos de la persona ni el token.
+fn motivo_de_deteccion(i: &Informe) -> String {
+    i.diagnostico
+        .as_ref()
+        .and_then(|m| m.clave().rsplit("::").next())
+        .map_or_else(
+            || "sin-plataforma".to_owned(),
+            |c| c.to_lowercase().replace('_', "-"),
+        )
 }
 
 /// Motivo de un informe de la plataforma para la linea `pendiente`.
@@ -397,7 +493,7 @@ fn personas_yml(principal: &Path, plataforma: &Conexion) -> Result<Accion, Infor
     }
     let (p, _) = match plataforma {
         Ok(p) => p,
-        Err(m) => return Ok(nueva(Decision::Pendiente).motivo(*m)),
+        Err(m) => return Ok(nueva(Decision::Pendiente).motivo(m.clone())),
     };
     let u = match p.identidad(Rol::Persona) {
         Ok(u) => u,
@@ -426,7 +522,7 @@ fn personas_yml(principal: &Path, plataforma: &Conexion) -> Result<Accion, Infor
 fn puertas(plataforma: &Conexion, v: &mut Vec<Accion>) {
     let leidas = match plataforma {
         Ok((p, prov)) => Ok((p.leer_puertas(Rol::Persona, "develop"), *prov)),
-        Err(m) => Err((*m).to_owned()),
+        Err(m) => Err(m.clone()),
     };
     for puerta in modelo() {
         let nombre = puerta.nombre_completo();
@@ -438,7 +534,7 @@ fn puertas(plataforma: &Conexion, v: &mut Vec<Accion>) {
             Ok((Ok(r), _)) if r.rulesets.contains(&nombre) => nueva(Decision::Existe),
             Ok((Ok(_), prov)) => {
                 let mut a = nueva(Decision::Persona)
-                    .motivo("sin-checks")
+                    .motivo("aplicar-con-task-118")
                     .orden(ORDEN_PUERTAS);
                 if *prov == Proveedor::GitHub {
                     a.propuesta = Some(ruleset_github(&puerta));
diff --git a/crates/tc2/src/init/mod.rs b/crates/tc2/src/init/mod.rs
index 059efb9..9ccf451 100644
--- a/crates/tc2/src/init/mod.rs
+++ b/crates/tc2/src/init/mod.rs
@@ -470,11 +470,11 @@ mod tests {
             "taskcodetwo",
             Decision::Persona,
         )
-        .motivo("sin-checks")
+        .motivo("aplicar-con-task-118")
         .orden(ORDEN_PUERTAS);
         assert_eq!(
             a.linea(),
-            "accion ruta=taskcodetwo-main clase=ruleset dueno=taskcodetwo decision=persona motivo=sin-checks orden=tc2%20init%20--aplicar-puertas"
+            "accion ruta=taskcodetwo-main clase=ruleset dueno=taskcodetwo decision=persona motivo=aplicar-con-task-118 orden=tc2%20init%20--aplicar-puertas"
         );
         a.propuesta = Some(serde_json::json!({"name": "taskcodetwo-main"}));
         let v: serde_json::Value = serde_json::from_str(&a.json()).unwrap_or_default();
diff --git a/crates/tc2/src/plataforma/puertas.rs b/crates/tc2/src/plataforma/puertas.rs
index c8ee03b..ee5c27b 100644
--- a/crates/tc2/src/plataforma/puertas.rs
+++ b/crates/tc2/src/plataforma/puertas.rs
@@ -250,16 +250,28 @@ pub fn ruleset_github(p: &Puerta) -> serde_json::Value {
             }
         }));
     }
-    json!({
+    // Sin `integration_id` cualquier identidad con permiso de escribir checks
+    // los daria por buenos (F1-5 lo descarta): la propuesta lleva una clave
+    // que GitHub no admite, asi que no se puede aplicar tal cual.
+    let nota = (!p.checks.is_empty()).then_some(NOTA_SIN_ORIGEN);
+    let mut v = json!({
         "name": p.nombre_completo(),
         "target": target,
         "enforcement": "active",
         "bypass_actors": [],
         "conditions": {"ref_name": {"include": include, "exclude": []}},
         "rules": reglas
-    })
+    });
+    if let (Some(n), Some(o)) = (nota, v.as_object_mut()) {
+        o.insert("_nota".to_owned(), n.into());
+    }
+    v
 }
 
+/// Nota de una propuesta con checks sin origen: no es aplicable.
+pub const NOTA_SIN_ORIGEN: &str =
+    "no aplicable: checks sin integration_id hasta F6-1; la aplica TASK-118";
+
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -311,7 +323,14 @@ mod tests {
             main["rules"][3]["parameters"]["strict_required_status_checks_policy"],
             false
         );
+        // Con checks sin origen, la propuesta no es aplicable tal cual.
+        assert_eq!(develop["_nota"], NOTA_SIN_ORIGEN);
+        assert!(
+            develop["rules"][3]["parameters"]["required_status_checks"][0]["integration_id"]
+                .is_null()
+        );
         let tareas = ruleset_github(&m[3]);
+        assert!(tareas.get("_nota").is_none());
         assert_eq!(
             tareas["conditions"]["ref_name"]["include"][0],
             "refs/heads/taskcodetwo/tareas"
diff --git a/docs/adr/F1-14-distribucion-e-instalacion.md b/docs/adr/F1-14-distribucion-e-instalacion.md
index cabe502..22ebc38 100644
--- a/docs/adr/F1-14-distribucion-e-instalacion.md
+++ b/docs/adr/F1-14-distribucion-e-instalacion.md
@@ -5,6 +5,7 @@
 - Requisitos: RR-16 (instalación fácil en cualquier ordenador) y RR-19 (progreso en vivo: la status line llama al CLI en cada refresco) de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); RR-12 (equipo) para la cadena de suministro; RF-14 (inventario de skills y agentes) para el manifiesto
 - Depende de: [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (git como proceso, contrato de salida y códigos), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (GitHub por HTTP con JWT de App: el binario lleva TLS), [ADR F1-8](F1-8-seguridad-modelo-de-amenazas.md) (el CLI en la zona Z1 de la máquina de la persona), [ADR F1-9](F1-9-formato-de-tarea-plan-y-especificacion.md) (YAML estándar que rechaza claves duplicadas), [ADR F1-10](F1-10-carriles-y-modo-automatico.md) (suelo en el binario que la config solo endurece), [ADR F1-11](F1-11-extensiones-declaradas.md) (decisión 10: comandos sí, hooks solo como atajo opcional; `verificacion` y revisoras) y [ADR F1-12](F1-12-actualizacion-entre-versiones.md) (nombres `tc2` y `taskcodetwo`, npm `taskcodetwo` libre, convivencia con la 0.16)
 - Decide: Carlos, con sus decisiones del 2026-11-14 (medir Rust frente a TypeScript con el mismo comando; manifiesto de recursos con cuatro niveles y engram no obligatorio; nombres de F1-12; comandos deterministas sobre hooks; no trocear; no descargar toolchains, targets ni imágenes: «haz la comparación con lo que ya está instalado») y la consigna «la opción recomendada» (D1a a D9a del plan de TASK-014)
+- Adenda (2026-11-17, TASK-028, D4 de su plan): la primera `tc2 init` (F3-1) **no** comprueba el manifiesto de recursos (punto 8). Esa comprobación llega con `tc2 doctor --recursos` (F3-8, con el inventario de F3-9), y `init` la llamará cuando exista.
 
 ## Contexto
 
diff --git a/docs/adr/F1-18-hito-de-decisiones.md b/docs/adr/F1-18-hito-de-decisiones.md
index edf0854..d574f58 100644
--- a/docs/adr/F1-18-hito-de-decisiones.md
+++ b/docs/adr/F1-18-hito-de-decisiones.md
@@ -5,6 +5,7 @@
 - Requisitos: RR-13 (actualizar proyectos existentes) y RR-12 (equipo) de [REQUISITOS-FUNCIONALES](../plan/REQUISITOS-FUNCIONALES.md); RR-14 (se construye con la 0.16) como precedente
 - Depende de: [ADR F1-11](F1-11-extensiones-declaradas.md) (inventario parche/propio, check `estado`, decisión 10 «comandos sí, hooks solo como atajo opcional») y [ADR F1-12](F1-12-actualizacion-entre-versiones.md) (informe de `tc2 actualizar --simular`, transacción con criterio CR1, huecos bloqueantes con código 5, decisión 8); usa [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) y [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (rama de estado y escritura atómica), [ADR F1-4](F1-4-operaciones-git-y-contrato-de-salida.md) (contrato de salida y códigos), [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (ninguna IA firma lo que firma una persona) y [ADR F1-10](F1-10-carriles-y-modo-automatico.md) (carril efectivo por ficheros, errata solo con `docs/`)
 - Decide: Carlos, con sus decisiones del 2026-11-14 (se llama «Hito de decisiones (ADR)», no «Hito 0»; obligatorio pero proporcional; comando determinista y hooks solo como atajo; nombres `tc2`, `taskcodetwo` y rama `taskcodetwo/tareas`; orden de pilotos de F1-12; no trocear; PoC con datos medidos) y la consigna «la opción recomendada» (D1a, D2a, D3a, D4a y D5a del plan de TASK-094)
+- Adenda (2026-11-17, TASK-028, D4 de su plan): la primera `tc2 init` (F3-1) **no** abre el hito de decisiones: no crea `hito-decisiones.yml` ni el informe de contexto de la decisión 2. Deja el repo listo (rama de estado, ramas, configuración y propuesta de puertas), y el contrato de esta ADR en `init` queda para una tarea posterior de la Fase 3, sin fecha todavía.
 
 ## Contexto
 
diff --git a/docs/adr/F1-5-puertas-en-github.md b/docs/adr/F1-5-puertas-en-github.md
index fa3e631..32d828a 100644
--- a/docs/adr/F1-5-puertas-en-github.md
+++ b/docs/adr/F1-5-puertas-en-github.md
@@ -365,7 +365,7 @@ ha aplicado ningún ruleset):
 
    | Operación | Comando dueño | Idempotencia y resultados |
    |---|---|---|
-   | `aplicar-puertas` | `init` (F3-1), con la identidad de la persona; exige administrar el repo | lee los rulesets `taskcode-*`; igual al modelo → `sin-cambios`; distinto → los actualiza; sin rulesets en el plan (403) → `rechazada` (3) con `motivo=plan`; se niega con un `integration_id` de relleno (no verificado: ninguna escritura en GitHub) |
+   | `aplicar-puertas` | `init` (F3-1), con la identidad de la persona; exige administrar el repo (adenda de TASK-028: F3-1 solo propone; la aplica TASK-118) | lee los rulesets `taskcode-*`; igual al modelo → `sin-cambios`; distinto → los actualiza; sin rulesets en el plan (403) → `rechazada` (3) con `motivo=plan`; se niega con un `integration_id` de relleno (no verificado: ninguna escritura en GitHub) |
    | `leer-puertas` | `doctor` (F3-8), `review` | lectura (`GET /rulesets` y `/rules/branches/{rama}`); compara con el modelo e informa lo que falta o sobra; 403 de plan → «puertas no impuestas por la plataforma» (PoC A) |
 
 2. **Reglas por rama** (cierra lo que dejaron F1-1 y F1-3):
@@ -548,7 +548,7 @@ entre el declarado y el mínimo por ficheros), no el que el agente declara.
   planes de pago.
 - **F2-6** implementa `aplicar-puertas` y `leer-puertas` y prueba en E2E,
   contra un repo de pruebas, todo lo marcado «no verificado».
-- **F3-1** (`init`) aplica las puertas o explica por qué no puede; **F3-8**
+- **F3-1** (`init`) propone las puertas y explica por qué no las aplica todavía (adenda de TASK-028); las aplica **TASK-118**; **F3-8**
   (`doctor`) las compara con el modelo.
 - **F6** implementa el workflow `carril` y el check `revision-ia`.
 - **Este repo no cambia** (RR-14): sin rulesets ni CODEOWNERS hasta el relevo;
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 2c3706b7cb68b2e455377ec33ab0766aae68db39..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-028/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-028/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-028/revision/informe-revision-1-code-quality-reviewer.md         |   93 +++++
 tareas/03-en-revision/TASK-028/revision/informe-revision-1-rust-wasm-reviewer.md            |  141 +++++++
 tareas/03-en-revision/TASK-028/revision/peticion-revision-1-code-quality-reviewer.md        |   83 ++++
 tareas/03-en-revision/TASK-028/revision/peticion-revision-1-rust-wasm-reviewer.md           | 2186 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-028/tarea.md                                    |   20 +-
 7 files changed, 2518 insertions(+), 5 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 2c3706b7cb68b2e455377ec33ab0766aae68db39..HEAD -- ":(literal)crates/tc2/tests/init.rs"
````

````
crates/tc2/tests/init.rs | 192 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-
 1 file changed, 191 insertions(+), 1 deletion(-)
````
