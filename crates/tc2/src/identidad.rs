//! Identidad de la persona (F2-13, RR-12): el «quien» de una tarea es la
//! cuenta de GitHub o GitLab autenticada, no un texto libre.
//!
//! - Con terminal, la persona se identifica por la API con su token
//!   (`GET /user` por el puerto [`Plataforma`], sin `gh` ni `glab`, F1-14):
//!   tiene que ser `User`, no `web-flow` y estar en `.taskcode/personas.yml`
//!   por su `id` (F1-6, decision 3). `tc2 identidad` la guarda en la cache
//!   local.
//! - Sin terminal, quien ejecuta es el agente, en nombre de la persona de la
//!   cache (D3, RN-06): no pide nada a la red y su transicion lleva
//!   `decidido_por: agente`. Sin cache, caducada (7 dias) o de otra
//!   plataforma, se rechaza pidiendo `tc2 identidad`.
//! - Sin plataforma reconocida no hay identidad (D4): no hay vuelta a texto
//!   libre.
//!
//! La cache vive en `<git-common-dir>/tc2/identidad.yml`: no se commitea y la
//! comparten los worktrees del clon. Editarla no aprueba nada: la aprobacion
//! sigue siendo la firma de F1-6/F4-2 (riesgo aceptado en el plan).

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

use crate::config;
use crate::contrato::{Informe, Resultado};
use crate::git::{self, Git, Paso, Subcomando};
use crate::i18n::catalogo::identidad as t;
use crate::i18n::{Mensaje, Texto};
use crate::plataforma::personas::RUTA as RUTA_PERSONAS;
use crate::plataforma::{
    Credenciales, GitHub, GitLab, Personas, Plataforma, Proveedor, Rol, Usuario, detectar,
};
use crate::tarea::Asignado;

/// Nombre de la orden y de la operacion en la linea.
pub const NOMBRE: &str = "identidad";

/// Vigencia de la identidad guardada: 7 dias desde que se verifico (D2).
pub const TTL_SEGUNDOS: u64 = 7 * 24 * 60 * 60;

/// Margen para un reloj adelantado: una cache «del futuro» mas alla de esto
/// no vale.
const MARGEN_RELOJ: u64 = 5 * 60;

/// Carpeta y fichero de la cache bajo la carpeta comun del repo.
const CARPETA_CACHE: &str = "tc2";
const FICHERO_CACHE: &str = "identidad.yml";

/// Tamano maximo de la cache que se lee (la que escribe `tc2` ocupa menos
/// de 400 bytes).
pub const LIMITE_CACHE: u64 = 64 * 1024;

/// Cuenta de la plataforma que puede llevar tareas: persona de la lista.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cuenta {
    /// Plataforma de la cuenta (los `id` de GitHub y GitLab no se comparan).
    pub proveedor: Proveedor,
    /// Id numerico.
    pub id: u64,
    /// Login (para ensenar).
    pub login: String,
}

/// Quien ejecuta una operacion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quien {
    /// La persona, verificada ahora por la API con su token (terminal).
    Persona(Cuenta),
    /// El agente, sin terminal, en nombre de la persona de la cache.
    Agente(Cuenta),
}

impl Quien {
    /// La cuenta de la persona (la propia o en cuyo nombre actua el agente).
    pub fn cuenta(&self) -> &Cuenta {
        match self {
            Self::Persona(c) | Self::Agente(c) => c,
        }
    }

    /// Valor de `decidido_por` de la transicion.
    pub fn decidido_por(&self) -> &'static str {
        match self {
            Self::Persona(_) => "persona",
            Self::Agente(_) => "agente",
        }
    }

    /// El `asignado_a` que escribe una toma: siempre la persona, nunca un
    /// texto.
    pub fn asignado(&self) -> Asignado {
        let c = self.cuenta();
        Asignado::Persona {
            id: c.id,
            login: c.login.clone(),
        }
    }
}

/// Por que una cuenta no puede llevar una tarea.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoAsignable {
    /// Cuenta `Bot`.
    Bot,
    /// Otro tipo que no es `User` ni `Bot` (`Organization`, `Mannequin`).
    NoPersona,
    /// `web-flow`, la cuenta de la web de GitHub.
    WebFlow,
    /// No esta en `.taskcode/personas.yml`.
    FueraDeLista,
    /// La API dio un login vacio o con espacios o controles.
    LoginNoValido,
}

