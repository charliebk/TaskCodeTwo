//! Bordes de la rama de estado que pidio la revision de la ronda 1 de
//! TASK-018: remoto anadido despues, `tareas/` con cambios o divergente,
//! cerrojo ocupado, remoto que deniega el `fetch`, rutas que chocan por
//! mayusculas, `montar` concurrente y pendientes que fallan al volver.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::path::{Path, PathBuf};
use std::time::Duration;

use comun::ops::{Anotar, CrearUnico, FallaSi, catalogo};
use comun::{Fixture, contrato, escribir, registro_permitido};
use tc2::contrato::{Informe, Resultado};
use tc2::estado::{self, Cerrojo, Contexto, Operacion, Politica};
use tc2::git::Git;

fn rapida() -> Politica {
    Politica {
        intentos: 3,
        base: Duration::from_millis(1),
        espera_cerrojo: Duration::from_secs(30),
    }
}

fn escribir_estado(g: &Git, dir: &Path, op: &dyn Operacion) -> Informe {
    let mut c = Contexto::nuevo(g, dir, &catalogo);
    c.politica = rapida();
    let i = estado::transaccion(&c, op);
    contrato(&i);
    i
}

fn rama_en(f: &Fixture, repo: &Path) -> String {
    f.crudo(
        repo,
        &[
            "for-each-ref",
            "--format=%(objectname)",
            "refs/heads/taskcode/tareas",
        ],
    )
}

fn repo_sin_remoto(f: &Fixture, nombre: &str) -> PathBuf {
    let solo = f.raiz.join(nombre);
    f.crudo(&f.raiz, &["init", "-q", "-b", "develop", nombre]);
    escribir(&solo, "README.md", "x\n");
    f.crudo(&solo, &["add", "README.md"]);
    f.crudo(&solo, &["commit", "-q", "-m", "inicial"]);
    solo
}

fn ficheros(f: &Fixture, repo: &Path) -> String {
    f.crudo(repo, &["ls-tree", "-r", "--name-only", "taskcode/tareas"])
}

#[test]
fn remoto_anadido_despues_publica_la_historia_local() {
    let f = Fixture::nueva("bordes-remoto-despues");
    let solo = repo_sin_remoto(&f, "solo");
    let g = f.git();
    assert!(estado::montar(&g, &solo).resultado.es_exito());
    let i = escribir_estado(&g, &solo, &Anotar::nueva("notas/a.md", true));
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());

    // Se anade un remoto vacio.
    f.crudo(
        &f.raiz,
        &["init", "-q", "--bare", "-b", "develop", "remoto.git"],
    );
    let bare = f.raiz.join("remoto.git");
    f.crudo(&solo, &["remote", "add", "origin", bare.to_str().unwrap()]);
    let i = escribir_estado(&g, &solo, &Anotar::nueva("notas/b.md", true));
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("tareas"), Some("al-dia"), "{}", i.linea());
    assert_eq!(ficheros(&f, &bare), "LEEME.md\nnotas/a.md\nnotas/b.md\n");
    assert_eq!(rama_en(&f, &solo), rama_en(&f, &bare));
    assert!(solo.join("tareas/notas/b.md").exists());
}

#[test]
fn tareas_con_cambios_o_divergente_lo_dice_y_no_pierde_nada() {
    let f = Fixture::nueva("bordes-tareas");
    let (bare, clon) = f.remoto_y_clon();
    let g = f.git();
    assert!(estado::montar(&g, &clon).resultado.es_exito());
    let tareas = clon.join("tareas");

    // Fichero sin versionar de la persona que choca con la operacion: se
    // publica, pero `tareas/` no se adelanta y el fichero queda intacto.
    escribir(&tareas, "notas/y.md", "de la persona\n");
    let i = escribir_estado(&g, &clon, &Anotar::nueva("notas/y.md", true));
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("tareas"), Some("con-cambios"));
    assert!(ficheros(&f, &bare).contains("notas/y.md"));
    assert_eq!(
        std::fs::read_to_string(tareas.join("notas/y.md")).unwrap(),
        "de la persona\n"
    );
    std::fs::remove_file(tareas.join("notas/y.md")).unwrap();
    f.crudo(
        &tareas,
        &["merge", "-q", "--ff-only", "origin/taskcode/tareas"],
    );

    // Commit a mano en `tareas/` sin publicar: la rama local diverge de lo
    // publicado; se publica igual y la linea lo dice.
    escribir(&tareas, "manual.md", "a mano\n");
    f.crudo(&tareas, &["add", "manual.md"]);
    f.crudo(&tareas, &["commit", "-q", "-m", "a mano"]);
    let i = escribir_estado(&g, &clon, &Anotar::nueva("notas/z.md", true));
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("tareas"), Some("diverge"));
    assert!(
        tareas.join("manual.md").exists(),
        "el commit de la persona sigue"
    );

    // Sin remoto, el mismo choque no puede publicarse en ningun sitio:
    // `rechazada` y nada escrito.
    let solo = repo_sin_remoto(&f, "solo");
    assert!(estado::montar(&g, &solo).resultado.es_exito());
    escribir(&solo.join("tareas"), "notas/y.md", "de la persona\n");
    let antes = rama_en(&f, &solo);
    let i = escribir_estado(&g, &solo, &Anotar::nueva("notas/y.md", true));
    assert_eq!(
        i.linea(),
        "estado rechazada operacion=anotar motivo=tareas-con-cambios"
    );
    assert_eq!(rama_en(&f, &solo), antes);
    let i = escribir_estado(&g, &solo, &Anotar::nueva("notas/otra.md", true));
    assert_eq!(i.valor("tareas"), Some("al-dia"), "{}", i.linea());
}

