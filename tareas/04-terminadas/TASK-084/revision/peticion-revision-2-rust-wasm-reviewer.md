# Peticion de revision — TASK-084 (ronda 2)

- Tarea: TASK-084 — F3-12 Progreso en vivo: fases que faltan y tiempo restante
- Rama revisada: feature/task-084-f3-12-progreso-en-vivo-fases-que-faltan
- Rama base: develop
- Commit revisado (HEAD): f44ce49bb8f74f58264b879674eb039a7a5492f6
- Fecha: 2026-11-18
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-084 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde efee1212394fb2f28018b63e4dc78f56a39b4f6e (el commit revisado en la ronda anterior)
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

- tareas/03-en-revision/TASK-084/tarea.md
- tareas/03-en-revision/TASK-084/planificacion/
- crates/tc2/src/init/inventario.rs
- crates/tc2/src/main.rs
- crates/tc2/src/progreso/fichero.rs
- crates/tc2/src/progreso/linea.rs
- crates/tc2/src/progreso/mod.rs
- crates/tc2/src/tablero.rs
- crates/tc2/tests/progreso.rs
- docs/adr/F1-17-progreso-y-estimacion.md
- docs/distribucion/instalar.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 4 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| R1-01 | IMPORTANTE | corregido | crates/tc2/src/progreso/mod.rs:282 | informe-revision-1-rust-wasm-reviewer.md |

## Commits a revisar (git log efee1212394fb2f28018b63e4dc78f56a39b4f6e..HEAD)

````
f44ce49 docs(TASK-084): cita de settings.local.json y seguir en instalar.md y F1-17
8d43627 fix(TASK-084): correcciones de la ronda 1 de revision
0ca8ebf chore(TASK-084): informes de revision ronda 1
4cc0c36 chore(TASK-084): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff efee1212394fb2f28018b63e4dc78f56a39b4f6e..HEAD)

