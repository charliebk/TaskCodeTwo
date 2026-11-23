//! F3-9 (TASK-080): `tc2 doctor --recursos` con el binario sobre repos
//! reales en carpetas temporales y una carpeta de Claude Code falsa
//! (`CLAUDE_CONFIG_DIR`). Nunca se lee ni se escribe el `~/.claude` real ni
//! el `.claude` de este repo. Un test (o mas) por criterio.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use comun::{Fixture, escribir};
use tc2::estado;
use tc2::tarea::{Cuerpo, Tarea};

const RAMA: &str = estado::RAMA;

fn skill(nombre: &str) -> String {
    format!("---\nname: {nombre}\ndescription: prueba\n---\n# {nombre}\n")
}

fn agente(nombre: &str) -> String {
    format!("---\nname: {nombre}\ndescription: prueba\n---\n")
}

/// Como se monta la maquina falsa.
#[derive(Clone, Copy)]
struct Maquina {
    /// Con el plugin `taskcodetwo` (sin skills) instalado y habilitado.
    taskcodetwo: bool,
    /// Con el plugin de la 0.16 (`taskcode-plugin`) habilitado.
    v016: bool,
    /// Crea las skills de usuario de mas en el orden inverso (C10).
    al_reves: bool,
}

const NORMAL: Maquina = Maquina {
    taskcodetwo: true,
    v016: false,
    al_reves: false,
};

/// La carpeta de Claude Code falsa: skills y agents de usuario, el plugin
/// `bueno@mk` habilitado con `installPath`, `apagado@mk` deshabilitado en
/// `settings.json` y, segun `m`, `taskcodetwo` y la 0.16.
fn maquina(f: &Fixture, m: Maquina) -> PathBuf {
    let claude = f.raiz.join("claude");
    let cache = f.raiz.join("cache");
    let mut extra = vec!["s1", "s2", "s3"];
    if m.al_reves {
        extra.reverse();
    }
    for s in std::iter::once("alfa").chain(extra) {
        escribir(&claude, &format!("skills/{s}/SKILL.md"), &skill(s));
    }
    escribir(&claude, "agents/revisor-u.md", &agente("revisor-u"));
    escribir(&cache, "bueno/skills/rev/SKILL.md", &skill("rev"));
    escribir(&cache, "bueno/agents/ag.md", &agente("ag"));
    escribir(&cache, "apagado/skills/x/SKILL.md", &skill("x"));
    escribir(
        &cache,
        "taskcodetwo/.claude-plugin/plugin.json",
        "{\"name\": \"taskcodetwo\"}",
    );
    escribir(&cache, "v016/skills/approve/SKILL.md", &skill("approve"));
    escribir(
        &cache,
        "v016/.claude-plugin/plugin.json",
        "{\"name\": \"taskcode-plugin\"}",
    );
    let entrada = |dir: &str, v: &str| serde_json::json!([{"scope": "user", "installPath": cache.join(dir), "version": v}]);
    let mut plugins = serde_json::json!({
        "bueno@mk": entrada("bueno", "1.2.3"),
        "apagado@mk": entrada("apagado", "0.1.0"),
    });
    if m.taskcodetwo {
        plugins["taskcodetwo@tc"] = entrada("taskcodetwo", "2.0.0");
    }
    if m.v016 {
        plugins["taskcode-plugin@tc"] = entrada("v016", "0.16.1");
    }
    escribir(
        &claude,
        "plugins/installed_plugins.json",
        &serde_json::json!({"version": 2, "plugins": plugins}).to_string(),
    );
    escribir(
        &claude,
        "settings.json",
        "{\"enabledPlugins\": {\"bueno@mk\": true, \"apagado@mk\": false}}",
    );
    claude
}

fn tarea(id: &str, estado: &str, skills: &[&str]) -> String {
    let mut t = Tarea::nueva(id, &format!("Titulo de {id}"), "2026-01-01");
    t.sprint = 1;
    t.estado = estado.to_owned();
    // Una descartada sin `motivo` no se lee (y no probaria nada).
    if estado == "descartada" {
        t.motivo = Some("ya no hace falta".to_owned());
    }
    t.roles.revision.skills = skills.iter().map(|s| (*s).to_owned()).collect();
    t.roles.revision.agente = Some("general-purpose".to_owned());
    t.escribir(&Cuerpo::leer(""))
}

