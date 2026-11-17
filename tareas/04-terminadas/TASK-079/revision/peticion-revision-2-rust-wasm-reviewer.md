# Peticion de revision — TASK-079 (ronda 2)

- Tarea: TASK-079 — F3-5b status y roadmap siempre frescos
- Rama revisada: feature/task-079-f3-5b-status-y-roadmap-siempre-frescos
- Rama base: develop
- Commit revisado (HEAD): ef977f1ce8994e9da5faa4b817fb815f24af56ef
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-079 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 67b8a3455d064a006a5464c96db0e3877b4a4834 (el commit revisado en la ronda anterior)
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

- tareas/03-en-revision/TASK-079/tarea.md
- tareas/03-en-revision/TASK-079/planificacion/
- crates/tc2/src/estado/frescura.rs
- crates/tc2/src/estado/lectura.rs
- crates/tc2/src/estado/montaje.rs
- crates/tc2/src/estado/transaccion.rs
- crates/tc2/src/i18n/catalogo/estado.rs
- crates/tc2/src/tablero.rs
- crates/tc2/tests/frescura.rs
- crates/tc2/tests/tablero.rs
- docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md

## Hallazgos de la ronda 1 pendientes de verificar

Tus 5 hallazgo(s) MENOR de la ronda 1 no se reverifican (los comprueba
quien coordina): recibes esta ronda porque la correccion
toca ficheros de tu dominio. Revisa en el diff de esta ronda lo que la correccion
cambia en tu terreno y si abre un fallo.

## Commits a revisar (git log 67b8a3455d064a006a5464c96db0e3877b4a4834..HEAD)

````
ef977f1 docs(TASK-079): adendas de F1-2 y F1-4 por los menores de la revision 1
68f054e test(TASK-079): marca recreada por la lectura, comparar con frescura, C10 y C11
2af3390 fix(TASK-079): rama reescrita y ref bloqueada no son sin-red; marca del futuro es nunca
ba24ba8 chore(TASK-079): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff 67b8a3455d064a006a5464c96db0e3877b4a4834..HEAD)

