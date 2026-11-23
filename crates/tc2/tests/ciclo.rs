//! F3-7 (TASK-034): `tc2 descartar`, `tc2 pausa` y `tc2 reanudar` con el
//! binario real, como los lanza el agente (sin terminal: quien los lanza
//! sale de la cache de identidad).
//!
//! Como en `tests/toma.rs`: `origin` apunta a `ssh://git@github.com/o/r.git`
//! y el transporte ssh se sustituye con `GIT_SSH_COMMAND`, que sirve el
//! remoto bare de la fixture; sin red, un «ssh» que responde «Connection
//! refused» (una ruta inexistente daria `denegada`, no `sinred`).

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::path::{Path, PathBuf};
use std::process::Output;

use comun::lanzar;
use comun::remoto::{PERSONAS, identificar};
use comun::{Fixture, escribir, texto, valor};
use tc2::contrato::Resultado;
use tc2::estado::{self, Cambio, Contexto, Estado, Fallo, Operacion};

const RAMA_ESTADO: &str = estado::RAMA;

/// `tc2 <args>` en `dir` con el remoto: (codigo, la unica linea de stdout).
/// Un exito no escribe en stderr; un fallo, una linea.
fn tc2(f: &Fixture, dir: &Path, args: &[&str]) -> (Option<i32>, String) {
    salida(&lanzar(dir, f.entorno_ssh(), args))
}

fn salida(o: &Output) -> (Option<i32>, String) {
    let out = texto(&o.stdout);
    assert_eq!(out.lines().count(), 1, "{out}");
    let err = texto(&o.stderr);
    if o.status.code() == Some(0) {
        assert!(err.is_empty(), "stderr en un exito: {err}");
    } else {
        assert_eq!(err.lines().count(), 1, "{err}");
    }
    (o.status.code(), out.trim_end().to_owned())
}

/// Las claves de una linea de contrato, en orden (sin comando ni
/// resultado).
fn claves(linea: &str) -> Vec<&str> {
    linea
        .split(' ')
        .skip(2)
        .filter_map(|kv| kv.split_once('=').map(|(k, _)| k))
        .collect()
}

/// Remoto con `personas.yml` y las tareas de `titulos` (`tc2 new`); el clon
/// `a` que devuelve tiene `origin` en GitHub y la identidad de ana.
fn preparar(f: &Fixture, titulos: &[&str]) -> (PathBuf, PathBuf) {
    let (bare, prep) = f.remoto_y_clon();
    escribir(&prep, ".taskcode/personas.yml", PERSONAS);
    f.crudo(&prep, &["add", ".taskcode/personas.yml"]);
    f.crudo(&prep, &["commit", "-q", "-m", "personas"]);
    f.crudo(&prep, &["push", "-q", "origin", "develop"]);
    for t in titulos {
        let o = lanzar(&prep, f.entorno(), &["new", t]);
        assert_eq!(o.status.code(), Some(0), "{}", texto(&o.stdout));
    }
    let a = f.clon_github(&bare, "a");
    identificar(&a, 1001, "ana");
    (bare, a)
}

fn publicado(f: &Fixture, bare: &Path, ruta: &str) -> String {
    f.crudo(bare, &["show", &format!("{RAMA_ESTADO}:{ruta}")])
}

fn tarea(f: &Fixture, bare: &Path, id: &str) -> String {
    publicado(f, bare, &format!("{id}/tarea.md"))
}

fn punta(f: &Fixture, bare: &Path) -> String {
    f.crudo(bare, &["rev-parse", RAMA_ESTADO])
}

fn worktree(dir: &Path, id: &str) -> PathBuf {
    dir.join(".taskcodetwo").join("worktrees").join(id)
}

/// Lo que `tarea.md` gana con una fila: `antes` con `actualizado` de hoy y
/// la fila `fase` de la transicion (la ultima linea de `despues`).
fn con_fila(antes: &str, despues: &str, fase: &str) -> String {
    let hoy = tc2::toma::instante(tc2::identidad::ahora());
    let actualizado = antes
        .lines()
        .find_map(|l| l.strip_prefix("actualizado: "))
        .unwrap()
        .to_owned();
    let fila = despues.lines().last().unwrap();
    assert!(fila.starts_with(&format!("| {}T", &hoy[..10])), "{fila}");
    assert!(
        fila.ends_with(&format!(" | {fase} | manual | agente | {} |", tc2::VERSION)),
        "{fila}"
    );
    format!(
        "{}{fila}\n",
        antes.replacen(
            &format!("actualizado: {actualizado}"),
            &format!("actualizado: \"{}\"", &hoy[..10]),
            1
        )
    )
}