/// Remoto bare y clon con `config` en `develop` (si se da), la rama de
/// estado con `tareas` y `.claude/skills|agents` del proyecto. Devuelve
/// (bare, clon).
fn repo(f: &Fixture, config: Option<&str>, tareas: &[(&str, String)]) -> (PathBuf, PathBuf) {
    let (bare, clon) = f.remoto_y_clon();
    if let Some(c) = config {
        escribir(&clon, ".taskcode/config.yml", c);
        f.crudo(&clon, &["add", ".taskcode/config.yml"]);
        f.crudo(&clon, &["commit", "-q", "-m", "config"]);
        f.crudo(&clon, &["push", "-q", "origin", "develop"]);
    }
    let dir = f.raiz.join("estado");
    f.crudo(&f.raiz, &["init", "-q", "-b", RAMA, "estado"]);
    escribir(&dir, "LEEME.md", "estado\n");
    for (id, texto) in tareas {
        escribir(&dir, &format!("{id}/tarea.md"), texto);
    }
    f.crudo(&dir, &["add", "-A"]);
    f.crudo(&dir, &["commit", "-q", "-m", "estado"]);
    f.crudo(&dir, &["push", "-q", bare.to_str().unwrap(), RAMA]);
    f.crudo(&clon, &["fetch", "-q", "origin"]);
    escribir(&clon, ".claude/skills/proy/SKILL.md", &skill("proy"));
    escribir(&clon, ".claude/agents/proy-ag.md", &agente("proy-ag"));
    (bare, clon)
}

/// (codigo, stdout, stderr) de `tc2 <args>` en `dir` con la carpeta de
/// Claude Code `claude`.
fn tc2(f: &Fixture, dir: &Path, claude: &Path, args: &[&str]) -> (i32, String, String) {
    tc2_con_path(f, dir, claude, args, None)
}

/// Como [`tc2`], con `PATH` dado si es `Some`.
fn tc2_con_path(
    f: &Fixture,
    dir: &Path,
    claude: &Path,
    args: &[&str],
    path: Option<&OsStr>,
) -> (i32, String, String) {
    let mut c = comun::orden_tc2();
    if let Some(p) = path {
        c.env("PATH", p);
    }
    let o = c
        .current_dir(dir)
        .args(args)
        .envs(f.entorno())
        .env("GIT_CEILING_DIRECTORIES", &f.raiz)
        .env("CLAUDE_CONFIG_DIR", claude)
        .output()
        .expect("lanzar tc2");
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).into_owned(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

fn json(out: &str) -> serde_json::Value {
    serde_json::from_str(out).unwrap_or_else(|e| panic!("{e}: {out}"))
}

/// Los hallazgos del JSON como `<id> <estado> [motivo] [tarea]`, de `area`.
fn hallazgos(v: &serde_json::Value) -> Vec<String> {
    v["hallazgos"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["area"] == "recursos")
        .map(|h| {
            let mut s = format!(
                "{} {}",
                h["id"].as_str().unwrap(),
                h["estado"].as_str().unwrap()
            );
            for k in ["motivo", "tarea"] {
                if let Some(x) = h[k].as_str() {
                    s.push_str(&format!(" {k}={x}"));
                }
            }
            s
        })
        .collect()
}

/// C1: el inventario del JSON es exactamente lo instalado y habilitado, con
/// su origen, ordenado por (tipo, id), sin el plugin deshabilitado.
#[test]
fn inventario_exacto_con_su_origen() {
    let f = Fixture::nueva("recursos-c1");
    let claude = maquina(&f, NORMAL);
    let (_, clon) = repo(&f, None, &[]);
    let (_, out, err) = tc2(&f, &clon, &claude, &["doctor", "--recursos", "--json"]);
    let v = json(&out);
    let inv: Vec<String> = v["inventario"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            format!(
                "{} {} {}{}",
                e["tipo"].as_str().unwrap(),
                e["id"].as_str().unwrap(),
                e["origen"].as_str().unwrap(),
                e["version"]
                    .as_str()
                    .map(|x| format!(" {x}"))
                    .unwrap_or_default()
            )
        })
        .collect();
    assert_eq!(
        inv,
        [
            "skill alfa usuario",
            "skill bueno:rev plugin 1.2.3",
            "skill proy proyecto",
            "skill s1 usuario",
            "skill s2 usuario",
            "skill s3 usuario",
            "agente bueno:ag plugin 1.2.3",
            "agente general-purpose integrado",
            "agente proy-ag proyecto",
            "agente revisor-u usuario",
        ],
        "{err}"
    );
    assert!(!out.contains("apagado"), "{out}");
    assert_eq!(v["claves"]["skills"], "6");
    assert_eq!(v["claves"]["agentes"], "4");
}

