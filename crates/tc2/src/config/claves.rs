//! Tabla declarativa de las claves de `.taskcode/config.yml` (F2-7): la
//! fuente unica del validador, de los valores por defecto y del esquema JSON
//! publicado. Cada clave de primer nivel cita el ADR que la pide.

use serde_json::{Map, Value, json};

use crate::carril::Carril;
use crate::i18n::catalogo::config as t;
use crate::i18n::{Idioma, Texto};

/// Tope de claves de primer nivel (plan, F2-7; F1-11, decision 1). 13 desde
/// la adenda de TASK-081 en F1-11 (`dias_sin_actividad`, F3-10 D1a).
pub const TOPE_CLAVES: usize = 13;

/// Fases de `tc2` en las que puede correr un derivado (F1-11 se lo pide a
/// F2-7): las ordenes que escriben en la rama de estado.
pub const FASES: [&str; 10] = [
    "new",
    "plan",
    "approve",
    "start",
    "review",
    "finish",
    "descartar",
    "pausa",
    "liberar",
    "reasignar",
];

/// Modelos de la escala de F1-15.
pub const MODELOS: [&str; 3] = ["haiku", "sonnet", "opus"];

/// Dominios cerrados de F1-15 (decision 3).
pub const DOMINIOS: [&str; 2] = ["puertas-seguridad", "rendimiento"];

/// Claves planas de la configuracion 0.16 que ya no existen en la 2.0: su
/// error nombra `tc2 actualizar` (F1-12). `modo_flujo` y `tope_rondas`
/// siguen siendo claves de la 2.0.
pub const CLAVES_016: [&str; 30] = [
    "rama_base",
    "agente_revisor_por_defecto",
    "limite_wip",
    "comando_sincronizacion",
    "rutas_sincronizacion",
    "timeout_sincronizacion",
    "fases_sincronizacion",
    "excluir_de_revision",
    "contexto_revision",
    "cierre_por_defecto",
    "plataforma_remota",
    "url_base_remoto",
    "comando_verificacion",
    "timeout_verificacion",
    "comando_verificacion_selectiva",
    "timeout_verificacion_selectiva",
    "comando_heldout",
    "informe_cobertura",
    "umbral_cobertura",
    "rutas_documentacion",
    "comando_verificacion_documentacion",
    "rutas_verificacion_parcial",
    "documento_estado",
    "comando_verificacion_freno",
    "presupuesto_freno_s",
    "verificacion_por_lote",
    "comando_verificacion_pesada",
    "verificacion_pesada_en",
    "umbral_crecimiento_suite",
    "tareas_entre_podas",
];

/// Regla de un texto, ademas de su lista de valores.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Regla {
    /// Cualquier texto de la lista de valores.
    Valor,
    /// Nombre o etiqueta: no vacio, sin caracteres de control ni espacios en
    /// los extremos.
    Nombre,
    /// Orden de shell: no vacia y sin caracteres de control.
    Orden,
    /// Glob relativo: sin negacion, rutas absolutas ni `..`.
    Glob,
    /// Glob de `confianza_incluye`: como [`Regla::Glob`] y nunca todo el
    /// arbol.
    GlobAcotado,
    /// Ruta de `derivados[].escribe`: normalizada, bajo `tareas/`.
    RutaTareas,
    /// Expresion cron de 5 campos.
    Cron,
    /// URL de la API de la plataforma.
    Url,
}

impl Regla {
    /// Lo que exige, del catalogo. El esquema JSON publicado usa el texto en
    /// espanol (decision de F2-8: el esquema no se traduce).
    pub(crate) fn texto(self) -> Option<&'static Texto> {
        match self {
            Self::Valor => None,
            Self::Nombre => Some(&t::REGLA_NOMBRE),
            Self::Orden => Some(&t::REGLA_ORDEN),
            Self::Glob => Some(&t::REGLA_GLOB),
            Self::GlobAcotado => Some(&t::REGLA_GLOB_ACOTADO),
            Self::RutaTareas => Some(&t::REGLA_RUTA_TAREAS),
            Self::Cron => Some(&t::REGLA_CRON),
            Self::Url => Some(&t::REGLA_URL),
        }
    }
}