/// Escribe en la rama de estado por la transaccion (como otra fase).
struct Escribir {
    ruta: String,
    de: &'static str,
    a: &'static str,
}

impl Operacion for Escribir {
    fn nombre(&self) -> &str {
        "escribir"
    }
    fn anotacion(&self) -> String {
        format!("escribir|{}", self.ruta)
    }
    fn exige_red(&self) -> bool {
        true
    }
    fn aplicar(&self, e: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        let t = e.leer(&self.ruta)?.unwrap_or_default();
        assert!(t.contains(self.de), "{t}");
        Ok(vec![Cambio::nuevo(
            self.ruta.clone(),
            t.replacen(self.de, self.a, 1),
        )])
    }
}

fn sin_catalogo(_: &str) -> Option<Box<dyn Operacion>> {
    None
}

/// Cambia `de` por `a` en el `tarea.md` publicado de `id`.
fn editar(f: &Fixture, dir: &Path, id: &str, de: &'static str, a: &'static str) {
    let g = tc2::git::Git::nuevo().con_entorno(f.entorno_ssh());
    let op = Escribir {
        ruta: format!("{id}/tarea.md"),
        de,
        a,
    };
    let i = estado::transaccion(&Contexto::nuevo(&g, dir, &sin_catalogo, None), &op);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
}

/// C1: desde planificada, en diseno, en curso y en revision sale 0
/// `descartar ok`; el `tarea.md` publicado cambia solo en estado, motivo,
/// actualizado y una fila `descartar` (asignado y rama se conservan).
#[test]
fn c1_descarta_desde_cada_estado_y_el_resto_byte_a_byte() {
    let f = Fixture::nueva("ciclo-c1");
    let (bare, a) = preparar(&f, &["Uno", "Dos", "Tres", "Cuatro"]);
    editar(
        &f,
        &a,
        "TASK-002",
        "estado: \"planificada\"",
        "estado: \"en-diseno\"",
    );
    for id in ["TASK-003", "TASK-004"] {
        let (c, out) = tc2(&f, &a, &["start", id]);
        assert_eq!(c, Some(0), "{out}");
    }
    editar(
        &f,
        &a,
        "TASK-004",
        "estado: \"en-curso\"",
        "estado: \"en-revision\"",
    );
    for (id, de) in [
        ("TASK-001", "planificada"),
        ("TASK-002", "en-diseno"),
        ("TASK-003", "en-curso"),
        ("TASK-004", "en-revision"),
    ] {
        let antes = tarea(&f, &bare, id);
        let (c, out) = tc2(&f, &a, &["descartar", id, "ya no hace falta: «x» | y"]);
        assert_eq!(c, Some(0), "{out}");
        assert!(
            out.starts_with(&format!("descartar ok id={id} commit=")),
            "{out}"
        );
        let despues = tarea(&f, &bare, id);
        let esperado = con_fila(
            &antes.replacen(
                &format!("estado: \"{de}\"\nmotivo: null"),
                "estado: \"descartada\"\nmotivo: \"ya no hace falta: «x» | y\"",
                1,
            ),
            &despues,
            "descartar",
        );
        assert_eq!(despues, esperado, "{id}");
    }
    // Asignado y rama siguen (D7) en las que se habian empezado.
    let md = tarea(&f, &bare, "TASK-003");
    assert!(
        md.contains("\nasignado_a: {id: 1001, login: \"ana\"}\n"),
        "{md}"
    );
    assert!(md.contains("\nrama: \"feature/task-003-tres\"\n"), "{md}");
}

