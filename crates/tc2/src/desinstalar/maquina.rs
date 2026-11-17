//! Inventario y plan de `desinstalar --maquina`: el binario y lo que dejan
//! junto a el los instaladores y `actualizar --cli`, el bloque del perfil
//! del shell, el PATH de usuario de Windows, la cache del lanzador, la
//! status line y, solo como aviso, el plugin y los hooks.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use distribucion::{MARCA_INICIO_PERFIL, STATUS_LINE};

use super::texto::{hooks_con_tc2, quitar_bloque, quitar_del_path, quitar_miembro};
use super::{Accion, Contexto, Decision, Efecto, leer, no_se_lee, orden_borrar, t};
use crate::contrato::Informe;
use crate::git::misma_ruta;
use crate::instalacion::{Modo, Sistema, orden_del_gestor, ruta_old};

/// Perfiles del shell donde `install.sh` puede dejar el bloque.
const PERFILES: [&str; 4] = [".profile", ".bashrc", ".zshrc", ".bash_profile"];

/// Hijos de una carpeta (vacio si no existe).
fn hijos(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .map(|d| d.flatten().map(|e| e.path()).collect())
        .unwrap_or_default()
}

/// Un temporal que deja un instalador o `actualizar --cli` junto al binario:
/// `.tc2-<v>-<pid>.descarga` o `.tc2.nuevo.<pid>[.exe]`.
fn es_temporal(nombre: &str) -> bool {
    (nombre.starts_with(".tc2-") && nombre.ends_with(".descarga"))
        || nombre.starts_with(".tc2.nuevo.")
}

/// Expande `%LOCALAPPDATA%` y `%USERPROFILE%` como lo haria Windows al
/// comparar entradas del PATH (las unicas que puede usar un instalador).
fn expandir(c: &Contexto<'_>, e: &str) -> String {
    let mut s = e.to_owned();
    for (var, valor) in [
        ("%LOCALAPPDATA%", &c.entorno.local_app_data),
        ("%USERPROFILE%", &c.entorno.home),
    ] {
        if let Some(v) = valor {
            let mut out = String::new();
            let mut resto = s.as_str();
            while let Some(i) = resto.to_ascii_uppercase().find(var) {
                out.push_str(&resto[..i]);
                out.push_str(&t(v));
                resto = &resto[i + var.len()..];
            }
            out.push_str(resto);
            s = out;
        }
    }
    s
}

