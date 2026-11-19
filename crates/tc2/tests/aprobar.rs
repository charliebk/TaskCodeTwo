//! F4-2 (TASK-037): `tc2 approve` contra un remoto bare de verdad.
//!
//! El rechazo del agente se prueba con el binario (sin terminal, quien lo
//! lanza sale de la cache de identidad). La aprobacion de la persona, que el
//! binario solo da con terminal y un token real, se prueba con la biblioteca
//! (`plan::aprobar_en` con `Quien::Persona`, como `tests/liberar.rs`) y una
//! firma SSH real con una clave desechable sin frase (`gpg.format=ssh` y
//! `user.signingkey` por `GIT_CONFIG_COUNT`, sin tocar la configuracion del
//! usuario).
//!
//! Como en `tests/plan.rs`: `origin` apunta a `ssh://git@github.com/o/r.git`
//! y el transporte ssh se sustituye con `GIT_SSH_COMMAND`.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use comun::{Fixture, escribir};
use tc2::contrato::Informe;
use tc2::estado::{self, Operacion};
use tc2::identidad::{Cache, Cuenta, Quien};
use tc2::plataforma::Proveedor;

const RAMA_ESTADO: &str = estado::RAMA;
const ORIGIN: &str = "ssh://git@github.com/o/r.git";
const PERSONAS: &str =
    "---\npersonas:\n  - {id: 1001, login: \"ana\"}\n  - {id: 1002, login: \"beto\"}\n";
const CONFIG: &str = "---\nformato: 2\nmodo_flujo: automatico\n";
const PLAN: &str = "TASK-001/planificacion/plan-final.md";
const FUNC: &[&str] = &[
    "Funcion",
    "--objetivo",
    "Una funcion nueva",
    "--criterio",
    "hace algo",
];

fn entorno_ssh(f: &Fixture) -> Vec<(OsString, OsString)> {
    let bare = f
        .raiz
        .join("remoto.git")
        .to_string_lossy()
        .replace('\\', "/");
    let mut e = f.entorno();
    e.push((
        "GIT_SSH_COMMAND".into(),
        format!(
            "sh -c 'case \"$2\" in *receive-pack*) exec git receive-pack \"$0\";; *) exec git upload-pack \"$0\";; esac' '{bare}'"
        )
        .into(),
    ));
    e.push(("GIT_SSH_VARIANT".into(), "simple".into()));
    e
}

/// Configuracion de firma por variables (`git -c` sin tocar ficheros):
/// `gpg.format=ssh`, `user.signingkey=<clave>` y, para ver que ninguna otra
/// operacion empieza a firmar, `commit.gpgSign=true`.
fn con_firma(mut e: Vec<(OsString, OsString)>, clave: &Path) -> Vec<(OsString, OsString)> {
    let clave = clave.to_string_lossy().replace('\\', "/");
    let pares = [
        ("gpg.format", "ssh".to_owned()),
        ("user.signingkey", clave),
        ("commit.gpgSign", "true".to_owned()),
    ];
    e.push(("GIT_CONFIG_COUNT".into(), pares.len().to_string().into()));
    for (n, (k, v)) in pares.into_iter().enumerate() {
        e.push((format!("GIT_CONFIG_KEY_{n}").into(), k.into()));
        e.push((format!("GIT_CONFIG_VALUE_{n}").into(), v.into()));
    }
    e
}

fn lanzar(dir: &Path, entorno: Vec<(OsString, OsString)>, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tc2"))
        .current_dir(dir)
        .args(args)
        .envs(entorno)
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("TC2_IDIOMA", "es")
        .env_remove("TC2_TOKEN_PERSONA")
        .env_remove("TC2_TOKEN_AGENTE")
        .output()
        .expect("lanzar tc2")
}

fn texto(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// `tc2 <args>` con el entorno dado: (codigo, la unica linea de stdout, stderr).
fn tc2_con(
    dir: &Path,
    entorno: Vec<(OsString, OsString)>,
    args: &[&str],
) -> (Option<i32>, String, String) {
    let o = lanzar(dir, entorno, args);
    let out = texto(&o.stdout);
    assert_eq!(out.lines().count(), 1, "{out}");
    (o.status.code(), out.trim_end().to_owned(), texto(&o.stderr))
}

fn tc2(f: &Fixture, dir: &Path, args: &[&str]) -> (Option<i32>, String) {
    let (c, out, _) = tc2_con(dir, entorno_ssh(f), args);
    (c, out)
}

fn valor<'a>(linea: &'a str, clave: &str) -> Option<&'a str> {
    linea
        .split(' ')
        .find_map(|kv| kv.strip_prefix(clave)?.strip_prefix('='))
}

