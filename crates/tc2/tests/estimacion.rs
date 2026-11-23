//! F3-13 (TASK-083): estimacion por fase de cada tarea. Paridad con la PoC
//! de F1-17 sobre un corpus fijo (el fixture lo genero `poc-f1-17.py` una
//! vez; aqui no hace falta Python) y E2E con el binario real contra un
//! remoto bare, como `tests/toma.rs` (GitHub por el host y el transporte ssh
//! sustituido por `GIT_SSH_COMMAND`).

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::path::{Path, PathBuf};
use std::process::Output;

use comun::remoto::{PERSONAS, identificar};
use comun::{Fixture, escribir, texto, valor};
use tc2::carril::Carril;
use tc2::contrato::Resultado;
use tc2::estado::{self, Cambio, Contexto, Estado, Fallo, Leida, Operacion};
use tc2::estimacion::{
    self, Commits, Estimacion, Objetivo, PorFase, Rango, eventos, historico, real::real,
};
use tc2::tarea::{self, Cuerpo, Tarea};

const RAMA_ESTADO: &str = estado::RAMA;

// ---------------------------------------------------------------- paridad (C1, C2)

const FIXTURE: &str = include_str!("fixtures/estimacion/paridad-poc-f1-17.json");

fn leidas_de(tareas: &serde_json::Map<String, serde_json::Value>) -> Vec<Leida> {
    tareas
        .iter()
        .map(|(ruta, t)| (ruta.clone(), tarea::leer(t.as_str().unwrap())))
        .collect()
}

fn f64_de(v: &serde_json::Value) -> f64 {
    v.as_f64().unwrap()
}

/// C2 (y C1 con su corpus): sobre los corpus que pasaron por
/// `poc-f1-17.py`, el real de cada tarea, los grupos de troceo, los
/// descartes y la mediana y el p80 de cada fase y del total, con su nivel
/// y `n`, coinciden exactamente (sin tolerancia) con los de la PoC.
#[test]
fn c2_paridad_exacta_con_la_poc() {
    let v: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    assert_eq!(v["umbral"], 45);
    assert_eq!(v["n_min"], estimacion::N_MIN);
    let mut comprobadas = 0;
    for c in v["corpus"].as_array().unwrap() {
        let nombre = c["nombre"].as_str().unwrap();
        let leidas = leidas_de(c["tareas"].as_object().unwrap());
        assert!(
            leidas.iter().all(|(_, d)| d.is_ok()),
            "{nombre}: {leidas:?}"
        );
        let commits = Commits::de_log(c["log"].as_str().unwrap());
        let h = historico(&leidas, &commits);
        // Real de cada tarea usable, antes de agrupar.
        for (id, r) in c["reales"].as_object().unwrap() {
            let (_, doc) = leidas
                .iter()
                .find(|(ruta, _)| ruta.starts_with(id.as_str()))
                .unwrap();
            let cuerpo = match doc.as_ref().unwrap() {
                tarea::Documento::Formato2(_, c) => c,
                tarea::Documento::Formato1(_, c) => c,
            };
            let mio = real(&eventos(cuerpo, commits.de_tarea(id))).unwrap();
            for f in estimacion::FASES {
                assert_eq!(mio.fase(f), Some(f64_de(&r[f])), "{nombre} {id} {f}");
            }
            comprobadas += 1;
        }
        // Descartes por motivo.
        let descartes: Vec<(String, Vec<String>)> = h
            .descartes
            .iter()
            .map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect();
        let esperados: Vec<(String, Vec<String>)> = c["descartes"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.as_str().unwrap().to_owned())
                        .collect(),
                )
            })
            .collect();
        assert_eq!(descartes, esperados, "{nombre}");
        // Grupos de troceo con su real sumado.
        let grupos = c["grupos"].as_array().unwrap();
        assert_eq!(h.muestras.len(), grupos.len(), "{nombre}");
        for g in grupos {
            let m = h
                .muestras
                .iter()
                .find(|m| m.id == g["id"].as_str().unwrap())
                .unwrap_or_else(|| panic!("{nombre}: falta el grupo {}", g["id"]));
            for f in ["implementacion", "rondas", "cierre", "total"] {
                assert_eq!(m.real.fase(f), Some(f64_de(&g[f])), "{nombre} {} {f}", m.id);
            }
        }
        // Estimaciones: nivel, n, mediana y p80 por fase y total.
        for e in c["estimaciones"].as_array().unwrap() {
            let o = &e["objetivo"];
            let obj = Objetivo {
                id: "TASK-999".to_owned(),
                carril: o["carril"].as_str().and_then(Carril::desde),
                complejidad: o["complejidad"].as_str().map(str::to_owned),
                criterios: usize::try_from(o["criterios"].as_u64().unwrap()).unwrap(),
            };
            let mia = estimacion::estimar(&obj, &h.muestras, "t");
            assert_eq!(mia.nivel, e["nivel"].as_str().unwrap(), "{nombre} {o}");
            assert_eq!(mia.n as u64, e["n"].as_u64().unwrap(), "{nombre} {o}");
            if mia.sin_datos() {
                assert!(mia.por_fase.is_none() && mia.total.is_none());
                continue;
            }
            let p = mia.por_fase.unwrap();
            for f in estimacion::FASES {
                let r = p.fase(f).unwrap();
                assert_eq!(
                    (r.mediana, r.p80),
                    (f64_de(&e[f][0]), f64_de(&e[f][1])),
                    "{nombre} {o} {f}"
                );
            }
            let t = mia.total.unwrap();
            assert_eq!(
                (t.mediana, t.p80),
                (f64_de(&e["total"][0]), f64_de(&e["total"][1])),
                "{nombre} {o} total"
            );
        }
    }
    assert!(comprobadas >= 20, "{comprobadas}");
    // El corpus de C2: 6 del mismo carril, complejidad y tamano
    // (implementacion de 12 a 60) y una con duracion negativa.
    let c2 = &v["corpus"][0];
    assert_eq!(c2["estimaciones"][0]["nivel"], "carril+complejidad+tamano");
    assert_eq!(c2["estimaciones"][0]["n"], 6);
    assert_eq!(c2["descartes"]["duracion-negativa"][0], "TASK-007");
    let cascada = &v["corpus"][1];
    assert_eq!(cascada["estimaciones"][0]["nivel"], "carril");
    assert_eq!(cascada["estimaciones"][0]["n"], 5);
}

