//! F3-4: `tc2 roadmap` con el binario real sobre clones con remoto bare
//! (C2 a C6 de TASK-031) y sobre la instantanea de ToolsaWeb de TASK-030.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use comun::{Fixture, escribir};
use tc2::estado::{self, Estado, Fallo};
use tc2::plataforma::Personas;
use tc2::roadmap::{self, generar::ids_de_la_linea, mermaid, zona};
use tc2::situacion::{self, ROADMAP};
use tc2::tarea::{Cuerpo, Tarea};

const RAMA: &str = estado::RAMA;

/// (codigo, stdout, stderr) de `tc2 <args>` en `dir`.
fn tc2(f: &Fixture, dir: &Path, args: &[&str]) -> (i32, String, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_tc2"))
        .current_dir(dir)
        .args(args)
        .envs(f.entorno())
        .env("LC_ALL", "C")
        .env("TC2_IDIOMA", "es")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("lanzar tc2");
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).into_owned(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

/// `tc2 <args>` que tiene que salir con `codigo`; devuelve stdout.
fn exito(f: &Fixture, dir: &Path, args: &[&str], codigo: i32) -> String {
    let (c, out, err) = tc2(f, dir, args);
    assert_eq!(c, codigo, "tc2 {args:?}: {out} {err}");
    out
}

/// `ROADMAP.md` publicado en el remoto.
fn publicado(f: &Fixture, bare: &Path) -> String {
    let o = Command::new("git")
        .current_dir(bare)
        .args(["show", &format!("{RAMA}:{ROADMAP}")])
        .envs(f.entorno())
        .output()
        .unwrap();
    assert!(o.status.success());
    // Sin pasar por `String::from_utf8_lossy` de `crudo`: los bytes tal cual.
    String::from_utf8(o.stdout).unwrap()
}

/// Lote de `import` fuera del repo.
fn lote(f: &Fixture, nombre: &str) -> PathBuf {
    let ruta = f.raiz.join(format!("{nombre}.md"));
    std::fs::write(
        &ruta,
        format!(
            "---\nlote: {nombre}\nsprint: 2\n---\n# Plan\n\n### H1 Uno de {nombre}\n> Objetivo\n- Criterio\n\n### H2 Dos de {nombre}\n> Objetivo\n- Criterio\nDependencias: H1\n"
        ),
    )
    .unwrap();
    ruta
}

/// Clon con remoto, la rama de estado creada por un `new` y montada en
/// `tareas/`.
fn con_estado_montado(f: &Fixture) -> (PathBuf, PathBuf) {
    let (bare, clon) = f.remoto_y_clon();
    exito(f, &clon, &["new", "Primera"], 0);
    assert!(estado::montar(&f.git(), &clon).resultado.es_exito());
    (bare, clon)
}

/// C2: la zona manual editada en `tareas/ROADMAP.md` (con CRLF) se publica
/// con `--escribir` y sigue identica byte a byte tras `new`, `import` y otro
/// `--escribir`; la carpeta montada queda al dia.
#[test]
fn zona_manual_editada_sobrevive_a_new_import_y_escribir() {
    let f = Fixture::nueva("roadmap-zona");
    let (bare, clon) = con_estado_montado(&f);
    let zona_nueva = "<!-- manual:inicio (a mano) -->\r\n## Hitos\r\n\r\n### H2 — Segundo  \r\nCon \u{a0}espacios raros\t y | barras\r\n### H0 — Base\r\n<!-- manual:fin -->\r\n";
    let fichero = clon.join("tareas").join(ROADMAP);
    let antes = std::fs::read_to_string(&fichero).unwrap();
    let Ok(Some(vieja)) = zona::extraer(&antes) else {
        panic!("sin zona: {antes}")
    };
    std::fs::write(&fichero, antes.replace(vieja, zona_nueva)).unwrap();

    let out = exito(&f, &clon, &["roadmap", "--escribir"], 0);
    assert!(out.contains(" tareas=al-dia"), "{out}");
    let comprobar = |paso: &str| {
        let r = publicado(&f, &bare);
        assert_eq!(zona::extraer(&r), Ok(Some(zona_nueva)), "tras {paso}");
        assert_eq!(
            f.crudo(&clon.join("tareas"), &["status", "--porcelain"]),
            "",
            "tras {paso}"
        );
    };
    comprobar("roadmap --escribir");
    // La zona manda en el orden de los hitos y en los apartados.
    assert!(publicado(&f, &bare).contains("\n## H0 — Base\n"));
    exito(&f, &clon, &["new", "Segunda"], 0);
    comprobar("new");
    let ruta = lote(&f, "zona");
    exito(&f, &clon, &["import", ruta.to_str().unwrap()], 0);
    comprobar("import");
    let out = exito(&f, &clon, &["roadmap", "--escribir"], 0);
    assert!(out.starts_with("roadmap sin-cambios"), "{out}");
    comprobar("el segundo --escribir");
    assert!(publicado(&f, &bare).contains("\n## H2 — Segundo\n"));
    // Lo que hay en la carpeta montada es lo publicado.
    assert_eq!(
        std::fs::read_to_string(&fichero).unwrap(),
        publicado(&f, &bare)
    );
}

/// D3: una zona rota en `tareas/` se rechaza sin publicar; rota en lo
/// publicado, `new` no se bloquea y deja el ROADMAP anterior, y `next` y
/// `--comparar` lo avisan.
#[test]
fn zona_rota_no_bloquea_y_se_avisa() {
    let f = Fixture::nueva("roadmap-rota");
    let (bare, clon) = con_estado_montado(&f);
    let fichero = clon.join("tareas").join(ROADMAP);
    let roto = "# Roadmap\n<!-- manual:inicio -->\n### H1 — Uno\n";
    std::fs::write(&fichero, roto).unwrap();
    let antes = f.huella(&bare, &clon);
    let (c, out, err) = tc2(&f, &clon, &["roadmap", "--escribir"]);
    assert_eq!(c, 3, "{out}");
    assert!(
        out.starts_with("roadmap rechazada motivo=roadmap-roto roto=sin-fin"),
        "{out}"
    );
    assert!(err.contains("tareas/ROADMAP.md"), "{err}");
    // Solo se preparo el fichero de la persona; nada publicado.
    assert_eq!(
        antes.replace(" M ROADMAP.md", "M  ROADMAP.md"),
        f.huella(&bare, &clon)
    );

    // Lo publicado con la zona rota (un commit a mano en otro clon).
    let otro = f.otro_clon(&bare, "otro");
    f.crudo(&otro, &["checkout", "-q", RAMA]);
    escribir(&otro, ROADMAP, roto);
    f.crudo(&otro, &["commit", "-q", "-am", "zona rota"]);
    f.crudo(&otro, &["push", "-q", "origin", RAMA]);
    let otro_nuevo = f.otro_clon(&bare, "otro-nuevo");
    let out = exito(&f, &otro_nuevo, &["new", "Pese a todo"], 0);
    assert!(out.starts_with("new ok id=TASK-002"), "{out}");
    assert_eq!(publicado(&f, &bare), roto, "se deja el anterior");
    let out = exito(&f, &otro_nuevo, &["next"], 0);
    assert!(out.contains(" roadmap-roto=sin-fin"), "{out}");
    let out = exito(&f, &otro_nuevo, &["roadmap", "--comparar"], 3);
    assert!(
        out.starts_with("roadmap rechazada motivo=roadmap-roto roto=sin-fin"),
        "{out}"
    );
}

/// Refs del remoto y del clon.
fn refs(f: &Fixture, bare: &Path, clon: &Path) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    for (lado, dir) in [("remoto", bare), ("clon", clon)] {
        for l in f
            .crudo(dir, &["for-each-ref", "--format=%(refname) %(objectname)"])
            .lines()
        {
            let (r, sha) = l.split_once(' ').unwrap();
            m.insert(format!("{lado}:{r}"), sha.to_owned());
        }
    }
    m
}