fn ana() -> Cuenta {
    Cuenta {
        proveedor: Proveedor::GitHub,
        id: 1001,
        login: "ana".to_owned(),
    }
}

/// Remoto con personas y configuracion, la tarea de `FUNC` planificada por
/// el agente y su plan guardado; el clon `a` tiene `origin` en GitHub y la
/// identidad (cache) de ana.
fn preparar(f: &Fixture) -> (PathBuf, PathBuf) {
    let (bare, prep) = f.remoto_y_clon();
    escribir(&prep, ".taskcode/personas.yml", PERSONAS);
    escribir(&prep, ".taskcode/config.yml", CONFIG);
    f.crudo(&prep, &["add", ".taskcode"]);
    f.crudo(&prep, &["commit", "-q", "-m", "personas y config"]);
    f.crudo(&prep, &["push", "-q", "origin", "develop"]);
    let mut v = vec!["new"];
    v.extend_from_slice(FUNC);
    let o = lanzar(&prep, f.entorno(), &v);
    assert_eq!(o.status.code(), Some(0), "{}", texto(&o.stdout));
    f.crudo(&f.raiz, &["clone", "-q", bare.to_str().unwrap(), "a"]);
    let a = f.raiz.join("a");
    f.crudo(&a, &["remote", "set-url", "origin", ORIGIN]);
    Cache::en(&a.join(".git"))
        .guardar(&ana(), tc2::identidad::ahora())
        .unwrap();
    let (c, out) = tc2(f, &a, &["plan", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    (bare, a)
}

/// Publica `contenido` como `plan-final.md` con `plan guardar` (el agente).
fn guardar_plan(f: &Fixture, a: &Path, entorno: Vec<(OsString, OsString)>, contenido: &str) {
    let fuera = f.raiz.join("plan.md");
    std::fs::write(&fuera, contenido).unwrap();
    let (c, out, err) = tc2_con(
        a,
        entorno,
        &[
            "plan",
            "guardar",
            "TASK-001",
            "plan-final.md",
            fuera.to_str().unwrap(),
        ],
    );
    assert_eq!(c, Some(0), "{out} {err}");
}

fn punta(f: &Fixture, bare: &Path) -> String {
    f.crudo(bare, &["rev-parse", RAMA_ESTADO]).trim().to_owned()
}

/// Clave SSH desechable sin frase en la carpeta de la fixture.
fn clave(f: &Fixture) -> PathBuf {
    let c = f.raiz.join("clave");
    let o = Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-C", "prueba", "-f"])
        .arg(&c)
        .output()
        .expect("lanzar ssh-keygen");
    assert!(o.status.success(), "{}", texto(&o.stderr));
    c
}

fn sin_catalogo(_: &str) -> Option<Box<dyn Operacion>> {
    None
}

/// `approve` de la persona ana con la biblioteca: la linea de contrato.
fn como_persona(dir: &Path, entorno: Vec<(OsString, OsString)>) -> String {
    let g = tc2::git::Git::nuevo().con_entorno(entorno);
    let q = || -> Result<Quien, Informe> { Ok(Quien::Persona(ana())) };
    let e = tc2::toma::Entorno {
        dir,
        git: &g,
        quien: &q,
        catalogo: &sin_catalogo,
        ahora: tc2::identidad::ahora(),
    };
    tc2::plan::aprobar_en(&e, "TASK-001").linea()
}

/// Criterio 1 (E2E): el agente (sin terminal) no aprueba: `approve
/// rechazada motivo=identidad rol=persona` y el remoto intacto, tambien con
/// `--decidido-por automatico` (que ni existe: `uso`).
#[test]
fn c1_el_agente_no_aprueba_y_el_remoto_queda_intacto() {
    let f = Fixture::nueva("approve-c1");
    let (bare, a) = preparar(&f);
    guardar_plan(&f, &a, entorno_ssh(&f), "# Plan\n\nhecho\n");
    let antes = f.huella(&bare, &a);
    let (c, out, err) = tc2_con(&a, entorno_ssh(&f), &["approve", "TASK-001"]);
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(
        out,
        "approve rechazada id=TASK-001 motivo=identidad rol=persona"
    );
    assert_eq!(err.lines().count(), 1, "{err}");
    assert!(err.contains("RR-04"), "{err}");
    let (c, out) = tc2(
        &f,
        &a,
        &["approve", "TASK-001", "--decidido-por", "automatico"],
    );
    assert_eq!(
        (c, out.as_str()),
        (Some(2), "approve uso motivo=decidido-por")
    );
    // MEN-3 de la revision: un fichero de cambios que no existe es `uso`.
    let (c, out) = tc2(
        &f,
        &a,
        &["approve", "TASK-001", "--pedir-cambios", "nofichero"],
    );
    assert_eq!((c, out.as_str()), (Some(2), "approve uso motivo=no-se-lee"));
    assert_eq!(f.huella(&bare, &a), antes);
    assert!(
        f.crudo(
            &bare,
            &["show", &format!("{RAMA_ESTADO}:TASK-001/tarea.md")]
        )
        .contains("\nplan_aprobado: false\n")
    );
}

/// Criterio 3 (E2E, firma SSH real): la persona aprueba; el commit verifica
/// con `allowed_signers`, su `Plan-Blob` es el blob de `plan-final.md` de
/// ese commit, la tarea queda con `plan_aprobado: true` y la fila
/// `approve | automatico | persona`. Repetir no firma otra vez
/// (`sin-cambios`). Con `commit.gpgSign=true`, `plan guardar` sigue sin
/// firmar: solo `approve` firma.
#[test]
fn c3_la_persona_aprueba_con_su_firma_y_el_plan_blob() {
    let f = Fixture::nueva("approve-c3");
    let (bare, a) = preparar(&f);
    let clave = clave(&f);
    let firma = con_firma(entorno_ssh(&f), &clave);
    guardar_plan(&f, &a, firma.clone(), "# Plan\n\nhecho\n");
    let guardado = punta(&f, &bare);
    let cuerpo = f.crudo(&bare, &["cat-file", "-p", &guardado]);
    assert!(!cuerpo.contains("gpgsig"), "plan guardar firmo: {cuerpo}");

    let out = como_persona(&a, firma.clone());
    assert!(out.starts_with("approve ok id=TASK-001 commit="), "{out}");
    let sha = punta(&f, &bare);
    assert_eq!(valor(&out, "commit"), sha.get(..12), "{out}");
    let publica = std::fs::read_to_string(format!("{}.pub", clave.display())).unwrap();
    let permitidos = f.raiz.join("allowed_signers");
    std::fs::write(&permitidos, format!("prueba@example.com {publica}")).unwrap();
    let permitidos = permitidos.to_string_lossy().replace('\\', "/");
    let v = Command::new("git")
        .current_dir(&bare)
        .envs(f.entorno())
        .args([
            "-c",
            &format!("gpg.ssh.allowedSignersFile={permitidos}"),
            "verify-commit",
            &sha,
        ])
        .output()
        .unwrap();
    assert!(v.status.success(), "{}", texto(&v.stderr));
    assert!(texto(&v.stderr).contains("Good \"git\" signature"));
    let trailer = f.crudo(
        &bare,
        &[
            "log",
            "-1",
            "--format=%(trailers:key=Plan-Blob,valueonly)",
            &sha,
        ],
    );
    let blob = f.crudo(&bare, &["rev-parse", &format!("{sha}:{PLAN}")]);
    assert_eq!(trailer.trim(), blob.trim());
    let md = f.crudo(&bare, &["show", &format!("{sha}:TASK-001/tarea.md")]);
    assert!(md.contains("\nplan_aprobado: true\n"), "{md}");
    let fila = md.lines().last().unwrap();
    assert!(
        fila.ends_with(&format!(
            " | approve | automatico | persona | {} |",
            tc2::VERSION
        )),
        "{fila}"
    );
    // Repetir: nada que escribir ni que firmar.
    let otra = como_persona(&a, firma);
    assert!(
        otra.starts_with("approve sin-cambios id=TASK-001"),
        "{otra}"
    );
    assert_eq!(punta(&f, &bare), sha);
}

/// Criterio 3 (E2E): con una clave que no firma (no existe), `approve
/// rechazada motivo=firma` y el remoto intacto.
#[test]
fn c3_una_clave_que_no_firma_rechaza_sin_publicar() {
    let f = Fixture::nueva("approve-firma");
    let (bare, a) = preparar(&f);
    guardar_plan(&f, &a, entorno_ssh(&f), "# Plan\n\nhecho\n");
    let antes = f.huella(&bare, &a);
    let out = como_persona(&a, con_firma(entorno_ssh(&f), &f.raiz.join("no-existe")));
    assert_eq!(out, "approve rechazada id=TASK-001 motivo=firma");
    assert_eq!(f.huella(&bare, &a), antes);
}

/// Criterio 3 (E2E): el esqueleto de `plan-final.md` no se aprueba.
#[test]
fn c3_el_esqueleto_no_se_aprueba() {
    let f = Fixture::nueva("approve-esqueleto");
    let (bare, a) = preparar(&f);
    let antes = f.huella(&bare, &a);
    let clave = clave(&f);
    let out = como_persona(&a, con_firma(entorno_ssh(&f), &clave));
    assert_eq!(out, "approve rechazada id=TASK-001 motivo=plan-esqueleto");
    assert_eq!(f.huella(&bare, &a), antes);
}

/// D1a (E2E): el agente pide cambios con `--pedir-cambios` (publica
/// `cambios-pedidos-1.md`) y el siguiente `plan`, con los mismos roles, abre
/// la ronda 2.
#[test]
fn d1a_pedir_cambios_y_plan_abre_la_ronda_2() {
    let f = Fixture::nueva("approve-d1a");
    let (bare, a) = preparar(&f);
    let cambios = f.raiz.join("cambios.md");
    std::fs::write(&cambios, "Falta el plan de pruebas\n").unwrap();
    let ruta = cambios.to_str().unwrap();
    let (c, out) = tc2(&f, &a, &["approve", "TASK-001", "--pedir-cambios", ruta]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("approve ok id=TASK-001 commit="), "{out}");
    assert_eq!(
        valor(&out, "cambios-pedidos"),
        Some("cambios-pedidos-1.md"),
        "{out}"
    );
    assert_eq!(
        f.crudo(
            &bare,
            &[
                "show",
                &format!("{RAMA_ESTADO}:TASK-001/planificacion/brainstorm/cambios-pedidos-1.md")
            ]
        ),
        "Falta el plan de pruebas\n"
    );
    let (c, out) = tc2(&f, &a, &["plan", "TASK-001"]);
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "ronda"), Some("2"), "{out}");
    // Otros cambios sobre la ronda 2 abierta, si; repetirlos, sin cambios.
    let (c, out) = tc2(&f, &a, &["approve", "TASK-001", "--pedir-cambios", ruta]);
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "ronda"), Some("2"), "{out}");
    let p = punta(&f, &bare);
    let (c, out) = tc2(&f, &a, &["approve", "TASK-001", "--pedir-cambios", ruta]);
    assert_eq!(c, Some(0), "{out}");
    assert!(out.starts_with("approve sin-cambios"), "{out}");
    assert_eq!(punta(&f, &bare), p);
}