/// C1: el caso del enunciado, con la PoC: con el commit de las 11:40 la
/// implementacion es 85 (la regla de F1-17); el 75 del enunciado es el de la
/// misma tarea sin ese commit. Rondas 30 y cierre 10 en los dos.
#[test]
fn c1_real_del_enunciado_igual_que_la_poc() {
    let v: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    for caso in v["c1"].as_array().unwrap() {
        let mut filas = String::new();
        for t in caso["transiciones"].as_array().unwrap() {
            filas.push_str(&format!(
                "| {} | {} | manual | persona | 0.1 |\n",
                t[0].as_str().unwrap(),
                t[1].as_str().unwrap()
            ));
        }
        let cuerpo = Cuerpo::leer(&format!(
            "## Transiciones\n\n| fecha | fase | modo | decidido_por | version |\n|---|---|---|---|---|\n{filas}"
        ));
        let commits: Vec<i64> = caso["commits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| estimacion::real::segundos_de(c.as_str().unwrap()).unwrap())
            .collect();
        let r = real(&eventos(&cuerpo, commits)).unwrap();
        for f in ["implementacion", "rondas", "cierre", "total"] {
            assert_eq!(r.fase(f), Some(f64_de(&caso[f])), "{} {f}", caso["nombre"]);
        }
    }
    assert_eq!(v["c1"][0]["implementacion"], 85.0);
    assert_eq!(v["c1"][1]["implementacion"], 75.0);
}

// ---------------------------------------------------------------- E2E

fn tc2(f: &Fixture, dir: &Path, args: &[&str]) -> Output {
    comun::orden_tc2()
        .current_dir(dir)
        .args(args)
        .envs(f.entorno_ssh())
        .output()
        .expect("lanzar tc2")
}

/// (codigo, stdout, stderr) de `tc2 <args>`.
fn correr(f: &Fixture, dir: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let o = tc2(f, dir, args);
    (o.status.code(), texto(&o.stdout), texto(&o.stderr))
}

/// Remoto con `personas.yml` en `develop` y el clon `a` de ana.
fn preparar(f: &Fixture) -> (PathBuf, PathBuf) {
    let (bare, prep) = f.remoto_y_clon();
    escribir(&prep, ".taskcode/personas.yml", PERSONAS);
    f.crudo(&prep, &["add", ".taskcode/personas.yml"]);
    f.crudo(&prep, &["commit", "-q", "-m", "personas"]);
    f.crudo(&prep, &["push", "-q", "origin", "develop"]);
    let a = f.clon_github(&bare, "a");
    identificar(&a, 1001, "ana");
    (bare, a)
}

/// Escribe ficheros en la rama de estado por la transaccion.
struct Poner(Vec<(String, String)>);

impl Operacion for Poner {
    fn nombre(&self) -> &str {
        "poner"
    }
    fn anotacion(&self) -> String {
        "poner".to_owned()
    }
    fn exige_red(&self) -> bool {
        true
    }
    fn aplicar(&self, _: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        Ok(self
            .0
            .iter()
            .map(|(r, t)| Cambio::nuevo(r.clone(), t.clone()))
            .collect())
    }
}

fn sin_catalogo(_: &str) -> Option<Box<dyn Operacion>> {
    None
}

fn poner(f: &Fixture, dir: &Path, ficheros: &[(String, String)]) {
    let g = tc2::git::Git::nuevo().con_entorno(f.entorno_ssh());
    let i = estado::transaccion(
        &Contexto::nuevo(&g, dir, &sin_catalogo, None),
        &Poner(ficheros.to_vec()),
    );
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
}

fn publicado(f: &Fixture, bare: &Path, id: &str) -> String {
    f.crudo(bare, &["show", &format!("{RAMA_ESTADO}:{id}/tarea.md")])
}

/// 2026-10-01T09:00:00Z.
const BASE: u64 = 1_790_845_200;

fn fila(ts: u64, fase: &str) -> String {
    format!(
        "| {} | {fase} | manual | persona | 0.1 |\n",
        tc2::toma::instante(ts)
    )
}

/// Una terminada de formato 2 (carril funcionalidad, complejidad media, dos
/// criterios) con `imp` minutos de implementacion (sin huecos de mas de 45),
/// 10 de rondas y 5 de cierre, el dia `dia` desde [`BASE`].
fn terminada(id: &str, dia: u64, imp: u64) -> (String, String) {
    let mut t = Tarea::nueva(id, &format!("Hecha {id}"), "2026-10-01");
    "terminada".clone_into(&mut t.estado);
    t.carril = Some("funcionalidad".to_owned());
    t.complejidad = Some("media".to_owned());
    t.plan_aprobado = true;
    let s = BASE + dia * 86_400;
    let r1 = s + imp * 60;
    let cuerpo = format!(
        "## Objetivo\n\nHecha.\n\n## Criterios de aceptacion\n\n- [x] a\n- [x] b\n\n## Transiciones\n\n| fecha | fase | modo | decidido_por | version |\n|---|---|---|---|---|\n{}{}{}{}",
        fila(s, "start"),
        fila(r1, "review"),
        fila(r1 + 600, "review"),
        fila(r1 + 900, "finish"),
    );
    (format!("{id}/tarea.md"), t.escribir(&Cuerpo::leer(&cuerpo)))
}

/// El bloque `estimacion:` de un `tarea.md`.
fn bloque(md: &str) -> String {
    let i = md.find("\nestimacion:").unwrap() + 1;
    let fin = md[i..].find("\ntokens_diseno:").unwrap() + i + 1;
    md[i..fin].to_owned()
}

fn leer2(md: &str) -> (Tarea, Cuerpo) {
    match tarea::leer(md) {
        Ok(tarea::Documento::Formato2(t, c)) => (t, c),
        otro => panic!("{otro:?}"),
    }
}

/// C3: con menos de 5 terminadas parecidas en todos los niveles,
/// `tc2 estimar TASK-NNN` dice «sin tareas parecidas (n=3<5)» sin cifras, y
/// el `tarea.md` de `new` lleva `nivel: sin-datos`, `n`, `por_fase: null` y
/// `total: null`.
#[test]
fn c3_sin_datos_nunca_un_numero_inventado() {
    let f = Fixture::nueva("estimacion-c3");
    let (bare, a) = preparar(&f);
    let tres: Vec<(String, String)> = (1..=3)
        .map(|i| terminada(&format!("TASK-{i:03}"), i, 10 * i))
        .collect();
    poner(&f, &a, &tres);
    let (c, out, err) = correr(&f, &a, &["new", "Abierta", "--criterio", "uno"]);
    assert_eq!(c, Some(0), "{out} {err}");
    assert_eq!(valor(&out, "id"), Some("TASK-004"), "{out}");
    assert_eq!(valor(&out, "estimacion"), Some("sin-datos"), "{out}");
    assert_eq!(valor(&out, "estimacion-n"), Some("3"), "{out}");
    assert_eq!(valor(&out, "estimacion-p80"), None, "{out}");
    let md = publicado(&f, &bare, "TASK-004");
    let b = bloque(&md);
    assert!(
        b.contains("\n  nivel: \"sin-datos\"\n  n: 3\n  por_fase: null\n  total: null\n"),
        "{b}"
    );
    let (c, out, err) = correr(&f, &a, &["estimar", "TASK-004"]);
    assert_eq!(c, Some(0), "{out} {err}");
    let lineas: Vec<&str> = out.lines().collect();
    assert_eq!(lineas.len(), 2, "{out}");
    assert!(
        lineas[0].starts_with("estimar ok id=TASK-004 estimacion=sin-datos estimacion-n=3 "),
        "{out}"
    );
    assert_eq!(lineas[1], "estimacion: sin tareas parecidas (n=3<5)");
    assert!(!lineas[1].chars().any(|c| c == '≈'), "{out}");
    // En JSON, una linea con las claves y sin la de la persona.
    let (c, out, _) = correr(&f, &a, &["estimar", "--json", "TASK-004"]);
    assert_eq!((c, out.lines().count()), (Some(0), 1), "{out}");
    let j: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(j["claves"]["estimacion"], "sin-datos");
    assert_eq!(j["claves"]["estimacion-nivel"], "sin-datos");
}

/// C4: tras `new` e `import`, cada `tarea.md` creado tiene `estimacion` no
/// nula y se relee sin error; tras `tc2 estimar`, ninguna abierta queda con
/// `estimacion: null` y las terminadas quedan byte a byte (y una abierta que
/// ya la tenia, tambien).
#[test]
fn c4_new_import_y_estimar_rellenan_sin_tocar_las_terminadas() {
    let f = Fixture::nueva("estimacion-c4");
    let (bare, a) = preparar(&f);
    let mut ficheros: Vec<(String, String)> = (1..=6)
        .map(|i| terminada(&format!("TASK-{i:03}"), i, 5 * i + 5))
        .collect();
    // Abiertas de antes de F3-13: con `estimacion: null`, y una descartada.
    for (id, estado) in [
        ("TASK-007", "planificada"),
        ("TASK-008", "en-curso"),
        ("TASK-009", "descartada"),
    ] {
        let mut t = Tarea::nueva(id, &format!("Vieja {id}"), "2026-11-01");
        estado.clone_into(&mut t.estado);
        if estado == "descartada" {
            t.motivo = Some("no".to_owned());
        }
        ficheros.push((format!("{id}/tarea.md"), t.escribir(&Cuerpo::plantilla())));
    }
    poner(&f, &a, &ficheros);
    // new
    let (c, out, err) = correr(
        &f,
        &a,
        &["new", "Nueva", "--criterio", "uno", "--criterio", "dos"],
    );
    assert_eq!(c, Some(0), "{out} {err}");
    assert_eq!(valor(&out, "estimacion-nivel"), Some("repo"), "{out}");
    assert_eq!(valor(&out, "estimacion-n"), Some("6"), "{out}");
    let md = publicado(&f, &bare, "TASK-010");
    let (t, _) = leer2(&md);
    let e = Estimacion::leer(t.estimacion().unwrap()).unwrap();
    assert_eq!((e.nivel.as_str(), e.n), ("repo", 6));
    assert_eq!(
        valor(&out, "estimacion").map(str::to_owned),
        e.total.map(|r| estimacion::minutos(r.mediana).to_string())
    );
    // import de tres
    let lote = "---\nlote: c4\nsprint: 1\n---\n# Plan\n\n### A Uno\n> Objetivo uno\n- Criterio a\n\n### B Dos\n> Objetivo dos\n- Criterio b\n\n### C Tres\n> Objetivo tres\n- Criterio c\n";
    let ruta = f.raiz.join("c4.md");
    std::fs::write(&ruta, lote).unwrap();
    let (c, out, err) = correr(&f, &a, &["import", ruta.to_str().unwrap()]);
    assert_eq!(c, Some(0), "{out} {err}");
    for id in ["TASK-011", "TASK-012", "TASK-013"] {
        let (t, _) = leer2(&publicado(&f, &bare, id));
        let e = Estimacion::leer(t.estimacion().unwrap()).unwrap();
        assert_eq!((e.nivel.as_str(), e.n), ("repo", 6), "{id}");
    }
    let antes: Vec<String> = (1..=13)
        .map(|i| publicado(&f, &bare, &format!("TASK-{i:03}")))
        .collect();
    // estimar
    let (c, out, err) = correr(&f, &a, &["estimar"]);
    assert_eq!(c, Some(0), "{out} {err}");
    assert!(
        out.starts_with("estimar ok actualizadas=2 ids=TASK-007,TASK-008 commit="),
        "{out}"
    );
    assert_eq!(out.lines().count(), 1, "{out}");
    for i in 1..=13 {
        let id = format!("TASK-{i:03}");
        let md = publicado(&f, &bare, &id);
        let (t, _) = leer2(&md);
        if ["TASK-007", "TASK-008"].contains(&id.as_str()) {
            assert!(t.estimacion().is_some(), "{id}");
            // Solo cambia el bloque de la estimacion.
            let viejo = &antes[i - 1];
            assert_eq!(
                md.replace(&bloque(&md), "estimacion: null\n"),
                *viejo,
                "{id}"
            );
        } else {
            assert_eq!(md, antes[i - 1], "{id} cambio");
        }
        if !["terminada", "descartada"].contains(&t.estado.as_str()) {
            assert!(t.estimacion().is_some(), "{id} sigue con estimacion: null");
        }
    }
    // Repetirla no cambia nada.
    let (c, out, _) = correr(&f, &a, &["estimar"]);
    assert_eq!(c, Some(0), "{out}");
    assert!(
        out.starts_with("estimar sin-cambios actualizadas=0"),
        "{out}"
    );
}

/// C5: tras terminar una muestra mas, `start` recalcula (`calculada` a la
/// hora del `start`, `n` sube en 1) y el resto del `tarea.md` queda como lo
/// deja la toma; `pausa`, `reanudar`, `reasignar` y `liberar` no cambian
/// `estimacion`.
#[test]
fn c5_start_recalcula_y_el_ciclo_no_la_toca() {
    let f = Fixture::nueva("estimacion-c5");
    let (bare, a) = preparar(&f);
    let cinco: Vec<(String, String)> = (1..=5)
        .map(|i| terminada(&format!("TASK-{i:03}"), i, 5 * i + 5))
        .collect();
    poner(&f, &a, &cinco);
    let (c, out, err) = correr(&f, &a, &["new", "Abierta", "--criterio", "uno"]);
    assert_eq!(c, Some(0), "{out} {err}");
    assert_eq!(valor(&out, "estimacion-n"), Some("5"), "{out}");
    // Termina una mas.
    let sexta = terminada("TASK-007", 7, 40);
    poner(&f, &a, std::slice::from_ref(&sexta));
    let antes = publicado(&f, &bare, "TASK-006");
    let t0 = tc2::identidad::ahora();
    let (c, out, err) = correr(&f, &a, &["start", "TASK-006"]);
    let t1 = tc2::identidad::ahora();
    assert_eq!(c, Some(0), "{out} {err}");
    assert!(err.is_empty(), "{err}");
    assert_eq!(out.lines().count(), 1, "{out}");
    assert_eq!(valor(&out, "estimacion-n"), Some("6"), "{out}");
    assert_eq!(valor(&out, "estimacion-nivel"), Some("repo"), "{out}");
    let despues = publicado(&f, &bare, "TASK-006");
    let nuevo = bloque(&despues);
    let e = Estimacion::leer(&nuevo).unwrap();
    assert_eq!(e.n, Estimacion::leer(&bloque(&antes)).unwrap().n + 1);
    let cuando = estimacion::real::segundos_de(&e.calculada).unwrap();
    assert!(
        (t0 as i64..=t1 as i64).contains(&cuando),
        "{} fuera de [{t0}, {t1}]",
        e.calculada
    );
    // La fila de la toma lleva la misma hora que `calculada`.
    let fila = despues.lines().last().unwrap();
    assert!(
        fila.starts_with(&format!("| {} | start |", e.calculada)),
        "{fila}"
    );
    let actualizado = antes
        .lines()
        .find_map(|l| l.strip_prefix("actualizado: "))
        .unwrap();
    let esperado = antes
        .replacen("estado: \"planificada\"", "estado: \"en-curso\"", 1)
        .replacen(
            "asignado_a: null",
            "asignado_a: {id: 1001, login: \"ana\"}",
            1,
        )
        .replacen(
            &format!("actualizado: {actualizado}"),
            &format!("actualizado: \"{}\"", &e.calculada[..10]),
            1,
        )
        .replacen(&bloque(&antes), &nuevo, 1);
    assert_eq!(despues, format!("{esperado}{fila}\n"));
    // Las terminadas, byte a byte.
    for (ruta, md) in cinco.iter().chain(std::iter::once(&sexta)) {
        let id = ruta.trim_end_matches("/tarea.md");
        assert_eq!(&publicado(&f, &bare, id), md, "{id}");
    }
    // El ciclo no toca la estimacion (la rama, publicada: si no, `reasignar`
    // no deja la tarea).
    f.git_ssh(&a, &["push", "-q", "origin", "feature/task-006-abierta"]);
    for args in [
        vec!["pausa", "TASK-006"],
        vec!["reanudar", "TASK-006"],
        vec!["reasignar", "TASK-006", "beto"],
    ] {
        let (c, out, err) = correr(&f, &a, &args);
        assert_eq!(c, Some(0), "{args:?}: {out} {err}");
        assert_eq!(valor(&out, "estimacion"), None, "{out}");
        assert_eq!(bloque(&publicado(&f, &bare, "TASK-006")), nuevo, "{args:?}");
    }
    let b = f.clon_github(&bare, "b");
    identificar(&b, 1002, "beto");
    let (c, out, err) = correr(&f, &b, &["liberar", "TASK-006"]);
    assert_eq!(c, Some(0), "{out} {err}");
    let md = publicado(&f, &bare, "TASK-006");
    assert!(md.contains("\nasignado_a: null\n"), "{md}");
    assert_eq!(bloque(&md), nuevo, "liberar");
}

/// La tarea de C6: terminada con la estimacion `por_fase.implementacion:
/// [30, 52]` (si `con_estimacion`) y la historia de C1 sin el commit de las
/// 11:40 (el evento de las 10:20 es una fila de coste): real 75, 30 y 10.
fn tarea_c6(id: &str, con_estimacion: bool) -> (String, String) {
    let mut t = Tarea::nueva(id, &format!("Medida {id}"), "2026-11-17");
    "terminada".clone_into(&mut t.estado);
    if con_estimacion {
        t.poner_estimacion(&Estimacion {
            calculada: "2026-11-17T09:00:00Z".to_owned(),
            nivel: "repo".to_owned(),
            n: 6,
            por_fase: Some(PorFase {
                implementacion: Some(Rango {
                    mediana: 30.0,
                    p80: 52.0,
                }),
                rondas: Some(Rango {
                    mediana: 30.0,
                    p80: 40.0,
                }),
                cierre: Some(Rango {
                    mediana: 10.0,
                    p80: 12.0,
                }),
            }),
            total: Some(Rango {
                mediana: 70.0,
                p80: 90.0,
            }),
            agente: None,
        });
    }
    let h = |hh: &str| format!("2026-11-17T{hh}:00Z");
    let filas: String = [
        ("10:00", "start"),
        ("10:30", "pausa"),
        ("11:30", "reanudar"),
        ("13:00", "review"),
        ("13:30", "review"),
        ("13:40", "finish"),
    ]
    .iter()
    .map(|(x, fase)| format!("| {} | {fase} | manual | persona | 0.1 |\n", h(x)))
    .collect();
    let cuerpo = format!(
        "## Transiciones\n\n| fecha | fase | modo | decidido_por | version |\n|---|---|---|---|---|\n{filas}\n## Coste\n\n| fecha | fase | fuente | total |\n|---|---|---|---|\n| {} | implementacion | sesion:x | 100 |\n",
        h("10:20")
    );
    (format!("{id}/tarea.md"), t.escribir(&Cuerpo::leer(&cuerpo)))
}

/// C6: con `[30, 52]` guardado y 75 de real, `estimar --error` da 45 min,
/// 0,60 y 0,40 en implementacion, y lo mismo por fase y en total; sin
/// estimacion guardada, «sin estimacion guardada» y ninguna cifra.
#[test]
fn c6_error_por_fase_y_total() {
    let f = Fixture::nueva("estimacion-c6");
    let (bare, a) = preparar(&f);
    poner(
        &f,
        &a,
        &[tarea_c6("TASK-001", true), tarea_c6("TASK-002", false)],
    );
    let punta = f.crudo(&bare, &["rev-parse", RAMA_ESTADO]);
    let (c, out, err) = correr(&f, &a, &["estimar", "--error", "TASK-001"]);
    assert_eq!(c, Some(0), "{out} {err}");
    let primera = out.lines().next().unwrap();
    for (k, v) in [
        ("error-implementacion", "45"),
        ("relativo-implementacion", "0.60"),
        ("razon-implementacion", "0.40"),
        ("error-rondas", "0"),
        ("relativo-rondas", "0.00"),
        ("razon-rondas", "1.00"),
        ("error-cierre", "0"),
        ("error-total", "45"),
        ("relativo-total", "0.39"),
        ("razon-total", "0.61"),
    ] {
        assert_eq!(valor(primera, k), Some(v), "{k}: {out}");
    }
    assert_eq!(
        out.lines().nth(1),
        Some(
            "error: implementacion 45 min (relativo 0,60; estimado/real 0,40); rondas 0 min (relativo 0,00; estimado/real 1,00); cierre 0 min (relativo 0,00; estimado/real 1,00); total 45 min (relativo 0,39; estimado/real 0,61)"
        )
    );
    // Sin estimacion guardada.
    let (c, out, err) = correr(&f, &a, &["estimar", "--error", "TASK-002"]);
    assert_eq!(c, Some(3), "{out} {err}");
    assert_eq!(out, "estimar rechazada id=TASK-002 motivo=sin-estimacion\n");
    assert!(
        err.starts_with("sin estimacion guardada en TASK-002"),
        "{err}"
    );
    assert!(
        !err.chars()
            .any(|c| c.is_ascii_digit() && !"0002".contains(c)),
        "{err}"
    );
    // Solo lectura: la rama de estado no se movio.
    assert_eq!(f.crudo(&bare, &["rev-parse", RAMA_ESTADO]), punta);
    // Uso.
    let (c, out, _) = correr(&f, &a, &["estimar", "--error"]);
    assert_eq!((c, out.as_str()), (Some(2), "estimar uso motivo=sin-id\n"));
    let (c, out, _) = correr(&f, &a, &["estimar", "TASK-1"]);
    assert_eq!(
        (c, out.as_str()),
        (Some(2), "estimar uso motivo=id-invalido\n")
    );
}

/// C7: `tc2 estimar TASK-NNN` sale de funciones puras, del estado y de un
/// solo `git log` (ni `fetch` ni red ni modelos); la tarea estimada nunca es
/// muestra de si misma (una terminada se estima con las otras cinco).
#[test]
fn c7_un_git_log_y_nunca_ella_misma() {
    let f = Fixture::nueva("estimacion-c7");
    let (_, a) = preparar(&f);
    let seis: Vec<(String, String)> = (1..=6)
        .map(|i| terminada(&format!("TASK-{i:03}"), i, 5 * i + 5))
        .collect();
    poner(&f, &a, &seis);
    let g = tc2::git::Git::nuevo().con_entorno(f.entorno_ssh());
    let e = estimacion::Entorno {
        dir: &a,
        git: &g,
        catalogo: &sin_catalogo,
        ahora: tc2::identidad::ahora(),
    };
    let (i, linea) = estimacion::orden(&["TASK-003".to_owned()], &e);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("estimacion-n"), Some("5"), "{}", i.linea());
    assert_eq!(
        i.valor("estimacion-nivel"),
        Some("carril+complejidad+tamano")
    );
    assert!(linea.unwrap().starts_with("estimacion: ≈"));
    let registro = g.registro();
    comun::registro_permitido(&registro);
    let logs = registro
        .iter()
        .filter(|l| l.starts_with("git log "))
        .count();
    assert_eq!(logs, 1, "{registro:?}");
    // Nada que hable con el remoto (`ls-remote --get-url` solo lee la
    // configuracion) ni que escriba.
    for l in &registro {
        for sub in ["fetch", "push", "commit-tree", "update-ref"] {
            assert!(!l.starts_with(&format!("git {sub} ")), "{l}");
        }
        assert!(
            !l.starts_with("git ls-remote ") || l.starts_with("git ls-remote --get-url"),
            "{l}"
        );
    }
}