/// C2: `recursos: [{id: fantasma, tipo: skill}]` da codigo 3, la linea con
/// `motivo=no-instalado tipo=skill` y arreglo no vacio, y una linea en
/// stderr.
#[test]
fn recurso_de_la_config_no_instalado_falla() {
    let f = Fixture::nueva("recursos-c2");
    let claude = maquina(&f, NORMAL);
    let (_, clon) = repo(
        &f,
        Some(
            "---\nformato: 2\nrecursos:\n  - {id: fantasma, tipo: skill}\n  - {id: alfa, tipo: skill}\n",
        ),
        &[],
    );
    let (c, out, err) = tc2(&f, &clon, &claude, &["doctor", "--recursos"]);
    assert_eq!(c, 3, "{out} {err}");
    let linea = out
        .lines()
        .find(|l| l.starts_with("recursos fantasma "))
        .unwrap_or_else(|| panic!("{out}"));
    let arreglo = linea
        .strip_prefix("recursos fantasma falla motivo=no-instalado tipo=skill arreglo=")
        .unwrap_or_else(|| panic!("{linea}"));
    assert!(!arreglo.is_empty());
    assert_eq!(err.lines().count(), 1, "{err}");
    assert!(err.contains("recursos fantasma falla"), "{err}");
    // La que si esta, `ok`.
    let (_, out, _) = tc2(&f, &clon, &claude, &["doctor", "--recursos", "--json"]);
    let hs = hallazgos(&json(&out));
    assert!(hs.contains(&"alfa ok".to_owned()), "{hs:?}");
    assert!(
        hs.contains(&"fantasma falla motivo=no-instalado".to_owned()),
        "{hs:?}"
    );
}

/// C3: una tarea abierta con `roles.revision.skills` que nombra algo no
/// instalado da `falla` con `tarea=` y arreglo; si existe, nada; una
/// terminada o descartada no cuenta.
#[test]
fn skill_de_una_tarea_abierta_no_instalada_falla() {
    let f = Fixture::nueva("recursos-c3");
    let claude = maquina(&f, NORMAL);
    let (_, clon) = repo(
        &f,
        None,
        &[
            ("TASK-002", tarea("TASK-002", "planificada", &["no-esta"])),
            ("TASK-003", tarea("TASK-003", "planificada", &["bueno:rev"])),
            (
                "TASK-004",
                tarea("TASK-004", "terminada", &["tampoco-esta"]),
            ),
            // H5 (ronda 1): una descartada tampoco cuenta.
            (
                "TASK-005",
                tarea("TASK-005", "descartada", &["descartada-no-esta"]),
            ),
        ],
    );
    let (c, out, _) = tc2(&f, &clon, &claude, &["doctor", "--recursos", "--json"]);
    assert_eq!(c, 3, "{out}");
    let v = json(&out);
    let hs = hallazgos(&v);
    assert!(
        hs.contains(&"no-esta falla motivo=no-instalado tarea=TASK-002".to_owned()),
        "{hs:?}"
    );
    assert!(
        hs.contains(&"bueno:rev ok tarea=TASK-003".to_owned()),
        "{hs:?}"
    );
    assert!(!out.contains("tampoco-esta"), "{out}");
    assert!(!out.contains("descartada-no-esta"), "{out}");
    assert!(!out.contains("TASK-005"), "{out}");
    let falla = v["hallazgos"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["id"] == "no-esta")
        .unwrap();
    assert_eq!(
        falla["arreglo"],
        "instalar-skill-o-cambiar-roles-de-TASK-002"
    );
    // Instalada la skill, ya no falla.
    escribir(&claude, "skills/no-esta/SKILL.md", &skill("no-esta"));
    let (c, out, _) = tc2(&f, &clon, &claude, &["doctor", "--recursos", "--json"]);
    assert_eq!(c, 0, "{out}");
    let hs = hallazgos(&json(&out));
    assert!(hs.iter().all(|h| !h.contains(" falla")), "{hs:?}");
}