// --- F4-3 (TASK-038): `tc2 approve --hito <n>` ---

/// `approve --hito <hito>` de la persona ana con la biblioteca.
fn hito_como_persona(dir: &Path, entorno: Vec<(OsString, OsString)>, hito: u32) -> String {
    let g = tc2::git::Git::nuevo().con_entorno(entorno);
    let q = || -> Result<Quien, Informe> { Ok(Quien::Persona(ana())) };
    let e = tc2::toma::Entorno {
        dir,
        git: &g,
        quien: &q,
        catalogo: &sin_catalogo,
        ahora: tc2::identidad::ahora(),
    };
    tc2::plan::aprobar_hito_en(&e, hito).linea()
}

/// Remoto con personas y configuracion y `n` tareas del sprint 4
/// (`TASK-001`...), cada una planificada por el agente y con su plan propio
/// guardado; mas `TASK-<n+1>` en el sprint 3, planificada y con plan. El
/// clon `a` tiene `origin` en GitHub y la identidad (cache) de ana.
fn preparar_hito(f: &Fixture, n: usize) -> (PathBuf, PathBuf) {
    let (bare, prep) = f.remoto_y_clon();
    escribir(&prep, ".taskcode/personas.yml", PERSONAS);
    escribir(&prep, ".taskcode/config.yml", CONFIG);
    f.crudo(&prep, &["add", ".taskcode"]);
    f.crudo(&prep, &["commit", "-q", "-m", "personas y config"]);
    f.crudo(&prep, &["push", "-q", "origin", "develop"]);
    for i in 1..=n + 1 {
        let titulo = format!("Funcion {i}");
        let sprint = if i <= n { "4" } else { "3" };
        let o = lanzar(
            &prep,
            f.entorno(),
            &[
                "new",
                &titulo,
                "--objetivo",
                "Una funcion nueva",
                "--criterio",
                "hace algo",
                "--sprint",
                sprint,
            ],
        );
        assert_eq!(o.status.code(), Some(0), "{}", texto(&o.stdout));
    }
    f.crudo(&f.raiz, &["clone", "-q", bare.to_str().unwrap(), "a"]);
    let a = f.raiz.join("a");
    f.crudo(&a, &["remote", "set-url", "origin", ORIGIN]);
    Cache::en(&a.join(".git"))
        .guardar(&ana(), tc2::identidad::ahora())
        .unwrap();
    for i in 1..=n + 1 {
        let id = format!("TASK-{i:03}");
        let (c, out) = tc2(f, &a, &["plan", &id]);
        assert_eq!(c, Some(0), "{out}");
        let fuera = f.raiz.join(format!("plan-{i}.md"));
        std::fs::write(&fuera, format!("# Plan {i}\n\nhecho\n")).unwrap();
        let (c, out) = tc2(
            f,
            &a,
            &[
                "plan",
                "guardar",
                &id,
                "plan-final.md",
                fuera.to_str().unwrap(),
            ],
        );
        assert_eq!(c, Some(0), "{out}");
    }
    (bare, a)
}

