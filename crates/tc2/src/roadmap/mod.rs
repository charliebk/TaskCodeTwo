//! `tc2 roadmap` (F3-4): la vista para todos, en `ROADMAP.md` de la rama de
//! estado (ADR F1-1, decision 3).
//!
//! - [`de_estado`] genera el `ROADMAP.md` con el mismo calculo que `next`
//!   ([`crate::situacion`]) y la zona manual del que ya hay, byte a byte; si
//!   esa zona esta rota, devuelve el anterior tal cual (D3: no se bloquea la
//!   operacion; `next` y `--comparar` lo avisan). Es el generador que recibe
//!   cada transaccion de estado ([`crate::estado::Contexto::nuevo`]): se
//!   regenera, nunca se mergea (ADR F1-2, D2.4).
//! - La orden `roadmap [--comparar | --escribir] [--json]` (D2): sin
//!   opciones imprime el ROADMAP que toca sobre el estado recien traido
//!   (F3-5b: trae la rama de estado y solo escribe
//!   `origin/taskcodetwo/tareas` y la marca de sincronizacion); `--comparar`
//!   dice si lo publicado esta al dia (`ok`) o no (`rechazada
//!   motivo=desfasado|roadmap-roto`), sin escribir; `--escribir` lo publica
//!   como una transaccion mas ([`Regenerar`]). La hora de la ultima
//!   sincronizacion va en `remoto=` y `sincronizado=` y, en el texto, a
//!   stderr: el stdout sigue siendo el ROADMAP byte a byte.
//! - La zona manual se edita en `tareas/ROADMAP.md` y se publica con
//!   `--escribir` (D4): si la zona de la carpeta montada cambio, se valida, se
//!   prepara (`git add`) y es la que se publica, byte a byte, salvo que la
//!   publicada haya cambiado desde la de partida (`zona-divergente`); tras
//!   publicar, la carpeta se adelanta a lo publicado. Si no se publica, el
//!   indice vuelve a como estaba. Los cambios fuera de la zona no se publican
//!   y se avisan.

pub mod generar;
pub mod mermaid;
pub mod zona;

use std::path::{Path, PathBuf};

use crate::Salida;
use crate::contrato::{Informe, Resultado, Valor, corto};
use crate::estado::{self, CARPETA, Cambio, Catalogo, Estado, Fallo, Frescura, Operacion, RAMA};
use crate::git::{Git, Subcomando, correr, exigir, localizar_principal, worktrees};
use crate::i18n::catalogo::cli as txt_cli;
use crate::i18n::catalogo::roadmap as txt;
use crate::i18n::{Idioma, Mensaje};
use crate::plataforma::Personas;
use crate::situacion::{self, ROADMAP, Situacion};

/// Nombre de la orden.
pub const NOMBRE: &str = "roadmap";

/// Lista de personas de la base para convertir los `asignado_a` heredados
/// (D6); sin lista, ninguna: se muestran tal cual.
pub fn personas(g: &Git, dir: &Path) -> Personas {
    personas_y_aviso(g, dir).0
}

/// Como [`personas`], y si hay que avisar: la lista existe pero no se pudo
/// leer (no avisa si simplemente no hay lista).
pub fn personas_y_aviso(g: &Git, dir: &Path) -> (Personas, bool) {
    match crate::identidad::personas_de_base(g, dir) {
        Ok(p) => (p, false),
        Err(i) => (
            Personas::default(),
            i.valor("motivo") != Some("sin-personas"),
        ),
    }
}

/// Aviso: `tareas/ROADMAP.md` tiene cambios solo fuera de la zona manual.
const FUERA_DE_ZONA: &str = "fuera-de-zona";
/// Aviso: la lista de personas existe y no se pudo leer.
const PERSONAS_ILEGIBLES: &str = "personas-ilegibles";

/// `avisos=` con los avisos que haya (ninguno: sin la clave).
fn con_avisos(i: Informe, avisos: &[&str]) -> Informe {
    if avisos.is_empty() {
        return i;
    }
    i.con_lista("avisos", avisos.iter().map(|a| (*a).to_owned()).collect())
}

