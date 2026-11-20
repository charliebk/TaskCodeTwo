//! F6-3 (TASK-049): `tc2 revisoras` lanzado como binario sobre un repo git
//! real, con el plugin dado por `--plugin` (los fixtures de
//! `fixtures/revisoras`, que copian el bloque yaml de las revisoras del
//! plugin 0.16.2) y las rutas por la entrada estandar.
//!
//! - C1: un diff Rust + Vue da las dos revisoras de dominio, linea de
//!   contrato y JSON, y el detalle por revisora.
//! - C2 y decision 2 de F1-15: las skills del proyecto salen de la base
//!   (`origin/develop`), no del worktree; el informe de verificacion se cita
//!   con tope de tamano (seguimiento de TASK-044).

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use comun::{Fixture, escribir};

fn plugin() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/revisoras")
}

/// Lanza `tc2 revisoras <args>` en `dir` con `entrada` por stdin; da
/// (codigo, stdout).
fn revisoras(
    f: &Fixture,
    dir: &Path,
    args: &[&str],
    entrada: impl AsRef<[u8]>,
) -> (Option<i32>, String) {
    let vacia = f.raiz.join("claude-vacia");
    std::fs::create_dir_all(&vacia).unwrap();
    let mut c = Command::new(env!("CARGO_BIN_EXE_tc2"))
        .arg("revisoras")
        .args(args)
        .current_dir(dir)
        .envs(f.entorno())
        .env("TC2_IDIOMA", "es")
        .env("CLAUDE_CONFIG_DIR", &vacia)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("no se pudo lanzar tc2: {e}"));
    c.stdin.take().unwrap().write_all(entrada.as_ref()).unwrap();
    let o = c.wait_with_output().unwrap();
    eprintln!("  {}", String::from_utf8_lossy(&o.stderr));
    (
        o.status.code(),
        String::from_utf8_lossy(&o.stdout).into_owned(),
    )
}

/// C1 (integracion): `src/a.rs` y `web/App.vue` por stdin lanzan las dos
/// revisoras de dominio con S08, S12 y M04; la linea cumple el contrato y el
/// JSON y el detalle dicen lo mismo.
#[test]
fn rust_y_vue_por_stdin_dan_las_dos_de_dominio() {
    let f = Fixture::nueva("revisoras-c1");
    let (_bare, clon) = f.remoto_y_clon();
    let p = format!("taskcode-plugin={}", plugin().display());
    let base = ["--rutas", "-", "--complejidad", "simple", "--plugin", &p];
    let (codigo, linea) = revisoras(&f, &clon, &base, "src/a.rs\nweb/App.vue\n");
    assert_eq!(codigo, Some(0), "{linea}");
    assert_eq!(
        linea.trim_end(),
        "revisoras ok revisoras=taskcode-plugin:angular-vue-reviewer,taskcode-plugin:rust-wasm-reviewer \
         agentes=general-purpose modelo=sonnet modelo-api=claude-sonnet-5-5 reglas=M04,S08,S12 \
         avisos=config-de-base-no-leida carril=funcionalidad complejidad=simple"
    );
    let detalle = f.raiz.join("detalle.json");
    let d = detalle.to_str().unwrap();
    let mut con_json = base.to_vec();
    con_json.extend(["--json", "--detalle", d]);
    let (codigo, json) = revisoras(&f, &clon, &con_json, "web/App.vue\r\nsrc/a.rs\r\n");
    assert_eq!(codigo, Some(0), "{json}");
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["comando"], "revisoras");
    assert_eq!(v["resultado"], "ok");
    assert_eq!(
        v["claves"]["revisoras"],
        serde_json::json!([
            "taskcode-plugin:angular-vue-reviewer",
            "taskcode-plugin:rust-wasm-reviewer"
        ])
    );
    assert_eq!(
        v["claves"]["reglas"],
        serde_json::json!(["M04", "S08", "S12"])
    );
    let det: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&detalle).unwrap()).unwrap();
    assert_eq!(
        det["revisoras"][1]["rutas"],
        serde_json::json!(["src/a.rs"])
    );
    assert!(
        det["revisoras"][1]["skill"]
            .as_str()
            .unwrap()
            .ends_with("SKILL.md"),
        "{det}"
    );
    // Sin --rutas: uso, codigo 2.
    let (codigo, linea) = revisoras(&f, &clon, &["--plugin", &p], "");
    assert_eq!(
        (codigo, linea.trim_end()),
        (Some(2), "revisoras uso motivo=sin-rutas")
    );
}