#[test]
fn cerrojo_ocupado_y_remoto_que_deniega_el_fetch() {
    let f = Fixture::nueva("bordes-cerrojo");
    let (_bare, clon) = f.remoto_y_clon();
    let g = f.git();
    assert!(estado::montar(&g, &clon).resultado.es_exito());

    // Otro proceso tiene el cerrojo: `agotada motivo=cerrojo` al vencer la
    // espera, sin escribir nada.
    let tomado = Cerrojo::tomar(&clon.join(".git"), Duration::from_secs(1)).unwrap();
    let mut c = Contexto::nuevo(&g, &clon, &catalogo);
    c.politica.espera_cerrojo = Duration::from_millis(300);
    let i = estado::transaccion(&c, &Anotar::nueva("notas/a.md", true));
    contrato(&i);
    assert_eq!(i.linea(), "estado agotada operacion=anotar motivo=cerrojo");
    assert_eq!(i.salida(false).codigo, 5);
    let politica = Politica {
        espera_cerrojo: Duration::from_millis(300),
        ..rapida()
    };
    let i = estado::montar_con(&g, &clon, politica);
    assert_eq!(i.linea(), "montar-estado agotada motivo=cerrojo");
    drop(tomado);

    // `origin` que no es un repositorio: `denegada` (8), tambien al montar.
    f.crudo(
        &clon,
        &[
            "remote",
            "set-url",
            "origin",
            f.raiz.join("no-existe").to_str().unwrap(),
        ],
    );
    let i = escribir_estado(&g, &clon, &Anotar::nueva("notas/a.md", false));
    assert_eq!(
        i.linea(),
        "estado denegada operacion=anotar motivo=remoto-no-existe"
    );
    assert_eq!(f.crudo(&clon, &["for-each-ref", "refs/taskcode/"]), "");
    let nuevo = f.raiz.join("nuevo");
    f.crudo(&f.raiz, &["clone", "-q", clon.to_str().unwrap(), "nuevo"]);
    f.crudo(
        &nuevo,
        &[
            "remote",
            "set-url",
            "origin",
            f.raiz.join("no-existe").to_str().unwrap(),
        ],
    );
    let i = estado::montar(&g, &nuevo);
    contrato(&i);
    assert_eq!(i.linea(), "montar-estado denegada motivo=remoto-no-existe");
    assert!(
        !clon.join(".git/taskcode-estado-tmp").exists(),
        "no quedan temporales"
    );
}