/// Los avisos de un informe, en una linea por aviso para stderr.
fn texto_de_avisos(i: &Informe, idioma: Idioma) -> String {
    let avisos = i.claves.iter().find_map(|(k, v)| match v {
        Valor::Lista(l) if k == "avisos" => Some(l.as_slice()),
        _ => None,
    });
    let mut s = String::new();
    for a in avisos.unwrap_or_default() {
        let m = match a.as_str() {
            FUERA_DE_ZONA => Mensaje::nuevo(&txt::AVISO_FUERA_DE_ZONA).con("donde", donde()),
            PERSONAS_ILEGIBLES => Mensaje::nuevo(&txt::AVISO_PERSONAS_ILEGIBLES),
            _ => continue,
        };
        s.push_str(&m.texto(idioma));
        s.push('\n');
    }
    s
}

/// `tareas/ROADMAP.md`, para los mensajes.
fn donde() -> String {
    format!("{CARPETA}/{ROADMAP}")
}

/// El generador que recibe cada transaccion de estado.
pub fn generador(personas: &Personas) -> impl Fn(&dyn Estado) -> Result<String, Fallo> + '_ {
    move |e: &dyn Estado| de_estado(e, personas)
}

/// El `ROADMAP.md` de `estado`: generado alrededor de su zona manual (o la
/// de por defecto), o el que hay tal cual si su zona esta rota (D3).
pub fn de_estado(estado: &dyn Estado, personas: &Personas) -> Result<String, Fallo> {
    let s = situacion::calcular(estado)?;
    Ok(de_situacion(&s, personas).unwrap_or_else(|| s.roadmap.clone().unwrap_or_default()))
}

/// El ROADMAP que toca para `s`, o `None` si su zona manual esta rota.
fn de_situacion(s: &Situacion, personas: &Personas) -> Option<String> {
    let zona = match s.roadmap.as_deref().map(zona::extraer) {
        None | Some(Ok(None)) => zona::POR_DEFECTO,
        Some(Ok(Some(z))) => z,
        Some(Err(_)) => return None,
    };
    Some(generar::generar(s, zona, personas))
}

fn roto(donde: &str, motivo: &str) -> Fallo {
    Fallo::rechazada(
        "roadmap-roto",
        Mensaje::nuevo(&txt::ZONA_ROTA)
            .con("donde", donde)
            .con("motivo", motivo),
    )
}

/// La zona manual de un ROADMAP para comparar ediciones: la zona; sin zona,
/// ninguna; rota, el texto entero (cualquier cambio cuenta).
fn zona_de(texto: &str) -> Option<String> {
    match zona::extraer(texto) {
        Ok(z) => z.map(str::to_owned),
        Err(_) => Some(texto.to_owned()),
    }
}

/// Lo que la persona edito en `tareas/ROADMAP.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Edicion {
    /// El fichero editado (normalizado como lo guardaria git), con su zona
    /// valida.
    texto: String,
    /// La zona de la que partio: la del `HEAD` de la carpeta montada.
    base: Option<String>,
}

/// `roadmap --escribir` como operacion de la rama de estado: no cambia nada
/// salvo, si la persona edito la zona manual en `tareas/`, el `ROADMAP.md`
/// con esa zona; el generador de la transaccion hace el resto. Exige red
/// (D2b de F1-2): sin red no queda pendiente.
///
/// Si la zona publicada ya no es la de la que partio la edicion (otra
/// persona publico la suya entre medias) y tampoco es la editada, se
/// rechaza con `zona-divergente` en vez de pisarla (IMP-1 de la revision 1).
#[derive(Debug, Clone, Default)]
pub struct Regenerar {
    editado: Option<Edicion>,
}

impl Regenerar {
    /// Sin edicion de la persona.
    pub fn nueva() -> Self {
        Self::default()
    }

    /// Con el `ROADMAP.md` editado por la persona (su zona ya validada) y la
    /// zona de la que partio la edicion.
    pub fn con_editado(texto: String, base: Option<String>) -> Self {
        Self {
            editado: Some(Edicion { texto, base }),
        }
    }
}

