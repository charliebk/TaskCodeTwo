//! `tc2 roadmap` (F3-4): la vista para todos, en `ROADMAP.md` de la rama de
//! estado (ADR F1-1, decision 3).
//!
//! - [`de_estado`] genera el `ROADMAP.md` con el mismo calculo que `next`
//!   ([`crate::situacion`]) y la zona manual del que ya hay, byte a byte; si
//!   esa zona esta rota, devuelve el anterior tal cual (D3: no se bloquea la
//!   operacion; `next` y `--comparar` lo avisan). Es el generador que recibe
//!   cada transaccion de estado ([`crate::estado::Contexto::nuevo`]): se
//!   regenera, nunca se mergea (ADR F1-2, D2.4).
//! - La orden `roadmap [--comparar | --escribir] [--json]` (D2): sin
//!   opciones imprime el ROADMAP que toca sobre el estado que conoce el clon
//!   (solo lee); `--comparar` dice si lo publicado esta al dia (`ok`) o no
//!   (`rechazada motivo=desfasado|roadmap-roto`), sin escribir; `--escribir`
//!   lo publica como una transaccion mas ([`Regenerar`]).
//! - La zona manual se edita en `tareas/ROADMAP.md` y se publica con
//!   `--escribir` (D4): si el fichero de la carpeta montada tiene cambios, se
//!   prepara (`git add`) y su zona es la que se publica, byte a byte; tras
//!   publicar, la carpeta se adelanta a lo publicado.

pub mod generar;
pub mod mermaid;
pub mod zona;

use std::path::{Path, PathBuf};

use crate::Salida;
use crate::contrato::{Informe, Resultado, Valor, corto};
use crate::estado::{self, CARPETA, Cambio, Catalogo, Estado, Fallo, Operacion, RAMA};
use crate::git::{Git, Subcomando, correr, exigir, localizar_principal, worktrees};
use crate::i18n::catalogo::cli as txt_cli;
use crate::i18n::catalogo::roadmap as txt;
use crate::i18n::{Idioma, Mensaje};
use crate::plataforma::Personas;
use crate::situacion::{self, ROADMAP, Situacion};

/// Nombre de la orden.
pub const NOMBRE: &str = "roadmap";

/// Lista de personas de la base para convertir los `asignado_a` heredados
/// (D6); sin lista, ninguna: se muestran tal cual.
pub fn personas(g: &Git, dir: &Path) -> Personas {
    crate::identidad::personas_de_base(g, dir).unwrap_or_default()
}

/// El generador que recibe cada transaccion de estado.
pub fn generador(personas: &Personas) -> impl Fn(&dyn Estado) -> Result<String, Fallo> + '_ {
    move |e: &dyn Estado| de_estado(e, personas)
}

/// El `ROADMAP.md` de `estado`: generado alrededor de su zona manual (o la
/// de por defecto), o el que hay tal cual si su zona esta rota (D3).
pub fn de_estado(estado: &dyn Estado, personas: &Personas) -> Result<String, Fallo> {
    let s = situacion::calcular(estado)?;
    Ok(de_situacion(&s, personas).unwrap_or_else(|| s.roadmap.clone().unwrap_or_default()))
}

/// El ROADMAP que toca para `s`, o `None` si su zona manual esta rota.
fn de_situacion(s: &Situacion, personas: &Personas) -> Option<String> {
    let zona = match s.roadmap.as_deref().map(zona::extraer) {
        None | Some(Ok(None)) => zona::POR_DEFECTO,
        Some(Ok(Some(z))) => z,
        Some(Err(_)) => return None,
    };
    Some(generar::generar(s, zona, personas))
}

fn roto(donde: &str, motivo: &str) -> Fallo {
    Fallo::rechazada(
        "roadmap-roto",
        Mensaje::nuevo(&txt::ZONA_ROTA)
            .con("donde", donde)
            .con("motivo", motivo),
    )
}

/// `roadmap --escribir` como operacion de la rama de estado: no cambia nada
/// salvo, si la persona edito la zona manual en `tareas/`, el `ROADMAP.md`
/// con esa zona; el generador de la transaccion hace el resto. Exige red
/// (D2b de F1-2): sin red no queda pendiente.
#[derive(Debug, Clone, Default)]
pub struct Regenerar {
    /// `tareas/ROADMAP.md` preparado por la persona, con su zona valida.
    editado: Option<String>,
}