/// R1-01: `git::historia` con commits reales. Un commit `TASK-001` en el
/// `develop` del remoto a las 11:40 parte la pausa de C1: el real de
/// implementacion pasa de 75 a 85 y el error frente a los 30 guardados, de
/// 45 a 55. Con otro formato de `git log` no se leeria y seguiria en 45.
#[test]
fn r1_01_un_commit_de_develop_cambia_el_real() {
    let f = Fixture::nueva("estimacion-r1-01");
    let (bare, a) = preparar(&f);
    poner(&f, &a, &[tarea_c6("TASK-001", true)]);
    let error = |f: &Fixture| {
        let (c, out, err) = correr(f, &a, &["estimar", "--error", "TASK-001"]);
        assert_eq!(c, Some(0), "{out} {err}");
        assert_eq!(valor(&out, "historia"), None, "{out}");
        valor(&out, "error-implementacion").map(str::to_owned)
    };
    assert_eq!(error(&f).as_deref(), Some("45"));
    // El commit, en el `develop` del remoto desde otro clon.
    let p = f.clon_github(&bare, "p");
    let fecha = "2026-11-17T11:40:00Z";
    f.git_ssh_con(
        &p,
        &[
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "feat(TASK-001): a medias",
        ],
        &[("GIT_COMMITTER_DATE", fecha), ("GIT_AUTHOR_DATE", fecha)],
    );
    f.git_ssh(&p, &["push", "-q", "origin", "develop"]);
    // Sin `fetch`, `estimar` no lo ve; tras el `fetch`, si.
    assert_eq!(error(&f).as_deref(), Some("45"));
    f.git_ssh(&a, &["fetch", "-q", "origin"]);
    assert_eq!(error(&f).as_deref(), Some("55"));
}