/// Forma de un valor.
#[derive(Debug, Clone)]
pub(crate) enum Nodo {
    /// Entero en `[min, max]` (`max` `None`: sin tope).
    Entero { min: u64, max: Option<u64> },
    /// Booleano; con `solo`, el unico valor admitido.
    Booleano { solo: Option<bool> },
    /// Texto; si `valores` no esta vacio, uno de ellos.
    Texto {
        valores: &'static [&'static str],
        regla: Regla,
    },
    /// Lista con al menos `min` elementos; con `primero`, el primer elemento
    /// es ese texto.
    Lista {
        de: Box<Nodo>,
        min: usize,
        primero: Option<&'static str>,
    },
    /// Mapa cerrado: solo estas claves.
    Objeto(Vec<Campo>),
    /// Una de varias formas.
    UnaDe(Vec<Nodo>),
}

/// Una clave de un mapa cerrado.
#[derive(Debug, Clone)]
pub(crate) struct Campo {
    pub nombre: &'static str,
    pub nodo: Nodo,
    pub obligatorio: bool,
    pub defecto: Option<Value>,
    /// ADR de origen (obligatorio en las de primer nivel).
    pub origen: &'static str,
    pub descripcion: &'static str,
}

impl Campo {
    fn opcional(nombre: &'static str, nodo: Nodo, descripcion: &'static str) -> Self {
        Self {
            nombre,
            nodo,
            obligatorio: false,
            defecto: None,
            origen: "",
            descripcion,
        }
    }

    fn obligatorio(nombre: &'static str, nodo: Nodo, descripcion: &'static str) -> Self {
        Self {
            obligatorio: true,
            ..Self::opcional(nombre, nodo, descripcion)
        }
    }

    fn con_defecto(mut self, v: Value) -> Self {
        self.defecto = Some(v);
        self
    }

    fn de(mut self, origen: &'static str) -> Self {
        self.origen = origen;
        self
    }
}

fn texto(regla: Regla) -> Nodo {
    Nodo::Texto {
        valores: &[],
        regla,
    }
}

fn uno_de(valores: &'static [&'static str]) -> Nodo {
    Nodo::Texto {
        valores,
        regla: Regla::Valor,
    }
}

fn lista(de: Nodo, min: usize) -> Nodo {
    Nodo::Lista {
        de: Box::new(de),
        min,
        primero: None,
    }
}

fn entero(min: u64, max: u64) -> Nodo {
    Nodo::Entero {
        min,
        max: Some(max),
    }
}

/// Runner de `completa`, `pesada` y la verificacion programada (F1-7,
/// decision 5): `{github: windows|linux}` o `[self-hosted, ...]`.
fn runner() -> Nodo {
    Nodo::UnaDe(vec![
        Nodo::Objeto(vec![Campo::obligatorio(
            "github",
            uno_de(&["windows", "linux"]),
            "runner alojado de GitHub",
        )]),
        Nodo::Lista {
            de: Box::new(texto(Regla::Nombre)),
            min: 1,
            primero: Some("self-hosted"),
        },
    ])
}

fn perfil(nombre: &'static str, con_runner: bool, obligatorio: bool) -> Campo {
    let mut campos = vec![Campo::obligatorio(
        "timeout",
        entero(1, 21_600),
        "segundos; en rapida es un presupuesto que no bloquea, en las demas agotarlo es rojo",
    )];
    if con_runner {
        campos.push(Campo::obligatorio(
            "runner",
            runner(),
            "donde corre en la CI",
        ));
    }
    let c = Campo::opcional(nombre, Nodo::Objeto(campos), "perfil de F1-7");
    Campo { obligatorio, ..c }
}

fn verificacion() -> Nodo {
    Nodo::Objeto(vec![
        Campo::obligatorio(
            "comando",
            texto(Regla::Orden),
            "comando del proyecto; se llama con --perfil <p>",
        ),
        Campo::obligatorio(
            "shell",
            uno_de(&["cmd", "bash", "pwsh", "sh"]),
            "shell con la que se lanza en todas partes (H16)",
        ),
        Campo::obligatorio(
            "perfiles",
            Nodo::Objeto(vec![
                perfil("rapida", false, false),
                perfil("impacto", false, false),
                perfil("completa", true, true),
                perfil("pesada", true, false),
            ]),
            "perfiles de F1-7; completa obligatoria",
        ),
        Campo::opcional(
            "huella_arbol",
            lista(texto(Regla::Glob), 0),
            "ficheros versionados de la clave de cache (F1-7 dec. 2)",
        )
        .con_defecto(json!([])),
        Campo::opcional(
            "huella_local",
            texto(Regla::Orden),
            "imprime lo que la suite lee fuera del arbol",
        ),
        Campo::opcional(
            "programada",
            Nodo::Objeto(vec![
                Campo::obligatorio(
                    "comando",
                    texto(Regla::Orden),
                    "pasos que dependen de la red",
                ),
                Campo::obligatorio("cron", texto(Regla::Cron), "cuando corre"),
                Campo::obligatorio("runner", runner(), "donde corre"),
            ]),
            "job programado fuera del check cacheado",
        ),
    ])
}

