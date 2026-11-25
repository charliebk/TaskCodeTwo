//! C3 de F2-3: montaje de la rama de estado, caminos del remoto (hook que
//! deniega, contencion hasta `agotada`) y sin red con pendientes. Responde
//! las preguntas abiertas de F1-2 asignadas a F2-3.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::cell::Cell;
use std::path::PathBuf;

use comun::ops::{Anotar, CrearUnico, escribir_estado};
use comun::{Fixture, contrato, escribir, registro_permitido};
use tc2::contrato::Resultado;
use tc2::estado::{self, Cambio, Estado, Fallo, Operacion};
use tc2::git::Git;

#[test]
fn montar_crea_monta_repite_y_vuelve_a_montar() {
    let f = Fixture::nueva("estado-montar");
    let (bare, clon) = f.remoto_y_clon();
    let g = f.git();

    let i = estado::montar(&g, &clon);
    contrato(&i);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert!(clon.join("tareas/LEEME.md").exists());
    let publicada = f.rama_en(&bare);
    assert!(!publicada.is_empty(), "la rama de estado esta en el remoto");
    assert_eq!(
        f.crudo(&bare, &["rev-list", "--count", "taskcodetwo/tareas"])
            .trim(),
        "1"
    );
    // Huerfana: sin historia comun con develop.
    let mb = std::process::Command::new("git")
        .current_dir(&bare)
        .args(["merge-base", "develop", "taskcodetwo/tareas"])
        .output()
        .unwrap();
    assert!(!mb.status.success());
    assert_eq!(
        f.crudo(&clon, &["status", "--porcelain"]),
        "",
        "tareas/ excluida"
    );

    // Idempotente.
    let i = estado::montar(&g, &clon);
    contrato(&i);
    assert_eq!(
        i.linea(),
        "montar-estado sin-cambios rama=taskcodetwo/tareas ruta=tareas"
    );

    // Otro clon la encuentra en el remoto y solo la monta.
    let otro = f.otro_clon(&bare, "otro");
    let i = estado::montar(&g, &otro);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(f.rama_en(&bare), publicada, "no escribio nada");
    assert!(otro.join("tareas/LEEME.md").exists());

    // Carpeta borrada (como con `git clean -ffdx`): se vuelve a montar.
    std::fs::remove_dir_all(clon.join("tareas")).unwrap();
    let i = estado::montar(&g, &clon);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert!(clon.join("tareas/LEEME.md").exists());

    // Una escritura con la carpeta borrada sigue siendo correcta: adelanta
    // la rama local y lo dice.
    std::fs::remove_dir_all(otro.join("tareas")).unwrap();
    let i = escribir_estado(&g, &otro, &Anotar::nueva("notas/sin-montar.md", true));
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("tareas"), Some("sin-montar"));
    assert_eq!(f.rama_en(&otro), f.rama_en(&bare));
    let i = estado::montar(&g, &otro);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert!(otro.join("tareas/notas/sin-montar.md").exists());

    // `tareas/` ocupada por otra cosa (un repo de la 0.16).
    let ocupado = f.otro_clon(&bare, "ocupado");
    escribir(&ocupado, "tareas/00-planificadas/TASK-001/tarea.md", "x\n");
    let i = estado::montar(&g, &ocupado);
    contrato(&i);
    assert_eq!(
        i.linea(),
        "montar-estado rechazada ruta=tareas motivo=tareas-ocupada"
    );
    assert_eq!(i.salida(false).codigo, 3);

    registro_permitido(&g.registro());
}