impl Operacion for Regenerar {
    fn nombre(&self) -> &str {
        NOMBRE
    }
    fn anotacion(&self) -> String {
        format!("{NOMBRE}|escribir")
    }
    fn exige_red(&self) -> bool {
        true
    }
    fn aplicar(&self, e: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
        let actual = e.leer(ROADMAP)?;
        match &self.editado {
            Some(ed) => {
                let publicada = actual.as_deref().and_then(zona_de);
                if publicada != ed.base && publicada != zona_de(&ed.texto) {
                    return Err(Fallo::rechazada(
                        "zona-divergente",
                        Mensaje::nuevo(&txt::ZONA_DIVERGENTE).con("donde", donde()),
                    ));
                }
                if actual.as_deref() == Some(ed.texto.as_str()) {
                    Ok(Vec::new())
                } else {
                    Ok(vec![Cambio::nuevo(ROADMAP, ed.texto.clone())])
                }
            }
            None => match actual.as_deref().map(zona::extraer) {
                Some(Err(r)) => Err(roto(ROADMAP, r.motivo())),
                _ => Ok(Vec::new()),
            },
        }
    }
}

/// Reconstruye [`Regenerar`] desde su anotacion (nunca queda pendiente:
/// exige red).
pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
    (a == format!("{NOMBRE}|escribir")).then(|| Box::new(Regenerar::nueva()) as Box<dyn Operacion>)
}

/// Que hace la orden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modo {
    /// Imprimir el ROADMAP que toca (solo lee).
    Imprimir,
    /// Decir si lo publicado esta al dia (solo lee).
    Comparar,
    /// Publicarlo.
    Escribir,
}

/// `roadmap [--comparar | --escribir]` (sin `--json`, que quita `lib.rs`)
/// en la carpeta actual; `catalogo` re-ejecuta las pendientes.
pub fn orden(args: &[String], json: bool, idioma: Idioma, catalogo: Catalogo<'_>) -> Salida {
    let modo = match args {
        [] => Modo::Imprimir,
        [a] if a == "--comparar" => Modo::Comparar,
        [a] if a == "--escribir" => Modo::Escribir,
        _ => {
            return Informe::nuevo(NOMBRE, Resultado::Uso)
                .con("motivo", "argumentos")
                .diagnostico(&txt::USO_ROADMAP)
                .salida_en(json, idioma);
        }
    };
    let dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return Informe::nuevo(NOMBRE, Resultado::Error)
                .diagnostico(Mensaje::nuevo(&txt_cli::CARPETA_ACTUAL).con("error", e.to_string()))
                .salida_en(json, idioma);
        }
    };
    let g = Git::nuevo();
    let ahora = crate::identidad::ahora();
    // La frescura de las dos que leen (F3-5b): en el texto de `roadmap`, la
    // hora va a stderr (el stdout es el ROADMAP byte a byte, C9).
    let mut frescura = None;
    let i = match modo {
        Modo::Imprimir => match imprimir(&g, &dir, ahora) {
            Ok((texto, i, f)) if !json => {
                let mut stderr = texto_de_avisos(&i, idioma);
                if let Some(m) = f.linea(ahora) {
                    stderr.push_str(&m.texto(idioma));
                    stderr.push('\n');
                }
                return Salida {
                    codigo: 0,
                    stdout: texto,
                    stderr,
                };
            }
            Ok((texto, i, f)) => {
                frescura = Some(f);
                i.con("texto", texto)
            }
            Err(i) => i,
        },
        Modo::Comparar => {
            let (i, f) = comparar(&g, &dir, ahora);
            frescura = f;
            i
        }
        Modo::Escribir => escribir_en(&g, &dir, catalogo),
    };
    let mut s = i.salida_en(json, idioma);
    // Los avisos van a stderr con codigo 0 (como el de titulos parecidos);
    // si falla, stderr es el diagnostico.
    if s.codigo == 0 {
        s.stderr.push_str(&texto_de_avisos(&i, idioma));
        if let Some(m) = frescura.and_then(|f| f.aviso(ahora)) {
            s.stderr.push_str(&m.texto(idioma));
            s.stderr.push('\n');
        }
    }
    s
}

fn renombrado(mut i: Informe) -> Informe {
    NOMBRE.clone_into(&mut i.comando);
    i
}

fn de_fallo(f: Fallo) -> Informe {
    match f {
        Fallo::Rechazada {
            motivo,
            diagnostico,
        } => Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", motivo)
            .diagnostico(diagnostico),
        Fallo::Error(d) => Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(d),
    }
}

