//! Escenario «remoto bare servido por un ssh falso» de los tests de
//! integracion (A8-01, A8-16): personas, identidad y operaciones de la
//! biblioteca lanzadas por una persona.
//!
//! `origin` apunta a `ssh://git@github.com/o/r.git` (GitHub por el host) y
//! el transporte ssh se sustituye con `GIT_SSH_COMMAND`, que sirve el remoto
//! bare de la fixture; sin red, un «ssh» que responde «Connection refused».

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use tc2::contrato::{Informe, Resultado};
use tc2::estado::{self, Contexto, Operacion};
use tc2::git::Git;
use tc2::identidad::{Cache, Cuenta, Quien};
use tc2::plataforma::Proveedor;
use tc2::toma::Entorno;

use super::git::{git_en, git_salida};
use super::ops::{Escribir, Poner, sin_catalogo};
use super::{Fixture, escribir, lanzar, volcado};

/// `origin` de los clones: GitHub por el host, servido por el «ssh» falso.
pub const ORIGIN: &str = "ssh://git@github.com/o/r.git";
/// `personas.yml` con ana (1001) y beto (1002).
pub const PERSONAS: &str =
    "---\npersonas:\n  - {id: 1001, login: \"ana\"}\n  - {id: 1002, login: \"beto\"}\n";
/// `config.yml` de la base con el modo que `plan` congela.
pub const CONFIG_AUTOMATICO: &str = "---\nformato: 2\nmodo_flujo: automatico\n";

impl Fixture {
    /// Entorno de git con el «ssh» que sirve el remoto bare de la fixture
    /// (`upload-pack` para leer, `receive-pack` para publicar).
    pub fn entorno_ssh(&self) -> Vec<(OsString, OsString)> {
        let bare = self
            .raiz
            .join("remoto.git")
            .to_string_lossy()
            .replace('\\', "/");
        let mut e = self.entorno();
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

    /// Entorno de git cuyo «ssh» no llega al host, como sin red.
    pub fn entorno_sin_red(&self) -> Vec<(OsString, OsString)> {
        let mut e = self.entorno();
        e.push((
            "GIT_SSH_COMMAND".into(),
            "sh -c 'echo \"ssh: connect to host github.com port 22: Connection refused\" >&2; exit 255'"
                .into(),
        ));
        e.push(("GIT_SSH_VARIANT".into(), "simple".into()));
        e
    }

    /// `git <args>` en `dir` con el remoto de la fixture.
    pub fn git_ssh(&self, dir: &Path, args: &[&str]) -> String {
        self.git_ssh_con(dir, args, &[])
    }

    /// Como [`Fixture::git_ssh`], con variables de mas (fechas de un commit).
    pub fn git_ssh_con(&self, dir: &Path, args: &[&str], vars: &[(&str, &str)]) -> String {
        let mut e = self.entorno_ssh();
        e.extend(vars.iter().map(|(k, v)| ((*k).into(), (*v).into())));
        git_en(dir, &e, args)
    }

    /// Remoto con `personas.yml` (y `config.yml`, si se da) en `develop` y
    /// las tareas de `altas` (cada una, los argumentos de `tc2 new`) creadas
    /// desde un clon de preparacion; devuelve el bare y el clon `a`, con
    /// `origin` en GitHub y la identidad de ana.
    pub fn preparar(&self, p: &Preparar<'_>) -> (PathBuf, PathBuf) {
        let (bare, prep) = self.remoto_y_clon();
        escribir(&prep, ".taskcode/personas.yml", p.personas);
        if let Some(c) = p.config {
            escribir(&prep, ".taskcode/config.yml", c);
        }
        self.crudo(&prep, &["add", ".taskcode"]);
        self.crudo(&prep, &["commit", "-q", "-m", "personas y config"]);
        self.crudo(&prep, &["push", "-q", "origin", "develop"]);
        for alta in p.altas {
            let mut args = vec!["new"];
            args.extend_from_slice(alta);
            let o = lanzar(&prep, self.entorno(), &args);
            assert_eq!(o.status.code(), Some(0), "{}", volcado(&o));
        }
        let a = self.clon_github(&bare, "a");
        identificar(&a, 1001, "ana");
        (bare, a)
    }

    /// `op` por la transaccion desde `dir`, con el remoto de la fixture;
    /// tiene que salir `ok`.
    pub fn transaccion(&self, dir: &Path, op: &dyn Operacion) {
        let g = Git::nuevo().con_entorno(self.entorno_ssh());
        let i = estado::transaccion(&Contexto::nuevo(&g, dir, &sin_catalogo, None), op);
        assert_eq!(i.resultado, Resultado::Ok, "{}", i.linea());
    }

    /// Cambia `ruta` en la rama de estado por la transaccion (como otra
    /// fase): `cambiar` recibe su texto (si existe) y da el nuevo.
    pub fn cambiar_estado(
        &self,
        dir: &Path,
        ruta: &str,
        cambiar: impl Fn(Option<String>) -> String + 'static,
    ) {
        let op = Escribir {
            ruta: ruta.to_owned(),
            cambiar: Box::new(cambiar),
        };
        self.transaccion(dir, &op);
    }

    /// Escribe `ficheros` (ruta y texto) en la rama de estado por la
    /// transaccion desde `dir`.
    pub fn poner(&self, dir: &Path, ficheros: &[(String, String)]) {
        self.transaccion(dir, &Poner(ficheros.to_vec()));
    }

    /// Publica en `bare` la rama de estado con `ficheros` (ruta y texto)
    /// desde el repo aparte `estado` de la raiz, sin pasar por `tc2`;
    /// devuelve el commit publicado.
    pub fn publicar_estado(
        &self,
        bare: &Path,
        ficheros: &[(impl AsRef<str>, impl AsRef<str>)],
    ) -> String {
        let dir = self.raiz.join("estado");
        self.crudo(&self.raiz, &["init", "-q", "-b", estado::RAMA, "estado"]);
        for (ruta, texto) in ficheros {
            escribir(&dir, ruta.as_ref(), texto.as_ref());
        }
        self.crudo(&dir, &["add", "-A"]);
        self.crudo(&dir, &["commit", "-q", "-m", "estado"]);
        self.crudo(&dir, &["push", "-q", bare.to_str().unwrap(), estado::RAMA]);
        self.crudo(&dir, &["rev-parse", "HEAD"]).trim().to_owned()
    }

    /// Punta de la rama de estado en `bare`.
    pub fn punta(&self, bare: &Path) -> String {
        self.crudo(bare, &["rev-parse", estado::RAMA])
            .trim()
            .to_owned()
    }

    /// `ruta` publicada en la rama de estado de `bare`.
    pub fn publicado(&self, bare: &Path, ruta: &str) -> String {
        String::from_utf8_lossy(&self.publicado_bytes(bare, ruta)).into_owned()
    }

    /// Como [`Fixture::publicado`], con los bytes tal cual.
    pub fn publicado_bytes(&self, bare: &Path, ruta: &str) -> Vec<u8> {
        let show = format!("{}:{ruta}", estado::RAMA);
        git_salida(bare, &self.entorno(), &["show", &show]).stdout
    }

    /// Otro clon del remoto con `origin` en GitHub.
    pub fn clon_github(&self, bare: &Path, nombre: &str) -> PathBuf {
        let d = self.otro_clon(bare, nombre);
        self.crudo(&d, &["remote", "set-url", "origin", ORIGIN]);
        d
    }
}

/// Escenario de [`Fixture::preparar`]: `personas.yml`, `config.yml` y las
/// altas. Por defecto, ana y beto, sin configuracion y sin tareas.
pub struct Preparar<'a> {
    pub personas: &'a str,
    pub config: Option<&'a str>,
    pub altas: &'a [&'a [&'a str]],
}

impl Default for Preparar<'_> {
    fn default() -> Self {
        Self {
            personas: PERSONAS,
            config: None,
            altas: &[],
        }
    }
}

