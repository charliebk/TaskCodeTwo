# Peticion de revision — TASK-023 (ronda 1)

- Tarea: TASK-023 — F2-7 Configuracion minima con esquema validado y valores por defecto
- Rama revisada: feature/task-023-f2-7-configuracion-minima-con-esquema-va
- Rama base: develop
- Commit revisado (HEAD): 25e7d2ce796319aac3b8ab5b4d26518c6c197f78
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-023 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-023/tarea.md
- tareas/03-en-revision/TASK-023/planificacion/
- crates/tc2/src/carril.rs
- crates/tc2/src/config/claves.rs
- crates/tc2/src/config/mod.rs
- crates/tc2/src/config/validar.rs
- crates/tc2/src/git/estado.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/tarea/yaml.rs
- crates/tc2/src/yaml.rs
- crates/tc2/tests/config.rs

## Commits a revisar (git log develop..HEAD)

````
25e7d2c chore(TASK-023): autorrevision
29cf97f feat(TASK-023): configuracion minima con esquema validado y valores por defecto
cb57d49 chore(TASK-023): tarea en curso
````

## Diff de tu dominio (9 fichero(s) de 11; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/carril.rs b/crates/tc2/src/carril.rs
new file mode 100644
index 0000000..837e361
--- /dev/null
+++ b/crates/tc2/src/carril.rs
@@ -0,0 +1,131 @@
+//! Carriles de la politica (ADR F1-10, decision 1): el sitio unico de la
+//! tabla de suelo. F2-7 solo necesita las columnas que valida la
+//! configuracion (roles de diseno minimos y perfil); F4 y F6 amplian este
+//! modulo en vez de copiar la tabla.
+
+use serde::Deserialize;
+
+/// Carril de una tarea, de menor a mayor exigencia (F1-10, decision 1).
+/// `release` y `hotfix` son tipos, no carriles.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
+#[serde(rename_all = "kebab-case")]
+pub enum Carril {
+    /// Solo documentacion.
+    Errata,
+    /// Correccion.
+    Fix,
+    /// Funcionalidad nueva.
+    Funcionalidad,
+    /// Toca `.github/`, `.taskcode/`, `CODEOWNERS` o rutas declaradas.
+    RiesgoAlto,
+}
+
+impl Carril {
+    /// Los cuatro carriles, en orden.
+    pub const TODOS: [Self; 4] = [
+        Self::Errata,
+        Self::Fix,
+        Self::Funcionalidad,
+        Self::RiesgoAlto,
+    ];
+
+    /// Nombre en la configuracion y en la linea.
+    pub fn nombre(self) -> &'static str {
+        match self {
+            Self::Errata => "errata",
+            Self::Fix => "fix",
+            Self::Funcionalidad => "funcionalidad",
+            Self::RiesgoAlto => "riesgo-alto",
+        }
+    }
+
+    /// Carril por su nombre.
+    pub fn desde(nombre: &str) -> Option<Self> {
+        Self::TODOS.into_iter().find(|c| c.nombre() == nombre)
+    }
+
+    /// Roles de diseno minimos del suelo (columna «Roles de diseno»; «todos»
+    /// en riesgo alto son los cuatro roles base, como en la PoC de F1-10).
+    pub fn roles_min(self) -> u64 {
+        match self {
+            Self::Errata => 0,
+            Self::Fix | Self::Funcionalidad => 1,
+            Self::RiesgoAlto => 4,
+        }
+    }
+
+    /// Perfil de `verificacion` del suelo (columna «Perfil»).
+    pub fn perfil(self) -> Perfil {
+        match self {
+            Self::Errata | Self::Fix | Self::Funcionalidad | Self::RiesgoAlto => Perfil::Completa,
+        }
+    }
+}
+
+/// Perfil de verificacion exigido por la politica (F1-10, columna «Perfil»;
+/// F1-7), de menor a mayor.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
+pub enum Perfil {
+    /// Atajo local, no es puerta.
+    #[serde(rename = "rapida")]
+    Rapida,
+    /// Tests afectados.
+    #[serde(rename = "impacto")]
+    Impacto,
+    /// La suite completa.
+    #[serde(rename = "completa")]
+    Completa,
+    /// La completa y los tests pesados.
+    #[serde(rename = "completa y pesada")]
+    CompletaYPesada,
+}
+
+/// Nombres de los perfiles, en orden.
+pub(crate) const NOMBRES_PERFIL: [&str; 4] = ["rapida", "impacto", "completa", "completa y pesada"];
+
+impl Perfil {
+    /// Los cuatro perfiles, en orden.
+    pub const TODOS: [Self; 4] = [
+        Self::Rapida,
+        Self::Impacto,
+        Self::Completa,
+        Self::CompletaYPesada,
+    ];
+
+    /// Nombre en la configuracion.
+    pub fn nombre(self) -> &'static str {
+        match self {
+            Self::Rapida => NOMBRES_PERFIL[0],
+            Self::Impacto => NOMBRES_PERFIL[1],
+            Self::Completa => NOMBRES_PERFIL[2],
+            Self::CompletaYPesada => NOMBRES_PERFIL[3],
+        }
+    }
+
+    /// Los perfiles que no bajan de este (este incluido), por nombre.
+    pub(crate) fn y_mayores(self) -> &'static [&'static str] {
+        let i = Self::TODOS.iter().position(|p| *p == self).unwrap_or(0);
+        NOMBRES_PERFIL.get(i..).unwrap_or(&NOMBRES_PERFIL)
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn nombres_y_orden() {
+        for c in Carril::TODOS {
+            assert_eq!(Carril::desde(c.nombre()), Some(c));
+        }
+        assert_eq!(Carril::desde("release"), None);
+        assert!(Carril::Errata < Carril::RiesgoAlto);
+        assert_eq!(
+            Perfil::Completa.y_mayores(),
+            ["completa", "completa y pesada"]
+        );
+        for p in Perfil::TODOS {
+            assert_eq!(p.y_mayores().first(), Some(&p.nombre()));
+        }
+    }
+}
diff --git a/crates/tc2/src/config/claves.rs b/crates/tc2/src/config/claves.rs
new file mode 100644
index 0000000..1cd2236
--- /dev/null
+++ b/crates/tc2/src/config/claves.rs
@@ -0,0 +1,655 @@
+//! Tabla declarativa de las claves de `.taskcode/config.yml` (F2-7): la
+//! fuente unica del validador, de los valores por defecto y del esquema JSON
+//! publicado. Cada clave de primer nivel cita el ADR que la pide.
+
+use serde_json::{Map, Value, json};
+
+use crate::carril::Carril;
+
+/// Tope de claves de primer nivel (plan, F2-7; F1-11, decision 1).
+pub const TOPE_CLAVES: usize = 12;
+
+/// Fases de `tc2` en las que puede correr un derivado (F1-11 se lo pide a
+/// F2-7): las ordenes que escriben en la rama de estado.
+pub const FASES: [&str; 8] = [
+    "new",
+    "plan",
+    "approve",
+    "start",
+    "review",
+    "finish",
+    "descartar",
+    "pausa",
+];
+
+/// Modelos de la escala de F1-15.
+pub const MODELOS: [&str; 3] = ["haiku", "sonnet", "opus"];
+
+/// Dominios cerrados de F1-15 (decision 3).
+pub const DOMINIOS: [&str; 2] = ["puertas-seguridad", "rendimiento"];
+
+/// Claves planas de la configuracion 0.16 que ya no existen en la 2.0: su
+/// error nombra `tc2 actualizar` (F1-12). `modo_flujo` y `tope_rondas`
+/// siguen siendo claves de la 2.0.
+pub const CLAVES_016: [&str; 30] = [
+    "rama_base",
+    "agente_revisor_por_defecto",
+    "limite_wip",
+    "comando_sincronizacion",
+    "rutas_sincronizacion",
+    "timeout_sincronizacion",
+    "fases_sincronizacion",
+    "excluir_de_revision",
+    "contexto_revision",
+    "cierre_por_defecto",
+    "plataforma_remota",
+    "url_base_remoto",
+    "comando_verificacion",
+    "timeout_verificacion",
+    "comando_verificacion_selectiva",
+    "timeout_verificacion_selectiva",
+    "comando_heldout",
+    "informe_cobertura",
+    "umbral_cobertura",
+    "rutas_documentacion",
+    "comando_verificacion_documentacion",
+    "rutas_verificacion_parcial",
+    "documento_estado",
+    "comando_verificacion_freno",
+    "presupuesto_freno_s",
+    "verificacion_por_lote",
+    "comando_verificacion_pesada",
+    "verificacion_pesada_en",
+    "umbral_crecimiento_suite",
+    "tareas_entre_podas",
+];
+
+/// Regla de un texto, ademas de su lista de valores.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub(crate) enum Regla {
+    /// Cualquier texto de la lista de valores.
+    Valor,
+    /// Nombre o etiqueta: no vacio, sin caracteres de control ni espacios en
+    /// los extremos.
+    Nombre,
+    /// Orden de shell: no vacia y sin caracteres de control.
+    Orden,
+    /// Glob relativo: sin negacion, rutas absolutas ni `..`.
+    Glob,
+    /// Glob de `confianza_incluye`: como [`Regla::Glob`] y nunca todo el
+    /// arbol.
+    GlobAcotado,
+    /// Ruta de `derivados[].escribe`: normalizada, bajo `tareas/`.
+    RutaTareas,
+    /// Expresion cron de 5 campos.
+    Cron,
+    /// URL de la API de la plataforma.
+    Url,
+}
+
+impl Regla {
+    /// Lo que exige, para el esquema y los mensajes.
+    pub(crate) fn texto(self) -> Option<&'static str> {
+        match self {
+            Self::Valor => None,
+            Self::Nombre => {
+                Some("texto no vacio, sin caracteres de control ni espacios en los extremos")
+            }
+            Self::Orden => Some(
+                "orden no vacia y sin caracteres de control (rutas de Windows entre comillas simples o con /)",
+            ),
+            Self::Glob => Some("glob relativo: sin negacion (!), rutas absolutas ni .."),
+            Self::GlobAcotado => Some(
+                "glob relativo: sin negacion (!), rutas absolutas ni .., y nunca todo el arbol (**, *)",
+            ),
+            Self::RutaTareas => Some(
+                "ruta normalizada bajo tareas/, sin \\, :, comodines, caracteres de control, nombres reservados de Windows ni .git; nunca tareas/ entera, tareas/ROADMAP.md ni tareas/TASK-NNN/ (sin distinguir mayusculas)",
+            ),
+            Self::Cron => Some("expresion cron de 5 campos"),
+            Self::Url => Some("URL https de la API (o http://127.0.0.1:<puerto> en pruebas)"),
+        }
+    }
+}
+
+/// Forma de un valor.
+#[derive(Debug, Clone)]
+pub(crate) enum Nodo {
+    /// Entero en `[min, max]` (`max` `None`: sin tope).
+    Entero { min: u64, max: Option<u64> },
+    /// Booleano; con `solo`, el unico valor admitido.
+    Booleano { solo: Option<bool> },
+    /// Texto; si `valores` no esta vacio, uno de ellos.
+    Texto {
+        valores: &'static [&'static str],
+        regla: Regla,
+    },
+    /// Lista con al menos `min` elementos; con `primero`, el primer elemento
+    /// es ese texto.
+    Lista {
+        de: Box<Nodo>,
+        min: usize,
+        primero: Option<&'static str>,
+    },
+    /// Mapa cerrado: solo estas claves.
+    Objeto(Vec<Campo>),
+    /// Una de varias formas.
+    UnaDe(Vec<Nodo>),
+}
+
+/// Una clave de un mapa cerrado.
+#[derive(Debug, Clone)]
+pub(crate) struct Campo {
+    pub nombre: &'static str,
+    pub nodo: Nodo,
+    pub obligatorio: bool,
+    pub defecto: Option<Value>,
+    /// ADR de origen (obligatorio en las de primer nivel).
+    pub origen: &'static str,
+    pub descripcion: &'static str,
+}
+
+impl Campo {
+    fn opcional(nombre: &'static str, nodo: Nodo, descripcion: &'static str) -> Self {
+        Self {
+            nombre,
+            nodo,
+            obligatorio: false,
+            defecto: None,
+            origen: "",
+            descripcion,
+        }
+    }
+
+    fn obligatorio(nombre: &'static str, nodo: Nodo, descripcion: &'static str) -> Self {
+        Self {
+            obligatorio: true,
+            ..Self::opcional(nombre, nodo, descripcion)
+        }
+    }
+
+    fn con_defecto(mut self, v: Value) -> Self {
+        self.defecto = Some(v);
+        self
+    }
+
+    fn de(mut self, origen: &'static str) -> Self {
+        self.origen = origen;
+        self
+    }
+}
+
+fn texto(regla: Regla) -> Nodo {
+    Nodo::Texto {
+        valores: &[],
+        regla,
+    }
+}
+
+fn uno_de(valores: &'static [&'static str]) -> Nodo {
+    Nodo::Texto {
+        valores,
+        regla: Regla::Valor,
+    }
+}
+
+fn lista(de: Nodo, min: usize) -> Nodo {
+    Nodo::Lista {
+        de: Box::new(de),
+        min,
+        primero: None,
+    }
+}
+
+fn entero(min: u64, max: u64) -> Nodo {
+    Nodo::Entero {
+        min,
+        max: Some(max),
+    }
+}
+
+/// Runner de `completa`, `pesada` y la verificacion programada (F1-7,
+/// decision 5): `{github: windows|linux}` o `[self-hosted, ...]`.
+fn runner() -> Nodo {
+    Nodo::UnaDe(vec![
+        Nodo::Objeto(vec![Campo::obligatorio(
+            "github",
+            uno_de(&["windows", "linux"]),
+            "runner alojado de GitHub",
+        )]),
+        Nodo::Lista {
+            de: Box::new(texto(Regla::Nombre)),
+            min: 1,
+            primero: Some("self-hosted"),
+        },
+    ])
+}
+
+fn perfil(nombre: &'static str, con_runner: bool, obligatorio: bool) -> Campo {
+    let mut campos = vec![Campo::obligatorio(
+        "timeout",
+        entero(1, 21_600),
+        "segundos; en rapida es un presupuesto que no bloquea, en las demas agotarlo es rojo",
+    )];
+    if con_runner {
+        campos.push(Campo::obligatorio(
+            "runner",
+            runner(),
+            "donde corre en la CI",
+        ));
+    }
+    let c = Campo::opcional(nombre, Nodo::Objeto(campos), "perfil de F1-7");
+    Campo { obligatorio, ..c }
+}
+
+fn verificacion() -> Nodo {
+    Nodo::Objeto(vec![
+        Campo::obligatorio(
+            "comando",
+            texto(Regla::Orden),
+            "comando del proyecto; se llama con --perfil <p>",
+        ),
+        Campo::obligatorio(
+            "shell",
+            uno_de(&["cmd", "bash", "pwsh", "sh"]),
+            "shell con la que se lanza en todas partes (H16)",
+        ),
+        Campo::obligatorio(
+            "perfiles",
+            Nodo::Objeto(vec![
+                perfil("rapida", false, false),
+                perfil("impacto", false, false),
+                perfil("completa", true, true),
+                perfil("pesada", true, false),
+            ]),
+            "perfiles de F1-7; completa obligatoria",
+        ),
+        Campo::opcional(
+            "huella_arbol",
+            lista(texto(Regla::Glob), 0),
+            "ficheros versionados de la clave de cache (F1-7 dec. 2)",
+        )
+        .con_defecto(json!([])),
+        Campo::opcional(
+            "huella_local",
+            texto(Regla::Orden),
+            "imprime lo que la suite lee fuera del arbol",
+        ),
+        Campo::opcional(
+            "programada",
+            Nodo::Objeto(vec![
+                Campo::obligatorio(
+                    "comando",
+                    texto(Regla::Orden),
+                    "pasos que dependen de la red",
+                ),
+                Campo::obligatorio("cron", texto(Regla::Cron), "cuando corre"),
+                Campo::obligatorio("runner", runner(), "donde corre"),
+            ]),
+            "job programado fuera del check cacheado",
+        ),
+    ])
+}
+
+fn derivado() -> Nodo {
+    Nodo::Objeto(vec![
+        Campo::obligatorio("comando", texto(Regla::Orden), "que ejecuta"),
+        Campo::obligatorio(
+            "tipo",
+            uno_de(&["genera", "comprueba"]),
+            "si escribe ficheros o solo comprueba",
+        ),
+        Campo::opcional(
+            "escribe",
+            lista(texto(Regla::RutaTareas), 1),
+            "lo unico que un genera puede escribir; obligatoria en genera, prohibida en comprueba",
+        ),
+        Campo::opcional(
+            "fases",
+            lista(uno_de(&FASES), 1),
+            "en que transiciones corre (por defecto, en todas)",
+        )
+        .con_defecto(json!(FASES)),
+        Campo::opcional("timeout", entero(1, 600), "segundos").con_defecto(json!(60)),
+    ])
+}
+
+fn politica_de(c: Carril) -> Campo {
+    Campo::opcional(
+        c.nombre(),
+        Nodo::Objeto(vec![
+            Campo::opcional(
+                "roles_diseno",
+                entero(c.roles_min(), 5),
+                "roles de diseno minimos; no baja del suelo",
+            ),
+            Campo::opcional(
+                "plan_aprobado",
+                Nodo::Booleano { solo: Some(true) },
+                "solo se puede exigir, no quitar",
+            ),
+            Campo::opcional(
+                "paradas_extra",
+                lista(uno_de(&["start", "review"]), 0),
+                "paradas del automatico que se anaden",
+            )
+            .con_defecto(json!([])),
+            Campo::opcional(
+                "revisoras_extra",
+                lista(texto(Regla::Nombre), 0),
+                "revisoras que se anaden",
+            )
+            .con_defecto(json!([])),
+            Campo::opcional(
+                "lote",
+                Nodo::Booleano { solo: Some(false) },
+                "solo se puede sacar del lote",
+            ),
+            Campo::opcional(
+                "perfil",
+                Nodo::Texto {
+                    valores: c.perfil().y_mayores(),
+                    regla: Regla::Valor,
+                },
+                "perfil de verificacion; no baja del suelo",
+            ),
+        ]),
+        "endurece la fila del carril",
+    )
+}
+
+fn senales() -> Nodo {
+    Nodo::Objeto(vec![
+        Campo::opcional(
+            "etiquetas",
+            lista(texto(Regla::Nombre), 0),
+            "etiquetas que se anaden",
+        )
+        .con_defecto(json!([])),
+        Campo::opcional(
+            "palabras",
+            lista(texto(Regla::Nombre), 0),
+            "palabras que se anaden",
+        )
+        .con_defecto(json!([])),
+        Campo::opcional("rutas", lista(texto(Regla::Glob), 0), "globs que se anaden")
+            .con_defecto(json!([])),
+    ])
+}
+
+fn opcional_entero(nombre: &'static str, descripcion: &'static str) -> Campo {
+    Campo::opcional(nombre, Nodo::Entero { min: 1, max: None }, descripcion)
+}
+
+/// Las claves de primer nivel, en orden.
+pub(crate) fn tabla() -> Vec<Campo> {
+    vec![
+        Campo::obligatorio(
+            "formato",
+            entero(2, 2),
+            "version del formato del proyecto; con 2 manda la 2.0",
+        )
+        .de("F1-12 D4a, decision 6"),
+        Campo::opcional(
+            "politica_carriles",
+            Nodo::Objeto(Carril::TODOS.into_iter().map(politica_de).collect()),
+            "endurece la politica por carril; solo los cuatro carriles",
+        )
+        .con_defecto(json!({}))
+        .de("F1-10 decision 2"),
+        Campo::opcional(
+            "rutas_carril",
+            Nodo::Objeto(
+                [Carril::Fix, Carril::RiesgoAlto]
+                    .into_iter()
+                    .map(|c| {
+                        Campo::opcional(
+                            c.nombre(),
+                            lista(texto(Regla::Glob), 1),
+                            "globs que suben a este carril",
+                        )
+                    })
+                    .collect(),
+            ),
+            "rutas que suben el carril minimo; solo fix y riesgo-alto",
+        )
+        .con_defecto(json!({}))
+        .de("F1-10 decision 2"),
+        Campo::opcional(
+            "verificacion",
+            verificacion(),
+            "contrato de verificacion de F1-7",
+        )
+        .de("F1-11 decision 1"),
+        Campo::opcional(
+            "derivados",
+            lista(derivado(), 0),
+            "comandos de las transiciones que el check estado repite",
+        )
+        .con_defecto(json!([]))
+        .de("F1-11 decision 1"),
+        Campo::opcional(
+            "confianza_incluye",
+            lista(texto(Regla::GlobAcotado), 0),
+            "ficheros que suma la huella de confianza",
+        )
+        .con_defecto(json!([]))
+        .de("F1-11 decision 1; F1-8 decision 5"),
+        Campo::opcional(
+            "plataforma",
+            Nodo::Objeto(vec![
+                Campo::obligatorio(
+                    "proveedor",
+                    uno_de(&["github", "gitlab"]),
+                    "plataforma del remoto",
+                ),
+                Campo::obligatorio("api", texto(Regla::Url), "URL de la API"),
+            ]),
+            "plataforma de un host que no es github.com ni gitlab.com",
+        )
+        .de("F1-13 decision 7"),
+        Campo::opcional(
+            "recursos",
+            lista(
+                Nodo::Objeto(vec![
+                    Campo::obligatorio(
+                        "id",
+                        texto(Regla::Nombre),
+                        "nombre de la skill, el agente o el ejecutable",
+                    ),
+                    Campo::obligatorio(
+                        "tipo",
+                        uno_de(&["skill", "agente", "ejecutable"]),
+                        "que es",
+                    ),
+                    Campo::opcional("rol", uno_de(&["revisor"]), "rol que cubre"),
+                    Campo::opcional(
+                        "dominios",
+                        lista(uno_de(&DOMINIOS), 1),
+                        "dominios de F1-15 que cubre",
+                    )
+                    .con_defecto(json!([])),
+                ]),
+                0,
+            ),
+            "recursos necesarios que el proyecto anade; solo anade",
+        )
+        .con_defecto(json!([]))
+        .de("F1-14 D6a; F1-15 decision 6"),
+        Campo::opcional(
+            "seleccion",
+            Nodo::Objeto(vec![
+                Campo::opcional(
+                    "modelos_permitidos",
+                    lista(uno_de(&MODELOS), 1),
+                    "solo restringe la escala",
+                )
+                .con_defecto(json!(MODELOS)),
+                Campo::opcional(
+                    "dominios_extra",
+                    Nodo::Objeto(
+                        DOMINIOS
+                            .into_iter()
+                            .map(|d| Campo::opcional(d, senales(), "senales que se anaden"))
+                            .collect(),
+                    ),
+                    "solo anade senales a un dominio de la lista",
+                )
+                .con_defecto(json!({})),
+            ]),
+            "seleccion de skills, agentes y modelo",
+        )
+        .con_defecto(json!({"modelos_permitidos": MODELOS, "dominios_extra": {}}))
+        .de("F1-15 decision 3 y S15"),
+        Campo::opcional(
+            "modo_flujo",
+            uno_de(&["manual", "semiautomatico", "automatico"]),
+            "modo del flujo; lo lee tambien la 0.16",
+        )
+        .con_defecto(json!("manual"))
+        .de("F1-12 decision 1 (convivencia); RF-21"),
+        Campo::opcional(
+            "tope_rondas",
+            entero(1, 10),
+            "rondas de revision sin preguntar",
+        )
+        .con_defecto(json!(2))
+        .de("F1-11 PoC B; F1-16 D6a"),
+        Campo::opcional(
+            "lote",
+            Nodo::Objeto(vec![
+                opcional_entero("presupuesto_tokens", "presupuesto del lote en tokens"),
+                opcional_entero("presupuesto_horas", "presupuesto del lote en horas"),
+                opcional_entero("tope_tokens_tarea", "tope de tokens por tarea"),
+            ]),
+            "presupuesto por defecto de los lotes; sin cifras inventadas",
+        )
+        .de("F1-16, consecuencias"),
+    ]
+}
+
+/// JSON Schema 2020-12 de la configuracion, generado de [`tabla`].
+pub fn esquema_json() -> String {
+    let raiz = Nodo::Objeto(tabla());
+    let mut v = a_esquema(&raiz);
+    if let Value::Object(m) = &mut v {
+        m.insert(
+            "$schema".into(),
+            json!("https://json-schema.org/draft/2020-12/schema"),
+        );
+        m.insert(
+            "title".into(),
+            json!("Configuracion de TaskCode 2 (.taskcode/config.yml)"),
+        );
+        m.insert(
+            "description".into(),
+            json!("Generado por tc2 (config::esquema_json). La primera linea del fichero es ---. Una clave o un valor desconocido falla cerrado."),
+        );
+    }
+    let mut s = serde_json::to_string_pretty(&v).unwrap_or_default();
+    s.push('\n');
+    s
+}
+
+fn a_esquema(n: &Nodo) -> Value {
+    match n {
+        Nodo::Entero { min, max } => {
+            let mut m = Map::new();
+            m.insert("type".into(), json!("integer"));
+            m.insert("minimum".into(), json!(min));
+            if let Some(x) = max {
+                m.insert("maximum".into(), json!(x));
+            }
+            Value::Object(m)
+        }
+        Nodo::Booleano { solo: Some(b) } => json!({"const": b}),
+        Nodo::Booleano { solo: None } => json!({"type": "boolean"}),
+        Nodo::Texto { valores, regla } => {
+            let mut m = Map::new();
+            m.insert("type".into(), json!("string"));
+            if valores.is_empty() {
+                m.insert("minLength".into(), json!(1));
+            } else {
+                m.insert("enum".into(), json!(valores));
+            }
+            if let Some(t) = regla.texto() {
+                m.insert("description".into(), json!(t));
+            }
+            Value::Object(m)
+        }
+        Nodo::Lista { de, min, primero } => {
+            let mut m = Map::new();
+            m.insert("type".into(), json!("array"));
+            m.insert("items".into(), a_esquema(de));
+            if *min > 0 {
+                m.insert("minItems".into(), json!(min));
+            }
+            if let Some(p) = primero {
+                m.insert("prefixItems".into(), json!([{"const": p}]));
+            }
+            Value::Object(m)
+        }
+        Nodo::Objeto(campos) => {
+            let mut props = Map::new();
+            let mut requeridas = Vec::new();
+            for c in campos {
+                let mut e = a_esquema(&c.nodo);
+                if let Value::Object(m) = &mut e {
+                    let d = if c.origen.is_empty() {
+                        c.descripcion.to_owned()
+                    } else {
+                        format!("{} [{}]", c.descripcion, c.origen)
+                    };
+                    let d = match m.get("description").and_then(Value::as_str) {
+                        Some(regla) => format!("{d}. {regla}"),
+                        None => d,
+                    };
+                    m.insert("description".into(), json!(d));
+                    if let Some(x) = &c.defecto {
+                        m.insert("default".into(), x.clone());
+                    }
+                }
+                props.insert(c.nombre.into(), e);
+                if c.obligatorio {
+                    requeridas.push(c.nombre);
+                }
+            }
+            let mut m = Map::new();
+            m.insert("type".into(), json!("object"));
+            m.insert("additionalProperties".into(), json!(false));
+            m.insert("properties".into(), Value::Object(props));
+            if !requeridas.is_empty() {
+                m.insert("required".into(), json!(requeridas));
+            }
+            Value::Object(m)
+        }
+        Nodo::UnaDe(formas) => json!({"anyOf": formas.iter().map(a_esquema).collect::<Vec<_>>()}),
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn claves_de_primer_nivel_no_pasan_de_12_y_cada_una_cita_su_adr() {
+        let t = tabla();
+        assert!(t.len() <= TOPE_CLAVES, "{} claves", t.len());
+        for c in &t {
+            assert!(!c.origen.is_empty(), "{} sin origen", c.nombre);
+        }
+        let mut nombres: Vec<_> = t.iter().map(|c| c.nombre).collect();
+        nombres.sort_unstable();
+        nombres.dedup();
+        assert_eq!(nombres.len(), t.len(), "claves repetidas");
+        assert_eq!(
+            t.iter()
+                .filter(|c| c.obligatorio)
+                .map(|c| c.nombre)
+                .collect::<Vec<_>>(),
+            ["formato"]
+        );
+        for c in CLAVES_016 {
+            assert!(t.iter().all(|x| x.nombre != c), "{c} es de la 2.0");
+        }
+    }
+}
diff --git a/crates/tc2/src/config/mod.rs b/crates/tc2/src/config/mod.rs
new file mode 100644
index 0000000..e2a78ab
--- /dev/null
+++ b/crates/tc2/src/config/mod.rs
@@ -0,0 +1,521 @@
+//! Configuracion del proyecto, `.taskcode/config.yml` (F2-7).
+//!
+//! - [`leer`]: funcion pura. Exige la primera linea `---` y `formato: 2`
+//!   (F1-12), lee el YAML con las opciones estrictas de [`crate::yaml`] y lo
+//!   valida contra la tabla de [`claves`]. Devuelve la [`Config`] con los
+//!   valores por defecto ya aplicados o **todos** los errores con la ruta de
+//!   cada clave. Una clave o un valor desconocido falla cerrado (F1-10
+//!   decision 2, F1-11 decision 1).
+//! - [`de_base`]: la lee de `origin/develop`, nunca del worktree de la tarea
+//!   (F1-10 decision 3). No hace `fetch`: lo hace quien la llama.
+//! - [`esquema_json`]: el esquema publicado en
+//!   `crates/tc2/esquemas/config.schema.json`, generado de la misma tabla.
+//!
+//! Un fallo sale como `config rechazada motivo=<m> errores=<n> claves=...`
+//! (codigo 3, F1-4): no se reintenta.
+
+mod claves;
+mod validar;
+
+use std::collections::BTreeMap;
+use std::path::Path;
+
+use serde::{Deserialize, Deserializer};
+use serde_json::Value;
+
+pub use claves::{CLAVES_016, DOMINIOS, FASES, MODELOS, TOPE_CLAVES, esquema_json};
+
+use crate::carril::{Carril, Perfil};
+use crate::contrato::{Informe, Resultado};
+use crate::git::{self, Git, Paso, Subcomando};
+use crate::plataforma::Proveedor;
+
+/// Nombre de la operacion en la linea de contrato.
+pub const NOMBRE: &str = "config";
+
+/// Ruta de la configuracion en el repo.
+pub const RUTA: &str = ".taskcode/config.yml";
+
+/// Rama de la que se lee (F1-10 decision 3).
+pub const BASE: &str = "origin/develop";
+
+/// Configuracion validada, con los valores por defecto aplicados.
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Config {
+    /// Siempre 2.
+    pub formato: u8,
+    /// Endurecimiento por carril (F1-10 decision 2).
+    pub politica_carriles: BTreeMap<Carril, PoliticaCarril>,
+    /// Globs que suben el carril minimo (solo `fix` y `riesgo-alto`).
+    pub rutas_carril: BTreeMap<Carril, Vec<String>>,
+    /// Contrato de verificacion (F1-11, F1-7); `None` si no se declara.
+    pub verificacion: Option<Verificacion>,
+    /// Derivados de las transiciones (F1-11).
+    pub derivados: Vec<Derivado>,
+    /// Ficheros que suma la huella de confianza (F1-8 decision 5).
+    pub confianza_incluye: Vec<String>,
+    /// Plataforma de un host propio (F1-13 decision 7).
+    pub plataforma: Option<PlataformaConfig>,
+    /// Recursos necesarios que anade el proyecto (F1-14 D6a).
+    pub recursos: Vec<Recurso>,
+    /// Seleccion de skills, agentes y modelo (F1-15).
+    pub seleccion: Seleccion,
+    /// Modo del flujo.
+    pub modo_flujo: ModoFlujo,
+    /// Rondas de revision sin preguntar.
+    pub tope_rondas: u8,
+    /// Presupuesto por defecto de los lotes (F1-16); `None` si no se declara.
+    pub lote: Option<Lote>,
+}
+
+/// `politica_carriles.<carril>`: lo que no se declara es el suelo.
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct PoliticaCarril {
+    /// Roles de diseno minimos.
+    pub roles_diseno: Option<u8>,
+    /// Solo `Some(true)`.
+    pub plan_aprobado: Option<bool>,
+    /// Paradas que se anaden (`start`, `review`).
+    pub paradas_extra: Vec<String>,
+    /// Revisoras que se anaden.
+    pub revisoras_extra: Vec<String>,
+    /// Solo `Some(false)`.
+    pub lote: Option<bool>,
+    /// Perfil, nunca por debajo del suelo.
+    pub perfil: Option<Perfil>,
+}
+
+/// `verificacion` (F1-11, tabla `esquema`).
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Verificacion {
+    /// Comando del proyecto.
+    pub comando: String,
+    /// `cmd`, `bash`, `pwsh` o `sh`.
+    pub shell: String,
+    /// Perfiles.
+    pub perfiles: Perfiles,
+    /// Globs de la clave de cache.
+    pub huella_arbol: Vec<String>,
+    /// Orden que imprime lo que la suite lee fuera del arbol.
+    pub huella_local: Option<String>,
+    /// Verificacion programada.
+    pub programada: Option<Programada>,
+}
+
+/// `verificacion.perfiles`.
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Perfiles {
+    /// Atajo local.
+    pub rapida: Option<PerfilVerificacion>,
+    /// Tests afectados.
+    pub impacto: Option<PerfilVerificacion>,
+    /// Suite completa (obligatoria).
+    pub completa: PerfilVerificacion,
+    /// Tests pesados.
+    pub pesada: Option<PerfilVerificacion>,
+}
+
+/// Un perfil de `verificacion`.
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct PerfilVerificacion {
+    /// Segundos.
+    pub timeout: u32,
+    /// Solo en `completa` y `pesada`.
+    pub runner: Option<Runner>,
+}
+
+/// Runner de la CI (F1-7 decision 5).
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
+#[serde(untagged)]
+pub enum Runner {
+    /// `{github: windows|linux}`.
+    GitHub {
+        /// `windows` o `linux`.
+        github: String,
+    },
+    /// `[self-hosted, ...]`.
+    Etiquetas(Vec<String>),
+}
+
+/// `verificacion.programada`.
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Programada {
+    /// Pasos que dependen de la red.
+    pub comando: String,
+    /// Cron de 5 campos.
+    pub cron: String,
+    /// Donde corre.
+    pub runner: Runner,
+}
+
+/// Un derivado (F1-11).
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Derivado {
+    /// Que ejecuta.
+    pub comando: String,
+    /// `genera` o `comprueba`.
+    pub tipo: TipoDerivado,
+    /// Rutas que puede escribir, normalizadas (vacia en `comprueba`).
+    #[serde(default)]
+    pub escribe: Vec<String>,
+    /// Fases en las que corre.
+    pub fases: Vec<String>,
+    /// Segundos.
+    pub timeout: u16,
+}
+
+/// Tipo de un derivado.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
+#[serde(rename_all = "lowercase")]
+pub enum TipoDerivado {
+    /// Escribe ficheros.
+    Genera,
+    /// Solo comprueba.
+    Comprueba,
+}
+
+/// `plataforma` (F1-13 decision 7).
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct PlataformaConfig {
+    /// GitHub o GitLab.
+    #[serde(deserialize_with = "de_proveedor")]
+    pub proveedor: Proveedor,
+    /// URL de la API.
+    pub api: String,
+}
+
+fn de_proveedor<'de, D: Deserializer<'de>>(d: D) -> Result<Proveedor, D::Error> {
+    let s = String::deserialize(d)?;
+    match s.as_str() {
+        "github" => Ok(Proveedor::GitHub),
+        "gitlab" => Ok(Proveedor::GitLab),
+        otro => Err(serde::de::Error::custom(format!(
+            "proveedor desconocido {otro}"
+        ))),
+    }
+}
+
+/// Un recurso declarado (F1-14 D6a).
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Recurso {
+    /// Nombre.
+    pub id: String,
+    /// `skill`, `agente` o `ejecutable`.
+    pub tipo: String,
+    /// `revisor`, si lo es.
+    pub rol: Option<String>,
+    /// Dominios de F1-15 que cubre.
+    pub dominios: Vec<String>,
+}
+
+/// `seleccion` (F1-15).
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Seleccion {
+    /// Subconjunto de la escala.
+    pub modelos_permitidos: Vec<String>,
+    /// Senales que se anaden por dominio.
+    pub dominios_extra: BTreeMap<String, Senales>,
+}
+
+/// Senales de un dominio (F1-15 decision 3).
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Senales {
+    /// Etiquetas.
+    pub etiquetas: Vec<String>,
+    /// Palabras.
+    pub palabras: Vec<String>,
+    /// Globs.
+    pub rutas: Vec<String>,
+}
+
+/// Modo del flujo.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
+#[serde(rename_all = "lowercase")]
+pub enum ModoFlujo {
+    /// Pregunta en cada fase.
+    Manual,
+    /// Pregunta en algunas.
+    Semiautomatico,
+    /// Solo las paradas fijas.
+    Automatico,
+}
+
+/// `lote` (F1-16): sin cifras por defecto.
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
+#[serde(deny_unknown_fields)]
+pub struct Lote {
+    /// Tokens del lote.
+    pub presupuesto_tokens: Option<u64>,
+    /// Horas del lote.
+    pub presupuesto_horas: Option<u64>,
+    /// Tokens por tarea.
+    pub tope_tokens_tarea: Option<u64>,
+}
+
+impl Config {
+    /// La configuracion minima (`formato: 2` y nada mas): la politica sin
+    /// endurecer. F6 la usa para evaluar el PR que arregla una configuracion
+    /// invalida en la base (F1-10, caso abierto; ver [`Invalida`]).
+    pub fn suelo() -> Self {
+        Self {
+            formato: 2,
+            politica_carriles: BTreeMap::new(),
+            rutas_carril: BTreeMap::new(),
+            verificacion: None,
+            derivados: Vec::new(),
+            confianza_incluye: Vec::new(),
+            plataforma: None,
+            recursos: Vec::new(),
+            seleccion: Seleccion {
+                modelos_permitidos: MODELOS.iter().map(|m| (*m).to_owned()).collect(),
+                dominios_extra: BTreeMap::new(),
+            },
+            modo_flujo: ModoFlujo::Manual,
+            tope_rondas: 2,
+            lote: None,
+        }
+    }
+}
+
+/// Por que se rechaza la configuracion.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Motivo {
+    /// No hay `.taskcode/config.yml` en la base.
+    SinConfig,
+    /// Falta `---` o `formato: 2` (un proyecto de la 0.16).
+    Formato,
+    /// YAML mal formado, clave duplicada, ancla o alias.
+    Yaml,
+    /// No cumple el esquema.
+    ConfigInvalida,
+}
+
+impl Motivo {
+    /// Nombre en la linea.
+    pub fn nombre(self) -> &'static str {
+        match self {
+            Self::SinConfig => "sin-config",
+            Self::Formato => "formato",
+            Self::Yaml => "yaml",
+            Self::ConfigInvalida => "config-invalida",
+        }
+    }
+}
+
+/// Un error con la ruta de su clave (`politica_carriles.fix.aprobaciones`).
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct ErrorConfig {
+    /// Ruta de la clave; vacia si el error es del documento entero.
+    pub ruta: String,
+    /// Que esta mal.
+    pub mensaje: String,
+}
+
+impl ErrorConfig {
+    pub(crate) fn nuevo(ruta: &str, mensaje: impl Into<String>) -> Self {
+        Self {
+            ruta: ruta.to_owned(),
+            mensaje: mensaje.into(),
+        }
+    }
+}
+
+/// Configuracion rechazada, con todos sus errores.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Invalida {
+    /// Motivo.
+    pub motivo: Motivo,
+    /// Errores, en el orden en que se encontraron (al menos uno).
+    pub errores: Vec<ErrorConfig>,
+}
+
+impl Invalida {
+    fn una(motivo: Motivo, ruta: &str, mensaje: impl Into<String>) -> Self {
+        Self {
+            motivo,
+            errores: vec![ErrorConfig::nuevo(ruta, mensaje)],
+        }
+    }
+
+    /// Informe `config rechazada` (codigo 3) con el primer error y su arreglo
+    /// en el diagnostico.
+    pub fn informe(&self) -> Informe {
+        let mut rutas: Vec<String> = Vec::new();
+        for e in &self.errores {
+            if !e.ruta.is_empty() && !rutas.contains(&e.ruta) {
+                rutas.push(e.ruta.clone());
+            }
+        }
+        let mut i = Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", self.motivo.nombre())
+            .con("errores", self.errores.len().to_string());
+        if !rutas.is_empty() {
+            i = i.con_lista("claves", rutas);
+        }
+        let primero = self
+            .errores
+            .first()
+            .map(|e| {
+                if e.ruta.is_empty() {
+                    e.mensaje.clone()
+                } else {
+                    format!("{}: {}", e.ruta, e.mensaje)
+                }
+            })
+            .unwrap_or_default();
+        let arreglo = match self.motivo {
+            Motivo::SinConfig => "crea la configuracion con tc2 init".to_owned(),
+            Motivo::Formato => "es un proyecto de la 0.16: migralo con tc2 actualizar".to_owned(),
+            Motivo::Yaml => format!("corrige el YAML de {RUTA}"),
+            Motivo::ConfigInvalida
+                if self
+                    .errores
+                    .iter()
+                    .any(|e| e.mensaje.contains("tc2 actualizar")) =>
+            {
+                "quedan claves de la 0.16: tc2 actualizar las traduce".to_owned()
+            }
+            Motivo::ConfigInvalida => {
+                format!("corrige {RUTA} (esquema: crates/tc2/esquemas/config.schema.json)")
+            }
+        };
+        let extra = match self.errores.len() {
+            0 | 1 => String::new(),
+            n => format!(" (y {} error(es) mas)", n - 1),
+        };
+        i.diagnostico(format!("{RUTA}: {primero}{extra}. Arreglo: {arreglo}"))
+    }
+}
+
+/// Lee y valida el texto de `.taskcode/config.yml`. Sin E/S.
+pub fn leer(texto: &str) -> Result<Config, Invalida> {
+    let primera = texto.split('\n').next().unwrap_or("");
+    if primera.strip_suffix('\r').unwrap_or(primera) != "---" {
+        let que = if primera.starts_with('\u{feff}') {
+            "la primera linea tiene que ser --- (sin BOM)"
+        } else {
+            "la primera linea tiene que ser ---"
+        };
+        return Err(Invalida::una(Motivo::Formato, "", que));
+    }
+    let raiz: Value = serde_saphyr::from_str_with_options(texto, crate::yaml::opciones())
+        .map_err(|e| Invalida::una(Motivo::Yaml, "", format!("YAML no valido: {e}")))?;
+    let formato = match &raiz {
+        Value::Object(m) => m.get("formato"),
+        Value::Null => None,
+        _ => {
+            return Err(Invalida::una(
+                Motivo::ConfigInvalida,
+                "",
+                "la raiz tiene que ser un mapa",
+            ));
+        }
+    };
+    match formato {
+        Some(f) if f.as_u64() == Some(2) => {}
+        Some(f) => {
+            return Err(Invalida::una(
+                Motivo::Formato,
+                "formato",
+                format!("se esperaba 2, no {f}"),
+            ));
+        }
+        None => {
+            return Err(Invalida::una(
+                Motivo::Formato,
+                "formato",
+                "falta la clave obligatoria",
+            ));
+        }
+    }
+    let tabla = claves::Nodo::Objeto(claves::tabla());
+    let mut errores = Vec::new();
+    validar::validar(&tabla, &raiz, "", &mut errores);
+    validar::cruzadas(&raiz, &mut errores);
+    if !errores.is_empty() {
+        return Err(Invalida {
+            motivo: Motivo::ConfigInvalida,
+            errores,
+        });
+    }
+    let mut v = raiz;
+    validar::aplicar_defectos(&tabla, &mut v);
+    normalizar_escribe(&mut v);
+    serde_json::from_value(v).map_err(|e| {
+        Invalida::una(
+            Motivo::ConfigInvalida,
+            "",
+            format!("error interno al construir la configuracion validada: {e}"),
+        )
+    })
+}
+
+/// Las rutas de `escribe` se guardan normalizadas (F1-11: se comparan ya
+/// normalizadas).
+fn normalizar_escribe(v: &mut Value) {
+    let Some(ds) = v.get_mut("derivados").and_then(Value::as_array_mut) else {
+        return;
+    };
+    for d in ds {
+        if let Some(es) = d.get_mut("escribe").and_then(Value::as_array_mut) {
+            for e in es {
+                if let Some(n) = e.as_str().and_then(|t| validar::ruta_tareas(t).ok()) {
+                    *e = Value::String(n);
+                }
+            }
+        }
+    }
+}
+
+/// Lee la configuracion de `origin/develop` (F1-10 decision 3), nunca del
+/// worktree. `Err` lleva el informe: `config rechazada` si falta o no
+/// vale, o el de la capa git si no se pudo leer.
+pub fn de_base(git: &Git, dir: &Path) -> Result<Config, Informe> {
+    let texto = texto_de_base(git, dir)?;
+    leer(&texto).map_err(|e| e.informe())
+}
+
+fn texto_de_base(git: &Git, dir: &Path) -> Paso<String> {
+    if git::commit_de(git, dir, NOMBRE, BASE)?.is_none() {
+        return Err(Informe::nuevo(NOMBRE, Resultado::Rechazada)
+            .con("motivo", "sin-base")
+            .diagnostico(format!(
+                "no existe {BASE}: haz fetch o crea la rama develop en el remoto"
+            )));
+    }
+    let listado = git::exigir(
+        git,
+        dir,
+        NOMBRE,
+        Subcomando::LsTree,
+        &["--name-only", "--end-of-options", BASE, "--", RUTA],
+    )?;
+    if listado.trim() != RUTA {
+        return Err(
+            Invalida::una(Motivo::SinConfig, "", format!("no hay {RUTA} en {BASE}")).informe(),
+        );
+    }
+    git::estado::leer_blob(git, dir, NOMBRE, BASE, RUTA)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn config_minima_da_los_valores_por_defecto() {
+        assert_eq!(leer("---\nformato: 2\n"), Ok(Config::suelo()));
+        assert_eq!(leer("---\r\nformato: 2\r\n"), Ok(Config::suelo()));
+    }
+}
diff --git a/crates/tc2/src/config/validar.rs b/crates/tc2/src/config/validar.rs
new file mode 100644
index 0000000..5045663
--- /dev/null
+++ b/crates/tc2/src/config/validar.rs
@@ -0,0 +1,391 @@
+//! Recorrido del arbol YAML contra la tabla de [`super::claves`]: junta
+//! todos los errores con la ruta de cada clave y, si no hay ninguno, aplica
+//! los valores por defecto de la tabla.
+
+use serde_json::Value;
+
+use super::ErrorConfig;
+use super::claves::{CLAVES_016, Campo, Nodo, Regla};
+use crate::plataforma::http::Base;
+
+/// Errores de `valor` frente a `nodo`, con `ruta` como prefijo.
+pub(crate) fn validar(nodo: &Nodo, valor: &Value, ruta: &str, errores: &mut Vec<ErrorConfig>) {
+    let mut err = |mensaje: String| errores.push(ErrorConfig::nuevo(ruta, mensaje));
+    match nodo {
+        Nodo::Entero { min, max } => {
+            let ok = valor
+                .as_u64()
+                .is_some_and(|n| n >= *min && max.is_none_or(|m| n <= m));
+            if !ok {
+                err(match max {
+                    Some(m) if m == min => format!("se esperaba {min}, no {}", corto(valor)),
+                    Some(m) => format!("se esperaba un entero de {min} a {m}, no {}", corto(valor)),
+                    None => format!("se esperaba un entero de {min} o mas, no {}", corto(valor)),
+                });
+            }
+        }
+        Nodo::Booleano { solo } => match (solo, valor.as_bool()) {
+            (Some(b), Some(x)) if *b == x => {}
+            (Some(b), _) => err(format!("solo admite {b}")),
+            (None, Some(_)) => {}
+            (None, None) => err(format!("se esperaba true o false, no {}", corto(valor))),
+        },
+        Nodo::Texto { valores, regla } => {
+            let Some(t) = valor.as_str() else {
+                err(format!("se esperaba un texto, no {}", corto(valor)));
+                return;
+            };
+            if !valores.is_empty() && !valores.contains(&t) {
+                err(format!(
+                    "valor no admitido «{t}» (admite: {})",
+                    valores.join(", ")
+                ));
+            } else if let Some(m) = comprobar(*regla, t) {
+                err(m);
+            }
+        }
+        Nodo::Lista { de, min, primero } => {
+            let Some(l) = valor.as_array() else {
+                err(format!("se esperaba una lista, no {}", corto(valor)));
+                return;
+            };
+            if l.len() < *min {
+                err(format!(
+                    "se esperaba una lista con al menos {min} elemento(s)"
+                ));
+            }
+            if let Some(p) = primero
+                && l.first().and_then(Value::as_str) != Some(p)
+            {
+                err(format!("el primer elemento tiene que ser «{p}»"));
+            }
+            for (i, x) in l.iter().enumerate() {
+                validar(de, x, &format!("{ruta}[{i}]"), errores);
+            }
+        }
+        Nodo::Objeto(campos) => {
+            let Some(m) = valor.as_object() else {
+                err(format!("se esperaba un mapa, no {}", corto(valor)));
+                return;
+            };
+            for (k, x) in m {
+                let r = unir(ruta, k);
+                match campos.iter().find(|c| c.nombre == k) {
+                    Some(c) => validar(&c.nodo, x, &r, errores),
+                    None => errores.push(ErrorConfig::nuevo(&r, no_admitida(ruta, k))),
+                }
+            }
+            for c in campos
+                .iter()
+                .filter(|c| c.obligatorio && !m.contains_key(c.nombre))
+            {
+                errores.push(ErrorConfig::nuevo(
+                    &unir(ruta, c.nombre),
+                    "falta la clave obligatoria",
+                ));
+            }
+        }
+        Nodo::UnaDe(formas) => {
+            let encaja = formas.iter().any(|f| {
+                let mut e = Vec::new();
+                validar(f, valor, ruta, &mut e);
+                e.is_empty()
+            });
+            if !encaja {
+                let resumen: Vec<String> = formas.iter().map(resumir).collect();
+                err(format!(
+                    "no encaja con ninguna forma admitida: {}",
+                    resumen.join(" o ")
+                ));
+            }
+        }
+    }
+}
+
+/// Reglas que cruzan campos y que la tabla no expresa.
+pub(crate) fn cruzadas(raiz: &Value, errores: &mut Vec<ErrorConfig>) {
+    if let Some(ds) = raiz.get("derivados").and_then(Value::as_array) {
+        for (i, d) in ds.iter().enumerate() {
+            let escribe = d.get("escribe").is_some();
+            match d.get("tipo").and_then(Value::as_str) {
+                Some("genera") if !escribe => errores.push(ErrorConfig::nuevo(
+                    &format!("derivados[{i}].escribe"),
+                    "obligatoria en un derivado de tipo genera",
+                )),
+                Some("comprueba") if escribe => errores.push(ErrorConfig::nuevo(
+                    &format!("derivados[{i}].escribe"),
+                    "prohibida en un derivado de tipo comprueba",
+                )),
+                _ => {}
+            }
+        }
+    }
+    if let Some(rs) = raiz.get("recursos").and_then(Value::as_array) {
+        let mut vistos = Vec::new();
+        for (i, r) in rs.iter().enumerate() {
+            if let Some(id) = r.get("id").and_then(Value::as_str) {
+                if vistos.contains(&id) {
+                    errores.push(ErrorConfig::nuevo(
+                        &format!("recursos[{i}].id"),
+                        format!("«{id}» repetido"),
+                    ));
+                }
+                vistos.push(id);
+            }
+        }
+    }
+}
+
+/// Rellena las claves que faltan con el valor por defecto de la tabla.
+pub(crate) fn aplicar_defectos(nodo: &Nodo, valor: &mut Value) {
+    match (nodo, valor) {
+        (Nodo::Objeto(campos), Value::Object(m)) => {
+            for c in campos {
+                if !m.contains_key(c.nombre)
+                    && let Some(d) = &c.defecto
+                {
+                    m.insert(c.nombre.to_owned(), d.clone());
+                }
+                if let Some(x) = m.get_mut(c.nombre) {
+                    aplicar_defectos(&c.nodo, x);
+                }
+            }
+        }
+        (Nodo::Lista { de, .. }, Value::Array(l)) => {
+            for x in l {
+                aplicar_defectos(de, x);
+            }
+        }
+        _ => {}
+    }
+}
+
+fn no_admitida(ruta: &str, clave: &str) -> String {
+    if ruta.is_empty() && CLAVES_016.contains(&clave) {
+        "clave no admitida: es de la configuracion de la 0.16 (tc2 actualizar la traduce)"
+            .to_owned()
+    } else {
+        "clave no admitida".to_owned()
+    }
+}
+
+fn unir(ruta: &str, clave: &str) -> String {
+    if ruta.is_empty() {
+        clave.to_owned()
+    } else {
+        format!("{ruta}.{clave}")
+    }
+}
+
+/// El valor en el mensaje, corto y sin saltos de linea.
+fn corto(v: &Value) -> String {
+    let s = match v {
+        Value::Null => "nada".to_owned(),
+        Value::String(t) => format!("«{t}»"),
+        Value::Array(_) => "una lista".to_owned(),
+        Value::Object(_) => "un mapa".to_owned(),
+        otro => otro.to_string(),
+    };
+    let s: String = s.chars().filter(|c| !c.is_control()).take(60).collect();
+    s
+}
+
+fn resumir(n: &Nodo) -> String {
+    match n {
+        Nodo::Objeto(c) => format!(
+            "un mapa con {}",
+            c.iter()
+                .map(|x: &Campo| x.nombre)
+                .collect::<Vec<_>>()
+                .join(", ")
+        ),
+        Nodo::Lista {
+            primero: Some(p), ..
+        } => format!("una lista que empieza por {p}"),
+        Nodo::Lista { .. } => "una lista".to_owned(),
+        _ => "un valor".to_owned(),
+    }
+}
+
+/// Mensaje de error si `t` no cumple `regla`.
+pub(crate) fn comprobar(regla: Regla, t: &str) -> Option<String> {
+    let control = t.chars().any(char::is_control);
+    match regla {
+        Regla::Valor => None,
+        Regla::Nombre => (t.is_empty() || control || t.trim() != t).then(|| {
+            "texto vacio, con espacios en los extremos o con caracteres de control".to_owned()
+        }),
+        Regla::Orden => {
+            if t.trim().is_empty() {
+                Some("orden vacia".to_owned())
+            } else if control {
+                Some("la orden tiene un caracter de control (¿un \\v de una ruta de Windows entre comillas dobles? escribela entre comillas simples o con /)".to_owned())
+            } else {
+                None
+            }
+        }
+        Regla::Glob | Regla::GlobAcotado => {
+            if let Some(m) = glob_malo(t) {
+                Some(format!("glob no admitido «{t}»: {m}"))
+            } else if regla == Regla::GlobAcotado
+                && matches!(t, "*" | "**" | "**/*" | "**/**" | "./**" | "./*")
+            {
+                Some(format!("glob no admitido «{t}»: nunca todo el arbol"))
+            } else {
+                None
+            }
+        }
+        Regla::RutaTareas => ruta_tareas(t)
+            .err()
+            .map(|m| format!("ruta no admitida «{t}»: {m}")),
+        Regla::Cron => {
+            let campos: Vec<&str> = t.split_whitespace().collect();
+            let ok = campos.len() == 5
+                && campos.iter().all(|c| {
+                    c.bytes()
+                        .all(|b| b.is_ascii_digit() || b"*,/-".contains(&b))
+                });
+            (!ok).then(|| {
+                format!("cron no admitido «{t}»: se esperan 5 campos con cifras, *, ',', / o -")
+            })
+        }
+        Regla::Url => Base::nueva(t)
+            .err()
+            .map(|m| format!("URL no admitida: {m}")),
+    }
+}
+
+fn glob_malo(g: &str) -> Option<&'static str> {
+    if g.is_empty() {
+        Some("vacio")
+    } else if g.chars().any(char::is_control) {
+        Some("caracter de control")
+    } else if g.starts_with('!') {
+        Some("sin negaciones")
+    } else if g.starts_with(['/', '\\']) || g.as_bytes().get(1) == Some(&b':') {
+        Some("sin rutas absolutas")
+    } else if g.split(['/', '\\']).any(|p| p == "..") {
+        Some("sin ..")
+    } else {
+        None
+    }
+}
+
+/// Normaliza una ruta de `derivados[].escribe` (F1-11, decision 1): sin
+/// segmentos `.` ni vacios y con `..` resuelto; bajo `tareas/`.
+pub(crate) fn ruta_tareas(t: &str) -> Result<String, &'static str> {
+    if t.contains(['\\', ':']) {
+        return Err("sin \\ ni :");
+    }
+    if t.contains(['*', '?', '[', ']', '{', '}']) {
+        return Err("sin comodines");
+    }
+    if t.chars().any(char::is_control) {
+        return Err("caracter de control");
+    }
+    if t.starts_with('/') {
+        return Err("sin rutas absolutas");
+    }
+    let mut partes: Vec<&str> = Vec::new();
+    for p in t.split('/') {
+        match p {
+            "" | "." => {}
+            ".." => {
+                if partes.pop().is_none() {
+                    return Err("sale de la raiz");
+                }
+            }
+            p => partes.push(p),
+        }
+    }
+    if partes
+        .iter()
+        .any(|p| crate::git::estado::reservado_en_windows(p))
+    {
+        return Err("nombre reservado de Windows");
+    }
+    if partes.iter().any(|p| p.eq_ignore_ascii_case(".git")) {
+        return Err("nunca .git");
+    }
+    match partes.as_slice() {
+        ["tareas", resto @ ..] if !resto.is_empty() => {
+            let segunda = resto.first().copied().unwrap_or("");
+            if resto.len() == 1 && segunda.eq_ignore_ascii_case("ROADMAP.md") {
+                Err("tareas/ROADMAP.md es reservada")
+            } else if es_carpeta_de_tarea(segunda) {
+                Err("las carpetas tareas/TASK-NNN/ son reservadas")
+            } else {
+                Ok(partes.join("/"))
+            }
+        }
+        ["tareas"] => Err("nunca tareas/ entera"),
+        _ => Err("tiene que ir bajo tareas/"),
+    }
+}
+
+fn es_carpeta_de_tarea(p: &str) -> bool {
+    p.get(..5).is_some_and(|x| x.eq_ignore_ascii_case("TASK-"))
+        && p.get(5..)
+            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn reglas_de_texto() {
+        assert!(
+            comprobar(Regla::Orden, "scripts\u{b}erificar.bat")
+                .is_some_and(|m| m.contains("comillas simples"))
+        );
+        assert!(comprobar(Regla::Orden, "  ").is_some());
+        assert!(comprobar(Regla::Orden, "cargo test").is_none());
+        for g in ["!docs/**", "/abs", "C:/x", "a/../b", "..", ""] {
+            assert!(comprobar(Regla::Glob, g).is_some(), "{g}");
+        }
+        assert!(comprobar(Regla::Glob, "src/auth/**").is_none());
+        assert!(comprobar(Regla::Glob, "a..b/*").is_none());
+        assert!(comprobar(Regla::GlobAcotado, "**").is_some());
+        assert!(comprobar(Regla::GlobAcotado, "*").is_some());
+        assert!(comprobar(Regla::GlobAcotado, "scripts/*.py").is_none());
+        assert!(comprobar(Regla::Cron, "0 3 * * 1-5").is_none());
+        assert!(comprobar(Regla::Cron, "0 3 * *").is_some());
+        assert!(comprobar(Regla::Cron, "@daily x y z w").is_some());
+        assert!(comprobar(Regla::Url, "https://git.empresa.com/api/v4").is_none());
+        assert!(comprobar(Regla::Url, "http://git.empresa.com").is_some());
+        assert!(comprobar(Regla::Nombre, " x").is_some());
+        assert!(comprobar(Regla::Nombre, "security-review").is_none());
+    }
+
+    #[test]
+    fn rutas_de_escribe() {
+        assert_eq!(ruta_tareas("tareas/hitos.md"), Ok("tareas/hitos.md".into()));
+        assert_eq!(
+            ruta_tareas("./tareas//x/../hitos.md"),
+            Ok("tareas/hitos.md".into())
+        );
+        for (r, m) in [
+            ("tareas", "entera"),
+            ("tareas/", "entera"),
+            ("docs/x.md", "bajo tareas/"),
+            ("tareas/../../x", "raiz"),
+            ("tareas/../docs/x", "bajo tareas/"),
+            ("../x", "raiz"),
+            ("tareas/roadmap.MD", "ROADMAP"),
+            ("tareas/task-012/tarea.md", "TASK-NNN"),
+            ("tareas/.GIT/x", ".git"),
+            ("tareas/aux.md", "Windows"),
+            ("tareas\\x", "\\"),
+            ("tareas/*.md", "comodines"),
+            ("/tareas/x", "absolutas"),
+        ] {
+            let e = ruta_tareas(r).err().unwrap_or("ok");
+            assert!(e.contains(m), "{r}: {e}");
+        }
+        assert_eq!(
+            ruta_tareas("tareas/TASK-mapa.md"),
+            Ok("tareas/TASK-mapa.md".into())
+        );
+    }
+}
diff --git a/crates/tc2/src/git/estado.rs b/crates/tc2/src/git/estado.rs
index f6f9123..95e30da 100644
--- a/crates/tc2/src/git/estado.rs
+++ b/crates/tc2/src/git/estado.rs
@@ -55,7 +55,7 @@ pub(crate) fn ruta_valida(ruta: &str) -> bool {
 
 /// Nombres que Windows reserva aunque lleven extension (`aux.md`): un clon en
 /// Windows no podria sacarlos al worktree.
-fn reservado_en_windows(parte: &str) -> bool {
+pub(crate) fn reservado_en_windows(parte: &str) -> bool {
     let base = parte
         .split('.')
         .next()
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 8a0cf4f..c873886 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -4,15 +4,20 @@
 //! (capa de operaciones Git deterministas, F2-2) y [`estado`] (rama de
 //! estado `taskcode/tareas`, F2-3), [`alta`] (alta de tareas con reserva de
 //! ID, F2-4) [`tarea`] (modelo de tarea y frontmatter YAML, F2-5) y
-//! [`plataforma`] (puerto Plataforma y adaptador GitHub, F2-6). Las capas
+//! [`plataforma`] (puerto Plataforma y adaptadores GitHub y GitLab, F2-6 y
+//! F2-9), [`carril`] (tabla de suelo de F1-10) y [`config`] (configuracion
+//! con esquema validado, F2-7). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod alta;
+pub mod carril;
+pub mod config;
 pub mod contrato;
 pub mod estado;
 pub mod git;
 pub mod plataforma;
 pub mod tarea;
+mod yaml;
 
 use contrato::{Informe, Resultado};
 
diff --git a/crates/tc2/src/tarea/yaml.rs b/crates/tc2/src/tarea/yaml.rs
index 6d16c46..c2b4de1 100644
--- a/crates/tc2/src/tarea/yaml.rs
+++ b/crates/tc2/src/tarea/yaml.rs
@@ -12,8 +12,8 @@
 
 use std::collections::HashSet;
 
+use crate::yaml::opciones;
 use serde::de::DeserializeOwned;
-use serde_saphyr::{DuplicateKeyPolicy, MergeKeyPolicy, Options};
 
 use super::ErrorTarea;
 
@@ -148,25 +148,6 @@ pub(crate) fn partir(texto: &str) -> Result<Partes<'_>, ErrorTarea> {
     })
 }
 