/// La situacion del estado recien traido (F3-5b), su commit y su frescura.
fn situacion_en(g: &Git, dir: &Path, ahora: u64) -> Result<(Situacion, String, Frescura), Informe> {
    let (vista, frescura) = estado::leer_fresco(g, dir, ahora).map_err(renombrado)?;
    let commit = vista.commit().map(corto).unwrap_or_default();
    let s = situacion::calcular(&vista).map_err(de_fallo)?;
    Ok((s, commit, frescura))
}

/// Personas de la base y los avisos que tocan.
fn personas_con_avisos(g: &Git, dir: &Path) -> (Personas, Vec<&'static str>) {
    let (p, ilegibles) = personas_y_aviso(g, dir);
    (
        p,
        if ilegibles {
            vec![PERSONAS_ILEGIBLES]
        } else {
            Vec::new()
        },
    )
}

/// El ROADMAP que toca sobre el estado recien traido, con el informe `ok`
/// (su `commit=`, sus `avisos=`, `remoto=` y `sincronizado=`). Con la zona
/// rota, `rechazada motivo=roadmap-roto`.
pub fn imprimir_en(g: &Git, dir: &Path) -> Result<(String, Informe), Informe> {
    imprimir(g, dir, crate::identidad::ahora()).map(|(t, i, _)| (t, i))
}

/// [`imprimir_en`] con su frescura. El texto es solo el ROADMAP: la hora
/// nunca entra en el (si no, `--comparar` saldria siempre desfasado).
fn imprimir(g: &Git, dir: &Path, ahora: u64) -> Result<(String, Informe, Frescura), Informe> {
    let (s, commit, frescura) = situacion_en(g, dir, ahora)?;
    let (personas, avisos) = personas_con_avisos(g, dir);
    match de_situacion(&s, &personas) {
        Some(t) => {
            let i = con_avisos(
                Informe::nuevo(NOMBRE, Resultado::Ok).con("commit", commit),
                &avisos,
            );
            Ok((t, frescura.anotar(i), frescura))
        }
        None => Err(zona_rota(&s, &commit)),
    }
}

fn zona_rota(s: &Situacion, commit: &str) -> Informe {
    let motivo = s.roadmap_roto().map_or("", zona::Rota::motivo);
    de_fallo(roto(ROADMAP, motivo))
        .con("roto", motivo)
        .con("commit", commit)
}

/// `--comparar`: `ok` si el `ROADMAP.md` publicado es el que toca;
/// `rechazada motivo=desfasado` si no; `rechazada motivo=roadmap-roto` si
/// su zona esta rota (D3). Trae antes la rama de estado (F3-5b) y no escribe
/// nada mas.
pub fn comparar_en(g: &Git, dir: &Path) -> Informe {
    comparar(g, dir, crate::identidad::ahora()).0
}

/// [`comparar_en`] con su frescura (`None` si no se llego a leer).
fn comparar(g: &Git, dir: &Path, ahora: u64) -> (Informe, Option<Frescura>) {
    let (s, commit, frescura) = match situacion_en(g, dir, ahora) {
        Ok(x) => x,
        Err(i) => return (i, None),
    };
    let (personas, avisos) = personas_con_avisos(g, dir);
    let i = match de_situacion(&s, &personas) {
        None => zona_rota(&s, &commit),
        Some(t) if s.roadmap.as_deref() == Some(t.as_str()) => con_avisos(
            Informe::nuevo(NOMBRE, Resultado::Ok).con("commit", commit),
            &avisos,
        ),
        Some(_) => Informe::nuevo(NOMBRE, Resultado::Rechazada)
            .con("motivo", "desfasado")
            .con("commit", commit)
            .diagnostico(&txt::DESFASADO),
    };
    (frescura.anotar(i), Some(frescura))
}

/// `tareas/` montada (la carpeta del worktree de la rama de estado).
fn carpeta_montada(g: &Git, dir: &Path) -> Result<Option<PathBuf>, Informe> {
    let principal = localizar_principal(g, dir).map_err(renombrado)?;
    Ok(worktrees(g, &principal, NOMBRE)?
        .into_iter()
        .find(|w| w.rama.as_deref() == Some(RAMA) && w.ruta.exists())
        .map(|w| w.ruta))
}

/// Que hay en `tareas/ROADMAP.md` respecto a su `HEAD`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Lectura {
    /// Sin cambios.
    Limpia,
    /// Cambios solo fuera de la zona manual: no se publican y se avisa.
    FueraDeZona,
    /// La zona manual editada.
    Zona(Edicion),
}