/// Cambia `de` por `a` (tiene que estar) en el `tarea.md` publicado de `id`.
pub fn editar(f: &Fixture, dir: &Path, id: &str, de: &str, a: &str) {
    let (de, a) = (de.to_owned(), a.to_owned());
    f.cambiar_estado(dir, &format!("{id}/tarea.md"), move |t| {
        let t = t.unwrap_or_default();
        assert!(t.contains(&de), "{t}");
        t.replacen(&de, &a, 1)
    });
}

/// Lo que `tarea.md` gana con una fila: `antes` con `actualizado` de hoy y
/// la fila `fase` de `por` (la ultima linea de `despues`).
pub fn con_fila(antes: &str, despues: &str, fase: &str, por: &str) -> String {
    let hoy = tc2::toma::instante(tc2::identidad::ahora());
    let actualizado = antes
        .lines()
        .find_map(|l| l.strip_prefix("actualizado: "))
        .unwrap()
        .to_owned();
    let fila = despues.lines().last().unwrap();
    assert!(fila.starts_with(&format!("| {}T", &hoy[..10])), "{fila}");
    assert!(
        fila.ends_with(&format!(" | {fase} | manual | {por} | {} |", tc2::VERSION)),
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

/// Worktree de la tarea `id` en el clon `dir`.
pub fn worktree(dir: &Path, id: &str) -> PathBuf {
    dir.join(".taskcodetwo").join("worktrees").join(id)
}

/// La cuenta de GitHub `id`/`login`.
pub fn cuenta(id: u64, login: &str) -> Cuenta {
    Cuenta {
        proveedor: Proveedor::GitHub,
        id,
        login: login.to_owned(),
    }
}

/// Identifica el clon `dir` como la persona `id`/`login` (cache vigente).
pub fn identificar(dir: &Path, id: u64, login: &str) {
    Cache::en(&dir.join(".git"))
        .guardar(&cuenta(id, login), tc2::identidad::ahora())
        .unwrap();
}

/// `op` con la biblioteca, lanzada por la persona `cuenta` en un terminal
/// (`Quien::Persona`, que el binario solo da con un token real): la linea de
/// contrato.
pub fn como_persona(
    dir: &Path,
    entorno: Vec<(OsString, OsString)>,
    cuenta: Cuenta,
    op: impl FnOnce(&Entorno<'_>) -> Informe,
) -> String {
    let g = Git::nuevo().con_entorno(entorno);
    let q = move || -> Result<Quien, Informe> { Ok(Quien::Persona(cuenta.clone())) };
    let e = Entorno {
        dir,
        git: &g,
        quien: &q,
        catalogo: &sin_catalogo,
        ahora: tc2::identidad::ahora(),
    };
    op(&e).linea()
}
