//! Operacion `commit-estado` de la tabla de F1-4: commits de la rama de
//! estado construidos sin tocar ningun worktree ni el indice del repo
//! (F2-3). El arbol se arma en un indice temporal sobre el commit base y el
//! commit sale de `commit-tree`; publicarlo es un `push <sha>:<rama>` sin
//! force. Asi, tras un push rechazado no hay nada local que descartar: no
//! hace falta `reset` (prohibido por F1-4 D3).

use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::puerto::{HASH_OBJECT_STDIN, UPDATE_INDEX_STDIN};
use super::rama::error_de_fichero;
use super::{Git, Paso, Subcomando, correr, exigida, exigir, lanzado};
use crate::contrato::{Informe, Resultado};
use crate::i18n::Mensaje;
use crate::i18n::catalogo::git as txt;

/// Un fichero que la operacion anade o cambia en la rama de estado. No hay
/// borrado: las carpetas de tarea no se borran nunca (ADR F1-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cambio {
    /// Ruta relativa a la raiz de la rama, con `/`.
    pub ruta: String,
    /// Contenido completo del fichero.
    pub contenido: String,
}

impl Cambio {
    /// Cambio de `ruta` con `contenido`.
    pub fn nuevo(ruta: impl Into<String>, contenido: impl Into<String>) -> Self {
        Self {
            ruta: ruta.into(),
            contenido: contenido.into(),
        }
    }
}

/// Ruta valida dentro de la rama: relativa, con `/`, sin `.`, `..`, partes
/// vacias ni `.git`.
pub(crate) fn ruta_valida(ruta: &str) -> bool {
    !ruta.is_empty()
        && !ruta.contains('\\')
        && !ruta.contains(',')
        && !ruta.contains(':')
        && !ruta.chars().any(char::is_control)
        && !ruta.contains(['<', '>', '"', '|', '?', '*'])
        && ruta.split('/').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && !p.eq_ignore_ascii_case(".git")
                && !p.ends_with(['.', ' '])
                && !reservado_en_windows(p)
        })
        && Path::new(ruta)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}

/// Nombres que Windows reserva aunque lleven extension (`aux.md`): un clon en
/// Windows no podria sacarlos al worktree.
pub(crate) fn reservado_en_windows(parte: &str) -> bool {
    let base = parte
        .split('.')
        .next()
        .unwrap_or(parte)
        .to_ascii_uppercase();
    matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((base.starts_with("COM") || base.starts_with("LPT"))
            && base.len() == 4
            && base
                .as_bytes()
                .get(3)
                .is_some_and(|b| (b'1'..=b'9').contains(b)))
}

/// Carpeta comun del repo (`.git` del principal), absoluta.
pub(crate) fn dir_comun(git: &Git, dir: &Path, op: &str) -> Paso<PathBuf> {
    let s = exigir(
        git,
        dir,
        op,
        Subcomando::RevParse,
        &["--path-format=absolute", "--git-common-dir"],
    )?;
    Ok(PathBuf::from(s.trim()))
}

/// Rutas de todos los ficheros de `commit`, ordenadas.
pub(crate) fn listar_arbol(git: &Git, dir: &Path, op: &str, commit: &str) -> Paso<Vec<String>> {
    let s = exigir(
        git,
        dir,
        op,
        Subcomando::LsTree,
        &["-r", "-z", "--name-only", "--end-of-options", commit],
    )?;
    let mut v: Vec<String> = s
        .split('\0')
        .filter(|r| !r.is_empty())
        .map(str::to_owned)
        .collect();
    v.sort();
    Ok(v)
}

/// Contenido de `ruta` en `commit`.
pub(crate) fn leer_blob(git: &Git, dir: &Path, op: &str, commit: &str, ruta: &str) -> Paso<String> {
    exigir(
        git,
        dir,
        op,
        Subcomando::Show,
        &[&format!("{commit}:{ruta}")],
    )
}