/// C4 y C5: texto de 5 lineas como mucho con la primera fija y `y N mas:
/// tc2 doctor --recursos --json`; `--json` en una linea, sin ANSI, con
/// `hallazgos` (tambien los ok) e `inventario`.
#[test]
fn texto_acotado_y_json_completo() {
    let f = Fixture::nueva("recursos-c4");
    let claude = maquina(&f, NORMAL);
    let (_, clon) = repo(
        &f,
        Some(
            "---\nformato: 2\nrecursos:\n  - {id: f1, tipo: skill}\n  - {id: f2, tipo: agente}\n  - {id: tc2-no-existe-en-el-path, tipo: ejecutable}\n",
        ),
        &[],
    );
    let (c, out, err) = tc2(&f, &clon, &claude, &["doctor", "--recursos"]);
    assert_eq!(c, 3, "{out} {err}");
    let lineas: Vec<&str> = out.lines().collect();
    assert!(lineas.len() <= 5, "{out}");
    let primera = lineas[0];
    let (cabeza, huella) = primera.rsplit_once(" inventario=").unwrap();
    assert!(
        cabeza.starts_with("doctor rechazada fallos=3 avisos=")
            && cabeza.ends_with(" skills=6 agentes=4"),
        "{primera}"
    );
    assert!(cabeza.contains(" no-comprobado="), "{primera}");
    assert!(!primera.contains("plataforma="), "{primera}");
    assert_eq!(huella.len(), 12);
    assert!(huella.chars().all(|c| c.is_ascii_hexdigit()), "{huella}");
    assert!(!out.contains('\u{1b}') && !err.contains('\u{1b}'));

    let (cj, sj, _) = tc2(&f, &clon, &claude, &["doctor", "--recursos", "--json"]);
    assert_eq!(cj, c);
    assert_eq!(sj.lines().count(), 1);
    assert!(!sj.contains('\u{1b}'));
    let v = json(&sj);
    let n = |k: &str| v["claves"][k].as_str().unwrap().parse::<usize>().unwrap();
    let malos = n("fallos") + n("avisos") + n("no-comprobado");
    let todos = v["hallazgos"].as_array().unwrap();
    assert!(todos.iter().any(|h| h["estado"] == "ok"), "{sj}");
    assert_eq!(
        todos.len(),
        malos + todos.iter().filter(|h| h["estado"] == "ok").count()
    );
    assert_eq!(v["claves"]["inventario"], huella);
    assert!(v["inventario"].as_array().is_some_and(|a| a.len() == 10));
    assert!(malos > 4, "el caso tiene que no caber: {sj}");
    assert_eq!(lineas.len(), 5);
    assert_eq!(
        lineas[4],
        format!("y {} mas: tc2 doctor --recursos --json", malos - 3)
    );
}

/// Huella de una carpeta: rutas, fecha de modificacion y contenido,
/// ordenados. `.git` no entra: sus refs y su arbol los dicen `for-each-ref`
/// y `status`, y el propio `git status` de la foto refresca el indice.
fn huella_carpeta(raiz: &Path) -> Vec<String> {
    let mut pila = vec![raiz.to_path_buf()];
    let mut v = Vec::new();
    while let Some(d) = pila.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if e.file_name() == ".git" {
                continue;
            }
            if p.is_dir() {
                v.push(format!("{} dir", p.display()));
                pila.push(p);
            } else {
                let m = std::fs::metadata(&p).unwrap();
                v.push(format!(
                    "{} {:?} {:?}",
                    p.display(),
                    m.modified().ok(),
                    std::fs::read(&p).unwrap()
                ));
            }
        }
    }
    v.sort();
    v
}

