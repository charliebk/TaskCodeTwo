//! Las plantillas de `plan` (F4-1): funciones puras de texto, portadas de
//! `src/core/plan-brainstorm.ts` del plugin 0.16.2 y del `planTemplate` de
//! `src/commands/plan.ts`, con la plantilla de `plan-final.md` de F1-9
//! (punto 7).
//!
//! El CLI no llama a ningun modelo: deja las peticiones escritas en la rama
//! de estado y quien orquesta lanza a los agentes. Lo que cambia frente a la
//! 0.16:
//!
//! - Ningun agente escribe en `tareas/` (con `tareas/` sucio la transaccion
//!   se rechaza): las salidas y el plan se publican con `tc2 plan guardar`,
//!   y las peticiones lo dicen.
//! - `taskctl approve` pasa a `tc2 approve`.
//! - Cada ronda nueva relanza sus roles (una ronda nueva solo se abre si
//!   cambia el juego de roles), asi que el bloque de re-planificacion no
//!   dice que las salidas sean de una ronda anterior.
//! - El bloque de complejidad dice tambien el carril, que puede subir el
//!   numero de roles por encima de la complejidad.

use super::heuristica::{Enunciado, Resolucion};
use super::roles::{ESCALERA, Rol, UNIFICADOR};
use crate::carril::Carril;
use crate::dato;

/// Carpeta de la planificacion dentro de la carpeta de la tarea.
pub const PLANIFICACION: &str = "planificacion";
/// Subcarpeta del brainstorm.
pub const BRAINSTORM: &str = "brainstorm";
/// Nombre del plan.
pub const PLAN_FINAL: &str = "plan-final.md";

/// Lo que comparten las plantillas de una ronda.
#[derive(Debug, Clone, Copy)]
pub struct Datos<'a> {
    /// `TASK-NNN`.
    pub id: &'a str,
    /// Titulo de la tarea.
    pub titulo: &'a str,
    /// Tipo de la tarea.
    pub tipo: &'a str,
    /// Complejidad declarada.
    pub complejidad: Option<&'a str>,
    /// Carril con el que se planifica.
    pub carril: Carril,
    /// Ronda.
    pub ronda: u32,
    /// `AAAA-MM-DD`.
    pub fecha: &'a str,
    /// Objetivo y criterios.
    pub enunciado: &'a Enunciado,
    /// Complejidad declarada frente a heuristica.
    pub resolucion: &'a Resolucion,
}

/// `peticion-<rol>-<ronda>.md` (el id ya lleva `brainstorm-`).
pub fn nombre_peticion_rol(rol: &Rol, ronda: u32) -> String {
    format!("peticion-{}-{ronda}.md", rol.id)
}

/// `salida-<rol>-<ronda>.md`: la respuesta del agente de ese rol.
pub fn nombre_salida_rol(rol: &Rol, ronda: u32) -> String {
    format!("salida-{}-{ronda}.md", rol.id)
}

/// `peticion-unificador-<ronda>.md`.
pub fn nombre_peticion_unificador(ronda: u32) -> String {
    format!("peticion-unificador-{ronda}.md")
}

/// `peticion-plan-<ronda>.md`: con un solo rol no hay unificador y esta
/// peticion va al propio rol (TASK-042 de la 0.16).
pub fn nombre_peticion_redaccion(ronda: u32) -> String {
    format!("peticion-plan-{ronda}.md")
}

/// `cambios-pedidos-<ronda>.md`: lo que la persona pide cambiar del plan
/// de esa ronda (`tc2 approve --pedir-cambios`). Es el testigo de que la
/// ronda esta cerrada: el siguiente `plan` abre la siguiente (D1a de
/// TASK-037).
pub fn nombre_cambios_pedidos(ronda: u32) -> String {
    format!("cambios-pedidos-{ronda}.md")
}

/// `TASK-NNN/planificacion/plan-final.md`.
pub fn ruta_plan_final(id: &str) -> String {
    format!("{id}/{PLANIFICACION}/{PLAN_FINAL}")
}

/// `TASK-NNN/planificacion/brainstorm/<nombre>`.
pub fn ruta_brainstorm(id: &str, nombre: &str) -> String {
    format!("{id}/{PLANIFICACION}/{BRAINSTORM}/{nombre}")
}