pub(super) fn plan(c: &Contexto<'_>, v: &mut Vec<Accion>) -> Result<(), Informe> {
    let e = c.entorno;
    let windows = e.sistema == Sistema::Windows;
    // Lo que se va a quitar entero: para saber si una carpeta queda vacia.
    let mut quitados: BTreeSet<PathBuf> = BTreeSet::new();
    let mut path = if windows {
        c.maquina
            .leer_path_usuario()
            .map_err(|er| no_se_lee(Path::new("HKCU\\Environment\\Path"), &er))?
    } else {
        None
    };
    // Indice de la accion de autoborrado, si la hay.
    let mut autoborrado: Option<usize> = None;
    let mut exe_cubierto = false;

    // Plugin y marketplace: de Claude Code, la persona (D3). Se lee una vez.
    let instalados = instalados(c)?;
    let plugins = plugins(instalados.as_ref());

    for dir in e.dirs_de_instalacion() {
        let bin = dir.join(e.sistema.binario());
        let es_exe = bin.is_file() && misma_ruta(&bin, &e.exe);
        exe_cubierto |= es_exe;
        // Solo es nuestro si es este `tc2`, si esta en la carpeta propia de
        // Windows o si dice ser `tc2` (`version --json`). Un enlace a otra
        // cosa o una herramienta ajena llamada `tc2` se quedan.
        let hay_bin = bin.is_file() && (es_exe || e.dir_propio(&dir) || es_tc2(&bin));
        if bin.is_file() && !hay_bin {
            v.push(
                Accion::nueva(t(&bin), "binario", "ajeno", Decision::Conserva).motivo("no-es-tc2"),
            );
        }
        let en_uso = hay_bin
            && windows
            && c.maquina
                .en_uso_por_otro(&bin, e.pid)
                .map_err(|er| no_se_lee(&bin, &er))?;
        let mut grupo: Vec<Accion> = Vec::new();
        let path_antes = path.clone();
        // Lo que este grupo quita: solo cuenta para las carpetas si se aplica.
        let mut del_grupo: BTreeSet<PathBuf> = BTreeSet::new();
        let mut temporales: Vec<PathBuf> = hijos(&dir)
            .into_iter()
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(es_temporal)
            })
            .collect();
        temporales.sort();
        for p in temporales {
            grupo.push(Accion::borra(t(&p), "temporal", Efecto::Fichero(p.clone())));
            del_grupo.insert(p);
        }
        let old = ruta_old(&bin);
        if old.is_file() {
            grupo.push(Accion::borra(
                t(&old),
                "binario-old",
                Efecto::Fichero(old.clone()),
            ));
            del_grupo.insert(old);
        }
        if hay_bin {
            if es_exe && windows {
                // El binario en marcha no se puede borrar en Windows: lo borra
                // un proceso auxiliar al salir (D4).
                autoborrado = Some(v.len() + grupo.len());
                grupo.push(
                    Accion::borra(
                        t(&bin),
                        "binario",
                        Efecto::Autoborrado {
                            ficheros: vec![bin.clone()],
                            dirs: Vec::new(),
                        },
                    )
                    .motivo("al-salir"),
                );
            } else {
                grupo.push(Accion::borra(
                    t(&bin),
                    "binario",
                    Efecto::Fichero(bin.clone()),
                ));
            }
            del_grupo.insert(bin.clone());
        }
        if e.dir_propio(&dir)
            && dir.is_dir()
            && hijos(&dir)
                .iter()
                .all(|h| quitados.contains(h) || del_grupo.contains(h))
        {
            if let (true, Some(i)) = (es_exe && windows, autoborrado) {
                if let Some(Accion {
                    efecto: Efecto::Autoborrado { dirs, .. },
                    ..
                }) = grupo.get_mut(i - v.len())
                {
                    dirs.push(dir.clone());
                }
                grupo.push(
                    Accion::nueva(t(&dir), "carpeta", "taskcodetwo", Decision::Borra)
                        .motivo("al-salir"),
                );
            } else {
                grupo.push(Accion::borra(
                    t(&dir),
                    "carpeta",
                    Efecto::CarpetaSiVacia(dir.clone()),
                ));
            }
            del_grupo.insert(dir.clone());
        }
        // La entrada del PATH de usuario (inversa de `Add-PathUsuario`).
        if let Some(valor) = path.clone()
            && let Some(nuevo) = quitar_del_path(&valor, &t(&dir), |x| expandir(c, x))
        {
            grupo.push(Accion::borra(
                format!("HKCU\\Environment\\Path:{}", t(&dir)),
                "path-usuario",
                Efecto::PathUsuario {
                    viejo: valor,
                    nuevo: nuevo.clone(),
                },
            ));
            path = Some(nuevo);
        }
        if en_uso {
            // Otro proceso ejecuta este binario: nada de su grupo se toca
            // (ni el PATH), para no dejarlo a medias (C8).
            for a in &mut grupo {
                a.decision = Decision::Pendiente;
                a.motivo = Some("en-uso".to_owned());
                a.efecto = Efecto::Ninguno;
            }
            if autoborrado.is_some_and(|i| i >= v.len()) {
                autoborrado = None;
            }
            path = path_antes;
        } else {
            quitados.extend(del_grupo);
        }
        v.extend(grupo);
    }

    // El `tc2` que se ejecuta, si no es de un instalador.
    if !exe_cubierto {
        match e.modo() {
            Modo::Plugin => {
                let orden = plugins.first().map_or_else(
                    || "claude plugin uninstall 'taskcodetwo'".to_owned(),
                    Clone::clone,
                );
                v.push(
                    Accion::nueva(t(&e.exe), "binario", "plugin", Decision::Persona).orden(orden),
                );
            }
            Modo::Gestor(g) => {
                v.push(
                    Accion::nueva(t(&e.exe), "binario", "gestor", Decision::Persona)
                        .motivo(g)
                        .orden(orden_del_gestor(g)),
                );
            }
            Modo::Instalador => {
                if e.exe.is_file() {
                    v.push(
                        Accion::nueva(t(&e.exe), "binario", "taskcodetwo", Decision::Persona)
                            .motivo("fuera-de-los-instaladores")
                            .orden(orden_borrar(e.sistema, &e.exe)),
                    );
                }
            }
        }
    }

    // Bloque de marcas de los perfiles del shell (Unix).
    if !windows && let Some(home) = &e.home {
        for nombre in PERFILES {
            let p = home.join(nombre);
            let Some(bytes) = leer(&p)? else { continue };
            let Ok(texto) = String::from_utf8(bytes.clone()) else {
                if bytes
                    .windows(MARCA_INICIO_PERFIL.len())
                    .any(|w| w == MARCA_INICIO_PERFIL.as_bytes())
                {
                    v.push(
                        Accion::nueva(t(&p), "perfil", "taskcodetwo", Decision::Persona)
                            .motivo("no-utf8"),
                    );
                }
                continue;
            };
            match quitar_bloque(&texto) {
                Ok(None) => {}
                Ok(Some(nuevo)) => v.push(Accion::borra(
                    t(&p),
                    "perfil",
                    Efecto::Reescribir {
                        ruta: p.clone(),
                        antes: bytes,
                        despues: nuevo.into_bytes(),
                    },
                )),
                Err(m) => v.push(
                    Accion::nueva(t(&p), "perfil", "taskcodetwo", Decision::Persona)
                        .motivo(m.motivo()),
                ),
            }
        }
    }

    // Cache del lanzador (`taskcodetwo/cli/`, F1-14) y la carpeta raiz.
    for raiz in e.raices_de_cache() {
        let cli = raiz.join("cli");
        if cli.is_dir() {
            if e.modo() == Modo::Plugin && e.exe.starts_with(&cli) {
                // La version que se ejecuta es del plugin: la quita la persona
                // tras desinstalar el plugin; las demas, ya.
                let mut versiones = hijos(&cli);
                versiones.sort();
                for ver in versiones {
                    if e.exe.starts_with(&ver) {
                        v.push(
                            Accion::nueva(t(&ver), "cache-cli", "taskcodetwo", Decision::Persona)
                                .motivo("en-uso-por-el-plugin")
                                .orden(orden_borrar(e.sistema, &ver)),
                        );
                    } else {
                        v.push(Accion::borra(
                            t(&ver),
                            "cache-cli",
                            Efecto::Carpeta(ver.clone()),
                        ));
                    }
                }
            } else {
                v.push(Accion::borra(
                    t(&cli),
                    "cache-cli",
                    Efecto::Carpeta(cli.clone()),
                ));
                quitados.insert(cli.clone());
            }
        }
        if raiz.is_dir() && hijos(&raiz).iter().all(|h| quitados.contains(h)) {
            match autoborrado.and_then(|i| v.get_mut(i)) {
                Some(Accion {
                    efecto: Efecto::Autoborrado { dirs, .. },
                    ..
                }) if e.dir_por_defecto().is_some_and(|d| d.starts_with(&raiz)) => {
                    dirs.push(raiz.clone());
                    v.push(
                        Accion::nueva(t(&raiz), "carpeta", "taskcodetwo", Decision::Borra)
                            .motivo("al-salir"),
                    );
                }
                _ => v.push(Accion::borra(
                    t(&raiz),
                    "carpeta",
                    Efecto::CarpetaSiVacia(raiz.clone()),
                )),
            }
        }
    }

    claude(c, v, plugins, instalados.as_ref())
}

