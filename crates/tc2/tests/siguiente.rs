//! F3-3: `tc2 next` con el binario real sobre un clon con remoto bare (C3 y
//! C4 de TASK-030). La rama de estado se publica desde un repo aparte, sin
//! pasar por `tc2`.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::path::{Path, PathBuf};
use std::process::Output;

use comun::{Fixture, escribir};
use tc2::estado;
use tc2::tarea::{Cuerpo, Tarea};

const RAMA: &str = estado::RAMA;

fn tc2(f: &Fixture, dir: &Path, idioma: &str, args: &[&str]) -> Output {
    comun::orden_tc2()
        .current_dir(dir)
        .args(args)
        .envs(f.entorno())
        .env("TC2_IDIOMA", idioma)
        .output()
        .expect("lanzar tc2")
}

/// (codigo, stdout, stderr).
fn next(f: &Fixture, dir: &Path, idioma: &str, args: &[&str]) -> (i32, String, String) {
    let mut v = vec!["next"];
    v.extend_from_slice(args);
    let o = tc2(f, dir, idioma, &v);
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).into_owned(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

fn tarea(id: &str, sprint: u32, estado: &str, deps: &[&str]) -> String {
    let mut t = Tarea::nueva(id, "t", "2026-01-01");
    t.sprint = sprint;
    t.estado = estado.to_owned();
    if estado == "descartada" {
        t.motivo = Some("ya no hace falta".to_owned());
    }
    t.dependencias = deps.iter().map(|d| (*d).to_owned()).collect();
    t.escribir(&Cuerpo::leer(""))
}

/// Publica en `bare` la rama de estado con un juego de tareas: H2 va antes
/// que H1 en la zona manual del ROADMAP; TASK-005 es ilegible y bloquea a
/// TASK-010; TASK-007 esta descartada y bloquea a TASK-008. Devuelve el
/// commit publicado.
fn publicar_estado(f: &Fixture, bare: &Path) -> String {
    let dir = f.raiz.join("estado");
    f.crudo(&f.raiz, &["init", "-q", "-b", RAMA, "estado"]);
    for (id, md) in [
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
    ] {
        escribir(&dir, &format!("{id}/tarea.md"), &md);
    }
    escribir(
        &dir,
        "ROADMAP.md",
        "# Roadmap\n\n<!-- manual:inicio -->\n### H2 — Antes\n### H1 — Despues\n<!-- manual:fin -->\n",
    );
    f.crudo(&dir, &["add", "-A"]);
    f.crudo(&dir, &["commit", "-q", "-m", "estado"]);
    f.crudo(&dir, &["push", "-q", bare.to_str().unwrap(), RAMA]);
    f.crudo(&dir, &["rev-parse", "HEAD"]).trim().to_owned()
}

/// Clon con la rama de estado ya traida; devuelve (bare, clon, commit).
fn con_estado(f: &Fixture) -> (PathBuf, PathBuf, String) {
    let (bare, clon) = f.remoto_y_clon();
    let commit = publicar_estado(f, &bare);
    f.crudo(&clon, &["fetch", "-q", "origin"]);
    (bare, clon, commit)
}

/// La linea de `next` sin las claves de frescura de F3-5b.
fn esperada(commit: &str) -> String {
    format!(
        "next ok listas=TASK-006,TASK-002 en-marcha=TASK-003 commit={} ilegibles=TASK-005 depende-de-descartada=TASK-008>TASK-007",
        &commit[..12]
    )
}

/// Comprueba que `out` es `esperada(commit)` con `remoto=<remoto>
/// sincronizado=<hora o nunca>` detras (F3-5b).
fn con_frescura(out: &str, commit: &str, remoto: &str) {
    let resto = out
        .strip_prefix(&esperada(commit))
        .unwrap_or_else(|| panic!("{out}"));
    let hora = resto
        .strip_prefix(&format!(" remoto={remoto} sincronizado="))
        .and_then(|h| h.strip_suffix('\n'))
        .unwrap_or_else(|| panic!("{out}"));
    assert!(
        hora == "nunca" || (hora.len() == 20 && hora.ends_with('Z')),
        "{out}"
    );
}

/// C3 y C4: la linea en es y en (mismo stdout salvo la hora, codigo 0, sin
/// stderr), con el ilegible en `ilegibles=`; no escribe nada (el `fetch` de
/// F3-5b no mueve ninguna ref: no hay nada nuevo).
#[test]
fn next_en_es_y_en() {
    let f = Fixture::nueva("next-es-en");
    let (bare, clon, commit) = con_estado(&f);
    let antes = f.huella(&bare, &clon);
    for idioma in ["es", "en"] {
        let (c, out, err) = next(&f, &clon, idioma, &[]);
        assert_eq!((c, err.as_str()), (0, ""), "{out}");
        con_frescura(&out, &commit, "consultado");
    }
    assert_eq!(antes, f.huella(&bare, &clon), "next escribio algo");
    // Contrato de F1-4 (claves en minusculas con guiones, commit de 12).
    let i = tc2::siguiente::siguiente_en(&f.git(), &clon);
    comun::contrato(&i);
    con_frescura(&format!("{}\n", i.linea()), &commit, "consultado");
}

/// C3: `--json` valido con las mismas claves y las listas en array.
#[test]
fn next_json_valido() {
    let f = Fixture::nueva("next-json");
    let (_bare, clon, commit) = con_estado(&f);
    let (c, out, err) = next(&f, &clon, "es", &["--json"]);
    assert_eq!((c, err.as_str()), (0, ""));
    assert_eq!(out.lines().count(), 1);
    let v: serde_json::Value = serde_json::from_str(&out).expect("JSON valido");
    assert_eq!(v["comando"], "next");
    assert_eq!(v["resultado"], "ok");
    let k = &v["claves"];
    assert_eq!(k["listas"], serde_json::json!(["TASK-006", "TASK-002"]));
    assert_eq!(k["en-marcha"], serde_json::json!(["TASK-003"]));
    assert_eq!(k["commit"], &commit[..12]);
    assert_eq!(k["ilegibles"], serde_json::json!(["TASK-005"]));
}

/// C3: sin red funciona sobre la copia del clon, y `commit=` dice que foto
/// uso; lo publicado despues no se ve. Sin red de verdad (F3-5b): `origin`
/// por ssh con un «ssh» que no llega al host (una ruta inexistente seria
/// `denegado`, no sin red); sale `remoto=sin-red` con el aviso en stderr.
#[test]
fn next_sin_red_usa_la_copia_del_clon() {
    let f = Fixture::nueva("next-sinred");
    let (bare, clon, commit) = con_estado(&f);
    // Otro commit en el remoto que este clon no ha traido.
    let dir = f.raiz.join("estado");
    escribir(
        &dir,
        "TASK-009/tarea.md",
        &tarea("TASK-009", 2, "planificada", &[]),
    );
    f.crudo(&dir, &["add", "-A"]);
    f.crudo(&dir, &["commit", "-q", "-m", "otra"]);
    f.crudo(&dir, &["push", "-q", bare.to_str().unwrap(), RAMA]);
    f.crudo(
        &clon,
        &[
            "remote",
            "set-url",
            "origin",
            "ssh://git@github.com/o/r.git",
        ],
    );
    let o = comun::orden_tc2()
        .current_dir(&clon)
        .arg("next")
        .envs(f.entorno())
        .env(
            "GIT_SSH_COMMAND",
            "sh -c 'echo \"ssh: connect to host github.com port 22: Connection refused\" >&2; exit 255'",
        )
        .env("GIT_SSH_VARIANT", "simple")
        .output()
        .expect("lanzar tc2");
    let out = String::from_utf8_lossy(&o.stdout);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(0), "{out} {err}");
    // El clon trajo la rama con un `git fetch` a mano: la marca no se
    // renovo, asi que es `nunca` (la hora nunca sale mas fresca de lo real).
    con_frescura(&out, &commit, "sin-red");
    assert!(out.ends_with(" sincronizado=nunca\n"), "{out}");
    assert!(err.starts_with("aviso: sin red"), "{err}");
}