/// Las llamadas `git <sub>` de `tc2 <args>` (por `GIT_TRACE`), con el
/// subcomando y sus argumentos.
fn llamadas(f: &Fixture, dir: &Path, args: &[&str]) -> (Option<i32>, String, Vec<String>) {
    let traza = f.raiz.join(format!("traza-{}.txt", args[0]));
    let _ = std::fs::remove_file(&traza);
    let o = comun::orden_tc2()
        .current_dir(dir)
        .args(args)
        .envs(f.entorno_ssh())
        .env("GIT_TRACE", &traza)
        .output()
        .unwrap();
    let t = std::fs::read_to_string(&traza).unwrap_or_default();
    let subs = t
        .lines()
        .filter_map(|l| l.split_once("trace: built-in: git ").map(|(_, r)| r))
        .map(|r| {
            let mut p: Vec<&str> = r.split_whitespace().collect();
            while p.first() == Some(&"-c") {
                p.drain(..2.min(p.len()));
            }
            p.join(" ")
        })
        .collect();
    (o.status.code(), texto(&o.stdout), subs)
}

fn cuantas(subs: &[String], prefijo: &str) -> usize {
    subs.iter().filter(|s| s.starts_with(prefijo)).count()
}

/// R1-05: `new`, `import` (un lote de tres) y `start` leen la historia con
/// un solo `git log` (C7 en las cuatro ordenes). R1-06: `start` lee todos
/// los `tarea.md` (para el historico) una sola vez: en la transaccion, no
/// en el sondeo.
#[test]
fn r1_05_un_git_log_en_new_import_y_start() {
    let f = Fixture::nueva("estimacion-r1-05");
    let (_, a) = preparar(&f);
    let seis: Vec<(String, String)> = (1..=6)
        .map(|i| terminada(&format!("TASK-{i:03}"), i, 5 * i + 5))
        .collect();
    poner(&f, &a, &seis);
    let (c, out, subs) = llamadas(&f, &a, &["new", "Nueva", "--criterio", "uno"]);
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "estimacion-n"), Some("6"), "{out}");
    assert_eq!(cuantas(&subs, "log "), 1, "new: {subs:?}");
    let lote = "---\nlote: r105\nsprint: 1\n---\n# Plan\n\n### A Uno\n> Objetivo uno\n- Criterio a\n\n### B Dos\n> Objetivo dos\n- Criterio b\n\n### C Tres\n> Objetivo tres\n- Criterio c\n";
    let ruta = f.raiz.join("r105.md");
    std::fs::write(&ruta, lote).unwrap();
    let (c, out, subs) = llamadas(&f, &a, &["import", ruta.to_str().unwrap()]);
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "creadas"), Some("3"), "{out}");
    assert_eq!(cuantas(&subs, "log "), 1, "import: {subs:?}");
    let (c, out, subs) = llamadas(&f, &a, &["start", "TASK-007"]);
    assert_eq!(c, Some(0), "{out}");
    assert_eq!(valor(&out, "estimacion-n"), Some("6"), "{out}");
    assert_eq!(cuantas(&subs, "log "), 1, "start: {subs:?}");
    // Dos `cat-file --batch` con todos los `tarea.md`, los dos en la
    // transaccion: el del historico y el del `ROADMAP.md`. El sondeo solo
    // lee la tarea (`show`); si calculara el historico, serian tres.
    assert_eq!(cuantas(&subs, "cat-file --batch"), 2, "start: {subs:?}");
}