/// Contenido de varias `rutas` de `commit` con un solo `git cat-file
/// --batch` (F2-4b): en el mismo orden, `None` si no existe o no es un
/// fichero. Una ruta con salto de linea no cabe en la lista de stdin: esa se
/// lee sola con [`leer_blob`].
pub(crate) fn leer_blobs(
    git: &Git,
    dir: &Path,
    op: &str,
    commit: &str,
    rutas: &[&str],
) -> Paso<Vec<Option<String>>> {
    let (en_lote, sueltas): (Vec<&str>, Vec<&str>) =
        rutas.iter().partition(|r| !r.contains(['\n', '\r']));
    let mut leidos: std::collections::HashMap<&str, Option<String>> =
        std::collections::HashMap::new();
    if !en_lote.is_empty() {
        let entrada: String = en_lote.iter().map(|r| format!("{commit}:{r}\n")).collect();
        let sub = Subcomando::CatFile;
        let s = exigida(
            op,
            sub,
            git.ejecutar_con_entrada(dir, sub, &["--batch"], entrada.as_bytes()),
        )?;
        let contenidos = partir_batch(&s.stdout, en_lote.len()).ok_or_else(|| {
            Informe::nuevo(op, Resultado::Error).diagnostico(&txt::CAT_FILE_INCOMPLETO)
        })?;
        leidos.extend(en_lote.iter().copied().zip(contenidos));
    }
    for r in sueltas {
        leidos.insert(r, Some(leer_blob(git, dir, op, commit, r)?));
    }
    Ok(rutas
        .iter()
        .map(|r| leidos.get(r).cloned().flatten())
        .collect())
}

/// Parte la salida de `cat-file --batch` en `n` objetos: por cada uno,
/// `<sha> <tipo> <bytes>\n<contenido>\n`, o `<nombre> missing\n`. Solo los
/// `blob` dan contenido. `None` si la salida no tiene esa forma.
fn partir_batch(salida: &[u8], n: usize) -> Option<Vec<Option<String>>> {
    let mut v = Vec::with_capacity(n);
    let mut resto = salida;
    for _ in 0..n {
        let fin = resto.iter().position(|b| *b == b'\n')?;
        let cabecera = std::str::from_utf8(resto.get(..fin)?).ok()?;
        resto = resto.get(fin + 1..)?;
        let mut campos = cabecera.rsplitn(3, ' ');
        let ultimo = campos.next()?;
        if ultimo == "missing" || ultimo == "ambiguous" {
            v.push(None);
            continue;
        }
        let bytes: usize = ultimo.parse().ok()?;
        let tipo = campos.next()?;
        let contenido = resto.get(..bytes)?;
        if resto.get(bytes) != Some(&b'\n') {
            return None;
        }
        resto = resto.get(bytes + 1..)?;
        v.push((tipo == "blob").then(|| String::from_utf8_lossy(contenido).into_owned()));
    }
    resto.is_empty().then_some(v)
}

/// Mensaje completo de un commit.
pub(crate) fn mensaje_de(git: &Git, dir: &Path, op: &str, commit: &str) -> Paso<String> {
    exigir(
        git,
        dir,
        op,
        Subcomando::Show,
        &["-s", "--format=%B", "--end-of-options", commit],
    )
}

static TEMPORALES: AtomicUsize = AtomicUsize::new(0);

/// Carpeta temporal bajo `<comun>/taskcodetwo/estado-tmp/`; se borra al
/// soltarla, tambien si la operacion termina con error.
struct Temporal(PathBuf);