/// Tiempo maximo de `<binario> version --json` al comprobar si es `tc2`.
const ESPERA_VERSION: std::time::Duration = std::time::Duration::from_secs(10);

/// Si `bin` es un `tc2`: no es un enlace simbolico y responde a
/// `version --json` con su version y su protocolo (ADR F1-14, punto 5).
fn es_tc2(bin: &Path) -> bool {
    if std::fs::symlink_metadata(bin).is_ok_and(|m| m.file_type().is_symlink()) {
        return false;
    }
    let mut cmd = std::process::Command::new(bin);
    cmd.args(["version", "--json"]);
    let Ok(Some(o)) = crate::actualizar_cli::ejecutar_con_limite(cmd, ESPERA_VERSION) else {
        return false;
    };
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap_or_default();
    o.status.success()
        && v.get("version").is_some_and(serde_json::Value::is_string)
        && v.get("protocolo").is_some_and(serde_json::Value::is_u64)
}

/// `plugins/installed_plugins.json` de Claude Code (solo lectura), si esta
/// y es JSON. La lectura es la del inventario (F3-9).
fn instalados(c: &Contexto<'_>) -> Result<Option<serde_json::Value>, Informe> {
    let Some(dir) = c.entorno.dir_claude() else {
        return Ok(None);
    };
    match crate::inventario::leer_instalados(&dir) {
        Ok(v) => Ok(v.and_then(Result::ok)),
        Err(e) => Err(no_se_lee(&crate::inventario::ruta_instalados(&dir), &e)),
    }
}