/// C2: una revisora del proyecto en la base entra; una que solo esta en el
/// worktree no (F1-15 decision 2); un README suelto va a la generica. El
/// informe de verificacion se cita si vale y no si pasa del tope.
#[test]
fn skills_de_la_base_e_informe_con_tope() {
    let f = Fixture::nueva("revisoras-c2");
    let (_bare, clon) = f.remoto_y_clon();
    let sql = "---\nname: revisora-sql\n---\n\n```yaml\nrol: revisor\npatrones_archivo: [\"**/*.sql\"]\n```\n";
    escribir(&clon, ".claude/skills/revisora-sql/SKILL.md", sql);
    f.crudo(&clon, &["add", ".claude"]);
    f.crudo(&clon, &["commit", "-q", "-m", "revisora del proyecto"]);
    f.crudo(&clon, &["push", "-q", "origin", "develop"]);
    f.crudo(&clon, &["fetch", "-q", "origin"]);
    // Solo en el arbol de trabajo (lo que anadiria un PR): no cuenta.
    let colada =
        "---\nname: colada\n---\n\n```yaml\nrol: revisor\npatrones_archivo: [\"**/*.md\"]\n```\n";
    escribir(&clon, ".claude/skills/colada/SKILL.md", colada);
    let p = format!("taskcode-plugin={}", plugin().display());
    let informe = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/informe-verificacion/xtask.json");
    let seccion = f.raiz.join("seccion.md");
    let args = [
        "--rutas",
        "-",
        "--complejidad",
        "simple",
        "--plugin",
        &p,
        "--informe",
        informe.to_str().unwrap(),
        "--seccion",
        seccion.to_str().unwrap(),
        "--artefacto",
        "informe-verificacion-completa",
    ];
    let (codigo, linea) = revisoras(&f, &clon, &args, "db/001.sql\nREADME.md\n");
    assert_eq!(codigo, Some(0), "{linea}");
    assert!(
        linea.contains("revisoras=revisora-sql,taskcode-plugin:code-quality-reviewer "),
        "{linea}"
    );
    assert!(!linea.contains("colada"), "{linea}");
    assert!(linea.contains("reglas=M04,S08,S12 "), "{linea}");
    assert!(linea.trim_end().ends_with(" informe=verde"), "{linea}");
    let s = std::fs::read_to_string(&seccion).unwrap();
    assert!(s.starts_with("## Informe de verificacion (F5-4)"), "{s}");
    assert!(s.contains("`informe-verificacion-completa`"), "{s}");
    // Un informe de mas de 64 KiB no se cita ni se escribe seccion.
    let grande = f.raiz.join("grande.json");
    std::fs::write(&grande, " ".repeat(64 * 1024 + 1)).unwrap();
    std::fs::remove_file(&seccion).unwrap();
    let mut args = args.to_vec();
    args[7] = grande.to_str().unwrap();
    let (codigo, linea) = revisoras(&f, &clon, &args, "README.md\n");
    assert_eq!(codigo, Some(0), "{linea}");
    assert!(linea.trim_end().ends_with(" informe=grande"), "{linea}");
    assert!(!seccion.exists());
}

/// Hallazgo 1 de la ronda 1: con `--nul` (la salida de `git diff
/// --name-only -z`) una ruta no ASCII llega literal y casa con su revisora;
/// el detalle la lleva tal cual.
#[test]
fn ruta_no_ascii_separada_por_nul() {
    let f = Fixture::nueva("revisoras-nul");
    let (_bare, clon) = f.remoto_y_clon();
    let p = format!("taskcode-plugin={}", plugin().display());
    let detalle = f.raiz.join("detalle.json");
    let args = [
        "--rutas",
        "-",
        "--nul",
        "--complejidad",
        "simple",
        "--plugin",
        &p,
        "--detalle",
        detalle.to_str().unwrap(),
    ];
    let (codigo, linea) = revisoras(&f, &clon, &args, "é.rs\0");
    assert_eq!(codigo, Some(0), "{linea}");
    assert!(
        linea.contains(" revisoras=taskcode-plugin:rust-wasm-reviewer "),
        "{linea}"
    );
    let det: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&detalle).unwrap()).unwrap();
    assert_eq!(det["revisoras"][0]["rutas"], serde_json::json!(["é.rs"]));
}

/// MENOR de rust-wasm, ronda 1: por stdin, mas de `TOPE_RUTAS` bytes es
/// error, igual que con fichero (antes se cortaba en silencio).
#[test]
fn rutas_por_stdin_por_encima_del_tope_fallan() {
    let f = Fixture::nueva("revisoras-tope");
    let (_bare, clon) = f.remoto_y_clon();
    let p = format!("taskcode-plugin={}", plugin().display());
    let mut entrada = "src/a.rs\n".repeat(4 * 1024 * 1024 / 9 + 1).into_bytes();
    entrada.truncate(4 * 1024 * 1024 + 1);
    let (codigo, linea) = revisoras(&f, &clon, &["--rutas", "-", "--plugin", &p], &entrada);
    assert_ne!(codigo, Some(0), "{linea}");
    assert!(linea.starts_with("revisoras error"), "{linea}");
}