impl Regenerar {
    /// Sin edicion de la persona.
    pub fn nueva() -> Self {
        Self::default()
    }

    /// Con el `ROADMAP.md` editado por la persona (su zona ya validada).
    pub fn con_editado(texto: String) -> Self {
        Self {
            editado: Some(texto),
        }
    }
}

impl Operacion for Regenerar {
    fn nombre(&self) -> &str {
        NOMBRE
    }
    fn anotacion(&self) -> String {
        format!("{NOMBRE}|escribir")
    }
    fn exige_red(&self) -> bool {
        true
    }
    fn aplicar(&self, e: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        let actual = e.leer(ROADMAP)?;
        match &self.editado {
            Some(t) if actual.as_deref() == Some(t.as_str()) => Ok(Vec::new()),
            Some(t) => Ok(vec![Cambio::nuevo(ROADMAP, t.clone())]),
            None => match actual.as_deref().map(zona::extraer) {
                Some(Err(r)) => Err(roto(ROADMAP, r.motivo())),
                _ => Ok(Vec::new()),
            },
        }
    }
}

/// Reconstruye [`Regenerar`] desde su anotacion (nunca queda pendiente:
/// exige red).
pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
    (a == format!("{NOMBRE}|escribir")).then(|| Box::new(Regenerar::nueva()) as Box<dyn Operacion>)
}

/// Que hace la orden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modo {
    /// Imprimir el ROADMAP que toca (solo lee).
    Imprimir,
    /// Decir si lo publicado esta al dia (solo lee).
    Comparar,
    /// Publicarlo.
    Escribir,
}

/// `roadmap [--comparar | --escribir]` (sin `--json`, que quita `lib.rs`)
/// en la carpeta actual; `catalogo` re-ejecuta las pendientes.
pub fn orden(args: &[String], json: bool, idioma: Idioma, catalogo: Catalogo<'_>) -> Salida {
    let modo = match args {
        [] => Modo::Imprimir,
        [a] if a == "--comparar" => Modo::Comparar,
        [a] if a == "--escribir" => Modo::Escribir,
        _ => {
            return Informe::nuevo(NOMBRE, Resultado::Uso)
                .con("motivo", "argumentos")
                .diagnostico(&txt::USO_ROADMAP)
                .salida_en(json, idioma);
        }
    };
    let dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return Informe::nuevo(NOMBRE, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt_cli::CARPETA_ACTUAL).con("error", e.to_string()))
                .salida_en(json, idioma);
        }
    };
    let g = Git::nuevo();
    match modo {
        Modo::Imprimir => match imprimir_en(&g, &dir) {
            Ok((texto, _)) if !json => Salida {
                codigo: 0,
                stdout: texto,
                stderr: String::new(),
            },
            Ok((texto, commit)) => Informe::nuevo(NOMBRE, Resultado::Ok)
                .con("commit", commit)
                .con("texto", texto)
                .salida_en(true, idioma),
            Err(i) => i.salida_en(json, idioma),
        },
        Modo::Comparar => comparar_en(&g, &dir).salida_en(json, idioma),
        Modo::Escribir => escribir_en(&g, &dir, catalogo).salida_en(json, idioma),
    }
}

fn renombrado(mut i: Informe) -> Informe {
    NOMBRE.clone_into(&mut i.comando);
    i
}

fn de_fallo(f: Fallo) -> Informe {
    match f {
        Fallo::Rechazada {
            motivo,
            diagnostico,
        } => Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", motivo)
            .diagnostico(diagnostico),
        Fallo::Error(d) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
    }
}

/// La situacion del estado que conoce el clon (sin `fetch`) y su commit.
fn situacion_en(g: &Git, dir: &Path) -> Result<(Situacion, String), Informe> {
    let vista = estado::leer(g, dir).map_err(renombrado)?;
    let commit = vista.commit().map(corto).unwrap_or_default();
    let s = situacion::calcular(&vista).map_err(de_fallo)?;
    Ok((s, commit))
}