/// `doctor` cuenta el historico: `estado historico ok usables=N
/// descartadas=M`.
#[test]
fn doctor_cuenta_el_historico() {
    let f = Fixture::nueva("estimacion-doctor");
    let (_, a) = preparar(&f);
    let mut fs: Vec<(String, String)> = (1..=3)
        .map(|i| terminada(&format!("TASK-{i:03}"), i, 10))
        .collect();
    // Una con la review antes del start: duracion negativa.
    let (r, md) = terminada("TASK-004", 4, 10);
    let mal = md
        .replacen("| start |", "| review-x |", 1)
        .replacen("| review |", "| start |", 1);
    let mal = mal.replacen("| review-x |", "| review |", 1);
    fs.push((r, mal));
    poner(&f, &a, &fs);
    let g = tc2::git::Git::nuevo().con_entorno(f.entorno_ssh());
    let sin =
        |_: &tc2::plataforma::Personas| -> Result<tc2::tablero::Conectada, tc2::tablero::Consulta> {
            Err(tc2::tablero::Consulta::NoConfigurada)
        };
    let (hs, _) = tc2::doctor::comprobar(&tc2::doctor::Entorno {
        dir: &a,
        git: &g,
        conectar: &sin,
        claude: None,
    })
    .unwrap_or_else(|i| panic!("{}", i.linea()));
    let h: Vec<String> = hs
        .iter()
        .filter(|h| h.area == "estado" && h.id == "historico")
        .map(tc2::doctor::Hallazgo::linea)
        .collect();
    assert_eq!(h, ["estado historico ok usables=3 descartadas=1"]);
}
