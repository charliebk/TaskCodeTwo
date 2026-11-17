//! Funciones puras de `desinstalar`: quitar el bloque de marcas de un
//! perfil, quitar un miembro de un JSON sin tocar el resto de bytes, buscar
//! `tc2` en los hooks y quitar una entrada del PATH de usuario de Windows.

use distribucion::{LINEA_PERFIL, MARCA_FIN_PERFIL, MARCA_INICIO_PERFIL};

/// Linea sin su final (`\n` o `\r\n`).
fn sin_final(l: &str) -> &str {
    l.strip_suffix('\n')
        .map_or(l, |x| x.strip_suffix('\r').unwrap_or(x))
}

/// Por que un perfil con marcas no se toca.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarcasRaras {
    /// No hay exactamente un inicio y despues un fin.
    Incompletas,
    /// Entre las marcas no esta solo la linea que escribe `install.sh`.
    Tocadas,
}

impl MarcasRaras {
    /// Valor de `motivo=`.
    pub fn motivo(self) -> &'static str {
        match self {
            Self::Incompletas => "marcas-incompletas",
            Self::Tocadas => "bloque-modificado",
        }
    }
}

/// Inversa de `escribir_bloque` de `install.sh`: quita el bloque entre las
/// marcas y la linea en blanco que el instalador escribio delante. El resto
/// queda identico byte a byte. `Ok(None)` si no hay marcas; `Err` si las
/// marcas no son un bloque entero de tres lineas (no se toca nada).
pub fn quitar_bloque(texto: &str) -> Result<Option<String>, MarcasRaras> {
    let lineas: Vec<&str> = texto.split_inclusive('\n').collect();
    let inicios: Vec<usize> = (0..lineas.len())
        .filter(|&i| sin_final(lineas[i]) == MARCA_INICIO_PERFIL)
        .collect();
    let fines: Vec<usize> = (0..lineas.len())
        .filter(|&i| sin_final(lineas[i]) == MARCA_FIN_PERFIL)
        .collect();
    match (inicios.as_slice(), fines.as_slice()) {
        ([], []) => Ok(None),
        ([i], [f]) if i < f => {
            let (i, f) = (*i, *f);
            if f != i + 2 || !sin_final(lineas[i + 1]).starts_with(LINEA_PERFIL) {
                return Err(MarcasRaras::Tocadas);
            }
            let desde = if i > 0 && sin_final(lineas[i - 1]).is_empty() {
                i - 1
            } else {
                i
            };
            let mut s = String::with_capacity(texto.len());
            for l in lineas.get(..desde).unwrap_or_default() {
                s.push_str(l);
            }
            for l in lineas.get(f + 1..).unwrap_or_default() {
                s.push_str(l);
            }
            Ok(Some(s))
        }
        _ => Err(MarcasRaras::Incompletas),
    }
}

/// Un miembro del objeto de primer nivel: clave y donde empieza y acaba.
#[derive(Debug)]
struct Miembro {
    clave: String,
    inicio: usize,
    fin: usize,
}

/// Recorre el objeto de primer nivel de `texto` (JSON ya validado) y da sus
/// miembros, y el indice de la `{` de apertura.
fn miembros(texto: &str) -> Option<(usize, Vec<Miembro>)> {
    let b = texto.as_bytes();
    let mut i = 0;
    let saltar = |i: &mut usize| {
        while *i < b.len() && b[*i].is_ascii_whitespace() {
            *i += 1;
        }
    };
    // Fin de una cadena que empieza en `i` (en la comilla).
    let fin_cadena = |mut i: usize| -> Option<usize> {
        i += 1;
        while i < b.len() {
            match b[i] {
                b'\\' => i += 2,
                b'"' => return Some(i + 1),
                _ => i += 1,
            }
        }
        None
    };
    saltar(&mut i);
    if b.get(i) != Some(&b'{') {
        return None;
    }
    let apertura = i;
    i += 1;
    let mut v = Vec::new();
    loop {
        saltar(&mut i);
        match b.get(i)? {
            b'}' => return Some((apertura, v)),
            b',' => {
                i += 1;
                continue;
            }
            b'"' => {}
            _ => return None,
        }
        let inicio = i;
        let fc = fin_cadena(i)?;
        let clave: String = serde_json::from_str(texto.get(i..fc)?).ok()?;
        i = fc;
        saltar(&mut i);
        if b.get(i) != Some(&b':') {
            return None;
        }
        i += 1;
        saltar(&mut i);
        // El valor: hasta la `,` o la `}` del primer nivel.
        let mut prof = 0usize;
        while i < b.len() {
            match b[i] {
                b'"' => {
                    i = fin_cadena(i)?;
                    continue;
                }
                b'{' | b'[' => prof += 1,
                b'}' | b']' if prof > 0 => prof -= 1,
                b',' | b'}' if prof == 0 => break,
                _ => {}
            }
            i += 1;
        }
        let mut fin = i;
        while fin > inicio && b[fin - 1].is_ascii_whitespace() {
            fin -= 1;
        }
        v.push(Miembro { clave, inicio, fin });
    }
}

