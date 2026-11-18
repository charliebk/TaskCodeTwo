# Peticion de revision — TASK-081 (ronda 1)

- Tarea: TASK-081 — F3-10 Liberar y reasignar tareas abandonadas
- Rama revisada: feature/task-081-f3-10-liberar-y-reasignar-tareas-abandon
- Rama base: develop
- Commit revisado (HEAD): 6f41ef2e6595a749fa6ea41439152efc1dfb5182
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-081 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-081/tarea.md
- tareas/03-en-revision/TASK-081/planificacion/
- crates/tc2/src/ciclo/liberar.rs
- crates/tc2/src/ciclo/mod.rs
- crates/tc2/src/ciclo/operacion.rs
- crates/tc2/src/config/claves.rs
- crates/tc2/src/config/mod.rs
- crates/tc2/src/git/mod.rs
- crates/tc2/src/git/rama.rs
- crates/tc2/src/git/worktree.rs
- crates/tc2/src/i18n/catalogo/ciclo.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/tablero.rs
- crates/tc2/src/inventario.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/plataforma/personas.rs
- crates/tc2/src/situacion.rs
- crates/tc2/src/tablero.rs
- crates/tc2/src/tarea/cuerpo.rs
- crates/tc2/src/toma/mod.rs
- crates/tc2/tests/liberar.rs
- crates/tc2/tests/tablero.rs

## Commits a revisar (git log develop..HEAD)

````
6f41ef2 docs(TASK-081): autorrevision con rust-wasm-reviewer y code-quality-reviewer
bfc11c6 docs(TASK-081): adendas de F1-2, F1-3, F1-9 y F1-11 por liberar y reasignar
e809973 feat(TASK-081): tareas sin actividad y aviso ya-no-tuyas en status y board
3da3c0a feat(TASK-081): tc2 liberar y tc2 reasignar
c7c8b81 refactor(TASK-081): actividad de la tarea, fechas de ramas y worktree publicado
6a864b5 chore(TASK-081): tarea en curso
````

