# Peticion de revision — TASK-125 (ronda 1)

- Tarea: TASK-125 — Seguimiento de F3-10: quitar la copia vieja de la regla de actividad
- Rama revisada: fix/task-125-seguimiento-de-f3-10-quitar-la-copia-vie
- Rama base: develop
- Commit revisado (HEAD): 41944471c24da878a2e28fc776ebb4a2b3aab7e5
- Fecha: 2026-11-18
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-125 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-125/tarea.md
- tareas/03-en-revision/TASK-125/planificacion/
- crates/tc2/src/ciclo/liberar.rs
- crates/tc2/src/situacion.rs

## Commits a revisar (git log develop..HEAD)

````
4194447 chore(TASK-125): autorrevision
d52d640 fix(TASK-125): quitar situacion::actividad, la copia vieja de la regla
6345b36 chore(TASK-125): tarea en curso
````

## Diff de tu dominio (2 fichero(s) de 2; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/ciclo/liberar.rs b/crates/tc2/src/ciclo/liberar.rs
index e0d7397..7d97493 100644
--- a/crates/tc2/src/ciclo/liberar.rs
+++ b/crates/tc2/src/ciclo/liberar.rs
@@ -575,6 +575,9 @@ mod tests {
             Some(rama)
         );
         assert_eq!(actividad_conocida(None, None, AHORA), None);
+        // Sin fila, solo el commit de la rama (TASK-125: lo cubria el test
+        // de `situacion::actividad`, que ya no existe).
+        assert_eq!(actividad_conocida(None, Some(rama), AHORA), Some(rama));
         let borde = AHORA + MARGEN_FUTURO;
         assert_eq!(
             actividad_conocida(Some(fila), Some(borde), AHORA),
diff --git a/crates/tc2/src/situacion.rs b/crates/tc2/src/situacion.rs
index 40c774b..5c5e2e4 100644
--- a/crates/tc2/src/situacion.rs
+++ b/crates/tc2/src/situacion.rs
@@ -39,16 +39,10 @@ pub struct Ficha {
 /// Segundos de un dia.
 const DIA: u64 = 86_400;
 
-/// La actividad de una tarea (F3-10, D2a): el instante mas reciente entre
-/// la ultima fila de sus transiciones (tambien `pausa` y `reanudar`: la
-/// pausa no exime) y el ultimo commit de `origin/<rama>` (`commit_rama`).
-/// Sin ninguna de las dos, `None`: no se sabe.
-pub fn actividad(ficha: &Ficha, commit_rama: Option<u64>) -> Option<u64> {
-    ficha.ultima_transicion.max(commit_rama)
-}
-
-/// Dias completos sin actividad desde `ultima` hasta `ahora`. Una actividad
-/// del futuro (reloj adelantado) cuenta como de hoy: 0.
+/// Dias completos sin actividad desde `ultima` hasta `ahora`. `ultima` es la
+/// actividad ya filtrada por [`crate::ciclo::liberar::actividad_conocida`]
+/// (F3-10, D2a): solo puede ir por delante de `ahora` dentro de su
+/// `MARGEN_FUTURO`, y entonces la resta satura y da 0.
 pub fn dias_sin_actividad(ultima: u64, ahora: u64) -> u64 {
     ahora.saturating_sub(ultima) / DIA
 }
@@ -212,22 +206,16 @@ mod tests {
         Ficha::de(&Documento::Formato2(t, Cuerpo::leer(cuerpo)))
     }
 
-    /// TASK-081 (D2a): la actividad es la mas reciente de la ultima fila y
-    /// del commit de la rama; sin ninguna, no se sabe. Los dias son
-    /// completos y una actividad del futuro cuenta como de hoy.
+    /// TASK-081 (D2a): la ultima fila cuenta tambien si es `pausa`. Los dias
+    /// son completos; la regla de la actividad (y su margen para un reloj
+    /// adelantado) vive en `ciclo::liberar::actividad_conocida` (TASK-125).
     #[test]
-    fn actividad_y_dias_sin_actividad() {
+    fn dias_sin_actividad_completos() {
         let f = ficha(
             "## Transiciones\n| fecha | fase |\n|---|---|\n| 2026-11-17T10:20:30Z | pausa |\n",
         );
         let fila = 1_794_910_830;
         assert_eq!(f.ultima_transicion, Some(fila));
-        assert_eq!(actividad(&f, None), Some(fila));
-        assert_eq!(actividad(&f, Some(fila - 5)), Some(fila));
-        assert_eq!(actividad(&f, Some(fila + 5)), Some(fila + 5));
-        let sin = ficha("");
-        assert_eq!(actividad(&sin, None), None);
-        assert_eq!(actividad(&sin, Some(3)), Some(3));
         assert_eq!(dias_sin_actividad(fila, fila + 7 * DIA - 1), 6);
         assert_eq!(dias_sin_actividad(fila, fila + 7 * DIA), 7);
         assert_eq!(dias_sin_actividad(fila + 100, fila), 0);
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-125/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-125/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-125/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````