/// La orden que publica `destino` (relativo a `planificacion/`).
fn guardar(id: &str, destino: &str) -> String {
    format!("tc2 plan guardar {id} {destino} <fichero>")
}

/// «1 rol», «2 roles», «ningun rol».
fn plural(n: usize) -> String {
    match n {
        0 => "ningun rol".to_owned(),
        1 => "1 rol".to_owned(),
        n => format!("{n} roles"),
    }
}

/// Tope del titulo en la cabecera de las peticiones.
const TOPE_TITULO: usize = 200;

/// La cabecera de cada peticion. El titulo es texto de la persona (o de
/// quien escribio la tarea): va por [`dato::plano`], en una linea y sin
/// formato (F6-7).
fn cabecera(d: &Datos<'_>) -> String {
    format!(
        "- Tarea: {} — {}\n- Tipo: {} · Complejidad declarada: {}\n- Carril: {}\n- Ronda: {}\n- Fecha: {}\n",
        d.id,
        dato::plano(d.titulo, TOPE_TITULO),
        d.tipo,
        d.complejidad
            .unwrap_or("no declarada (decide la heuristica)"),
        d.carril.nombre(),
        d.ronda,
        d.fecha
    )
}

/// El enunciado en una cerca que no puede cerrar ([`dato::cerca`]): es texto
/// de la persona.
fn enunciado(e: &Enunciado) -> String {
    let criterios = if e.criterios.is_empty() {
        "(la tarea no declara criterios de aceptacion)".to_owned()
    } else {
        e.criterios.join("\n")
    };
    let f = dato::cerca(&[&e.objetivo, &criterios]);
    format!(
        "## Enunciado de la tarea\n\n### Objetivo\n\n{f}\n{}\n{f}\n\n### Criterios de aceptacion\n\n{f}\n{criterios}\n{f}\n",
        e.objetivo
    )
}

/// Declarada frente a heuristica, siempre (tambien cuando coinciden), y el
/// carril, que puede subir el numero.
fn bloque_complejidad(d: &Datos<'_>, roles: &[Rol]) -> String {
    let r = d.resolucion;
    let senales = if r.senales.is_empty() {
        "  (ninguna senal encontrada — el texto de la tarea no aporta ninguna)\n".to_owned()
    } else {
        r.senales
            .iter()
            .map(|s| format!("  - {}: {} (+{})\n", s.clave, s.detalle, s.puntos))
            .collect()
    };
    let veredicto = if r.discrepa() {
        format!(
            "Los dos niveles NO coinciden. Por complejidad se lanzan {}, que es el mayor de los dos numeros: infraestimar sale caro y sobreestimar es barato. La discrepancia se te senala aqui a proposito — puede significar que el enunciado de la tarea no refleja lo que de verdad cuesta.",
            plural(usize::from(r.roles))
        )
    } else {
        format!(
            "Los dos niveles coinciden. Por complejidad se lanzan {}.",
            plural(usize::from(r.roles))
        )
    };
    let tope = if r.tope_hotfix {
        "- Se aplico el tope de `hotfix`: un hotfix abrevia el brainstorm previo (nunca la revision posterior).\n"
    } else {
        ""
    };
    // Los condicionales (rendimiento) cuentan en el numero final aunque no
    // salgan de la escalera: se dice para que el numero cuadre.
    let condicionales: Vec<&str> = roles
        .iter()
        .filter(|x| !ESCALERA.iter().any(|e| e.id == x.id))
        .map(|x| x.titulo)
        .collect();
    let incluye = if condicionales.is_empty() {
        String::new()
    } else {
        format!(
            "; incluye el rol condicional {}, que entra por las etiquetas o por `politica_carriles`",
            condicionales.join(", ")
        )
    };
    format!(
        "## Complejidad: declarada frente a heuristica\n\n- Declarada en la tarea: **{}**\n- Heuristica ({} puntos): **{}**\n- Senales encontradas:\n{senales}{tope}\n{veredicto}\n\nNumero final con el carril `{}`: {}{incluye} (el carril y `politica_carriles` solo suben el numero).\n",
        r.declarado
            .as_deref()
            .unwrap_or("no declarada (decide la heuristica)"),
        r.puntos,
        r.heuristico,
        d.carril.nombre(),
        plural(roles.len())
    )
}

