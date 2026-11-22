# Peticion de revision — TASK-052 (ronda 2)

- Tarea: TASK-052 — F6-6 Rondas incrementales
- Rama revisada: feature/task-052-f6-6-rondas-incrementales
- Rama base: develop
- Commit revisado (HEAD): 3df9643e52dc72400bde144775a4131f7e726ca0
- Fecha: 2026-11-22
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-052 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 8822e8ee1e21657de2fd037cc0d334b27fe58006 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (11 hallazgo(s)))

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

- tareas/03-en-revision/TASK-052/tarea.md
- tareas/03-en-revision/TASK-052/planificacion/
- .github/workflows/revision-ia.yml
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/hallazgos/mod.rs
- crates/tc2/src/hallazgos/orden.rs
- crates/tc2/src/hallazgos/ronda.rs
- crates/tc2/src/i18n/catalogo/hallazgos.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/hallazgos.rs
- docs/adr/F1-10-carriles-y-modo-automatico.md
- docs/adr/F1-16-lotes-desatendidos.md
- docs/adr/F1-8-seguridad-modelo-de-amenazas.md
- docs/ci/revision-ia.md
- xtask/src/workflows.rs

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 5 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | .github/workflows/revision-ia.yml:363 | informe-revision-1-code-quality-reviewer.md |

## Commits a revisar (git log 8822e8ee1e21657de2fd037cc0d334b27fe58006..HEAD)

````
3df9643 chore(TASK-052): estado de los hallazgos de la ronda 1
d032160 docs(TASK-052): limites de la ronda ligera y del ancla del cierre, checks read y merges de la base
d4ecb7a fix(TASK-052): workflow revision-ia con filter=all y diff desde tc2 hallazgos desde
cf116db fix(TASK-052): correcciones de la ronda 1 en tc2 hallazgos
b06cf7d chore(TASK-052): informes de la ronda 1 con cambios solicitados
76e3904 chore(TASK-052): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 8822e8ee1e21657de2fd037cc0d334b27fe58006..HEAD)

````diff
diff --git a/.github/workflows/revision-ia.yml b/.github/workflows/revision-ia.yml
index 6e72898..46dcfe1 100644
--- a/.github/workflows/revision-ia.yml
+++ b/.github/workflows/revision-ia.yml
@@ -284,7 +284,8 @@ jobs:
           unzip -p "$RUNNER_TEMP/informe.zip" informe-verificacion.json | head -c 65537 > "$RUNNER_TEMP/informe.json" || rm -f "$RUNNER_TEMP/informe.json"
       # F6-6: la ronda anterior es el ultimo check revision-ia de la App del
       # revisor en un commit anterior del PR (no la rama de estado, que
-      # escribe el agente). Se vuelcan en crudo los check-runs de cada commit,
+      # escribe el agente). Se vuelcan en crudo todos los check-runs de cada
+      # commit (filter=all: no solo el ultimo, que puede ser un incompleta),
       # del mas nuevo al mas viejo (GITHUB_TOKEN con checks: read), y decide
       # tc2; del elegido, sus paginas de anotaciones. Pasado el tope de
       # commits, falla (el check sale incompleta).
@@ -310,7 +311,7 @@ jobs:
           k=0
           for c in $(tail -n +2 "$RUNNER_TEMP/commits.txt"); do
             f="$dir/checks/$k.json"
-            gh api --method GET "repos/$REPO/commits/$c/check-runs" -f check_name=revision-ia -f per_page=100 > "$f"
+            gh api --method GET "repos/$REPO/commits/$c/check-runs" -f check_name=revision-ia -f filter=all -f per_page=100 > "$f"
             checks+=(--check-runs "$f")
             k=$((k + 1))
             linea=$(cd base && tc2 hallazgos anterior "${checks[@]}")
@@ -323,6 +324,7 @@ jobs:
           done
           anotaciones=()
           rango=("origin/$BASE...$HEAD_SHA")
+          desde=""
           if [ "$anterior" != ninguno ]; then
             for p in $(seq 1 30); do
               f="$dir/anotaciones/$p.json"
@@ -330,10 +332,18 @@ jobs:
               anotaciones+=(--anotaciones "$f")
               [ "$(jq 'length' "$f")" -eq 100 ] || break
             done
-            # Tras un rebase el commit anterior no es ancestro: el diff vuelve
-            # a base...head y los abiertos se arrastran igual.
-            if git -C base merge-base --is-ancestor "$sha" "$HEAD_SHA"; then rango=("$sha" "$HEAD_SHA"); fi
+            # El diff incremental sale del commit anterior sin lo que entro
+            # despues de la base (merge de la base o «Update branch»), que no
+            # es del PR: tc2 hallazgos desde. Tras un rebase (el commit
+            # anterior no es ancestro), o si esa mezcla tiene conflictos, el
+            # diff vuelve a base...head (falla en cerrado: se revisa todo) y
+            # los abiertos se arrastran igual.
+            linea=$(cd base && tc2 hallazgos desde --anterior "$sha" --head "$HEAD_SHA" --base "origin/$BASE")
+            echo "$linea"
+            desde=$(printf '%s\n' "$linea" | sed -nE 's/.* desde=([0-9a-f]{40}|[0-9a-f]{64})( .*)?$/\1/p')
+            if [ -n "$desde" ]; then rango=("$desde" "$HEAD_SHA"); fi
           fi
+          printf '%s' "$desde" > "$dir/desde.txt"
           git -c core.quotepath=false --literal-pathspecs -C base diff --name-only -z --no-renames "${rango[@]}" > "$dir/rutas.txt"
           tarea=()
           id=$(printf '%s' "$RAMA" | sed -nE 's#^(feature|fix|hotfix)/task-([0-9]{3,})-.*#TASK-\2#p')
@@ -354,13 +364,15 @@ jobs:
           salida="$RUNNER_TEMP/revision-ia"
           mkdir -p "$salida/prompts" "$salida/diffs"
           # F6-6: la ronda la decidio el paso anterior. En incremental el diff
-          # es el de lo cambiado desde el commit de la ronda anterior.
+          # es el de lo cambiado desde el commit de la ronda anterior, sin lo
+          # que entro de la base despues (el `desde` de ese paso).
           ronda="$salida/ronda/ronda.json"
           modo=$(jq -r '.modo' "$ronda")
           numero=$(jq -r '.ronda' "$ronda")
           anterior=$(jq -r '.sha_anterior // ""' "$ronda")
+          desde=$(cat "$salida/ronda/desde.txt")
           rango=("origin/$BASE...$HEAD_SHA")
-          if [ "$modo" = incremental ] && git -C base merge-base --is-ancestor "$anterior" "$HEAD_SHA"; then rango=("$anterior" "$HEAD_SHA"); fi
+          if [ "$modo" = incremental ] && [ -n "$desde" ]; then rango=("$desde" "$HEAD_SHA"); fi
           # Rutas literales separadas por NUL: sin -z git entrecomilla y escapa
           # lo no ASCII ("\303\251.rs") y la ruta no casaria con nada.
           git -c core.quotepath=false --literal-pathspecs -C base diff --name-only -z --no-renames "${rango[@]}" > "$salida/rutas.txt"
@@ -433,7 +445,11 @@ jobs:
               if [ "$modo" = incremental ]; then
                 echo "## Hallazgos abiertos de la ronda anterior"
                 echo