-/// Opciones de lectura: duplicadas, claves de fusion, anclas y alias como
-/// error (ninguna tarea real los usa; un alias que crece es un vector de
-/// denegacion de servicio, F1-8).
-fn opciones() -> Options {
-    let mut o = Options::default();
-    // Booleanos y numeros solo con la grafia de YAML 1.2 (`yes`, `1_0` o `010`
-    // no son bool ni entero, como para un parser estandar).
-    o.strict_booleans = true;
-    o.underscored_numbers_as_strings = true;
-    o.leading_zero_numbers_as_strings = true;
-    o.duplicate_keys = DuplicateKeyPolicy::Error;
-    o.merge_keys = MergeKeyPolicy::Error;
-    let mut presupuesto = o.budget.unwrap_or_default();
-    presupuesto.max_aliases = 0;
-    presupuesto.max_anchors = 0;
-    o.budget = Some(presupuesto);
-    o
-}
-
 /// Valida el YAML entero y decodifica las claves conocidas en `T` (las
 /// desconocidas se validan como YAML y se ignoran: las conserva [`partir`]).
 pub(crate) fn decodificar<T: DeserializeOwned>(yaml: &str) -> Result<T, ErrorTarea> {
diff --git a/crates/tc2/src/yaml.rs b/crates/tc2/src/yaml.rs
new file mode 100644
index 0000000..c0c7678
--- /dev/null
+++ b/crates/tc2/src/yaml.rs
@@ -0,0 +1,25 @@
+//! Lectura YAML estricta compartida (ADR F1-14, punto 1: el YAML vive detras
+//! de un modulo propio). La usan el frontmatter de `tarea.md` (F2-5) y la
+//! configuracion `.taskcode/config.yml` (F2-7): la politica contra alias y
+//! claves duplicadas de F1-8 existe una sola vez.
+
+use serde_saphyr::{DuplicateKeyPolicy, MergeKeyPolicy, Options};
+
+/// Opciones de lectura: duplicadas, claves de fusion, anclas y alias como
+/// error (ninguna tarea ni configuracion real los usa; un alias que crece es
+/// un vector de denegacion de servicio, F1-8).
+pub(crate) fn opciones() -> Options {
+    let mut o = Options::default();
+    // Booleanos y numeros solo con la grafia de YAML 1.2 (`yes`, `1_0` o `010`
+    // no son bool ni entero, como para un parser estandar).
+    o.strict_booleans = true;
+    o.underscored_numbers_as_strings = true;
+    o.leading_zero_numbers_as_strings = true;
+    o.duplicate_keys = DuplicateKeyPolicy::Error;
+    o.merge_keys = MergeKeyPolicy::Error;
+    let mut presupuesto = o.budget.unwrap_or_default();
+    presupuesto.max_aliases = 0;
+    presupuesto.max_anchors = 0;
+    o.budget = Some(presupuesto);
+    o
+}
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-023/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-023/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-023/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/config.rs"
````

````
crates/tc2/tests/config.rs | 377 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 377 insertions(+)
````