impl NoAsignable {
    /// Valor de `cuenta=` en la linea.
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Bot => "bot",
            Self::NoPersona => "no-persona",
            Self::WebFlow => "web-flow",
            Self::FueraDeLista => "fuera-de-lista",
            Self::LoginNoValido => "login-no-valido",
        }
    }

    /// El rechazo, con el arreglo para la persona.
    pub fn informe(self, u: &Usuario) -> Informe {
        let login = crate::plataforma::errores::sanear(&u.login);
        let m = match self {
            Self::Bot => Mensaje::nuevo(&t::CUENTA_BOT).con("login", login),
            Self::NoPersona => Mensaje::nuevo(&t::CUENTA_NO_PERSONA)
                .con("login", login)
                .con("tipo", crate::plataforma::errores::sanear(&u.tipo)),
            Self::WebFlow => Mensaje::nuevo(&t::CUENTA_WEB_FLOW),
            Self::FueraDeLista => Mensaje::nuevo(&t::CUENTA_FUERA_DE_LISTA)
                .con("login", login)
                .con("id", u.id.to_string())
                .con("ruta", RUTA_PERSONAS)
                .con("base", config::BASE),
            Self::LoginNoValido => Mensaje::nuevo(&t::LOGIN_DE_LA_API)
                .con("login", login)
                .con("id", u.id.to_string()),
        };
        Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", "identidad")
            .con("rol", "persona")
            .con("cuenta", self.nombre())
            .diagnostico(m)
    }
}

/// La cuenta `u` de `proveedor`, si puede llevar tareas: `User`, no
/// `web-flow` y en la lista por su `id`. Una cuenta `Bot` nunca llega a
/// `asignado_a`.
pub fn asignable(
    u: &Usuario,
    personas: &Personas,
    proveedor: Proveedor,
) -> Result<Cuenta, NoAsignable> {
    match u.tipo.as_str() {
        "User" => {}
        "Bot" => return Err(NoAsignable::Bot),
        _ => return Err(NoAsignable::NoPersona),
    }
    if u.login == "web-flow" {
        return Err(NoAsignable::WebFlow);
    }
    if !personas.contiene(u.id) {
        return Err(NoAsignable::FueraDeLista);
    }
    // El mismo criterio de login que `asignado_a` (un login que la tarea no
    // podria guardar no se acepta aqui).
    Asignado::persona(u.id, &u.login).ok_or(NoAsignable::LoginNoValido)?;
    Ok(Cuenta {
        proveedor,
        id: u.id,
        login: u.login.clone(),
    })
}

/// Identidad guardada en la cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guardada {
    /// La cuenta.
    pub cuenta: Cuenta,
    /// Cuando se verifico por la API, en segundos desde 1970.
    pub verificado: u64,
}

impl Guardada {
    /// Si sigue valiendo en `ahora`: menos de 7 dias y no del futuro.
    pub fn vigente(&self, ahora: u64) -> bool {
        self.verificado <= ahora.saturating_add(MARGEN_RELOJ)
            && ahora.saturating_sub(self.verificado) < TTL_SEGUNDOS
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GuardadaCruda {
    proveedor: String,
    id: u64,
    login: String,
    verificado: u64,
}

/// Cache local de la identidad (`<git-common-dir>/tc2/identidad.yml`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cache {
    ruta: PathBuf,
}

impl Cache {
    /// La cache del repo cuya carpeta comun es `comun`.
    pub fn en(comun: &Path) -> Self {
        Self {
            ruta: comun.join(CARPETA_CACHE).join(FICHERO_CACHE),
        }
    }

    /// Ruta del fichero.
    pub fn ruta(&self) -> &Path {
        &self.ruta
    }