/// Refs que cambian (o aparecen o desaparecen) entre dos fotos.
fn cambiadas(a: &BTreeMap<String, String>, b: &BTreeMap<String, String>) -> Vec<String> {
    let mut v: Vec<String> = a
        .keys()
        .chain(b.keys())
        .filter(|k| a.get(*k) != b.get(*k))
        .cloned()
        .collect();
    v.dedup();
    v
}

/// `status --porcelain` y arbol de `HEAD` del worktree principal.
fn principal(f: &Fixture, clon: &Path) -> (String, String) {
    (
        f.crudo(clon, &["status", "--porcelain"]),
        f.crudo(clon, &["rev-parse", "HEAD^{tree}"]),
    )
}

/// C3: `new`, `import` y `roadmap --escribir` solo mueven la rama de estado
/// (local y remota, y la del remoto); `roadmap` y `--comparar` no cambian
/// ninguna ref. El worktree principal no cambia nunca.
#[test]
fn solo_cambia_la_rama_de_estado() {
    let f = Fixture::nueva("roadmap-refs");
    let (bare, clon) = con_estado_montado(&f);
    let base = principal(&f, &clon);
    let ruta = lote(&f, "refs");
    let estado_refs = [
        format!("clon:refs/heads/{RAMA}"),
        format!("clon:refs/remotes/origin/{RAMA}"),
        format!("remoto:refs/heads/{RAMA}"),
    ];
    // Un commit ajeno que deja el ROADMAP desfasado, para que `--escribir`
    // tenga algo que publicar.
    let desfasar = || {
        let otro = f.raiz.join("ajeno");
        if !otro.exists() {
            f.otro_clon(&bare, "ajeno");
            f.crudo(&otro, &["checkout", "-q", RAMA]);
        }
        f.crudo(&otro, &["pull", "-q", "--ff-only", "origin", RAMA]);
        let mut t = Tarea::nueva("TASK-090", "Ajena", "2026-01-01");
        t.sprint = 1;
        escribir(&otro, "TASK-090/tarea.md", &t.escribir(&Cuerpo::leer("")));
        f.crudo(&otro, &["add", "TASK-090/tarea.md"]);
        f.crudo(&otro, &["commit", "-q", "-m", "ajena"]);
        f.crudo(&otro, &["push", "-q", "origin", RAMA]);
    };
    let pasos: Vec<(&str, Vec<&str>, bool)> = vec![
        ("new", vec!["new", "Segunda"], true),
        ("import", vec!["import", ruta.to_str().unwrap()], true),
        ("roadmap", vec!["roadmap"], false),
        ("roadmap --comparar", vec!["roadmap", "--comparar"], false),
        ("roadmap --escribir", vec!["roadmap", "--escribir"], true),
    ];
    for (nombre, args, escribe) in pasos {
        if nombre == "roadmap --escribir" {
            desfasar();
            f.crudo(&clon, &["fetch", "-q", "origin"]);
            let out = exito(&f, &clon, &["roadmap", "--comparar"], 3);
            assert!(out.contains("motivo=desfasado"), "{out}");
        }
        let antes = refs(&f, &bare, &clon);
        exito(&f, &clon, &args, 0);
        let despues = refs(&f, &bare, &clon);
        let c = cambiadas(&antes, &despues);
        if escribe {
            assert!(!c.is_empty(), "{nombre} no escribio");
            for r in &c {
                assert!(estado_refs.contains(r), "{nombre} movio {r}");
            }
        } else {
            assert!(c.is_empty(), "{nombre} movio {c:?}");
        }
        assert_eq!(principal(&f, &clon), base, "{nombre} toco el principal");
        assert_eq!(
            f.crudo(&clon.join("tareas"), &["status", "--porcelain"]),
            ""
        );
    }
    exito(&f, &clon, &["roadmap", "--comparar"], 0);
}