#[test]
fn repo_sin_remoto_el_commit_local_es_la_transaccion() {
    let f = Fixture::nueva("estado-solo");
    let solo = f.repo_sin_remoto("solo");
    let g = f.git();

    let i = estado::montar(&g, &solo);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    let i = escribir_estado(&g, &solo, &Anotar::nueva("notas/a.md", true));
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("tareas"), Some("al-dia"));
    assert!(solo.join("tareas/notas/a.md").exists());
    // Repetirla no escribe.
    let i = escribir_estado(&g, &solo, &Anotar::nueva("notas/a.md", true));
    assert_eq!(
        i.linea(),
        "estado sin-cambios operacion=anotar",
        "{}",
        i.linea()
    );
    // Rechazo de la operacion: nada escrito.
    let antes = f.rama_en(&solo);
    let i = escribir_estado(
        &g,
        &solo,
        &CrearUnico {
            ruta: "notas/a.md".into(),
        },
    );
    assert_eq!(
        i.linea(),
        "estado rechazada operacion=crear-unico motivo=ya-existe"
    );
    assert_eq!(f.rama_en(&solo), antes);

    let v = estado::leer(&g, &solo).unwrap();
    assert!(v.rutas().contains(&"notas/a.md".to_owned()));
    assert_eq!(
        v.leer("notas/a.md").unwrap().as_deref(),
        Some("notas/a.md\n")
    );
    registro_permitido(&g.registro());
}

#[test]
fn el_remoto_que_deniega_no_se_reintenta_y_la_contencion_acaba_en_agotada() {
    let f = Fixture::nueva("estado-remoto");
    let (bare, clon) = f.remoto_y_clon();
    let g = f.git();
    assert!(estado::montar(&g, &clon).resultado.es_exito());
    let antes = f.rama_en(&bare);

    // Hook o regla del remoto: `denegada` (8), un solo push.
    let hook = bare.join("hooks").join("pre-receive");
    std::fs::write(&hook, "#!/bin/sh\necho rama protegida >&2\nexit 1\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let pushes = |g: &Git| {
        g.registro()
            .iter()
            .filter(|l| l.starts_with("git push"))
            .count()
    };
    let previos = pushes(&g);
    let i = escribir_estado(&g, &clon, &Anotar::nueva("notas/x.md", true));
    assert_eq!(
        i.linea(),
        "estado denegada operacion=anotar motivo=remoto-rechaza"
    );
    assert_eq!(i.salida(false).codigo, 8);
    assert_eq!(pushes(&g) - previos, 1, "sin reintento");
    assert_eq!(f.rama_en(&bare), antes);
    assert_eq!(f.crudo(&clon, &["for-each-ref", "refs/taskcodetwo/"]), "");
    std::fs::remove_file(&hook).unwrap();

    // Contencion sostenida: en cada intento otro clon publica antes.
    let otro = f.otro_clon(&bare, "otro");
    assert!(estado::montar(&g, &otro).resultado.es_exito());
    let op = SiempreLlegaOtro {
        f: &f,
        otro: otro.join("tareas"),
        n: Cell::new(0),
    };
    let local_antes = f.rama_en(&clon);
    let i = escribir_estado(&g, &clon, &op);
    assert_eq!(i.linea(), "estado agotada operacion=pisada intentos=3");
    assert_eq!(i.salida(false).codigo, 5);
    assert_eq!(op.n.get(), 3, "re-ejecutada en cada intento");
    // Nada escrito: ni en el remoto (solo lo del otro), ni en local.
    let notas = f.crudo(
        &bare,
        &["ls-tree", "-r", "--name-only", "taskcodetwo/tareas"],
    );
    assert!(!notas.contains("notas/pisada.md"), "{notas}");
    assert_eq!(f.rama_en(&clon), local_antes);
    assert!(!clon.join("tareas/notas/pisada.md").exists());
    assert_eq!(f.crudo(&clon, &["for-each-ref", "refs/taskcodetwo/"]), "");
    registro_permitido(&g.registro());
}

/// Operacion que, cada vez que se ejecuta, hace que otro clon publique en
/// la rama de estado antes que ella.
struct SiempreLlegaOtro<'a> {
    f: &'a Fixture,
    otro: PathBuf,
    n: Cell<usize>,
}

impl Operacion for SiempreLlegaOtro<'_> {
    fn nombre(&self) -> &str {
        "pisada"
    }
    fn anotacion(&self) -> String {
        "pisada".into()
    }
    fn exige_red(&self) -> bool {
        true
    }
    fn aplicar(&self, _: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        let n = self.n.get() + 1;
        self.n.set(n);
        let ruta = format!("notas/otro-{n}.md");
        escribir(&self.otro, &ruta, "otro\n");
        self.f.crudo(&self.otro, &["add", &ruta]);
        self.f.crudo(&self.otro, &["commit", "-q", "-m", "otro"]);
        self.f
            .crudo(&self.otro, &["push", "-q", "origin", "taskcodetwo/tareas"]);
        Ok(vec![Cambio::nuevo("notas/pisada.md", "mio\n")])
    }
}