/// Bloque de una ronda 2 o posterior: puede haber un plan de una ronda
/// anterior (o solo el esqueleto, si nadie lo guardo) y los cambios que la
/// persona pidio sobre ella (`cambios-pedidos-<n-1>.md`, de
/// `tc2 approve --pedir-cambios`). Las plantillas no leen el estado, asi
/// que la ruta se da siempre, con «si existe».
fn replanificacion(d: &Datos<'_>) -> String {
    if d.ronda <= 1 {
        return String::new();
    }
    format!(
        "## Esto es una re-planificacion, no un primer pase\n\nPuede existir un plan de una ronda anterior en `{}` (si solo trae el esqueleto, nadie lo guardo todavia), y desde entonces ha cambiado el juego de roles (por ejemplo, subio el carril) o la persona pidio cambios sobre ese plan. **Leelo antes que nada.** Si existe `{}`, son los cambios que pidio la persona en la ronda {}: **atiendelos todos** y di en el plan como queda cada uno. Si trae un plan, incorpora lo nuevo al que ya hay en vez de replantearlo desde el enunciado como si fuera la primera vez; en cualquier caso **devuelve el plan COMPLETO**: tu respuesta sustituye al fichero entero.\n\nSi el enfoque entero esta mal, eso NO se arregla aqui: dilo en el plan y que alguien decida si se replantea.\n\n",
        ruta_plan_final(d.id),
        ruta_brainstorm(d.id, &nombre_cambios_pedidos(d.ronda - 1)),
        d.ronda - 1
    )
}

fn lista(xs: &[&str]) -> String {
    xs.iter().map(|x| format!("- {x}\n")).collect()
}

/// La peticion de un rol (con dos o mas roles, o en una ronda con
/// unificador).
pub fn peticion_rol(d: &Datos<'_>, rol: &Rol, otros: &[Rol]) -> String {
    let salida = nombre_salida_rol(rol, d.ronda);
    let companeros = if otros.is_empty() {
        "Eres el unico rol que se lanza en esta tarea.\n".to_owned()
    } else {
        let nombres: Vec<String> = otros.iter().map(|r| format!("**{}**", r.titulo)).collect();
        format!(
            "Trabajan en paralelo contigo, sin verte: {}. No cubras lo suyo — si lo haces, el unificador recibira el mismo punto de vista repetido y lo leera como confirmacion.\n",
            nombres.join(", ")
        )
    };
    format!(
        "# Peticion de brainstorm — {id}, rol {titulo} (ronda {ronda})\n\n{cab}- Rol: `{rid}` — lanzalo con el agente de ese mismo nombre\n- Vuelca tu respuesta en: `{salida}` (se publica con `{orden}`)\n\n## Tu pregunta\n\n> {pregunta}\n\n## Que miras\n\n{mira}\n## Que NO miras\n\nNo es una lista de cortesia. Si la ignoras y lo cubres todo \"por si acaso\", el acotado de contexto se deshace sin que se note y esta tarea deja de tener varios puntos de vista:\n\n{no_mira}\n{enun}\n## Como entregas\n\nEscribe en `{salida}`, con estas secciones y en este orden:\n\n{entregables}\n## Reglas\n\n- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es un documento.\n- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.\n- Prefiere lo concreto: rutas, nombres y casos reales por encima de recomendaciones genericas.\n- {companeros}",
        id = d.id,
        titulo = rol.titulo,
        ronda = d.ronda,
        cab = cabecera(d),
        rid = rol.id,
        orden = guardar(d.id, &format!("{BRAINSTORM}/{salida}")),
        pregunta = rol.pregunta,
        mira = lista(rol.mira),
        no_mira = lista(rol.no_mira),
        enun = enunciado(d.enunciado),
        entregables = lista(rol.entregables),
    )
}

/// Lo que tiene que traer el plan final.
fn que_trae(seccion_roles: &str) -> String {
    format!(
        "## Que tiene que traer el plan final\n\n- Enfoque propuesto, concreto: que se crea, que se extiende, en que orden.\n{seccion_roles}- Riesgos aceptados y que los contiene.\n- Plan de pruebas.\n- Lo que necesita decision de una persona antes de implementar. **El checkpoint humano es obligatorio**: este plan no vale hasta que alguien lo apruebe con `tc2 approve`.\n\n"
    )
}

