//! Los roles del brainstorm de la fase de diseno (F4-1, «mismos roles que
//! la 0.16»): copia literal de `src/core/roles-brainstorm.ts` del plugin
//! 0.16.2 (ids, preguntas, «que miras», «que no miras» y entregables).
//!
//! La heuristica y el carril fijan CUANTOS; este modulo fija CUALES, de
//! forma determinista: la escalera en orden de prioridad (arquitectura,
//! riesgos, testing, dominio) y los N primeros. Con un rol entra
//! arquitectura, el unico que propone una forma; con tres se cae dominio.
//!
//! Los roles condicionales no compiten por el presupuesto: entran por las
//! etiquetas de la tarea y se suman a la escalera, solo si esta lanza
//! [`MIN_ROLES_PARA_CONDICIONALES`] o mas (con 0 o 1 rol no hay unificador
//! y el condicional quedaria de autor unico de un plan cuya forma no le toca
//! proponer, TASK-080 de la 0.16).
//!
//! Cada `id` es el `name:` del agente que se lanza (`brainstorm-*`): tc2 no
//! trae agentes propios y no comprueba que esten instalados (inventario,
//! TASK-080).

/// Un rol del brainstorm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rol {
    /// El `name:` del agente de ese rol.
    pub id: &'static str,
    /// Nombre corto para cabeceras y la linea.
    pub titulo: &'static str,
    /// La unica pregunta del rol, literal en su peticion.
    pub pregunta: &'static str,
    /// Que mira este rol (el recorte de contexto).
    pub mira: &'static [&'static str],
    /// Que no mira, dicho explicitamente.
    pub no_mira: &'static [&'static str],
    /// Secciones que su salida tiene que traer.
    pub entregables: &'static [&'static str],
}

/// La escalera, en orden de prioridad (0.16.2).
pub const ESCALERA: [Rol; 4] = [
    Rol {
        id: "brainstorm-arquitectura",
        titulo: "arquitectura",
        pregunta: "Dado lo que ya existe, ¿donde encaja este cambio y que forma tiene?",
        mira: &[
            "Los modulos y ficheros que ya resuelven algo parecido, para extenderlos en vez de duplicarlos.",
            "Que se crea nuevo, que se extiende y en que orden se construye.",
            "Los limites que el cambio cruza: contratos publicos, formatos de fichero, esquemas.",
            "El precedente interno mas cercano: como se resolvio la ultima vez un problema de esta forma.",
        ],
        no_mira: &[
            "Como se prueba (es del rol de testing).",
            "Por donde se rompe (es del rol de riesgos).",
            "Las reglas de negocio (son del rol de dominio).",
        ],
        entregables: &[
            "Enfoque propuesto, con rutas y nombres concretos",
            "Que se extiende y que se crea",
            "Limites que cruza",
            "La decision de diseño que mas te preocupa (UNA sola)",
        ],
    },
    Rol {
        id: "brainstorm-riesgos",
        titulo: "riesgos",
        pregunta: "¿Por donde se rompe esto?",
        mira: &[
            "Bordes y estados intermedios: que queda a medias si el proceso muere a mitad.",
            "Fallos parciales y concurrencia: dos ejecuciones, un recurso ocupado, un permiso denegado.",
            "Compatibilidad hacia atras con los datos y ficheros que YA existen.",
            "La vuelta atras: si esto sale mal, como se deshace y que queda inservible.",
        ],
        no_mira: &[
            "Donde encaja el cambio (es del rol de arquitectura).",
            "Que aserciones escribir (es del rol de testing).",
            "Las reglas de negocio (son del rol de dominio).",
        ],
        entregables: &[
            "Modos de fallo, ordenados por gravedad, con el escenario concreto de cada uno",
            "Estados intermedios y fallos parciales",
            "Compatibilidad hacia atras",
            "Vuelta atras",
            "El riesgo que mas te preocupa (UNO solo)",
        ],
    },
    Rol {
        id: "brainstorm-testing",
        titulo: "testing",
        pregunta: "¿Como se demuestra que esto funciona, y como envejece?",
        mira: &[
            "Que es observable desde fuera, y contra que recurso real se comprueba.",
            "Para cada prueba propuesta, LA MUTACION DEL CODIGO FUENTE QUE LA PONDRIA ROJA. Si no sabes decirla, esa prueba no vale.",
            "Que pruebas ya existentes cambian de expectativa y cuales siguen valiendo de red de regresion.",
            "Como envejece: que se rompe dentro de seis meses cuando alguien extienda esto.",
        ],
        no_mira: &[
            "Donde encaja el cambio (es del rol de arquitectura).",
            "Por donde se rompe en produccion (es del rol de riesgos).",
            "Las reglas de negocio (son del rol de dominio).",
        ],
        entregables: &[
            "Que es observable sin llamar a ningun agente",
            "Plan de pruebas, cada una con su mutacion",
            "Aserciones trampa que hay que evitar en esta tarea concreta",
            "Que pruebas existentes cambian de expectativa",
            "Como envejece",
        ],
    },
    Rol {
        id: "brainstorm-dominio",
        titulo: "dominio",
        pregunta: "¿Que sabe quien va a usar esto que no se deduce leyendo el codigo?",
        mira: &[
            "El vocabulario real: que nombra cada cosa quien trabaja en este dominio.",
            "Los invariantes del negocio que ningun tipo del lenguaje impone y que aun asi no se pueden violar.",
            "Los casos que en el codigo son simetricos y en el negocio no lo son.",
            "Cuando una solucion correcta en codigo es incorrecta para quien la va a usar.",
        ],
        no_mira: &[
            "Donde encaja el cambio (es del rol de arquitectura).",
            "Por donde se rompe (es del rol de riesgos).",
            "Como se prueba (es del rol de testing).",
        ],
        entregables: &[
            "Vocabulario e invariantes que el enfoque tiene que respetar",
            "Donde una solucion tecnicamente correcta seria incorrecta aqui",
            "Lo que hace falta preguntarle a una persona antes de implementar",
        ],
    },
];