#[test]
fn sin_red_quedan_pendientes_y_se_re_ejecutan_al_volver() {
    let f = Fixture::nueva("estado-sinred");
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

    // Operacion de la tarea propia: queda pendiente.
    let i = escribir_estado(&g, &clon, &Anotar::nueva("notas/a.md", false));
    assert_eq!(
        i.linea(),
        "estado sinred operacion=anotar pendiente=si pendientes=1"
    );
    assert_eq!(i.salida(false).codigo, 4);
    // Operacion que reserva algo compartido: falla sin escribir nada.
    let i = escribir_estado(&g, &clon, &Anotar::nueva("notas/b.md", true));
    assert_eq!(i.linea(), "estado sinred operacion=anotar pendiente=no");
    let i = escribir_estado(
        &g,
        &clon,
        &CrearUnico {
            ruta: "notas/c.md".into(),
        },
    );
    assert_eq!(i.valor("pendientes"), Some("2"), "{}", i.linea());
    let refs = f.crudo(
        &clon,
        &["for-each-ref", "--format=%(refname)", "refs/taskcodetwo/"],
    );
    assert_eq!(
        refs,
        "refs/taskcodetwo/pendientes/000001\nrefs/taskcodetwo/pendientes/000002\n"
    );
    // Mientras, otra persona crea `notas/c.md`.
    let i = escribir_estado(
        &g,
        &otro,
        &CrearUnico {
            ruta: "notas/c.md".into(),
        },
    );
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());

    // Vuelve la red: la siguiente operacion re-ejecuta las pendientes sobre
    // el estado nuevo; la de `c` ya no procede y queda guardada aparte.
    f.crudo(&clon, &["remote", "set-url", "origin", &url]);
    let i = escribir_estado(&g, &clon, &Anotar::nueva("notas/d.md", true));
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("pendientes"), Some("1"), "{}", i.linea());
    assert_eq!(i.valor("rechazadas"), Some("1"), "{}", i.linea());
    let notas = f.crudo(
        &bare,
        &["ls-tree", "-r", "--name-only", "taskcodetwo/tareas"],
    );
    for n in ["notas/a.md", "notas/c.md", "notas/d.md"] {
        assert!(notas.contains(n), "{n}: {notas}");
    }
    assert!(!notas.contains("notas/b.md"));
    assert_eq!(
        f.crudo(&bare, &["show", "taskcodetwo/tareas:notas/c.md"]),
        "unico\n"
    );
    let refs = f.crudo(
        &clon,
        &["for-each-ref", "--format=%(refname)", "refs/taskcodetwo/"],
    );
    assert_eq!(refs, "refs/taskcodetwo/rechazadas/000002\n");
    // Lo rechazado no se pierde: sigue en un objeto de git.
    let msg = f.crudo(
        &clon,
        &[
            "log",
            "-1",
            "--format=%B",
            "refs/taskcodetwo/rechazadas/000002",
        ],
    );
    assert!(msg.contains("Operacion: crear-unico|notas/c.md"), "{msg}");
    // La lectura ve lo publicado.
    let v = estado::leer(&g, &clon).unwrap();
    assert!(
        v.rutas().contains(&"notas/d.md".to_owned()),
        "{:?}",
        v.rutas()
    );
    registro_permitido(&g.registro());
}