````diff
diff --git a/crates/tc2/src/estado/frescura.rs b/crates/tc2/src/estado/frescura.rs
index c88f3b3..fde31c3 100644
--- a/crates/tc2/src/estado/frescura.rs
+++ b/crates/tc2/src/estado/frescura.rs
@@ -25,6 +25,19 @@ pub(crate) enum Traida {
     Hecha,
     /// `origin` no tiene la rama de estado.
     SinRama,
+    /// `origin` tiene una rama de estado que no desciende de la que trajo
+    /// este clon (reescrita con force): el `fetch` sin `+` la rechaza
+    /// (`! [rejected] ... (non-fast-forward)`). No es falta de red.
+    Reescrita {
+        /// La linea `[rejected]` de git.
+        detalle: String,
+    },
+    /// La ref remota de este clon esta bloqueada (`cannot lock ref ...: File
+    /// exists`): otro `fetch` del mismo clon a la vez o un `.lock` huerfano.
+    Bloqueada {
+        /// La linea `cannot lock ref` de git.
+        detalle: String,
+    },
     /// No salio: la clasificacion y el stderr de git.
     Fallo {
         /// Por que.
@@ -42,10 +55,13 @@ impl Traida {
         match self {
             Self::Hecha => Remoto::Consultado,
             Self::SinRama => Remoto::SinRama,
+            Self::Reescrita { .. } => Remoto::Rechazado,
+            Self::Bloqueada { detalle } => Remoto::RefBloqueada(detalle.clone()),
+            // `RamaInexistente` no llega aqui: `traer_estado` la da como
+            // `SinRama`.
             Self::Fallo { fallo, .. } => match fallo {
                 FalloRemoto::SinRed => Remoto::SinRed,
                 FalloRemoto::Local => Remoto::ErrorLocal,
-                FalloRemoto::RamaInexistente => Remoto::SinRama,
                 otro => Remoto::Denegado(otro.motivo().to_owned()),
             },
         }
@@ -55,9 +71,14 @@ impl Traida {
 /// Trae solo la rama de estado (`fetch` de `refs/heads/taskcodetwo/tareas` a
 /// `refs/remotes/origin/taskcodetwo/tareas`, sin `+`) y, si sale bien,
 /// registra `ahora` en la marca. Es el `fetch` de cada intento de la
-/// transaccion y el de la lectura fresca: uno solo, sin copias. Un fallo al
-/// escribir la marca no se propaga: la hora se queda la anterior, nunca mas
-/// fresca de lo real. `Err` solo si no se pudo lanzar git.
+/// transaccion, el de la lectura fresca y el de `montar`: uno solo, sin
+/// copias. Un fallo al escribir la marca no se propaga: la hora se queda la
+/// anterior, nunca mas fresca de lo real. `Err` solo si no se pudo lanzar
+/// git.
+///
+/// Sin `-q`: con `-q`, git calla el rechazo non-fast-forward (stderr vacio,
+/// comprobado con git 2.56) y no se podria distinguir de otros fallos. En
+/// exito el stderr no se mira.
 pub(crate) fn traer_estado(
     git: &Git,
     principal: &Path,
@@ -65,25 +86,51 @@ pub(crate) fn traer_estado(
     ahora: u64,
 ) -> Result<Traida, Informe> {
     let refspec = format!("refs/heads/{RAMA}:refs/remotes/{REMOTO}/{RAMA}");
-    let s = correr(
-        git,
-        principal,
-        OP,
-        Subcomando::Fetch,
-        &["-q", REMOTO, &refspec],
-    )?;
+    let s = correr(git, principal, OP, Subcomando::Fetch, &[REMOTO, &refspec])?;
     if s.exito() {
         let _ = marca.registrar(ahora);
         return Ok(Traida::Hecha);
     }
-    Ok(match clasificar_fallo_remoto(&s.stderr) {
+    let primera = s.primera_linea_error();
+    Ok(clasificar_traida(s.stderr, primera))
+}
+
+/// `error` (1) de una escritura (transaccion o `montar`) que encuentra la
+/// rama de estado reescrita en `origin`: no se escribe nada ni queda
+/// pendiente (reintentar no lo arregla, y sin red no es).
+pub(crate) fn rama_reescrita(op: &str, detalle: &str) -> Informe {
+    Informe::nuevo(op, crate::contrato::Resultado::Error)
+        .con("motivo", "rama-reescrita")
+        .diagnostico(Mensaje::nuevo(&txt::RAMA_REESCRITA).con("detalle", detalle))
+}
+
+/// Clasifica el stderr de un `fetch` de la rama de estado que fallo. Antes
+/// que [`clasificar_fallo_remoto`] (que da como `sin-red` lo que no
+/// reconoce), dos fallos que no son de red: el rechazo non-fast-forward (la
+/// rama reescrita en `origin`; manda aunque haya ademas un `.lock`, porque
+/// es la causa) y la ref remota bloqueada por un `.lock`.
+fn clasificar_traida(stderr: String, primera: String) -> Traida {
+    let linea = |pred: &dyn Fn(&str) -> bool| {
+        stderr
+            .lines()
+            .map(str::trim)
+            .find(|l| pred(&l.to_ascii_lowercase()))
+            .map(str::to_owned)
+    };
+    if let Some(detalle) = linea(&|l| l.contains("[rejected]") && l.contains("non-fast-forward")) {
+        return Traida::Reescrita { detalle };
+    }
+    if let Some(detalle) = linea(&|l| l.contains("cannot lock ref") && l.contains("file exists")) {
+        return Traida::Bloqueada { detalle };
+    }
+    match clasificar_fallo_remoto(&stderr) {
         FalloRemoto::RamaInexistente => Traida::SinRama,
         fallo => Traida::Fallo {
             fallo,
-            primera: s.primera_linea_error(),
-            stderr: s.stderr,
+            primera,
+            stderr,
         },
-    })
+    }
 }
 
 /// Bytes como mucho del fichero de la marca (un numero y un salto de linea).
@@ -109,8 +156,19 @@ impl Marca {
     }
 
     /// Segundos desde 1970 del ultimo `fetch` con exito; `None` si no hay
-    /// marca o no se entiende (se trata como «nunca»).
+    /// marca, no se entiende o es posterior a ahora (se trata como «nunca»).
     pub fn leer(&self) -> Option<u64> {
+        self.leer_en(crate::identidad::ahora())
+    }
+
+    /// Como [`Marca::leer`] con `ahora` dado. Una hora posterior a `ahora`
+    /// (un reloj que iba adelantado al escribirla o un fichero tocado a
+    /// mano) es «nunca»: mostrarla haria la lectura mas fresca de lo real.
+    pub fn leer_en(&self, ahora: u64) -> Option<u64> {
+        self.leer_crudo().filter(|s| *s <= ahora)
+    }
+
+    fn leer_crudo(&self) -> Option<u64> {
         use std::io::Read;
         let mut texto = String::new();
         std::fs::File::open(&self.ruta)
@@ -150,6 +208,13 @@ pub enum Remoto {
     Consultado,
     /// Sin red (o un fallo que no se reconoce): se muestra lo ultimo traido.
     SinRed,
+    /// `origin` tiene la rama de estado reescrita (no desciende de la que
+    /// trajo este clon) y el `fetch` sin `+` no la trae.
+    Rechazado,
+    /// La ref remota de este clon esta bloqueada por un `.lock` (otro git a
+    /// la vez o uno huerfano); lleva la linea de git. Sale como
+    /// `error-local`, con su propio aviso.
+    RefBloqueada(String),
     /// `origin` responde pero no deja leer; el motivo es el de
     /// [`crate::git::FalloRemoto::motivo`] (`credenciales`,
     /// `remoto-no-existe`...).
@@ -168,8 +233,9 @@ impl Remoto {
         match self {
             Self::Consultado => "consultado",
             Self::SinRed => "sin-red",
+            Self::Rechazado => "rechazado",
             Self::Denegado(_) => "denegado",
-            Self::ErrorLocal => "error-local",
+            Self::ErrorLocal | Self::RefBloqueada(_) => "error-local",
             Self::SinRama => "sin-rama",
             Self::SinRemoto => "sin-remoto",
         }
@@ -180,8 +246,12 @@ impl Remoto {
         Some(match self {
             Self::Consultado | Self::SinRemoto => return None,
             Self::SinRed => Mensaje::nuevo(&txt::PORQUE_SIN_RED),
+            Self::Rechazado => Mensaje::nuevo(&txt::PORQUE_RECHAZADO),
             Self::Denegado(m) => Mensaje::nuevo(&txt::PORQUE_DENEGADO).con("motivo", m.clone()),
             Self::ErrorLocal => Mensaje::nuevo(&txt::PORQUE_ERROR_LOCAL),
+            Self::RefBloqueada(d) => {
+                Mensaje::nuevo(&txt::PORQUE_REF_BLOQUEADA).con("detalle", d.clone())
+            }
             Self::SinRama => Mensaje::nuevo(&txt::PORQUE_SIN_RAMA),
         })
     }
@@ -325,6 +395,112 @@ mod tests {
         Ok(())
     }
 
+    /// H2 (revision 1 de TASK-079): una marca con hora posterior a `ahora`
+    /// es «nunca», tambien sin red: nunca mas fresca de lo real.
+    #[test]
+    fn marca_del_futuro_es_nunca() -> std::io::Result<()> {
+        let comun = carpeta("futuro");
+        let m = Marca::en(&comun);
+        m.registrar(T + 1)?;
+        assert_eq!(m.leer_en(T), None, "un segundo despues de ahora");
+        assert_eq!(m.leer_en(T + 1), Some(T + 1), "igual a ahora vale");
+        assert_eq!(m.leer_en(T + 100), Some(T + 1));
+        m.registrar(99_999_999_999)?;
+        assert_eq!(m.leer(), None, "el caso del informe: ano 5138");
+        let f = Frescura {
+            remoto: Remoto::SinRed,
+            sincronizado: m.leer(),
+        };
+        assert_eq!(f.texto(), "nunca");
+        let _ = std::fs::remove_dir_all(&comun);
+        Ok(())
+    }
+
+    /// H1 (revision 1 de TASK-079): el stderr real de git 2.56 (sin `-q`)
+    /// del rechazo non-fast-forward y del `.lock`, y que lo demas sigue con
+    /// la clasificacion de siempre.
+    #[test]
+    fn clasifica_rechazo_y_ref_bloqueada() {
+        let rechazo = "From ssh://git@github.com/o/r\n ! [rejected]        taskcodetwo/tareas -> origin/taskcodetwo/tareas  (non-fast-forward)\n";
+        let lock = "error: cannot lock ref 'refs/remotes/origin/taskcodetwo/tareas': Unable to create 'C:/x/.git/refs/remotes/origin/taskcodetwo/tareas.lock': File exists.\n\nAnother git process seems to be running in this repository, or the lock file may be stale\nFrom ../remoto\n ! a5d3ff0..95116cd  taskcodetwo/tareas -> origin/taskcodetwo/tareas  (unable to update local ref)\n";
+        let ambos = format!(
+            "{rechazo}error: update_ref failed for ref 'refs/remotes/origin/taskcodetwo/tareas': cannot lock ref 'refs/remotes/origin/taskcodetwo/tareas': Unable to create 'C:/x/tareas.lock': File exists.\n"
+        );
+        let c = |s: &str| clasificar_traida(s.to_owned(), "p".to_owned());
+        assert_eq!(
+            c(rechazo),
+            Traida::Reescrita {
+                detalle: "! [rejected]        taskcodetwo/tareas -> origin/taskcodetwo/tareas  (non-fast-forward)".to_owned()
+            }
+        );
+        assert!(
+            matches!(c(&ambos), Traida::Reescrita { .. }),
+            "manda el rechazo"
+        );
+        let bloqueada = c(lock);
+        assert!(
+            matches!(&bloqueada, Traida::Bloqueada { detalle }
+                if detalle.starts_with("error: cannot lock ref 'refs/remotes/origin/")),
+            "{bloqueada:?}"
+        );
+        assert_eq!(c(lock).remoto().texto(), "error-local");
+        assert_eq!(c(rechazo).remoto().texto(), "rechazado");
+        // Un `cannot lock ref` por permisos no es un `.lock`: sigue `Local`.
+        let permiso = "error: cannot lock ref 'refs/heads/x': Permission denied";
+        assert_eq!(c(permiso).remoto(), Remoto::ErrorLocal);
+        assert_eq!(c("").remoto(), Remoto::SinRed);
+        assert_eq!(
+            c("fatal: couldn't find remote ref refs/heads/taskcodetwo/tareas"),
+            Traida::SinRama
+        );
+        assert_eq!(
+            c("git@github.com: Permission denied (publickey).\nfatal: Could not read from remote repository.").remoto(),
+            Remoto::Denegado("credenciales".to_owned())
+        );
+    }
+
+    /// Los avisos de los dos motivos nuevos, en los dos idiomas.
+    #[test]
+    fn aviso_de_rechazado_y_ref_bloqueada() {
+        let f = |remoto| Frescura {
+            remoto,
+            sincronizado: Some(T),
+        };
+        let textos = |remoto, ahora| {
+            let a = f(remoto).aviso(ahora);
+            (
+                a.as_ref().map(|m| m.texto(Idioma::Es)).unwrap_or_default(),
+                a.map(|m| m.texto(Idioma::En)).unwrap_or_default(),
+            )
+        };
+        let (es, en) = textos(Remoto::Rechazado, T + 180);
+        assert_eq!(
+            es,
+            "aviso: origin tiene la rama de estado reescrita (no sigue a la que trajo este clon) y git no la trae sin forzar; se muestra lo que este clon trajo el 2026-11-17T16:04:12Z (hace 3 min)"
+        );
+        assert!(
+            en.starts_with("warning: origin has the state branch rewritten"),
+            "{en}"
+        );
+        let (es, en) = textos(
+            Remoto::RefBloqueada("error: cannot lock ref 'x'".to_owned()),
+            T,
+        );
+        assert_eq!(
+            es,
+            "aviso: la ref de la rama de estado de este clon esta bloqueada por otro git a la vez o un .lock huerfano (error: cannot lock ref 'x'); se muestra lo que este clon trajo el 2026-11-17T16:04:12Z (hace menos de 1 min)"
+        );
+        assert!(
+            en.starts_with("warning: this clone's state branch ref is locked"),
+            "{en}"
+        );
+        let base = Informe::nuevo("status", Resultado::Ok);
+        assert_eq!(
+            f(Remoto::Rechazado).anotar(base).linea(),
+            "status ok remoto=rechazado sincronizado=2026-11-17T16:04:12Z"
+        );
+    }
+
     #[test]
     fn hace_en_minutos_horas_y_dias() {
         let es = |a: u64, b: u64| hace(a, b).texto(Idioma::Es);
diff --git a/crates/tc2/src/estado/lectura.rs b/crates/tc2/src/estado/lectura.rs
index de2b626..6d1094a 100644
--- a/crates/tc2/src/estado/lectura.rs
+++ b/crates/tc2/src/estado/lectura.rs
@@ -6,7 +6,7 @@
 use std::collections::{BTreeMap, BTreeSet};
 use std::path::{Path, PathBuf};
 
-use super::frescura::{Frescura, Marca, Remoto, traer_estado};
+use super::frescura::{Frescura, Marca, Remoto, Traida, traer_estado};
 use super::transaccion::Fallo;
 use super::{CARPETA, OP, RAMA};
 use crate::contrato::{Informe, Resultado};
@@ -188,17 +188,49 @@ pub fn leer_fresco<'a>(
     let (principal, referencia, remoto) = lectura_de(git, dir)?;
     let marca = Marca::en(&dir_comun(git, &principal, OP)?);
     let remoto = if remoto {
-        traer_estado(git, &principal, &marca, ahora)?.remoto()
+        con_reintento(&ESPERAS_REF_BLOQUEADA, || {
+            traer_estado(git, &principal, &marca, ahora)
+        })?
+        .remoto()
     } else {
         Remoto::SinRemoto
     };
     let frescura = Frescura {
         remoto,
-        sincronizado: marca.leer(),
+        sincronizado: marca.leer_en(ahora),
     };
     Ok((vista_de(git, &principal, &referencia)?, frescura))
 }
 
+/// Esperas antes de repetir el `fetch` de la lectura si la ref remota esta
+/// bloqueada (revision 1 de TASK-079, H4): la lectura no toma el cerrojo de
+/// estado, asi que su `fetch` puede coincidir con el de una transaccion u
+/// otra lectura del mismo clon, que sueltan el `.lock` en milisegundos. Un
+/// `.lock` huerfano sigue ahi tras las esperas y sale `remoto=error-local`.
+/// La transaccion no reintenta: su ref bloqueada es `error` (no hay
+/// reintento de `fetch` que heredar alli).
+const ESPERAS_REF_BLOQUEADA: [std::time::Duration; 2] = [
+    std::time::Duration::from_millis(200),
+    std::time::Duration::from_millis(500),
+];
+
+/// `traer` y, mientras salga [`Traida::Bloqueada`], otra vez tras cada
+/// espera de `esperas`.
+fn con_reintento(
+    esperas: &[std::time::Duration],
+    mut traer: impl FnMut() -> Result<Traida, Informe>,
+) -> Result<Traida, Informe> {
+    let mut t = traer()?;
+    for e in esperas {
+        if !matches!(t, Traida::Bloqueada { .. }) {
+            break;
+        }
+        std::thread::sleep(*e);
+        t = traer()?;
+    }
+    Ok(t)
+}
+
 /// Hora de la ultima sincronizacion del clon de `dir`, sin `fetch` (la de
 /// `doctor`, D4a). `None` si no se sincronizo nunca o no se puede saber.
 pub fn sincronizado_en(git: &Git, dir: &Path) -> Option<u64> {
@@ -264,3 +296,44 @@ fn vista_de<'a>(git: &'a Git, principal: &Path, referencia: &str) -> Result<Vist
             )),
     }
 }
+
+#[cfg(test)]
+mod tests {
+    use super::{Traida, con_reintento};
+    use std::time::Duration;
+
+    fn bloqueada() -> Traida {
+        Traida::Bloqueada {
+            detalle: "error: cannot lock ref 'x'".to_owned(),
+        }
+    }
+
+    /// H4 (revision 1 de TASK-079): la lectura repite el `fetch` mientras la
+    /// ref este bloqueada, como mucho una vez por espera, y para en cuanto
+    /// sale otra cosa.
+    #[test]
+    fn reintenta_solo_la_ref_bloqueada() {
+        let esperas = [Duration::ZERO, Duration::ZERO];
+        // Se suelta el `.lock` al segundo intento.
+        let mut n = 0;
+        let t = con_reintento(&esperas, || {
+            n += 1;
+            Ok(if n < 2 { bloqueada() } else { Traida::Hecha })
+        });
+        assert_eq!((t.ok(), n), (Some(Traida::Hecha), 2));
+        // Un `.lock` huerfano: tres intentos y se queda bloqueada.
+        let mut n = 0;
+        let t = con_reintento(&esperas, || {
+            n += 1;
+            Ok(bloqueada())
+        });
+        assert_eq!((t.ok(), n), (Some(bloqueada()), 3));
+        // Lo demas no se repite.
+        let mut n = 0;
+        let t = con_reintento(&esperas, || {
+            n += 1;
+            Ok(Traida::SinRama)
+        });
+        assert_eq!((t.ok(), n), (Some(Traida::SinRama), 1));
+    }
+}
diff --git a/crates/tc2/src/estado/montaje.rs b/crates/tc2/src/estado/montaje.rs
index 6f63724..eacccbb 100644
--- a/crates/tc2/src/estado/montaje.rs
+++ b/crates/tc2/src/estado/montaje.rs
@@ -12,6 +12,7 @@
 use std::path::{Path, PathBuf};
 
 use super::cerrojo::{Cerrojo, ErrorCerrojo};
+use super::frescura::{Marca, Traida, rama_reescrita, traer_estado};
 use super::lectura::Estado;
 use super::transaccion::{Contexto, Fallo, Operacion, Politica, transaccion};
 use super::{CARPETA, RAMA};
@@ -228,29 +229,26 @@ fn paso(git: &Git, dir: &Path, politica: Politica, puede_crear: bool) -> Result<
 
     let mut crear = previsto == Previsto::Crear;
     if previsto == Previsto::DelRemoto {
-        let refspec = format!("refs/heads/{RAMA}:{remota}");
-        let s = correr(
-            git,
-            &principal,
-            OP,
-            Subcomando::Fetch,
-            &["-q", REMOTO, &refspec],
-        )?;
-        if s.exito() {
-            exigir(git, &principal, OP, Subcomando::Branch, &[RAMA, &remota])?;
-        } else {
-            match clasificar_fallo_remoto(&s.stderr) {
-                // Desaparecio del remoto entre la lectura y el fetch.
-                FalloRemoto::RamaInexistente => crear = true,
-                FalloRemoto::SinRed => return Ok(sin_red(&s.primera_linea_error())),
+        // El mismo `fetch` que la transaccion y la lectura fresca: con
+        // exito renueva la marca de la ultima sincronizacion.
+        let marca = Marca::en(&comun);
+        match traer_estado(git, &principal, &marca, crate::identidad::ahora())? {
+            Traida::Hecha => {
+                exigir(git, &principal, OP, Subcomando::Branch, &[RAMA, &remota])?;
+            }
+            // Desaparecio del remoto entre la lectura y el fetch.
+            Traida::SinRama => crear = true,
+            Traida::Reescrita { detalle } => return Err(rama_reescrita(OP, &detalle)),
+            Traida::Bloqueada { detalle } => {
+                return Err(crate::git::remoto::error_local(OP, &detalle));
+            }
+            Traida::Fallo { fallo, primera, .. } => match fallo {
+                FalloRemoto::SinRed => return Ok(sin_red(&primera)),
                 FalloRemoto::Local => {
-                    return Err(crate::git::remoto::error_local(
-                        OP,
-                        &s.primera_linea_error(),
-                    ));
+                    return Err(crate::git::remoto::error_local(OP, &primera));
                 }
-                otro => return Ok(denegada(otro, &s.primera_linea_error())),
-            }
+                otro => return Ok(denegada(otro, &primera)),
+            },
         }
     }
     if crear {
diff --git a/crates/tc2/src/estado/transaccion.rs b/crates/tc2/src/estado/transaccion.rs
index 2cd1f44..42af1a0 100644
--- a/crates/tc2/src/estado/transaccion.rs
+++ b/crates/tc2/src/estado/transaccion.rs
@@ -13,7 +13,7 @@ use std::path::Path;
 use std::time::Duration;
 
 use super::cerrojo::{Cerrojo, ErrorCerrojo};
-use super::frescura::{Marca, Traida, traer_estado};
+use super::frescura::{Marca, Traida, rama_reescrita, traer_estado};
 use super::lectura::{Estado, Vista};
 use super::pendientes::{self, ANOTACION, Destino};
 use super::{CARPETA, OP, RAMA};
@@ -283,6 +283,14 @@ fn intento(
             // que se monto sin remoto y al que luego se anadio `origin`),
             // se parte de ella para publicar su historia, no de cero.
             Traida::SinRama => commit_de(git, principal, OP, &rama_local())?,
+            // Ni una ni otra es falta de red (revision 1 de TASK-079, H1):
+            // no se guarda como pendiente. La rama reescrita no la arregla
+            // reintentar; la ref bloqueada es un fallo de este clon, como
+            // `FalloRemoto::Local`.
+            Traida::Reescrita { detalle } => return Err(rama_reescrita(OP, &detalle)),
+            Traida::Bloqueada { detalle } => {
+                return Err(crate::git::remoto::error_local(OP, &detalle));
+            }
             Traida::Fallo {
                 fallo,
                 stderr,
diff --git a/crates/tc2/src/i18n/catalogo/estado.rs b/crates/tc2/src/i18n/catalogo/estado.rs
index d6b9b8d..52181af 100644
--- a/crates/tc2/src/i18n/catalogo/estado.rs
+++ b/crates/tc2/src/i18n/catalogo/estado.rs
@@ -79,6 +79,10 @@ crate::i18n::textos! {
         es: "{detalle}; queda pendiente de publicar: se publicara en la siguiente operacion con red",
         en: "{detalle}; pending publication: it will be published in the next operation with network access",
     }
+    RAMA_REESCRITA {
+        es: "{detalle}; origin tiene la rama de estado reescrita y no sigue a la que trajo este clon: no se ha escrito nada; averigua quien la reescribio antes de seguir",
+        en: "{detalle}; origin has the state branch rewritten and it does not follow the one this clone fetched: nothing was written; find out who rewrote it before going on",
+    }
     NO_SE_PUDO_ADELANTAR {
         es: "no se pudo adelantar {rama}: {error}",
         en: "could not fast-forward {rama}: {error}",
@@ -105,6 +109,14 @@ crate::i18n::textos! {
         es: "no se pudo traer la rama de estado por un fallo de este clon",
         en: "the state branch could not be fetched because of a failure in this clone",
     }
+    PORQUE_RECHAZADO {
+        es: "origin tiene la rama de estado reescrita (no sigue a la que trajo este clon) y git no la trae sin forzar",
+        en: "origin has the state branch rewritten (it does not follow the one this clone fetched) and git does not fetch it without forcing",
+    }
+    PORQUE_REF_BLOQUEADA {
+        es: "la ref de la rama de estado de este clon esta bloqueada por otro git a la vez o un .lock huerfano ({detalle})",
+        en: "this clone's state branch ref is locked by another git at the same time or a stale .lock ({detalle})",
+    }
     PORQUE_SIN_RAMA {
         es: "origin no tiene la rama de estado",
         en: "origin does not have the state branch",
diff --git a/crates/tc2/src/tablero.rs b/crates/tc2/src/tablero.rs
index de9baba..bca8168 100644
--- a/crates/tc2/src/tablero.rs
+++ b/crates/tc2/src/tablero.rs
@@ -10,7 +10,8 @@
 //!
 //! - `status [--json]`: una linea de equipo; no consulta la plataforma. Con
 //!   `--json`, `status ok en-diseno=... en-curso=... en-revision=... listas=...
-//!   commit=... remoto=... sincronizado=... pendientes=N [tareas=...]`.
+//!   commit=... remoto=... sincronizado=... pendientes=N [tareas=...]`. En
+//!   texto, `· pendientes N` al final solo si N > 0.
 //! - `board [--sin-plataforma] [--json]`: la vista completa por grupos (en
 //!   diseno, en curso, en revision y listas para empezar), con ID, titulo,
 //!   quien y PR; terminadas y descartadas solo como recuento.
@@ -441,8 +442,13 @@ pub fn status_en(e: &Entorno<'_>) -> Result<(String, Informe, Vec<Mensaje>), Inf
     }
     i = l.frescura.anotar(i);
     let mut avisos: Vec<Mensaje> = l.frescura.aviso(l.ahora).into_iter().collect();
+    let mut linea = linea_status(t);
     if let Some(local) = l.local {
         i = i.con("pendientes", local.pendientes.to_string());
+        // En texto solo si hay alguna: es lo que falta por publicar.
+        if local.pendientes > 0 {
+            let _ = write!(linea, " · pendientes {}", local.pendientes);
+        }
         if let Some(tareas) = local.tareas {
             i = i.con("tareas", tareas);
             let m = if tareas == "diverge" {
@@ -453,7 +459,7 @@ pub fn status_en(e: &Entorno<'_>) -> Result<(String, Informe, Vec<Mensaje>), Inf
             avisos.push(m.con("carpeta", estado::CARPETA));
         }
     }
-    Ok((linea_status(t), i, avisos))
+    Ok((linea, i, avisos))
 }
 
 /// Que hace `board` con la plataforma.
diff --git a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
index 2d3c301..5ae690e 100644
--- a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
+++ b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
@@ -568,10 +568,38 @@ D1a..D6a del plan de TASK-079, aceptadas por Carlos para todo el H3):
   tiene cambios sin publicar, que impedirán adelantarla).
 - **Riesgos aceptados**: ~1 s más por orden contra GitHub; sin tiempo límite
   en el puerto, un host mudo cuelga la lectura lo que tarde TCP (igual que la
-  transacción); dos `fetch` a la vez en el mismo clon pueden dar `cannot lock
-  ref`, que se clasifica como `sin-red` (se muestra lo último traído con
-  aviso; no se escribe nada).
-
-Evidencia: `crates/tc2/tests/frescura.rs` (C1..C9 de TASK-079), los
-unitarios de `estado/frescura.rs` y la suite de la transacción sin cambios
+  transacción).
+- **Fallos del `fetch` que no son de red** (revisión 1 de TASK-079, H1 y
+  H4). El `fetch` de la rama de estado va sin `-q`: con `-q`, git 2.56 calla
+  el rechazo non-fast-forward (stderr vacío) y salía `sin-red`. Antes de la
+  clasificación de siempre se reconocen dos casos:
+  - **Rama reescrita en `origin`** (`! [rejected] … (non-fast-forward)`: la
+    rama de estado del remoto no desciende de la que trajo el clon). La
+    lectura sale 0 con `remoto=rechazado` y su aviso; una escritura
+    (transacción o `montar`) sale `error` (1) con `motivo=rama-reescrita` y
+    la línea de git, sin escribir nada ni dejar pendiente (reintentar no lo
+    arregla). Antes la transacción lo tomaba por `sinred` y guardaba la
+    operación como pendiente que nunca se publicaría.
+  - **Ref remota bloqueada** (`cannot lock ref …: File exists`: otro `fetch`
+    del mismo clon a la vez o un `.lock` huérfano). La lectura, que no toma
+    el cerrojo de estado, repite el `fetch` dos veces (a 0,2 s y 0,5 s) y,
+    si sigue bloqueada, sale 0 con `remoto=error-local` y un aviso que dice
+    el `.lock` y la línea de git. Una escritura sale `error` (1) con
+    `motivo=error-local`, como cualquier fallo del propio clon, sin dejar
+    pendiente; no se le añade reintento (no lo tenía). Riesgo aceptado: si
+    una lectura y una escritura del mismo clon traen a la vez, la escritura
+    puede salir `error` y hay que repetirla (estresado en la revisión con 30
+    y 8 procesos: 0 fallos). Un `cannot lock ref` por permisos sigue siendo
+    el `error-local` de siempre.
+- **La hora nunca más fresca de lo real** (H2): una marca con hora
+  posterior a ahora (reloj adelantado al escribirla, fichero tocado a mano)
+  se lee como «nunca».
+- **`montar` trae con el mismo `fetch`** (M3): cuando monta la rama desde
+  `origin` usa `traer_estado`, así que también renueva la marca.
+
+Evidencia: `crates/tc2/tests/frescura.rs` (C1..C9 de TASK-079; C10 y C11 de
+la revisión 1, rama reescrita y `.lock` huérfano), los unitarios de
+`estado/frescura.rs` (`clasifica_rechazo_y_ref_bloqueada`,
+`marca_del_futuro_es_nunca`) y de `estado/lectura.rs`
+(`reintenta_solo_la_ref_bloqueada`), y la suite de la transacción
 (`estado_*`, `alta`, `toma`, `ciclo`, `importar`).
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index 548be13..a1019de 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -9,7 +9,7 @@
 - Adenda (2026-11-17, TASK-033, F3-6): `start [--json] TASK-NNN` compone `fetch`, `validar-base`, `comprobar-id`, `commit-estado`, `crear-rama-y-worktree` y `reanudar` sin operaciones nuevas (la base de un `hotfix` es el último tag por versión de `refs/tags/` contenido en `origin/main`, leído con `for-each-ref --merged`, o su punta). Línea de éxito: `start ok id=… rama=… worktree=… base=… estado=<commit de la toma> asignado=<login>`; dueño con la tarea empezada: `sin-cambios` o `ok creado=rama,worktree` sin escribir estado. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `tomada` (con `asignado=<login>`), `asignado-heredado`, `terminada`, `descartada`, `estado-no-valido`, `dependencias-pendientes` (con `dependencias=`), `rama-invalida`, `tipo-release`, `ocupada` (con `ramas=`), `carpeta-ocupada` (la carpeta del worktree existe y no es el worktree registrado de la rama; se comprueba antes de publicar la toma, sin `estado=`) e `identidad`; `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas`; sin red, `sinred pendiente=no` (4)
 - Adenda (2026-11-17, TASK-034, F3-7): `descartar [--json] TASK-NNN "<motivo>"`, `pausa [--json] TASK-NNN` y `reanudar [--json] TASK-NNN` componen `fetch` (solo `descartar`), `commit-estado` y `quitar-worktree` (solo `descartar`, tras publicar) sin operaciones nuevas. Líneas: `descartar ok id=… commit=… intentos=… tareas=… worktree=quitado|ninguno|conservado [worktree-motivo=cambios-sin-guardar|sin-publicar|sin-rama|en-uso|error]` (el worktree conservado no hace fallar la orden: el descarte ya está publicado y la rama no se toca nunca); `pausa ok id=… commit=… intentos=… tareas=…` y `reanudar ok id=… commit=… intentos=… tareas=…` (las claves de la transacción, como `descartar`, sin `worktree`). `sin-cambios` (0): descartar una descartada (vale el motivo de entonces y se intenta quitar el worktree), pausar una en pausa o reanudar una que no lo está. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `de-otra-persona` (con `asignado=<login>`; D2: puede quien la lleva o cualquiera si no la lleva nadie), `asignado-heredado`, `terminada`, `descartada` (pausa y reanudar), `estado-no-valido` e `identidad`; `start` gana `pausada` (con `arreglo=tc2 reanudar`). `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas` y, en `descartar`, `sin-motivo` y `motivo-invalido` (vacío, solo espacios, saltos de línea o caracteres de control, o más de 500 caracteres). Sin red: `descartar sinred pendiente=no` (4); `pausa`/`reanudar sinred pendiente=si` (4), que la siguiente transacción con red publica (adenda de TASK-034 en F1-2)
 - Adenda (2026-11-17, TASK-035, F3-8): `doctor [--json]` compone lecturas sin escribir nada (ni `fetch`): `git version` entra en la lista permitida (decisión 3; solo lectura), y la lista cerrada de la plataforma gana `leer-plan` (detalle en la [adenda de F1-13](F1-13-adaptador-de-plataforma.md)). Salida: primera línea `doctor <ok|rechazada> fallos=N avisos=N no-comprobado=N plataforma=ok|no-configurada|sin-credenciales|sin-red|error [plan=… impuestas=N sustituidas=N no-disponibles=N [no-comprobadas=N]] [commit=…]` (`no-comprobadas` solo si alguna garantía cuya configuración se lee no se pudo leer: no cuenta como impuesta; ronda 1 de TASK-035); detrás, los hallazgos que no son `ok` (`<zona> <id> <falla|aviso|no-comprobado> [motivo=…] [claves] [arreglo=…]`), los peores primero, y si no caben en 5 líneas, 3 y `y N mas: tc2 doctor --json`. `--json`: una línea con las claves y `hallazgos` (todos, también los `ok`). Código 3 (`rechazada`) con alguna `falla` (D5a de TASK-035) y 0 con avisos o no-comprobados; fuera de un repo, `doctor rechazada motivo=no-es-repo` (3); argumentos de más, `uso motivo=argumentos` (2; `--recursos` llega con F3-9). Git por debajo de 2.56: `aviso motivo=git-no-verificado` (D3a de TASK-035); sin git o con una versión que no se entiende, `falla`. Diagnóstico de una línea en stderr solo con alguna `falla`
-- Adenda (2026-11-17, TASK-079, F3-5b): `status`, `board`, `next` y `roadmap` (y `roadmap --comparar`) traen antes la rama de estado con el mismo `fetch -q origin refs/heads/taskcodetwo/tareas:refs/remotes/origin/taskcodetwo/tareas` de la transacción (sin operaciones nuevas, sin cerrojo) y solo escriben esa ref remota y `.git/taskcodetwo/sincronizacion`. Claves nuevas en las cuatro, detrás de las que ya tenían: `remoto=consultado|sin-red|denegado|error-local|sin-rama|sin-remoto` (cómo fue el `fetch`; sin `origin` no se hace) y `sincronizado=<AAAA-MM-DDTHH:MM:SSZ>|nunca` (hora UTC del último `fetch` con éxito, de una lectura o de una transacción). Un fallo del remoto no cambia el resultado: sale `ok` (0) con lo último traído y un aviso de una línea en stderr con la hora y «hace N» (o que no se sincronizó nunca). `status` gana además `pendientes=N` (refs de `refs/taskcodetwo/pendientes/`) y, si la carpeta montada no se puede adelantar, `tareas=con-cambios|diverge` con su aviso (código 0). En texto, la hora va al final de la línea de `status` (`· sincronizado …`) y de la cabecera de `board`; en `roadmap`, a stderr (`estado sincronizado con origin el … (hace …)`), porque su stdout sigue siendo el `ROADMAP.md` byte a byte. `doctor` no hace `fetch` y añade `sincronizado=` (de la marca) detrás de `commit=`
+- Adenda (2026-11-17, TASK-079, F3-5b): `status`, `board`, `next` y `roadmap` (y `roadmap --comparar`) traen antes la rama de estado con el mismo `fetch origin refs/heads/taskcodetwo/tareas:refs/remotes/origin/taskcodetwo/tareas` de la transacción y de `montar` (sin `-q`, para que git no calle el rechazo non-fast-forward; sin operaciones nuevas, sin cerrojo) y solo escriben esa ref remota y `.git/taskcodetwo/sincronizacion`. Claves nuevas en las cuatro, detrás de las que ya tenían: `remoto=consultado|sin-red|rechazado|denegado|error-local|sin-rama|sin-remoto` (cómo fue el `fetch`; sin `origin` no se hace; `rechazado`: la rama de estado de `origin` está reescrita y no desciende de la traída; `error-local` incluye la ref remota bloqueada por un `.lock`, que la lectura reintenta dos veces antes y nombra en su aviso; una escritura que encuentra la rama reescrita sale `error` con `motivo=rama-reescrita`, y la ref bloqueada, `error` con `motivo=error-local`, sin quedar pendiente) y `sincronizado=<AAAA-MM-DDTHH:MM:SSZ>|nunca` (hora UTC del último `fetch` con éxito, de una lectura, de una transacción o de `montar`; una marca con hora posterior a ahora cuenta como `nunca`). Un fallo del remoto no cambia el resultado: sale `ok` (0) con lo último traído y un aviso de una línea en stderr con la hora y «hace N» (o que no se sincronizó nunca). `status` gana además `pendientes=N` (refs de `refs/taskcodetwo/pendientes/`) y, si la carpeta montada no se puede adelantar, `tareas=con-cambios|diverge` con su aviso (código 0). En texto, la hora va al final de la línea de `status` (`· sincronizado …`, seguida de `· pendientes N` solo si N > 0) y de la cabecera de `board`; en `roadmap`, a stderr (`estado sincronizado con origin el … (hace …)`), porque su stdout sigue siendo el `ROADMAP.md` byte a byte. `doctor` no hace `fetch` y añade `sincronizado=` (de la marca) detrás de `commit=`
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 67b8a3455d064a006a5464c96db0e3877b4a4834..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-079/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-079/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-079/revision/informe-revision-1-code-quality-reviewer.md         |   21 ++
 tareas/03-en-revision/TASK-079/revision/informe-revision-1-rust-wasm-reviewer.md            |   21 ++
 tareas/03-en-revision/TASK-079/revision/peticion-revision-1-code-quality-reviewer.md        |  143 +++++++++
 tareas/03-en-revision/TASK-079/revision/peticion-revision-1-rust-wasm-reviewer.md           | 1741 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-079/tarea.md                                    |    6 +-
 7 files changed, 1930 insertions(+), 2 deletions(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff 67b8a3455d064a006a5464c96db0e3877b4a4834..HEAD -- ":(literal)crates/tc2/tests/frescura.rs" ":(literal)crates/tc2/tests/tablero.rs"
````

````
crates/tc2/tests/frescura.rs | 148 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++---
 crates/tc2/tests/tablero.rs  |  15 ++++++++++++++-
 2 files changed, 159 insertions(+), 4 deletions(-)
````