/// Un rol que entra por las etiquetas de la tarea, fuera del presupuesto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Condicional {
    /// El rol.
    pub rol: Rol,
    /// Etiquetas que lo disparan, en minusculas; basta una.
    pub etiquetas: &'static [&'static str],
}

/// Los condicionales (0.16.2): hoy, rendimiento.
pub const CONDICIONALES: [Condicional; 1] = [Condicional {
    rol: Rol {
        id: "brainstorm-rendimiento",
        titulo: "rendimiento",
        pregunta: "¿Cuanto cuesta esto en ejecucion con los datos reales, y donde se va ese coste?",
        mira: &[
            "El coste algoritmico con el volumen real de datos: que crece mas que lineal y a partir de que tamaño se nota.",
            "Las asignaciones y copias en los bucles calientes: lo que se crea en cada iteracion o en cada frame.",
            "Las fronteras caras: copias entre JS y WASM, mensajes y transferibles hacia Web Workers, lo que bloquea el hilo principal.",
            "Las cargas grandes: mallas 3D, teselas e imagenes pesadas, cuanta memoria ocupan y cuando se liberan.",
            "Como se mide antes de optimizar: con que datos reales, que metrica y que umbral decide si hace falta.",
        ],
        no_mira: &[
            "La forma del cambio y la escalabilidad que fija el diseño (es del rol de arquitectura).",
            "Por donde se rompe (es del rol de riesgos).",
            "Que aserciones escribir (es del rol de testing).",
            "Las reglas de negocio (son del rol de dominio).",
        ],
        entregables: &[
            "Donde se va el coste, con el volumen de datos real de cada punto",
            "Fronteras caras: JS-WASM, Web Workers e hilo principal",
            "Que medir antes de optimizar, con que datos y que umbral",
            "La optimizacion que NO harias todavia (UNA sola)",
        ],
    },
    etiquetas: &["rendimiento", "wasm", "workers", "3d"],
}];

/// Por debajo de este numero de roles de la escalera no se suma ningun
/// condicional.
pub const MIN_ROLES_PARA_CONDICIONALES: usize = 2;

/// El agente unificador: no es un rol del brainstorm, consolida los que hay.
pub const UNIFICADOR: &str = "brainstorm-unificador";

/// Todos los roles (la escalera y los condicionales), para buscar uno por
/// su id (las salidas de `plan guardar`).
pub fn todos() -> impl Iterator<Item = &'static Rol> {
    ESCALERA.iter().chain(CONDICIONALES.iter().map(|c| &c.rol))
}

/// Si alguna de `etiquetas` dispara `c` (sin distinguir mayusculas).
fn dispara(c: &Condicional, etiquetas: &[String]) -> bool {
    etiquetas
        .iter()
        .any(|e| c.etiquetas.contains(&e.trim().to_lowercase().as_str()))
}