/// TASK-076 (paso 3 del plan): `leer_varios` da lo mismo que `leer` una a
/// una, pero con un solo `git cat-file --batch` y ningun `git show`.
#[test]
fn leer_varios_en_un_solo_proceso_y_mismo_resultado() {
    let f = Fixture::nueva("estado-leer-varios");
    let (_bare, clon) = f.remoto_y_clon();
    let g = f.git();
    assert_eq!(estado::montar(&g, &clon).resultado, Resultado::Ok);
    for r in ["notas/a b.md", "notas/ñandú.md", "c.md"] {
        let i = escribir_estado(&g, &clon, &Anotar::nueva(r, true));
        assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    }
    let rutas: Vec<String> = [
        "notas/a b.md",
        "no-existe.md",
        "notas/ñandú.md",
        "notas",
        "c.md",
        "notas/a b.md",
    ]
    .iter()
    .map(|r| (*r).to_owned())
    .collect();

    let una_a_una = f.git();
    let v = estado::leer(&una_a_una, &clon).unwrap();
    let esperado: Vec<Option<String>> = rutas.iter().map(|r| v.leer(r).unwrap()).collect();
    assert_eq!(esperado[0].as_deref(), Some("notas/a b.md\n"));
    assert_eq!(esperado[1], None);
    assert_eq!(esperado[3], None, "una carpeta no es un fichero");

    let en_lote = f.git();
    let v = estado::leer(&en_lote, &clon).unwrap();
    let antes = en_lote.registro().len();
    assert_eq!(v.leer_varios(&rutas).unwrap(), esperado);
    let llamadas: Vec<String> = en_lote.registro().split_off(antes);
    assert_eq!(llamadas, ["git cat-file --batch"], "{llamadas:?}");
    // Sin nada del commit que leer, no se lanza nada.
    let antes = en_lote.registro().len();
    assert_eq!(v.leer_varios(&["no-existe.md".to_owned()]).unwrap(), [None]);
    assert_eq!(en_lote.registro().len(), antes);
    registro_permitido(&en_lote.registro());
}

/// Escribe `n` ficheros nuevos `escala/<prefijo>-<i>.md` en una operacion.
struct Muchos {
    prefijo: &'static str,
    n: usize,
}

impl Operacion for Muchos {
    fn nombre(&self) -> &str {
        "muchos"
    }
    fn anotacion(&self) -> String {
        format!("muchos {}", self.n)
    }
    fn exige_red(&self) -> bool {
        true
    }
    fn aplicar(&self, _: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        Ok((0..self.n)
            .map(|i| {
                Cambio::nuevo(
                    format!("escala/{}-{i:04}.md", self.prefijo),
                    format!("fichero {i} de {}\n", self.prefijo),
                )
            })
            .collect())
    }
}

/// TASK-127: una transaccion con 1000 ficheros termina en `ok` (en Windows
/// fallaba `hash-object` por la longitud de la linea de ordenes) y lanza los
/// mismos procesos de git que una con 10: rutas y entradas van por stdin.
#[test]
fn mil_ficheros_en_una_transaccion_con_procesos_acotados() {
    let f = Fixture::nueva("estado-mil");
    let (_bare, clon) = f.remoto_y_clon();
    let i = estado::montar(&f.git(), &clon);
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());

    let procesos = |prefijo: &'static str, n: usize| {
        let g = f.git();
        let i = escribir_estado(&g, &clon, &Muchos { prefijo, n });
        assert_eq!(i.resultado, Resultado::Ok, "{n}: {}", i.linea());
        g.registro()
    };
    let diez = procesos("diez", 10);
    let mil = procesos("mil", 1000);

    let rama = f.rama_en(&clon);
    let rama = rama.trim();
    let listado = f.crudo(
        &clon,
        &["ls-tree", "--name-only", &format!("{rama}:escala")],
    );
    let mil_en_rama = listado.lines().filter(|l| l.starts_with("mil-")).count();
    assert_eq!(mil_en_rama, 1000, "los 1000 ficheros estan en la rama");
    assert_eq!(
        f.crudo(&clon, &["show", &format!("{rama}:escala/mil-0999.md")]),
        "fichero 999 de mil\n"
    );

    assert_eq!(
        diez.len(),
        mil.len(),
        "los procesos no crecen con los ficheros:\n{diez:#?}\n{mil:#?}"
    );
    for l in mil
        .iter()
        .filter(|l| l.starts_with("git hash-object") || l.starts_with("git update-index"))
    {
        assert!(
            !l.contains("blob-") && !l.contains("escala/"),
            "sin rutas en la linea de ordenes: {l}"
        );
    }
    registro_permitido(&mil);
}