/// La peticion de redaccion: un solo rol y sin unificador; su respuesta es
/// el plan.
pub fn peticion_redaccion(d: &Datos<'_>, rol: &Rol) -> String {
    format!(
        "# Peticion de redaccion del plan — {id} (ronda {ronda})\n\n{cab}- Rol: `{rid}` — lanzalo con el agente de ese mismo nombre\n- Quien orquesta publica tu respuesta con: `{orden}`\n\n**Esta tarea se planifica con un solo rol y no hay unificador**: tu respuesta ES el plan final. Redactala con las secciones de abajo; quien orquesta la publica tal cual en `plan-final.md` (tu no escribes ficheros), sin salida intermedia que consolidar.\n\n{replan}## Tu pregunta\n\n> {pregunta}\n\n## Que miras\n\n{mira}\n## Que NO miras\n\nQue lo mires solo tu no te autoriza a cubrirlo todo: lo que queda fuera de tu pregunta no se mira, pero se DICE. Anota en el plan lo que no cubres, para que quien lo apruebe sepa que ese lado no se ha revisado:\n\n{no_mira}\n{complejidad}\n{trae}## Reglas\n\n- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es el plan.\n- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.\n\n{enun}",
        id = d.id,
        ronda = d.ronda,
        cab = cabecera(d),
        rid = rol.id,
        orden = guardar(d.id, PLAN_FINAL),
        replan = replanificacion(d),
        pregunta = rol.pregunta,
        mira = lista(rol.mira),
        no_mira = lista(rol.no_mira),
        complejidad = bloque_complejidad(d, std::slice::from_ref(rol)),
        trae = que_trae(
            "- Lo que el rol no cubrio, contrastado contra los criterios de aceptacion.\n"
        ),
        enun = enunciado(d.enunciado),
    )
}

/// La peticion al unificador: consolida las salidas de los `roles` de esta
/// ronda (con cero roles, redacta desde el enunciado).
pub fn peticion_unificador(d: &Datos<'_>, roles: &[Rol]) -> String {
    let salidas: String = if roles.is_empty() {
        "(ninguna: esta tarea no lanza brainstorm, ver el bloque de complejidad)\n".to_owned()
    } else {
        roles
            .iter()
            .map(|r| {
                format!(
                    "- `{}` — {}\n",
                    ruta_brainstorm(d.id, &nombre_salida_rol(r, d.ronda)),
                    r.titulo
                )
            })
            .collect()
    };
    let como = match roles.len() {
        0 => "No hay salidas de brainstorm que consolidar: redacta el plan directamente a partir del enunciado. Deja dicho en el plan que se redacto sin brainstorm y por que (el numero de roles sale del carril y del lookup de complejidad, no de un descuido).\n".to_owned(),
        1 => "**Esta tarea se planifico con un solo rol**, asi que aqui no hay desacuerdos que resolver: no te los inventes ni trates la ausencia de discrepancia como una senal de nada.\n\n1. Lee la salida de arriba. Si falta o esta sin rellenar, dilo en el plan en vez de suplirla en silencio.\n2. **Contrasta esa propuesta contra el enunciado de la tarea**, que tienes al final. Lo util que puedes aportar aqui no es mediar entre puntos de vista, es senalar QUE QUEDO SIN CUBRIR: criterios de aceptacion que el rol no toca, riesgos que no mira porque no era su papel, decisiones que da por hechas.\n3. Cada afirmacion del plan que venga del rol se atribuye a el; lo que anadas tu, tambien.\n".to_owned(),
        _ => "1. Lee las salidas de arriba. **Si alguna falta o esta sin rellenar, sigue adelante con las que haya y escribe en el plan cual falto**: un plan con un punto de vista menos, dicho, vale mas que un plan que finge estar completo.\n2. **No promedies.** Donde dos roles discrepen, el plan dice quien propone que, cual gana y por que. Un desacuerdo resuelto con una frase intermedia que no defiende nadie es la peor salida posible de este paso.\n3. **Si no discrepan en nada, eso es la alarma, no la nota de calidad**: significa que los roles recibieron el mismo contexto o que alguno no hizo su trabajo. Dilo en el plan.\n4. Cada afirmacion del plan que venga de un rol se atribuye a ese rol.\n".to_owned(),
    };
    let seccion = match roles.len() {
        0 => "",
        1 => "- Lo que el rol no cubrio, contrastado contra los criterios de aceptacion.\n",
        _ => "- Los desacuerdos entre roles y como se resuelve cada uno.\n",
    };
    format!(
        "# Peticion al unificador — {id} (ronda {ronda})\n\n{cab}- Rol: `{UNIFICADOR}` — lanzalo con el agente de ese mismo nombre\n- Vuelca el resultado con: `{orden}`\n\n{replan}## Salidas que tienes que consolidar\n\n{salidas}\n{complejidad}\n## Como consolidas\n\n{como}\n{trae}{enun}",
        id = d.id,
        ronda = d.ronda,
        cab = cabecera(d),
        orden = guardar(d.id, PLAN_FINAL),
        replan = replanificacion(d),
        complejidad = bloque_complejidad(d, roles),
        trae = que_trae(seccion),
        enun = enunciado(d.enunciado),
    )
}

