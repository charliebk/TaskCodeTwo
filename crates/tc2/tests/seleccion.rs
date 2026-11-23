//! F4-6 (TASK-085), criterio 1 fuera de la muestra del ADR: `tc2 seleccion`
//! con el binario real sobre 10 tareas terminadas de TaskCodeTwo elegidas
//! por regla fija (ID mas alto por estrato, escrita en `estratos.txt` de la
//! fixture), con su `tarea.md` (formato 1 de la 0.16) y las rutas de su
//! merge en `develop` copiadas en `tests/fixtures/seleccion-taskcodetwo/`.
//!
//! El inventario es una carpeta de Claude Code propia con los agentes de
//! brainstorm del plugin (nunca la de la maquina) y el repo un Git temporal
//! sin base (`sin-base`): la seleccion sale solo de las tablas del ADR. El
//! test comprueba el contrato, el determinismo y que la tabla que sale del
//! binario es la dorada `tabla.md` de la fixture (la de la evidencia de
//! TASK-085); si la propuesta coincide con la de una persona lo marca Carlos
//! sobre esa tabla (columna «Comprobado a mano (Carlos)»).
//! `TC2_TABLA_SELECCION=1` la imprime.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::path::{Path, PathBuf};

use comun::Fixture;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/seleccion-taskcodetwo")
}

fn seleccion(f: &Fixture, dir: &Path, claude: &Path, id: &str) -> serde_json::Value {
    let d = fixtures().join(id);
    let o = comun::orden_tc2()
        .current_dir(dir)
        .args(["seleccion", "--json", "--tarea-fichero"])
        .arg(d.join("tarea.md"))
        .arg("--rutas")
        .arg(d.join("rutas.txt"))
        .envs(f.entorno())
        .env("GIT_CEILING_DIRECTORIES", &f.raiz)
        .env("CLAUDE_CONFIG_DIR", claude)
        .output()
        .expect("lanzar tc2");
    let out = String::from_utf8_lossy(&o.stdout).into_owned();
    assert_eq!(o.status.code(), Some(0), "{id}: {out}");
    serde_json::from_str(&out).unwrap_or_else(|e| panic!("{id}: {e}: {out}"))
}

fn lista(v: &serde_json::Value) -> String {
    let xs: Vec<String> = v
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str())
                .map(|x| {
                    x.rsplit(':')
                        .next()
                        .unwrap_or(x)
                        .trim_start_matches("brainstorm-")
                        .to_owned()
                })
                .collect()
        })
        .unwrap_or_default();
    if xs.is_empty() {
        "—".to_owned()
    } else {
        xs.join(", ")
    }
}

/// Criterio 1: las 10 tareas dan una seleccion por rol, dos ejecuciones dan
/// el mismo JSON, lo que la regla garantiza por estrato se cumple (la fix
/// va por el carril fix; la que toca `.github/` tiene `puertas-seguridad`
/// por esa ruta y su aviso; la de solo ADR va a la generica) y la tabla es
/// la dorada.
#[test]
fn diez_tareas_de_taskcodetwo() {
    let f = Fixture::nueva("seleccion-diez");
    let claude = comun::claude_con_brainstorm(&f.raiz, &[]);
    let dir = f.raiz.join("repo");
    std::fs::create_dir_all(&dir).unwrap();
    f.crudo(&dir, &["init", "-q", "-b", "develop"]);
    let estratos = std::fs::read_to_string(fixtures().join("estratos.txt")).unwrap();
    let filas: Vec<(&str, &str)> = estratos
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.split_once(' '))
        .collect();
    assert_eq!(filas.len(), 10, "{estratos}");
    let mut tabla = String::from(
        "| Tarea | Estrato | Carril | Complejidad | Dominios D / R | Diseño | Implementación | Revisión | Modelo D / I / R | Reglas D · I · R | Avisos | Comprobado a mano (Carlos) |\n|---|---|---|---|---|---|---|---|---|---|---|---|\n",
    );
    for (id, estrato) in &filas {
        let a = seleccion(&f, &dir, &claude, id);
        let b = seleccion(&f, &dir, &claude, id);
        assert_eq!(a, b, "{id}: no es determinista");
        let c = &a["claves"];
        assert_eq!(c["id"], *id, "{a}");
        for k in ["diseno-modelo", "implementacion-modelo", "revision-modelo"] {
            assert!(
                ["haiku", "sonnet", "opus", "-"].contains(&c[k].as_str().unwrap_or_default()),
                "{id} {k}: {a}"
            );
        }
        let dominios = c["dominios"].as_str().unwrap_or_default();
        match *estrato {
            "fix" => assert_eq!(c["carril"], "fix", "{a}"),
            "toca-github" => {
                assert!(dominios.contains("puertas-seguridad"), "{a}");
                // Por la ruta del diff, diga lo que diga el texto.
                assert!(
                    c["senales"].as_array().is_some_and(|v| v
                        .iter()
                        .any(|x| x == "puertas-seguridad:ruta:.github/**")),
                    "{a}"
                );
                assert!(lista(&c["avisos"]).contains("puertas-seguridad"), "{a}");
            }
            "solo-adr" => assert_eq!(
                lista(&c["revision-revisoras"]),
                "code-quality-reviewer",
                "{a}"
            ),
            _ => {}
        }
        let roles = c["diseno-agentes"].as_array().map_or(0, |v| {
            v.iter()
                .filter(|x| !x.as_str().unwrap_or_default().ends_with("unificador"))
                .count()
        });
        let reglas = |k: &str| {
            c[k].as_array()
                .map(|v| {
                    v.iter()
                        .filter_map(|x| x.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default()
        };
        let avisos: Vec<&str> = c["avisos"]
            .as_array()
            .map(|v| v.iter().filter_map(|x| x.as_str()).collect())
            .unwrap_or_default();
        tabla.push_str(&format!(
            "| TaskCodeTwo/{id} | {estrato} | {} | {} | {} / {dominios} | {roles}: {} | {} | {} | {} / {} / {} | {} · {} · {} | {} | |\n",
            c["carril"].as_str().unwrap_or_default(),
            c["complejidad"].as_str().unwrap_or_default(),
            c["dominios-diseno"].as_str().unwrap_or_default(),
            lista(&c["diseno-agentes"]),
            lista(&c["implementacion-skills"]),
            lista(&c["revision-revisoras"]),
            c["diseno-modelo"].as_str().unwrap_or_default(),
            c["implementacion-modelo"].as_str().unwrap_or_default(),
            c["revision-modelo"].as_str().unwrap_or_default(),
            reglas("diseno-reglas"),
            reglas("implementacion-reglas"),
            reglas("revision-reglas"),
            if avisos.is_empty() {
                "—".to_owned()
            } else {
                avisos.join(", ")
            },
        ));
    }
    if std::env::var_os("TC2_TABLA_SELECCION").is_some_and(|v| v == "1") {
        println!("{tabla}");
    }
    let dorada = std::fs::read_to_string(fixtures().join("tabla.md")).unwrap();
    assert_eq!(tabla, dorada.replace("\r\n", "\n"));
}