/// C1 de TASK-038 (E2E, firma SSH real): `approve --hito 4` aprueba los
/// diez planes del sprint 4 en un unico commit nuevo, que verifica con
/// `allowed_signers`, con diez `Plan-Blob` iguales a los blobs de cada
/// `plan-final.md` de ese commit, la linea `Operacion: approve-hito|4|...`
/// y `aprobadas=` con las diez; la del sprint 3 no se toca. Cada tarea
/// lleva la misma fila `approve | automatico | persona`. Repetir no firma
/// otra vez.
#[test]
fn f4_3_approve_hito_aprueba_diez_en_un_commit_firmado() {
    let f = Fixture::nueva("approve-hito");
    let (bare, a) = preparar_hito(&f, 10);
    let clave = clave(&f);
    let firma = con_firma(entorno_ssh(&f), &clave);
    let antes = punta(&f, &bare);

    let out = hito_como_persona(&a, firma.clone(), 4);
    println!("{out}");
    assert!(out.starts_with("approve ok hito=4 commit="), "{out}");
    let ids: Vec<String> = (1..=10).map(|i| format!("TASK-{i:03}")).collect();
    assert_eq!(
        valor(&out, "aprobadas"),
        Some(ids.join(",").as_str()),
        "{out}"
    );
    let sha = punta(&f, &bare);
    assert_eq!(valor(&out, "commit"), sha.get(..12), "{out}");
    // Un unico commit nuevo sobre el de antes.
    let padre = f.crudo(&bare, &["rev-parse", &format!("{sha}^")]);
    assert_eq!(padre.trim(), antes);

    let publica = std::fs::read_to_string(format!("{}.pub", clave.display())).unwrap();
    let permitidos = f.raiz.join("allowed_signers");
    std::fs::write(&permitidos, format!("prueba@example.com {publica}")).unwrap();
    let permitidos = permitidos.to_string_lossy().replace('\\', "/");
    let v = Command::new("git")
        .current_dir(&bare)
        .envs(f.entorno())
        .args([
            "-c",
            &format!("gpg.ssh.allowedSignersFile={permitidos}"),
            "verify-commit",
            &sha,
        ])
        .output()
        .unwrap();
    assert!(v.status.success(), "{}", texto(&v.stderr));

    let trailers = f.crudo(
        &bare,
        &[
            "log",
            "-1",
            "--format=%(trailers:key=Plan-Blob,valueonly)",
            &sha,
        ],
    );
    let trailers: Vec<&str> = trailers.lines().filter(|l| !l.is_empty()).collect();
    let blobs: Vec<String> = ids
        .iter()
        .map(|id| {
            f.crudo(
                &bare,
                &[
                    "rev-parse",
                    &format!("{sha}:{id}/planificacion/plan-final.md"),
                ],
            )
            .trim()
            .to_owned()
        })
        .collect();
    assert_eq!(trailers, blobs);
    let cuerpo = f.crudo(&bare, &["log", "-1", "--format=%B", &sha]);
    assert!(
        cuerpo.contains("\nOperacion: approve-hito|4|1001|ana\n"),
        "{cuerpo}"
    );

    let mut filas = Vec::new();
    for id in &ids {
        let md = f.crudo(&bare, &["show", &format!("{sha}:{id}/tarea.md")]);
        assert!(md.contains("\nplan_aprobado: true\n"), "{md}");
        let fila = md.lines().last().unwrap().to_owned();
        assert!(
            fila.ends_with(&format!(
                " | approve | automatico | persona | {} |",
                tc2::VERSION
            )),
            "{fila}"
        );
        filas.push(fila);
    }
    filas.dedup();
    assert_eq!(filas.len(), 1, "{filas:?}");
    let otra = f.crudo(&bare, &["show", &format!("{sha}:TASK-011/tarea.md")]);
    assert!(otra.contains("\nplan_aprobado: false\n"), "{otra}");
    let cambiados = f.crudo(&bare, &["diff", "--name-only", &antes, &sha]);
    let mut esperados: Vec<String> = ids.iter().map(|id| format!("{id}/tarea.md")).collect();
    esperados.push("ROADMAP.md".to_owned());
    esperados.sort();
    let mut cambiados: Vec<String> = cambiados.lines().map(str::to_owned).collect();
    cambiados.sort();
    assert!(
        cambiados.iter().all(|c| esperados.contains(c)),
        "{cambiados:?}"
    );

    // Repetir: nada que escribir ni que firmar.
    let otra = hito_como_persona(&a, firma, 4);
    println!("{otra}");
    assert!(otra.starts_with("approve sin-cambios hito=4"), "{otra}");
    assert_eq!(punta(&f, &bare), sha);
}