/// Quita el miembro `clave` del objeto de primer nivel de `texto` sin tocar
/// ningun otro byte: con la coma y el espacio que lo separan del siguiente
/// (o del anterior, si es el ultimo). `None` si no esta, si el JSON no es un
/// objeto o si el resultado no es el mismo JSON sin esa clave.
pub fn quitar_miembro(texto: &str, clave: &str) -> Option<String> {
    let antes: serde_json::Value = serde_json::from_str(texto).ok()?;
    let (apertura, ms) = miembros(texto)?;
    let pos = ms.iter().position(|m| m.clave == clave)?;
    let m = &ms[pos];
    let (desde, hasta) = match (pos.checked_sub(1).and_then(|p| ms.get(p)), ms.get(pos + 1)) {
        // Hay uno detras: desde la clave hasta el inicio del siguiente.
        (_, Some(sig)) => (m.inicio, sig.inicio),
        // Es el ultimo con uno delante: desde el final del anterior.
        (Some(ant), None) => (ant.fin, m.fin),
        // Es el unico.
        (None, None) => (apertura + 1, m.fin),
    };
    let mut s = String::with_capacity(texto.len());
    s.push_str(texto.get(..desde)?);
    s.push_str(texto.get(hasta..)?);
    let despues: serde_json::Value = serde_json::from_str(&s).ok()?;
    let mut esperado = antes;
    esperado.as_object_mut()?.remove(clave);
    (despues == esperado).then_some(s)
}

/// Si un `command` de un hook nombra `tc2` como palabra (`tc2`, `tc2.exe` o
/// una ruta que acaba en ellos).
pub fn nombra_tc2(command: &str) -> bool {
    command
        .split(|c: char| c.is_whitespace() || "\"'`;&|()/\\".contains(c))
        .any(|p| p.eq_ignore_ascii_case("tc2") || p.eq_ignore_ascii_case("tc2.exe"))
}