    /// Lo guardado; `None` si no hay o no se entiende (se trata como si no
    /// hubiera: hay que volver a lanzar `tc2 identidad`).
    pub fn leer(&self) -> Option<Guardada> {
        use std::io::Read;
        // Con tope: un fichero mayor no es una cache de `tc2` (se trata como
        // corrupta, es decir, como si no hubiera).
        let mut texto = String::new();
        std::fs::File::open(&self.ruta)
            .ok()?
            .take(LIMITE_CACHE + 1)
            .read_to_string(&mut texto)
            .ok()?;
        if texto.len() as u64 > LIMITE_CACHE {
            return None;
        }
        let c: GuardadaCruda =
            serde_saphyr::from_str_with_options(&texto, crate::yaml::opciones()).ok()?;
        let proveedor = match c.proveedor.as_str() {
            "github" => Proveedor::GitHub,
            "gitlab" => Proveedor::GitLab,
            _ => return None,
        };
        Asignado::persona(c.id, &c.login)?;
        Some(Guardada {
            cuenta: Cuenta {
                proveedor,
                id: c.id,
                login: c.login,
            },
            verificado: c.verificado,
        })
    }

    /// Guarda `cuenta` verificada en `ahora`. Escribe un temporal y lo
    /// renombra: quien lee no ve nunca un fichero a medias.
    pub fn guardar(&self, cuenta: &Cuenta, ahora: u64) -> Paso<()> {
        let error = |e: std::io::Error| {
            Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
                Mensaje::nuevo(&t::NO_SE_ESCRIBE_CACHE)
                    .con("ruta", self.ruta.display().to_string())
                    .con("error", e.to_string()),
            )
        };
        if let Some(padre) = self.ruta.parent() {
            std::fs::create_dir_all(padre).map_err(error)?;
        }
        let texto = format!(
            "---\nproveedor: \"{}\"\nid: {}\nlogin: {}\nverificado: {ahora}\n",
            cuenta.proveedor.nombre(),
            cuenta.id,
            crate::tarea::yaml::cadena(&cuenta.login),
        );
        let tmp = self
            .ruta
            .with_extension(format!("yml.tmp-{}", std::process::id()));
        std::fs::write(&tmp, texto).map_err(error)?;
        std::fs::rename(&tmp, &self.ruta).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            error(e)
        })
    }
}

/// Segundos desde 1970 (0 si el reloj esta antes, que deja cualquier cache
/// por caducada o del futuro).
pub fn ahora() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Rechazo por falta de una identidad guardada que valga.
fn sin_guardada(estado: &'static str, que: Mensaje) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Rechazada)
        .con("motivo", "identidad")
        .con("cache", estado)
        .diagnostico(Mensaje::nuevo(&t::SIN_IDENTIDAD_GUARDADA).con_mensaje("estado", que))
}

/// Quien ejecuta: con terminal, la persona del token verificada ahora por la
/// API; sin terminal, el agente en nombre de la persona de la cache vigente,
/// sin pedir nada a la red. La persona tiene que estar en `personas` (por
/// `id`; su login sale de la lista si esta, que refresca un renombrado).
pub fn quien_ejecuta(
    plataforma: &dyn Plataforma,
    proveedor: Proveedor,
    personas: &Personas,
    cache: &Cache,
    tty: bool,
    ahora: u64,
) -> Paso<Quien> {
    if tty {
        let u = plataforma.identidad(Rol::Persona).map_err(|mut i| {
            NOMBRE.clone_into(&mut i.comando);
            i
        })?;
        return asignable(&u, personas, proveedor)
            .map(Quien::Persona)
            .map_err(|n| n.informe(&u));
    }
    let Some(g) = cache.leer() else {
        return Err(sin_guardada("falta", Mensaje::nuevo(&t::CACHE_FALTA)));
    };
    if g.cuenta.proveedor != proveedor {
        return Err(sin_guardada(
            "otra-plataforma",
            Mensaje::nuevo(&t::CACHE_OTRA_PLATAFORMA),
        ));
    }
    if !g.vigente(ahora) {
        return Err(sin_guardada("caducada", Mensaje::nuevo(&t::CACHE_CADUCADA)));
    }
    if !personas.contiene(g.cuenta.id) {
        return Err(sin_guardada(
            "fuera-de-lista",
            Mensaje::nuevo(&t::CACHE_FUERA_DE_LISTA)
                .con("login", g.cuenta.login.clone())
                .con("id", g.cuenta.id.to_string()),
        ));
    }
    let mut cuenta = g.cuenta;
    if let Some(p) = personas.por_id(cuenta.id) {
        cuenta.login.clone_from(&p.login);
    }
    Ok(Quien::Agente(cuenta))
}

