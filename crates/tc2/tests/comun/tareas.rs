//! `tarea.md` y juegos de tareas de prueba para publicar en la rama de
//! estado.

use tc2::estado::ROADMAP;
use tc2::tarea::{Cuerpo, Tarea};

/// 2026-10-01T09:00:00Z.
pub const BASE: u64 = 1_790_845_200;

/// Cabecera de la tabla de transiciones de un `tarea.md`.
pub const CABECERA: &str =
    "## Transiciones\n\n| fecha | fase | modo | decidido_por | version |\n|---|---|---|---|---|\n";

/// Fila de transiciones de `fase` a las `ts`.
pub fn fila(ts: u64, fase: &str) -> String {
    format!(
        "| {} | {fase} | manual | persona | 0.1 |\n",
        tc2::fecha::instante(ts)
    )
}

/// Una terminada («Hecha {id}», plan aprobado, con `criterios` marcados y
/// lo que ponga `ajustar`) con `imp` minutos de implementacion (sin huecos
/// de mas de 45), 10 de rondas y 5 de cierre, el dia `dia` desde [`BASE`]:
/// su ruta y su texto.
pub fn terminada(
    id: &str,
    dia: u64,
    imp: u64,
    criterios: &str,
    ajustar: impl FnOnce(&mut Tarea),
) -> (String, String) {
    let mut t = Tarea::nueva(id, &format!("Hecha {id}"), "2026-10-01");
    "terminada".clone_into(&mut t.estado);
    ajustar(&mut t);
    t.plan_aprobado = true;
    let s = BASE + dia * 86_400;
    let r1 = s + imp * 60;
    let cuerpo = format!(
        "## Objetivo\n\nHecha.\n\n## Criterios de aceptacion\n\n{criterios}\n\n{CABECERA}{}{}{}{}",
        fila(s, "start"),
        fila(r1, "review"),
        fila(r1 + 600, "review"),
        fila(r1 + 900, "finish"),
    );
    (format!("{id}/tarea.md"), t.escribir(&Cuerpo::leer(&cuerpo)))
}

/// `tarea.md` de `id` («Titulo de {id}») en `sprint` y `estado`, con sus
/// dependencias; una descartada lleva `motivo` (sin el no se lee).
pub fn tarea(id: &str, sprint: u32, estado: &str, deps: &[&str]) -> String {
    let mut t = Tarea::nueva(id, &format!("Titulo de {id}"), "2026-01-01");
    t.sprint = sprint;
    t.estado = estado.to_owned();
    if estado == "descartada" {
        t.motivo = Some("ya no hace falta".to_owned());
    }
    t.dependencias = deps.iter().map(|d| (*d).to_owned()).collect();
    t.escribir(&Cuerpo::leer(""))
}

/// Juego de tareas de `next` (rutas y textos): H2 va antes que H1 en la
/// zona manual del ROADMAP; TASK-005 es ilegible y bloquea a TASK-010;
/// TASK-007 esta descartada y bloquea a TASK-008.
pub fn juego_de_next() -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = [
        ("TASK-001", tarea("TASK-001", 1, "terminada", &[])),
        (
            "TASK-002",
            tarea("TASK-002", 1, "planificada", &["TASK-001"]),
        ),
        ("TASK-003", tarea("TASK-003", 1, "en-curso", &[])),
        (
            "TASK-004",
            tarea("TASK-004", 2, "planificada", &["TASK-003"]),
        ),
        ("TASK-005", "---\nid: TASK-005\nroto: [\n---\n".to_owned()),
        ("TASK-006", tarea("TASK-006", 2, "planificada", &[])),
        ("TASK-007", tarea("TASK-007", 2, "descartada", &[])),
        (
            "TASK-008",
            tarea("TASK-008", 2, "planificada", &["TASK-007"]),
        ),
        (
            "TASK-010",
            tarea("TASK-010", 2, "planificada", &["TASK-005"]),
        ),
    ]
    .into_iter()
    .map(|(id, md)| (format!("{id}/tarea.md"), md))
    .collect();
    v.push((
        ROADMAP.to_owned(),
        "# Roadmap\n\n<!-- manual:inicio -->\n### H2 — Antes\n### H1 — Despues\n<!-- manual:fin -->\n"
            .to_owned(),
    ));
    v
}