/// Lee `tareas/ROADMAP.md` sin tocar el indice: el texto como lo guardaria
/// git (`hash-object` aplica los filtros de la ruta), con la zona validada
/// antes de nada (MEN-1 de la revision 1) y comparada con la del `HEAD` de la
/// carpeta, de la que partio la edicion.
fn editado_en(g: &Git, carpeta: &Path) -> Result<Lectura, Informe> {
    let s = exigir(
        g,
        carpeta,
        NOMBRE,
        Subcomando::Status,
        &["--porcelain", "--", ROADMAP],
    )?;
    if s.trim().is_empty() {
        return Ok(Lectura::Limpia);
    }
    let sha = exigir(
        g,
        carpeta,
        NOMBRE,
        Subcomando::HashObject,
        &["-w", "--", ROADMAP],
    )?;
    let texto = exigir(
        g,
        carpeta,
        NOMBRE,
        Subcomando::CatFile,
        &["blob", sha.trim()],
    )?;
    let mia = match zona::extraer(&texto) {
        Ok(Some(z)) => z.to_owned(),
        Ok(None) => return Err(de_fallo(roto(&donde(), "sin-zona")).con("roto", "sin-zona")),
        Err(r) => return Err(de_fallo(roto(&donde(), r.motivo())).con("roto", r.motivo())),
    };
    let base = crate::git::estado::leer_blob(g, carpeta, NOMBRE, "HEAD", ROADMAP)
        .ok()
        .and_then(|t| zona_de(&t));
    if base.as_deref() == Some(mia.as_str()) {
        return Ok(Lectura::FueraDeZona);
    }
    Ok(Lectura::Zona(Edicion { texto, base }))
}

/// Lo que hay que deshacer si `--escribir` no publica: el contenido que el
/// indice tenia para `ROADMAP.md` antes de preparar la edicion.
struct Preparado {
    previo: String,
}

/// Prepara la edicion (sin remoto, el adelanto de la carpeta va dentro de la
/// transaccion y la necesita preparada) recordando lo que habia en el indice.
fn preparar(g: &Git, carpeta: &Path) -> Result<Preparado, Informe> {
    let previo = crate::git::estado::leer_blob(g, carpeta, NOMBRE, "", ROADMAP)?;
    exigir(g, carpeta, NOMBRE, Subcomando::Add, &["--", ROADMAP])?;
    Ok(Preparado { previo })
}

/// Deja el indice como estaba (MEN-2 de la revision 1) con la unica
/// escritura de indice que permite la capa Git (`git add`): se pone el
/// contenido previo, se prepara y se devuelve el de la persona, que se
/// tenia en memoria. Si no se puede devolver, se dice.
fn restaurar_indice(g: &Git, carpeta: &Path, p: &Preparado) -> Result<(), Informe> {
    let ruta = carpeta.join(ROADMAP);
    let error = |e: std::io::Error| {
        Informe::nuevo(NOMBRE, Resultado::Error).diagnostico(
            Mensaje::nuevo(&txt::NO_SE_RESTAURO)
                .con("donde", donde())
                .con("error", e.to_string()),
        )
    };
    let mio = std::fs::read(&ruta).map_err(error)?;
    std::fs::write(&ruta, p.previo.as_bytes()).map_err(error)?;
    let preparado = exigir(g, carpeta, NOMBRE, Subcomando::Add, &["--", ROADMAP]);
    std::fs::write(&ruta, &mio).map_err(error)?;
    preparado.map(|_| ())
}

