//! Operacion `commit-estado` de la tabla de F1-4: commits de la rama de
//! estado construidos sin tocar ningun worktree ni el indice del repo
//! (F2-3). El arbol se arma en un indice temporal sobre el commit base y el
//! commit sale de `commit-tree`; publicarlo es un `push <sha>:<rama>` sin
//! force. Asi, tras un push rechazado no hay nada local que descartar: no
//! hace falta `reset` (prohibido por F1-4 D3).

use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::rama::error_de_fichero;
use super::{Git, Paso, Subcomando, correr, exigir, git_fallo};
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
        let s = git
            .ejecutar_con_entrada(dir, Subcomando::CatFile, &["--batch"], entrada.as_bytes())
            .map_err(|e| {
                Informe::nuevo(op, Resultado::Error)
                    .con("git", Subcomando::CatFile.nombre())
                    .diagnostico(e.mensaje())
            })?;
        if s.codigo != 0 {
            return Err(Informe::nuevo(op, Resultado::Error)
                .con("git", Subcomando::CatFile.nombre())
                .diagnostico(
                    Mensaje::nuevo(&txt::GIT_FALLO)
                        .con("subcomando", Subcomando::CatFile.nombre())
                        .con("error", s.stderr.lines().next().unwrap_or_default()),
                ));
        }
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
    let s = git
        .ejecutar_con_indice(dir, sub, args, indice)
        .map_err(|e| {
            Informe::nuevo(op, Resultado::Error)
                .con("git", sub.nombre())
                .diagnostico(e.mensaje())
        })?;
    if s.exito() {
        Ok(s.stdout)
    } else {
        Err(Informe::nuevo(op, Resultado::Error)
            .con("git", sub.nombre())
            .diagnostico(git_fallo(sub, &s)))
    }
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
        let mut ficheros = Vec::with_capacity(cambios.len());
        for (i, c) in cambios.iter().enumerate() {
            let f = tmp.0.join(format!("blob-{i}"));
            std::fs::write(&f, c.contenido.as_bytes()).map_err(|e| {
                Informe::nuevo(op, Resultado::Error).diagnostico(error_de_fichero(&f, &e))
            })?;
            ficheros.push(texto(op, &f)?);
        }
        let mut args = vec!["-w", "--no-filters", "--"];
        args.extend(ficheros.iter().map(String::as_str));
        let shas = exigir(git, dir, op, Subcomando::HashObject, &args)?;
        let shas: Vec<&str> = shas.lines().map(str::trim).collect();
        if shas.len() != cambios.len() {
            return Err(
                Informe::nuevo(op, Resultado::Error).diagnostico(&txt::HASH_OBJECT_INCOMPLETO)
            );
        }
        let entradas: Vec<String> = cambios
            .iter()
            .zip(&shas)
            .map(|(c, sha)| format!("100644,{sha},{}", c.ruta))
            .collect();
        let mut args = vec!["--add"];
        for e in &entradas {
            args.push("--cacheinfo");
            args.push(e);
        }
        con_indice(git, dir, op, Subcomando::UpdateIndex, &args, &indice)?;
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
    args.extend(["-m", mensaje]);
    let sha = exigir(git, dir, op, Subcomando::CommitTree, &args)?;
    Ok(Some(sha.trim().to_owned()))
}

/// Si `origin` esta configurado. `ls-remote --get-url` no habla con el
/// remoto: sin `origin`, devuelve el nombre tal cual.
pub(crate) fn hay_remoto(git: &Git, dir: &Path, op: &str) -> Paso<bool> {
    let s = correr(
        git,
        dir,
        op,
        Subcomando::LsRemote,
        &["--get-url", super::REMOTO],
    )?;
    Ok(s.exito() && s.stdout.trim() != super::REMOTO && !s.stdout.trim().is_empty())
}

/// URL de `origin`, si esta configurado (sin hablar con el remoto).
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