#[test]
fn rutas_que_chocan_por_mayusculas_y_llamadas_mal_formadas() {
    let f = Fixture::nueva("bordes-rutas");
    let solo = repo_sin_remoto(&f, "solo");
    let g = f.git();
    assert!(estado::montar(&g, &solo).resultado.es_exito());
    let i = escribir_estado(&g, &solo, &Anotar::nueva("notas/Mayus.md", true));
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    let i = escribir_estado(&g, &solo, &Anotar::nueva("notas/mayus.md", true));
    assert_eq!(
        i.linea(),
        "estado rechazada operacion=anotar motivo=ruta-duplicada"
    );
    // Una ruta no portable no llega a escribirse.
    let antes = rama_en(&f, &solo);
    let i = escribir_estado(&g, &solo, &Anotar::nueva("notas/aux.md", true));
    assert_eq!(i.resultado, Resultado::Error, "{}", i.linea());
    assert_eq!(i.valor("motivo"), Some("ruta-invalida"));
    assert_eq!(rama_en(&f, &solo), antes);

    // Anotacion de varias lineas y politica sin intentos: `uso` (2).
    let i = escribir_estado(&g, &solo, &Anotar::nueva("notas/a\nb.md", true));
    assert_eq!(
        i.linea(),
        "estado uso operacion=anotar motivo=anotacion-invalida"
    );
    assert_eq!(i.salida(false).codigo, 2);
    let mut c = Contexto::nuevo(&g, &solo, &catalogo);
    c.politica.intentos = 0;
    let i = estado::transaccion(&c, &Anotar::nueva("notas/a.md", true));
    assert_eq!(i.linea(), "estado uso operacion=anotar motivo=sin-intentos");
}

#[test]
fn montar_a_la_vez_desde_varios_hilos_del_mismo_clon() {
    let f = Fixture::nueva("bordes-montar");
    let (bare, clon) = f.remoto_y_clon();
    // La rama ya existe en el remoto (otro clon la creo).
    let otro = f.otro_clon(&bare, "otro");
    assert!(estado::montar(&f.git(), &otro).resultado.es_exito());

    let lineas: Vec<String> = std::thread::scope(|s| {
        let hilos: Vec<_> = (0..4)
            .map(|_| {
                s.spawn(|| {
                    let g = f.git();
                    estado::montar(&g, &clon).linea()
                })
            })
            .collect();
        hilos.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let oks = lineas
        .iter()
        .filter(|l| l.starts_with("montar-estado ok "))
        .count();
    let sin = lineas
        .iter()
        .filter(|l| l.starts_with("montar-estado sin-cambios "))
        .count();
    assert_eq!((oks, sin), (1, 3), "{lineas:?}");
    assert!(clon.join("tareas/LEEME.md").exists());
}

#[test]
fn pendientes_que_ya_no_proceden_o_fallan_van_a_rechazadas_sin_reutilizar_numero() {
    let f = Fixture::nueva("bordes-pendientes");
    let (bare, clon) = f.remoto_y_clon();
    let g = f.git();
    assert!(estado::montar(&g, &clon).resultado.es_exito());
    let otro = f.otro_clon(&bare, "otro");
    assert!(estado::montar(&g, &otro).resultado.es_exito());
    let url = bare.to_str().unwrap().to_owned();
    let sin_red = || {
        f.crudo(
            &clon,
            &["remote", "set-url", "origin", "http://127.0.0.1:1/x.git"],
        );
    };
    let con_red = || {
        f.crudo(&clon, &["remote", "set-url", "origin", &url]);
    };

    // Una pendiente que al volver la red falla con error interno.
    sin_red();
    let op = FallaSi {
        ruta: "notas/f.md".into(),
        veneno: "notas/veneno.md".into(),
    };
    let i = escribir_estado(&g, &clon, &op);
    assert_eq!(i.valor("pendiente"), Some("si"), "{}", i.linea());
    let i = escribir_estado(&g, &otro, &Anotar::nueva("notas/veneno.md", true));
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    con_red();
    // No bloquea: la operacion nueva se publica y la pendiente queda aparte.
    let i = escribir_estado(&g, &clon, &Anotar::nueva("notas/g.md", true));
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("rechazadas"), Some("1"), "{}", i.linea());
    assert!(!ficheros(&f, &bare).contains("notas/f.md"));

    // La siguiente pendiente no reutiliza el numero de la rechazada.
    sin_red();
    let i = escribir_estado(
        &g,
        &clon,
        &CrearUnico {
            ruta: "notas/h.md".into(),
        },
    );
    assert_eq!(i.valor("pendiente"), Some("si"), "{}", i.linea());
    assert_eq!(
        f.crudo(
            &clon,
            &["for-each-ref", "--format=%(refname)", "refs/taskcode/"]
        ),
        "refs/taskcode/pendientes/000002\nrefs/taskcode/rechazadas/000001\n"
    );
    con_red();
    let i = escribir_estado(&g, &clon, &Anotar::nueva("notas/i.md", true));
    assert_eq!(i.valor("pendientes"), Some("1"), "{}", i.linea());
    assert!(ficheros(&f, &bare).contains("notas/h.md"));
    registro_permitido(&g.registro());
}

#[test]
fn pendiente_que_choca_por_mayusculas_no_bloquea_el_clon() {
    let f = Fixture::nueva("bordes-choque-pendiente");
    let (bare, clon) = f.remoto_y_clon();
    let g = f.git();
    assert!(estado::montar(&g, &clon).resultado.es_exito());
    let otro = f.otro_clon(&bare, "otro");
    assert!(estado::montar(&g, &otro).resultado.es_exito());
    let url = bare.to_str().unwrap().to_owned();

    f.crudo(
        &clon,
        &["remote", "set-url", "origin", "http://127.0.0.1:1/x.git"],
    );
    let i = escribir_estado(&g, &clon, &Anotar::nueva("notas/Mayus.md", false));
    assert_eq!(i.valor("pendiente"), Some("si"), "{}", i.linea());
    let i = escribir_estado(&g, &otro, &Anotar::nueva("notas/mayus.md", true));
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    f.crudo(&clon, &["remote", "set-url", "origin", &url]);

    // La pendiente ya no procede (chocaria con lo publicado): va a
    // rechazadas y las escrituras siguientes del clon se publican.
    let i = escribir_estado(&g, &clon, &Anotar::nueva("notas/g.md", true));
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("rechazadas"), Some("1"), "{}", i.linea());
    let i = escribir_estado(&g, &clon, &Anotar::nueva("notas/h.md", true));
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("rechazadas"), None, "{}", i.linea());
    let publicados = ficheros(&f, &bare);
    assert!(publicados.contains("notas/g.md") && publicados.contains("notas/h.md"));
    assert!(!publicados.contains("notas/Mayus.md"), "{publicados}");
    assert_eq!(
        f.crudo(
            &clon,
            &["for-each-ref", "--format=%(refname)", "refs/taskcode/"]
        ),
        "refs/taskcode/rechazadas/000001\n"
    );
}