/// El esqueleto de `plan-final.md` (F1-9, punto 7): los titulos de la 0.16
/// en su orden, la cabecera de origen en una linea, `Duracion estimada: N
/// min`, «Desacuerdos» con dos o mas roles o «Lo que el rol no cubrio» con
/// uno (nunca las dos) y `## Troceo` opcional.
pub fn plan_final(id: &str, titulo: &str, roles: &[Rol]) -> String {
    let titulos: Vec<&str> = roles.iter().map(|r| r.titulo).collect();
    let (origen, seccion) = match roles.len() {
        0 => (
            "(Sin brainstorm: el carril y la complejidad resuelven cero roles; el plan se redacta desde el enunciado.)".to_owned(),
            "",
        ),
        1 => (
            format!(
                "(Respuesta del unico rol, {}, publicada con tc2 plan guardar; sin unificador: el plan senala lo que el rol no cubrio.)",
                titulos.join(", ")
            ),
            "## Lo que el rol no cubrio\n\n\n",
        ),
        n => (
            format!(
                "(Lo consolida el unificador a partir de {n} roles: {}; los desacuerdos se senalan, no se promedian.)",
                titulos.join(", ")
            ),
            "## Desacuerdos entre roles, y como se resuelven\n\n\n",
        ),
    };
    format!(
        "# Plan — {id}: {titulo}\n\n{origen}\n\n## Enfoque propuesto\n\nDuracion estimada: N min\n\n\n{seccion}## Riesgos aceptados y que los contiene\n\n\n## Plan de pruebas\n\nTests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):\n\n\n## Lo que necesita decision de una persona\n\n\n## Troceo\n\n"
    )
}

#[cfg(test)]
mod tests {
    use super::super::heuristica;
    use super::super::roles::{ESCALERA, seleccionar};
    use super::*;

    fn datos<'a>(e: &'a Enunciado, r: &'a Resolucion, ronda: u32) -> Datos<'a> {
        Datos {
            id: "TASK-007",
            titulo: "Siete",
            tipo: "feature",
            complejidad: None,
            carril: Carril::Funcionalidad,
            ronda,
            fecha: "2026-11-18",
            enunciado: e,
            resolucion: r,
        }
    }

    #[test]
    fn nombres_de_fichero_de_la_016() {
        let a = &ESCALERA[0];
        assert_eq!(
            nombre_peticion_rol(a, 2),
            "peticion-brainstorm-arquitectura-2.md"
        );
        assert_eq!(
            nombre_salida_rol(a, 1),
            "salida-brainstorm-arquitectura-1.md"
        );
        assert_eq!(nombre_peticion_unificador(3), "peticion-unificador-3.md");
        assert_eq!(nombre_peticion_redaccion(1), "peticion-plan-1.md");
        assert_eq!(
            ruta_plan_final("TASK-001"),
            "TASK-001/planificacion/plan-final.md"
        );
    }