/// C3: sin rama de estado, `rechazada` (3) con el diagnostico en es y en.
#[test]
fn next_sin_rama_de_estado_es_rechazada() {
    let f = Fixture::nueva("next-sin-rama");
    let (_bare, clon) = f.remoto_y_clon();
    let (c, out, es) = next(&f, &clon, "es", &[]);
    assert_eq!(
        (c, out.as_str()),
        (3, "next rechazada motivo=sin-rama-de-estado\n")
    );
    let (c, _, en) = next(&f, &clon, "en", &[]);
    assert_eq!(c, 3);
    assert_eq!(es, "monta la rama de estado en tareas/ primero\n");
    assert_eq!(en, "mount the state branch in tareas/ first\n");
}

/// Argumentos de mas: `uso` con el diagnostico en es y en.
#[test]
fn next_con_argumentos_de_mas_es_uso() {
    let f = Fixture::nueva("next-uso");
    let (c, out, es) = next(&f, &f.raiz, "es", &["x"]);
    assert_eq!(
        (c, out.as_str()),
        (2, "next uso motivo=argumentos-de-mas\n")
    );
    let (_, _, en) = next(&f, &f.raiz, "en", &["x"]);
    assert_eq!(
        (es.as_str(), en.as_str()),
        ("uso: next [--json]\n", "usage: next [--json]\n")
    );
}