impl Temporal {
    fn nueva(comun: &Path, op: &str) -> Paso<Self> {
        let n = TEMPORALES.fetch_add(1, Ordering::SeqCst);
        let ruta = crate::huella::en_comun(comun)
            .join(crate::huella::CARPETA_TEMPORALES)
            .join(format!("{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&ruta);
        std::fs::create_dir_all(&ruta).map_err(|e| {
            Informe::nuevo(op, Resultado::Error).diagnostico(error_de_fichero(&ruta, &e))
        })?;
        Ok(Self(ruta))
    }
}

impl Drop for Temporal {
    fn drop(&mut self) {
        // Borrar es limpieza: si falla, no hay a quien avisar desde `Drop` y
        // la carpeta no afecta a la correccion (la vacia el siguiente que
        // use el mismo nombre). La carpeta padre se quita si queda vacia
        // (`remove_dir` no borra una con contenido de otro proceso).
        let _ = std::fs::remove_dir_all(&self.0);
        if let Some(padre) = self.0.parent() {
            let _ = std::fs::remove_dir(padre);
        }
    }
}

fn texto(op: &str, p: &Path) -> Paso<String> {
    p.to_str().map(str::to_owned).ok_or_else(|| {
        Informe::nuevo(op, Resultado::Error).diagnostico(&txt::RUTA_TEMPORAL_NO_UTF8)
    })
}

fn con_indice(
    git: &Git,
    dir: &Path,
    op: &str,
    sub: Subcomando,
    args: &[&str],
    indice: &Path,
) -> Paso<String> {
    Ok(exigida(op, sub, git.ejecutar_con_indice(dir, sub, args, indice))?.stdout)
}

/// `hash-object` (sin indice) o `update-index` (con `indice`) en su unica
/// forma con entrada por stdin; un codigo distinto de 0 es `error`.
fn con_entrada(
    git: &Git,
    dir: &Path,
    op: &str,
    sub: Subcomando,
    indice: Option<&Path>,
    entrada: &[u8],
) -> Paso<Vec<u8>> {
    let r = match indice {
        None => git.ejecutar_con_entrada(dir, sub, HASH_OBJECT_STDIN, entrada),
        Some(i) => git.ejecutar_con_indice_y_entrada(dir, sub, UPDATE_INDEX_STDIN, i, entrada),
    };
    Ok(exigida(op, sub, r)?.stdout)
}

/// Escribe en el almacen de objetos los ficheros de `rutas` (una ruta por
/// linea, `n` en total) con un solo `hash-object --stdin-paths` y devuelve
/// su sha en el mismo orden. Error si no hay exactamente uno por fichero:
/// emparejar por orden daria a cada ruta el contenido de otra.
fn hashear(git: &Git, dir: &Path, op: &str, rutas: &str, n: usize) -> Paso<Vec<String>> {
    let salida = con_entrada(git, dir, op, Subcomando::HashObject, None, rutas.as_bytes())?;
    let shas: Vec<String> = String::from_utf8_lossy(&salida)
        .lines()
        .map(|l| l.trim().to_owned())
        .collect();
    if shas.len() == n {
        Ok(shas)
    } else {
        Err(Informe::nuevo(op, Resultado::Error).diagnostico(&txt::HASH_OBJECT_INCOMPLETO))
    }
}

/// Como se sella el commit de la rama de estado (F4-2).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Sello {
    /// Sin firma: el de todas las operaciones salvo `approve`.
    #[default]
    Normal,
    /// Firmado con la clave de quien lo lanza (`commit-tree -S`, con
    /// `gpg.format` y `user.signingkey` de su configuracion) y con un
    /// trailer `<clave>: <blob>` por cada `(clave, ruta)`, con el blob de
    /// esa ruta en el arbol del propio commit (`Plan-Blob`, F1-6 D3).
    Firmado {
        /// `(clave del trailer, ruta en la rama de estado)`.
        trailers: Vec<(&'static str, String)>,
    },
}

/// Construye un commit con el arbol de `base` mas `cambios` y `base` como
/// padre (sin padre si `base` es `None`: la raiz de la rama huerfana). No
/// mueve ninguna rama. `None` si el arbol queda igual que el de `base`.
pub(crate) fn construir_commit(
    git: &Git,
    dir: &Path,
    op: &str,
    base: Option<&str>,
    cambios: &[Cambio],
    mensaje: &str,
) -> Paso<Option<String>> {
    construir_commit_sellado(git, dir, op, base, cambios, mensaje, &Sello::Normal)
}

/// Como [`construir_commit`], con el `sello`. Con [`Sello::Firmado`], si el
/// arbol no cambia no se firma nada (`None` antes de `commit-tree`); si la
/// firma falla, `rechazada motivo=firma` (no hay commit que publicar).
pub(crate) fn construir_commit_sellado(
    git: &Git,
    dir: &Path,
    op: &str,
    base: Option<&str>,
    cambios: &[Cambio],
    mensaje: &str,
    sello: &Sello,
) -> Paso<Option<String>> {
    if let Some(malo) = cambios.iter().find(|c| !ruta_valida(&c.ruta)) {
        return Err(Informe::nuevo(op, Resultado::Error)
            .con("motivo", "ruta-invalida")
            .diagnostico(
                Mensaje::nuevo(&txt::RUTA_INVALIDA_EN_ESTADO).con("ruta", malo.ruta.as_str()),
            ));
    }
    let comun = dir_comun(git, dir, op)?;
    let tmp = Temporal::nueva(&comun, op)?;
    let indice = tmp.0.join("indice");
    if let Some(b) = base {
        con_indice(git, dir, op, Subcomando::ReadTree, &[b], &indice)?;
    }
    if !cambios.is_empty() {
        // Rutas y entradas van por stdin, no por argumentos: con muchos
        // ficheros la linea de ordenes pasaria del tope de Windows (unos
        // 32 K caracteres) y los procesos no crecen con el numero de cambios
        // (TASK-127).
        let mut rutas = String::new();
        for (i, c) in cambios.iter().enumerate() {
            let f = tmp.0.join(format!("blob-{i}"));
            std::fs::write(&f, c.contenido.as_bytes()).map_err(|e| {
                Informe::nuevo(op, Resultado::Error).diagnostico(error_de_fichero(&f, &e))
            })?;
            rutas.push_str(&texto(op, &f)?);
            rutas.push('\n');
        }
        let shas = hashear(git, dir, op, &rutas, cambios.len())?;
        let mut entradas = String::new();
        for (c, sha) in cambios.iter().zip(&shas) {
            entradas.push_str(&format!("100644 {sha}\t{}\0", c.ruta));
        }
        con_entrada(
            git,
            dir,
            op,
            Subcomando::UpdateIndex,
            Some(&indice),
            entradas.as_bytes(),
        )?;
    }
    let arbol = con_indice(git, dir, op, Subcomando::WriteTree, &[], &indice)?;
    let arbol = arbol.trim();
    if let Some(b) = base {
        let previo = exigir(
            git,
            dir,
            op,
            Subcomando::RevParse,
            &["--verify", "--end-of-options", &format!("{b}^{{tree}}")],
        )?;
        if previo.trim() == arbol {
            return Ok(None);
        }
    }
    let mut args = vec![arbol];
    if let Some(b) = base {
        args.extend(["-p", b]);
    }
    let Sello::Firmado { trailers } = sello else {
        args.extend(["-m", mensaje]);
        let sha = exigir(git, dir, op, Subcomando::CommitTree, &args)?;
        return Ok(Some(sha.trim().to_owned()));
    };
    let mut sellado = mensaje.to_owned();
    for (clave, ruta) in trailers {
        // El blob del arbol de este mismo commit, no el de la base.
        let blob = exigir(
            git,
            dir,
            op,
            Subcomando::RevParse,
            &["--verify", "--end-of-options", &format!("{arbol}:{ruta}")],
        )?;
        sellado.push_str(&format!("{clave}: {}\n", blob.trim()));
    }
    args.extend(["-m", &sellado]);
    let s = lanzado(op, Subcomando::CommitTree, git.ejecutar_firmado(dir, &args))?;
    if !s.exito() {
        return Err(Informe::nuevo(op, Resultado::Rechazada)
            .con("motivo", "firma")
            .diagnostico(Mensaje::nuevo(&txt::FIRMA_FALLO).con("error", s.primera_linea_error())));
    }
    Ok(Some(s.stdout.trim().to_owned()))
}

/// Si `origin` esta configurado (sin hablar con el remoto).
pub(crate) fn hay_remoto(git: &Git, dir: &Path, op: &str) -> Paso<bool> {
    Ok(url_origin(git, dir, op)?.is_some())
}

/// URL de `origin`, si esta configurado. `ls-remote --get-url` no habla con
/// el remoto: sin `origin`, devuelve el nombre tal cual.
pub(crate) fn url_origin(git: &Git, dir: &Path, op: &str) -> Paso<Option<String>> {
    let s = correr(
        git,
        dir,
        op,
        Subcomando::LsRemote,
        &["--get-url", super::REMOTO],
    )?;
    let url = s.stdout.trim();
    Ok((s.exito() && url != super::REMOTO && !url.is_empty()).then(|| url.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::{partir_batch, ruta_valida};

    #[test]
    fn salida_de_cat_file_batch() {
        let s = b"abc blob 5\nhola\n\nx:no missing\ndef tree 3\nabc\nghi blob 0\n\n";
        assert_eq!(
            partir_batch(s, 4),
            Some(vec![
                Some("hola\n".to_owned()),
                None,
                None,
                Some(String::new())
            ])
        );
        // Bytes que no casan con el tamano, de mas o de menos: no se entiende.
        assert_eq!(partir_batch(b"abc blob 9\nhola\n", 1), None);
        assert_eq!(partir_batch(b"abc blob 2\nhola\n", 1), None);
        assert_eq!(partir_batch(s, 3), None);
        assert_eq!(partir_batch(s, 5), None);
        assert_eq!(partir_batch(b"", 0), Some(vec![]));
        // Contenido multibyte: el tamano es en bytes.
        let t = "a ñ".as_bytes();
        let mut m = format!("abc blob {}\n", t.len()).into_bytes();
        m.extend_from_slice(t);
        m.push(b'\n');
        assert_eq!(partir_batch(&m, 1), Some(vec![Some("a ñ".to_owned())]));
    }

    #[test]
    fn rutas_de_la_rama_de_estado() {
        for ok in [
            "ROADMAP.md",
            "TASK-001/tarea.md",
            "a b/c.md",
            "auxiliar.md",
            "com10.md",
            "console.md",
        ] {
            assert!(ruta_valida(ok), "{ok}");
        }
        for malo in [
            "",
            "/abs.md",
            "a//b",
            "./a",
            "a/../b",
            "..",
            ".git/config",
            "a\\b",
            "C:/x",
            "a,b",
            "a\nb",
            "notas/aux.md",
            "CON",
            "x/com1.txt",
            "LPT9",
            "a?b.md",
            "a*b",
            "a<b",
            "a|b",
            "a\"b",
            "fin.",
            "fin ",
            "carpeta./x",
        ] {
            assert!(!ruta_valida(malo), "{malo}");
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests_stdin {
    use super::*;
    use crate::git::prueba::{aislar, entorno, sh};
    use crate::i18n::Idioma;

    /// Repo vacio de verdad en una carpeta temporal propia.
    fn repo(nombre: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("tc2-estado-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        aislar(&d).unwrap();
        sh(&d, &d, &["init", "-q"]).unwrap();
        d
    }

    /// TASK-127: `hashear` da un sha por fichero y, si `hash-object` no
    /// devuelve exactamente uno por fichero, `error` con
    /// `HASH_OBJECT_INCOMPLETO` en vez de emparejar mal rutas y contenidos.
    #[test]
    fn hashear_exige_un_sha_por_fichero() {
        let d = repo("hashear");
        let git = Git::nuevo();
        let f = d.join("blob");
        std::fs::write(&f, "hola\n").unwrap();
        let rutas = format!("{}\n", f.display());

        let shas = hashear(&git, &d, "prueba", &rutas, 1).expect("un fichero, un sha");
        assert_eq!(shas.len(), 1);
        assert_eq!(shas[0].len(), 40, "{shas:?}");

        for n in [0, 2] {
            let i = hashear(&git, &d, "prueba", &rutas, n).expect_err("cuenta distinta");
            assert_eq!(i.resultado, Resultado::Error);
            let incompleto = Informe::nuevo("prueba", Resultado::Error)
                .diagnostico(&txt::HASH_OBJECT_INCOMPLETO);
            assert_eq!(
                i.diagnostico_en(Idioma::Es),
                incompleto.diagnostico_en(Idioma::Es),
                "n={n}"
            );
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Git falso que delega en el real salvo lo que intercepte su guion:
    /// `cmd` en Windows, `sh` en el resto, que van antes de la llamada al
    /// git real. El subcomando es el quinto argumento (`-c color.ui=never -c
    /// core.quotepath=false <sub>`).
    fn git_falso(d: &Path, cmd: &str, sh: &str) -> Git {
        if cfg!(windows) {
            let f = d.join("git-falso.cmd");
            std::fs::write(&f, format!("@echo off\r\n{cmd}git %*\r\n")).unwrap();
            Git::con_binario(f)
        } else {
            let f = d.join("git-falso.sh");
            std::fs::write(&f, format!("#!/bin/sh\n{sh}exec git \"$@\"\n")).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o755)).unwrap();
            }
            Git::con_binario(f)
        }
    }

    /// Git falso que responde a `hash-object` con un unico sha sea cual sea
    /// el numero de rutas.
    fn git_que_pierde_shas(d: &Path) -> Git {
        let sha = "0123456789012345678901234567890123456789";
        git_falso(
            d,
            &format!("if \"%5\"==\"hash-object\" (\r\n  echo {sha}\r\n  exit /b 0\r\n)\r\n"),
            &format!("if [ \"$5\" = hash-object ]; then cat >/dev/null; echo {sha}; exit 0; fi\n"),
        )
    }

    /// A4-02: un `cat-file --batch` que falla con stderr que empieza por una
    /// linea en blanco da en el diagnostico la primera linea no vacia, como
    /// cualquier otra llamada a git.
    #[test]
    fn leer_blobs_diagnostica_la_primera_linea_no_vacia() {
        let d = repo("leer-blobs");
        let git = git_falso(
            &d,
            "if \"%5\"==\"cat-file\" (\r\n  >&2 echo.\r\n  >&2 echo fatal: x\r\n  exit /b 1\r\n)\r\n",
            "if [ \"$5\" = cat-file ]; then cat >/dev/null; printf '\\nfatal: x\\n' >&2; exit 1; fi\n",
        );
        let i = leer_blobs(&git, &d, "prueba", "HEAD", &["a.md"]).expect_err("cat-file falla");
        assert_eq!(i.resultado, Resultado::Error);
        assert!(i.linea().contains("git=cat-file"), "{}", i.linea());
        let diag = i.diagnostico_en(Idioma::Es).unwrap_or_default();
        assert!(diag.contains("fatal: x"), "{diag}");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// TASK-127: si `hash-object` devuelve menos shas que ficheros,
    /// `construir_commit` falla con `HASH_OBJECT_INCOMPLETO` y no construye
    /// ningun commit (emparejar por orden daria a cada ruta otro contenido).
    #[test]
    fn construir_commit_rechaza_shas_incompletos() {
        let d = repo("shas-incompletos");
        let git = git_que_pierde_shas(&d);
        let cambios = [Cambio::nuevo("a.md", "a\n"), Cambio::nuevo("b.md", "b\n")];
        let i = construir_commit(&git, &d, "prueba", None, &cambios, "m")
            .expect_err("dos ficheros y un solo sha");
        assert_eq!(i.resultado, Resultado::Error);
        let incompleto =
            Informe::nuevo("prueba", Resultado::Error).diagnostico(&txt::HASH_OBJECT_INCOMPLETO);
        assert_eq!(
            i.diagnostico_en(Idioma::Es),
            incompleto.diagnostico_en(Idioma::Es),
            "{}",
            i.linea()
        );
        assert!(
            !git.registro()
                .iter()
                .any(|l| l.starts_with("git commit-tree")),
            "{:?}",
            git.registro()
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    /// MEN-2 de TASK-037: con un cambio que modifica el fichero del
    /// trailer, `Plan-Blob` es el blob del arbol del propio commit (el
    /// nuevo), no el de la base. Firma real con una clave SSH desechable y
    /// la configuracion aislada.
    #[test]
    fn plan_blob_es_el_del_arbol_del_propio_commit() {
        let d = repo("plan-blob");
        let clave = d.join("clave");
        let k = std::process::Command::new("ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "", "-C", "prueba", "-f"])
            .arg(&clave)
            .output()
            .unwrap();
        assert!(k.status.success(), "{}", String::from_utf8_lossy(&k.stderr));
        let config = d.join("gitconfig");
        std::fs::write(
            &config,
            format!(
                "[user]\n\tname = Prueba\n\temail = prueba@example.com\n[gpg]\n\tformat = ssh\n[user]\n\tsigningkey = {}\n",
                clave.to_string_lossy().replace('\\', "/")
            ),
        )
        .unwrap();
        let git = Git::nuevo().con_entorno(entorno(&d));
        let base = construir_commit(
            &git,
            &d,
            "prueba",
            None,
            &[Cambio::nuevo("plan.md", "v1\n")],
            "base\n",
        )
        .unwrap()
        .unwrap();
        let sello = Sello::Firmado {
            trailers: vec![("Plan-Blob", "plan.md".to_owned())],
        };
        let sha = construir_commit_sellado(
            &git,
            &d,
            "prueba",
            Some(&base),
            &[Cambio::nuevo("plan.md", "v2\n")],
            "estado: approve\n\nOperacion: approve|x\n",
            &sello,
        )
        .unwrap()
        .unwrap();
        let blob = |r: &str| {
            exigir(&git, &d, "prueba", Subcomando::RevParse, &[r])
                .unwrap()
                .trim()
                .to_owned()
        };
        let nuevo = blob(&format!("{sha}:plan.md"));
        let viejo = blob(&format!("{base}:plan.md"));
        assert_ne!(nuevo, viejo);
        let cuerpo = exigir(&git, &d, "prueba", Subcomando::CatFile, &["-p", &sha]).unwrap();
        assert!(cuerpo.contains("gpgsig "), "{cuerpo}");
        assert!(
            cuerpo.ends_with(&format!("Operacion: approve|x\nPlan-Blob: {nuevo}\n")),
            "{cuerpo}"
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    /// TASK-127: si git falla con la entrada por stdin, `con_entrada` da
    /// `error` con el subcomando, no `Ok`; con datos buenos, un sha por ruta.
    #[test]
    fn con_entrada_da_error_si_git_falla() {
        let d = repo("con-entrada");
        let git = Git::nuevo();
        let entrada = format!("{}\n", d.join("no-existe").display());
        let r = con_entrada(
            &git,
            &d,
            "prueba",
            Subcomando::HashObject,
            None,
            entrada.as_bytes(),
        );
        let i = r.expect_err("hash-object de un fichero que no existe");
        assert_eq!(i.resultado, Resultado::Error);
        assert!(i.linea().contains("git=hash-object"), "{}", i.linea());

        let indice = d.join("indice");
        let r = con_entrada(
            &git,
            &d,
            "prueba",
            Subcomando::UpdateIndex,
            Some(&indice),
            b"basura sin forma\0",
        );
        let i = r.expect_err("update-index con una entrada mal formada");
        assert!(i.linea().contains("git=update-index"), "{}", i.linea());

        let f = d.join("blob");
        std::fs::write(&f, "hola\n").unwrap();
        let entrada = format!("{}\n", f.display());
        let r = con_entrada(
            &git,
            &d,
            "prueba",
            Subcomando::HashObject,
            None,
            entrada.as_bytes(),
        );
        let shas = String::from_utf8(r.expect("hash-object de un fichero que existe")).unwrap();
        assert_eq!(shas.trim().len(), 40, "{shas}");
        let _ = std::fs::remove_dir_all(&d);
    }
}