    /// Las peticiones llevan la pregunta literal, el «que no miras», la
    /// orden de guardar y `tc2 approve`.
    #[test]
    fn peticiones_con_lo_suyo() {
        let e = Enunciado {
            objetivo: "Hacer ```x```".into(),
            criterios: vec!["uno".into()],
        };
        let r = heuristica::resolver(None, &[], "feature", &[], &e);
        let d = datos(&e, &r, 1);
        let roles = seleccionar(2, &[]);
        let p = peticion_rol(&d, &roles[0], &roles[1..]);
        assert!(p.contains("> Dado lo que ya existe, ¿donde encaja"), "{p}");
        assert!(p.contains("Trabajan en paralelo contigo, sin verte: **riesgos**."));
        assert!(p.contains(
            "`tc2 plan guardar TASK-007 brainstorm/salida-brainstorm-arquitectura-1.md <fichero>`"
        ));
        let u = peticion_unificador(&d, &roles);
        assert!(u.contains("`brainstorm-unificador`") && u.contains("tc2 approve"));
        assert!(u.contains(
            "- `TASK-007/planificacion/brainstorm/salida-brainstorm-riesgos-1.md` — riesgos"
        ));
        assert!(!u.contains("taskctl") && !u.contains("re-planificacion"));
        let red = peticion_redaccion(&datos(&e, &r, 2), &roles[0]);
        assert!(
            red.contains("re-planificacion")
                && red.contains("tc2 plan guardar TASK-007 plan-final.md")
        );
        assert!(red.contains("Numero final con el carril `funcionalidad`: 1 rol ("));
        // La ronda 2 no da por hecho un plan guardado.
        assert!(red.contains("Puede existir un plan de una ronda anterior"));
        assert!(!red.contains("Ya existe un plan"));
        // Con el condicional, el numero final lo dice.
        let cinco = seleccionar(4, &["wasm".to_owned()]);
        let u = peticion_unificador(&d, &cinco);
        assert!(
            u.contains(
                "Numero final con el carril `funcionalidad`: 5 roles; incluye el rol condicional rendimiento,"
            ),
            "{u}"
        );
    }

    /// MEN-2 de TASK-037: la peticion de la ronda n+1 (redaccion y
    /// unificador) cita `cambios-pedidos-<n>.md`; la de la ronda 1, no.
    #[test]
    fn la_ronda_siguiente_cita_los_cambios_pedidos() {
        let e = Enunciado {
            objetivo: "Hacer algo".into(),
            criterios: vec!["uno".into()],
        };
        let r = heuristica::resolver(None, &[], "feature", &[], &e);
        let roles = seleccionar(2, &[]);
        let ruta = "`TASK-007/planificacion/brainstorm/cambios-pedidos-2.md`";
        let red = peticion_redaccion(&datos(&e, &r, 3), &roles[0]);
        let uni = peticion_unificador(&datos(&e, &r, 3), &roles);
        for p in [&red, &uni] {
            assert!(
                p.contains(&format!(
                    "Si existe {ruta}, son los cambios que pidio la persona en la ronda 2"
                )),
                "{p}"
            );
        }
        let primera = peticion_redaccion(&datos(&e, &r, 1), &roles[0]);
        assert!(!primera.contains("cambios-pedidos"), "{primera}");
    }

    /// El esqueleto: «Lo que el rol no cubrio» con uno, «Desacuerdos» con
    /// dos o mas, ninguna de las dos con cero.
    #[test]
    fn esqueleto_segun_los_roles() {
        let uno = plan_final("TASK-001", "Uno", &seleccionar(1, &[]));
        assert!(
            uno.starts_with("# Plan — TASK-001: Uno\n\n(Respuesta del unico rol, arquitectura,")
        );
        assert!(uno.contains("## Lo que el rol no cubrio") && !uno.contains("Desacuerdos"));
        assert!(uno.contains("\nDuracion estimada: N min\n"));
        let cuatro = plan_final("TASK-001", "Uno", &seleccionar(4, &[]));
        assert!(cuatro.contains("## Desacuerdos entre roles") && !cuatro.contains("no cubrio"));
        assert!(cuatro.contains("a partir de 4 roles: arquitectura, riesgos, testing, dominio;"));
        let cero = plan_final("TASK-001", "Uno", &[]);
        assert!(!cero.contains("Desacuerdos") && !cero.contains("no cubrio"));
        for p in [&uno, &cuatro, &cero] {
            assert!(p.ends_with("## Troceo\n\n"), "{p}");
        }
    }
}
