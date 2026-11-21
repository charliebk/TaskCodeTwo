# Peticion de revision — TASK-046 (ronda 3)

- Tarea: TASK-046 — F5-6 Hook local rapido, opcional y con confianza
- Rama revisada: feature/task-046-f5-6-hook-local-rapido-opcional-y-con-co
- Rama base: develop
- Commit revisado (HEAD): 3d047d4017e9230ebb149eb4284a71ff22d5f883
- Fecha: 2026-11-21
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-046 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 83f83d690e1069f9fa2c28c04630593d1c1fd353 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (15 hallazgo(s)))

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
informe de esta ronda (informe-revision-3-code-quality-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-046/tarea.md
- tareas/03-en-revision/TASK-046/planificacion/
- crates/tc2/src/confianza/cierre.rs
- crates/tc2/src/confianza/mod.rs
- crates/tc2/tests/verificar.rs
- docs/ci/verificacion-reutilizable.md

## Hallazgos de la ronda 2 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 6 hallazgo(s) MENOR de la ronda 2 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido (83f83d6) | crates/tc2/src/verificacion/ejecutar.rs:52-75 | informe-revision-2-code-quality-reviewer.md |
| CRITICO-1 | CRITICO | corregido (1c9f9e9) | crates/tc2/src/confianza/cierre.rs (`entrecomillados`, `candidatos`) | informe-revision-2-code-quality-reviewer.md |

## Commits a revisar (git log 83f83d690e1069f9fa2c28c04630593d1c1fd353..HEAD)

````
3d047d4 chore(TASK-046): ronda extra 3 para verificar el CRITICO-1
0532f34 chore(TASK-046): informes y estado de los hallazgos de la ronda 2
e3e626c docs(TASK-046): cierre por nombre de los scripts raros y cifra del presupuesto tras la ronda 2
1c9f9e9 fix(TASK-046): scripts de nombre raro entran en el cierre por su nombre
15a1859 chore(TASK-046): peticion de revision ronda 2
````

## Diff desde la ronda anterior (git diff 83f83d690e1069f9fa2c28c04630593d1c1fd353..HEAD)

````diff
diff --git a/crates/tc2/src/confianza/cierre.rs b/crates/tc2/src/confianza/cierre.rs
index e774048..e5161e2 100644
--- a/crates/tc2/src/confianza/cierre.rs
+++ b/crates/tc2/src/confianza/cierre.rs
@@ -50,6 +50,12 @@ pub struct Fichero {
 pub struct Indice {
     rutas: HashMap<String, String>,
     todas: Vec<String>,
+    /// Scripts versionados cuyo nombre de fichero lleva algun caracter que
+    /// el tokenizador no junta (espacio, apostrofo...): `(clave del nombre,
+    /// ruta)`. Entran en el cierre si su nombre aparece literalmente en un
+    /// texto del cierre, sin depender de emparejar comillas (ronda 2,
+    /// CRITICO-1: `echo Don't & call "%~dp0lib\mi paso.bat"`).
+    raros: Vec<(String, String)>,
 }
 
 fn clave_busqueda(ruta: &str) -> String {
@@ -72,7 +78,34 @@ impl Indice {
             .iter()
             .map(|r| (clave_busqueda(r), r.clone()))
             .collect();
-        Self { rutas, todas }
+        let raros = todas
+            .iter()
+            .filter(|r| es_script(r))
+            .filter_map(|r| {
+                let nombre = r.rsplit('/').next().unwrap_or(r);
+                (!nombre.chars().all(es_de_token)).then(|| (clave_busqueda(nombre), r.clone()))
+            })
+            .collect();
+        Self {
+            rutas,
+            todas,
+            raros,
+        }
+    }
+
+    /// Los scripts de nombre raro cuyo nombre aparece literalmente en
+    /// `texto` (sin distinguir mayusculas en Windows y macOS). Puede incluir
+    /// de mas; nunca deja fuera uno nombrado.
+    fn por_nombre(&self, texto: &str) -> Vec<String> {
+        if self.raros.is_empty() {
+            return Vec::new();
+        }
+        let texto = clave_busqueda(texto);
+        self.raros
+            .iter()
+            .filter(|(nombre, _)| texto.contains(nombre.as_str()))
+            .map(|(_, ruta)| ruta.clone())
+            .collect()
     }
 
     /// La ruta versionada que es `candidato`, si se ejecuta como script.
@@ -264,6 +297,7 @@ pub fn calcular(
         .iter()
         .filter_map(|c| indice.script(c).map(str::to_owned))
         .collect();
+    pendientes.extend(indice.por_nombre(orden));
     let mut globs_vacios = Vec::new();
     for (texto, g) in incluye.iter().zip(compilar_globs(incluye)?) {
         let antes = pendientes.len();
@@ -288,15 +322,27 @@ pub fn calcular(
             continue;
         }
         let (tamano, mtime) = (meta.len(), mtime_ns(&meta));
+        let escanea = es_script(&rel) && tamano <= MAX_ESCANEO;
         let f = match cache.get(rel.as_str()) {
             // Sin `mtime` del sistema (0) no hay cache: se lee siempre.
-            Some(c) if c.tamano == tamano && c.mtime == mtime && mtime != 0 => (*c).clone(),
+            Some(c) if c.tamano == tamano && c.mtime == mtime && mtime != 0 => {
+                // Los scripts de nombre raro dependen del indice actual, no
+                // de la cache: si el repo tiene alguno, el texto se vuelve a
+                // leer (sin hashear) para buscar su nombre.
+                if escanea && !indice.raros.is_empty() {
+                    let datos = std::fs::read(&ruta).map_err(ErrorCierre::Lectura)?;
+                    pendientes.extend(indice.por_nombre(&String::from_utf8_lossy(&datos)));
+                }
+                (*c).clone()
+            }
             _ => {
                 let datos = std::fs::read(&ruta).map_err(ErrorCierre::Lectura)?;
                 hasheados += datos.len() as u64;
                 let dir = rel.rsplit_once('/').map_or("", |(d, _)| d);
-                let candidatos = if es_script(&rel) && tamano <= MAX_ESCANEO {
-                    candidatos(&String::from_utf8_lossy(&datos), dir)
+                let candidatos = if escanea {
+                    let texto = String::from_utf8_lossy(&datos);
+                    pendientes.extend(indice.por_nombre(&texto));
+                    candidatos(&texto, dir)
                 } else {
                     Vec::new()
                 };
diff --git a/crates/tc2/src/confianza/mod.rs b/crates/tc2/src/confianza/mod.rs
index 8323853..bbf9046 100644
--- a/crates/tc2/src/confianza/mod.rs
+++ b/crates/tc2/src/confianza/mod.rs
@@ -40,7 +40,6 @@ pub const NOMBRE: &str = "confiar";
 /// Claves de la configuracion cuya orden se puede confiar hoy.
 pub const CLAVES: [&str; 1] = ["verificacion.comando"];
 
-/// Version del fichero del almacen.
 /// Version del fichero del almacen. La 2 (ronda 1 de TASK-046) cambia los
 /// `candidatos` (comillas y no ASCII): un almacen de la 1 se lee vacio y todo
 /// vuelve a `nunca` (falla cerrado).
diff --git a/docs/ci/verificacion-reutilizable.md b/docs/ci/verificacion-reutilizable.md
index e28b44a..dda8ee7 100644
--- a/docs/ci/verificacion-reutilizable.md
+++ b/docs/ci/verificacion-reutilizable.md
@@ -546,14 +546,27 @@ de `cmd`), en Windows 11, binario `--release`, cuatro ejecuciones el
 | en frío (todos los `mtime` cambiados: 1 210 462 bytes rehasheados) | 318 – 595 |
 | en caliente (0 bytes rehasheados) | 249 – 419 |
 
-Con el binario de depuración, el peor caso fue 530 ms (en frío). Se lanza a
+Con el binario de depuración, el peor caso fue 530 ms (en frío). Tras la
+ronda 2 de revisión (scripts de nombre raro por su nombre), dos ejecuciones
+en release: 284 – 401 ms sin confiar, 322 – 368 ms en frío y 258 – 292 ms en
+caliente; y con un script de nombre raro más en el cierre (52 ficheros, los
+textos se releen también en caliente, medida a mano sin dejarla en la
+prueba): 239 – 306, 306 – 319 y 254 – 279 ms. Se lanza a
 mano: `cargo test -p tc2 --release --test verificar -- --ignored --nocapture`
 (la verificación del repo no corre las pruebas ignoradas).
 
 **Cómo se forma el cierre.** Se siguen los tokens con forma de ruta (llevan
 `.` o `/`, con letras también no ASCII, como `verificación.bat`) y, enteros,
 los textos entre comillas de una línea (`call "%~dp0lib\mi paso.bat"`), con
-`%~dp0` resuelto a la carpeta del script. En Windows y macOS las rutas se
+`%~dp0` resuelto a la carpeta del script. Como emparejar comillas falla con
+un apóstrofo suelto (`echo Don't & call "%~dp0lib\mi paso.bat"`) o con
+comillas anidadas (`"$(dirname "$0")/lib/mi paso.sh"`), además todo script
+versionado cuyo nombre lleve un carácter que el tokenizador no junta
+(espacio, apóstrofo…) entra en el cierre si su nombre de fichero aparece
+literalmente en la orden o en el texto de un fichero del cierre, se siga o
+no la ruta: puede sumar de más a la huella, nunca dejar fuera uno nombrado.
+Si el repo tiene scripts así, los textos del cierre se releen (sin
+rehashear) también en caliente. En Windows y macOS las rutas se
 comparan sin distinguir mayúsculas. Los globs de `confianza_incluye` admiten
 `./` al principio y siguen la misma regla de mayúsculas; un glob que no
 compila es error de la configuración (no se confía ni se ejecuta nada) y uno
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 83f83d690e1069f9fa2c28c04630593d1c1fd353..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-046/revision/informe-revision-2-code-quality-reviewer.md  |  89 ++++++++++++
 tareas/03-en-revision/TASK-046/revision/informe-revision-2-rust-wasm-reviewer.md     |  70 ++++++++++
 tareas/03-en-revision/TASK-046/revision/peticion-revision-2-code-quality-reviewer.md | 811 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-046/revision/peticion-revision-2-rust-wasm-reviewer.md    | 812 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-046/tarea.md                                              |   5 +
 5 files changed, 1787 insertions(+)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 83f83d690e1069f9fa2c28c04630593d1c1fd353..HEAD -- ":(literal)crates/tc2/tests/verificar.rs"
````

````
crates/tc2/tests/verificar.rs | 90 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 90 insertions(+)
````