-                echo "Ronda $numero (F6-6): el diff de abajo es lo cambiado desde el commit $anterior de la ronda anterior. Estos hallazgos siguen abiertos; cierra solo los que el diff arregla (ver «Lo que devuelves»)."
+                if [ -n "$desde" ]; then
+                  echo "Ronda $numero (F6-6): el diff de abajo es lo cambiado desde el commit $anterior de la ronda anterior, sin lo que entro de la base despues. Estos hallazgos siguen abiertos; cierra solo los que el diff arregla (ver «Lo que devuelves»)."
+                else
+                  echo "Ronda $numero (F6-6): el diff de abajo es el de todo el PR contra $BASE (el commit $anterior de la ronda anterior ya no es ancestro, o no se pudo restar lo que entro de la base). Estos hallazgos siguen abiertos; cierra solo los que el diff arregla (ver «Lo que devuelves»)."
+                fi
                 echo
                 cat "$salida/abiertos/abiertos-$i.md"
                 echo
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index 622e481..bdfa00d 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -51,6 +51,10 @@ pub enum Subcomando {
     Merge,
     /// `git merge-base`
     MergeBase,
+    /// `git merge-tree` (solo `--write-tree`: escribe objetos, no mueve
+    /// ninguna ref ni toca el worktree ni el indice; el diff incremental de
+    /// la revision IA, F6-6)
+    MergeTree,
     /// `git pull`
     Pull,
     /// `git push`
@@ -98,6 +102,7 @@ impl Subcomando {
             Self::LsTree => "ls-tree",
             Self::Merge => "merge",
             Self::MergeBase => "merge-base",
+            Self::MergeTree => "merge-tree",
             Self::Pull => "pull",
             Self::Push => "push",
             Self::ReadTree => "read-tree",
diff --git a/crates/tc2/src/hallazgos/mod.rs b/crates/tc2/src/hallazgos/mod.rs
index fa929bb..8e26581 100644
--- a/crates/tc2/src/hallazgos/mod.rs
+++ b/crates/tc2/src/hallazgos/mod.rs
@@ -919,9 +919,11 @@ pub fn validar_con(
         }
         Ok(Ok(Value::Array(l))) => (l, Vec::new()),
         Ok(Ok(Value::Object(o))) => {
+            // Una clave que falta es una lista vacia (las dos igual): sin
+            // `hallazgos`, los cierres validos no se pierden.
             let lista = |k: &str| match o.get(k) {
                 Some(Value::Array(l)) => Some(l.clone()),
-                None if k == "cierres" => Some(Vec::new()),
+                None => Some(Vec::new()),
                 _ => None,
             };
             let solo_esas = o.keys().all(|k| k == "hallazgos" || k == "cierres");
@@ -1853,7 +1855,9 @@ pub struct InformeTarea {
 
 impl InformeTarea {
     /// El informe de la tarea `tarea` con lo leido del check de la ronda
-    /// `ronda`.
+    /// `ronda`. Nuevos, persisten y cerrados se cuentan como en el check: en
+    /// una ronda `ligera` (lo dice la linea `Ronda N (ligera)` de su resumen)
+    /// la ronda no sube y no hay nada nuevo, asi que todo abierto persiste.
     pub fn nuevo(
         tarea: &str,
         (pr, sha, ronda): (u64, &str, u32),
@@ -1862,10 +1866,11 @@ impl InformeTarea {
         aceptados: Vec<AceptadoDe>,
     ) -> Self {
         let (criticos, importantes, menores) = recuento(&aceptados);
+        let ligera = ronda::modo_de_resumen(resumen) == Some(ronda::Modo::Ligera);
         let abiertos = |nueva: bool| {
             aceptados
                 .iter()
-                .filter(|a| a.abierto() && (a.ronda >= ronda) == nueva)
+                .filter(|a| a.abierto() && (!ligera && a.ronda >= ronda) == nueva)
                 .count()
         };
         Self {
@@ -2470,7 +2475,7 @@ mod tests {
                 "respuesta:json-invalido",
             ),
             (
-                "```json\n{\"cierres\": []}\n```".to_owned(),
+                "```json\n{\"hallazgos\": [], \"cierres\": null}\n```".to_owned(),
                 "respuesta:json-invalido",
             ),
         ];
@@ -2489,6 +2494,11 @@ mod tests {
         );
         assert_eq!(v.motivos(), ["I1:id-repetido"]);
         assert_eq!(v.cierres.len(), 1);
+        // MEN-4 (rust) de la ronda 1: sin `hallazgos` (como sin `cierres`),
+        // es una lista vacia y el cierre valido no se pierde.
+        let solo_cierres = format!("```json\n{{\"cierres\": [{}]}}\n```", cierre("I1", 12));
+        let v = validar_con("r", &solo_cierres, &d, Some(&informe), &abiertos);
+        assert_eq!((v.cierres.len(), v.rechazados.len()), (1, 0), "{v:?}");
         // Sin informe, un abierto con paso_rojo no se cierra (falla en
         // cerrado); con el paso ya en verde (fmt), si.
         let v = validar_con("r", &con(&[cierre("F1", 5)]), &d, None, &abiertos);
@@ -2514,6 +2524,71 @@ mod tests {
         assert_eq!(v.aceptados.len(), 1);
     }
 
+    /// MEN-1 (rust) de la ronda 1: el informe de la tarea cuenta nuevos,
+    /// persisten y cerrados igual que el check, tambien en una ronda `ligera`
+    /// (la ronda no sube y no hay nada nuevo). El modo sale de la linea
+    /// `Ronda N (<modo>)` del resumen, que se lee de vuelta para cada modo.
+    #[test]
+    fn el_informe_de_la_tarea_cuenta_como_el_check() {
+        let menor = |id: &str, fichero: &str| {
+            let mut a = abierto(id, "MENOR", 8, EV_BORDE);
+            a.ronda = 2;
+            a.hallazgo.fichero = fichero.to_owned();
+            a
+        };
+        let verde = ronda::Anterior {
+            ronda: 2,
+            sha: "a".repeat(40),
+            dictamen: Dictamen::Verde,
+            abiertos: vec![menor("M1", "b.rs"), menor("M2", "c.rs")],
+        };
+        let sha = "b".repeat(40);
+        let cuenta = |v: &Veredicto| {
+            let resumen = resumen_veredicto(v, "");
+            assert_eq!(ronda::modo_de_resumen(&resumen), Some(v.modo), "{resumen}");
+            let i = InformeTarea::nuevo(
+                "TASK-001",
+                (1, &sha, v.ronda),
+                (v.dictamen.conclusion(), v.dictamen),
+                &resumen,
+                v.aceptados.clone(),
+            );
+            assert_eq!(
+                (i.nuevos, i.persisten, i.cerrados),
+                (v.nuevos, v.persisten, v.cerrados),
+                "{:?}",
+                v.modo
+            );
+        };
+        let d = ronda::decidir(Some(&verde), &["b.rs".to_owned()], 2, false);
+        assert_eq!(d.modo, ronda::Modo::Ligera);
+        let v = veredicto_ronda(&[], Some(&d));
+        assert_eq!((v.nuevos, v.persisten, v.cerrados), (0, 1, 1));
+        cuenta(&v);
+        // Incremental con un nuevo: ese cuenta como nuevo y M2 persiste.
+        let d = ronda::decidir(
+            Some(&verde),
+            &["b.rs".to_owned(), "x.rs".to_owned()],
+            2,
+            false,
+        );
+        let nuevo = validar("r", &una(BORDE), &diff(), None);
+        let v = veredicto_ronda(&[Some(nuevo.clone())], Some(&d));
+        assert_eq!(
+            (v.modo, v.nuevos, v.persisten),
+            (ronda::Modo::Incremental, 1, 1)
+        );
+        cuenta(&v);
+        // Completa (ronda 1) y tope.
+        cuenta(&veredicto_ronda(&[Some(nuevo)], None));
+        let mut rojo = verde.clone();
+        rojo.dictamen = Dictamen::Rojo;
+        rojo.abiertos
+            .push(abierto("I1", "IMPORTANTE", 12, EV_BORDE));
+        let d = ronda::decidir(Some(&rojo), &[], 2, false);
+        cuenta(&veredicto_ronda(&[], Some(&d)));
+    }
+
     /// F6-6, criterio 2: lo publicado por F6-5 (v2) se sigue leyendo: un
     /// `raw_details` sin ronda ni estado es ronda 1 y abierto, y el titulo
     /// sin `ronda=` es ronda 1. Un `tope` sin abiertos graves no casa.
diff --git a/crates/tc2/src/hallazgos/orden.rs b/crates/tc2/src/hallazgos/orden.rs
index 9757142..47bbe34 100644
--- a/crates/tc2/src/hallazgos/orden.rs
+++ b/crates/tc2/src/hallazgos/orden.rs
@@ -41,6 +41,10 @@
 //!   anterior=<sha12|ninguno> abiertos=N tope=T avisos=<...>`.
 //! - `abiertos --ronda <json> --detalle <json> --salida <dir>`: los abiertos
 //!   CRITICO e IMPORTANTE de cada revisora (`abiertos-<i>.json` y `.md`).
+//! - `desde --anterior <sha> --head <sha> --base <ref>`: desde donde se
+//!   saca el diff incremental, sin lo que entro de la base por un merge
+//!   ([`desde_incremental`]). Linea: `hallazgos ok desde=<oid>|base
+//!   motivo=<...>`.
 //! - `validar --abiertos <json>` valida tambien los cierres; `veredicto
 //!   --ronda <json>` cuenta nuevos, persisten y cerrados (la linea gana
 //!   `ronda=R modo=<m> nuevos=N persisten=P cerrados=C`).
@@ -197,6 +201,9 @@ pub fn orden(a: &[String], json: bool, idioma: Idioma) -> Salida {
         [s, resto @ ..] if s == "abiertos" => match abiertos(resto) {
             Ok(i) | Err(i) => i.salida_en(json, idioma),
         },
+        [s, resto @ ..] if s == "desde" => match desde(resto) {
+            Ok(i) | Err(i) => i.salida_en(json, idioma),
+        },
         [] => uso("sin-suborden").salida_en(json, idioma),
         _ => uso("suborden-desconocida").salida_en(json, idioma),
     }
@@ -422,7 +429,7 @@ fn anterior(a: &[String]) -> Result<Informe, Informe> {
     let dir = carpeta_actual()?;
     let (origen, _, avisos) = origen_y_config(origen, &crate::git::Git::nuevo(), &dir);
     let commits = leer_checks(&checks)?;
-    let elegido = origen.and_then(|o| super::ronda::elegir(commits, o));
+    let elegido = origen.and_then(|o| super::ronda::elegir(commits, o).check);
     let mut i = Informe::nuevo(NOMBRE, Resultado::Ok);
     i = match &elegido {
         Some(c) => i
@@ -502,8 +509,10 @@ fn ronda(a: &[String]) -> Result<Informe, Informe> {
         }
     }
     let commits = leer_checks(&checks)?;
-    let elegido = origen.and_then(|o| super::ronda::elegir(commits, o));
-    let previa = match &elegido {
+    let eleccion = origen
+        .map(|o| super::ronda::elegir(commits, o))
+        .unwrap_or_default();
+    let previa = match &eleccion.check {
         None => None,
         Some(c) => {
             let mut aceptados = Vec::new();
@@ -535,7 +544,7 @@ fn ronda(a: &[String]) -> Result<Informe, Informe> {
             Some(previa)
         }
     };
-    let d = super::ronda::decidir(previa.as_ref(), &cambiados, tope);
+    let d = super::ronda::decidir(previa.as_ref(), &cambiados, tope, eleccion.ligera_pendiente);
     std::fs::create_dir_all(&dir_salida).map_err(|e| fallo_io(&dir_salida, e.to_string()))?;
     let v = serde_json::to_value(&d).map_err(|e| fallo_io(&dir_salida, e.to_string()))?;
     escribir_json(&dir_salida.join("ronda.json"), &v)?;
@@ -553,6 +562,116 @@ fn ronda(a: &[String]) -> Result<Informe, Informe> {
         .con_lista("avisos", avisos))
 }
 
+/// Un SHA de commit (SHA-1 o SHA-256), en minusculas.
+fn es_sha(t: &str) -> bool {
+    matches!(t.len(), 40 | 64) && t.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
+}
+
+/// Una ref de la base (`origin/develop`): sin opciones ni rangos.
+fn es_ref(t: &str) -> bool {
+    !t.is_empty()
+        && t.len() <= 200
+        && !t.starts_with('-')
+        && !t.contains("..")
+        && t.chars()
+            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '_' | '-'))
+}
+
+/// Desde donde se saca el diff incremental de `anterior` a `head` (F6-6):
+/// sin lo que entro de la base en la rama (un merge de `base` o «Update
+/// branch») despues de la ronda anterior, que no es del PR.
+///
+/// - `anterior` no es ancestro de `head` (rebase): `None`, el diff vuelve a
+///   `base...head`;
+/// - el `merge-base` de `base` y `head` ya estaba en `anterior` (no entro
+///   nada de la base desde entonces): `anterior`;
+/// - si no, el arbol de `anterior` con lo que trajo la base
+///   (`git merge-tree --write-tree anterior <merge-base>`): el diff de ese
+///   arbol a `head` es solo lo del PR. Si esa mezcla tiene conflictos, o git
+///   falla, `None` (falla en cerrado: se revisa todo el PR).
+///
+/// Da tambien el motivo para la linea.
+pub fn desde_incremental(
+    g: &crate::git::Git,
+    dir: &Path,
+    base: &str,
+    anterior: &str,
+    head: &str,
+) -> (Option<String>, &'static str) {
+    use crate::git::Subcomando;
+    match crate::git::es_ancestro(g, dir, NOMBRE, anterior, head) {
+        Ok(true) => {}
+        Ok(false) => return (None, "no-ancestro"),
+        Err(_) => return (None, "git"),
+    }
+    let Ok(mb) = crate::git::exigir(g, dir, NOMBRE, Subcomando::MergeBase, &[base, head]) else {
+        return (None, "sin-merge-base");
+    };
+    let mb = mb.trim().to_owned();
+    if !es_sha(&mb) {
+        return (None, "sin-merge-base");
+    }
+    match crate::git::es_ancestro(g, dir, NOMBRE, &mb, anterior) {
+        Ok(true) => return (Some(anterior.to_owned()), "directo"),
+        Ok(false) => {}
+        Err(_) => return (None, "git"),
+    }
+    let Ok(s) = crate::git::correr(
+        g,
+        dir,
+        NOMBRE,
+        Subcomando::MergeTree,
+        &["--write-tree", "--no-messages", anterior, &mb],
+    ) else {
+        return (None, "git");
+    };
+    let arbol = s
+        .stdout
+        .lines()
+        .next()
+        .unwrap_or_default()
+        .trim()
+        .to_owned();
+    match s.codigo {
+        0 if es_sha(&arbol) => (Some(arbol), "merge"),
+        1 => (None, "conflicto"),
+        _ => (None, "git"),
+    }
+}
+
+/// `desde --anterior <sha> --head <sha> --base <ref>`
+/// ([`desde_incremental`]). Linea: `hallazgos ok desde=<oid>|base
+/// motivo=<directo|merge|no-ancestro|sin-merge-base|conflicto|git>`.
+fn desde(a: &[String]) -> Result<Informe, Informe> {
+    let (mut anterior, mut head, mut base): (Option<String>, Option<String>, Option<String>) =
+        (None, None, None);
+    let mut it = a.iter();
+    while let Some(x) = it.next() {
+        let mut valor = || it.next().cloned().ok_or_else(|| uso("falta-valor"));
+        match x.as_str() {
+            "--anterior" if anterior.is_none() => anterior = Some(valor()?),
+            "--head" if head.is_none() => head = Some(valor()?),
+            "--base" if base.is_none() => base = Some(valor()?),
+            _ => return Err(uso("argumento-desconocido")),
+        }
+    }
+    let (Some(anterior), Some(head), Some(base)) = (anterior, head, base) else {
+        return Err(uso("faltan-argumentos"));
+    };
+    if !es_sha(&anterior) || !es_sha(&head) {
+        return Err(uso("sha-invalido"));
+    }
+    if !es_ref(&base) {
+        return Err(uso("base-invalida"));
+    }
+    let dir = carpeta_actual()?;
+    let (desde, motivo) =
+        desde_incremental(&crate::git::Git::nuevo(), &dir, &base, &anterior, &head);
+    Ok(Informe::nuevo(NOMBRE, Resultado::Ok)
+        .con("desde", desde.unwrap_or_else(|| "base".to_owned()))
+        .con("motivo", motivo))
+}
+
 /// Una `ronda.json` del contrato vigente.
 fn leer_decision(p: &Path) -> Result<super::ronda::Decision, Informe> {
     let d: super::ronda::Decision = leer_json(p, "una ronda.json de tc2 hallazgos ronda")?;
diff --git a/crates/tc2/src/hallazgos/ronda.rs b/crates/tc2/src/hallazgos/ronda.rs
index 29ccf27..fadf48c 100644
--- a/crates/tc2/src/hallazgos/ronda.rs
+++ b/crates/tc2/src/hallazgos/ronda.rs
@@ -105,8 +105,8 @@ impl CheckRun {
 
 /// El check `revision-ia` de un commit que cuenta: el mas nuevo de ese
 /// nombre que publico la App `origen` (la API los da del mas nuevo al mas
-/// viejo), y solo si ya termino. Lo comparten `leer-revision` (F6-5) y la
-/// ronda siguiente (F6-6).
+/// viejo), y solo si ya termino. Es la regla de `leer-revision` (F6-5); la
+/// ronda siguiente (F6-6) mira todos los del commit ([`elegir`]).
 pub fn revision_de_origen(runs: Vec<CheckRun>, origen: u64) -> Option<CheckRun> {
     runs.into_iter()
         .find(|r| {
@@ -115,20 +115,76 @@ pub fn revision_de_origen(runs: Vec<CheckRun>, origen: u64) -> Option<CheckRun>
         .filter(|r| r.status == "completed" && r.id != 0)
 }
 
+/// El modo de una ronda leido del resumen de su check: la linea `Ronda N
+/// (<modo>): ...` que escribe [`super::resumen_veredicto`]. El texto del
+/// modelo no puede fingirla: va por [`super::plano`], sin saltos de linea, y
+/// siempre dentro de una fila de tabla o de un elemento de lista.
+pub fn modo_de_resumen(resumen: &str) -> Option<Modo> {
+    resumen.lines().find_map(|l| {
+        let resto = l.strip_prefix("Ronda ")?;
+        let (numero, resto) = resto.split_once(" (")?;
+        if numero.is_empty() || !numero.bytes().all(|b| b.is_ascii_digit()) {
+            return None;
+        }
+        let (modo, _) = resto.split_once("): ")?;
+        [Modo::Completa, Modo::Incremental, Modo::Ligera, Modo::Tope]
+            .into_iter()
+            .find(|m| m.texto() == modo)
+    })
+}
+
+/// Lo que da [`elegir`].
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct Eleccion {
+    /// El check que sirve de ronda anterior.
+    pub check: Option<CheckRun>,
+    /// Si entre el head y ese check hubo una ronda `ligera`: su codigo no lo
+    /// vio ningun modelo, asi que la ronda siguiente no puede ser otra
+    /// `ligera`.
+    pub ligera_pendiente: bool,
+}
+
 /// El check que sirve de ronda anterior: recorriendo los commits del PR del
-/// mas nuevo al mas viejo (sin el head), el primero cuyo `revision-ia` de la
-/// App `origen` dice `rojo` o `verde` en su titulo. Una ronda `incompleta`
-/// no cuenta ni sirve de base, y una `tope` tampoco (no reviso nada; lleva
-/// los mismos abiertos que la ronda que la origino).
-pub fn elegir(commits: Vec<Vec<CheckRun>>, origen: u64) -> Option<CheckRun> {
-    commits.into_iter().find_map(|runs| {
-        revision_de_origen(runs, origen).filter(|r| {
-            matches!(
+/// mas nuevo al mas viejo (sin el head) y, en cada commit, sus check-runs
+/// `revision-ia` terminados de la App `origen` del mas nuevo al mas viejo
+/// (por `id`), el primero cuyo titulo dice `rojo` o `verde` y que no fue una
+/// ronda `ligera`. Una ronda `incompleta` no cuenta ni sirve de base (aunque
+/// sea la ultima de su commit: se mira la anterior del mismo commit), una
+/// `tope` tampoco (no reviso nada; lleva los mismos abiertos que la ronda que
+/// la origino) y una `ligera` tampoco: no la vio ningun modelo, y la base
+/// del diff incremental sigue siendo el ultimo commit revisado por modelo.
+pub fn elegir(commits: Vec<Vec<CheckRun>>, origen: u64) -> Eleccion {
+    let mut ligera_pendiente = false;
+    for mut runs in commits {
+        runs.retain(|r| {
+            r.name == crate::revision::REVISION_IA
+                && r.app.as_ref().is_some_and(|a| a.id == origen)
+                && r.status == "completed"
+                && r.id != 0
+        });
+        runs.sort_by_key(|r| std::cmp::Reverse(r.id));
+        for r in runs {
+            if !matches!(
                 Dictamen::de_titulo(r.titulo()),
                 Some(Dictamen::Rojo | Dictamen::Verde)
-            )
-        })
-    })
+            ) {
+                continue;
+            }
+            let resumen = r.output.as_ref().and_then(|o| o.summary.as_deref());
+            if resumen.and_then(modo_de_resumen) == Some(Modo::Ligera) {
+                ligera_pendiente = true;
+                continue;
+            }
+            return Eleccion {
+                check: Some(r),
+                ligera_pendiente,
+            };
+        }
+    }
+    Eleccion {
+        check: None,
+        ligera_pendiente,
+    }
 }
 
 /// La ronda anterior, leida de su check y de sus anotaciones.
@@ -165,18 +221,25 @@ pub fn anterior(check: &CheckRun, aceptados: Vec<AceptadoDe>) -> Option<Anterior
 
 /// La ronda siguiente a `anterior` (F6-6, decisiones 4 a 6 del plan de
 /// TASK-052), con `cambiados` los ficheros del diff desde el commit
-/// anterior y `tope` el tope efectivo de rondas:
+/// anterior, `tope` el tope efectivo de rondas y `ligera_pendiente` si desde
+/// ese commit ya hubo una ronda `ligera` ([`Eleccion`]):
 ///
 /// - sin anterior: ronda 1, `completa`;
-/// - anterior `verde` y los cambios solo tocan ficheros con MENOR abiertos:
-///   `ligera` (sin modelo; la ronda no sube; los MENOR de ficheros
-///   cambiados se cierran);
+/// - anterior `verde`, sin otra `ligera` desde el y los cambios solo tocan
+///   ficheros con MENOR abiertos: `ligera` (sin modelo; la ronda no sube;
+///   los MENOR de ficheros cambiados se cierran). No sirve de base: lo que
+///   entro en ella lo ve la ronda siguiente, que ya no puede ser `ligera`;
 /// - anterior `rojo` y la ronda siguiente pasa de `tope`: `tope` (sin
 ///   modelo; los abiertos se arrastran todos);
 /// - si no, `incremental`: la ronda sube, los MENOR de ficheros cambiados
 ///   se cierran (no pasan por el cierre del modelo) y los CRITICO e
 ///   IMPORTANTE siguen abiertos hasta un cierre valido.
-pub fn decidir(anterior: Option<&Anterior>, cambiados: &[String], tope: u32) -> Decision {
+pub fn decidir(
+    anterior: Option<&Anterior>,
+    cambiados: &[String],
+    tope: u32,
+    ligera_pendiente: bool,
+) -> Decision {
     let tope = tope.max(1);
     let Some(a) = anterior else {
         return Decision {
@@ -197,6 +260,7 @@ pub fn decidir(anterior: Option<&Anterior>, cambiados: &[String], tope: u32) ->
         .map(|x| x.hallazgo.fichero.as_str())
         .collect();
     let (modo, ronda) = if a.dictamen == Dictamen::Verde
+        && !ligera_pendiente
         && cambiados.iter().all(|c| menores.contains(c.as_str()))
     {
         (Modo::Ligera, a.ronda)
@@ -274,17 +338,45 @@ pub fn abiertos_markdown(abiertos: &[AceptadoDe]) -> String {
     s
 }
 
-/// Cuantas rondas extra pide la tarea: los elementos de lista de su seccion
-/// `## Rondas extra` (F1-16; uno por ronda).
+/// Cuantas rondas extra pide la tarea: los elementos de lista de primer
+/// nivel (`- ...` o `* ...`, con texto) de sus secciones `## Rondas extra` y
+/// `## Ronda extra` (F1-16; uno por ronda). Lo que va dentro de un bloque de
+/// codigo (``` o ~~~) no cuenta.
 pub fn rondas_extra(cuerpo: &crate::tarea::Cuerpo) -> u32 {
-    cuerpo.seccion("Rondas extra").map_or(0, |s| {
-        let n = s
-            .texto
-            .lines()
-            .filter(|l| l.starts_with("- ") || l.starts_with("* "))
-            .count();
-        u32::try_from(n).unwrap_or(u32::MAX)
-    })
+    let n: usize = ["Rondas extra", "Ronda extra"]
+        .into_iter()
+        .filter_map(|t| cuerpo.seccion(t))
+        .map(|s| elementos_de_lista(&s.texto))
+        .sum();
+    u32::try_from(n).unwrap_or(u32::MAX)
+}
+
+/// Elementos de lista de primer nivel con texto, fuera de los bloques de
+/// codigo.
+fn elementos_de_lista(texto: &str) -> usize {
+    let mut valla: Option<&str> = None;
+    let mut n = 0;
+    for l in texto.lines() {
+        let t = l.trim_start();
+        let abre = ["```", "~~~"].into_iter().find(|v| t.starts_with(v));
+        match (valla, abre) {
+            (None, Some(v)) => {
+                valla = Some(v);
+                continue;
+            }
+            (Some(v), Some(w)) if v == w => {
+                valla = None;
+                continue;
+            }
+            (Some(_), _) => continue,
+            (None, None) => {}
+        }
+        let item = l.strip_prefix("- ").or_else(|| l.strip_prefix("* "));
+        if item.is_some_and(|x| !x.trim().is_empty()) {
+            n += 1;
+        }
+    }
+    n
 }
 
 #[cfg(test)]
@@ -322,7 +414,7 @@ mod tests {
     #[test]
     fn decidir_aplica_tope_ligera_e_incremental() {
         // Sin anterior (o con una incompleta, que `elegir` salta): ronda 1.
-        let d = decidir(None, &rutas(&["x"]), 2);
+        let d = decidir(None, &rutas(&["x"]), 2, false);
         assert_eq!((d.ronda, d.modo, d.sha_anterior), (1, Modo::Completa, None));
         // Rojo en la ronda 2 con tope 2: tope, sin modelo, con todo arrastrado
         // (tambien el MENOR de un fichero cambiado).
@@ -331,12 +423,12 @@ mod tests {
             2,
             vec![a("I1", "IMPORTANTE", "a.rs"), a("M1", "MENOR", "b.rs")],
         );
-        let d = decidir(Some(&rojo), &rutas(&["a.rs", "b.rs"]), 2);
+        let d = decidir(Some(&rojo), &rutas(&["a.rs", "b.rs"]), 2, false);
         assert_eq!((d.ronda, d.modo, d.tope), (3, Modo::Tope, 2));
         assert_eq!(ids(&d.abiertos), ["I1", "M1"]);
         assert!(d.cerrados.is_empty());
         // Una ronda extra en la tarea sube el tope: incremental.
-        let d = decidir(Some(&rojo), &rutas(&["a.rs", "b.rs"]), 3);
+        let d = decidir(Some(&rojo), &rutas(&["a.rs", "b.rs"]), 3, false);
         assert_eq!((d.ronda, d.modo), (3, Modo::Incremental));
         assert_eq!(d.sha_anterior.as_deref(), Some("a".repeat(40).as_str()));
         // El MENOR de un fichero cambiado se cierra sin modelo; el grave sigue.
@@ -349,13 +441,38 @@ mod tests {
             2,
             vec![a("M1", "MENOR", "b.rs"), a("M2", "MENOR", "c.rs")],
         );
-        let d = decidir(Some(&verde), &rutas(&["b.rs"]), 2);
+        let d = decidir(Some(&verde), &rutas(&["b.rs"]), 2, false);
         assert_eq!((d.ronda, d.modo), (2, Modo::Ligera));
         assert_eq!(ids(&d.cerrados), ["M1"]);
         assert_eq!(ids(&d.abiertos), ["M2"]);
         // Un cambio fuera de ellos: incremental (y en verde no hay tope).
-        let d = decidir(Some(&verde), &rutas(&["b.rs", "otro.rs"]), 2);
+        let d = decidir(Some(&verde), &rutas(&["b.rs", "otro.rs"]), 2, false);
+        assert_eq!((d.ronda, d.modo), (3, Modo::Incremental));
+    }
+
+    /// IMP-1 (rust) y MEN-3 (calidad) de la ronda 1: si desde la ronda con
+    /// modelo ya hubo una `ligera`, la siguiente no puede ser otra `ligera`
+    /// (lo que entro en ella no lo vio ningun modelo): es `incremental` desde
+    /// el commit revisado por modelo.
+    #[test]
+    fn tras_una_ligera_la_ronda_siguiente_va_con_modelo() {
+        let verde = previa(
+            Dictamen::Verde,
+            2,
+            vec![a("M1", "MENOR", "b.rs"), a("M2", "MENOR", "c.rs")],
+        );
+        let d = decidir(Some(&verde), &rutas(&["b.rs"]), 2, true);
         assert_eq!((d.ronda, d.modo), (3, Modo::Incremental));
+        assert_eq!(d.sha_anterior.as_deref(), Some("a".repeat(40).as_str()));
+        assert_eq!(ids(&d.cerrados), ["M1"]);
+    }
+
+    fn check_con(id: u64, titulo: &str, resumen: &str) -> CheckRun {
+        let mut c = check(id, 7, titulo, "completed");
+        if let Some(o) = c.output.as_mut() {
+            o.summary = Some(resumen.to_owned());
+        }
+        c
     }
 
     fn check(id: u64, app: u64, titulo: &str, estado: &str) -> CheckRun {
@@ -390,8 +507,8 @@ mod tests {
             )],
             vec![check(1, 7, rojo, "completed")],
         ];
-        assert_eq!(elegir(commits.clone(), 7).map(|c| c.id), Some(1));
-        assert_eq!(elegir(commits, 8), None);
+        assert_eq!(elegir(commits.clone(), 7).check.map(|c| c.id), Some(1));
+        assert_eq!(elegir(commits, 8), Eleccion::default());
         // `anterior` exige que el check case con sus anotaciones.
         let c = check(1, 7, rojo, "completed");
         let i1 = a("I1", "IMPORTANTE", "a.rs");
@@ -403,6 +520,68 @@ mod tests {
         assert_eq!(anterior(&c, vec![cerrado]), None);
     }
 
+    /// MEN-1 (calidad) de la ronda 1: con varios checks en el mismo commit,
+    /// cuenta el mas nuevo (por `id`, venga en el orden que venga) que no sea
+    /// `incompleta`: un rojo seguido de un incompleta en el mismo commit sigue
+    /// siendo la ronda anterior.
+    #[test]
+    fn en_un_commit_cuenta_el_mas_nuevo_que_no_es_incompleta() {
+        let rojo = "revision-ia: rojo (ronda=1 criticos=0 importantes=1 menores=0)";
+        let verde = "revision-ia: verde (ronda=1 criticos=0 importantes=0 menores=0)";
+        let incompleta = "revision-ia: incompleta (sin saldo)";
+        for runs in [
+            vec![
+                check(9, 7, incompleta, "completed"),
+                check(8, 7, rojo, "completed"),
+            ],
+            vec![
+                check(8, 7, rojo, "completed"),
+                check(9, 7, incompleta, "completed"),
+            ],
+        ] {
+            assert_eq!(elegir(vec![runs], 7).check.map(|c| c.id), Some(8));
+        }
+        // Dos completos en el mismo commit: el mas nuevo.
+        let runs = vec![
+            check(8, 7, rojo, "completed"),
+            check(10, 7, verde, "completed"),
+        ];
+        assert_eq!(elegir(vec![runs], 7).check.map(|c| c.id), Some(10));
+    }
+
+    /// IMP-1 (rust) de la ronda 1: una ronda `ligera` (lo dice la linea
+    /// `Ronda N (ligera)` de su resumen) no sirve de ronda anterior: se elige
+    /// el check con modelo de antes y se avisa de que hubo una ligera.
+    #[test]
+    fn una_ligera_no_sirve_de_ronda_anterior() {
+        let verde = "revision-ia: verde (ronda=2 criticos=0 importantes=0 menores=1)";
+        let ligera = check_con(
+            6,
+            verde,
+            "**Veredicto: verde** (conclusion success).\n\nRonda 2 (ligera): 0 nuevos, 1 persisten, 1 cerrados.\n",
+        );
+        let con_modelo = check_con(
+            5,
+            verde,
+            "**Veredicto: verde**.\n\nRonda 2 (incremental): 0 nuevos, 2 persisten, 1 cerrados.\n",
+        );
+        let e = elegir(vec![vec![ligera.clone()], vec![con_modelo.clone()]], 7);
+        assert_eq!(e.check.map(|c| c.id), Some(5));
+        assert!(e.ligera_pendiente);
+        let e = elegir(vec![vec![con_modelo]], 7);
+        assert_eq!(e.check.map(|c| c.id), Some(5));
+        assert!(!e.ligera_pendiente);
+        // El texto del modelo no finge la linea: va en un elemento de lista.
+        let fingida = check_con(
+            4,
+            verde,
+            "Ronda 2 (incremental): 0 nuevos.\n\n- Ronda 2 (ligera): 0 nuevos\n",
+        );
+        assert_eq!(elegir(vec![vec![fingida]], 7).check.map(|c| c.id), Some(4));
+        assert_eq!(modo_de_resumen("Ronda x (ligera): 0"), None);
+        assert_eq!(modo_de_resumen("Ronda 3 (otra): 0"), None);
+    }
+
     /// Cada abierto grave va a su revisora si esta elegida, o a la que tiene
     /// su fichero; los MENOR no se reparten. `## Rondas extra` cuenta los
     /// elementos de primer nivel.
@@ -439,4 +618,20 @@ mod tests {
         assert_eq!(rondas_extra(&cuerpo), 2);
         assert_eq!(rondas_extra(&crate::tarea::Cuerpo::leer("## Otra\n")), 0);
     }
+
+    /// MEN-4 (calidad) y MEN-2 (rust) de la ronda 1: `## Rondas extra` cuenta
+    /// los elementos `- ` y `* ` con texto, fuera de los bloques de codigo, y
+    /// `## Ronda extra` tambien vale (F1-16 nombra las dos).
+    #[test]
+    fn rondas_extra_cuenta_solo_elementos_con_texto() {
+        let n = |t: &str| rondas_extra(&crate::tarea::Cuerpo::leer(t));
+        assert_eq!(n("## Rondas extra\n\n* Ronda 3\n* Ronda 4\n"), 2);
+        assert_eq!(n("## Rondas extra\n\n- \n-\n*   \n- Ronda 3\n"), 1);
+        assert_eq!(
+            n("## Rondas extra\n\n```\n- ejemplo\n- otro\n```\n~~~\n* x\n~~~\n- Ronda 3\n"),
+            1
+        );
+        assert_eq!(n("## Ronda extra\n\n- a\n"), 1);
+        assert_eq!(n("## Ronda extra\n\n- a\n\n## Rondas extra\n\n- b\n"), 2);
+    }
 }
diff --git a/crates/tc2/src/i18n/catalogo/hallazgos.rs b/crates/tc2/src/i18n/catalogo/hallazgos.rs
index 7e24f3f..f338129 100644
--- a/crates/tc2/src/i18n/catalogo/hallazgos.rs
+++ b/crates/tc2/src/i18n/catalogo/hallazgos.rs
@@ -2,8 +2,8 @@
 
 crate::i18n::textos! {
     USO_HALLAZGOS {
-        es: "uso: hallazgos [--json] validar --respuesta <fichero> --diff <fichero> --revisora <id> [--informe <informe.json>] [--abiertos <abiertos.json>] [--salida <salida.json>] [--resumen <salida.md>] | hallazgos [--json] veredicto --salida <carpeta> [--hallazgos <salida-de-validar.json>]... [--extra <lineas.txt>] [--ronda <ronda.json>] | hallazgos [--json] anterior --check-runs <json>... [--origen <app-id>] | hallazgos [--json] ronda --check-runs <json>... [--anotaciones <json>]... --rutas <fichero-nul> [--tarea TASK-NNN] [--origen <app-id>] --salida <carpeta> | hallazgos [--json] abiertos --ronda <ronda.json> --detalle <detalle.json> --salida <carpeta> | hallazgos [--json] instrucciones",
-        en: "usage: hallazgos [--json] validar --respuesta <file> --diff <file> --revisora <id> [--informe <report.json>] [--abiertos <open.json>] [--salida <output.json>] [--resumen <output.md>] | hallazgos [--json] veredicto --salida <folder> [--hallazgos <validar-output.json>]... [--extra <lines.txt>] [--ronda <ronda.json>] | hallazgos [--json] anterior --check-runs <json>... [--origen <app-id>] | hallazgos [--json] ronda --check-runs <json>... [--anotaciones <json>]... --rutas <nul-file> [--tarea TASK-NNN] [--origen <app-id>] --salida <folder> | hallazgos [--json] abiertos --ronda <ronda.json> --detalle <detail.json> --salida <folder> | hallazgos [--json] instrucciones",
+        es: "uso: hallazgos [--json] validar --respuesta <fichero> --diff <fichero> --revisora <id> [--informe <informe.json>] [--abiertos <abiertos.json>] [--salida <salida.json>] [--resumen <salida.md>] | hallazgos [--json] veredicto --salida <carpeta> [--hallazgos <salida-de-validar.json>]... [--extra <lineas.txt>] [--ronda <ronda.json>] | hallazgos [--json] anterior --check-runs <json>... [--origen <app-id>] | hallazgos [--json] ronda --check-runs <json>... [--anotaciones <json>]... --rutas <fichero-nul> [--tarea TASK-NNN] [--origen <app-id>] --salida <carpeta> | hallazgos [--json] abiertos --ronda <ronda.json> --detalle <detalle.json> --salida <carpeta> | hallazgos [--json] desde --anterior <sha> --head <sha> --base <ref> | hallazgos [--json] instrucciones",
+        en: "usage: hallazgos [--json] validar --respuesta <file> --diff <file> --revisora <id> [--informe <report.json>] [--abiertos <open.json>] [--salida <output.json>] [--resumen <output.md>] | hallazgos [--json] veredicto --salida <folder> [--hallazgos <validar-output.json>]... [--extra <lines.txt>] [--ronda <ronda.json>] | hallazgos [--json] anterior --check-runs <json>... [--origen <app-id>] | hallazgos [--json] ronda --check-runs <json>... [--anotaciones <json>]... --rutas <nul-file> [--tarea TASK-NNN] [--origen <app-id>] --salida <folder> | hallazgos [--json] abiertos --ronda <ronda.json> --detalle <detail.json> --salida <folder> | hallazgos [--json] desde --anterior <sha> --head <sha> --base <ref> | hallazgos [--json] instrucciones",
     }
     NO_SE_LEE {
         es: "no se puede leer ni escribir {ruta}: {error}; comprueba la ruta y los permisos y vuelve a lanzar la orden",
@@ -18,7 +18,7 @@ crate::i18n::textos! {
         en: "{ruta} is not {que}: {error}; pass the file exactly as the previous workflow step (or the GitHub API) left it, unedited, and run the command again",
     }
     ANTERIOR_INCOHERENTE {
-        es: "el check revision-ia {check} del commit {sha} no casa con sus anotaciones (titulo, conclusion y hallazgos abiertos): falta una pagina de anotaciones o el check esta alterado; vuelve a lanzar el workflow y, si sigue, revisa ese check en GitHub antes de seguir (la ronda no se calcula sin el: falla en cerrado)",
-        en: "the revision-ia check {check} on commit {sha} does not match its annotations (title, conclusion and open findings): an annotations page is missing or the check was altered; re-run the workflow and, if it persists, inspect that check on GitHub before going on (the round is not computed without it: fail closed)",
+        es: "el check revision-ia {check} del commit {sha} no casa con sus anotaciones (titulo, conclusion y hallazgos abiertos): o no se leyo una pagina de anotaciones, o se publico a medias (fallo un lote de anotaciones), o esta alterado. La ronda no se calcula sin el (falla en cerrado). Primero vuelve a lanzar este workflow (si fue la lectura, pasa). Si sigue, vuelve a lanzar (Re-run) el workflow revision-ia del commit {sha}: publica en ese commit un check completo, que pasa delante del roto porque de cada commit cuenta el mas nuevo que no sea incompleta; despues, vuelve a lanzar este. Si el check esta alterado, revisalo en GitHub antes de seguir",
+        en: "the revision-ia check {check} on commit {sha} does not match its annotations (title, conclusion and open findings): an annotations page was not read, it was published halfway (an annotations batch failed), or it was altered. The round is not computed without it (fail closed). First re-run this workflow (if it was the read, it passes). If it persists, re-run the revision-ia workflow of commit {sha}: it publishes a complete check on that commit, which takes precedence over the broken one because on each commit the newest one that is not incompleta counts; then re-run this one. If the check was altered, inspect it on GitHub before going on",
     }
 }
diff --git a/docs/adr/F1-10-carriles-y-modo-automatico.md b/docs/adr/F1-10-carriles-y-modo-automatico.md
index 270e09e..d44e8bd 100644
--- a/docs/adr/F1-10-carriles-y-modo-automatico.md
+++ b/docs/adr/F1-10-carriles-y-modo-automatico.md
@@ -751,11 +751,13 @@ Evidencia: `crates/tc2/tests/aprobar.rs` y los unitarios de
 ## Adenda (F6-6, TASK-052) — `tope` del revisor IA en CI
 
 - `tope_rondas` ya lo usa la CI: `tc2 hallazgos ronda` toma `tope_rondas` de
-  la configuración de la base más un elemento por cada línea `- ...` de
-  primer nivel en `## Rondas extra` de la tarea. Pasado ese tope con un
+  la configuración de la base más uno por cada elemento de lista de primer
+  nivel con texto en `## Rondas extra` (o `## Ronda extra`) de la tarea. Pasado ese tope con un
   CRITICO o IMPORTANTE abierto, el check `revision-ia` sale `tope`
   (`failure`) sin consultar al modelo y con los abiertos arrastrados. Es la
   señal en CI de la parada `preguntar` de esta ADR: la traducción a esa
   parada en el modo automático es de F4-4. Con el anterior en verde y cambios
   solo en ficheros con MENOR abiertos, la ronda es `ligera` (sin modelo,
-  verde, la ronda no sube): los MENORES no gastan ronda.
+  verde, la ronda no sube): los MENORES no gastan ronda. Una `ligera` no
+  sirve de base del diff siguiente y no se encadena con otra: lo que entró
+  en ella lo ve el modelo en la ronda siguiente.
diff --git a/docs/adr/F1-16-lotes-desatendidos.md b/docs/adr/F1-16-lotes-desatendidos.md
index ed53e5f..b2de225 100644
--- a/docs/adr/F1-16-lotes-desatendidos.md
+++ b/docs/adr/F1-16-lotes-desatendidos.md
@@ -936,8 +936,9 @@ taskcodetwo/tareas: lotes/noche-173-210.yml (diario), tareas/TASK-NNN/tarea.md (
 ## Adenda (F6-6, TASK-052) — `## Rondas extra` sube el tope de la CI
 
 - El formato que lee `tc2 hallazgos ronda` es un elemento de lista de primer
-  nivel (`- ...` o `* ...`) por ronda extra en `## Rondas extra` de la tarea
-  (rama de estado); los subelementos sangrados no cuentan. Cada uno suma uno
+  nivel con texto (`- ...` o `* ...`) por ronda extra en `## Rondas extra` o
+  `## Ronda extra` de la tarea (rama de estado); los subelementos sangrados,
+  los elementos vacíos y lo que va dentro de un bloque de código no cuentan. Cada uno suma uno
   a `tope_rondas` de la base. Con D6a nadie escribe esa sección de noche: la
   ronda extra la decide la persona por la mañana. Si el agente la escribiera,
   solo ganaría rondas (tokens), no saltaría ninguna puerta: la aprobación de
diff --git a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
index 43e16e2..ae35466 100644
--- a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
+++ b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
@@ -831,8 +831,9 @@ Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
   `revision-ia` de la ronda anterior con el `GITHUB_TOKEN` (`checks: read`,
   lectura ya admitida por la lista blanca de la decisión 1 para `buscar` de
   F5-2). El llamador tiene que darlo (un reutilizable no gana permisos que
-  el llamador no da); sin él, el paso «Ronda anterior» falla y el check sale
-  `incompleta` (falla en cerrado). El lint del llamador (`llamador-revision`)
+  el llamador no da); sin él, GitHub rechaza el run entero al validarlo y no
+  se publica ningún check `revision-ia` (falla en cerrado; no verificado aún
+  en un run real). El lint del llamador (`llamador-revision`)
   admite `checks: read` y rechaza `checks: write`. Ningún job nuevo.
 - **La ronda anterior sale del check de la App del revisor**, filtrado por
   su `app.id` (`plataforma.revisor_id` de la base), no del informe
diff --git a/docs/ci/revision-ia.md b/docs/ci/revision-ia.md
index 1ef537b..44f3bb5 100644
--- a/docs/ci/revision-ia.md
+++ b/docs/ci/revision-ia.md
@@ -144,9 +144,15 @@ jobs:
       base: ${{ github.event.workflow_run.pull_requests[0].base.ref }}
 ```
 
-Sin `checks: read` en el llamador, `preparar` no puede pedirlo (un
-reutilizable no gana permisos que el llamador no da): el paso «Ronda
-anterior» falla y el check sale `incompleta` (falla en cerrado).
+Sin `checks: read` en el llamador, el reutilizable no puede pedirlo (no gana
+permisos que el llamador no da) y GitHub rechaza el run entero al validarlo,
+antes de arrancar ningún job: el workflow de revisión sale en rojo con un
+error de permisos («... is requesting 'checks: read', but is only allowed
+'checks: none'») y **no se publica ningún check `revision-ia`**, ni siquiera
+`incompleta` (falla en cerrado: si `revision-ia` es un check requerido, el PR
+no se puede integrar). Un llamador que ya fijaba una versión anterior por SHA
+tiene que añadir `checks: read` al subir el SHA. No verificado aún en un run
+real.
 
 `workflows: [verificacion]` es el `name:` del workflow del proyecto que
 llama a `verificacion.yml`. `<sha>` es el commit de TaskCodeTwo con este
@@ -373,32 +379,49 @@ viendo.
 En `preparar`, el paso «Ronda anterior» recorre los commits del PR (`git
 rev-list --first-parent origin/<base>..<head>`, sin el head, como mucho 100;
 si hay más, falla y el check sale `incompleta`) del más nuevo al más viejo y
-vuelca en crudo sus check-runs `revision-ia` (`GITHUB_TOKEN` con `checks:
-read`). Tras cada volcado, `tc2 hallazgos anterior` dice si ya hay ronda
-anterior: el `revision-ia` más nuevo y terminado de la App del revisor
-(`plataforma.revisor_id` de la configuración de la base; sin él, no hay
-ronda anterior) que dice `rojo` o `verde`. Una ronda `incompleta` no cuenta
-ni sirve de base; una `tope` tampoco (no revisó nada). Del elegido se bajan
-sus páginas de anotaciones y `tc2 hallazgos ronda` exige que título,
-conclusión y abiertos casen (la regla de `leer-revision`); si no, falla
-(falla en cerrado) y el check sale `incompleta`. Con los ficheros cambiados
-desde ese commit (`git diff --name-only <sha>..<head>`; tras un rebase que lo
-deja fuera de la historia, `base...head`) decide `ronda.json`:
+vuelca en crudo todos sus check-runs `revision-ia` (`GITHUB_TOKEN` con
+`checks: read`; `filter=all`, no solo el último de cada commit). Tras cada
+volcado, `tc2 hallazgos anterior` dice si ya hay ronda anterior: de cada
+commit, del más nuevo al más viejo, el `revision-ia` terminado de la App del
+revisor (`plataforma.revisor_id` de la configuración de la base; sin él, no
+hay ronda anterior) más nuevo que dice `rojo` o `verde`. Una ronda
+`incompleta` no cuenta ni sirve de base (si en el mismo commit hay antes un
+`rojo`, cuenta ese); una `tope` tampoco (no revisó nada) y una `ligera`
+tampoco (no la vio ningún modelo; se reconoce por la línea `Ronda N (ligera)`
+de su resumen). Del elegido se bajan sus páginas de anotaciones y `tc2
+hallazgos ronda` exige que título, conclusión y abiertos casen (la regla de
+`leer-revision`); si no, falla (falla en cerrado) y el check sale
+`incompleta`, con un mensaje que dice cómo salir: si el check se publicó a
+medias (falló un lote de anotaciones), relanzar el workflow `revision-ia`
+del commit de ese check publica en él uno completo, que pasa delante.
+
+El diff incremental sale de `tc2 hallazgos desde --anterior <sha> --head
+<head> --base origin/<base>`: desde el commit anterior, **sin lo que entró
+de la base después** (un merge de la base o el botón «Update branch»), que
+no es del PR. Si desde entonces no entró nada de la base, es `<sha>..<head>`;
+si entró, es el diff desde el árbol de `<sha>` con lo que trajo la base
+(`git merge-tree --write-tree <sha> <merge-base>`) hasta el head. Tras un
+rebase que deja el commit anterior fuera de la historia, o si esa mezcla
+tiene conflictos, el diff vuelve a `base...head` (falla en cerrado: se revisa
+todo el PR) y los abiertos se arrastran igual. Con los ficheros cambiados de
+ese diff decide `ronda.json`:
 
 | Modo | Cuándo | Qué pasa |
 |---|---|---|
 | `completa` | sin ronda anterior | ronda 1: todo el diff contra la base |
 | `incremental` | el resto | la ronda sube; cada revisora ve el diff desde el commit anterior y sus abiertos CRITICO e IMPORTANTE |
-| `ligera` | anterior `verde` y los cambios solo tocan ficheros con MENOR abiertos | sin modelo; verde; la ronda no sube |
+| `ligera` | anterior `verde`, sin otra `ligera` desde él, y los cambios solo tocan ficheros con MENOR abiertos | sin modelo; verde; la ronda no sube; no sirve de base del diff siguiente |
 | `tope` | anterior `rojo` y la ronda siguiente pasa del tope | sin modelo; dictamen `tope` (`failure`) con los abiertos arrastrados |
 
-El tope efectivo es `tope_rondas` de la configuración de la base más un
-elemento por cada línea `- ...` de primer nivel en `## Rondas extra` de la
-tarea (rama de estado). La ronda extra la decide la persona (F1-16, D6a):
+El tope efectivo es `tope_rondas` de la configuración de la base más uno por
+cada elemento de lista de primer nivel con texto (`- ...` o `* ...`, fuera de
+los bloques de código) en `## Rondas extra` (o `## Ronda extra`) de la tarea
+(rama de estado). La ronda extra la decide la persona (F1-16, D6a):
 F4-4 traducirá `tope` a la parada `preguntar` de F1-10.
 
 Cierre (falla en cerrado): en modo incremental la respuesta es un objeto
-`{"hallazgos": [...], "cierres": [...]}`. Un abierto CRITICO o IMPORTANTE solo
+`{"hallazgos": [...], "cierres": [...]}` (la clave que falte cuenta como
+lista vacía). Un abierto CRITICO o IMPORTANTE solo
 se cierra con un cierre `{id, fichero, linea, motivo}` cuya línea esté en el
 lado nuevo del diff incremental que vio la revisora; si el abierto traía
 `paso_rojo`, ese paso tiene que estar ya en verde en el informe F5-4 del head
@@ -426,6 +449,18 @@ F6-6 (decisión 3 de TASK-052).
 - La evidencia de cada hallazgo se valida pero no se ejecuta (fuera de
   F6-6; decisión 3 de TASK-052). En el cierre solo se contrasta `paso_rojo`
   con el informe F5-4.
+- Ronda `ligera` (decisión 4 de TASK-052): da verde sin consultar al modelo
+  a un cambio que solo toca ficheros con MENOR abiertos, sea cual sea el
+  cambio (también uno que meta un CRITICO), y da esos MENOR por cerrados sin
+  mirar si se arreglaron. Lo acota que no sirve de base: la ronda siguiente
+  sale desde el último commit revisado por modelo, y no puede ser otra
+  `ligera`, así que lo que entró en la ligera lo ve el modelo en el push
+  siguiente. Si la ligera es el último push antes de integrar, ese cambio
+  se integra sin revisión IA (la revisión por pares de `review` sigue).
+- El ancla de un cierre solo prueba que esa línea está en el lado nuevo del
+  diff incremental (de cualquier fichero: un arreglo puede estar en otro,
+  p. ej. en el test); que esa línea arregle el abierto lo juzga el modelo.
+  Los CRITICO e IMPORTANTE sin cierre válido siguen abiertos.
 - Rondas (F6-6): sin saldo en la API la cadena real no pasa de la ronda 1
   (`incompleta` no cuenta). Una revisora que reporta un abierto con otro id
   infla `nuevos`; la anotación de un abierto que persiste conserva la línea
diff --git a/xtask/src/workflows.rs b/xtask/src/workflows.rs
index 86ee117..4f958a7 100644
--- a/xtask/src/workflows.rs
+++ b/xtask/src/workflows.rs
@@ -2239,6 +2239,16 @@ fn rondas_en_el_workflow(texto: &str) -> Result<(), String> {
             "tc2 hallazgos abiertos ",
             "preparar no corre tc2 hallazgos abiertos",
         ),
+        // El diff incremental sin lo que entro de la base por un merge.
+        (
+            "tc2 hallazgos desde ",
+            "preparar no corre tc2 hallazgos desde",
+        ),
+        // Todos los check-runs de cada commit, no solo el ultimo.
+        (
+            "-f check_name=revision-ia -f filter=all ",
+            "preparar no pide filter=all",
+        ),
     ] {
         if !preparar.contains(pieza) {
             return Err(error.to_owned());
@@ -2529,7 +2539,17 @@ fn cada_mutacion_del_revisor_da_su_error() {
         );
     }
     // F6-6: una mutacion por regla de las rondas da rojo.
-    let rondas: [(&str, &str, &str); 7] = [
+    let rondas: [(&str, &str, &str); 9] = [
+        (
+            "linea=$(cd base && tc2 hallazgos desde ",
+            "linea=$(cd base && tc2 hallazgos base ",
+            "preparar no corre tc2 hallazgos desde",
+        ),
+        (
+            " -f filter=all -f per_page=100",
+            " -f per_page=100",
+            "preparar no pide filter=all",
+        ),
         (
             "      actions: read\n      checks: read\n    outputs:",
             "      actions: read\n    outputs:",
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 8822e8ee1e21657de2fd037cc0d334b27fe58006..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-052/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-052/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-052/revision/informe-revision-1-code-quality-reviewer.md         |   91 ++++
 tareas/03-en-revision/TASK-052/revision/informe-revision-1-rust-wasm-reviewer.md            |   72 +++
 tareas/03-en-revision/TASK-052/revision/peticion-revision-1-code-quality-reviewer.md        |  788 ++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-052/revision/peticion-revision-1-rust-wasm-reviewer.md           | 2661 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-052/tarea.md                                    |   11 +-
 7 files changed, 3620 insertions(+), 3 deletions(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 8822e8ee1e21657de2fd037cc0d334b27fe58006..HEAD -- ":(literal)crates/tc2/tests/comun/mod.rs" ":(literal)crates/tc2/tests/hallazgos.rs"
````

````
crates/tc2/tests/comun/mod.rs |  3 ++-
 crates/tc2/tests/hallazgos.rs | 80 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 2 files changed, 82 insertions(+), 1 deletion(-)
````