## Diff de tu dominio (20 fichero(s) de 25; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/ciclo/liberar.rs b/crates/tc2/src/ciclo/liberar.rs
new file mode 100644
index 0000000..c0e8a67
--- /dev/null
+++ b/crates/tc2/src/ciclo/liberar.rs
@@ -0,0 +1,660 @@
+//! Liberar y reasignar una tarea como operaciones de la rama de estado
+//! (F3-10, RR-12; ADR F1-2 D2b: exigen red). Como [`super::Descartar`],
+//! pasan por el punto unico de escritura, deciden solo con el estado que
+//! reciben y reescriben unicamente el `tarea.md` de su tarea: frontmatter,
+//! una fila nueva en `## Transiciones` y el resto byte a byte.
+//!
+//! - Quien (D3 de TASK-081): el dueno siempre (tambien su agente); una
+//!   tarea sin dueno, cualquiera. La de otra persona de la lista, solo con
+//!   [`Dejar::umbral`] dias o mas sin actividad (si no, `rechazada
+//!   motivo=activa`) y lanzada por una persona en un terminal (el agente no
+//!   quita a nadie una tarea: `requiere-persona`). Un `asignado_a` heredado
+//!   de la 0.16 no espera los dias, pero tambien lo decide una persona.
+//! - Actividad (D2a): la mas reciente de la ultima fila de `## Transiciones`
+//!   (la pausa no exime) y del ultimo commit de `origin/<rama>`, que llega
+//!   de fuera ya traido (la rama no esta en la rama de estado). Sin ninguna
+//!   de las dos no se sabe y no frena.
+//! - [`Liberar`] (D6a): `en-curso` pasa a `planificada`; `en-diseno` y
+//!   `planificada` se quedan; `asignado_a: null`; `rama` y `plan_aprobado`,
+//!   igual. `en-revision` se rechaza (tiene PR: se reasigna). Una ya libre,
+//!   `sin-cambios`.
+//! - [`Reasignar`]: solo `asignado_a`, a la persona de la lista que llega
+//!   ya validada; conserva el estado. Ya suya, `sin-cambios`.
+
+use std::cell::RefCell;
+
+use super::operacion::{ABIERTAS, Lanzada};
+use crate::estado::{Cambio, Estado, Fallo, Operacion, por_estado, tarea_de};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::ciclo as txt;
+use crate::identidad::Quien;
+use crate::situacion::dias_sin_actividad;
+use crate::tarea::{Asignado, Cuerpo, Tarea};
+use crate::toma::operacion::EN_CURSO;
+
+/// Nombre de `liberar`, de su operacion y de su fila de transicion.
+pub const LIBERAR: &str = "liberar";
+
+/// Nombre de `reasignar`, de su operacion y de su fila de transicion.
+pub const REASIGNAR: &str = "reasignar";
+
+/// Dias sin actividad por defecto (clave `dias_sin_actividad`, D1a).
+pub const DIAS_POR_DEFECTO: u64 = 7;
+
+/// Estado al que vuelve una tarea en curso liberada.
+const PLANIFICADA: &str = "planificada";
+
+/// Estados desde los que se libera (D6a).
+pub const LIBERABLES: [&str; 3] = [PLANIFICADA, "en-diseno", EN_CURSO];
+
+/// Lo comun de las dos: la tarea, quien la lanza y cuando, la actividad de
+/// su rama publicada y el umbral de dias; y lo que vio el ultimo intento.
+#[derive(Debug)]
+pub struct Dejar {
+    base: Lanzada,
+    actividad_rama: Option<u64>,
+    dias: u64,
+    anterior: RefCell<Option<String>>,
+    activa: RefCell<Option<u64>>,
+}
+
+impl Dejar {
+    fn nueva(
+        id: &str,
+        asignado: Asignado,
+        decidido_por: &'static str,
+        ahora: u64,
+        actividad_rama: Option<u64>,
+        dias: u64,
+    ) -> Self {
+        Self {
+            base: Lanzada::nueva(id, asignado, decidido_por, ahora),
+            actividad_rama,
+            dias,
+            anterior: RefCell::new(None),
+            activa: RefCell::new(None),
+        }
+    }
+
+    /// Login de quien la llevaba si el ultimo intento la rechazo por
+    /// `activa` o `requiere-persona`.
+    pub fn dueno(&self) -> Option<String> {
+        self.base.dueno.borrow().clone()
+    }
+
+    /// Quien la llevaba (login o texto heredado) si el ultimo intento la
+    /// cambio.
+    pub fn anterior(&self) -> Option<String> {
+        self.anterior.borrow().clone()
+    }
+
+    /// Dias sin actividad de la tarea si el ultimo intento la rechazo por
+    /// `activa`.
+    pub fn activa(&self) -> Option<u64> {
+        *self.activa.borrow()
+    }
+
+    /// Dias sin actividad a partir de los cuales otra persona puede dejarla.
+    pub fn umbral(&self) -> u64 {
+        self.dias
+    }
+
+    /// Si quien la lanza puede dejar la tarea `t` (D3).
+    fn permiso(&self, t: &Tarea, cuerpo: &Cuerpo, orden: &str) -> Result<(), Fallo> {
+        *self.base.dueno.borrow_mut() = None;
+        *self.activa.borrow_mut() = None;
+        let id = self.base.id.as_str();
+        let Some(a) = t.asignado_a.as_ref() else {
+            return Ok(());
+        };
+        if a.misma_persona(&self.base.asignado) {
+            return Ok(());
+        }
+        *self.base.dueno.borrow_mut() = a.login().map(str::to_owned);
+        if let Some(login) = a.login() {
+            let actividad = cuerpo.ultima_fecha().max(self.actividad_rama);
+            if let Some(k) = actividad.map(|x| dias_sin_actividad(x, self.base.ahora))
+                && k < self.dias
+            {
+                *self.activa.borrow_mut() = Some(k);
+                return Err(Fallo::rechazada(
+                    "activa",
+                    Mensaje::nuevo(&txt::ACTIVA)
+                        .con("id", id)
+                        .con("login", login)
+                        .con("dias", k.to_string())
+                        .con("umbral", self.dias.to_string())
+                        .con("orden", orden),
+                ));
+            }
+        }
+        if self.base.decidido_por != "persona" {
+            let quien = a
+                .login()
+                .map(|l| format!("@{l}"))
+                .or_else(|| a.heredado().map(str::to_owned))
+                .unwrap_or_default();
+            return Err(Fallo::rechazada(
+                "requiere-persona",
+                Mensaje::nuevo(&txt::REQUIERE_PERSONA)
+                    .con("id", id)
+                    .con("quien", quien)
+                    .con("orden", orden),
+            ));
+        }
+        Ok(())
+    }
+
+    /// Anota quien la llevaba antes de cambiarla.
+    fn anotar_anterior(&self, t: &Tarea) {
+        *self.anterior.borrow_mut() = t
+            .asignado_a
+            .as_ref()
+            .and_then(|a| a.login().or_else(|| a.heredado()))
+            .map(str::to_owned);
+    }
+
+    /// `|<actividad o ->|<dias>|<ahora>` de la anotacion.
+    fn cola(&self) -> String {
+        format!(
+            "{}|{}|{}",
+            self.actividad_rama
+                .map_or_else(|| "-".to_owned(), |x| x.to_string()),
+            self.dias,
+            self.base.ahora
+        )
+    }
+}
+
+/// Liberar la tarea `id`: queda sin asignar y lista para empezar.
+#[derive(Debug)]
+pub struct Liberar {
+    d: Dejar,
+}
+
+impl Liberar {
+    /// La liberacion de `id` por `quien` en `ahora` (segundos desde 1970,
+    /// UTC), con la actividad de su rama publicada (`actividad_rama`) y el
+    /// umbral de `dias`.
+    pub fn nueva(
+        id: &str,
+        quien: &Quien,
+        ahora: u64,
+        actividad_rama: Option<u64>,
+        dias: u64,
+    ) -> Self {
+        Self {
+            d: Dejar::nueva(
+                id,
+                quien.asignado(),
+                quien.decidido_por(),
+                ahora,
+                actividad_rama,
+                dias,
+            ),
+        }
+    }
+
+    /// Lo comun que vio el ultimo intento.
+    pub fn dejar(&self) -> &Dejar {
+        &self.d
+    }
+}
+
+impl Operacion for Liberar {
+    fn nombre(&self) -> &str {
+        LIBERAR
+    }
+
+    fn anotacion(&self) -> String {
+        format!("{}|{}", self.d.base.anotacion(LIBERAR), self.d.cola())
+    }
+
+    fn exige_red(&self) -> bool {
+        // Cambia quien lleva una tarea compartida (D2b de F1-2).
+        true
+    }
+
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        *self.d.anterior.borrow_mut() = None;
+        let (mut t, cuerpo) = tarea_de(estado, &self.d.base.id)?;
+        if !LIBERABLES.contains(&t.estado.as_str()) {
+            return Err(por_estado(&self.d.base.id, &t.estado, LIBERAR, &LIBERABLES));
+        }
+        if t.asignado_a.is_none() && t.estado != EN_CURSO {
+            // Ya libre: nada que escribir.
+            return Ok(Vec::new());
+        }
+        self.d.permiso(&t, &cuerpo, LIBERAR)?;
+        self.d.anotar_anterior(&t);
+        if t.estado == EN_CURSO {
+            PLANIFICADA.clone_into(&mut t.estado);
+        }
+        t.asignado_a = None;
+        Ok(self.d.base.escribir(t, cuerpo, LIBERAR))
+    }
+}
+
+/// Reasignar la tarea `id` a otra persona de la lista.
+#[derive(Debug)]
+pub struct Reasignar {
+    d: Dejar,
+    a: Asignado,
+}
+
+impl Reasignar {
+    /// La reasignacion de `id` a `a` (una persona de la lista, ya validada)
+    /// por `quien` en `ahora`, con la actividad de su rama publicada y el
+    /// umbral de `dias`.
+    pub fn nueva(
+        id: &str,
+        quien: &Quien,
+        a: Asignado,
+        ahora: u64,
+        actividad_rama: Option<u64>,
+        dias: u64,
+    ) -> Self {
+        Self {
+            d: Dejar::nueva(
+                id,
+                quien.asignado(),
+                quien.decidido_por(),
+                ahora,
+                actividad_rama,
+                dias,
+            ),
+            a,
+        }
+    }
+
+    /// Lo comun que vio el ultimo intento.
+    pub fn dejar(&self) -> &Dejar {
+        &self.d
+    }
+
+    /// La persona a la que se reasigna.
+    pub fn destino(&self) -> &Asignado {
+        &self.a
+    }
+}
+
+impl Operacion for Reasignar {
+    fn nombre(&self) -> &str {
+        REASIGNAR
+    }
+
+    fn anotacion(&self) -> String {
+        format!(
+            "{}|{}|{}|{}",
+            self.d.base.anotacion(REASIGNAR),
+            self.a.id().unwrap_or_default(),
+            self.a.login().unwrap_or_default(),
+            self.d.cola()
+        )
+    }
+
+    fn exige_red(&self) -> bool {
+        // Cambia quien lleva una tarea compartida (D2b de F1-2).
+        true
+    }
+
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        *self.d.anterior.borrow_mut() = None;
+        let (mut t, cuerpo) = tarea_de(estado, &self.d.base.id)?;
+        if !ABIERTAS.contains(&t.estado.as_str()) {
+            return Err(por_estado(&self.d.base.id, &t.estado, REASIGNAR, &ABIERTAS));
+        }
+        if t.asignado_a
+            .as_ref()
+            .is_some_and(|x| x.misma_persona(&self.a))
+        {
+            // Ya la lleva: nada que escribir.
+            return Ok(Vec::new());
+        }
+        self.d.permiso(&t, &cuerpo, REASIGNAR)?;
+        self.d.anotar_anterior(&t);
+        t.asignado_a = Some(self.a.clone());
+        Ok(self.d.base.escribir(t, cuerpo, REASIGNAR))
+    }
+}
+
+/// Reconstruye una operacion de este modulo desde su anotacion (para el
+/// catalogo): `liberar|TASK-NNN|<id>|<login>|<actividad o ->|<dias>|<ahora>`
+/// o `reasignar|TASK-NNN|<id>|<login>|<id destino>|<login destino>|<actividad
+/// o ->|<dias>|<ahora>`. Exigen red y no quedan pendientes; si se
+/// re-ejecutaran, sin persona delante: `decidido_por: agente`.
+pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
+    let partes: Vec<&str> = a.split('|').collect();
+    let (nombre, id) = (*partes.first()?, *partes.get(1)?);
+    if !crate::git::es_id(id) {
+        return None;
+    }
+    let asignado = Asignado::persona(partes.get(2)?.parse().ok()?, partes.get(3)?)?;
+    let cola = |p: &[&str]| -> Option<(Option<u64>, u64, u64)> {
+        let [actividad, dias, ahora] = p else {
+            return None;
+        };
+        let actividad = match *actividad {
+            "-" => None,
+            x => Some(x.parse().ok()?),
+        };
+        Some((actividad, dias.parse().ok()?, ahora.parse().ok()?))
+    };
+    match nombre {
+        LIBERAR => {
+            let (actividad, dias, ahora) = cola(partes.get(4..)?)?;
+            Some(Box::new(Liberar {
+                d: Dejar::nueva(id, asignado, "agente", ahora, actividad, dias),
+            }))
+        }
+        REASIGNAR => {
+            let destino = Asignado::persona(partes.get(4)?.parse().ok()?, partes.get(5)?)?;
+            let (actividad, dias, ahora) = cola(partes.get(6..)?)?;
+            Some(Box::new(Reasignar {
+                d: Dejar::nueva(id, asignado, "agente", ahora, actividad, dias),
+                a: destino,
+            }))
+        }
+        _ => None,
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::grafo::DESCARTADA;
+    use crate::identidad::Cuenta;
+    use crate::plataforma::Proveedor;
+    use crate::roadmap::tests::EnMemoria;
+    use std::collections::BTreeMap;
+
+    /// 2026-11-17T10:20:30Z.
+    const AHORA: u64 = 1_794_910_830;
+    const DIA: u64 = 86_400;
+
+    fn cuenta(id: u64, login: &str) -> Cuenta {
+        Cuenta {
+            proveedor: Proveedor::GitHub,
+            id,
+            login: login.to_owned(),
+        }
+    }
+
+    /// Ana desde su agente (sin terminal).
+    fn ana() -> Quien {
+        Quien::Agente(cuenta(7, "ana"))
+    }
+
+    /// Beto en un terminal.
+    fn beto() -> Quien {
+        Quien::Persona(cuenta(9, "beto"))
+    }
+
+    fn persona(id: u64, login: &str) -> Asignado {
+        Asignado::persona(id, login).unwrap_or_else(|| unreachable!())
+    }
+
+    /// Un `tarea.md` con una fila de transicion de hace `hace` segundos.
+    fn md(id: &str, estado: &str, asignado: Option<Asignado>, hace: u64) -> String {
+        let mut t = Tarea::nueva(id, &format!("Tarea {id}"), "2026-11-01");
+        t.estado = estado.to_owned();
+        if estado == DESCARTADA {
+            t.motivo = Some("no".to_owned());
+        }
+        t.plan_aprobado = true;
+        t.asignado_a = asignado;
+        t.rama = Some(crate::alta::rama_de(id, &t.tipo, &t.titulo));
+        let fila = format!(
+            "| {} | start | manual | persona | 0.1 |\n",
+            crate::toma::instante(AHORA - hace)
+        );
+        t.escribir(&Cuerpo::plantilla()) + &fila
+    }
+
+    fn estado(tareas: &[(&str, String)]) -> EnMemoria {
+        EnMemoria(
+            tareas
+                .iter()
+                .map(|(id, t)| (format!("{id}/tarea.md"), t.clone()))
+                .collect::<BTreeMap<_, _>>(),
+        )
+    }
+
+    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
+        match r {
+            Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
+            _ => None,
+        }
+    }
+
+    fn texto(r: &Result<Vec<Cambio>, Fallo>) -> String {
+        match r.as_deref() {
+            Ok([c]) => c.contenido.clone(),
+            otro => format!("{otro:?}"),
+        }
+    }
+
+    fn fila(fase: &str, por: &str) -> String {
+        format!(
+            "| 2026-11-17T10:20:30Z | {fase} | manual | {por} | {} |\n",
+            crate::VERSION
+        )
+    }
+
+    /// C1 (unitario): el dueno (su agente) libera una en curso: `planificada`,
+    /// `asignado_a: null`, `actualizado` y la fila; `rama` y `plan_aprobado`
+    /// y el resto, igual. En diseno se queda en diseno.
+    #[test]
+    fn liberar_escribe_solo_lo_suyo() {
+        let yo = ana().asignado();
+        for (de, a) in [("en-curso", "planificada"), ("en-diseno", "en-diseno")] {
+            let antes = md("TASK-001", de, Some(yo.clone()), 0);
+            let s = estado(&[("TASK-001", antes.clone())]);
+            let op = Liberar::nueva("TASK-001", &ana(), AHORA, None, 7);
+            let r = op.aplicar(&s);
+            let esperado = antes
+                .replace(&format!("estado: \"{de}\""), &format!("estado: \"{a}\""))
+                .replace("asignado_a: {id: 7, login: \"ana\"}", "asignado_a: null")
+                .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
+                + &fila("liberar", "agente");
+            assert_eq!(texto(&r), esperado, "{de}");
+            assert!(esperado.contains("\nplan_aprobado: true\n"));
+            assert!(esperado.contains("\nrama: \"feature/task-001-tarea-task-001\"\n"));
+            assert_eq!(op.dejar().anterior().as_deref(), Some("ana"));
+            assert!(op.exige_red());
+        }
+    }
+
+    /// C3 (unitario): reasignar cambia solo `asignado_a` (al objeto de la
+    /// lista) y anade la fila; el estado se conserva. Ya suya, nada.
+    #[test]
+    fn reasignar_escribe_solo_el_asignado() {
+        let yo = ana().asignado();
+        for e in ABIERTAS {
+            let antes = md("TASK-001", e, Some(yo.clone()), 0);
+            let s = estado(&[("TASK-001", antes.clone())]);
+            let op = Reasignar::nueva("TASK-001", &ana(), persona(9, "beto"), AHORA, None, 7);
+            let esperado = antes
+                .replace(
+                    "asignado_a: {id: 7, login: \"ana\"}",
+                    "asignado_a: {id: 9, login: \"beto\"}",
+                )
+                .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
+                + &fila("reasignar", "agente");
+            assert_eq!(texto(&op.aplicar(&s)), esperado, "{e}");
+            assert_eq!(op.dejar().anterior().as_deref(), Some("ana"));
+            // A quien ya la lleva: sin cambios.
+            let op = Reasignar::nueva("TASK-001", &ana(), persona(7, "ana"), AHORA, None, 7);
+            assert_eq!(op.aplicar(&s), Ok(Vec::new()), "{e}");
+        }
+    }
+
+    /// C5 (unitario): la de otra persona con actividad de hace menos de N
+    /// dias (por la fila o por el commit de la rama) se rechaza `activa`
+    /// con los dias; con N o mas, procede (una persona en un terminal).
+    #[test]
+    fn otra_persona_activa_o_inactiva() {
+        let de_ana = Some(persona(7, "ana"));
+        let reciente = estado(&[("TASK-001", md("TASK-001", "en-curso", de_ana.clone(), DIA))]);
+        let vieja = estado(&[("TASK-001", md("TASK-001", "en-curso", de_ana, 7 * DIA))]);
+        // (motivo del rechazo o «ok», dias de `activa`, dueno) de liberar o
+        // reasignar con la actividad de la rama `rama`.
+        let probar = |s: &EnMemoria, reasignar: bool, rama: Option<u64>| {
+            let (r, d) = if reasignar {
+                let op = Reasignar::nueva("TASK-001", &beto(), persona(9, "beto"), AHORA, rama, 7);
+                (op.aplicar(s), (op.dejar().activa(), op.dejar().dueno()))
+            } else {
+                let op = Liberar::nueva("TASK-001", &beto(), AHORA, rama, 7);
+                (op.aplicar(s), (op.dejar().activa(), op.dejar().dueno()))
+            };
+            let m = match &r {
+                Ok(c) if c.len() == 1 => "ok".to_owned(),
+                otro => motivo(otro).unwrap_or("?").to_owned(),
+            };
+            (m, d.0, d.1)
+        };
+        for reasignar in [false, true] {
+            let (m, dias, dueno) = probar(&reciente, reasignar, None);
+            assert_eq!(
+                (m.as_str(), dias, dueno.as_deref()),
+                ("activa", Some(1), Some("ana"))
+            );
+            let (m, dias, _) = probar(&vieja, reasignar, None);
+            assert_eq!((m.as_str(), dias), ("ok", None));
+            // Un commit reciente en la rama la mantiene activa aunque la
+            // fila sea vieja.
+            let (m, dias, _) = probar(&vieja, reasignar, Some(AHORA - 2 * DIA));
+            assert_eq!((m.as_str(), dias), ("activa", Some(2)));
+            // Una actividad del futuro cuenta como de hoy.
+            let (m, dias, _) = probar(&vieja, reasignar, Some(AHORA + 30 * DIA));
+            assert_eq!((m.as_str(), dias), ("activa", Some(0)));
+        }
+    }
+
+    /// C6 (unitario): el agente no quita a nadie una tarea aunque haya
+    /// pasado N (`requiere-persona`); sobre la propia procede; una sin dueno,
+    /// cualquiera.
+    #[test]
+    fn el_agente_no_quita_tareas() {
+        let vieja = |a: Option<Asignado>| md("TASK-001", "en-curso", a, 30 * DIA);
+        let s = estado(&[("TASK-001", vieja(Some(persona(9, "beto"))))]);
+        let op = Liberar::nueva("TASK-001", &ana(), AHORA, None, 7);
+        assert_eq!(motivo(&op.aplicar(&s)), Some("requiere-persona"));
+        assert_eq!(op.dejar().dueno().as_deref(), Some("beto"));
+        let op = Reasignar::nueva("TASK-001", &ana(), persona(7, "ana"), AHORA, None, 7);
+        assert_eq!(motivo(&op.aplicar(&s)), Some("requiere-persona"));
+        // Propia (de ana) o sin dueno: el agente si.
+        for a in [Some(persona(7, "ana")), None] {
+            let s = estado(&[("TASK-001", vieja(a))]);
+            let r = Liberar::nueva("TASK-001", &ana(), AHORA, None, 7).aplicar(&s);
+            assert!(r.is_ok_and(|c| c.len() == 1));
+        }
+    }
+
+    /// C11 (unitario): liberar una terminada, descartada o en revision se
+    /// rechaza por estado; una ya libre, sin cambios. Reasignar una
+    /// terminada o descartada, tambien por estado.
+    #[test]
+    fn rechazos_por_estado_y_ya_libre() {
+        let yo = Some(ana().asignado());
+        let s = estado(&[
+            ("TASK-001", md("TASK-001", "terminada", yo.clone(), 0)),
+            ("TASK-002", md("TASK-002", DESCARTADA, yo.clone(), 0)),
+            ("TASK-003", md("TASK-003", "en-revision", yo, 0)),
+            ("TASK-004", md("TASK-004", "planificada", None, 0)),
+            ("TASK-005", md("TASK-005", "en-diseno", None, 0)),
+        ]);
+        for (id, esperado) in [
+            ("TASK-001", "terminada"),
+            ("TASK-002", "descartada"),
+            ("TASK-003", "estado-no-valido"),
+            ("TASK-050", "no-existe"),
+        ] {
+            let r = Liberar::nueva(id, &ana(), AHORA, None, 7).aplicar(&s);
+            assert_eq!(motivo(&r), Some(esperado), "{id}");
+        }
+        for id in ["TASK-004", "TASK-005"] {
+            let op = Liberar::nueva(id, &ana(), AHORA, None, 7);
+            assert_eq!(op.aplicar(&s), Ok(Vec::new()), "{id}");
+            assert_eq!(op.dejar().anterior(), None);
+        }
+        for (id, esperado) in [("TASK-001", "terminada"), ("TASK-002", "descartada")] {
+            let r = Reasignar::nueva(id, &ana(), persona(9, "beto"), AHORA, None, 7).aplicar(&s);
+            assert_eq!(motivo(&r), Some(esperado), "{id}");
+        }
+        // En revision si se reasigna (D6a).
+        let r =
+            Reasignar::nueva("TASK-003", &ana(), persona(9, "beto"), AHORA, None, 7).aplicar(&s);
+        assert!(r.is_ok_and(|c| c.len() == 1));
+    }
+
+    /// C12 (unitario): un `asignado_a` heredado de la 0.16 se reasigna sin
+    /// esperar N dias (lo decide una persona) y queda como objeto; el
+    /// agente, `requiere-persona`.
+    #[test]
+    fn heredado_se_reasigna_a_un_objeto() {
+        let heredado = md("TASK-001", "en-curso", None, 0)
+            .replace("asignado_a: null", "asignado_a: \"ana@x.com\"");
+        let s = estado(&[("TASK-001", heredado.clone())]);
+        let op = Reasignar::nueva("TASK-001", &beto(), persona(9, "beto"), AHORA, None, 7);
+        let esperado = heredado
+            .replace(
+                "asignado_a: \"ana@x.com\"",
+                "asignado_a: {id: 9, login: \"beto\"}",
+            )
+            .replace("actualizado: \"2026-11-01\"", "actualizado: \"2026-11-17\"")
+            + &fila("reasignar", "persona");
+        assert_eq!(texto(&op.aplicar(&s)), esperado);
+        assert_eq!(op.dejar().anterior().as_deref(), Some("ana@x.com"));
+        let op = Reasignar::nueva("TASK-001", &ana(), persona(9, "beto"), AHORA, None, 7);
+        assert_eq!(motivo(&op.aplicar(&s)), Some("requiere-persona"));
+        assert_eq!(op.dejar().dueno(), None);
+    }
+
+    /// Las anotaciones se reconstruyen igual y las malas no.
+    #[test]
+    fn anotacion_ida_y_vuelta() {
+        let ops: [Box<dyn Operacion>; 3] = [
+            Box::new(Liberar::nueva("TASK-001", &ana(), AHORA, None, 7)),
+            Box::new(Liberar::nueva("TASK-001", &ana(), AHORA, Some(5), 30)),
+            Box::new(Reasignar::nueva(
+                "TASK-001",
+                &ana(),
+                persona(9, "beto"),
+                AHORA,
+                Some(5),
+                7,
+            )),
+        ];
+        let anotaciones: Vec<String> = ops.iter().map(|o| o.anotacion()).collect();
+        assert_eq!(
+            anotaciones,
+            [
+                "liberar|TASK-001|7|ana|-|7|1794910830",
+                "liberar|TASK-001|7|ana|5|30|1794910830",
+                "reasignar|TASK-001|7|ana|9|beto|5|7|1794910830",
+            ]
+        );
+        for (o, a) in ops.iter().zip(&anotaciones) {
+            let r = desde_anotacion(a);
+            assert_eq!(
+                r.as_ref()
+                    .map(|x| (x.nombre().to_owned(), x.anotacion(), x.exige_red())),
+                Some((o.nombre().to_owned(), a.clone(), true))
+            );
+        }
+        for mala in [
+            "liberar|TASK-001|7|ana|-|7",
+            "liberar|TASK-001|7|ana|x|7|1",
+            "liberar|TASK-001|7|ana|-|7|1|2",
+            "liberar|TASK-1|7|ana|-|7|1",
+            "liberar|TASK-001|0|ana|-|7|1",
+            "reasignar|TASK-001|7|ana|0|beto|-|7|1",
+            "reasignar|TASK-001|7|ana|9||-|7|1",
+            "reasignar|TASK-001|7|ana|9|beto|-|7",
+            "descartar|TASK-001|7|ana|m",
+        ] {
+            assert!(desde_anotacion(mala).is_none(), "{mala}");
+        }
+    }
+}
diff --git a/crates/tc2/src/ciclo/mod.rs b/crates/tc2/src/ciclo/mod.rs
index 094fcca..a4fbf0e 100644
--- a/crates/tc2/src/ciclo/mod.rs
+++ b/crates/tc2/src/ciclo/mod.rs
@@ -17,22 +17,42 @@
 //!   lanza y la fila por la transaccion; sin red quedan pendientes y la
 //!   siguiente transaccion con red las publica (D4, ADR F1-2 punto 5).
 //!