/// Los roles que lanza una tarea con `n` roles de diseno: los `n` primeros
/// de la escalera y, si son al menos [`MIN_ROLES_PARA_CONDICIONALES`], los
/// condicionales que disparan sus `etiquetas`, al final y en su orden.
///
/// `n` por encima de la escalera (la configuracion admite hasta 5,
/// `politica_carriles.<carril>.roles_diseno`) suma los condicionales que
/// falten, en su orden, aunque ninguna etiqueta los pida: 5 son los cinco
/// roles. Es la unica diferencia con `seleccionarRolesDeTarea` de la 0.16,
/// que con mas de 4 fallaba (alli el numero no podia pasar de 4).
pub fn seleccionar(n: u8, etiquetas: &[String]) -> Vec<Rol> {
    let n = usize::from(n);
    let mut roles: Vec<Rol> = ESCALERA.iter().take(n).copied().collect();
    if roles.len() < MIN_ROLES_PARA_CONDICIONALES {
        return roles;
    }
    let mut extra = n.saturating_sub(ESCALERA.len());
    for c in &CONDICIONALES {
        if dispara(c, etiquetas) {
            roles.push(c.rol);
        } else if extra > 0 {
            extra -= 1;
            roles.push(c.rol);
        }
    }
    roles
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(roles: &[Rol]) -> Vec<&'static str> {
        roles.iter().map(|r| r.id).collect()
    }

    fn etiquetas(e: &[&str]) -> Vec<String> {
        e.iter().map(|x| (*x).to_owned()).collect()
    }

    /// Criterio 1 de TASK-036: los mismos roles que la 0.16.2, en el mismo
    /// orden, con sus preguntas literales, y el mismo unificador.
    #[test]
    fn mismos_roles_y_preguntas_que_la_016() {
        let leidos: Vec<(&str, &str, &str)> =
            todos().map(|r| (r.id, r.titulo, r.pregunta)).collect();
        assert_eq!(
            leidos,
            [
                (
                    "brainstorm-arquitectura",
                    "arquitectura",
                    "Dado lo que ya existe, ¿donde encaja este cambio y que forma tiene?"
                ),
                ("brainstorm-riesgos", "riesgos", "¿Por donde se rompe esto?"),
                (
                    "brainstorm-testing",
                    "testing",
                    "¿Como se demuestra que esto funciona, y como envejece?"
                ),
                (
                    "brainstorm-dominio",
                    "dominio",
                    "¿Que sabe quien va a usar esto que no se deduce leyendo el codigo?"
                ),
                (
                    "brainstorm-rendimiento",
                    "rendimiento",
                    "¿Cuanto cuesta esto en ejecucion con los datos reales, y donde se va ese coste?"
                ),
            ]
        );
        assert_eq!(UNIFICADOR, "brainstorm-unificador");
        assert_eq!(
            CONDICIONALES[0].etiquetas,
            ["rendimiento", "wasm", "workers", "3d"]
        );
        assert_eq!(MIN_ROLES_PARA_CONDICIONALES, 2);
        // Ningun rol se deja una lista vacia.
        for r in todos() {
            assert!(
                !r.mira.is_empty() && !r.no_mira.is_empty() && !r.entregables.is_empty(),
                "{}",
                r.id
            );
        }
    }

    /// La seleccion: prefijo de la escalera; el condicional por etiqueta
    /// solo con 2 o mas; 5 son los cinco aunque no haya etiqueta.
    #[test]
    fn seleccion_de_roles() {
        let rend = etiquetas(&["UI", " WASM "]);
        for (n, e, esperado) in [
            (0, etiquetas(&[]), vec![]),
            (0, rend.clone(), vec![]),
            (1, rend.clone(), vec!["brainstorm-arquitectura"]),
            (
                2,
                etiquetas(&["ui"]),
                vec!["brainstorm-arquitectura", "brainstorm-riesgos"],
            ),
            (
                2,
                rend.clone(),
                vec![
                    "brainstorm-arquitectura",
                    "brainstorm-riesgos",
                    "brainstorm-rendimiento",
                ],
            ),
            (
                4,
                etiquetas(&[]),
                vec![
                    "brainstorm-arquitectura",
                    "brainstorm-riesgos",
                    "brainstorm-testing",
                    "brainstorm-dominio",
                ],
            ),
            (
                5,
                etiquetas(&[]),
                vec![
                    "brainstorm-arquitectura",
                    "brainstorm-riesgos",
                    "brainstorm-testing",
                    "brainstorm-dominio",
                    "brainstorm-rendimiento",
                ],
            ),
            (
                5,
                rend,
                vec![
                    "brainstorm-arquitectura",
                    "brainstorm-riesgos",
                    "brainstorm-testing",
                    "brainstorm-dominio",
                    "brainstorm-rendimiento",
                ],
            ),
        ] {
            assert_eq!(ids(&seleccionar(n, &e)), esperado, "n={n} {e:?}");
        }
    }
}