````diff
diff --git a/crates/tc2/src/init/inventario.rs b/crates/tc2/src/init/inventario.rs
index 5411d1e..61c3fdf 100644
--- a/crates/tc2/src/init/inventario.rs
+++ b/crates/tc2/src/init/inventario.rs
@@ -241,6 +241,10 @@ pub(super) fn plan(c: &Contexto<'_>) -> Result<Vec<Accion>, Informe> {
 
 /// Donde se recomienda pegar la status line: el `settings.local.json` del
 /// proyecto, que no sustituye la de la 0.16 de `~/.claude/settings.json`.
+/// Es el ambito «Project local» de
+/// <https://code.claude.com/docs/en/settings#settings-files-and-who-they-affect>
+/// (consultada el 2026-11-18): solo para quien lo escribe y en ese proyecto,
+/// fuera de git, y por encima de los otros dos.
 pub(super) const RUTA_STATUS_LINE: &str = ".claude/settings.local.json";
 
 /// La propuesta de la status line: la orden exacta en `orden=` y el bloque
diff --git a/crates/tc2/src/main.rs b/crates/tc2/src/main.rs
index c481558..e6875fc 100644
--- a/crates/tc2/src/main.rs
+++ b/crates/tc2/src/main.rs
@@ -37,27 +37,39 @@ fn main() -> ExitCode {
     }
     // F3-12: `status --seguir` escribe a trozos (una linea cada vez que
     // cambia), que `Salida` no sabe hacer. Termina con Ctrl+C o al cerrarse
-    // stdout, con codigo 0.
+    // stdout (en pocos segundos: fuera de una terminal repite la linea para
+    // enterarse, `progreso::REPETIR_SEGUIR`), con codigo 0.
     if args.len() == 2 && args[0] == tc2::tablero::STATUS && args[1] == tc2::tablero::SEGUIR {
         let dir = std::env::current_dir().unwrap_or_default();
         let mut stdout = std::io::stdout();
+        let repetir =
+            (!std::io::IsTerminal::is_terminal(&stdout)).then_some(tc2::progreso::REPETIR_SEGUIR);
         let _ = tc2::progreso::seguir(
             &dir,
             &mut stdout,
             &tc2::identidad::ahora,
             &|| false,
             tc2::progreso::ESPERA_SEGUIR,
+            repetir,
         );
         return ExitCode::SUCCESS;
     }
+    // La status line (`estado --stdin`) siempre sale con 0 y stderr vacio,
+    // tambien si Claude Code cerro stdout antes de leerla (revision 1 de
+    // TASK-084, R1-03): no hay a quien avisar.
+    let es_status_line = args.first().map(String::as_str) == Some(tc2::progreso::ESTADO);
     let salida = tc2::ejecutar_en(idioma, args);
     // Si no se puede escribir la salida (stdout cerrado), no se da por bueno:
     // quien lee el codigo 0 creeria que recibio la respuesta.
     let mut stdout = std::io::stdout();
-    if let Err(e) = stdout
+    let escrito = stdout
         .write_all(salida.stdout.as_bytes())
-        .and_then(|()| stdout.flush())
-    {
+        .and_then(|()| stdout.flush());
+    if es_status_line {
+        let _ = std::io::stderr().write_all(salida.stderr.as_bytes());
+        return ExitCode::from(salida.codigo);
+    }
+    if let Err(e) = escrito {
         let m = Mensaje::nuevo(&txt::NO_SE_PUDO_ESCRIBIR).con("error", e.to_string());
         let _ = writeln!(std::io::stderr(), "{}", m.texto(idioma));
         return ExitCode::from(1);
diff --git a/crates/tc2/src/progreso/fichero.rs b/crates/tc2/src/progreso/fichero.rs
index b0b0ab4..44ad69d 100644
--- a/crates/tc2/src/progreso/fichero.rs
+++ b/crates/tc2/src/progreso/fichero.rs
@@ -165,13 +165,16 @@ pub fn leer(carpeta: &Path, id: &str) -> Lectura {
 }
 
 /// Escribe `p` en `carpeta`: antes, el que habia pasa a `.anterior.json`.
-/// Cada paso, de una vez ([`crate::huella::escribir_local`]).
+/// Cada paso, de una vez ([`crate::huella::escribir_local`]). Si el anterior
+/// no se puede escribir, el progreso se escribe igual (revision 1 de
+/// TASK-084, R1-04): el anterior solo es el respaldo de un fichero a medias,
+/// y sin el nuevo la linea se quedaria con el dato viejo.
 pub fn escribir(carpeta: &Path, p: &Progreso) -> std::io::Result<()> {
     let r = ruta(carpeta, &p.id);
     // Solo un anterior valido: si el actual quedo a medias, el anterior que
     // habia sigue siendo el ultimo bueno.
     if let Some(previo) = de(&r, &p.id) {
-        crate::huella::escribir_local(&ruta_anterior(carpeta, &p.id), &previo.json())?;
+        let _ = crate::huella::escribir_local(&ruta_anterior(carpeta, &p.id), &previo.json());
     }
     crate::huella::escribir_local(&r, &p.json())
 }
@@ -291,4 +294,20 @@ pub(crate) mod tests {
         let _ = std::fs::remove_dir_all(&d);
         Ok(())
     }
+
+    /// Revision 1 de TASK-084 (R1-04): si el anterior no se puede escribir
+    /// (aqui, una carpeta con su nombre), el progreso se escribe igual.
+    #[test]
+    fn sin_anterior_escribe_el_progreso() -> std::io::Result<()> {
+        let d = dir("anterior-roto");
+        let mut p = ejemplo("TASK-001");
+        escribir(&d, &p)?;
+        std::fs::create_dir_all(ruta_anterior(&d, "TASK-001").join("ocupado"))?;
+        p.calculada = 2_000;
+        escribir(&d, &p)?;
+        assert_eq!(leer(&d, "TASK-001"), Lectura::Bueno(p));
+        assert!(ruta_anterior(&d, "TASK-001").is_dir());
+        let _ = std::fs::remove_dir_all(&d);
+        Ok(())
+    }
 }
diff --git a/crates/tc2/src/progreso/linea.rs b/crates/tc2/src/progreso/linea.rs
index 6ffc56b..94ed2a5 100644
--- a/crates/tc2/src/progreso/linea.rs
+++ b/crates/tc2/src/progreso/linea.rs
@@ -13,7 +13,7 @@
 //! worktree), cada hueco con el tope de 45 min de F1-17. Pausada, el reloj
 //! no corre. Todo lo que viene del fichero se sanea (F1-17, punto 8).
 
-use super::fichero::{PASO_MIN, Progreso};
+use super::fichero::{PASO_MIN, Progreso, TOPE_MIN};
 use crate::estimacion::{UMBRAL_MIN, minutos};
 use crate::tablero::sanear;
 
@@ -55,12 +55,24 @@ pub fn transcurrido(p: &Progreso, ahora: u64, actividad: Option<u64>) -> (f64, f
     (p.transcurrido + antes + abierto.min(UMBRAL_MIN), abierto)
 }
 
-/// La parte de lo que falta.
+/// Aviso de una tabla que ya no llega a lo transcurrido (revision 1 de
+/// TASK-084, R1-05).
+pub const SATURADA: &str = "estimacion saturada: mas de 480 min en la fase";
+
+/// La parte de lo que falta. Pasado el ultimo punto de la tabla
+/// ([`TOPE_MIN`]), el dato es el de ese punto y la linea lo avisa: la tabla
+/// no se alarga porque una fase de mas de 8 h activas ya no se parece a sus
+/// parecidas y el fichero creceria sin aportar nada.
 fn lo_que_falta(p: &Progreso, t: f64) -> String {
     let i = ((t / PASO_MIN as f64).floor().max(0.0) as usize).min(p.tabla.len().saturating_sub(1));
     let nivel = sanear(&p.nivel, MAX_NIVEL);
-    match p.tabla.get(i) {
-        None => format!("faltan: sin estimacion (n={})", p.n),
+    let saturada = if t > TOPE_MIN as f64 {
+        format!("{SEP}{SATURADA}")
+    } else {
+        String::new()
+    };
+    let parte = match p.tabla.get(i) {
+        None => return format!("faltan: sin estimacion (n={})", p.n),
         Some(x) if x.desbordada => format!(
             "faltan ≥{} min (fase en curso por encima de sus parecidas){SEP}n={} {nivel}",
             minutos(x.mediana),
@@ -72,7 +84,8 @@ fn lo_que_falta(p: &Progreso, t: f64) -> String {
             minutos(x.p80),
             p.n
         ),
-    }
+    };
+    format!("{parte}{saturada}")
 }
 
 /// `linea(progreso, ahora, ultima_actividad)`: la linea de la tarea.
@@ -252,6 +265,54 @@ mod tests {
         );
     }
 
+    /// Revision 1 de TASK-084 (M1): a mitad de tramo (23 min) vale el punto
+    /// de los 20, no el de los 25 (`floor`, no `round`).
+    #[test]
+    fn m1_a_mitad_de_tramo_vale_el_punto_anterior() {
+        let (t, c) = tarea(EN_CURSO, &[(0, "start")]);
+        let p = calcular(&t, &c, &historico(), &Commits::vacios(), T0);
+        let p = p.unwrap_or_else(|| unreachable!());
+        let a_los_20 = linea(&p, T0 + 20 * 60, None);
+        let a_los_25 = linea(&p, T0 + 25 * 60, None);
+        assert_ne!(a_los_20, a_los_25);
+        assert_eq!(linea(&p, T0 + 23 * 60, None), a_los_20);
+        assert_eq!(linea(&p, T0 + 24 * 60 + 59, None), a_los_20);
+    }
+
+    /// Revision 1 de TASK-084 (M2): la frontera de «sin actividad
+    /// registrada»: a los 45 min no, a los 46 si.
+    #[test]
+    fn m2_frontera_de_sin_actividad() {
+        let p = ejemplo("TASK-001");
+        let a_los_45 = linea(&p, p.ultimo_evento + 45 * 60, None);
+        assert!(!a_los_45.contains("sin actividad"), "{a_los_45}");
+        let a_los_46 = linea(&p, p.ultimo_evento + 46 * 60, None);
+        assert!(
+            a_los_46.ends_with(" · sin actividad registrada hace 46 min"),
+            "{a_los_46}"
+        );
+    }
+
+    /// Revision 1 de TASK-084 (R1-05): pasado el ultimo punto de la tabla,
+    /// la linea avisa de que el dato esta saturado; en el punto, no.
+    #[test]
+    fn r1_05_pasado_el_tope_avisa_saturada() {
+        assert!(SATURADA.contains(&format!("{TOPE_MIN} min")));
+        let mut p = ejemplo("TASK-001");
+        p.transcurrido = TOPE_MIN as f64;
+        let en_el_tope = linea(&p, p.ultimo_evento, None);
+        assert!(!en_el_tope.contains("saturada"), "{en_el_tope}");
+        p.transcurrido = TOPE_MIN as f64 + 1.0;
+        let pasado = linea(&p, p.ultimo_evento, None);
+        assert!(
+            pasado.ends_with(&format!("faltan ≈50 min (p80 65) · n=6 repo · {SATURADA}")),
+            "{pasado}"
+        );
+        // Sin tabla no hay dato que saturar.
+        p.tabla.clear();
+        assert!(!linea(&p, p.ultimo_evento, None).contains("saturada"));
+    }
+
     #[test]
     fn otras_formas() {
         let p = ejemplo("TASK-001");
diff --git a/crates/tc2/src/progreso/mod.rs b/crates/tc2/src/progreso/mod.rs
index 85773a5..ea4c5cd 100644
--- a/crates/tc2/src/progreso/mod.rs
+++ b/crates/tc2/src/progreso/mod.rs
@@ -127,18 +127,38 @@ fn actividad(gitdir: &Path) -> Option<u64> {
 }
 
 /// La linea de una tarea leida.
-fn linea_de(l: &Lectura, id: &str, gitdir: &Path, ahora: u64) -> Option<(u64, String)> {
+/// `gitdir` es la carpeta git de su worktree, si se encontro.
+fn linea_de(l: &Lectura, id: &str, gitdir: Option<&Path>, ahora: u64) -> Option<(u64, String)> {
+    let act = || gitdir.and_then(actividad);
     match l {
         Lectura::Ninguno => None,
-        Lectura::Bueno(p) => Some((p.calculada, linea::linea(p, ahora, actividad(gitdir)))),
+        Lectura::Bueno(p) => Some((p.calculada, linea::linea(p, ahora, act()))),
         Lectura::Anterior(p) => Some((
             p.calculada,
-            linea::con_antiguedad(&linea::linea(p, ahora, actividad(gitdir)), p, ahora),
+            linea::con_antiguedad(&linea::linea(p, ahora, act()), p, ahora),
         )),
         Lectura::Ilegible => Some((0, linea::ilegible(id))),
     }
 }
 
+/// Las carpetas git de los worktrees de tarea, por ID. Cada carpeta
+/// administrativa (`<comun>/worktrees/<nombre>`) guarda en su fichero
+/// `gitdir` la ruta del `.git` de su worktree (absoluta, o relativa a ella);
+/// el ID sale de esa ruta y no del nombre de la carpeta, al que git anade un
+/// numero si choca con otro (revision 1 de TASK-084, M4).
+fn gitdirs_de_tareas(comun: &Path) -> std::collections::HashMap<String, PathBuf> {
+    std::fs::read_dir(comun.join("worktrees"))
+        .into_iter()
+        .flatten()
+        .flatten()
+        .filter_map(|e| {
+            let admin = e.path();
+            let punto_git = junto(&admin, texto_corto(&admin.join("gitdir"))?.trim());
+            Some((id_de_worktree(punto_git.parent()?)?, admin))
+        })
+        .collect()
+}
+
 /// La linea de progreso para la carpeta `dir` a las `ahora`.
 pub fn linea_en(dir: &Path, ahora: u64) -> String {
     let Some(repo) = buscar(dir) else {
@@ -146,16 +166,17 @@ pub fn linea_en(dir: &Path, ahora: u64) -> String {
     };
     let carpeta = fichero::carpeta(&repo.comun);
     if let Some(id) = &repo.id {
-        return linea_de(&fichero::leer(&carpeta, id), id, &repo.gitdir, ahora)
+        return linea_de(&fichero::leer(&carpeta, id), id, Some(&repo.gitdir), ahora)
             .map_or_else(|| linea::SIN_TAREA.to_owned(), |(_, l)| l);
     }
     // En el principal (o en `tareas/`): la de calculo mas reciente.
     let ids = fichero::ids(&carpeta);
+    let gitdirs = gitdirs_de_tareas(&repo.comun);
     let mejor = ids
         .iter()
         .filter_map(|id| {
-            let gitdir = repo.comun.join("worktrees").join(id);
-            linea_de(&fichero::leer(&carpeta, id), id, &gitdir, ahora)
+            let gitdir = gitdirs.get(id).map(PathBuf::as_path);
+            linea_de(&fichero::leer(&carpeta, id), id, gitdir, ahora)
         })
         .max_by_key(|(calculada, _)| *calculada);
     match mejor {
@@ -272,28 +293,55 @@ pub fn quitar(g: &Git, dir: &Path, id: &str) -> Result<Refresco, String> {
 /// Cada cuanto mira `tc2 status --seguir`.
 pub const ESPERA_SEGUIR: std::time::Duration = std::time::Duration::from_secs(2);
 
+/// Vueltas de [`ESPERA_SEGUIR`] sin cambios tras las que `status --seguir`
+/// repite la linea cuando stdout no es una terminal (4 s: quien cierra el
+/// pipe ve terminar el proceso en unos 4 s como mucho).
+///
+/// Revision 1 de TASK-084 (R1-01): un proceso solo sabe que se cerro el
+/// extremo de lectura de su stdout cuando escribe (`EPIPE` en Unix, donde
+/// Rust ignora `SIGPIPE`; `ERROR_NO_DATA` en Windows). Sin escribir, con la
+/// linea fija (sin tarea en curso), `--seguir` quedaba vivo para siempre.
+/// Preguntar al sistema por el pipe (`poll` con `POLLERR` en Unix,
+/// `PeekNamedPipe` en Windows, que ademas pide permiso de lectura que el
+/// extremo de escritura no tiene) exige `unsafe`, prohibido en el workspace
+/// (`unsafe_code = "forbid"`); y una escritura vacia no llega al sistema
+/// (`Stdout` la absorbe en su buffer), asi que no detecta nada. Repetir la
+/// misma linea si llega al sistema y funciona igual en los dos. En una
+/// terminal no se repite (no hay pipe que se cierre: se corta con Ctrl+C o
+/// al cerrar la ventana), para no ensuciar la vista.
+pub const REPETIR_SEGUIR: u32 = 2;
+
 /// `tc2 status --seguir` (F1-17, punto 8): la misma lectura que la status
 /// line para `dir`, repetida cada `espera`. Escribe la linea al empezar y
-/// otra solo cuando cambia el texto (porque se reescribio el fichero o
-/// porque corrio el reloj), sin ANSI. Para cuando `parar()` es verdadero
-/// (despues de cada mirada) o cuando no se puede escribir (stdout cerrado:
-/// `Ok`). `reloj` y `parar` se inyectan en las pruebas; en el binario son la
-/// hora del sistema y nunca (lo corta Ctrl+C).
+/// otra cuando cambia el texto (porque se reescribio el fichero o porque
+/// corrio el reloj), sin ANSI; con `repetir = Some(n)`, tambien la repite
+/// tras `n` vueltas sin cambios, para enterarse de que stdout se cerro
+/// ([`REPETIR_SEGUIR`]). Para cuando `parar()` es verdadero (despues de
+/// cada mirada) o cuando no se puede escribir (stdout cerrado: `Ok`).
+/// `reloj` y `parar` se inyectan en las pruebas; en el binario son la hora
+/// del sistema y nunca (lo corta Ctrl+C o el cierre de stdout).
 pub fn seguir(
     dir: &Path,
     out: &mut dyn std::io::Write,
     reloj: &dyn Fn() -> u64,
     parar: &dyn Fn() -> bool,
     espera: std::time::Duration,
+    repetir: Option<u32>,
 ) -> std::io::Result<()> {
     let mut ultima: Option<String> = None;
+    let mut sin_cambios: u32 = 0;
     loop {
         let l = linea_en(dir, reloj());
-        if ultima.as_deref() != Some(l.as_str()) {
+        let cambia = ultima.as_deref() != Some(l.as_str());
+        if !cambia {
+            sin_cambios = sin_cambios.saturating_add(1);
+        }
+        if cambia || repetir.is_some_and(|n| sin_cambios >= n.max(1)) {
             if writeln!(out, "{l}").and_then(|()| out.flush()).is_err() {
                 return Ok(());
             }
             ultima = Some(l);
+            sin_cambios = 0;
         }
         if parar() {
             return Ok(());
@@ -466,6 +514,7 @@ mod tests {
             &|| 1_000,
             &parar,
             std::time::Duration::ZERO,
+            None,
         )?;
         let texto = String::from_utf8_lossy(&out).into_owned();
         let lineas: Vec<&str> = texto.lines().collect();
@@ -496,7 +545,73 @@ mod tests {
             &|| 1_000,
             &|| false,
             std::time::Duration::ZERO,
+            Some(REPETIR_SEGUIR),
+        )?;
+        // Revision 1 de TASK-084 (R1-01): con `repetir`, la linea fija se
+        // repite cada `n` vueltas sin cambios (por eso se entera del cierre);
+        // sin el, no.
+        for (repetir, esperadas) in [(Some(2), 3), (None, 1)] {
+            let vueltas = std::cell::Cell::new(0);
+            let parar = || {
+                vueltas.set(vueltas.get() + 1);
+                vueltas.get() >= 5
+            };
+            let mut out = Vec::new();
+            seguir(
+                &principal,
+                &mut out,
+                &|| 1_000,
+                &parar,
+                std::time::Duration::ZERO,
+                repetir,
+            )?;
+            let texto = String::from_utf8_lossy(&out).into_owned();
+            assert_eq!(texto.lines().count(), esperadas, "{repetir:?}: {texto}");
+            assert!(texto.lines().all(|l| l == lineas[1]), "{texto}");
+        }
+        let _ = std::fs::remove_dir_all(&raiz);
+        Ok(())
+    }
+
+    /// Revision 1 de TASK-084 (M4): desde el principal, la actividad de una
+    /// tarea es el `logs/HEAD` de la carpeta administrativa cuyo `gitdir`
+    /// apunta a su worktree, aunque esa carpeta no se llame como el ID (git
+    /// le anade un numero si choca).
+    #[test]
+    fn m4_actividad_con_carpeta_administrativa_de_otro_nombre() -> std::io::Result<()> {
+        let raiz = dir("admin");
+        let principal = raiz.join("p");
+        let comun = principal.join(".git");
+        std::fs::create_dir_all(&comun)?;
+        let carpeta = fichero::carpeta(&comun);
+        let p = fichero::tests::ejemplo("TASK-001");
+        fichero::escribir(&carpeta, &p)?;
+        let ahora = p.ultimo_evento + 100 * 60;
+        let sin = linea_en(&principal, ahora);
+        assert!(
+            sin.ends_with("sin actividad registrada hace 100 min"),
+            "{sin}"
+        );
+        // Una carpeta administrativa con el nombre del ID que no es la suya
+        // (apunta a otro sitio) no cuenta.
+        let cuando =
+            std::time::UNIX_EPOCH + std::time::Duration::from_secs(p.ultimo_evento + 90 * 60);
+        let poner = |nombre: &str, gitdir: &Path| -> std::io::Result<()> {
+            let admin = comun.join("worktrees").join(nombre);
+            std::fs::create_dir_all(admin.join("logs"))?;
+            std::fs::write(admin.join("gitdir"), format!("{}\n", gitdir.display()))?;
+            let head = std::fs::File::create(admin.join("logs").join("HEAD"))?;
+            head.set_modified(cuando)
+        };
+        poner("TASK-001", &raiz.join("otro").join(".git"))?;
+        assert_eq!(linea_en(&principal, ahora), sin);
+        // La suya, con sufijo (y `gitdir` relativo a ella): 10 min abiertos.
+        poner(
+            "TASK-0011",
+            Path::new("../../../.taskcodetwo/worktrees/TASK-001/.git"),
         )?;
+        let con = linea_en(&principal, ahora);
+        assert!(!con.contains("sin actividad"), "{con}");
         let _ = std::fs::remove_dir_all(&raiz);
         Ok(())
     }
diff --git a/crates/tc2/src/tablero.rs b/crates/tc2/src/tablero.rs
index 4344e1e..ce11251 100644
--- a/crates/tc2/src/tablero.rs
+++ b/crates/tc2/src/tablero.rs
@@ -274,9 +274,12 @@ fn avisos_ya_no_tuyas(v: &[(String, Quien)]) -> Vec<Mensaje> {
 
 /// Texto ajeno listo para imprimir (F1-17, decision 8): sin caracteres de
 /// control ni de formato (bidi, anchura cero), espacios juntados y como
-/// mucho `max` caracteres (con `…` si se corta).
+/// mucho `max` caracteres (con `…` si se corta). De formato: U+00AD (guion
+/// blando), U+061C (marca de letra arabe, bidi), U+180E (separador de
+/// vocales mongol), U+200B..U+200F, U+202A..U+202E, U+2060..U+2064
+/// (union de palabra y operadores invisibles), U+2066..U+2069 y U+FEFF.
 pub fn sanear(t: &str, max: usize) -> String {
-    let formato = |c: char| matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}');
+    let formato = |c: char| matches!(c, '\u{00AD}' | '\u{061C}' | '\u{180E}' | '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2060}'..='\u{2064}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}');
     let limpio: String = t
         .chars()
         .map(|c| if c.is_control() { ' ' } else { c })
@@ -980,6 +983,13 @@ mod tests {
     fn sanear_quita_controles_y_formato_y_acota() {
         assert_eq!(sanear("a\u{1b}[31mb\tc\n", 60), "a [31mb c");
         assert_eq!(sanear("x\u{202E}y\u{200B}z", 60), "xyz");
+        // Revision 1 de TASK-084 (R1-02): los demas de formato invisibles.
+        for c in [
+            '\u{00AD}', '\u{061C}', '\u{180E}', '\u{2060}', '\u{2061}', '\u{2062}', '\u{2063}',
+            '\u{2064}',
+        ] {
+            assert_eq!(sanear(&format!("x{c}y"), 60), "xy", "U+{:04X}", c as u32);
+        }
         assert_eq!(sanear("abcdef", 4), "abc…");
         assert_eq!(sanear("  dos   espacios ", 60), "dos espacios");
     }
diff --git a/docs/adr/F1-17-progreso-y-estimacion.md b/docs/adr/F1-17-progreso-y-estimacion.md
index 1a4c1b6..c64e9bd 100644
--- a/docs/adr/F1-17-progreso-y-estimacion.md
+++ b/docs/adr/F1-17-progreso-y-estimacion.md
@@ -20,7 +20,8 @@
   - **Qué guarda**: versión, ID, fase (`en-curso` es implementación; `en-revision`, rondas), si está pausada, las fases que faltan, lo transcurrido en la fase hasta el último evento (regla del real, punto 1), el último evento, la hora del cálculo, nivel, `n`, avisos saneados y la **tabla de lo que falta** de 0 a 480 min cada 5 (de `restante`). Su tamaño no depende del número de tareas (L1). Sin semillas (adenda de TASK-083), sin parecidas la tabla va vacía y la línea dice `faltan: sin estimacion (n=N)` (el texto de C5 de TASK-084).
   - **Cuándo se escribe**: en cada transición de la 2.0 que sale bien (`start`, `pausa`, `reanudar`); `descartar`, `liberar` y `reasignar` lo borran en el clon que las lanza. `start` lo calcula dentro de la toma con el mismo histórico y el mismo `git log` que la estimación (un solo `git log`, R1-05 de TASK-083); las demás releen el estado conocido (sin `fetch`) y un `git log` (O(N), solo en transiciones). Un fallo al escribirlo no cambia la transición: `progreso=no-escrito` y un aviso por stderr con el mismo código. `review` y `finish` (F6-2, F7-1) lo llamarán por `progreso::refrescar` cuando existan; hasta entonces la fase de rondas solo se alcanza con datos de prueba.
   - **Lectura (`tc2 estado --stdin`, la `STATUS_LINE` de F1-12)**: su firma no recibe `Git`; busca el repo subiendo desde `workspace.current_dir` del JSON (o `cwd`, o la carpeta del proceso) hasta un `.git` (carpeta, o fichero `gitdir:` con `commondir`) y abre un fichero (en el principal, los de las tareas en curso: la de cálculo más reciente y `+N en curso`). Lo transcurrido es el del cálculo más el reloj desde el último evento o desde el `mtime` de `logs/HEAD` del worktree; **cada hueco con el tope de 45 min** (el plan de TASK-084 lo concreta así frente al «hueco abierto entero» del punto 5: con el tope, la vista no salta de golpe a «desbordada» tras una pausa no declarada), y con más de 45 min sin actividad añade `sin actividad registrada hace N min`. Pausada, el reloj no corre. Un fichero ilegible da el anterior con `(dato de hace N min)`; sin ninguno, `progreso ilegible`. Siempre una línea y código 0; sin tarea, `tc2 · sin tarea en curso`. Medido en Windows 11 (2026-11-18), de lanzar el proceso a leer su salida: binario release, mediana de 17 ms en caliente (15 ejecuciones, 16-19 ms; la primera en frío, 83 ms); binario de pruebas, 18-20 ms con 10 y con 1000 tareas (C1 de TASK-084, que falla por encima de 200 ms). No incluye el arranque del shell con el que Claude Code la lanza (Git Bash o PowerShell en Windows).
-  - **`tc2 status --seguir`**: la misma lectura cada 2 s desde la carpeta actual; imprime una línea nueva solo si cambia el texto, sin ANSI, y termina al cerrarse stdout (código 0) o con Ctrl+C (lo corta el sistema).
+  - **`tc2 status --seguir`**: la misma lectura cada 2 s desde la carpeta actual; imprime una línea nueva solo si cambia el texto, sin ANSI, y termina al cerrarse stdout (código 0) o con Ctrl+C (lo corta el sistema). Revisión 1 de TASK-084 (R1-01): un proceso solo se entera del cierre de su stdout al escribir, así que, cuando stdout no es un terminal, repite la misma línea tras 2 vueltas sin cambios (4 s) y termina en unos 4 s tras el cierre; preguntar al sistema por la tubería (`poll` en Unix, `PeekNamedPipe` en Windows) exige `unsafe`, prohibido en el workspace, y una escritura vacía no llega al sistema. En un terminal no repite.
+  - **Saturada (R1-05 de la revisión 1 de TASK-084)**: pasados los 480 min de la tabla, lo que falta es el del último punto y la línea añade `estimacion saturada: mas de 480 min en la fase`; la tabla no se alarga (más de 8 h activas en una fase ya no se parece a sus parecidas y el fichero crecería sin aportar).
   - **Instalación (D2A)**: `tc2 init` solo propone la `statusLine` (línea `clase=status-line decision=pendiente motivo=pegar-a-mano orden=tc2 estado --stdin`, con el bloque JSON en `--json`); nunca lee ni escribe un `settings.json`, así que no sabe si ya está pegada y la propone en cada ejecución como `pendiente` (no `persona`: la segunda ejecución de `init` no pide nada nuevo, C2 de TASK-028). Se recomienda pegarla en `.claude/settings.local.json` del proyecto para no sustituir la de la 0.16 en `~/.claude/settings.json`; `tc2 desinstalar` solo reconoce la de `~/.claude/settings.json`.
   - **Lote (D3A)**: queda en F4-9; el fichero lleva `version` para añadirlo. **Tarea real (D4A)**: se acepta con el E2E de las transiciones existentes; la comprobación durante una tarea real queda para F9-7.
 
diff --git a/docs/distribucion/instalar.md b/docs/distribucion/instalar.md
index 35d85b7..6039f3c 100644
--- a/docs/distribucion/instalar.md
+++ b/docs/distribucion/instalar.md
@@ -222,7 +222,17 @@ motivo=pegar-a-mano orden=tc2%20estado%20--stdin` (con `--json`, el bloque
 entero en `propuesta`). Para usarla,
 añade a mano a `.claude/settings.local.json` del proyecto (recomendado: no
 sustituye la status line de la 0.16 de `~/.claude/settings.json`) o a tu
-`~/.claude/settings.json`:
+`~/.claude/settings.json`. Según la página oficial de ajustes de Claude Code
+([Settings files and who they
+affect](https://code.claude.com/docs/en/settings#settings-files-and-who-they-affect),
+consultada el 2026-11-18), `.claude/settings.local.json` es el ámbito
+«Project local»: solo te afecta a ti y solo en ese proyecto, Claude Code lo
+deja fuera de git cuando lo crea él (si lo creas a mano, añádelo tú a
+`.gitignore`), y en caso de choque gana a `.claude/settings.json` y a
+`~/.claude/settings.json`; por eso la status line de ese proyecto no pisa la
+de la 0.16 en los demás. En un repo git, Claude Code lo lee de la raíz del
+repo (en un worktree, de la del principal; en Windows, junto a
+`.claude/settings.json` de la carpeta donde arranca):
 
 ```json
 {
@@ -244,13 +254,20 @@ sustituye la status line de la 0.16 de `~/.claude/settings.json`) o a tu
 - **Cuándo cambia**: lo escriben `start`, `pausa` y `reanudar` (y lo borran
   `descartar`, `liberar` y `reasignar`); entre transiciones, la línea suma el
   reloj (con el tope de 45 min por hueco de F1-17) y, si pasan más de 45 min
-  sin commits, dice `sin actividad registrada hace N min`. Claude Code
+  sin commits, dice `sin actividad registrada hace N min`. Pasadas 8 h
+  activas en la fase (el último punto de la tabla de estimación), lo que
+  falta es el de ese punto y la línea añade `estimacion saturada: mas de
+  480 min en la fase`. Claude Code
   repinta la status line con sus eventos; para que corra el reloj aunque la
   sesión esté quieta, añade `"refreshInterval": 60` al bloque.
 - **En Windows**, Claude Code la lanza con Git Bash si está instalado (si
   no, con PowerShell); `tc2` tiene que estar en el PATH de ese shell.
 - **`tc2 status --seguir`** hace lo mismo en un terminal aparte: imprime la
   línea, y otra cada vez que cambia (mira cada 2 s); se para con Ctrl+C.
+  Si su salida no va a un terminal (`tc2 status --seguir | head -1`, otro
+  programa que lee), repite la misma línea cada 4 s sin cambios: es la única
+  forma de enterarse, sin código `unsafe`, de que quien leía cerró la tubería,
+  y entonces termina solo (código 0) en unos 4 s.
 - `tc2 desinstalar --maquina` quita la `statusLine` de
   `~/.claude/settings.json` si su `command` es exactamente
   `tc2 estado --stdin`; la de `.claude/settings.local.json` del proyecto la
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff efee1212394fb2f28018b63e4dc78f56a39b4f6e..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-084/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-084/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-084/revision/informe-revision-1-code-quality-reviewer.md         |   28 ++
 tareas/03-en-revision/TASK-084/revision/informe-revision-1-rust-wasm-reviewer.md            |   37 ++
 tareas/03-en-revision/TASK-084/revision/peticion-revision-1-code-quality-reviewer.md        |  152 +++++++
 tareas/03-en-revision/TASK-084/revision/peticion-revision-1-rust-wasm-reviewer.md           | 2198 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-084/tarea.md                                    |   10 +-
 7 files changed, 2422 insertions(+), 3 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff efee1212394fb2f28018b63e4dc78f56a39b4f6e..HEAD -- ":(literal)crates/tc2/tests/progreso.rs"
````

````
crates/tc2/tests/progreso.rs | 115 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 115 insertions(+)
````