+//! - `liberar [--json] TASK-NNN` y `reasignar [--json] TASK-NNN <login>`
+//!   (F3-10, RR-12): como `descartar`, `fetch` completo (exigen red), quien
+//!   la lanza y la operacion ([`liberar`]) por la transaccion; antes, el
+//!   destino de `reasignar` en la lista de personas de la base
+//!   (`fuera-de-lista`), la actividad de la rama publicada y el umbral de
+//!   dias (clave `dias_sin_actividad`), la operacion en seco (sus rechazos
+//!   salen sin esperar al push) y el worktree de la tarea en este clon: con
+//!   trabajo sin publicar, `rechazada motivo=sin-publicar` sin escribir
+//!   nada (D4). Publicada, el worktree se quita como en `descartar`. La rama
+//!   no se toca nunca: la siguiente persona la retoma con `start`.
+//!
 //! Lanzadas desde un worktree de tarea escriben el estado en el principal
 //! (lo localiza la transaccion). El entorno es el de `start`
 //! ([`crate::toma::Entorno`]).
 
+pub mod liberar;
 pub mod operacion;
 
 use std::path::Path;
 
 use crate::contrato::{Informe, Resultado, Valor};
-use crate::estado::{self, Operacion};
+use crate::estado::{self, Fallo, Operacion};
 use crate::git::{self, Git};
 use crate::i18n::catalogo::ciclo as txt;
 use crate::i18n::{Mensaje, Texto};
+use crate::tarea::Asignado;
 use crate::toma::Entorno;
 
-pub use operacion::{DESCARTAR, Descartar, MOTIVO_MAX, Pausar, desde_anotacion, motivo_valido};
+pub use liberar::{DIAS_POR_DEFECTO, Dejar, LIBERAR, Liberar, REASIGNAR, Reasignar};
+pub use operacion::{DESCARTAR, Descartar, MOTIVO_MAX, Pausar, motivo_valido};
+
+/// Reconstruye una operacion de este modulo desde su anotacion (para el
+/// catalogo): las de [`operacion`] y las de [`liberar`].
+pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
+    operacion::desde_anotacion(a).or_else(|| liberar::desde_anotacion(a))
+}
 
 /// Nombre de la orden `pausa` (y de su fila de transicion).
 pub const PAUSA: &str = crate::tarea::PAUSA;
@@ -81,6 +101,215 @@ pub fn orden_pausa(args: &[String], reanudar: bool, e: &Entorno<'_>) -> Informe
     }
 }
 