/// Publica en `bare` el juego de tareas de `tests/siguiente.rs` (ilegible,
/// descartada, H2 antes que H1) sin pasar por `tc2`.
fn publicar_estado(f: &Fixture, bare: &Path) {
    let dir = f.raiz.join("estado");
    f.crudo(&f.raiz, &["init", "-q", "-b", RAMA, "estado"]);
    let tarea = |id: &str, sprint: u32, estado: &str, deps: &[&str]| {
        let mut t = Tarea::nueva(id, &format!("Titulo de {id}"), "2026-01-01");
        t.sprint = sprint;
        t.estado = estado.to_owned();
        if estado == "descartada" {
            t.motivo = Some("ya no hace falta".to_owned());
        }
        t.dependencias = deps.iter().map(|d| (*d).to_owned()).collect();
        t.escribir(&Cuerpo::leer(""))
    };
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
        ("TASK-009", tarea("TASK-009", 2, "en-revision", &[])),
        (
            "TASK-010",
            tarea("TASK-010", 2, "planificada", &["TASK-005"]),
        ),
    ] {
        escribir(&dir, &format!("{id}/tarea.md"), &md);
    }
    escribir(
        &dir,
        ROADMAP,
        "# Roadmap\n\n<!-- manual:inicio -->\n### H2 — Antes\n### H1 — Despues\n<!-- manual:fin -->\n",
    );
    f.crudo(&dir, &["add", "-A"]);
    f.crudo(&dir, &["commit", "-q", "-m", "estado"]);
    f.crudo(&dir, &["push", "-q", bare.to_str().unwrap(), RAMA]);
}

