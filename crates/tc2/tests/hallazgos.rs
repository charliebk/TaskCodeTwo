//! F6-4 (TASK-050): `tc2 hallazgos` lanzado como binario sobre los ficheros
//! de `fixtures/hallazgos` (un diff de git y respuestas de revisora).
//!
//! - Criterio 1: un hallazgo sin evidencia, o anclado fuera del diff, sale
//!   rechazado con su motivo; una respuesta en prosa, sin bloque json, se
//!   rechaza entera (`sin-bloque`).
//! - Criterio 2: la salida `--salida` valida contra
//!   `esquemas/hallazgos.schema.json` (con un validador del subconjunto de
//!   JSON Schema que usa el esquema, sin `pattern`).

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn fixture(nombre: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(nombre)
}

/// Carpeta temporal propia del test, vacia.
fn carpeta(nombre: &str) -> PathBuf {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("hallazgos-{nombre}"));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Lanza `tc2 hallazgos <args>`; da (codigo, stdout, stderr).
fn hallazgos(args: &[&str]) -> (Option<i32>, String, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_tc2"))
        .arg("hallazgos")
        .args(args)
        .env("TC2_IDIOMA", "es")
        .output()
        .unwrap_or_else(|e| panic!("no se pudo lanzar tc2: {e}"));
    (
        o.status.code(),
        String::from_utf8_lossy(&o.stdout).into_owned(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

fn ruta(p: &Path) -> &str {
    p.to_str().unwrap()
}

const REVISORA: &str = "taskcode-plugin:rust-wasm-reviewer";

/// Criterio 1 y 2: tres hallazgos validos (uno de cada tipo), uno sin
/// evidencia y otro con la linea fuera del diff. La salida valida contra el
/// esquema y el resumen lista aceptados y rechazados.
#[test]
fn sin_evidencia_o_fuera_del_diff_se_rechaza() {
    let d = carpeta("mixta");
    let (salida, resumen) = (d.join("hallazgos.json"), d.join("resumen.md"));
    let diff = fixture("hallazgos/diff.txt");
    let respuesta = fixture("hallazgos/respuesta-mixta.md");
    let informe = fixture("informe-verificacion/xtask-rojo.json");
    let (codigo, linea, err) = hallazgos(&[
        "validar",
        "--respuesta",
        ruta(&respuesta),
        "--diff",
        ruta(&diff),
        "--revisora",
        REVISORA,
        "--informe",
        ruta(&informe),
        "--salida",
        ruta(&salida),
        "--resumen",
        ruta(&resumen),
    ]);
    assert_eq!(codigo, Some(0), "{linea}{err}");
    assert_eq!(
        linea.trim_end(),
        format!(
            "hallazgos ok revisora={REVISORA} aceptados=3 rechazados=2 \
             motivos=H4:sin-evidencia,H5:linea-fuera-del-diff informe=leido"
        )
    );
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&salida).unwrap()).unwrap();
    let esquema: Value = serde_json::from_str(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("esquemas/hallazgos.schema.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let errores = validar_esquema(&esquema, &esquema, &v, "$");
    assert!(errores.is_empty(), "{errores:?}");
    let tipos: Vec<&str> = v["aceptados"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["evidencia"]["tipo"].as_str().unwrap())
        .collect();
    assert_eq!(tipos, ["borde", "mutante", "reproduccion"]);
    // El validador del test no es ciego: una clave de mas o un motivo
    // desconocido no pasan.
    let mut malo = v.clone();
    malo["extra"] = Value::Bool(true);
    assert!(!validar_esquema(&esquema, &esquema, &malo, "$").is_empty());
    let mut malo = v.clone();
    malo["rechazados"][0]["motivo"] = Value::from("opinion");
    assert!(!validar_esquema(&esquema, &esquema, &malo, "$").is_empty());
    let mut malo = v.clone();
    malo["aceptados"][0]["evidencia"]["test"] = Value::from("t");
    assert!(!validar_esquema(&esquema, &esquema, &malo, "$").is_empty());
    // Los aceptados son una respuesta valida ($defs/respuesta).
    let r = validar_esquema(
        &esquema,
        &esquema["$defs"]["respuesta"],
        &v["aceptados"],
        "$r",
    );
    assert!(r.is_empty(), "{r:?}");
    let md = std::fs::read_to_string(&resumen).unwrap();
    for x in [
        "### Hallazgos de taskcode-plugin:rust-wasm-reviewer: 3 aceptados, 2 rechazados",
        "| H1 | CRITICO | src/calculo.rs | 5 | borde |",
        "| H4 | sin-evidencia |",
        "| H5 | linea-fuera-del-diff |",
    ] {
        assert!(md.contains(x), "falta «{x}» en:\n{md}");
    }
    // Con --json, las mismas claves en el contrato.
    let (codigo, json, _) = hallazgos(&[
        "--json",
        "validar",
        "--respuesta",
        ruta(&respuesta),
        "--diff",
        ruta(&diff),
        "--revisora",
        REVISORA,
    ]);
    assert_eq!(codigo, Some(0));
    let j: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(j["claves"]["aceptados"], "3");
    assert_eq!(
        j["claves"]["motivos"],
        serde_json::json!(["H4:sin-evidencia", "H5:linea-fuera-del-diff"])
    );
}

/// Criterio 1: un informe en prosa (lo que pedia el prompt de F6-3) no trae
/// bloque json: todo rechazado con `sin-bloque`.
#[test]
fn una_respuesta_en_prosa_se_rechaza_entera() {
    let (codigo, linea, err) = hallazgos(&[
        "validar",
        "--respuesta",
        ruta(&fixture("hallazgos/respuesta-prosa.md")),
        "--diff",
        ruta(&fixture("hallazgos/diff.txt")),
        "--revisora",
        REVISORA,
    ]);
    assert_eq!(codigo, Some(0), "{err}");
    assert_eq!(
        linea.trim_end(),
        format!(
            "hallazgos ok revisora={REVISORA} aceptados=0 rechazados=1 motivos=respuesta:sin-bloque"
        )
    );
}

/// `instrucciones` da el texto del prompt (con todos los motivos) y su
/// ejemplo pasa por `validar` contra el diff de los fixtures; los errores de
/// uso y de lectura dicen que hacer.
#[test]
fn instrucciones_y_errores_de_uso() {
    let (codigo, texto, _) = hallazgos(&["instrucciones"]);
    assert_eq!(codigo, Some(0));
    for m in [
        "`sin-bloque`",
        "`sin-evidencia`",
        "`linea-fuera-del-diff`",
        "`mutante-no-coincide`",
        "`borde-sin-diferencia`",
        "```json",
    ] {
        assert!(texto.contains(m), "falta {m}");
    }
    let d = carpeta("instrucciones");
    let p = d.join("instrucciones.md");
    std::fs::write(&p, &texto).unwrap();
    let (codigo, linea, _) = hallazgos(&[
        "validar",
        "--respuesta",
        ruta(&p),
        "--diff",
        ruta(&fixture("hallazgos/diff.txt")),
        "--revisora",
        "r",
    ]);
    assert_eq!(codigo, Some(0));
    assert_eq!(
        linea.trim_end(),
        "hallazgos ok revisora=r aceptados=2 rechazados=0 motivos="
    );
    let (codigo, json, _) = hallazgos(&["--json", "instrucciones"]);
    assert_eq!(codigo, Some(0));
    let j: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(j["claves"]["instrucciones"].as_str(), Some(texto.as_str()));
    // Uso.
    let (codigo, linea, err) = hallazgos(&["validar", "--respuesta", ruta(&p)]);
    assert_eq!(codigo, Some(2));
    assert_eq!(linea.trim_end(), "hallazgos uso motivo=sin-diff");
    assert!(err.contains("uso: hallazgos"), "{err}");
    let (codigo, linea, _) = hallazgos(&[]);
    assert_eq!(
        (codigo, linea.trim_end()),
        (Some(2), "hallazgos uso motivo=sin-suborden")
    );
    let (codigo, linea, _) = hallazgos(&["validar", "--revisora", "no vale"]);
    assert_eq!(
        (codigo, linea.trim_end()),
        (Some(2), "hallazgos uso motivo=revisora-invalida")
    );
    // Lectura.
    let falta = d.join("no-existe.md");
    let (codigo, linea, err) = hallazgos(&[
        "validar",
        "--respuesta",
        ruta(&falta),
        "--diff",
        ruta(&p),
        "--revisora",
        "r",
    ]);
    assert_eq!((codigo, linea.trim_end()), (Some(1), "hallazgos error"));
    assert!(
        err.contains("no-existe.md") && err.contains("comprueba la ruta"),
        "{err}"
    );
}

// --- Validador del subconjunto de JSON Schema que usa el esquema ------------

fn tipo_de(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(n) if n.is_i64() || n.is_u64() => "integer",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Errores de `v` contra `s` (con `$ref` a `#/$defs/...` de `raiz`):
/// `type`, `const`, `enum`, `required`, `properties`,
/// `additionalProperties: false`, `items`, `minItems`, `maxItems`,
/// `minLength`, `minimum`, `oneOf`, `anyOf`. Una palabra clave que no
/// conoce hace fallar el test (el esquema no puede crecer sin el validador).
fn validar_esquema(raiz: &Value, s: &Value, v: &Value, donde: &str) -> Vec<String> {
    let mut e = Vec::new();
    let Some(m) = s.as_object() else {
        return vec![format!("{donde}: esquema no es un objeto")];
    };
    for (k, x) in m {
        match k.as_str() {
            "$schema" | "title" | "description" | "$defs" | "pattern" => {}
            "$ref" => {
                let nombre = x.as_str().and_then(|r| r.strip_prefix("#/$defs/")).unwrap();
                e.extend(validar_esquema(raiz, &raiz["$defs"][nombre], v, donde));
            }
            "type" => {
                let t = tipo_de(v);
                let ok = match x {
                    Value::String(a) => a == t || (a == "number" && t == "integer"),
                    Value::Array(a) => a.iter().any(|y| y.as_str() == Some(t)),
                    _ => false,
                };
                if !ok {
                    e.push(format!("{donde}: tipo {t}, se pide {x}"));
                }
            }
            "const" if v != x => e.push(format!("{donde}: {v} no es {x}")),
            "const" => {}
            "enum" if !x.as_array().unwrap().contains(v) => {
                e.push(format!("{donde}: {v} fuera de {x}"));
            }
            "enum" => {}
            "required" => {
                for r in x.as_array().unwrap() {
                    let r = r.as_str().unwrap();
                    if v.as_object().is_some_and(|o| !o.contains_key(r)) {
                        e.push(format!("{donde}: falta {r}"));
                    }
                }
            }
            "properties" => {
                if let Some(o) = v.as_object() {
                    for (c, y) in o {
                        if let Some(sub) = x.get(c) {
                            e.extend(validar_esquema(raiz, sub, y, &format!("{donde}.{c}")));
                        }
                    }
                }
            }
            "additionalProperties" => {
                assert_eq!(x, &Value::Bool(false));
                if let (Some(o), Some(p)) = (v.as_object(), m.get("properties")) {
                    for c in o.keys() {
                        if p.get(c).is_none() {
                            e.push(format!("{donde}: clave desconocida {c}"));
                        }
                    }
                }
            }
            "items" => {
                if let Some(a) = v.as_array() {
                    for (i, y) in a.iter().enumerate() {
                        e.extend(validar_esquema(raiz, x, y, &format!("{donde}[{i}]")));
                    }
                }
            }
            "minItems" | "maxItems" => {
                if let Some(a) = v.as_array() {
                    let n = x.as_u64().unwrap() as usize;
                    if (k == "minItems" && a.len() < n) || (k == "maxItems" && a.len() > n) {
                        e.push(format!("{donde}: {k} {n}"));
                    }
                }
            }
            "minLength" => {
                if let Some(t) = v.as_str()
                    && (t.chars().count() as u64) < x.as_u64().unwrap()
                {
                    e.push(format!("{donde}: cadena corta"));
                }
            }
            "minimum" => {
                if let Some(n) = v.as_f64()
                    && n < x.as_f64().unwrap()
                {
                    e.push(format!("{donde}: menor que {x}"));
                }
            }
            "oneOf" | "anyOf" => {
                let validos = x
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|sub| validar_esquema(raiz, sub, v, donde).is_empty())
                    .count();
                if (k == "oneOf" && validos != 1) || (k == "anyOf" && validos == 0) {
                    e.push(format!("{donde}: {k} casa con {validos}"));
                }
            }
            otro => panic!("palabra clave del esquema sin validar: {otro}"),
        }
    }
    e
}