/// El ROADMAP que toca sobre el estado que conoce el clon, y el commit de
/// ese estado. Con la zona rota, `rechazada motivo=roadmap-roto`.
pub fn imprimir_en(g: &Git, dir: &Path) -> Result<(String, String), Informe> {
    let (s, commit) = situacion_en(g, dir)?;
    match de_situacion(&s, &personas(g, dir)) {
        Some(t) => Ok((t, commit)),
        None => Err(zona_rota(&s, &commit)),
    }
}

fn zona_rota(s: &Situacion, commit: &str) -> Informe {
    let motivo = s.roadmap_roto().map_or("", zona::Rota::motivo);
    de_fallo(roto(ROADMAP, motivo))
        .con("roto", motivo)
        .con("commit", commit)
}

/// `--comparar`: `ok` si el `ROADMAP.md` publicado es el que toca;
/// `rechazada motivo=desfasado` si no; `rechazada motivo=roadmap-roto` si
/// su zona esta rota (D3). No escribe nada.
pub fn comparar_en(g: &Git, dir: &Path) -> Informe {
    let (s, commit) = match situacion_en(g, dir) {
        Ok(x) => x,
        Err(i) => return i,
    };
    match de_situacion(&s, &personas(g, dir)) {
        None => zona_rota(&s, &commit),
        Some(t) if s.roadmap.as_deref() == Some(t.as_str()) => {
            Informe::nuevo(NOMBRE, Resultado::Ok).con("commit", commit)
        }
        Some(_) => Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", "desfasado")
            .con("commit", commit)
            .diagnostico(&txt::DESFASADO),
    }
}

/// `tareas/` montada (la carpeta del worktree de la rama de estado).
fn carpeta_montada(g: &Git, dir: &Path) -> Result<Option<PathBuf>, Informe> {
    let principal = localizar_principal(g, dir).map_err(renombrado)?;
    Ok(worktrees(g, &principal, NOMBRE)?
        .into_iter()
        .find(|w| w.rama.as_deref() == Some(RAMA) && w.ruta.exists())
        .map(|w| w.ruta))
}

/// Si `tareas/ROADMAP.md` tiene cambios, los prepara y devuelve lo
/// preparado (normalizado como lo guardaria git), con su zona validada.
fn editado_en(g: &Git, carpeta: &Path) -> Result<Option<String>, Informe> {
    let s = exigir(
        g,
        carpeta,
        NOMBRE,
        Subcomando::Status,
        &["--porcelain", "--", ROADMAP],
    )?;
    if s.trim().is_empty() {
        return Ok(None);
    }
    exigir(g, carpeta, NOMBRE, Subcomando::Add, &["--", ROADMAP])?;
    let texto = crate::git::estado::leer_blob(g, carpeta, NOMBRE, "", ROADMAP)?;
    let donde = format!("{CARPETA}/{ROADMAP}");
    match zona::extraer(&texto) {
        Ok(Some(_)) => Ok(Some(texto)),
        Ok(None) => Err(de_fallo(roto(&donde, "sin-zona")).con("roto", "sin-zona")),
        Err(r) => Err(de_fallo(roto(&donde, r.motivo())).con("roto", r.motivo())),
    }
}

/// `--escribir`: publica el ROADMAP que toca como una transaccion (D1).
pub fn escribir_en(g: &Git, dir: &Path, catalogo: Catalogo<'_>) -> Informe {
    let carpeta = match carpeta_montada(g, dir) {
        Ok(c) => c,
        Err(i) => return renombrado(i),
    };
    let editado = match carpeta.as_deref().map(|c| editado_en(g, c)).transpose() {
        Ok(e) => e.flatten(),
        Err(i) => return renombrado(i),
    };
    let op = match &editado {
        Some(t) => Regenerar::con_editado(t.clone()),
        None => Regenerar::nueva(),
    };
    let personas = personas(g, dir);
    let generar = generador(&personas);
    let c = estado::Contexto::nuevo(g, dir, catalogo, Some(&generar));
    let mut i = renombrado(estado::transaccion(&c, &op));
    if editado.is_some()
        && i.resultado == Resultado::Ok
        && i.valor("tareas") == Some("con-cambios")
        && let (Some(carpeta), Some(commit)) = (carpeta, i.valor("commit").map(str::to_owned))
        && adelantar_editada(g, &carpeta, &commit)
        && let Some(v) = i.claves.iter_mut().find(|(k, _)| k == "tareas")
    {
        v.1 = Valor::Texto("al-dia".to_owned());
    }
    i
}