/// C6: no escribe nada: las huellas de las carpetas temporales, `git
/// status --porcelain` y `git for-each-ref` son iguales antes y despues de
/// `--recursos` (texto y JSON) y del `doctor` normal.
#[test]
fn no_escribe_nada() {
    let f = Fixture::nueva("recursos-c6");
    let claude = maquina(&f, NORMAL);
    let (bare, clon) = repo(
        &f,
        Some("---\nformato: 2\nrecursos:\n  - {id: fantasma, tipo: skill}\n"),
        &[("TASK-002", tarea("TASK-002", "planificada", &["no-esta"]))],
    );
    let foto = || {
        let mut s = huella_carpeta(&claude);
        s.extend(huella_carpeta(&f.raiz.join("cache")));
        s.extend(huella_carpeta(&clon));
        for (dir, args) in [
            (&clon, vec!["status", "--porcelain", "--ignored"]),
            (
                &clon,
                vec!["for-each-ref", "--format=%(refname) %(objectname)"],
            ),
            (
                &bare,
                vec!["for-each-ref", "--format=%(refname) %(objectname)"],
            ),
        ] {
            s.extend(f.crudo(dir, &args).lines().map(str::to_owned));
        }
        s
    };
    let antes = foto();
    assert!(antes.len() > 10);
    for args in [
        vec!["doctor", "--recursos"],
        vec!["doctor", "--recursos", "--json"],
        vec!["doctor"],
    ] {
        let (c, out, err) = tc2(&f, &clon, &claude, &args);
        assert!(c == 0 || c == 3, "{args:?}: {out} {err}");
        let despues = foto();
        let distinta = antes
            .iter()
            .zip(&despues)
            .find(|(a, b)| a != b)
            .map(|(a, b)| format!("{:.200} / {:.200}", a, b));
        assert_eq!(
            (antes.len(), distinta),
            (despues.len(), None),
            "{args:?} cambio algo"
        );
    }
}

/// C7: con C2 montado, el `doctor` normal lleva el hallazgo agregado
/// `recursos no-instalados falla` con `ids=` y el arreglo `tc2 doctor
/// --recursos`.
#[test]
fn doctor_normal_agrega_los_no_instalados() {
    let f = Fixture::nueva("recursos-c7");
    let claude = maquina(&f, NORMAL);
    let (_, clon) = repo(
        &f,
        Some("---\nformato: 2\nrecursos:\n  - {id: fantasma, tipo: skill}\n"),
        &[],
    );
    let (c, out, _) = tc2(&f, &clon, &claude, &["doctor", "--json"]);
    assert_eq!(c, 3, "{out}");
    let v = json(&out);
    let h = v["hallazgos"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["area"] == "recursos" && h["id"] == "no-instalados")
        .unwrap_or_else(|| panic!("{out}"));
    assert_eq!(h["estado"], "falla");
    assert_eq!(h["ids"], "1");
    assert_eq!(h["arreglo"], "tc2 doctor --recursos");
    // Sin nada que falte, `ok`.
    let f = Fixture::nueva("recursos-c7-ok");
    let claude = maquina(&f, NORMAL);
    let (_, clon) = repo(&f, None, &[]);
    let (_, out, _) = tc2(&f, &clon, &claude, &["doctor", "--json"]);
    assert!(
        out.contains("{\"area\":\"recursos\",\"id\":\"no-instalados\",\"estado\":\"ok\"}"),
        "{out}"
    );
}

/// C8: `installed_plugins.json` corrupto o un `installPath` que no existe
/// dan `no-comprobado` con motivo; el plugin obligatorio no sale `ok` ni
/// `falla`.
#[test]
fn plugins_ilegibles_no_comprobado() {
    let f = Fixture::nueva("recursos-c8");
    let claude = maquina(&f, NORMAL);
    let (_, clon) = repo(&f, None, &[]);
    escribir(&claude, "plugins/installed_plugins.json", "{ corrupto");
    let (c, out, _) = tc2(&f, &clon, &claude, &["doctor", "--recursos", "--json"]);
    let hs = hallazgos(&json(&out));
    assert!(
        hs.contains(&"plugins no-comprobado motivo=json-invalido".to_owned()),
        "{hs:?}"
    );
    assert!(
        hs.contains(&"taskcodetwo no-comprobado motivo=inventario-incompleto".to_owned()),
        "{hs:?}"
    );
    assert_eq!(c, 0, "{out}");
    let instalados = serde_json::json!({"version": 2, "plugins": {
        "taskcodetwo@tc": [{"scope": "user", "installPath": f.raiz.join("no-existe"), "version": "2.0.0"}]
    }});
    escribir(
        &claude,
        "plugins/installed_plugins.json",
        &instalados.to_string(),
    );
    let (_, out, _) = tc2(&f, &clon, &claude, &["doctor", "--recursos", "--json"]);
    let hs = hallazgos(&json(&out));
    assert!(
        hs.contains(&"taskcodetwo@tc no-comprobado motivo=install-path-inexistente".to_owned()),
        "{hs:?}"
    );
    assert!(
        hs.contains(&"taskcodetwo no-comprobado motivo=inventario-incompleto".to_owned()),
        "{hs:?}"
    );
}