/// Escribe `ruta` con un commit firmado (como `approve`, F4-2) y el trailer
/// `Plan-Blob` de esa misma ruta.
struct Firmada {
    ruta: String,
}

impl Operacion for Firmada {
    fn nombre(&self) -> &str {
        "firmada"
    }
    fn anotacion(&self) -> String {
        format!("firmada|{}", self.ruta)
    }
    fn exige_red(&self) -> bool {
        true
    }
    fn aplicar(&self, _: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        Ok(vec![Cambio::nuevo(self.ruta.clone(), "firmado\n")])
    }
    fn sello(&self) -> estado::Sello {
        estado::Sello::Firmado {
            trailers: vec![("Plan-Blob", self.ruta.clone())],
        }
    }
}

/// MEN-1 de TASK-037 (D4a): con una pendiente encolada, el commit firmado
/// lleva solo lo de la operacion firmada; la pendiente sigue encolada y la
/// publica la siguiente operacion normal, sin firmar.
#[test]
fn las_pendientes_no_entran_en_el_commit_firmado() {
    let f = Fixture::nueva("estado-firmado");
    let (bare, clon) = f.remoto_y_clon();
    let clave = f.raiz.join("clave");
    let k = std::process::Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-C", "prueba", "-f"])
        .arg(&clave)
        .output()
        .unwrap();
    assert!(k.status.success(), "{}", String::from_utf8_lossy(&k.stderr));
    let mut entorno = f.entorno();
    for (k, v) in [
        ("GIT_CONFIG_COUNT", "2".to_owned()),
        ("GIT_CONFIG_KEY_0", "gpg.format".to_owned()),
        ("GIT_CONFIG_VALUE_0", "ssh".to_owned()),
        ("GIT_CONFIG_KEY_1", "user.signingkey".to_owned()),
        (
            "GIT_CONFIG_VALUE_1",
            clave.to_string_lossy().replace('\\', "/"),
        ),
    ] {
        entorno.push((k.into(), v.into()));
    }
    let g = Git::nuevo().con_entorno(entorno);
    assert!(estado::montar(&g, &clon).resultado.es_exito());
    let url = bare.to_str().unwrap().to_owned();
    f.crudo(
        &clon,
        &["remote", "set-url", "origin", "http://127.0.0.1:1/x.git"],
    );
    let i = escribir_estado(&g, &clon, &Anotar::nueva("notas/pendiente.md", false));
    assert_eq!(i.valor("pendiente"), Some("si"), "{}", i.linea());
    f.crudo(&clon, &["remote", "set-url", "origin", &url]);

    let i = escribir_estado(
        &g,
        &clon,
        &Firmada {
            ruta: "notas/firmada.md".into(),
        },
    );
    assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    assert_eq!(i.valor("pendientes"), None, "{}", i.linea());
    let arbol = f.crudo(
        &bare,
        &["ls-tree", "-r", "--name-only", "taskcodetwo/tareas"],
    );
    assert!(arbol.contains("notas/firmada.md"), "{arbol}");
    assert!(!arbol.contains("notas/pendiente.md"), "{arbol}");
    let cuerpo = f.crudo(&bare, &["cat-file", "-p", "taskcodetwo/tareas"]);
    assert!(cuerpo.contains("gpgsig "), "{cuerpo}");
    assert!(!cuerpo.contains("anotar|"), "{cuerpo}");
    assert_eq!(
        f.crudo(
            &clon,
            &["for-each-ref", "--format=%(refname)", "refs/taskcodetwo/"],
        ),
        "refs/taskcodetwo/pendientes/000001\n"
    );

    // La siguiente operacion normal publica la pendiente, sin firmar.
    let i = escribir_estado(&g, &clon, &Anotar::nueva("notas/otra.md", true));
    assert_eq!(i.valor("pendientes"), Some("1"), "{}", i.linea());
    let arbol = f.crudo(
        &bare,
        &["ls-tree", "-r", "--name-only", "taskcodetwo/tareas"],
    );
    assert!(arbol.contains("notas/pendiente.md"), "{arbol}");
    let cuerpo = f.crudo(&bare, &["cat-file", "-p", "taskcodetwo/tareas"]);
    assert!(!cuerpo.contains("gpgsig "), "{cuerpo}");
}