/// Convierte un `asignado_a` heredado de la 0.16 si su texto es un
/// `correos:` de una persona de la lista (D5); si no casa, queda como esta
/// (`doctor` avisa y `reasignar`, F3-10, lo resuelve).
pub fn convertir_heredado(a: Asignado, personas: &Personas) -> Asignado {
    let persona = a
        .heredado()
        .and_then(|texto| personas.por_correo(texto))
        .and_then(|p| Asignado::persona(p.id, &p.login));
    persona.unwrap_or(a)
}

/// Lo que necesita la orden `tc2 identidad` del entorno, para poder
/// probarla sin terminal real ni tokens del proceso.
pub struct Entorno<'a> {
    /// Carpeta desde la que se lanza.
    pub dir: &'a Path,
    /// Puerto Git.
    pub git: &'a Git,
    /// Si la entrada estandar es un terminal.
    pub tty: bool,
    /// Tokens por rol.
    pub credenciales: Box<dyn Credenciales>,
    /// Hora, en segundos desde 1970.
    pub ahora: u64,
}

/// Lista de personas de la base (`origin/develop`), nunca del worktree:
/// cambiarla exige un PR aprobado (F1-6, decision 3).
pub fn personas_de_base(g: &Git, dir: &Path) -> Paso<Personas> {
    let falta = || {
        Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", "sin-personas")
            .diagnostico(
                Mensaje::nuevo(&t::SIN_PERSONAS)
                    .con("ruta", RUTA_PERSONAS)
                    .con("base", config::BASE),
            )
    };
    if git::commit_de(g, dir, NOMBRE, config::BASE)?.is_none() {
        return Err(falta());
    }
    let listado = git::exigir(
        g,
        dir,
        NOMBRE,
        Subcomando::LsTree,
        &[
            "--name-only",
            "--end-of-options",
            config::BASE,
            "--",
            RUTA_PERSONAS,
        ],
    )?;
    if listado.trim() != RUTA_PERSONAS {
        return Err(falta());
    }
    let texto = git::estado::leer_blob(g, dir, NOMBRE, config::BASE, RUTA_PERSONAS)?;
    Personas::leer(&texto).map_err(|mut i| {
        NOMBRE.clone_into(&mut i.comando);
        i
    })
}

/// La clave `plataforma` de la configuracion de la base, si la hay. Sin
/// base o sin configuracion se detecta solo por el host; una configuracion
/// que no vale falla.
fn clave_plataforma(g: &Git, dir: &Path) -> Paso<Option<(&'static str, String)>> {
    match config::de_base(g, dir) {
        Ok(c) => Ok(c.plataforma.map(|p| (p.proveedor.nombre(), p.api))),
        Err(i) if matches!(i.valor("motivo"), Some("sin-base" | "sin-config")) => Ok(None),
        Err(mut i) => {
            NOMBRE.clone_into(&mut i.comando);
            Err(i)
        }
    }
}

/// Uso de la orden.
fn uso(diag: &'static Texto) -> Informe {
    Informe::nuevo(NOMBRE, Resultado::Uso)
        .con("motivo", "argumentos-de-mas")
        .diagnostico(diag)
}

/// `tc2 identidad`: con terminal y `TC2_TOKEN_PERSONA`, verifica la persona
/// por la API y la guarda en la cache. Sale `identidad ok rol=persona id=N
/// login=L`. Si no es una persona de la lista, sin red o sin plataforma
/// reconocida, falla sin tocar la cache.
pub fn orden(args: &[String], e: Entorno<'_>) -> Informe {
    if !args.is_empty() {
        return uso(&t::USO_IDENTIDAD);
    }
    match orden_paso(e) {
        Ok(i) | Err(i) => i,
    }
}