/// `--escribir`: publica el ROADMAP que toca como una transaccion (D1). La
/// zona editada en `tareas/` se prepara solo sin remoto; si no se publica, el
/// indice vuelve a como estaba y la edicion sigue en la carpeta.
pub fn escribir_en(g: &Git, dir: &Path, catalogo: Catalogo<'_>) -> Informe {
    let carpeta = match carpeta_montada(g, dir) {
        Ok(c) => c,
        Err(i) => return renombrado(i),
    };
    let lectura = match carpeta.as_deref().map(|c| editado_en(g, c)).transpose() {
        Ok(l) => l.unwrap_or(Lectura::Limpia),
        Err(i) => return renombrado(i),
    };
    let remoto = match crate::git::estado::hay_remoto(g, dir, NOMBRE) {
        Ok(r) => r,
        Err(i) => return renombrado(i),
    };
    let (personas, mut avisos) = personas_con_avisos(g, dir);
    let mut preparado = None;
    let mut editada = false;
    let op = match (lectura, carpeta.as_deref()) {
        (Lectura::Zona(ed), Some(c)) => {
            if !remoto {
                match preparar(g, c) {
                    Ok(p) => preparado = Some(p),
                    Err(i) => return renombrado(i),
                }
            }
            editada = true;
            Regenerar::con_editado(ed.texto, ed.base)
        }
        (Lectura::FueraDeZona, _) => {
            avisos.push(FUERA_DE_ZONA);
            Regenerar::nueva()
        }
        _ => Regenerar::nueva(),
    };
    let generar = generador(&personas);
    let c = estado::Contexto::nuevo(g, dir, catalogo, Some(&generar));
    let mut i = renombrado(estado::transaccion(&c, &op));
    if let (Some(p), Some(carpeta)) = (&preparado, carpeta.as_deref())
        && i.resultado != Resultado::Ok
        && let Err(e) = restaurar_indice(g, carpeta, p)
    {
        return renombrado(e);
    }
    if editada
        && i.resultado == Resultado::Ok
        && i.valor("tareas") == Some("con-cambios")
        && let (Some(carpeta), Some(commit)) =
            (carpeta.as_deref(), i.valor("commit").map(str::to_owned))
        && adelantar_editada(g, carpeta, &commit)
        && let Some(v) = i.claves.iter_mut().find(|(k, _)| k == "tareas")
    {
        v.1 = Valor::Texto("al-dia".to_owned());
    }
    con_avisos(i, &avisos)
}