/// Nombres con los que `tc2` encontraria un ejecutable `rtk` en una carpeta.
const RTK: [&str; 5] = ["rtk", "rtk.exe", "rtk.cmd", "rtk.bat", "rtk.com"];

/// El `PATH` del proceso sin las carpetas que tienen `rtk` (git sigue a
/// mano) y, si se da, con `extra` delante. Asi el resultado de `rtk` no
/// depende de la maquina (MEN-3 de la ronda 1).
fn path_sin_rtk(extra: Option<&Path>) -> OsString {
    let actual = std::env::var_os("PATH").unwrap_or_default();
    let dirs = extra
        .map(Path::to_path_buf)
        .into_iter()
        .chain(std::env::split_paths(&actual).filter(|d| RTK.iter().all(|n| !d.join(n).is_file())));
    std::env::join_paths(dirs).unwrap()
}

/// C9: sin `taskcodetwo`, una sola `falla` (la suya, con el arreglo de
/// instalarlo); con la 0.16, `aviso` de nivel estorba; sin engram, context7
/// ni rtk (PATH controlado), `aviso` de nivel conveniente y, con el plugin,
/// codigo 0; con un `rtk` en el PATH, `ok`.
#[test]
fn suelo_del_manifiesto() {
    let f = Fixture::nueva("recursos-c9");
    let claude = maquina(
        &f,
        Maquina {
            taskcodetwo: false,
            v016: true,
            al_reves: false,
        },
    );
    let (_, clon) = repo(&f, None, &[]);
    let sin_rtk = path_sin_rtk(None);
    let (c, out, _) = tc2_con_path(
        &f,
        &clon,
        &claude,
        &["doctor", "--recursos", "--json"],
        Some(&sin_rtk),
    );
    assert_eq!(c, 3, "{out}");
    let v = json(&out);
    let todos = v["hallazgos"].as_array().unwrap();
    let fallas: Vec<&serde_json::Value> = todos.iter().filter(|h| h["estado"] == "falla").collect();
    assert_eq!(fallas.len(), 1, "{out}");
    assert_eq!(fallas[0]["id"], "taskcodetwo");
    assert_eq!(fallas[0]["arreglo"], "claude plugin install taskcodetwo");
    let de = |id: &str| todos.iter().find(|h| h["id"] == id).unwrap().clone();
    assert_eq!(de("taskcode-plugin")["estado"], "aviso");
    assert_eq!(de("taskcode-plugin")["nivel"], "estorba");
    for id in ["engram", "context7", "rtk"] {
        assert_eq!(de(id)["estado"], "aviso", "{id}");
        assert_eq!(de(id)["nivel"], "conveniente", "{id}");
        assert_eq!(de(id)["motivo"], "no-instalado", "{id}");
    }
    assert_eq!(de("rtk")["tipo"], "ejecutable");
    let f = Fixture::nueva("recursos-c9-ok");
    let claude = maquina(&f, NORMAL);
    let (_, clon) = repo(&f, None, &[]);
    let (c, out, err) = tc2_con_path(
        &f,
        &clon,
        &claude,
        &["doctor", "--recursos"],
        Some(&sin_rtk),
    );
    assert_eq!((c, err.as_str()), (0, ""), "{out}");
    assert!(out.starts_with("doctor ok fallos=0 avisos="), "{out}");
    assert!(
        out.contains("recursos engram aviso motivo=no-instalado tipo=mcp nivel=conveniente"),
        "{out}"
    );
    assert!(
        out.contains("recursos rtk aviso motivo=no-instalado tipo=ejecutable nivel=conveniente"),
        "{out}"
    );
    // Con un `rtk` falso en el PATH, `ok`.
    let bin = f.raiz.join("bin");
    escribir(&bin, if cfg!(windows) { "rtk.exe" } else { "rtk" }, "");
    let con_rtk = path_sin_rtk(Some(&bin));
    let (_, out, _) = tc2_con_path(
        &f,
        &clon,
        &claude,
        &["doctor", "--recursos", "--json"],
        Some(&con_rtk),
    );
    let v = json(&out);
    let rtk = v["hallazgos"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["id"] == "rtk")
        .unwrap_or_else(|| panic!("{out}"));
    assert_eq!(rtk["estado"], "ok", "{out}");
}