fn orden_paso(e: Entorno<'_>) -> Paso<Informe> {
    if !e.tty {
        return Err(Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", "identidad")
            .con("rol", "persona")
            .diagnostico(&t::NECESITA_TERMINAL));
    }
    let comun = git::estado::dir_comun(e.git, e.dir, NOMBRE)?;
    let Some(url) = git::estado::url_origin(e.git, e.dir, NOMBRE)? else {
        return Err(Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", "plataforma")
            .diagnostico(&t::SIN_ORIGIN));
    };
    // Con terminal ya hace falta red (la API): se trae antes la rama base,
    // para que la lista de personas y la configuracion sean las de ahora
    // (quien acaba de entrar en la lista por un PR fusionado). Sin red,
    // `sinred` y la cache sin tocar.
    let rama_base = config::BASE.strip_prefix("origin/").unwrap_or(config::BASE);
    git::remoto::traer_rama(e.git, e.dir, NOMBRE, rama_base)?;
    let clave = clave_plataforma(e.git, e.dir)?;
    let destino =
        detectar(&url, clave.as_ref().map(|(p, a)| (*p, a.as_str()))).map_err(|mut i| {
            NOMBRE.clone_into(&mut i.comando);
            i
        })?;
    let personas = personas_de_base(e.git, e.dir)?;
    let plataforma: Box<dyn Plataforma> = match destino.proveedor {
        Proveedor::GitHub => Box::new(GitHub::nuevo(
            destino.api,
            &destino.proyecto,
            e.credenciales,
            personas.clone(),
        )?),
        Proveedor::GitLab => Box::new(GitLab::nuevo(
            destino.api,
            &destino.proyecto,
            e.credenciales,
            personas.clone(),
        )?),
    };
    let cache = Cache::en(&comun);
    let quien = quien_ejecuta(
        plataforma.as_ref(),
        destino.proveedor,
        &personas,
        &cache,
        true,
        e.ahora,
    )?;
    let c = quien.cuenta();
    cache.guardar(c, e.ahora)?;
    Ok(Informe::nuevo(NOMBRE, Resultado::Ok)
        .con("rol", "persona")
        .con("id", c.id.to_string())
        .con("login", c.login.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plataforma::{Firma, ModeloPuertas, ReglasLeidas};
    use std::cell::Cell;

    /// Plataforma de prueba: solo `identidad`, contando las llamadas.
    struct Falsa {
        usuario: Usuario,
        llamadas: Cell<usize>,
    }

    impl Falsa {
        fn de(login: &str, id: u64, tipo: &str) -> Self {
            Self {
                usuario: Usuario {
                    login: login.to_owned(),
                    id,
                    tipo: tipo.to_owned(),
                },
                llamadas: Cell::new(0),
            }
        }
    }

    fn sin_uso(op: &str) -> Informe {
        Informe::nuevo(op, Resultado::Error)
    }

    impl Plataforma for Falsa {
        fn abrir_pr(&self, _: &str, _: &str, _: &str, _: &str) -> Informe {
            sin_uso("abrir-pr")
        }
        fn estado_puertas(&self, _: Rol, _: u64, _: &ModeloPuertas) -> Informe {
            sin_uso("estado-puertas")
        }
        fn merge_pr(&self, _: u64, _: &str) -> Informe {
            sin_uso("merge-pr")
        }
        fn leer_puertas(&self, _: Rol, _: &str) -> Paso<ReglasLeidas> {
            Err(sin_uso("leer-puertas"))
        }
        fn leer_permisos_ci(&self) -> Informe {
            sin_uso("leer-permisos-ci")
        }
        fn firma_commit(&self, _: &str) -> Paso<Firma> {
            Err(sin_uso("firma-commit"))
        }
        fn identidad(&self, _: Rol) -> Paso<Usuario> {
            self.llamadas.set(self.llamadas.get() + 1);
            Ok(self.usuario.clone())
        }
    }

    fn carpeta(nombre: &str) -> PathBuf {
        let d = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("identidad-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    const AHORA: u64 = 1_800_000_000;

    /// C6: una cuenta `Bot` (o `web-flow`, o fuera de la lista) nunca acaba
    /// en `asignado_a`: error tipado. Sin terminal, `Quien` es `Agente` y la
    /// transicion lleva `decidido_por: agente`; con terminal, `persona`.
    #[test]
    fn bot_nunca_asignado_y_sin_tty_es_agente() {
        let personas = Personas::nuevas([7, 8]);
        let u = |login: &str, id, tipo: &str| Usuario {
            login: login.to_owned(),
            id,
            tipo: tipo.to_owned(),
        };
        let gh = Proveedor::GitHub;
        assert_eq!(
            asignable(&u("ci[bot]", 7, "Bot"), &personas, gh),
            Err(NoAsignable::Bot)
        );
        assert_eq!(
            asignable(&u("web-flow", 7, "User"), &personas, gh),
            Err(NoAsignable::WebFlow)
        );
        assert_eq!(
            asignable(&u("otra", 9, "User"), &personas, gh),
            Err(NoAsignable::FueraDeLista)
        );
        assert_eq!(
            asignable(&u("a b", 7, "User"), &personas, gh),
            Err(NoAsignable::LoginNoValido)
        );
        let bot = Falsa::de("ci[bot]", 7, "Bot");
        let cache = Cache::en(&carpeta("bot"));
        let r = quien_ejecuta(&bot, gh, &personas, &cache, true, AHORA);
        assert_eq!(
            r.err().map(|i| i.linea()).as_deref(),
            Some("identidad rechazada motivo=identidad rol=persona cuenta=bot")
        );
        // Con terminal: persona.
        let ana = Falsa::de("ana", 7, "User");
        let q = quien_ejecuta(&ana, gh, &personas, &cache, true, AHORA);
        assert_eq!(q.as_ref().map(Quien::decidido_por), Ok("persona"));
        assert_eq!(
            q.map(|q| q.asignado()),
            Ok(Asignado::Persona {
                id: 7,
                login: "ana".to_owned()
            })
        );
        // Sin terminal, con la cache de ana: agente en su nombre, sin
        // llamar a la plataforma.
        let cuenta = Cuenta {
            proveedor: gh,
            id: 7,
            login: "ana".to_owned(),
        };
        assert_eq!(cache.guardar(&cuenta, AHORA), Ok(()));
        let antes = ana.llamadas.get();
        let q = quien_ejecuta(&ana, gh, &personas, &cache, false, AHORA + 60);
        assert_eq!(q.as_ref().map(Quien::decidido_por), Ok("agente"));
        assert_eq!(q.map(|q| q.cuenta().clone()), Ok(cuenta));
        assert_eq!(ana.llamadas.get(), antes, "sin terminal no hay llamadas");
        let _ = std::fs::remove_dir_all(carpeta("bot"));
    }

    /// C7 (biblioteca): cache que falta, caducada, del futuro, de otra
    /// plataforma o de alguien que ya no esta en la lista: rechazada pidiendo
    /// `tc2 identidad`. El login sale de la lista si esta (renombrado).
    #[test]
    fn cache_vigencia_y_rechazos() -> Result<(), Informe> {
        let d = carpeta("cache");
        let cache = Cache::en(&d);
        let gh = Proveedor::GitHub;
        let personas = Personas::leer("personas:\n  - {id: 7, login: \"ana-nueva\"}\n")?;
        let p = Falsa::de("x", 1, "User");
        let linea = |ahora| {
            quien_ejecuta(&p, gh, &personas, &cache, false, ahora)
                .err()
                .map(|i| i.linea())
                .unwrap_or_default()
        };
        assert_eq!(
            linea(AHORA),
            "identidad rechazada motivo=identidad cache=falta"
        );
        let ana = Cuenta {
            proveedor: gh,
            id: 7,
            login: "ana".to_owned(),
        };
        assert_eq!(cache.guardar(&ana, AHORA), Ok(()));
        assert!(linea(AHORA + TTL_SEGUNDOS).ends_with("cache=caducada"));
        assert!(linea(AHORA - MARGEN_RELOJ - 1).ends_with("cache=caducada"));
        let q = quien_ejecuta(&p, gh, &personas, &cache, false, AHORA + TTL_SEGUNDOS - 1);
        assert_eq!(
            q.map(|q| q.cuenta().login.clone()).as_deref(),
            Ok("ana-nueva")
        );
        let otra = quien_ejecuta(&p, Proveedor::GitLab, &personas, &cache, false, AHORA);
        assert!(
            otra.err()
                .is_some_and(|i| i.linea().ends_with("cache=otra-plataforma"))
        );
        let fuera = quien_ejecuta(&p, gh, &Personas::nuevas([8]), &cache, false, AHORA);
        assert!(
            fuera
                .err()
                .is_some_and(|i| i.linea().ends_with("cache=fuera-de-lista"))
        );
        // Un fichero que no se entiende cuenta como si no hubiera.
        assert!(std::fs::write(cache.ruta(), "---\nid: x\n").is_ok());
        assert!(cache.leer().is_none());
        assert!(linea(AHORA).ends_with("cache=falta"));
        let _ = std::fs::remove_dir_all(&d);
        Ok(())
    }

    /// Un tipo que no es `User` ni `Bot` (`Organization`, `Mannequin`) no se
    /// presenta como bot: `cuenta=no-persona`.
    #[test]
    fn otro_tipo_de_cuenta_es_no_persona_y_no_bot() {
        let personas = Personas::nuevas([7]);
        for tipo in ["Organization", "Mannequin"] {
            let u = Usuario {
                login: "org".to_owned(),
                id: 7,
                tipo: tipo.to_owned(),
            };
            let r = asignable(&u, &personas, Proveedor::GitHub);
            assert_eq!(r, Err(NoAsignable::NoPersona), "{tipo}");
            let i = NoAsignable::NoPersona.informe(&u);
            assert_eq!(
                i.linea(),
                "identidad rechazada motivo=identidad rol=persona cuenta=no-persona"
            );
            for idioma in [crate::i18n::Idioma::Es, crate::i18n::Idioma::En] {
                let d = i.diagnostico_en(idioma).unwrap_or_default();
                assert!(d.contains(tipo) && !d.contains("bot"), "{d}");
            }
        }
    }

    /// La cache se lee con tope (64 KiB): mayor, cuenta como ausente. Un
    /// login de mas de 255 caracteres no vale ni en la API ni en la cache.
    #[test]
    fn cache_con_tope_y_login_con_longitud_maxima() {
        let d = carpeta("tope");
        let cache = Cache::en(&d);
        let cuenta = |login: &str| Cuenta {
            proveedor: Proveedor::GitHub,
            id: 7,
            login: login.to_owned(),
        };
        assert_eq!(cache.guardar(&cuenta("ana"), AHORA), Ok(()));
        assert!(cache.leer().is_some());
        // La misma cache con relleno de comentario hasta pasar el tope.
        let texto = std::fs::read_to_string(cache.ruta()).unwrap_or_default();
        let relleno = format!("{texto}#{}\n", "x".repeat(70 * 1024));
        assert!(std::fs::write(cache.ruta(), &relleno).is_ok());
        assert!(cache.leer().is_none(), "mayor que el tope: ausente");
        let justo = format!(
            "{texto}#{}\n",
            "x".repeat(usize::try_from(LIMITE_CACHE).unwrap_or(0) - texto.len() - 2)
        );
        assert_eq!(justo.len() as u64, LIMITE_CACHE);
        assert!(std::fs::write(cache.ruta(), &justo).is_ok());
        assert!(cache.leer().is_some(), "en el tope exacto: vale");
        // Login: 255 vale, 256 no.
        let (max, largo) = ("a".repeat(255), "a".repeat(256));
        assert!(Asignado::persona(7, &max).is_some());
        assert!(Asignado::persona(7, &largo).is_none());
        let u = Usuario {
            login: largo.clone(),
            id: 7,
            tipo: "User".to_owned(),
        };
        assert_eq!(
            asignable(&u, &Personas::nuevas([7]), Proveedor::GitHub),
            Err(NoAsignable::LoginNoValido)
        );
        assert_eq!(cache.guardar(&cuenta(&max), AHORA), Ok(()));
        assert!(cache.leer().is_some());
        assert_eq!(cache.guardar(&cuenta(&largo), AHORA), Ok(()));
        assert!(cache.leer().is_none(), "login de mas en la cache: ausente");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn convertir_no_toca_una_persona() -> Result<(), Informe> {
        let personas =
            Personas::leer("personas:\n  - {id: 7, login: \"ana\", correos: [\"ana@x.com\"]}\n")?;
        let p = Asignado::persona(9, "luis");
        assert_eq!(p.clone().map(|a| convertir_heredado(a, &personas)), p);
        Ok(())
    }
}