fn derivado() -> Nodo {
    Nodo::Objeto(vec![
        Campo::obligatorio("comando", texto(Regla::Orden), "que ejecuta"),
        Campo::obligatorio(
            "tipo",
            uno_de(&["genera", "comprueba"]),
            "si escribe ficheros o solo comprueba",
        ),
        Campo::opcional(
            "escribe",
            lista(texto(Regla::RutaTareas), 1),
            "lo unico que un genera puede escribir; obligatoria en genera, prohibida en comprueba",
        ),
        Campo::opcional(
            "fases",
            lista(uno_de(&FASES), 1),
            "en que transiciones corre (por defecto, en todas)",
        )
        .con_defecto(json!(FASES)),
        Campo::opcional("timeout", entero(1, 600), "segundos").con_defecto(json!(60)),
    ])
}

fn politica_de(c: Carril) -> Campo {
    Campo::opcional(
        c.nombre(),
        Nodo::Objeto(vec![
            Campo::opcional(
                "roles_diseno",
                entero(c.roles_min(), 5),
                "roles de diseno minimos; no baja del suelo",
            ),
            Campo::opcional(
                "plan_aprobado",
                Nodo::Booleano { solo: Some(true) },
                "solo se puede exigir, no quitar",
            ),
            Campo::opcional(
                "paradas_extra",
                lista(uno_de(&["start", "review"]), 0),
                "paradas del automatico que se anaden",
            )
            .con_defecto(json!([])),
            Campo::opcional(
                "revisoras_extra",
                lista(texto(Regla::Nombre), 0),
                "revisoras que se anaden",
            )
            .con_defecto(json!([])),
            Campo::opcional(
                "lote",
                Nodo::Booleano { solo: Some(false) },
                "solo se puede sacar del lote",
            ),
            Campo::opcional(
                "perfil",
                Nodo::Texto {
                    valores: c.perfil().y_mayores(),
                    regla: Regla::Valor,
                },
                "perfil de verificacion; no baja del suelo",
            ),
        ]),
        "endurece la fila del carril",
    )
}

fn senales() -> Nodo {
    Nodo::Objeto(vec![
        Campo::opcional(
            "etiquetas",
            lista(texto(Regla::Nombre), 0),
            "etiquetas que se anaden",
        )
        .con_defecto(json!([])),
        Campo::opcional(
            "palabras",
            lista(texto(Regla::Nombre), 0),
            "palabras que se anaden",
        )
        .con_defecto(json!([])),
        Campo::opcional("rutas", lista(texto(Regla::Glob), 0), "globs que se anaden")
            .con_defecto(json!([])),
    ])
}

fn opcional_entero(nombre: &'static str, descripcion: &'static str) -> Campo {
    Campo::opcional(nombre, Nodo::Entero { min: 1, max: None }, descripcion)
}