/// C10: dos ejecuciones, y otra con las carpetas creadas en otro orden, dan
/// el mismo stdout y la misma huella.
#[test]
fn determinista() {
    let mut salidas = Vec::new();
    for (i, al_reves) in [false, true].into_iter().enumerate() {
        let f = Fixture::nueva(&format!("recursos-c10-{i}"));
        let claude = maquina(&f, Maquina { al_reves, ..NORMAL });
        let (_, clon) = repo(&f, None, &[]);
        for args in [
            vec!["doctor", "--recursos"],
            vec!["doctor", "--recursos", "--json"],
        ] {
            let a = tc2(&f, &clon, &claude, &args);
            let b = tc2(&f, &clon, &claude, &args);
            assert_eq!(a, b, "{args:?}");
            salidas.push(a.1);
        }
    }
    assert_eq!(salidas[0], salidas[2]);
    assert_eq!(salidas[1], salidas[3]);
    let huella = |s: &str| {
        s.lines()
            .next()
            .unwrap()
            .rsplit_once("inventario=")
            .unwrap()
            .1
            .to_owned()
    };
    assert_eq!(huella(&salidas[0]), huella(&salidas[2]));
}

/// C1 y C8 con plugins sincronizados desde claude.ai: el habilitado entra
/// en el inventario con el prefijo de su `plugin.json`; uno en `.trash` no;
/// un `manifest.json` corrupto de otra cuenta sale `no-comprobado`.
#[test]
fn sincronizados_en_el_inventario() {
    let f = Fixture::nueva("recursos-sincronizados");
    let claude = maquina(&f, NORMAL);
    let cuenta = claude.join("plugins").join("synced").join("cuenta-a");
    escribir(
        &cuenta,
        "manifest.json",
        "{\"lastUpdated\": \"2026-11-01T00:00:00Z\", \"plugins\": [{\"name\": \"datos\", \"version\": \"3.0.0\", \"installationPreference\": \"available\", \"generation\": 2}]}",
    );
    escribir(
        &cuenta,
        "datos~g2/.claude-plugin/plugin.json",
        "{\"name\": \"data\"}",
    );
    escribir(
        &cuenta,
        "datos~g2/skills/analizar/SKILL.md",
        &skill("analizar"),
    );
    escribir(
        &claude,
        "plugins/.trash/basura/skills/z/SKILL.md",
        &skill("z"),
    );
    escribir(
        &claude,
        "plugins/synced/cuenta-rota/manifest.json",
        "{ roto",
    );
    let (_, clon) = repo(&f, None, &[]);
    let (_, out, _) = tc2(&f, &clon, &claude, &["doctor", "--recursos", "--json"]);
    let v = json(&out);
    let ids: Vec<&str> = v["inventario"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"data:analizar"), "{ids:?}");
    assert!(
        !ids.iter().any(|i| i.ends_with(":z") || *i == "z"),
        "{ids:?}"
    );
    let hs = hallazgos(&v);
    assert!(
        hs.contains(&"synced/cuenta-rota no-comprobado motivo=json-invalido".to_owned()),
        "{hs:?}"
    );
}

/// C11: `doctor --recursos x` es uso.
#[test]
fn argumentos_de_mas_son_uso() {
    let f = Fixture::nueva("recursos-c11");
    let claude = maquina(&f, NORMAL);
    let (_, clon) = repo(&f, None, &[]);
    let (c, out, err) = tc2(&f, &clon, &claude, &["doctor", "--recursos", "x"]);
    assert_eq!((c, out.as_str()), (2, "doctor uso motivo=argumentos\n"));
    assert_eq!(err.lines().count(), 1, "{err}");
}