/// Valor de `clave=` (lista separada por comas) de una linea de contrato.
fn lista(linea: &str, clave: &str) -> Vec<String> {
    linea
        .split_whitespace()
        .find_map(|kv| kv.strip_prefix(clave)?.strip_prefix('='))
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

/// C4 (repo de prueba) y C5: «Listas para empezar» y «En marcha» del
/// ROADMAP son `listas=` y `en-marcha=` de `tc2 next`; `--escribir` dos
/// veces seguidas da `sin-cambios` la segunda, y entonces lo publicado es lo
/// que imprime `roadmap`.
#[test]
fn roadmap_y_next_coinciden_y_escribir_es_idempotente() {
    let f = Fixture::nueva("roadmap-next");
    let (bare, clon) = f.remoto_y_clon();
    publicar_estado(&f, &bare);
    f.crudo(&clon, &["fetch", "-q", "origin"]);
    let next = exito(&f, &clon, &["next"], 0);
    let impreso = exito(&f, &clon, &["roadmap"], 0);
    assert_eq!(
        ids_de_la_linea(&impreso, "Listas para empezar"),
        Some(lista(&next, "listas"))
    );
    assert_eq!(
        ids_de_la_linea(&impreso, "En marcha"),
        Some(lista(&next, "en-marcha"))
    );
    assert_eq!(lista(&next, "listas"), ["TASK-006", "TASK-002"]);
    assert_eq!(lista(&next, "en-marcha"), ["TASK-009", "TASK-003"]);
    // Sin `--escribir`, lo publicado (de otra herramienta) no esta al dia.
    let out = exito(&f, &clon, &["roadmap", "--comparar"], 3);
    assert!(
        out.starts_with("roadmap rechazada motivo=desfasado"),
        "{out}"
    );

    let primera = exito(&f, &clon, &["roadmap", "--escribir"], 0);
    assert!(primera.starts_with("roadmap ok "), "{primera}");
    let segunda = exito(&f, &clon, &["roadmap", "--escribir"], 0);
    assert!(segunda.starts_with("roadmap sin-cambios"), "{segunda}");
    exito(&f, &clon, &["roadmap", "--comparar"], 0);
    assert_eq!(exito(&f, &clon, &["roadmap"], 0), publicado(&f, &bare));
    assert_eq!(impreso, publicado(&f, &bare));
    // `--json` lleva el texto y el commit.
    let j = exito(&f, &clon, &["roadmap", "--json"], 0);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert_eq!(v["claves"]["texto"], impreso.as_str());
    assert_eq!(v["claves"]["commit"].as_str().map(str::len), Some(12));
}

/// Uso: argumentos de mas o desconocidos, con el diagnostico en es y en.
#[test]
fn roadmap_uso() {
    let f = Fixture::nueva("roadmap-uso");
    for args in [
        vec!["roadmap", "x"],
        vec!["roadmap", "--comparar", "--escribir"],
    ] {
        let (c, out, err) = tc2(&f, &f.raiz, &args);
        assert_eq!(
            (c, out.as_str(), err.as_str()),
            (
                2,
                "roadmap uso motivo=argumentos\n",
                "uso: roadmap [--comparar | --escribir] [--json]\n"
            )
        );
    }
    let s = tc2::ejecutar_en(tc2::i18n::Idioma::En, ["roadmap", "x"]);
    assert_eq!(
        s.stderr,
        "usage: roadmap [--comparar | --escribir] [--json]\n"
    );
    let (c, out, _) = tc2(&f, &f.raiz, &["roadmap"]);
    assert_eq!(c, 3, "{out}");
}

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

const TW_TAREAS: &str = include_str!("fixtures/toolsaweb-1fc94425/tareas.tsv");
const TW_HITOS: &str = include_str!("fixtures/toolsaweb-1fc94425/hitos.txt");
const TW_ESPERADO: &str = include_str!("fixtures/toolsaweb-1fc94425/esperado.txt");

/// Titulo de 60 caracteres (C6).
fn titulo_60(id: &str) -> String {
    let mut t = format!("{id} — Título largo de prueba con ñ para medir");
    while t.chars().count() < 60 {
        t.push('x');
    }
    t
}

/// La instantanea de ToolsaWeb (TASK-030) como estado: un `tarea.md` de
/// formato 2 por fila de `tareas.tsv` y la zona manual con `hitos.txt`.
fn toolsaweb() -> EnMemoria {
    let mut m = BTreeMap::new();
    for l in TW_TAREAS
        .lines()
        .map(|l| l.trim_end_matches('\r'))
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let c: Vec<&str> = l.split('\t').collect();
        let mut t = Tarea::nueva(c[0], &titulo_60(c[0]), "2026-01-01");
        t.sprint = c[1].parse().unwrap();
        if c[3] == "si" {
            t.estado = "descartada".to_owned();
            t.motivo = Some("descartada".to_owned());
        } else {
            t.estado = c[2].to_owned();
        }
        t.dependencias = c
            .get(4)
            .unwrap_or(&"")
            .split(',')
            .filter(|d| !d.is_empty())
            .map(str::to_owned)
            .collect();
        m.insert(format!("{}/tarea.md", c[0]), t.escribir(&Cuerpo::leer("")));
    }
    let hitos: String = TW_HITOS
        .lines()
        .filter(|l| !l.starts_with('#') || l.starts_with("###"))
        .map(|l| format!("{l}\n"))
        .collect();
    m.insert(
        ROADMAP.to_owned(),
        format!("<!-- manual:inicio -->\n## Hitos\n\n{hitos}<!-- manual:fin -->\n"),
    );
    EnMemoria(m)
}