#[test]
fn remoto_que_responde_access_denied_es_denegada_por_credenciales() {
    let f = Fixture::nueva("bordes-access-denied");
    let (_bare, clon) = f.remoto_y_clon();
    let g = f.git();
    assert!(estado::montar(&g, &clon).resultado.es_exito());
    // Un servidor ssh que responde y deniega el acceso.
    let guion = f.raiz.join("ssh.sh");
    std::fs::write(
        &guion,
        "#!/bin/sh\necho 'remote: access denied' >&2\nexit 1\n",
    )
    .unwrap();
    let guion = guion.to_str().unwrap().replace('\\', "/");
    f.crudo(
        &clon,
        &["config", "core.sshCommand", &format!("sh '{guion}'")],
    );
    f.crudo(
        &clon,
        &[
            "remote",
            "set-url",
            "origin",
            "ssh://servidor.invalid/x.git",
        ],
    );
    let i = escribir_estado(&g, &clon, &Anotar::nueva("notas/a.md", false));
    assert_eq!(
        i.linea(),
        "estado denegada operacion=anotar motivo=credenciales"
    );
    assert_eq!(i.salida(false).codigo, 8);
    assert_eq!(f.crudo(&clon, &["for-each-ref", "refs/taskcode/"]), "");
}

/// Escribe dos rutas en el mismo lote.
struct Dos(&'static str, &'static str);

impl Operacion for Dos {
    fn nombre(&self) -> &str {
        "dos"
    }
    fn anotacion(&self) -> String {
        format!("dos|{}|{}", self.0, self.1)
    }
    fn exige_red(&self) -> bool {
        true
    }
    fn aplicar(
        &self,
        _: &dyn tc2::estado::Estado,
    ) -> Result<Vec<tc2::estado::Cambio>, tc2::estado::Fallo> {
        Ok(vec![
            tc2::estado::Cambio::nuevo(self.0, "1\n"),
            tc2::estado::Cambio::nuevo(self.1, "2\n"),
        ])
    }
}

#[test]
fn dos_rutas_del_mismo_lote_que_chocan_por_mayusculas() {
    let f = Fixture::nueva("bordes-lote");
    let solo = repo_sin_remoto(&f, "solo");
    let g = f.git();
    assert!(estado::montar(&g, &solo).resultado.es_exito());
    let antes = rama_en(&f, &solo);
    let i = escribir_estado(&g, &solo, &Dos("notas/A.md", "notas/a.md"));
    assert_eq!(
        i.linea(),
        "estado rechazada operacion=dos motivo=ruta-duplicada"
    );
    assert_eq!(rama_en(&f, &solo), antes);
    let i = escribir_estado(&g, &solo, &Dos("notas/A.md", "notas/B.md"));
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
}