/// Las claves de primer nivel, en orden.
pub(crate) fn tabla() -> Vec<Campo> {
    vec![
        Campo::obligatorio(
            "formato",
            entero(2, 2),
            "version del formato del proyecto; con 2 manda la 2.0",
        )
        .de("F1-12 D4a, decision 6"),
        Campo::opcional(
            "politica_carriles",
            Nodo::Objeto(Carril::TODOS.into_iter().map(politica_de).collect()),
            "endurece la politica por carril; solo los cuatro carriles",
        )
        .con_defecto(json!({}))
        .de("F1-10 decision 2"),
        Campo::opcional(
            "rutas_carril",
            Nodo::Objeto(
                [Carril::Fix, Carril::RiesgoAlto]
                    .into_iter()
                    .map(|c| {
                        Campo::opcional(
                            c.nombre(),
                            lista(texto(Regla::Glob), 1),
                            "globs que suben a este carril",
                        )
                    })
                    .collect(),
            ),
            "rutas que suben el carril minimo; solo fix y riesgo-alto",
        )
        .con_defecto(json!({}))
        .de("F1-10 decision 2"),
        Campo::opcional(
            "verificacion",
            verificacion(),
            "contrato de verificacion de F1-7",
        )
        .de("F1-11 decision 1"),
        Campo::opcional(
            "derivados",
            lista(derivado(), 0),
            "comandos de las transiciones que el check estado repite",
        )
        .con_defecto(json!([]))
        .de("F1-11 decision 1"),
        Campo::opcional(
            "confianza_incluye",
            lista(texto(Regla::GlobAcotado), 0),
            "ficheros que suma la huella de confianza",
        )
        .con_defecto(json!([]))
        .de("F1-11 decision 1; F1-8 decision 5"),
        Campo::opcional(
            "plataforma",
            Nodo::Objeto(vec![
                Campo::obligatorio(
                    "proveedor",
                    uno_de(&["github", "gitlab"]),
                    "plataforma del remoto",
                ),
                Campo::obligatorio("api", texto(Regla::Url), "URL de la API"),
                Campo::opcional(
                    "revisor_id",
                    Nodo::Entero { min: 0, max: None },
                    "origen de revision-ia: el app_id de la App del revisor en GitHub o su user.id en GitLab; 0 o ausente, no hay y tc2 init --aplicar-puertas se rechaza (TASK-118)",
                ),
            ]),
            "plataforma de un host que no es github.com ni gitlab.com",
        )
        .de("F1-13 decision 7"),
        Campo::opcional(
            "recursos",
            lista(
                Nodo::Objeto(vec![
                    Campo::obligatorio(
                        "id",
                        texto(Regla::Nombre),
                        "nombre de la skill, el agente o el ejecutable",
                    ),
                    Campo::obligatorio(
                        "tipo",
                        uno_de(&["skill", "agente", "ejecutable"]),
                        "que es",
                    ),
                    Campo::opcional(
                        "rol",
                        uno_de(&["revisor", "diseno", "ejecucion"]),
                        "rol que cubre; sin rol no se elige (F1-15 decision 8)",
                    ),
                    Campo::opcional(
                        "etiquetas",
                        lista(texto(Regla::Nombre), 1),
                        "etiquetas con las que compite una skill de ejecucion (S07)",
                    )
                    .con_defecto(json!([])),
                    Campo::opcional(
                        "prioridad",
                        entero(0, 1000),
                        "desempate de S07: mayor primero",
                    ),
                    Campo::opcional(
                        "dominios",
                        lista(uno_de(&DOMINIOS), 1),
                        "dominios de F1-15 que cubre",
                    )
                    .con_defecto(json!([])),
                ]),
                0,
            ),
            "recursos necesarios que el proyecto anade; solo anade",
        )
        .con_defecto(json!([]))
        .de("F1-14 D6a; F1-15 decision 6"),
        Campo::opcional(
            "seleccion",
            Nodo::Objeto(vec![
                Campo::opcional(
                    "modelos_permitidos",
                    lista(uno_de(&MODELOS), 1),
                    "solo restringe la escala",
                )
                .con_defecto(json!(MODELOS)),
                Campo::opcional(
                    "dominios_extra",
                    Nodo::Objeto(
                        DOMINIOS
                            .into_iter()
                            .map(|d| Campo::opcional(d, senales(), "senales que se anaden"))
                            .collect(),
                    ),
                    "solo anade senales a un dominio de la lista",
                )
                .con_defecto(json!({})),
            ]),
            "seleccion de skills, agentes y modelo",
        )
        .con_defecto(json!({"modelos_permitidos": MODELOS, "dominios_extra": {}}))
        .de("F1-15 decision 3 y S15"),
        Campo::opcional(
            "modo_flujo",
            uno_de(&["manual", "semiautomatico", "automatico"]),
            "modo del flujo; lo lee tambien la 0.16",
        )
        .con_defecto(json!("manual"))
        .de("F1-12 decision 1 (convivencia); RF-21"),
        Campo::opcional(
            "tope_rondas",
            entero(1, 10),
            "rondas de revision sin preguntar",
        )
        .con_defecto(json!(2))
        .de("F1-11 PoC B; F1-16 D6a"),
        Campo::opcional(
            "lote",
            Nodo::Objeto(vec![
                opcional_entero("presupuesto_tokens", "presupuesto del lote en tokens"),
                opcional_entero("presupuesto_horas", "presupuesto del lote en horas"),
                opcional_entero("tope_tokens_tarea", "tope de tokens por tarea"),
            ]),
            "presupuesto por defecto de los lotes; sin cifras inventadas",
        )
        .de("F1-16, consecuencias"),
        Campo::opcional(
            "dias_sin_actividad",
            entero(1, 365),
            "dias sin actividad tras los que otra persona puede liberar o reasignar una tarea y status y board la marcan",
        )
        .con_defecto(json!(7))
        .de("F1-11 adenda de TASK-081; F3-10 D1a"),
    ]
}