+/// `liberar TASK-NNN` (sin `--json`, que quita `lib.rs`).
+pub fn orden_liberar(args: &[String], e: &Entorno<'_>) -> Informe {
+    let u = |motivo: &str| uso(LIBERAR, motivo, &txt::USO_LIBERAR);
+    match args {
+        [id] if git::es_id(id) => dejar_en(e, id, None),
+        [_] => u("id-invalido"),
+        [] => u("sin-id"),
+        _ => u("argumentos-de-mas"),
+    }
+}
+
+/// `reasignar TASK-NNN <login>` (sin `--json`, que quita `lib.rs`). Sin
+/// login, `uso motivo=sin-login`. Nada se lee ni se escribe antes.
+pub fn orden_reasignar(args: &[String], e: &Entorno<'_>) -> Informe {
+    let u = |motivo: &str| uso(REASIGNAR, motivo, &txt::USO_REASIGNAR);
+    match args {
+        [] => u("sin-id"),
+        [id, ..] if !git::es_id(id) => u("id-invalido"),
+        [_] => u("sin-login"),
+        [id, login] => dejar_en(e, id, Some(login)),
+        _ => u("argumentos-de-mas"),
+    }
+}
+
+/// `liberar` (sin `destino`) o `reasignar` a `destino` de la tarea `id` en
+/// el repo de `e`.
+pub fn dejar_en(e: &Entorno<'_>, id: &str, destino: Option<&str>) -> Informe {
+    let nombre = if destino.is_some() {
+        REASIGNAR
+    } else {
+        LIBERAR
+    };
+    let i = match paso_dejar(e, id, destino) {
+        Ok(i) | Err(i) => i,
+    };
+    de_la_orden(i, nombre, id)
+}
+
+/// La persona de la lista de la base con ese login (sin distinguir
+/// mayusculas); si no esta, `rechazada motivo=fuera-de-lista`.
+fn destino_de(e: &Entorno<'_>, login: &str) -> Result<Asignado, Informe> {
+    let personas = crate::identidad::personas_de_base(e.git, e.dir)?;
+    personas
+        .por_login(login)
+        .and_then(|p| Asignado::persona(p.id, &p.login))
+        .ok_or_else(|| {
+            Informe::nuevo(REASIGNAR, Resultado::Rechazada)
+                .con("motivo", "fuera-de-lista")
+                .diagnostico(
+                    Mensaje::nuevo(&txt::FUERA_DE_LISTA)
+                        .con("login", crate::plataforma::errores::sanear(login))
+                        .con("ruta", crate::plataforma::personas::RUTA)
+                        .con("base", crate::config::BASE),
+                )
+        })
+}
+
+/// Umbral de dias sin actividad (clave `dias_sin_actividad` de la
+/// configuracion de la base); sin configuracion o si no vale, el de por
+/// defecto (D1a).
+pub fn dias_sin_actividad(g: &Git, dir: &Path) -> u64 {
+    crate::config::de_base(g, dir).map_or(DIAS_POR_DEFECTO, |c| u64::from(c.dias_sin_actividad))
+}
+
+/// Las dos operaciones, para tratarlas igual.
+enum Dejada {
+    Liberar(Liberar),
+    Reasignar(Reasignar),
+}
+
+impl Dejada {
+    fn op(&self) -> &dyn Operacion {
+        match self {
+            Self::Liberar(o) => o,
+            Self::Reasignar(o) => o,
+        }
+    }
+
+    fn dejar(&self) -> &Dejar {
+        match self {
+            Self::Liberar(o) => o.dejar(),
+            Self::Reasignar(o) => o.dejar(),
+        }
+    }
+}
+
+fn paso_dejar(e: &Entorno<'_>, id: &str, destino: Option<&str>) -> Result<Informe, Informe> {
+    let g = e.git;
+    let nombre = if destino.is_some() {
+        REASIGNAR
+    } else {
+        LIBERAR
+    };
+    let principal = git::localizar_principal(g, e.dir)?;
+    // Exige red: lo de ahora del remoto, tambien las ramas (la actividad de
+    // la de la tarea y si el worktree esta publicado).
+    if git::estado::hay_remoto(g, &principal, nombre)? {
+        let f = git::fetch(g, &principal);
+        if !f.resultado.es_exito() {
+            return Err(f);
+        }
+    }
+    let quien = (e.quien)()?;
+    let a = destino.map(|l| destino_de(e, l)).transpose()?;
+    let vista = estado::leer(g, e.dir)?;
+    let actividad_rama = match estado::tarea_de(&vista, id).ok().and_then(|(t, _)| t.rama) {
+        Some(rama) => git::fechas_de_ramas(g, &principal, nombre)?
+            .get(&rama)
+            .copied(),
+        None => None,
+    };
+    let dias = dias_sin_actividad(g, e.dir);
+    // Si se la queda quien la lanza, su worktree sigue siendo suyo.
+    let para_mi = a
+        .as_ref()
+        .is_some_and(|x| x.misma_persona(&quien.asignado()));
+    let op = match a {
+        Some(a) => Dejada::Reasignar(Reasignar::nueva(
+            id,
+            &quien,
+            a,
+            e.ahora,
+            actividad_rama,
+            dias,
+        )),
+        None => Dejada::Liberar(Liberar::nueva(id, &quien, e.ahora, actividad_rama, dias)),
+    };
+    // En seco sobre lo leido: los rechazos, sin esperar al push.
+    let cambia = match op.op().aplicar(&vista) {
+        Ok(c) => !c.is_empty(),
+        Err(Fallo::Rechazada {
+            motivo,
+            diagnostico,
+        }) => {
+            let i = Informe::nuevo(nombre, Resultado::Rechazada)
+                .con("motivo", motivo)
+                .diagnostico(diagnostico);
+            return Err(con_rechazo(i, op.dejar()));
+        }
+        Err(Fallo::Error(d)) => return Err(Informe::nuevo(nombre, Resultado::Error).diagnostico(d)),
+    };
+    // El trabajo sin publicar del worktree de la tarea en este clon (D4):
+    // antes de escribir nada.
+    if cambia
+        && !para_mi
+        && let Some(m) = git::worktree_publicado(g, e.dir, nombre, id)?
+    {
+        return Err(Informe::nuevo(nombre, Resultado::Rechazada)
+            .con("motivo", "sin-publicar")
+            .con("worktree-motivo", m)
+            .diagnostico(
+                Mensaje::nuevo(&txt::SIN_PUBLICAR)
+                    .con("id", id)
+                    .con("que", m),
+            ));
+    }
+    let t = escribir(e, op.op());
+    match t.resultado {
+        Resultado::Ok | Resultado::SinCambios => {
+            let mut i = t;
+            if let Some(x) = op.dejar().anterior() {
+                i = i.con("asignado-anterior", x);
+            }
+            if let Some(l) = destino_de_op(&op) {
+                i = i.con("asignado", l);
+            }
+            Ok(if para_mi {
+                i
+            } else {
+                con_worktree(i, g, &principal, e.dir, id)
+            })
+        }
+        _ => Err(con_rechazo(t, op.dejar())),
+    }
+}
+
+/// Login de la persona a la que se reasigna.
+fn destino_de_op(op: &Dejada) -> Option<String> {
+    match op {
+        Dejada::Reasignar(r) => r.destino().login().map(str::to_owned),
+        Dejada::Liberar(_) => None,
+    }
+}
+
+/// Un rechazo de `liberar` o `reasignar` con lo que sabe la operacion: quien
+/// la lleva, los dias sin actividad y el umbral, o el arreglo de una tarea
+/// de la 0.16.
+fn con_rechazo(mut i: Informe, d: &Dejar) -> Informe {
+    if i.resultado != Resultado::Rechazada {
+        return i;
+    }
+    match i.valor("motivo") {
+        Some("activa") => {
+            if let Some(k) = d.activa() {
+                i = i.con("dias", k.to_string());
+            }
+            i = i.con("umbral", d.umbral().to_string());
+        }
+        Some("formato-1") => i = i.con("arreglo", "tc2 actualizar"),
+        _ => {}
+    }
+    if matches!(i.valor("motivo"), Some("activa" | "requiere-persona"))
+        && let Some(l) = d.dueno()
+    {
+        i = i.con("asignado", l);
+    }
+    i
+}
+
 /// `descartar` de la tarea `id` con `motivo` (ya validado) en el repo de
 /// `e`.
 pub fn descartar_en(e: &Entorno<'_>, id: &str, motivo: &str) -> Informe {
diff --git a/crates/tc2/src/ciclo/operacion.rs b/crates/tc2/src/ciclo/operacion.rs
index 6432f38..c446c4d 100644
--- a/crates/tc2/src/ciclo/operacion.rs
+++ b/crates/tc2/src/ciclo/operacion.rs
@@ -45,18 +45,24 @@ pub fn motivo_valido(motivo: &str) -> bool {
     crate::tarea::titulo_valido(motivo) && motivo.chars().count() <= MOTIVO_MAX
 }
 
-/// Lo comun de las tres: la tarea, quien la lanza y cuando.
+/// Lo comun de las operaciones de una tarea (tambien las de
+/// [`super::liberar`]): la tarea, quien la lanza y cuando.
 #[derive(Debug)]
-struct Lanzada {
-    id: String,
-    asignado: Asignado,
-    decidido_por: &'static str,
-    ahora: u64,
-    dueno: RefCell<Option<String>>,
+pub(super) struct Lanzada {
+    pub(super) id: String,
+    pub(super) asignado: Asignado,
+    pub(super) decidido_por: &'static str,
+    pub(super) ahora: u64,
+    pub(super) dueno: RefCell<Option<String>>,
 }
 
 impl Lanzada {
-    fn nueva(id: &str, asignado: Asignado, decidido_por: &'static str, ahora: u64) -> Self {
+    pub(super) fn nueva(
+        id: &str,
+        asignado: Asignado,
+        decidido_por: &'static str,
+        ahora: u64,
+    ) -> Self {
         Self {
             id: id.to_owned(),
             asignado,
@@ -90,7 +96,7 @@ impl Lanzada {
     }
 
     /// El `tarea.md` con `actualizado` de hoy y la fila `fase`.
-    fn escribir(&self, mut t: Tarea, mut cuerpo: Cuerpo, fase: &str) -> Vec<Cambio> {
+    pub(super) fn escribir(&self, mut t: Tarea, mut cuerpo: Cuerpo, fase: &str) -> Vec<Cambio> {
         let instante = instante(self.ahora);
         t.actualizado = instante.get(..10).unwrap_or_default().to_owned();
         cuerpo.anadir_transicion(&Transicion {
@@ -107,7 +113,7 @@ impl Lanzada {
     }
 
     /// `<nombre>|TASK-NNN|<id>|<login>`.
-    fn anotacion(&self, nombre: &str) -> String {
+    pub(super) fn anotacion(&self, nombre: &str) -> String {
         format!(
             "{nombre}|{}|{}|{}",
             self.id,
diff --git a/crates/tc2/src/config/claves.rs b/crates/tc2/src/config/claves.rs
index a4d7107..86d0ff5 100644
--- a/crates/tc2/src/config/claves.rs
+++ b/crates/tc2/src/config/claves.rs
@@ -8,12 +8,13 @@ use crate::carril::Carril;
 use crate::i18n::catalogo::config as t;
 use crate::i18n::{Idioma, Texto};
 
-/// Tope de claves de primer nivel (plan, F2-7; F1-11, decision 1).
-pub const TOPE_CLAVES: usize = 12;
+/// Tope de claves de primer nivel (plan, F2-7; F1-11, decision 1). 13 desde
+/// la adenda de TASK-081 en F1-11 (`dias_sin_actividad`, F3-10 D1a).
+pub const TOPE_CLAVES: usize = 13;
 
 /// Fases de `tc2` en las que puede correr un derivado (F1-11 se lo pide a
 /// F2-7): las ordenes que escriben en la rama de estado.
-pub const FASES: [&str; 8] = [
+pub const FASES: [&str; 10] = [
     "new",
     "plan",
     "approve",
@@ -22,6 +23,8 @@ pub const FASES: [&str; 8] = [
     "finish",
     "descartar",
     "pausa",
+    "liberar",
+    "reasignar",
 ];
 
 /// Modelos de la escala de F1-15.
@@ -519,6 +522,13 @@ pub(crate) fn tabla() -> Vec<Campo> {
             "presupuesto por defecto de los lotes; sin cifras inventadas",
         )
         .de("F1-16, consecuencias"),
+        Campo::opcional(
+            "dias_sin_actividad",
+            entero(1, 365),
+            "dias sin actividad tras los que otra persona puede liberar o reasignar una tarea y status y board la marcan",
+        )
+        .con_defecto(json!(7))
+        .de("F1-11 adenda de TASK-081; F3-10 D1a"),
     ]
 }
 
@@ -626,9 +636,12 @@ mod tests {
     use super::*;
 
     #[test]
-    fn claves_de_primer_nivel_no_pasan_de_12_y_cada_una_cita_su_adr() {
+    fn claves_de_primer_nivel_no_pasan_del_tope_y_cada_una_cita_su_adr() {
         let t = tabla();
-        assert_eq!(TOPE_CLAVES, 12, "el tope lo fija el plan (F2-7)");
+        assert_eq!(
+            TOPE_CLAVES, 13,
+            "el tope lo fija el plan (F2-7) y la adenda de TASK-081 en F1-11"
+        );
         assert!(t.len() <= TOPE_CLAVES, "{} claves", t.len());
         for c in &t {
             assert!(!c.origen.is_empty(), "{} sin origen", c.nombre);
diff --git a/crates/tc2/src/config/mod.rs b/crates/tc2/src/config/mod.rs
index 6e005db..3013fe9 100644
--- a/crates/tc2/src/config/mod.rs
+++ b/crates/tc2/src/config/mod.rs
@@ -69,6 +69,9 @@ pub struct Config {
     pub tope_rondas: u8,
     /// Presupuesto por defecto de los lotes (F1-16); `None` si no se declara.
     pub lote: Option<Lote>,
+    /// Dias sin actividad tras los que otra persona puede liberar o
+    /// reasignar una tarea y `status`/`board` la marcan (F3-10, D1a).
+    pub dias_sin_actividad: u16,
 }
 
 /// `politica_carriles.<carril>`: lo que no se declara es el suelo.
@@ -286,6 +289,7 @@ impl Config {
             modo_flujo: ModoFlujo::Manual,
             tope_rondas: 2,
             lote: None,
+            dias_sin_actividad: 7,
         }
     }
 }
diff --git a/crates/tc2/src/git/mod.rs b/crates/tc2/src/git/mod.rs
index 061683c..e172593 100644
--- a/crates/tc2/src/git/mod.rs
+++ b/crates/tc2/src/git/mod.rs
@@ -23,15 +23,15 @@ pub use estado::Cambio;
 pub use guardar::guardar;
 pub use puerto::{ErrorGit, Git, PREFIJO_REFS_PROPIAS, SalidaCruda, SalidaGit, Subcomando};
 pub use rama::{
-    comprobar_id, crear_rama_y_worktree, ids_con_rama, localizar_principal, partes_de_rama,
-    reanudar, validar_base,
+    comprobar_id, comprobar_id_salvo, crear_rama_y_worktree, fechas_de_ramas, ids_con_rama,
+    localizar_principal, partes_de_rama, reanudar, validar_base,
 };
 pub use remoto::{
     ESPERAS_PUSH, FalloRemoto, actualizar_principal, actualizar_rama, clasificar_fallo_remoto,
     fetch, push,
 };
 pub use tag::tag_anotado;
-pub use worktree::quitar_worktree;
+pub use worktree::{quitar_worktree, worktree_publicado};
 
 use std::path::{Component, Path, PathBuf};
 
diff --git a/crates/tc2/src/git/rama.rs b/crates/tc2/src/git/rama.rs
index 7d52940..8ee5748 100644
--- a/crates/tc2/src/git/rama.rs
+++ b/crates/tc2/src/git/rama.rs
@@ -1,7 +1,7 @@
 //! Operaciones de rama y worktree de tarea: `localizar-principal`,
 //! `validar-base`, `comprobar-id`, `crear-rama-y-worktree` y `reanudar`.
 
-use std::collections::BTreeSet;
+use std::collections::{BTreeMap, BTreeSet};
 use std::path::{Path, PathBuf};
 
 use super::estado::hay_remoto;
@@ -54,11 +54,19 @@ pub fn validar_base(git: &Git, dir: &Path, base: &str) -> Informe {
 /// Mira si el ID ya tiene rama, en local o en el remoto (patron
 /// `*/task-nnn-*`). Ocupado → `rechazada motivo=ocupada`.
 pub fn comprobar_id(git: &Git, dir: &Path, id: &str) -> Informe {
+    comprobar_id_salvo(git, dir, id, None)
+}
+
+/// [`comprobar_id`] sin contar `propia`, la rama que la tarea ya tiene en su
+/// `rama:` (F3-10, paso 7 del plan de TASK-081): tras `liberar` o
+/// `reasignar`, la siguiente persona sigue en esa rama. Cualquier otra rama
+/// `*/task-nnn-*` sigue siendo `ocupada`.
+pub fn comprobar_id_salvo(git: &Git, dir: &Path, id: &str, propia: Option<&str>) -> Informe {
     const OP: &str = "comprobar-id";
     if !es_id(id) {
         return Informe::nuevo(OP, Resultado::Uso).con("motivo", "id-invalido");
     }
-    match ramas_del_id(git, dir, OP, id) {
+    match ramas_del_id(git, dir, OP, id).map(|r| ramas_ajenas(r, propia)) {
         Ok(r) if r.is_empty() => Informe::nuevo(OP, Resultado::Ok).con("id", id),
         Ok(r) => Informe::nuevo(OP, Resultado::Rechazada)
             .con("id", id)
@@ -69,6 +77,14 @@ pub fn comprobar_id(git: &Git, dir: &Path, id: &str) -> Informe {
     }
 }
 
+/// `ramas` sin `propia` (comparacion exacta del nombre).
+fn ramas_ajenas(mut ramas: Vec<String>, propia: Option<&str>) -> Vec<String> {
+    if let Some(p) = propia {
+        ramas.retain(|r| r != p);
+    }
+    ramas
+}
+
 fn ramas_del_id(git: &Git, dir: &Path, op: &str, id: &str) -> Paso<Vec<String>> {
     let patron = format!("*/{}-*", id.to_lowercase());
     let remoto = correr(
@@ -179,6 +195,35 @@ pub fn ids_con_rama(git: &Git, dir: &Path, op: &str) -> Paso<BTreeSet<u32>> {
         .collect())
 }
 
+/// Fecha del ultimo commit (committer, segundos desde 1970) de cada rama de
+/// `origin` tal como la conoce el clon (`refs/remotes/origin/*`), por nombre
+/// de rama sin `origin/` (F3-10, D2a). Un solo `for-each-ref` y sin red: la
+/// frescura es la del ultimo `fetch`.
+pub fn fechas_de_ramas(git: &Git, dir: &Path, op: &str) -> Paso<BTreeMap<String, u64>> {
+    let prefijo = format!("refs/remotes/{REMOTO}/");
+    let s = exigir(
+        git,
+        dir,
+        op,
+        Subcomando::ForEachRef,
+        &["--format=%(committerdate:unix) %(refname)", &prefijo],
+    )?;
+    Ok(fechas(&s, &prefijo))
+}
+
+/// Las lineas `<segundos> <ref>` de [`fechas_de_ramas`]; las que no se
+/// entienden (o `HEAD`, que no es una rama) se saltan.
+fn fechas(salida: &str, prefijo: &str) -> BTreeMap<String, u64> {
+    salida
+        .lines()
+        .filter_map(|l| {
+            let (fecha, r) = l.trim().split_once(' ')?;
+            let rama = r.strip_prefix(prefijo).filter(|r| *r != "HEAD")?;
+            Some((rama.to_owned(), fecha.parse().ok()?))
+        })
+        .collect()
+}
+
 /// Valida `id`, `tipo` y `slug` y devuelve el nombre de rama.
 pub(crate) fn rama_valida(op: &str, id: &str, tipo: &str, slug: &str) -> Paso<String> {
     let tipo_ok = !tipo.is_empty() && tipo.bytes().all(|b| b.is_ascii_lowercase());
@@ -426,7 +471,45 @@ pub(crate) fn renombrar(mut i: Informe, op: &str) -> Informe {
 
 #[cfg(test)]
 mod tests {
-    use super::{id_de_rama, nombre_de_rama, nombres, partes_de_rama};
+    use super::{fechas, id_de_rama, nombre_de_rama, nombres, partes_de_rama, ramas_ajenas};
+
+    /// TASK-081: las fechas por rama sin `origin/`, sin `HEAD` ni lineas
+    /// que no se entienden.
+    #[test]
+    fn fechas_de_las_ramas_remotas() {
+        let f = fechas(
+            "1794910830 refs/remotes/origin/feature/task-001-uno\n\
+             1 refs/remotes/origin/HEAD\n\
+             x refs/remotes/origin/feature/mala\n\
+             1700000000 refs/remotes/otro/feature/task-002-x\n\
+             1700000001 refs/remotes/origin/develop\n",
+            "refs/remotes/origin/",
+        );
+        let v: Vec<(&str, u64)> = f.iter().map(|(k, v)| (k.as_str(), *v)).collect();
+        assert_eq!(
+            v,
+            [
+                ("develop", 1_700_000_001),
+                ("feature/task-001-uno", 1_794_910_830)
+            ]
+        );
+    }
+
+    /// TASK-081 (paso 7): la rama propia de la tarea no cuenta como ocupada;
+    /// cualquier otra `*/task-nnn-*`, si.
+    #[test]
+    fn la_rama_propia_no_ocupa_el_id() {
+        let ramas = vec![
+            "feature/task-001-uno".to_owned(),
+            "fix/task-001-a-mano".to_owned(),
+        ];
+        assert_eq!(
+            ramas_ajenas(ramas.clone(), Some("feature/task-001-uno")),
+            ["fix/task-001-a-mano"]
+        );
+        assert_eq!(ramas_ajenas(ramas.clone(), None), ramas);
+        assert!(ramas_ajenas(vec![ramas[0].clone()], Some(&ramas[0])).is_empty());
+    }
 
     /// F3-6: ida y vuelta con `nombre_de_rama`; lo que no es la rama de esa
     /// tarea o no pasa la validacion, `None`.
diff --git a/crates/tc2/src/git/worktree.rs b/crates/tc2/src/git/worktree.rs
index bc6cd7a..375da0c 100644
--- a/crates/tc2/src/git/worktree.rs
+++ b/crates/tc2/src/git/worktree.rs
@@ -12,52 +12,90 @@ use crate::contrato::{Informe, Resultado};
 use crate::i18n::Texto;
 use crate::i18n::catalogo::git as txt;
 
-/// Quita `.taskcodetwo/worktrees/<id>`. No existe → `sin-cambios`. Con cambios
-/// sin guardar o commits sin publicar o HEAD suelto → `rechazada`, sin tocar nada.
-pub fn quitar_worktree(git: &Git, dir: &Path, id: &str) -> Informe {
-    quitar(git, dir, id).unwrap_or_else(|i| i)
+/// Como esta el worktree de una tarea.
+enum Revision {
+    /// No hay worktree registrado en `.taskcodetwo/worktrees/<id>`.
+    Ninguno,
+    /// Limpio y con todo publicado en `origin/<rama>`: se puede quitar.
+    Publicado {
+        /// Rama del worktree.
+        rama: String,
+    },
+    /// No se puede quitar sin perder algo: el motivo y que hacer.
+    Pendiente(&'static str, &'static Texto),
 }
 
-fn quitar(git: &Git, dir: &Path, id: &str) -> Paso<Informe> {
-    const OP: &str = "quitar-worktree";
-    if !es_id(id) {
-        return Err(Informe::nuevo(OP, Resultado::Uso).con("motivo", "id-invalido"));
-    }
-    let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, OP))?;
+/// Revisa el worktree de `id` del repo cuyo principal es `principal`: sin
+/// cambios sin guardar, con rama (no HEAD suelto) y con HEAD contenido en
+/// `origin/<rama>` tal como la conoce el clon.
+fn revisar(git: &Git, principal: &Path, op: &str, id: &str) -> Paso<Revision> {
     let wt = principal.join(CARPETA_WORKTREES).join(id);
-    let Some(w) = worktrees(git, &principal, OP)?
+    let Some(w) = worktrees(git, principal, op)?
         .into_iter()
         .find(|w| misma_ruta(&w.ruta, &wt))
     else {
-        return Ok(Informe::nuevo(OP, Resultado::SinCambios).con("id", id));
-    };
-    let rechazo = |motivo: &str, que_hacer: &'static Texto| {
-        Informe::nuevo(OP, Resultado::Rechazada)
-            .con("id", id)
-            .con("motivo", motivo)
-            .diagnostico(que_hacer)
+        return Ok(Revision::Ninguno);
     };
-    let status = exigir(git, &w.ruta, OP, Subcomando::Status, &["--porcelain", "-z"])?;
+    let status = exigir(git, &w.ruta, op, Subcomando::Status, &["--porcelain", "-z"])?;
     if !status.is_empty() {
-        return Ok(rechazo("cambios-sin-guardar", &txt::WORKTREE_CON_CAMBIOS));
+        return Ok(Revision::Pendiente(
+            "cambios-sin-guardar",
+            &txt::WORKTREE_CON_CAMBIOS,
+        ));
     }
     // Con HEAD suelto no hay rama remota con la que comparar: sus commits se
     // perderian al quitar el worktree.
-    let Some(rama) = &w.rama else {
-        return Ok(rechazo("sin-rama", &txt::WORKTREE_HEAD_SUELTO));
+    let Some(rama) = w.rama else {
+        return Ok(Revision::Pendiente("sin-rama", &txt::WORKTREE_HEAD_SUELTO));
     };
-    let publicada = match commit_de(
-        git,
-        &principal,
-        OP,
-        &format!("refs/remotes/{REMOTO}/{rama}"),
-    )? {
-        Some(remota) => es_ancestro(git, &w.ruta, OP, "HEAD", &remota)?,
+    let publicada = match commit_de(git, principal, op, &format!("refs/remotes/{REMOTO}/{rama}"))? {
+        Some(remota) => es_ancestro(git, &w.ruta, op, "HEAD", &remota)?,
         None => false,
     };
     if !publicada {
-        return Ok(rechazo("sin-publicar", &txt::WORKTREE_SIN_PUBLICAR));
+        return Ok(Revision::Pendiente(
+            "sin-publicar",
+            &txt::WORKTREE_SIN_PUBLICAR,
+        ));
+    }
+    Ok(Revision::Publicado { rama })
+}
+
+/// Lo que se perderia al dejar la tarea de `id` (F3-10, D4): `None` si no
+/// hay worktree o esta limpio y publicado; si no, el motivo
+/// (`cambios-sin-guardar`, `sin-rama` o `sin-publicar`). Es la misma
+/// comprobacion con la que [`quitar_worktree`] decide si lo quita. `op`
+/// nombra los fallos de git.
+pub fn worktree_publicado(git: &Git, dir: &Path, op: &str, id: &str) -> Paso<Option<&'static str>> {
+    let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, op))?;
+    Ok(match revisar(git, &principal, op, id)? {
+        Revision::Ninguno | Revision::Publicado { .. } => None,
+        Revision::Pendiente(motivo, _) => Some(motivo),
+    })
+}
+
+/// Quita `.taskcodetwo/worktrees/<id>`. No existe → `sin-cambios`. Con cambios
+/// sin guardar o commits sin publicar o HEAD suelto → `rechazada`, sin tocar nada.
+pub fn quitar_worktree(git: &Git, dir: &Path, id: &str) -> Informe {
+    quitar(git, dir, id).unwrap_or_else(|i| i)
+}
+
+fn quitar(git: &Git, dir: &Path, id: &str) -> Paso<Informe> {
+    const OP: &str = "quitar-worktree";
+    if !es_id(id) {
+        return Err(Informe::nuevo(OP, Resultado::Uso).con("motivo", "id-invalido"));
     }
+    let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, OP))?;
+    let rama = match revisar(git, &principal, OP, id)? {
+        Revision::Ninguno => return Ok(Informe::nuevo(OP, Resultado::SinCambios).con("id", id)),
+        Revision::Pendiente(motivo, que_hacer) => {
+            return Ok(Informe::nuevo(OP, Resultado::Rechazada)
+                .con("id", id)
+                .con("motivo", motivo)
+                .diagnostico(que_hacer));
+        }
+        Revision::Publicado { rama } => rama,
+    };
     let rel = worktree_relativo(id);
     exigir(git, &principal, OP, Subcomando::Worktree, &["remove", &rel])?;
     Ok(Informe::nuevo(OP, Resultado::Ok)
diff --git a/crates/tc2/src/i18n/catalogo/ciclo.rs b/crates/tc2/src/i18n/catalogo/ciclo.rs
index 965a891..046ad69 100644
--- a/crates/tc2/src/i18n/catalogo/ciclo.rs
+++ b/crates/tc2/src/i18n/catalogo/ciclo.rs
@@ -1,6 +1,31 @@
-//! Mensajes de `tc2 descartar`, `tc2 pausa` y `tc2 reanudar` (F3-7).
+//! Mensajes de `tc2 descartar`, `tc2 pausa` y `tc2 reanudar` (F3-7) y de
+//! `tc2 liberar` y `tc2 reasignar` (F3-10).
 
 crate::i18n::textos! {
+    USO_LIBERAR {
+        es: "uso: liberar [--json] TASK-NNN",
+        en: "usage: liberar [--json] TASK-NNN",
+    }
+    USO_REASIGNAR {
+        es: "uso: reasignar [--json] TASK-NNN <login>",
+        en: "usage: reasignar [--json] TASK-NNN <login>",
+    }
+    ACTIVA {
+        es: "la tarea {id} la lleva {login} y tuvo actividad hace {dias} dias (filas de Transiciones o commits de su rama publicada): solo su dueno puede lanzar tc2 {orden} sobre ella antes de {umbral} dias sin actividad (clave dias_sin_actividad)",
+        en: "task {id} is taken by {login} and had activity {dias} days ago (Transitions rows or commits on its published branch): only its owner can run tc2 {orden} on it before {umbral} days without activity (dias_sin_actividad key)",
+    }
+    REQUIERE_PERSONA {
+        es: "la tarea {id} la lleva {quien}: quitarsela a otra persona lo decide una persona en un terminal (tc2 {orden} con TC2_TOKEN_PERSONA), no el agente",
+        en: "task {id} is taken by {quien}: taking it from someone else is decided by a person at a terminal (tc2 {orden} with TC2_TOKEN_PERSONA), not by the agent",
+    }
+    FUERA_DE_LISTA {
+        es: "{login} no es una persona de {ruta} en {base}: reasigna a un login de la lista (un bot o un login mal escrito no valen)",
+        en: "{login} is not a person in {ruta} on {base}: reassign to a login on the list (a bot or a misspelt login are not valid)",
+    }
+    SIN_PUBLICAR {
+        es: "el worktree de {id} tiene trabajo que se perderia ({que}): guardalo y publica la rama (tc2 guardar y tc2 actualizar-rama) antes de dejar la tarea",
+        en: "the worktree of {id} has work that would be lost ({que}): save it and publish the branch (tc2 guardar and tc2 actualizar-rama) before leaving the task",
+    }
     USO_DESCARTAR {
         es: "uso: descartar [--json] TASK-NNN \"<motivo>\"",
         en: "usage: descartar [--json] TASK-NNN \"<reason>\"",
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index 009b406..b2fcb66 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/tablero.rs b/crates/tc2/src/i18n/catalogo/tablero.rs
index 330aaf7..0173d8d 100644
--- a/crates/tc2/src/i18n/catalogo/tablero.rs
+++ b/crates/tc2/src/i18n/catalogo/tablero.rs
@@ -37,6 +37,14 @@ crate::i18n::textos! {
         es: "aviso: la rama local {rama} tiene commits que no estan en lo publicado; {carpeta}/ no se adelanta: revisalos",
         en: "warning: the local branch {rama} has commits that are not in what was published; {carpeta}/ is not fast-forwarded: review them",
     }
+    AVISO_YA_NO_TUYA {
+        es: "aviso: {id} ya no es tuya: la lleva {quien}; tu worktree sigue en {ruta} y la rama no se ha tocado",
+        en: "warning: {id} is no longer yours: {quien} has it; your worktree is still at {ruta} and the branch was not touched",
+    }
+    AVISO_YA_NO_TUYA_LIBRE {
+        es: "aviso: {id} ya no es tuya: esta libre para quien la empiece; tu worktree sigue en {ruta} y la rama no se ha tocado",
+        en: "warning: {id} is no longer yours: it is free for whoever starts it; your worktree is still at {ruta} and the branch was not touched",
+    }
     AVISO_PR_SIN_REVISION {
         es: "aviso: con PR abierto listo y sin pasar a revision: {ids}",
         en: "warning: with an open ready pull request but not in review: {ids}",
diff --git a/crates/tc2/src/inventario.rs b/crates/tc2/src/inventario.rs
index 02388e6..552d270 100644
--- a/crates/tc2/src/inventario.rs
+++ b/crates/tc2/src/inventario.rs
@@ -603,7 +603,7 @@ fn dias_desde_epoca(anio: i64, mes: i64, dia: i64) -> i64 {
 /// Milisegundos desde la epoca de un texto ISO 8601 / RFC 3339
 /// (`AAAA-MM-DD`, opcionalmente `THH:MM[:SS[.fff]]` y `Z` o `+HH:MM`);
 /// `None` si no tiene esa forma o algun campo esta fuera de rango.
-fn ms_de_iso(s: &str) -> Option<i64> {
+pub(crate) fn ms_de_iso(s: &str) -> Option<i64> {
     let num = |x: &str| -> Option<i64> {
         (!x.is_empty() && x.bytes().all(|b| b.is_ascii_digit()))
             .then(|| x.parse().ok())
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index e856f6c..566de32 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -17,7 +17,8 @@
 //! [`siguiente`] (orden de trabajo y `next`, F3-3) y [`situacion`] y
 //! [`roadmap`] (el calculo comun y `roadmap`, F3-4) y [`tablero`]
 //! (`board` y `status`, F3-5) y [`toma`] (`start`, F3-6) y [`ciclo`]
-//! (`descartar`, `pausa` y `reanudar`, F3-7) y [`doctor`] (F3-8). Las capas
+//! (`descartar`, `pausa` y `reanudar`, F3-7; `liberar` y `reasignar`,
+//! F3-10) y [`doctor`] (F3-8). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
@@ -170,6 +171,16 @@ where
             let reanudar = orden == ciclo::REANUDAR;
             de_una_tarea(orden, |e| ciclo::orden_pausa(&resto, reanudar, e)).salida_en(json, idioma)
         }
+        [orden, resto @ ..] if orden == ciclo::LIBERAR => {
+            let (json, resto) = con_json(resto);
+            de_una_tarea(ciclo::LIBERAR, |e| ciclo::orden_liberar(&resto, e))
+                .salida_en(json, idioma)
+        }
+        [orden, resto @ ..] if orden == ciclo::REASIGNAR => {
+            let (json, resto) = con_json(resto);
+            de_una_tarea(ciclo::REASIGNAR, |e| ciclo::orden_reasignar(&resto, e))
+                .salida_en(json, idioma)
+        }
         [orden, resto @ ..] if orden == identidad::NOMBRE => {
             let (json, resto) = con_json(resto);
             orden_identidad(&resto).salida_en(json, idioma)
@@ -302,7 +313,8 @@ fn orden_identidad(args: &[String]) -> Informe {
 }
 
 /// Una orden sobre una tarea (`start`, F3-6; `descartar`, `pausa` y
-/// `reanudar`, F3-7) en la carpeta actual: quien la lanza sale de
+/// `reanudar`, F3-7; `liberar` y `reasignar`, F3-10) en la carpeta actual:
+/// quien la lanza sale de
 /// [`identidad::resolver`] con el terminal y los tokens del proceso.
 fn de_una_tarea(nombre: &str, orden: impl FnOnce(&toma::Entorno<'_>) -> Informe) -> Informe {
     use std::io::IsTerminal;
diff --git a/crates/tc2/src/plataforma/personas.rs b/crates/tc2/src/plataforma/personas.rs
index 12511d5..89ea04d 100644
--- a/crates/tc2/src/plataforma/personas.rs
+++ b/crates/tc2/src/plataforma/personas.rs
@@ -236,6 +236,16 @@ impl Personas {
         self.entradas.iter().find(|p| p.id == id)
     }
 
+    /// La persona de ese login, sin distinguir mayusculas (como los valida
+    /// [`Personas::leer`]: no hay dos que solo difieran en ellas). `reasignar`
+    /// (F3-10) la busca asi: la lista es la autoridad.
+    pub fn por_login(&self, login: &str) -> Option<&Persona> {
+        let login = login.to_lowercase();
+        self.entradas
+            .iter()
+            .find(|p| p.login.to_lowercase() == login)
+    }
+
     /// La persona que declara ese correo de la 0.16 (sin distinguir
     /// mayusculas).
     pub fn por_correo(&self, correo: &str) -> Option<&Persona> {
@@ -386,6 +396,21 @@ mod tests {
         Ok(())
     }
 
+    /// TASK-081: por login sin distinguir mayusculas; uno que no esta (mal
+    /// escrito, con espacios o vacio), `None`.
+    #[test]
+    fn por_login_sin_distinguir_mayusculas() -> Result<(), Informe> {
+        let l = Personas::leer(
+            "personas:\n  - {id: 7, login: \"Ana-G\"}\n  - {id: 9, login: \"luis\"}\n",
+        )?;
+        assert_eq!(l.por_login("ana-g").map(|p| p.id), Some(7));
+        assert_eq!(l.por_login("LUIS").map(|p| p.id), Some(9));
+        for malo in ["ana", "ana-g ", "", "luis[bot]"] {
+            assert!(l.por_login(malo).is_none(), "{malo:?}");
+        }
+        Ok(())
+    }
+
     #[test]
     fn correos_y_claves() -> Result<(), Informe> {
         let l = Personas::leer(
diff --git a/crates/tc2/src/situacion.rs b/crates/tc2/src/situacion.rs
index a742997..40c774b 100644
--- a/crates/tc2/src/situacion.rs
+++ b/crates/tc2/src/situacion.rs
@@ -31,6 +31,26 @@ pub struct Ficha {
     /// En pausa: la ultima fila `pausa`/`reanudar` de sus transiciones es
     /// `pausa` ([`crate::tarea::Cuerpo::en_pausa`], TASK-034).
     pub en_pausa: bool,
+    /// Instante (segundos desde 1970) de la ultima fila de sus transiciones
+    /// ([`crate::tarea::Cuerpo::ultima_fecha`], F3-10).
+    pub ultima_transicion: Option<u64>,
+}
+
+/// Segundos de un dia.
+const DIA: u64 = 86_400;
+
+/// La actividad de una tarea (F3-10, D2a): el instante mas reciente entre
+/// la ultima fila de sus transiciones (tambien `pausa` y `reanudar`: la
+/// pausa no exime) y el ultimo commit de `origin/<rama>` (`commit_rama`).
+/// Sin ninguna de las dos, `None`: no se sabe.
+pub fn actividad(ficha: &Ficha, commit_rama: Option<u64>) -> Option<u64> {
+    ficha.ultima_transicion.max(commit_rama)
+}
+
+/// Dias completos sin actividad desde `ultima` hasta `ahora`. Una actividad
+/// del futuro (reloj adelantado) cuenta como de hoy: 0.
+pub fn dias_sin_actividad(ultima: u64, ahora: u64) -> u64 {
+    ahora.saturating_sub(ultima) / DIA
 }
 
 impl Ficha {
@@ -59,6 +79,7 @@ impl Ficha {
             rama: rama.filter(|r| !r.trim().is_empty()),
             motivo,
             en_pausa: cuerpo.en_pausa(),
+            ultima_transicion: cuerpo.ultima_fecha(),
         }
     }
 }
@@ -180,3 +201,35 @@ pub fn calcular(estado: &dyn Estado) -> Result<Situacion, Fallo> {
         roadmap,
     })
 }
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::tarea::{Cuerpo, Tarea};
+
+    fn ficha(cuerpo: &str) -> Ficha {
+        let t = Tarea::nueva("TASK-001", "Uno", "2026-11-01");
+        Ficha::de(&Documento::Formato2(t, Cuerpo::leer(cuerpo)))
+    }
+
+    /// TASK-081 (D2a): la actividad es la mas reciente de la ultima fila y
+    /// del commit de la rama; sin ninguna, no se sabe. Los dias son
+    /// completos y una actividad del futuro cuenta como de hoy.
+    #[test]
+    fn actividad_y_dias_sin_actividad() {
+        let f = ficha(
+            "## Transiciones\n| fecha | fase |\n|---|---|\n| 2026-11-17T10:20:30Z | pausa |\n",
+        );
+        let fila = 1_794_910_830;
+        assert_eq!(f.ultima_transicion, Some(fila));
+        assert_eq!(actividad(&f, None), Some(fila));
+        assert_eq!(actividad(&f, Some(fila - 5)), Some(fila));
+        assert_eq!(actividad(&f, Some(fila + 5)), Some(fila + 5));
+        let sin = ficha("");
+        assert_eq!(actividad(&sin, None), None);
+        assert_eq!(actividad(&sin, Some(3)), Some(3));
+        assert_eq!(dias_sin_actividad(fila, fila + 7 * DIA - 1), 6);
+        assert_eq!(dias_sin_actividad(fila, fila + 7 * DIA), 7);
+        assert_eq!(dias_sin_actividad(fila + 100, fila), 0);
+    }
+}
diff --git a/crates/tc2/src/tablero.rs b/crates/tc2/src/tablero.rs
index bca8168..cf412f5 100644
--- a/crates/tc2/src/tablero.rs
+++ b/crates/tc2/src/tablero.rs
@@ -16,6 +16,15 @@
 //!   diseno, en curso, en revision y listas para empezar), con ID, titulo,
 //!   quien y PR; terminadas y descartadas solo como recuento.
 //!
+//! F3-10 (TASK-081): las dos cuentan como inactivas las tareas en curso o
+//! en revision con `dias_sin_actividad` dias o mas sin actividad (filas de
+//! transicion y commits de `origin/<rama>` tal como los conoce el clon):
+//! `status` con `inactivas=N` (en texto, `· sin actividad N` si N > 0) y
+//! `board` con `inactivas=` y la fila marcada. Si este clon tiene el
+//! worktree de una tarea abierta que ya no lleva la persona de su cache de
+//! identidad, `ya-no-tuyas=` y un aviso en stderr con quien la lleva ahora
+//! (codigo 0).
+//!
 //! El texto ajeno (titulos, logins, asignados heredados) se sanea al
 //! imprimirlo (F1-17, decision 8): sin caracteres de control ni de formato y
 //! con longitud acotada.
@@ -83,6 +92,9 @@ pub struct Tablero {
     /// Hora de la ultima sincronizacion para el texto (`None`: no se
     /// ensena, sin `origin`).
     pub sincronizado: Option<String>,
+    /// En curso o en revision sin actividad desde hace el umbral o mas
+    /// (F3-10, C9): ID y dias completos.
+    pub inactivas: BTreeMap<String, u64>,
 }
 
 impl Tablero {
@@ -149,6 +161,108 @@ pub fn tablero(s: &Situacion, personas: &Personas, commit: &str) -> Tablero {
     t
 }
 
+/// Las tareas en curso o en revision de `t` sin actividad desde hace
+/// `umbral` dias o mas en `ahora` (F3-10, D2a): la actividad es la de
+/// [`situacion::actividad`] con el commit de `origin/<rama>` de `fechas`
+/// tal como la conoce el clon (sin traer: avisa de mas, nunca de menos).
+/// Una tarea sin ninguna fecha no se marca: no se sabe.
+pub fn inactivas(
+    s: &Situacion,
+    t: &Tablero,
+    fechas: &BTreeMap<String, u64>,
+    ahora: u64,
+    umbral: u64,
+) -> BTreeMap<String, u64> {
+    let mut v = BTreeMap::new();
+    for f in t.en_curso.iter().chain(&t.en_revision) {
+        let Some(ficha) = s.fichas.iter().find(|x| x.nodo.id == f.id) else {
+            continue;
+        };
+        let rama = ficha.rama.as_ref().and_then(|r| fechas.get(r)).copied();
+        if let Some(a) = situacion::actividad(ficha, rama) {
+            let d = situacion::dias_sin_actividad(a, ahora);
+            if d >= umbral {
+                v.insert(f.id.clone(), d);
+            }
+        }
+    }
+    v
+}
+
+/// Las tareas abiertas con worktree en este clon que ya no lleva la persona
+/// de la cache de identidad (F3-10, D5a: el aviso al dueno anterior sin
+/// plataforma), con quien la lleva ahora. Sin cache o sin worktrees, nada.
+/// Solo lee: la cache sin comprobar su vigencia y `git worktree list`.
+fn ya_no_tuyas(
+    g: &Git,
+    dir: &Path,
+    op: &str,
+    s: &Situacion,
+    personas: &Personas,
+) -> Vec<(String, Quien)> {
+    let Ok(principal) = crate::git::localizar_principal(g, dir) else {
+        return Vec::new();
+    };
+    let Ok(comun) = crate::git::estado::dir_comun(g, &principal, op) else {
+        return Vec::new();
+    };
+    let Some(yo) = crate::identidad::Cache::en(&comun).leer() else {
+        return Vec::new();
+    };
+    let Ok(wts) = crate::git::worktrees(g, &principal, op) else {
+        return Vec::new();
+    };
+    let carpeta = principal.join(crate::git::CARPETA_WORKTREES);
+    let mut v = Vec::new();
+    for w in wts {
+        let Some(id) = w
+            .ruta
+            .file_name()
+            .and_then(|n| n.to_str())
+            .filter(|id| crate::git::misma_ruta(&w.ruta, &carpeta.join(id)))
+        else {
+            continue;
+        };
+        let Some(f) = s.fichas.iter().find(|f| f.nodo.id == id) else {
+            continue;
+        };
+        if !f
+            .nodo
+            .estado()
+            .is_some_and(|e| crate::ciclo::operacion::ABIERTAS.contains(&e))
+        {
+            continue;
+        }
+        let mia = f
+            .asignado
+            .clone()
+            .map(|a| crate::identidad::convertir_heredado(a, personas))
+            .is_some_and(|a| a.la_lleva(yo.cuenta.id));
+        if !mia {
+            v.push((
+                id.to_owned(),
+                situacion::quien(f.asignado.as_ref(), personas),
+            ));
+        }
+    }
+    v.sort_by(|a, b| a.0.cmp(&b.0));
+    v
+}
+
+/// El aviso de cada tarea que ya no es tuya, para stderr.
+fn avisos_ya_no_tuyas(v: &[(String, Quien)]) -> Vec<Mensaje> {
+    v.iter()
+        .map(|(id, q)| {
+            let m = match q {
+                Quien::Nadie => Mensaje::nuevo(&txt::AVISO_YA_NO_TUYA_LIBRE),
+                q => Mensaje::nuevo(&txt::AVISO_YA_NO_TUYA).con("quien", quien_impreso(q)),
+            };
+            m.con("id", id.as_str())
+                .con("ruta", crate::git::worktree_relativo(id))
+        })
+        .collect()
+}
+
 /// Texto ajeno listo para imprimir (F1-17, decision 8): sin caracteres de
 /// control ni de formato (bidi, anchura cero), espacios juntados y como
 /// mucho `max` caracteres (con `…` si se corta).
@@ -254,6 +368,9 @@ pub fn texto_board(t: &Tablero, plataforma: &str, pr: Option<&dyn Fn(&Fila) -> S
                 l.push_str("  ");
                 l.push_str(&c[3]);
             }
+            if let Some(d) = t.inactivas.get(&c[0]) {
+                let _ = write!(l, "  · sin actividad hace {d} días");
+            }
             o.push_str(l.trim_end());
             o.push('\n');
         }
@@ -291,6 +408,8 @@ struct Leido {
     local: Option<Local>,
     /// Cuando se leyo, para el «hace N».
     ahora: u64,
+    /// Las tareas con worktree en este clon que ya no son de quien lo usa.
+    ya_no_tuyas: Vec<(String, Quien)>,
 }
 
 /// El tablero del estado que conoce el clon de `dir` tras traer la rama de
@@ -326,6 +445,11 @@ fn tablero_en(g: &Git, dir: &Path, op: &str, con_local: bool) -> Result<Leido, I
     };
     let mut t = tablero(&s, &personas, &commit);
     t.sincronizado = frescura.en_texto();
+    // F3-10: sin red y sin escribir nada; un fallo deja sin marcas.
+    let fechas = crate::git::fechas_de_ramas(g, dir, op).unwrap_or_default();
+    let umbral = crate::ciclo::dias_sin_actividad(g, dir);
+    t.inactivas = inactivas(&s, &t, &fechas, ahora, umbral);
+    let ya_no_tuyas = ya_no_tuyas(g, dir, op, &s, &personas);
     Ok(Leido {
         tablero: t,
         avisos,
@@ -333,6 +457,7 @@ fn tablero_en(g: &Git, dir: &Path, op: &str, con_local: bool) -> Result<Leido, I
         frescura,
         local,
         ahora,
+        ya_no_tuyas,
     })
 }
 
@@ -459,6 +584,19 @@ pub fn status_en(e: &Entorno<'_>) -> Result<(String, Informe, Vec<Mensaje>), Inf
             avisos.push(m.con("carpeta", estado::CARPETA));
         }
     }
+    // F3-10: las que llevan el umbral sin actividad (en texto, solo si hay
+    // alguna) y las que ya no son de quien usa este clon.
+    i = i.con("inactivas", t.inactivas.len().to_string());
+    if !t.inactivas.is_empty() {
+        let _ = write!(linea, " · sin actividad {}", t.inactivas.len());
+    }
+    if !l.ya_no_tuyas.is_empty() {
+        i = i.con_lista(
+            "ya-no-tuyas",
+            l.ya_no_tuyas.iter().map(|(id, _)| id.clone()).collect(),
+        );
+        avisos.extend(avisos_ya_no_tuyas(&l.ya_no_tuyas));
+    }
     Ok((linea, i, avisos))
 }
 
@@ -520,12 +658,16 @@ pub fn board_en(e: &Entorno<'_>, modo: Modo) -> Result<(String, Informe, Vec<Men
             Err(c) => c,
         },
     };
-    let (texto, i) = board_de(&l.tablero, &consulta, &l.avisos);
-    Ok((
-        texto,
-        l.frescura.anotar(i),
-        l.frescura.aviso(l.ahora).into_iter().collect(),
-    ))
+    let (texto, mut i) = board_de(&l.tablero, &consulta, &l.avisos);
+    let mut avisos: Vec<Mensaje> = l.frescura.aviso(l.ahora).into_iter().collect();
+    if !l.ya_no_tuyas.is_empty() {
+        i = i.con_lista(
+            "ya-no-tuyas",
+            l.ya_no_tuyas.iter().map(|(id, _)| id.clone()).collect(),
+        );
+        avisos.extend(avisos_ya_no_tuyas(&l.ya_no_tuyas));
+    }
+    Ok((texto, l.frescura.anotar(i), avisos))
 }
 
 /// Texto e informe de `board` para un tablero y lo que se supo de la
@@ -589,6 +731,9 @@ fn informe_board(t: &Tablero, plataforma: &str, cruce: Option<&Cruce>, avisos: &
             }
         }
     }
+    if !t.inactivas.is_empty() {
+        i = i.con_lista("inactivas", t.inactivas.keys().cloned().collect());
+    }
     i = i
         .con("terminadas", t.terminadas.to_string())
         .con("descartadas", t.descartadas.to_string());
@@ -882,9 +1027,59 @@ mod tests {
             descartadas: 1,
             ilegibles: Vec::new(),
             sincronizado: None,
+            inactivas: BTreeMap::new(),
         }
     }
 
+    /// TASK-081 C9 (parte pura): en curso o en revision con el umbral o mas
+    /// sin actividad se marcan; un commit reciente de la rama lo evita; sin
+    /// fechas no se marca; las de otros grupos no cuentan. `board` marca la
+    /// fila y `status` cuenta.
+    #[test]
+    fn inactivas_por_filas_y_commits_de_la_rama() {
+        use crate::tarea::{Cuerpo, Documento, Tarea};
+        let ahora = 1_794_910_830;
+        let ficha = |id: &str, rama: &str, cuerpo: &str| {
+            let mut t = Tarea::nueva(id, "t", "2026-11-01");
+            t.rama = Some(rama.to_owned());
+            Ficha::de(&Documento::Formato2(t, Cuerpo::leer(cuerpo)))
+        };
+        let vieja = "## Transiciones\n| 2026-11-01T10:20:30Z | start |\n";
+        let s = Situacion {
+            fichas: vec![
+                ficha("TASK-002", "feature/b", vieja),
+                ficha("TASK-003", "feature/c", vieja),
+                ficha("TASK-004", "feature/d", ""),
+                ficha("TASK-001", "feature/a", vieja),
+                ficha("TASK-006", "feature/f", vieja),
+            ],
+            ..Situacion::default()
+        };
+        let fechas: BTreeMap<String, u64> = [("feature/c".to_owned(), ahora - 3600)].into();
+        let t = de_prueba();
+        let v = inactivas(&s, &t, &fechas, ahora, 7);
+        assert_eq!(
+            v.into_iter().collect::<Vec<_>>(),
+            [("TASK-002".to_owned(), 16)]
+        );
+        assert!(inactivas(&s, &t, &fechas, ahora, 17).is_empty());
+        let t = Tablero {
+            inactivas: [("TASK-002".to_owned(), 16)].into(),
+            ..de_prueba()
+        };
+        let texto = texto_board(&t, "no-consultada", None);
+        assert!(
+            texto.contains("  TASK-002  Titulo de TASK-002  @ana  · sin actividad hace 16 días\n"),
+            "{texto}"
+        );
+        assert!(
+            !texto.contains("TASK-003  Titulo de TASK-003  @luis  ·"),
+            "{texto}"
+        );
+        let linea = board_de(&t, &Consulta::NoConsultada, &[]).1.linea();
+        assert!(linea.contains(" inactivas=TASK-002 "), "{linea}");
+    }
+
     /// C3 (parte pura): la hora va al final de la linea de `status` y de la
     /// cabecera de `board`; sin hora (sin `origin`), no sale.
     #[test]
diff --git a/crates/tc2/src/tarea/cuerpo.rs b/crates/tc2/src/tarea/cuerpo.rs
index 9a16481..7ddf476 100644
--- a/crates/tc2/src/tarea/cuerpo.rs
+++ b/crates/tc2/src/tarea/cuerpo.rs
@@ -234,6 +234,21 @@ impl Cuerpo {
             .find(|fase| *fase == PAUSA || *fase == REANUDAR)
             == Some(PAUSA)
     }
+
+    /// Instante, en segundos desde 1970 (UTC), de la ultima fila de
+    /// `## Transiciones` cuya fecha se entiende (F3-10, D2a: la actividad de
+    /// la tarea; cuentan tambien `pausa` y `reanudar`). Sin seccion o sin
+    /// ninguna fila fechada, `None`. La cabecera y el separador no son
+    /// fechas y se saltan.
+    pub fn ultima_fecha(&self) -> Option<u64> {
+        let s = self.seccion(TRANSICIONES)?;
+        s.texto
+            .lines()
+            .rev()
+            .filter_map(|l| l.trim().strip_prefix('|')?.split('|').next())
+            .find_map(|fecha| crate::inventario::ms_de_iso(fecha.trim()))
+            .and_then(|ms| u64::try_from(ms / 1000).ok())
+    }
 }
 
 /// Fase de la fila que pone una tarea en pausa (`tc2 pausa`).
@@ -429,6 +444,32 @@ mod tests {
         assert!(!Cuerpo::leer("").en_pausa());
     }
 
+    /// TASK-081 (D2a): la ultima fila fechada de `## Transiciones`, tambien
+    /// una `pausa`; sin filas, ni cabecera ni separador cuentan; una fila de
+    /// otra seccion, tampoco.
+    #[test]
+    fn ultima_fecha_de_las_transiciones() {
+        let mut c = Cuerpo::plantilla();
+        assert_eq!(c.ultima_fecha(), None);
+        c.anadir_transicion(&T);
+        // 2026-11-17T10:00:00Z.
+        assert_eq!(c.ultima_fecha(), Some(1_794_909_600));
+        c.anadir_transicion(&Transicion {
+            fecha: "2026-11-20T00:00:01Z",
+            fase: PAUSA,
+            ..T
+        });
+        assert_eq!(c.ultima_fecha(), Some(1_795_132_801));
+        // Una fila final que no se entiende no tapa la anterior.
+        let c = Cuerpo::leer(&format!("{}| basura | x |\n", c.escribir()));
+        assert_eq!(c.ultima_fecha(), Some(1_795_132_801));
+        let c = Cuerpo::leer(
+            "## Objetivo\n| 2030-01-01T00:00:00Z | start |\n## Transiciones\n| fecha | fase |\n|---|---|\n",
+        );
+        assert_eq!(c.ultima_fecha(), None);
+        assert_eq!(Cuerpo::leer("").ultima_fecha(), None);
+    }
+
     #[test]
     fn cercas_como_commonmark() {
         let titulos = |t: &str| -> Vec<String> {
diff --git a/crates/tc2/src/toma/mod.rs b/crates/tc2/src/toma/mod.rs
index 4c2a575..72f6bba 100644
--- a/crates/tc2/src/toma/mod.rs
+++ b/crates/tc2/src/toma/mod.rs
@@ -16,8 +16,9 @@
 //! (identidad, D6); la toma en seco sobre lo leido (los rechazos salen sin
 //! esperar al push); la base valida (`origin/develop`, o para `hotfix` el
 //! ultimo tag de `origin/main` o su punta, D4; `release` se rechaza);
-//! `comprobar-id` (una rama `*/task-nnn-*` que no es de quien la lanza,
-//! `ocupada`); la carpeta del worktree libre (`carpeta-ocupada`); la
+//! `comprobar-id` (una rama `*/task-nnn-*` que no es la de su `rama:`,
+//! `ocupada`; la propia la retoma, F3-10); la carpeta del worktree libre
+//! (`carpeta-ocupada`); la
 //! transaccion; la rama y el worktree.
 //!
 //! Lanzada desde un worktree de tarea escribe el estado en el principal (lo
@@ -170,8 +171,10 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
         // 5c. Ya es suya y esta empezada: crear solo lo que falte.
         return Ok(destino.crear(None, &quien));
     }
-    // 5. El ID no tiene rama de nadie mas.
-    let c = git::comprobar_id(g, &principal, id);
+    // 5. El ID no tiene rama de nadie mas. La suya propia (la de `rama:`)
+    // no cuenta: tras `liberar` o `reasignar` (F3-10) la siguiente persona
+    // sigue en ella y `crear` la recupera con `git::reanudar`.
+    let c = git::comprobar_id_salvo(g, &principal, id, Some(&rama));
     if !c.resultado.es_exito() {
         return Err(c);
     }
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-081/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-081/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-081/tarea.md                                    | 35 +++++++++++++++++++++++++++++++++--
 3 files changed, 33 insertions(+), 2 deletions(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/liberar.rs" ":(literal)crates/tc2/tests/tablero.rs"
````

````
crates/tc2/tests/liberar.rs | 758 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/tablero.rs | 130 ++++++++++++++++++++++++++++
 2 files changed, 888 insertions(+)
````