/// C1 de TASK-038 (E2E): todo o nada (D2a). Con una tarea del hito aun
/// `planificada` y sin plan, `rechazada motivo=no-listas
/// tareas=TASK-004:estado-no-valido` y el remoto intacto: las otras dos,
/// listas, tampoco se aprueban ni se firma nada.
#[test]
fn f4_3_approve_hito_todo_o_nada() {
    let f = Fixture::nueva("approve-hito-nada");
    let (bare, a) = preparar_hito(&f, 2);
    let mut v = vec!["new"];
    v.extend_from_slice(FUNC);
    v.extend_from_slice(&["--sprint", "4"]);
    let (c, out) = tc2(&f, &a, &v);
    assert_eq!(c, Some(0), "{out}");
    let antes = f.huella(&bare, &a);
    let clave = clave(&f);
    let out = hito_como_persona(&a, con_firma(entorno_ssh(&f), &clave), 4);
    println!("{out}");
    assert_eq!(
        out,
        "approve rechazada hito=4 motivo=no-listas tareas=TASK-004:estado-no-valido"
    );
    assert_eq!(f.huella(&bare, &a), antes);
}

/// C2 de TASK-038 (E2E sin firma): el agente no aprueba un hito
/// (`rechazada motivo=identidad rol=persona`, remoto intacto); `--hito`
/// con ID, con `--pedir-cambios` o con un hito que no es un numero, `uso`.
#[test]
fn f4_3_approve_hito_agente_y_uso() {
    let f = Fixture::nueva("approve-hito-uso");
    let (bare, a) = preparar(&f);
    let antes = f.huella(&bare, &a);
    let (c, out, err) = tc2_con(&a, entorno_ssh(&f), &["approve", "--hito", "4"]);
    println!("{out}");
    assert_eq!(c, Some(3), "{out}");
    assert_eq!(out, "approve rechazada hito=4 motivo=identidad rol=persona");
    assert!(err.contains("RR-04"), "{err}");
    for (args, motivo) in [
        (vec!["approve", "TASK-001", "--hito", "4"], "hito-con-id"),
        (vec!["approve", "--hito", "4", "TASK-001"], "hito-con-id"),
        (
            vec!["approve", "--hito", "4", "--pedir-cambios", "x.md"],
            "hito-con-pedir-cambios",
        ),
        (vec!["approve", "--hito", "x"], "hito-invalido"),
        (vec!["approve", "--hito"], "sin-hito"),
    ] {
        let (c, out) = tc2(&f, &a, &args);
        println!("{out}");
        assert_eq!(
            (c, out.as_str()),
            (Some(2), format!("approve uso motivo={motivo}").as_str()),
            "{args:?}"
        );
    }
    assert_eq!(f.huella(&bare, &a), antes);
}