/// C4 (ToolsaWeb): las lineas del ROADMAP son las de `next` (la misma
/// situacion) y las del ROADMAP de ToolsaWeb en ese commit.
#[test]
fn toolsaweb_listas_y_en_marcha_como_next_y_su_roadmap() {
    let e = toolsaweb();
    let s = situacion::calcular(&e).unwrap();
    assert_eq!(s.fichas.len(), 404);
    let r = roadmap::de_estado(&e, &Personas::default()).unwrap();
    let listas = ids_de_la_linea(&r, "Listas para empezar").unwrap();
    let marcha = ids_de_la_linea(&r, "En marcha").unwrap();
    assert_eq!(listas, s.listas);
    assert_eq!(marcha, s.en_marcha);
    assert_eq!(listas.len(), 54);
    assert_eq!(
        Some(listas),
        ids_de_la_linea(TW_ESPERADO, "Listas para empezar")
    );
    assert_eq!(Some(marcha), ids_de_la_linea(TW_ESPERADO, "En marcha"));
}

/// C6: con la instantanea de ToolsaWeb y titulos de 60 caracteres el
/// fichero no pasa de 100 KB y cada bloque Mermaid (uno por hito abierto,
/// todos dibujados) tiene menos de 500 aristas y de 50 000 caracteres.
#[test]
fn toolsaweb_cabe() {
    let r = roadmap::de_estado(&toolsaweb(), &Personas::default()).unwrap();
    assert!(r.len() <= 100 * 1024, "{} bytes", r.len());
    let hitos_abiertos = r
        .lines()
        .filter(|l| {
            l.strip_prefix("## H")
                .is_some_and(|x| x.starts_with(|c: char| c.is_ascii_digit()))
        })
        .count();
    let bloques: Vec<&str> = r.split("```mermaid\n").skip(1).collect();
    assert!(hitos_abiertos > 1);
    assert_eq!(bloques.len(), hitos_abiertos, "un grafo por hito abierto");
    for b in bloques {
        let b = &b[..b.find("```").unwrap()];
        let aristas = b.lines().filter(|l| l.contains(" --> ")).count();
        assert!(aristas < mermaid::MAX_ARISTAS, "{aristas} aristas");
        assert!(b.chars().count() < mermaid::MAX_CARACTERES);
    }
    eprintln!(
        "ToolsaWeb: {} bytes, {hitos_abiertos} hitos abiertos",
        r.len()
    );
}