/// JSON Schema 2020-12 de la configuracion, generado de [`tabla`].
pub fn esquema_json() -> String {
    let raiz = Nodo::Objeto(tabla());
    let mut v = a_esquema(&raiz);
    if let Value::Object(m) = &mut v {
        m.insert(
            "$schema".into(),
            json!("https://json-schema.org/draft/2020-12/schema"),
        );
        m.insert(
            "title".into(),
            json!("Configuracion de TaskCode 2 (.taskcode/config.yml)"),
        );
        m.insert(
            "description".into(),
            json!("Generado por tc2 (config::esquema_json). La primera linea del fichero es ---. Una clave o un valor desconocido falla cerrado."),
        );
    }
    let mut s = serde_json::to_string_pretty(&v).unwrap_or_default();
    s.push('\n');
    s
}

fn a_esquema(n: &Nodo) -> Value {
    match n {
        Nodo::Entero { min, max } => {
            let mut m = Map::new();
            m.insert("type".into(), json!("integer"));
            m.insert("minimum".into(), json!(min));
            if let Some(x) = max {
                m.insert("maximum".into(), json!(x));
            }
            Value::Object(m)
        }
        Nodo::Booleano { solo: Some(b) } => json!({"const": b}),
        Nodo::Booleano { solo: None } => json!({"type": "boolean"}),
        Nodo::Texto { valores, regla } => {
            let mut m = Map::new();
            m.insert("type".into(), json!("string"));
            if valores.is_empty() {
                m.insert("minLength".into(), json!(1));
            } else {
                m.insert("enum".into(), json!(valores));
            }
            if let Some(r) = regla.texto() {
                m.insert("description".into(), json!(r.en_idioma(Idioma::Es)));
            }
            Value::Object(m)
        }
        Nodo::Lista { de, min, primero } => {
            let mut m = Map::new();
            m.insert("type".into(), json!("array"));
            m.insert("items".into(), a_esquema(de));
            if *min > 0 {
                m.insert("minItems".into(), json!(min));
            }
            if let Some(p) = primero {
                m.insert("prefixItems".into(), json!([{"const": p}]));
            }
            Value::Object(m)
        }
        Nodo::Objeto(campos) => {
            let mut props = Map::new();
            let mut requeridas = Vec::new();
            for c in campos {
                let mut e = a_esquema(&c.nodo);
                if let Value::Object(m) = &mut e {
                    let d = if c.origen.is_empty() {
                        c.descripcion.to_owned()
                    } else {
                        format!("{} [{}]", c.descripcion, c.origen)
                    };
                    let d = match m.get("description").and_then(Value::as_str) {
                        Some(regla) => format!("{d}. {regla}"),
                        None => d,
                    };
                    m.insert("description".into(), json!(d));
                    if let Some(x) = &c.defecto {
                        m.insert("default".into(), x.clone());
                    }
                }
                props.insert(c.nombre.into(), e);
                if c.obligatorio {
                    requeridas.push(c.nombre);
                }
            }
            let mut m = Map::new();
            m.insert("type".into(), json!("object"));
            m.insert("additionalProperties".into(), json!(false));
            m.insert("properties".into(), Value::Object(props));
            if !requeridas.is_empty() {
                m.insert("required".into(), json!(requeridas));
            }
            Value::Object(m)
        }
        Nodo::UnaDe(formas) => json!({"anyOf": formas.iter().map(a_esquema).collect::<Vec<_>>()}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claves_de_primer_nivel_no_pasan_del_tope_y_cada_una_cita_su_adr() {
        let t = tabla();
        assert_eq!(
            TOPE_CLAVES, 13,
            "el tope lo fija el plan (F2-7) y la adenda de TASK-081 en F1-11"
        );
        assert!(t.len() <= TOPE_CLAVES, "{} claves", t.len());
        for c in &t {
            assert!(!c.origen.is_empty(), "{} sin origen", c.nombre);
        }
        let mut nombres: Vec<_> = t.iter().map(|c| c.nombre).collect();
        nombres.sort_unstable();
        nombres.dedup();
        assert_eq!(nombres.len(), t.len(), "claves repetidas");
        assert_eq!(
            t.iter()
                .filter(|c| c.obligatorio)
                .map(|c| c.nombre)
                .collect::<Vec<_>>(),
            ["formato"]
        );
        for c in CLAVES_016 {
            assert!(t.iter().all(|x| x.nombre != c), "{c} es de la 2.0");
        }
    }
}