/// Un nombre o scope que se puede poner en una orden: letras, digitos, `.`,
/// `_`, `-` y `@`.
fn seguro(t: &str) -> bool {
    !t.is_empty()
        && t.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '@'))
}

/// Ordenes `claude plugin uninstall` del plugin de la 2.0, una por scope en
/// que este instalado. Una clave o un scope con otros caracteres no se pone
/// en una orden; la clave va entre comillas simples.
fn plugins(instalados: Option<&serde_json::Value>) -> Vec<String> {
    let mut ordenes = Vec::new();
    let Some(m) = instalados
        .and_then(|v| v.get("plugins"))
        .and_then(serde_json::Value::as_object)
    else {
        return ordenes;
    };
    for (clave, entradas) in m {
        if clave.split('@').next() != Some(crate::huella::NOMBRE) || !seguro(clave) {
            continue;
        }
        let scopes: Vec<&str> = entradas
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.get("scope").and_then(serde_json::Value::as_str))
                    .filter(|s| seguro(s))
                    .collect()
            })
            .unwrap_or_default();
        if scopes.is_empty() {
            ordenes.push(format!("claude plugin uninstall '{clave}'"));
        }
        for s in scopes {
            ordenes.push(format!("claude plugin uninstall '{clave}' --scope {s}"));
        }
    }
    ordenes
}

/// Lo de Claude Code: la status line (solo la de la 2.0, por su `command`
/// exacto), los hooks que nombran `tc2` (aviso; nunca se editan) y el plugin
/// (la persona) y el de la 0.16 (se conserva, F9-10).
fn claude(
    c: &Contexto<'_>,
    v: &mut Vec<Accion>,
    plugins: Vec<String>,
    instalados: Option<&serde_json::Value>,
) -> Result<(), Informe> {
    let Some(dir) = c.entorno.dir_claude() else {
        return Ok(());
    };
    let settings = dir.join("settings.json");
    if let Some(bytes) = leer(&settings)? {
        let texto = String::from_utf8(bytes.clone()).ok();
        let valor = texto
            .as_deref()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());
        match (texto, valor) {
            (Some(texto), Some(valor)) => {
                if let Some(sl) = valor.get("statusLine") {
                    let cmd = sl
                        .get("command")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default();
                    let ruta = format!("{}#statusLine", t(&settings));
                    if cmd.trim() == STATUS_LINE {
                        match quitar_miembro(&texto, "statusLine") {
                            Some(nuevo) => v.push(Accion::borra(
                                ruta,
                                "status-line",
                                Efecto::Reescribir {
                                    ruta: settings.clone(),
                                    antes: bytes,
                                    despues: nuevo.into_bytes(),
                                },
                            )),
                            None => v.push(
                                Accion::nueva(
                                    ruta,
                                    "status-line",
                                    "taskcodetwo",
                                    Decision::Persona,
                                )
                                .motivo("no-se-reescribe"),
                            ),
                        }
                    } else {
                        let dueno = if cmd.trim_start().starts_with("taskctl") {
                            "taskcode-016"
                        } else {
                            "ajeno"
                        };
                        v.push(Accion::nueva(
                            ruta,
                            "status-line",
                            dueno,
                            Decision::Conserva,
                        ));
                    }
                }
                for _ in hooks_con_tc2(&valor) {
                    v.push(
                        Accion::nueva(
                            format!("{}#hooks", t(&settings)),
                            "hook",
                            "ajeno",
                            Decision::Conserva,
                        )
                        .motivo("nombra-tc2"),
                    );
                }
            }
            _ => v.push(
                Accion::nueva(t(&settings), "settings", "claude-code", Decision::Conserva)
                    .motivo("json-invalido"),
            ),
        }
    }
    for orden in plugins {
        v.push(
            Accion::nueva(
                crate::huella::NOMBRE,
                "plugin",
                "claude-code",
                Decision::Persona,
            )
            .orden(orden),
        );
    }
    // El plugin de la 0.16 se nombra y no se toca (D6, F9-10).
    if let Some(m) = instalados
        .and_then(|v| v.get("plugins"))
        .and_then(serde_json::Value::as_object)
    {
        for clave in m
            .keys()
            .filter(|k| k.split('@').next() == Some("taskcode-plugin"))
        {
            v.push(
                Accion::nueva(clave.as_str(), "plugin", "taskcode-016", Decision::Conserva)
                    .motivo("desinstalacion-f9-10"),
            );
        }
    }
    Ok(())
}