/// Tras publicar la zona editada, deja `tareas/ROADMAP.md` como lo
/// publicado (que lleva esa zona byte a byte), lo prepara y adelanta la
/// carpeta: sin esto, el cambio ya publicado la dejaria atascada. Si algo
/// falla, la carpeta queda como estaba (con `tareas=con-cambios`).
fn adelantar_editada(g: &Git, carpeta: &Path, commit: &str) -> bool {
    let Ok(publicado) = crate::git::estado::leer_blob(g, carpeta, NOMBRE, commit, ROADMAP) else {
        return false;
    };
    std::fs::write(carpeta.join(ROADMAP), publicado.as_bytes()).is_ok()
        && exigir(g, carpeta, NOMBRE, Subcomando::Add, &["--", ROADMAP]).is_ok()
        && correr(
            g,
            carpeta,
            NOMBRE,
            Subcomando::Merge,
            &["-q", "--ff-only", commit],
        )
        .is_ok_and(|s| s.exito())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use crate::tarea::{Asignado, Cuerpo, Tarea};

    /// Estado en memoria.
    pub(crate) struct EnMemoria(pub(crate) BTreeMap<String, String>);

    impl Estado for EnMemoria {
        fn rutas(&self) -> Vec<String> {
            self.0.keys().cloned().collect()
        }
        fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo> {
            Ok(self.0.get(ruta).cloned())
        }
    }

    fn v2(
        id: &str,
        titulo: &str,
        sprint: u32,
        estado: &str,
        deps: &[&str],
        asignado: Option<&str>,
    ) -> String {
        let mut t = Tarea::nueva(id, titulo, "2026-01-01");
        t.sprint = sprint;
        t.estado = estado.to_owned();
        if estado == "descartada" {
            t.motivo = Some("ya no hace falta".to_owned());
        }
        t.dependencias = deps.iter().map(|d| (*d).to_owned()).collect();
        if let Some(login) = asignado {
            t.asignado_a = Asignado::persona(7, login);
        }
        t.escribir(&Cuerpo::leer(""))
    }

    /// Formato 1 (0.16), con `asignado_a` de texto libre.
    fn v1(id: &str, titulo: &str, sprint: u32, estado: &str, asignado: &str) -> String {
        format!(
            "---\nid: {id}\ntitulo: \"{titulo}\"\ntipo: feature\nsprint: {sprint}\netiquetas: []\n\
             complejidad: null\nmodelo_sugerido: sonnet\nestado: {estado}\nplan_aprobado: false\n\
             rama: feature/x\nasignado_a: {asignado}\nagente_revisor: general-purpose\n\
             skills_recomendados: []\nregla_seleccion_skill: null\nultimo_commit_revisado: null\n\
             revision_codex: false\ntokens_diseno: null\ntokens_implementacion: null\n\
             tokens_revision: null\ncreado: 2026-01-01\nactualizado: 2026-01-01\n\
             dependencias: []\n---\n"
        )
    }

    const ZONA: &str = "<!-- manual:inicio (a mano) -->\r\n## Hitos\r\n\r\n### H2 — Dos\r\nEl motivo.\r\n### H1 — Uno\r\n### H3 — Tres\r\n<!-- manual:fin -->\r\n";

    /// Un estado que pasa por cada seccion.
    pub(crate) fn estado_de_referencia() -> EnMemoria {
        let mut m = BTreeMap::new();
        for (id, md) in [
            (
                "TASK-001",
                v2("TASK-001", "Base", 1, "terminada", &[], None),
            ),
            (
                "TASK-002",
                v2(
                    "TASK-002",
                    "Motor",
                    1,
                    "en-curso",
                    &["TASK-001"],
                    Some("ana"),
                ),
            ),
            (
                "TASK-003",
                v1("TASK-003", "Revisar", 1, "en-revision", "ana@x.com"),
            ),
            (
                "TASK-004",
                v2(
                    "TASK-004",
                    "Usa el motor",
                    2,
                    "planificada",
                    &["TASK-002"],
                    None,
                ),
            ),
            (
                "TASK-005",
                v2("TASK-005", "Lista", 2, "planificada", &["TASK-001"], None),
            ),
            (
                "TASK-006",
                v2("TASK-006", "Ya no", 2, "descartada", &[], None),
            ),
            (
                "TASK-007",
                v2(
                    "TASK-007",
                    "Tras la descartada",
                    2,
                    "planificada",
                    &["TASK-006", "TASK-099"],
                    None,
                ),
            ),
            (
                "TASK-008",
                v1(
                    "TASK-008",
                    "Con a | b <!-- y \\\"comillas\\\" #1",
                    2,
                    "en-diseno",
                    "alguien de fuera",
                ),
            ),
            (
                "TASK-009",
                v2("TASK-009", "Cerrada", 3, "terminada", &[], None),
            ),
            ("TASK-010", "---\nid: roto\n".to_owned()),
            (
                "TASK-011",
                v2("TASK-011", "Ciclo a", 2, "planificada", &["TASK-012"], None),
            ),
            (
                "TASK-012",
                v2(
                    "TASK-012",
                    "Ciclo b",
                    2,
                    "planificada",
                    &["TASK-011", "TASK-010"],
                    None,
                ),
            ),
        ] {
            m.insert(format!("{id}/tarea.md"), md);
        }
        m.insert(
            situacion::ROADMAP.to_owned(),
            format!("# Viejo\n\n{ZONA}\n## Lo generado antes\n"),
        );
        EnMemoria(m)
    }

    pub(crate) fn personas() -> Personas {
        Personas::leer("personas:\n  - {id: 9, login: \"ana-x\", correos: [\"ana@x.com\"]}\n")
            .unwrap_or_default()
    }

    const REFERENCIA: &str = include_str!("../../tests/fixtures/roadmap/referencia.md");

    /// C5: la salida de referencia cubre cada seccion (con quien, revision,
    /// hito con dependencias y Mermaid, terminadas y descartadas); C2: la
    /// zona manual, con CRLF, byte a byte. Para rehacer la referencia:
    /// `TC2_REESCRIBIR=1 cargo test -p tc2 --lib referencia`.
    #[test]
    fn salida_de_referencia() {
        let r = de_estado(&estado_de_referencia(), &personas());
        assert!(r.is_ok(), "{r:?}");
        let Ok(r) = r else { return };
        if std::env::var_os("TC2_REESCRIBIR").is_some() {
            let ruta = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/roadmap/referencia.md"
            );
            assert!(std::fs::write(ruta, &r).is_ok());
            return;
        }
        assert_eq!(r, REFERENCIA);
        assert_eq!(zona::extraer(&r), Ok(Some(ZONA)));
        for seccion in [
            "\n## Estado del trabajo\n",
            "\n## En curso\n",
            "\n## En revisión\n",
            "\n## H2 — Dos\n",
            "\n## H1 — Uno\n",
            "\n## Terminadas\n",
            "\n## Descartadas\n",
            "```mermaid\n",
            "| @ana |",
            "| @ana-x |",
            "<details>",
        ] {
            assert!(r.contains(seccion), "falta {seccion:?}");
        }
        assert!(!r.contains("\n## H3"), "un hito cerrado no lleva apartado");
        // Mismos datos, mismos bytes (lo que da `sin-cambios`).
        let mut otra = estado_de_referencia();
        otra.0.insert(situacion::ROADMAP.to_owned(), r.clone());
        assert_eq!(de_estado(&otra, &personas()).ok(), Some(r));
    }

    /// TASK-034 (C3 y C9): la descartada sale con su motivo y fuera de los
    /// hitos; una planificada en pausa sale de las listas y el ROADMAP la
    /// marca, y una en curso en pausa sigue en marcha marcada. Reanudada,
    /// los mismos bytes que sin pausa.
    #[test]
    fn descartadas_con_motivo_y_pausas_marcadas() {
        let referencia = de_estado(&estado_de_referencia(), &personas()).unwrap_or_default();
        assert!(referencia.contains("\n- TASK-006 — Ya no (H2) — motivo: ya no hace falta\n"));
        assert!(!referencia.contains("| TASK-006 |"), "{referencia}");
        let fila =
            |fase: &str| format!("| 2026-11-17T10:00:00Z | {fase} | manual | agente | 0.1 |\n");
        let con_filas = |fases: &[&str]| {
            let mut e = estado_de_referencia();
            for id in ["TASK-005", "TASK-002"] {
                let ruta = format!("{id}/tarea.md");
                let mut t = e.0.get(&ruta).cloned().unwrap_or_default();
                t.push_str("## Transiciones\n\n| fecha | fase | modo | decidido_por | version |\n|---|---|---|---|---|\n");
                for f in fases {
                    t.push_str(&fila(f));
                }
                e.0.insert(ruta, t);
            }
            e
        };
        let e = con_filas(&["start", "pausa"]);
        let s = situacion::calcular(&e).unwrap_or_default();
        assert!(s.listas.is_empty(), "{:?}", s.listas);
        assert_eq!(s.en_marcha.first().map(String::as_str), Some("TASK-008"));
        let r = de_estado(&e, &personas()).unwrap_or_default();
        assert!(r.contains("- **Listas para empezar:** ninguna\n"), "{r}");
        assert!(r.contains("TASK-002 (en curso · en pausa)"), "{r}");
        assert!(
            r.contains("| 4 | TASK-005 | Lista | TASK-001 | en pausa | — |\n"),
            "{r}"
        );
        assert!(
            r.contains("| 5 | TASK-002 | Motor | TASK-001 | en curso · en pausa | @ana |\n"),
            "{r}"
        );
        assert!(r.contains("T005[\"TASK-005 · Lista\"]:::espera"), "{r}");
        // La linea de «En marcha» se sigue leyendo como la de `next`.
        assert_eq!(
            generar::ids_de_la_linea(&r, "En marcha"),
            Some(s.en_marcha.clone())
        );
        // Reanudada: como sin pausa.
        let e = con_filas(&["pausa", "reanudar"]);
        assert_eq!(de_estado(&e, &personas()).ok(), Some(referencia));
    }

    /// D3: con la zona rota se devuelve el anterior tal cual.
    #[test]
    fn zona_rota_deja_el_anterior() {
        let mut e = estado_de_referencia();
        let roto = "# Roadmap\n<!-- manual:inicio -->\n### H1 — Uno\n".to_owned();
        e.0.insert(situacion::ROADMAP.to_owned(), roto.clone());
        assert_eq!(de_estado(&e, &Personas::default()).ok(), Some(roto));
    }

    /// Sin ROADMAP.md (o sin zona), la zona de por defecto.
    #[test]
    fn sin_roadmap_la_zona_por_defecto() {
        let mut e = estado_de_referencia();
        e.0.remove(situacion::ROADMAP);
        let r = de_estado(&e, &Personas::default()).unwrap_or_default();
        assert_eq!(zona::extraer(&r), Ok(Some(zona::POR_DEFECTO)));
        assert!(r.starts_with(&format!("# Roadmap de tareas\n\n{}\n\n", generar::MARCA)));
    }
}