/// Tras publicar la zona editada, deja `tareas/ROADMAP.md` como lo
/// publicado (que lleva esa zona byte a byte), lo prepara y adelanta la
/// carpeta: sin esto, el cambio ya publicado la dejaria atascada. Si algo
/// falla, la carpeta queda como estaba (con `tareas=con-cambios`).
fn adelantar_editada(g: &Git, carpeta: &Path, commit: &str) -> bool {
    let Ok(publicado) = crate::git::estado::leer_blob(g, carpeta, NOMBRE, commit, ROADMAP) else {
        return false;
    };
    std::fs::write(carpeta.join(ROADMAP), publicado.as_bytes()).is_ok()
        && exigir(g, carpeta, NOMBRE, Subcomando::Add, &["--", ROADMAP]).is_ok()
        && correr(
            g,
            carpeta,
            NOMBRE,
            Subcomando::Merge,
            &["-q", "--ff-only", commit],
        )
        .is_ok_and(|s| s.exito())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use crate::tarea::{Asignado, Cuerpo, Tarea};

    /// Estado en memoria.
    struct EnMemoria(BTreeMap<String, String>);

    impl Estado for EnMemoria {
        fn rutas(&self) -> Vec<String> {
            self.0.keys().cloned().collect()
        }
        fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo> {
            Ok(self.0.get(ruta).cloned())
        }
    }

    fn v2(
        id: &str,
        titulo: &str,
        sprint: u32,
        estado: &str,
        deps: &[&str],
        asignado: Option<&str>,
    ) -> String {
        let mut t = Tarea::nueva(id, titulo, "2026-01-01");
        t.sprint = sprint;
        t.estado = estado.to_owned();
        if estado == "descartada" {
            t.motivo = Some("ya no hace falta".to_owned());
        }
        t.dependencias = deps.iter().map(|d| (*d).to_owned()).collect();
        if let Some(login) = asignado {
            t.asignado_a = Asignado::persona(7, login);
        }
        t.escribir(&Cuerpo::leer(""))
    }

    /// Formato 1 (0.16), con `asignado_a` de texto libre.
    fn v1(id: &str, titulo: &str, sprint: u32, estado: &str, asignado: &str) -> String {
        format!(
            "---\nid: {id}\ntitulo: \"{titulo}\"\ntipo: feature\nsprint: {sprint}\netiquetas: []\n\
             complejidad: null\nmodelo_sugerido: sonnet\nestado: {estado}\nplan_aprobado: false\n\
             rama: feature/x\nasignado_a: {asignado}\nagente_revisor: general-purpose\n\
             skills_recomendados: []\nregla_seleccion_skill: null\nultimo_commit_revisado: null\n\
             revision_codex: false\ntokens_diseno: null\ntokens_implementacion: null\n\
             tokens_revision: null\ncreado: 2026-01-01\nactualizado: 2026-01-01\n\
             dependencias: []\n---\n"
        )
    }

    const ZONA: &str = "<!-- manual:inicio (a mano) -->\r\n## Hitos\r\n\r\n### H2 — Dos\r\nEl motivo.\r\n### H1 — Uno\r\n### H3 — Tres\r\n<!-- manual:fin -->\r\n";

    /// Un estado que pasa por cada seccion.
    fn estado_de_referencia() -> EnMemoria {
        let mut m = BTreeMap::new();
        for (id, md) in [
            (
                "TASK-001",
                v2("TASK-001", "Base", 1, "terminada", &[], None),
            ),
            (
                "TASK-002",
                v2(
                    "TASK-002",
                    "Motor",
                    1,
                    "en-curso",
                    &["TASK-001"],
                    Some("ana"),
                ),
            ),
            (
                "TASK-003",
                v1("TASK-003", "Revisar", 1, "en-revision", "ana@x.com"),
            ),
            (
                "TASK-004",
                v2(
                    "TASK-004",
                    "Usa el motor",
                    2,
                    "planificada",
                    &["TASK-002"],
                    None,
                ),
            ),
            (
                "TASK-005",
                v2("TASK-005", "Lista", 2, "planificada", &["TASK-001"], None),
            ),
            (
                "TASK-006",
                v2("TASK-006", "Ya no", 2, "descartada", &[], None),
            ),
            (
                "TASK-007",
                v2(
                    "TASK-007",
                    "Tras la descartada",
                    2,
                    "planificada",
                    &["TASK-006", "TASK-099"],
                    None,
                ),
            ),
            (
                "TASK-008",
                v1(
                    "TASK-008",
                    "Con a | b <!-- y \\\"comillas\\\" #1",
                    2,
                    "en-diseno",
                    "alguien de fuera",
                ),
            ),
            (
                "TASK-009",
                v2("TASK-009", "Cerrada", 3, "terminada", &[], None),
            ),
            ("TASK-010", "---\nid: roto\n".to_owned()),
            (
                "TASK-011",
                v2("TASK-011", "Ciclo a", 2, "planificada", &["TASK-012"], None),
            ),
            (
                "TASK-012",
                v2(
                    "TASK-012",
                    "Ciclo b",
                    2,
                    "planificada",
                    &["TASK-011", "TASK-010"],
                    None,
                ),
            ),
        ] {
            m.insert(format!("{id}/tarea.md"), md);
        }
        m.insert(
            situacion::ROADMAP.to_owned(),
            format!("# Viejo\n\n{ZONA}\n## Lo generado antes\n"),
        );
        EnMemoria(m)
    }

    fn personas() -> Personas {
        Personas::leer("personas:\n  - {id: 9, login: \"ana-x\", correos: [\"ana@x.com\"]}\n")
            .unwrap_or_default()
    }

    const REFERENCIA: &str = include_str!("../../tests/fixtures/roadmap/referencia.md");

    /// C5: la salida de referencia cubre cada seccion (con quien, revision,
    /// hito con dependencias y Mermaid, terminadas y descartadas); C2: la
    /// zona manual, con CRLF, byte a byte. Para rehacer la referencia:
    /// `TC2_REESCRIBIR=1 cargo test -p tc2 --lib referencia`.
    #[test]
    fn salida_de_referencia() {
        let r = de_estado(&estado_de_referencia(), &personas());
        assert!(r.is_ok(), "{r:?}");
        let Ok(r) = r else { return };
        if std::env::var_os("TC2_REESCRIBIR").is_some() {
            let ruta = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/roadmap/referencia.md"
            );
            assert!(std::fs::write(ruta, &r).is_ok());
            return;
        }
        assert_eq!(r, REFERENCIA);
        assert_eq!(zona::extraer(&r), Ok(Some(ZONA)));
        for seccion in [
            "\n## Estado del trabajo\n",
            "\n## En curso\n",
            "\n## En revisión\n",
            "\n## H2 — Dos\n",
            "\n## H1 — Uno\n",
            "\n## Terminadas\n",
            "\n## Descartadas\n",
            "```mermaid\n",
            "| @ana |",
            "| @ana-x |",
            "<details>",
        ] {
            assert!(r.contains(seccion), "falta {seccion:?}");
        }
        assert!(!r.contains("\n## H3"), "un hito cerrado no lleva apartado");
        // Mismos datos, mismos bytes (lo que da `sin-cambios`).
        let mut otra = estado_de_referencia();
        otra.0.insert(situacion::ROADMAP.to_owned(), r.clone());
        assert_eq!(de_estado(&otra, &personas()).ok(), Some(r));
    }

    /// D3: con la zona rota se devuelve el anterior tal cual.
    #[test]
    fn zona_rota_deja_el_anterior() {
        let mut e = estado_de_referencia();
        let roto = "# Roadmap\n<!-- manual:inicio -->\n### H1 — Uno\n".to_owned();
        e.0.insert(situacion::ROADMAP.to_owned(), roto.clone());
        assert_eq!(de_estado(&e, &Personas::default()).ok(), Some(roto));
    }

    /// Sin ROADMAP.md (o sin zona), la zona de por defecto.
    #[test]
    fn sin_roadmap_la_zona_por_defecto() {
        let mut e = estado_de_referencia();
        e.0.remove(situacion::ROADMAP);
        let r = de_estado(&e, &Personas::default()).unwrap_or_default();
        assert_eq!(zona::extraer(&r), Ok(Some(zona::POR_DEFECTO)));
        assert!(r.starts_with(&format!("# Roadmap de tareas\n\n{}\n\n", generar::MARCA)));
    }
}