/// C2: sin motivo, vacio, solo espacios o con caracteres de control es uso
/// (2) y la punta del estado no cambia (ni nada del clon).
#[test]
fn c2_motivo_que_no_vale_es_uso_sin_tocar_nada() {
    let f = Fixture::nueva("ciclo-c2");
    let (bare, a) = preparar(&f, &["Uno"]);
    let antes = f.huella(&bare, &a);
    for (args, motivo) in [
        (vec!["descartar", "TASK-001"], "sin-motivo"),
        (vec!["descartar", "TASK-001", ""], "motivo-invalido"),
        (vec!["descartar", "TASK-001", "   "], "motivo-invalido"),
        (vec!["descartar", "TASK-001", "uno\ndos"], "motivo-invalido"),
        (vec!["descartar", "TASK-001", "a\u{7}b"], "motivo-invalido"),
    ] {
        let (c, out) = tc2(&f, &a, &args);
        assert_eq!(c, Some(2), "{out}");
        assert_eq!(out, format!("descartar uso motivo={motivo}"));
        assert_eq!(f.huella(&bare, &a), antes, "{args:?}");
    }
    assert!(tarea(&f, &bare, "TASK-001").contains("estado: \"planificada\""));
}

/// C3: el `ROADMAP.md` del mismo commit lista la descartada con su motivo
/// en Descartadas, fuera de hitos y listas, y `next` no la ofrece.
#[test]
fn c3_roadmap_del_mismo_commit_y_next_sin_ella() {
    let f = Fixture::nueva("ciclo-c3");
    let (bare, a) = preparar(&f, &["Uno", "Dos"]);
    let (_, out) = tc2(&f, &a, &["next"]);
    assert_eq!(valor(&out, "listas"), Some("TASK-001,TASK-002"), "{out}");
    let (c, out) = tc2(&f, &a, &["descartar", "TASK-002", "ya no hace falta"]);
    assert_eq!(c, Some(0), "{out}");
    let cambiados = f.crudo(&bare, &["show", "--name-only", "--format=", RAMA_ESTADO]);
    let mut cambiados: Vec<&str> = cambiados.lines().collect();
    cambiados.sort_unstable();
    assert_eq!(cambiados, ["ROADMAP.md", "TASK-002/tarea.md"]);
    let roadmap = publicado(&f, &bare, "ROADMAP.md");
    let descartadas = roadmap.split("\n## Descartadas\n").nth(1).unwrap();
    assert!(
        descartadas.contains("- TASK-002 — Dos (H")
            && descartadas.contains(") — motivo: ya no hace falta\n"),
        "{roadmap}"
    );
    let antes = roadmap.split("\n## Descartadas\n").next().unwrap();
    assert!(!antes.contains("TASK-002"), "{roadmap}");
    let (c, out) = tc2(&f, &a, &["next"]);
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "listas"), Some("TASK-001"), "{out}");
    assert!(!out.contains("TASK-002"), "{out}");
}

/// C4: tras descartar la de numero mas alto, `new` y un `import` siguen la
/// numeracion sin reutilizar su ID; la carpeta sigue en la rama de estado.
#[test]
fn c4_el_id_de_la_descartada_no_se_reutiliza() {
    let f = Fixture::nueva("ciclo-c4");
    let (bare, a) = preparar(&f, &["Uno", "Dos"]);
    let (c, out) = tc2(&f, &a, &["descartar", "TASK-002", "duplicada"]);
    assert_eq!(c, Some(0), "{out}");
    let (c, out) = tc2(&f, &a, &["new", "Tres"]);
    assert_eq!((c, valor(&out, "id")), (Some(0), Some("TASK-003")), "{out}");
    let lote = f.raiz.join("lote.md");
    std::fs::write(
        &lote,
        "---\nlote: ciclo\nsprint: 1\n---\n# Plan\n\n### H1 Cuarta tarea\n> Objetivo\n- Criterio\n",
    )
    .unwrap();
    let (c, out) = tc2(&f, &a, &["import", lote.to_str().unwrap()]);
    assert_eq!(c, Some(0), "{out}");
    let carpetas = f.crudo(&bare, &["ls-tree", "--name-only", RAMA_ESTADO]);
    let carpetas: Vec<&str> = carpetas
        .lines()
        .filter(|l| l.starts_with("TASK-"))
        .collect();
    assert_eq!(carpetas, ["TASK-001", "TASK-002", "TASK-003", "TASK-004"]);
    assert!(tarea(&f, &bare, "TASK-002").contains("estado: \"descartada\""));
    // La del import es la TASK-004 (lleva su lote).
    let md = tarea(&f, &bare, "TASK-004");
    assert!(md.contains("\nlote: \"ciclo/"), "{md}");
}