/// Los `command` de los hooks de un `settings.json` que nombran `tc2`.
pub fn hooks_con_tc2(v: &serde_json::Value) -> Vec<String> {
    fn recorrer(v: &serde_json::Value, out: &mut Vec<String>) {
        match v {
            serde_json::Value::Object(m) => {
                for (k, x) in m {
                    match x {
                        serde_json::Value::String(c) if k == "command" && nombra_tc2(c) => {
                            out.push(c.clone());
                        }
                        _ => recorrer(x, out),
                    }
                }
            }
            serde_json::Value::Array(a) => a.iter().for_each(|x| recorrer(x, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    if let Some(h) = v.get("hooks") {
        recorrer(h, &mut out);
    }
    out
}

/// Entrada del PATH normalizada para comparar como `Add-PathUsuario`: sin
/// `\` al final, con `\` como separador y sin distinguir mayusculas.
fn normal(e: &str) -> String {
    e.replace('/', "\\").trim_end_matches('\\').to_lowercase()
}

/// Inversa exacta de `Add-PathUsuario` de `install.ps1`: quita del valor
/// crudo del PATH de usuario las entradas que, expandidas con `expandir`,
/// son `dir`. Las demas entradas (y su orden, y las vacias) quedan igual.
/// `None` si no habia ninguna.
pub fn quitar_del_path(
    valor: &str,
    dir: &str,
    expandir: impl Fn(&str) -> String,
) -> Option<String> {
    let objetivo = normal(dir);
    let partes: Vec<&str> = valor.split(';').collect();
    let quedan: Vec<&str> = partes
        .iter()
        .copied()
        .filter(|e| e.is_empty() || normal(&expandir(e)) != objetivo)
        .collect();
    (quedan.len() != partes.len()).then(|| quedan.join(";"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOQUE: &str = "\n# >>> tc2 (TaskCodeTwo) >>>\ncase \":$PATH:\" in *\":/h/.local/bin:\"*) ;; *) export PATH=\"/h/.local/bin:$PATH\" ;; esac\n# <<< tc2 (TaskCodeTwo) <<<\n";

    #[test]
    fn quitar_bloque_deja_el_resto_byte_a_byte() {
        for original in ["", "alias ll='ls -l'\n", "# mio\r\n\r\nexport A=1\r\n"] {
            let con = format!("{original}{BLOQUE}");
            assert_eq!(quitar_bloque(&con), Ok(Some(original.to_owned())));
        }
        // Con algo detras del bloque, tambien.
        let con = format!("a\n{BLOQUE}b\n");
        assert_eq!(quitar_bloque(&con), Ok(Some("a\nb\n".to_owned())));
        assert_eq!(quitar_bloque("nada\n"), Ok(None));
        // Marcas tocadas a mano: no se toca.
        let sin_fin = "# >>> tc2 (TaskCodeTwo) >>>\nexport PATH=\"/v:$PATH\"\n";
        assert_eq!(quitar_bloque(sin_fin), Err(MarcasRaras::Incompletas));
        let doble = format!("{BLOQUE}{BLOQUE}");
        assert_eq!(quitar_bloque(&doble), Err(MarcasRaras::Incompletas));
        let otro = "# >>> tc2 (TaskCodeTwo) >>>\nrm -rf ~\n# <<< tc2 (TaskCodeTwo) <<<\n";
        assert_eq!(quitar_bloque(otro), Err(MarcasRaras::Tocadas));
    }

    #[test]
    fn quitar_miembro_sin_tocar_el_resto() {
        let t = "{\n  \"a\": 1,\n  \"statusLine\": {\"type\": \"command\", \"command\": \"x, }\"},\n  \"b\": [1, {\"c\": 2}]\n}\n";
        assert_eq!(
            quitar_miembro(t, "statusLine").as_deref(),
            Some("{\n  \"a\": 1,\n  \"b\": [1, {\"c\": 2}]\n}\n")
        );
        let ultimo = "{\r\n  \"a\": 1,\r\n  \"statusLine\": {}\r\n}";
        assert_eq!(
            quitar_miembro(ultimo, "statusLine").as_deref(),
            Some("{\r\n  \"a\": 1\r\n}")
        );
        assert_eq!(
            quitar_miembro("{\"statusLine\": 1}", "statusLine").as_deref(),
            Some("{}")
        );
        assert_eq!(quitar_miembro("{\"a\": 1}", "statusLine"), None);
        assert_eq!(quitar_miembro("[1]", "statusLine"), None);
        assert_eq!(quitar_miembro("{\"statusLine\": ", "statusLine"), None);
        // Una clave con escape no se confunde.
        let esc = "{\"status\\u004cine\": 1, \"x\": \"statusLine\"}";
        assert_eq!(
            quitar_miembro(esc, "statusLine").as_deref(),
            Some("{\"x\": \"statusLine\"}")
        );
    }

    #[test]
    fn hooks_que_nombran_tc2() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"tc2 estado"},{"type":"command","command":"/x/tc2.exe guardar"},{"type":"command","command":"tc23 x"},{"type":"command","command":"taskctl tc2x"}]}]},"command":"tc2"}"#,
        )
        .unwrap_or_default();
        assert_eq!(hooks_con_tc2(&v), vec!["tc2 estado", "/x/tc2.exe guardar"]);
    }

    #[test]
    fn quitar_del_path_inversa_de_add_path_usuario() {
        let exp = |e: &str| e.replace("%LOCALAPPDATA%", r"C:\U\L");
        let dir = r"C:\U\L\taskcodetwo\bin";
        assert_eq!(
            quitar_del_path(r"C:\a;%LOCALAPPDATA%\taskcodetwo\bin\;;D:\b", dir, exp).as_deref(),
            Some(r"C:\a;;D:\b")
        );
        assert_eq!(
            quitar_del_path(r"c:\u\l\TASKCODETWO\bin;C:\a", dir, exp).as_deref(),
            Some(r"C:\a")
        );
        assert_eq!(quitar_del_path(r"C:\a;C:\U\L\taskcodetwo", dir, exp), None);
    }
}
