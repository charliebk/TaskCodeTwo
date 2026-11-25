//! Escritura de un fichero de una vez (A8-04, que absorbe A7-02): la de los
//! ficheros locales de la carpeta comun (marca de sincronizacion, cache de
//! identidad y progreso), la del almacen de confianza y la de los ficheros
//! de la persona que reescribe `tc2 desinstalar`, en un solo sitio.
//!
//! Un temporal `<nombre>.tc2-tmp-<pid>` al lado del destino y un `rename`
//! encima: quien lee no ve nunca un fichero a medias, y un corte no deja el
//! destino vacio.

use std::io;
use std::path::Path;
use std::time::Duration;

/// Esperas antes de repetir el `rename` de [`atomica`]: en Windows falla
/// mientras un lector tiene el destino abierto sin `FILE_SHARE_DELETE`
/// (Python, Node o un antivirus; L2 de F1-17, punto 7). Unos 300 ms en total.
const ESPERAS_RENAME: [Duration; 5] = [
    Duration::from_millis(10),
    Duration::from_millis(20),
    Duration::from_millis(40),
    Duration::from_millis(80),
    Duration::from_millis(150),
];

/// Escribe `ruta` de una vez. Si `ruta` es un enlace simbolico se escribe su
/// destino (como `cat >` de `install.sh`); si el destino ya existe, el nuevo
/// conserva sus permisos. Crea la carpeta si falta, escribe el temporal
/// `<nombre>.tc2-tmp-<pid>` al lado y lo renombra encima. Si el `rename`
/// falla, se repite tras cada espera de [`ESPERAS_RENAME`]. Si falla la
/// escritura, los permisos o el ultimo `rename`, se borra el temporal y se
/// devuelve el error.
pub(crate) fn atomica(ruta: &Path, contenido: &[u8]) -> io::Result<()> {
    atomica_con(ruta, contenido, &ESPERAS_RENAME)
}

fn atomica_con(ruta: &Path, contenido: &[u8], esperas: &[Duration]) -> io::Result<()> {
    let destino = match std::fs::symlink_metadata(ruta) {
        Ok(m) if m.file_type().is_symlink() => std::fs::canonicalize(ruta)?,
        _ => ruta.to_path_buf(),
    };
    let permisos = std::fs::metadata(&destino).ok().map(|m| m.permissions());
    if let Some(padre) = destino.parent() {
        std::fs::create_dir_all(padre)?;
    }
    let mut nombre = destino.file_name().unwrap_or_default().to_os_string();
    nombre.push(format!(".tc2-tmp-{}", std::process::id()));
    let tmp = destino.with_file_name(nombre);
    let r = std::fs::write(&tmp, contenido)
        .and_then(|()| match permisos {
            Some(p) => std::fs::set_permissions(&tmp, p),
            None => Ok(()),
        })
        .and_then(|()| renombrar(&tmp, &destino, esperas));
    if r.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    r
}

/// `rename` de `tmp` a `destino`, repetido tras cada espera mientras falle.
fn renombrar(tmp: &Path, destino: &Path, esperas: &[Duration]) -> io::Result<()> {
    let mut r = std::fs::rename(tmp, destino);
    for e in esperas {
        if r.is_ok() {
            break;
        }
        std::thread::sleep(*e);
        r = std::fs::rename(tmp, destino);
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn carpeta(nombre: &str) -> PathBuf {
        let d = std::env::temp_dir()
            .join("tc2-tests")
            .join(format!("escritura-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    /// Crea la carpeta, sustituye el contenido y no deja temporales sueltos.
    #[test]
    fn de_una_vez_y_sin_temporales() -> io::Result<()> {
        let d = carpeta("escribir");
        let ruta = d.join("sub").join("f.json");
        atomica(&ruta, b"uno")?;
        atomica(&ruta, b"dos")?;
        assert_eq!(std::fs::read(&ruta)?, b"dos");
        let n = std::fs::read_dir(d.join("sub"))?.count();
        assert_eq!(n, 1);
        let _ = std::fs::remove_dir_all(&d);
        Ok(())
    }

    /// L2 de F1-17 (punto 7): en Windows, con un lector abierto sin
    /// `FILE_SHARE_DELETE` (como Python o Node), el `rename` falla; con el
    /// reintento entra en cuanto el lector cierra. Sin reintento, falla.
    #[cfg(windows)]
    #[test]
    fn reintenta_con_un_lector_abierto() -> io::Result<()> {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_SHARE_READ: u32 = 1;
        let d = carpeta("lector");
        let ruta = d.join("f.json");
        atomica(&ruta, b"viejo")?;
        let abrir = || {
            std::fs::OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ)
                .open(&ruta)
        };
        // Sin reintento, con el lector abierto: falla y no deja temporal.
        let lector = abrir()?;
        assert!(atomica_con(&ruta, b"nuevo", &[]).is_err());
        assert_eq!(std::fs::read_dir(&d)?.count(), 1);
        // Con reintento: el lector cierra a los 30 ms y la escritura entra.
        let hilo = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(30));
            drop(lector);
        });
        atomica(&ruta, b"nuevo")?;
        let _ = hilo.join();
        assert_eq!(std::fs::read(&ruta)?, b"nuevo");
        let _ = std::fs::remove_dir_all(&d);
        Ok(())
    }

    /// Un `rename` que no puede entrar (el destino es una carpeta no vacia)
    /// da el error, deja la carpeta como estaba y no deja el temporal.
    #[test]
    fn rename_que_falla_borra_el_temporal() -> io::Result<()> {
        let d = carpeta("rename");
        let ruta = d.join("destino");
        std::fs::create_dir_all(&ruta)?;
        std::fs::write(ruta.join("dentro"), b"x")?;
        assert!(atomica_con(&ruta, b"nuevo", &[]).is_err());
        assert!(ruta.is_dir());
        assert_eq!(std::fs::read(ruta.join("dentro"))?, b"x");
        let nombres: Vec<_> = std::fs::read_dir(&d)?
            .filter_map(|e| e.ok().map(|e| e.file_name()))
            .collect();
        assert_eq!(nombres, vec![std::ffi::OsString::from("destino")]);
        let _ = std::fs::remove_dir_all(&d);
        Ok(())
    }

    /// Por un enlace simbolico se escribe su destino, que conserva sus
    /// permisos (`0o600`); el enlace sigue siendo enlace.
    #[cfg(unix)]
    #[test]
    fn sigue_el_enlace_y_conserva_los_permisos() -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let d = carpeta("enlace");
        std::fs::create_dir_all(&d)?;
        let real = d.join("real");
        std::fs::write(&real, b"viejo")?;
        std::fs::set_permissions(&real, std::fs::Permissions::from_mode(0o600))?;
        let enlace = d.join("enlace");
        std::os::unix::fs::symlink(&real, &enlace)?;
        atomica(&enlace, b"nuevo")?;
        assert!(std::fs::symlink_metadata(&enlace)?.file_type().is_symlink());
        assert_eq!(std::fs::read(&real)?, b"nuevo");
        let modo = std::fs::metadata(&real)?.permissions().mode() & 0o777;
        assert_eq!(modo, 0o600);
        let _ = std::fs::remove_dir_all(&d);
        Ok(())
    }
}