/// C5: con el worktree limpio y la rama publicada, `worktree=quitado` y la
/// rama sigue en local y en el remoto; con cambios sin guardar, `ok` con
/// `worktree=conservado` y la carpeta intacta; lanzado desde dentro del
/// worktree, tambien se conserva.
#[test]
fn c5_worktree_quitado_o_conservado_y_la_rama_sigue() {
    let f = Fixture::nueva("ciclo-c5");
    let (bare, a) = preparar(&f, &["Uno", "Dos", "Tres"]);
    for id in ["TASK-001", "TASK-002", "TASK-003"] {
        let (c, out) = tc2(&f, &a, &["start", id]);
        assert_eq!(c, Some(0), "{out}");
    }
    let rama = "feature/task-001-uno";
    f.git_ssh(&worktree(&a, "TASK-001"), &["push", "-q", "origin", rama]);
    let (c, out) = tc2(&f, &a, &["descartar", "TASK-001", "no"]);
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "worktree"), Some("quitado"), "{out}");
    assert!(!worktree(&a, "TASK-001").exists());
    f.crudo(
        &a,
        &["rev-parse", "--verify", &format!("refs/heads/{rama}")],
    );
    f.crudo(
        &bare,
        &["rev-parse", "--verify", &format!("refs/heads/{rama}")],
    );
    // Con cambios sin guardar: se conserva y se dice.
    let wt2 = worktree(&a, "TASK-002");
    escribir(&wt2, "a-medias.txt", "sin guardar\n");
    let (c, out) = tc2(&f, &a, &["descartar", "TASK-002", "no"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("descartar ok id=TASK-002 "), "{out}");
    assert_eq!(valor(&out, "worktree"), Some("conservado"), "{out}");
    assert_eq!(
        valor(&out, "worktree-motivo"),
        Some("cambios-sin-guardar"),
        "{out}"
    );
    assert_eq!(
        std::fs::read_to_string(wt2.join("a-medias.txt")).unwrap(),
        "sin guardar\n"
    );
    assert!(tarea(&f, &bare, "TASK-002").contains("estado: \"descartada\""));
    // Desde dentro del worktree de la tarea (publicado y limpio): se
    // conserva, no se quita la carpeta en la que se esta.
    let wt3 = worktree(&a, "TASK-003");
    f.git_ssh(&wt3, &["push", "-q", "origin", "feature/task-003-tres"]);
    let (c, out) = tc2(&f, &wt3, &["descartar", "TASK-003", "no"]);
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "worktree"), Some("conservado"), "{out}");
    assert_eq!(valor(&out, "worktree-motivo"), Some("en-uso"), "{out}");
    assert!(wt3.join("README.md").is_file());
    // Repetir desde el principal termina de quitarlo (sin-cambios).
    let (c, out) = tc2(&f, &a, &["descartar", "TASK-003", "no"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(
        out.starts_with("descartar sin-cambios id=TASK-003 "),
        "{out}"
    );
    assert_eq!(valor(&out, "worktree"), Some("quitado"), "{out}");
}

/// C5 (ronda 1, hallazgo 1 de calidad): la rama de la tarea la publica otro
/// clon y este no la ha visto; el `fetch` previo de `descartar` la trae, el
/// worktree limpio cuenta como publicado y se quita.
#[test]
fn c5_rama_publicada_desde_otro_clon_quita_el_worktree() {
    let f = Fixture::nueva("ciclo-c5-otro-clon");
    let (bare, a) = preparar(&f, &["Uno"]);
    let (c, out) = tc2(&f, &a, &["start", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    let rama = "feature/task-001-uno";
    // La persona sigue la tarea en otro clon y publica la rama desde alli.
    let b = f.clon_github(&bare, "b");
    f.git_ssh(&b, &["switch", "-q", "-c", rama, "origin/develop"]);
    escribir(&b, "seguido.txt", "desde el otro clon\n");
    f.git_ssh(&b, &["add", "seguido.txt"]);
    f.git_ssh(&b, &["commit", "-q", "-m", "seguido"]);
    f.git_ssh(&b, &["push", "-q", "origin", rama]);
    // El primero no la ha visto.
    let remota = format!("refs/remotes/origin/{rama}");
    assert!(f.git_ssh(&a, &["for-each-ref", &remota]).trim().is_empty());
    let (c, out) = tc2(&f, &a, &["descartar", "TASK-001", "no"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("descartar ok id=TASK-001 "), "{out}");
    assert_eq!(valor(&out, "worktree"), Some("quitado"), "{out}");
    assert_eq!(valor(&out, "worktree-motivo"), None, "{out}");
    assert!(!worktree(&a, "TASK-001").exists());
    f.crudo(
        &a,
        &["rev-parse", "--verify", &format!("refs/heads/{rama}")],
    );
}

/// C5 (ronda 1, hallazgo 2 de calidad): descartar justo tras `start`, con la
/// rama sin publicar, publica el descarte y conserva el worktree
/// (`sin-publicar`); con HEAD suelto en el worktree, `sin-rama`. La carpeta,
/// intacta en los dos.
#[test]
fn c5_sin_publicar_o_con_head_suelto_conserva_el_worktree() {
    let f = Fixture::nueva("ciclo-c5-sin-publicar");
    let (bare, a) = preparar(&f, &["Uno", "Dos"]);
    for id in ["TASK-001", "TASK-002"] {
        let (c, out) = tc2(&f, &a, &["start", id]);
        assert_eq!(c, Some(0), "{out}");
    }
    let wt2 = worktree(&a, "TASK-002");
    f.git_ssh(&wt2, &["checkout", "-q", "--detach"]);
    for (id, motivo) in [("TASK-001", "sin-publicar"), ("TASK-002", "sin-rama")] {
        let wt = worktree(&a, id);
        let antes = std::fs::read_to_string(wt.join("README.md")).unwrap();
        let (c, out) = tc2(&f, &a, &["descartar", id, "no"]);
        assert_eq!(c, Some(0), "{out}");
        assert!(out.starts_with(&format!("descartar ok id={id} ")), "{out}");
        assert_eq!(valor(&out, "worktree"), Some("conservado"), "{out}");
        assert_eq!(valor(&out, "worktree-motivo"), Some(motivo), "{out}");
        assert_eq!(
            std::fs::read_to_string(wt.join("README.md")).unwrap(),
            antes
        );
        assert!(tarea(&f, &bare, id).contains("estado: \"descartada\""));
    }
}

/// Ronda 1, hallazgo 1 de rust (heredado): lanzadas desde una subcarpeta del
/// clon o de un worktree, las ordenes leen la lista de personas de la base
/// igual que desde la raiz.
#[test]
fn desde_una_subcarpeta_del_clon_o_del_worktree() {
    let f = Fixture::nueva("ciclo-subcarpeta");
    let (bare, a) = preparar(&f, &["Uno", "Dos"]);
    let sub = a.join("docs").join("dentro");
    std::fs::create_dir_all(&sub).unwrap();
    let (c, out) = tc2(&f, &sub, &["pausa", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("pausa ok id=TASK-001 "), "{out}");
    let (c, out) = tc2(&f, &sub, &["start", "TASK-002"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("start ok id=TASK-002 "), "{out}");
    assert!(worktree(&a, "TASK-002").join("README.md").is_file());
    let sub_wt = worktree(&a, "TASK-002").join("dentro");
    std::fs::create_dir_all(&sub_wt).unwrap();
    let (c, out) = tc2(&f, &sub_wt, &["pausa", "TASK-002"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("pausa ok id=TASK-002 "), "{out}");
    let (c, out) = tc2(&f, &sub, &["descartar", "TASK-001", "no"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("descartar ok id=TASK-001 "), "{out}");
    assert!(tarea(&f, &bare, "TASK-001").contains("estado: \"descartada\""));
}

/// C6: la tarea de otra persona da 3 `motivo=de-otra-persona asignado=` sin
/// publicar nada (en las tres ordenes); sin asignado o propia, `ok`.
#[test]
fn c6_de_otra_persona_se_rechaza_sin_asignado_o_propia_no() {
    let f = Fixture::nueva("ciclo-c6");
    let (bare, a) = preparar(&f, &["Uno", "Dos", "Tres"]);
    let b = f.clon_github(&bare, "b");
    identificar(&b, 1002, "beto");
    let (c, out) = tc2(&f, &b, &["start", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    f.git_ssh(&a, &["fetch", "-q", "origin"]);
    let antes = f.huella(&bare, &a);
    for args in [
        vec!["descartar", "TASK-001", "no"],
        vec!["pausa", "TASK-001"],
        vec!["reanudar", "TASK-001"],
    ] {
        let (c, out) = tc2(&f, &a, &args);
        assert_eq!(c, Some(3), "{out}");
        assert!(
            out.starts_with(&format!("{} rechazada id=TASK-001 ", args[0])),
            "{out}"
        );
        assert_eq!(valor(&out, "motivo"), Some("de-otra-persona"), "{out}");
        assert_eq!(valor(&out, "asignado"), Some("beto"), "{out}");
        assert_eq!(f.huella(&bare, &a), antes, "{args:?}");
    }
    // Sin asignado: cualquiera.
    for args in [
        vec!["pausa", "TASK-002"],
        vec!["reanudar", "TASK-002"],
        vec!["descartar", "TASK-002", "no"],
    ] {
        let (c, out) = tc2(&f, &a, &args);
        assert_eq!(c, Some(0), "{out}");
        assert!(
            out.starts_with(&format!("{} ok id=TASK-002 ", args[0])),
            "{out}"
        );
    }
    // Propia.
    let (c, out) = tc2(&f, &a, &["start", "TASK-003"]);
    assert_eq!(c, Some(0), "{out}");
    for args in [
        vec!["pausa", "TASK-003"],
        vec!["reanudar", "TASK-003"],
        vec!["descartar", "TASK-003", "no"],
    ] {
        let (c, out) = tc2(&f, &a, &args);
        assert_eq!(c, Some(0), "{out}");
        assert!(
            out.starts_with(&format!("{} ok id=TASK-003 ", args[0])),
            "{out}"
        );
    }
}

/// C7: descartar una descartada es `sin-cambios` sin commit y con el motivo
/// de entonces; una terminada, 3 `terminada`; pausar o reanudar una
/// terminada o descartada, 3.
#[test]
fn c7_descartada_terminada_y_sus_rechazos() {
    let f = Fixture::nueva("ciclo-c7");
    let (bare, a) = preparar(&f, &["Uno", "Dos"]);
    let (c, out) = tc2(&f, &a, &["descartar", "TASK-001", "el primero"]);
    assert_eq!(c, Some(0), "{out}");
    let p = punta(&f, &bare);
    let (c, out) = tc2(&f, &a, &["descartar", "TASK-001", "otro distinto"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(
        out.starts_with("descartar sin-cambios id=TASK-001"),
        "{out}"
    );
    assert_eq!(punta(&f, &bare), p);
    assert!(tarea(&f, &bare, "TASK-001").contains("\nmotivo: \"el primero\"\n"));
    editar(
        &f,
        &a,
        "TASK-002",
        "estado: \"planificada\"",
        "estado: \"terminada\"",
    );
    let antes = f.huella(&bare, &a);
    for (args, motivo) in [
        (vec!["descartar", "TASK-002", "no"], "terminada"),
        (vec!["pausa", "TASK-002"], "terminada"),
        (vec!["reanudar", "TASK-002"], "terminada"),
        (vec!["pausa", "TASK-001"], "descartada"),
        (vec!["reanudar", "TASK-001"], "descartada"),
    ] {
        let (c, out) = tc2(&f, &a, &args);
        assert_eq!((c, valor(&out, "motivo")), (Some(3), Some(motivo)), "{out}");
        assert_eq!(f.huella(&bare, &a), antes, "{args:?}");
    }
}

/// C8: sin red `descartar` da 4 `sinred pendiente=no` sin commit ni ref
/// pendiente; `pausa` de una tarea propia da `sinred pendiente=si` y la
/// siguiente transaccion con red la publica.
#[test]
fn c8_sin_red_descartar_falla_y_la_pausa_queda_pendiente() {
    let f = Fixture::nueva("ciclo-c8");
    let (bare, a) = preparar(&f, &["Uno"]);
    let (c, out) = tc2(&f, &a, &["start", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    let antes = f.huella(&bare, &a);
    let o = lanzar(&a, f.entorno_sin_red(), &["descartar", "TASK-001", "no"]);
    let (c, out) = salida(&o);
    assert_eq!(c, Some(4), "{out}");
    assert!(out.starts_with("descartar sinred id=TASK-001"), "{out}");
    assert_eq!(valor(&out, "pendiente"), Some("no"), "{out}");
    assert_eq!(f.huella(&bare, &a), antes);
    let pendientes = || f.crudo(&a, &["for-each-ref", estado::REFS_PENDIENTES]);
    assert!(pendientes().trim().is_empty());
    // La pausa queda pendiente.
    let p = punta(&f, &bare);
    let o = lanzar(&a, f.entorno_sin_red(), &["pausa", "TASK-001"]);
    let (c, out) = salida(&o);
    assert_eq!(c, Some(4), "{out}");
    assert!(out.starts_with("pausa sinred id=TASK-001"), "{out}");
    assert_eq!(valor(&out, "pendiente"), Some("si"), "{out}");
    assert_eq!(pendientes().lines().count(), 1);
    assert_eq!(punta(&f, &bare), p);
    // La siguiente transaccion con red la publica.
    let (c, out) = tc2(&f, &a, &["new", "Otra"]);
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "pendientes"), Some("1"), "{out}");
    assert!(pendientes().trim().is_empty());
    let md = tarea(&f, &bare, "TASK-001");
    let fila = md.lines().last().unwrap();
    assert!(
        fila.ends_with(&format!(" | pausa | manual | agente | {} |", tc2::VERSION)),
        "{md}"
    );
    assert!(md.contains("estado: \"en-curso\""), "{md}");
    assert!(
        publicado(&f, &bare, "ROADMAP.md").contains("TASK-001 (en curso · en pausa)"),
        "el ROADMAP no ve la pausa publicada"
    );
}

/// C9: `pausa` anade la fila sin cambiar estado, asignado, rama ni
/// worktree; una planificada en pausa sale de `next`, el ROADMAP la marca y
/// `start` la rechaza `motivo=pausada`; repetir da `sin-cambios`.
#[test]
fn c9_pausa_sin_cambiar_estado_fuera_de_next_y_start_la_rechaza() {
    let f = Fixture::nueva("ciclo-c9");
    let (bare, a) = preparar(&f, &["Uno", "Dos"]);
    let (c, out) = tc2(&f, &a, &["start", "TASK-002"]);
    assert_eq!(c, Some(0), "{out}");
    let antes = tarea(&f, &bare, "TASK-001");
    let (c, out) = tc2(&f, &a, &["pausa", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    // La linea completa, como la fija la adenda de TASK-034 en F1-4.
    assert!(out.starts_with("pausa ok id=TASK-001 commit="), "{out}");
    assert_eq!(
        claves(&out),
        ["id", "commit", "intentos", "tareas"],
        "{out}"
    );
    assert_eq!(valor(&out, "intentos"), Some("1"), "{out}");
    assert_eq!(valor(&out, "tareas"), Some("sin-montar"), "{out}");
    let despues = tarea(&f, &bare, "TASK-001");
    assert_eq!(despues, con_fila(&antes, &despues, "pausa"));
    let (_, out) = tc2(&f, &a, &["next"]);
    assert_eq!(valor(&out, "listas").unwrap_or_default(), "", "{out}");
    let roadmap = publicado(&f, &bare, "ROADMAP.md");
    assert!(
        roadmap.contains("| TASK-001 | Uno | — | en pausa |"),
        "{roadmap}"
    );
    assert!(
        roadmap.contains("- **Listas para empezar:** ninguna\n"),
        "{roadmap}"
    );
    let p = punta(&f, &bare);
    let (c, out) = tc2(&f, &a, &["start", "TASK-001"]);
    assert_eq!(
        (c, valor(&out, "motivo")),
        (Some(3), Some("pausada")),
        "{out}"
    );
    assert_eq!(valor(&out, "arreglo"), Some("tc2%20reanudar"), "{out}");
    let (c, out) = tc2(&f, &a, &["pausa", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("pausa sin-cambios id=TASK-001"), "{out}");
    assert_eq!(punta(&f, &bare), p);
    // Una en curso: estado, asignado, rama y worktree, igual.
    let antes = tarea(&f, &bare, "TASK-002");
    let (c, out) = tc2(&f, &a, &["pausa", "TASK-002"]);
    assert_eq!(c, Some(0), "{out}");
    let despues = tarea(&f, &bare, "TASK-002");
    assert_eq!(despues, con_fila(&antes, &despues, "pausa"));
    assert!(worktree(&a, "TASK-002").join("README.md").is_file());
    let roadmap = publicado(&f, &bare, "ROADMAP.md");
    assert!(
        roadmap.contains("TASK-002 (en curso · en pausa)"),
        "{roadmap}"
    );
}

/// C10: `reanudar` anade la fila y la tarea vuelve a `next`; reanudar una
/// que no esta en pausa da `sin-cambios`.
#[test]
fn c10_reanudar_la_devuelve_a_next() {
    let f = Fixture::nueva("ciclo-c10");
    let (bare, a) = preparar(&f, &["Uno"]);
    let p = punta(&f, &bare);
    let (c, out) = tc2(&f, &a, &["reanudar", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("reanudar sin-cambios id=TASK-001"), "{out}");
    assert_eq!(punta(&f, &bare), p);
    let (c, out) = tc2(&f, &a, &["pausa", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    let antes = tarea(&f, &bare, "TASK-001");
    let (c, out) = tc2(&f, &a, &["reanudar", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("reanudar ok id=TASK-001 commit="), "{out}");
    assert_eq!(
        claves(&out),
        ["id", "commit", "intentos", "tareas"],
        "{out}"
    );
    let despues = tarea(&f, &bare, "TASK-001");
    assert_eq!(despues, con_fila(&antes, &despues, "reanudar"));
    let (_, out) = tc2(&f, &a, &["next"]);
    assert_eq!(valor(&out, "listas"), Some("TASK-001"), "{out}");
    let (c, out) = tc2(&f, &a, &["start", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
}

/// C11: un ID mal formado es uso (2) como `start`, y `--json` da el mismo
/// contenido en una linea (tambien en un rechazo).
#[test]
fn c11_uso_y_json_en_una_linea() {
    let f = Fixture::nueva("ciclo-c11");
    let (bare, a) = preparar(&f, &["Uno"]);
    let antes = f.huella(&bare, &a);
    for (args, linea) in [
        (
            vec!["descartar", "TASK-1", "m"],
            "descartar uso motivo=id-invalido",
        ),
        (vec!["pausa"], "pausa uso motivo=sin-id"),
        (
            vec!["reanudar", "TASK-001", "x"],
            "reanudar uso motivo=argumentos-de-mas",
        ),
    ] {
        let (c, out) = tc2(&f, &a, &args);
        assert_eq!((c, out.as_str()), (Some(2), linea));
    }
    assert_eq!(f.huella(&bare, &a), antes);
    let (c, out) = tc2(&f, &a, &["pausa", "--json", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["comando"], "pausa");
    assert_eq!(v["resultado"], "ok");
    assert_eq!(v["claves"]["id"], "TASK-001");
    let (c, out) = tc2(&f, &a, &["descartar", "--json", "TASK-001", "por json"]);
    assert_eq!(c, Some(0), "{out}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["comando"], "descartar");
    assert_eq!(v["claves"]["id"], "TASK-001");
    assert_eq!(v["claves"]["worktree"], "ninguno");
    assert!(tarea(&f, &bare, "TASK-001").contains("\nmotivo: \"por json\"\n"));
    let (c, out) = tc2(&f, &a, &["reanudar", "--json", "TASK-050"]);
    assert_eq!(c, Some(3), "{out}");
    assert!(out.contains("\"motivo\":\"no-existe\""), "{out}");
}
