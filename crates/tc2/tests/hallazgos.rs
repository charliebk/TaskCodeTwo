//! F6-4 (TASK-050): `tc2 hallazgos` lanzado como binario sobre los ficheros
//! de `fixtures/hallazgos` (un diff de git y respuestas de revisora).
//!
//! - Criterio 1: un hallazgo sin evidencia, o anclado fuera del diff, sale
//!   rechazado con su motivo; una respuesta en prosa, sin bloque json, se
//!   rechaza entera (`sin-bloque`).
//! - Criterio 2: la salida `--salida` valida contra
//!   `esquemas/hallazgos.schema.json` (con un validador del subconjunto de
//!   JSON Schema que usa el esquema, sin `pattern`).
//!
//! F6-5 (TASK-051): `tc2 hallazgos veredicto` sobre las salidas de
//! `validar` de varias revisoras.
//!
//! - Criterio «el check refleja los hallazgos»: un IMPORTANTE aceptado da
//!   `failure` (rojo) y su anotacion `failure` con el aceptado en
//!   `raw_details`; solo MENOR da `success`; una respuesta en prosa da
//!   `failure` (incompleta); un CRITICO sin evidencia no pone el check en
//!   rojo; mas de 50 aceptados van en lotes.
//! - Criterio 2: `check.json`, los lotes y el informe de la tarea validan
//!   contra `$defs/veredicto`, `$defs/lote` y `$defs/informe-tarea`.

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
        "| H1 | CRITICO | correccion | src/calculo.rs | 5 | borde |",
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

/// Pasa `respuesta` por `validar` como la revisora `revisora` y deja su
/// salida en `d/<nombre>.json`.
fn validados(d: &Path, nombre: &str, revisora: &str, respuesta: &str) -> PathBuf {
    let salida = d.join(format!("{nombre}.json"));
    let (codigo, linea, err) = hallazgos(&[
        "validar",
        "--respuesta",
        ruta(&fixture(&format!("hallazgos/{respuesta}"))),
        "--diff",
        ruta(&fixture("hallazgos/diff.txt")),
        "--revisora",
        revisora,
        "--salida",
        ruta(&salida),
    ]);
    assert_eq!(codigo, Some(0), "{linea}{err}");
    salida
}

/// `tc2 hallazgos veredicto` con esas entradas en `d/<nombre>/`; da la
/// linea y el `check.json`.
fn veredicto(d: &Path, nombre: &str, entradas: &[&Path]) -> (String, Value) {
    let dir = d.join(nombre);
    let mut args = vec![
        "veredicto".to_owned(),
        "--salida".to_owned(),
        ruta(&dir).to_owned(),
    ];
    for e in entradas {
        args.push("--hallazgos".to_owned());
        args.push(ruta(e).to_owned());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let (codigo, linea, err) = hallazgos(&refs);
    assert_eq!(codigo, Some(0), "{linea}{err}");
    let check =
        serde_json::from_str(&std::fs::read_to_string(dir.join("check.json")).unwrap()).unwrap();
    (linea.trim_end().to_owned(), check)
}

/// Criterio «el check refleja los hallazgos», con el binario real: la regla
/// del rojo sin GitHub.
#[test]
fn el_check_refleja_los_hallazgos() {
    let d = carpeta("veredicto");
    let a = validados(&d, "a", "rev:a", "respuesta-importante.md");
    let b = validados(&d, "b", "rev:b", "respuesta-menor.md");
    let c = validados(&d, "c", "rev:c", "respuesta-prosa.md");

    // Un IMPORTANTE aceptado: rojo. El CRITICO sin evidencia de la misma
    // respuesta se rechaza y no cuenta.
    let (linea, check) = veredicto(&d, "rojo", &[&a, &b]);
    assert_eq!(
        linea,
        "hallazgos ok veredicto=rojo conclusion=failure criticos=0 importantes=1 menores=1 revisoras=2 incompletas=0 lotes=1"
    );
    assert_eq!(check["conclusion"], "failure");
    assert_eq!(
        check["output"]["title"],
        "revision-ia: rojo (criticos=0 importantes=1 menores=1)"
    );
    let notas = check["output"]["annotations"].as_array().unwrap();
    assert_eq!(notas.len(), 2);
    assert_eq!(notas[0]["annotation_level"], "failure");
    assert_eq!(notas[0]["path"], "src/calculo.rs");
    assert_eq!(
        (
            notas[0]["start_line"].as_u64(),
            notas[0]["end_line"].as_u64()
        ),
        (Some(12), Some(12))
    );
    assert_eq!(
        notas[0]["title"],
        "I1 IMPORTANTE fidelidad-evidencia (rev:a)"
    );
    assert_eq!(notas[1]["annotation_level"], "warning");
    // raw_details es el aceptado tal cual salio de validar, con su revisora.
    let salida_a: Value = serde_json::from_str(&std::fs::read_to_string(&a).unwrap()).unwrap();
    let crudo: Value = serde_json::from_str(notas[0]["raw_details"].as_str().unwrap()).unwrap();
    assert_eq!(
        crudo,
        serde_json::json!({"revisora": "rev:a", "hallazgo": salida_a["aceptados"][0]})
    );
    assert_eq!(salida_a["rechazados"][0]["id"], "C1");
    assert!(
        check["output"]["summary"]
            .as_str()
            .unwrap()
            .contains("**Veredicto: rojo**"),
        "{}",
        check["output"]["summary"]
    );

    // Solo MENOR: verde.
    let (linea, check) = veredicto(&d, "verde", &[&b]);
    assert_eq!(
        linea,
        "hallazgos ok veredicto=verde conclusion=success criticos=0 importantes=0 menores=1 revisoras=1 incompletas=0 lotes=1"
    );
    assert_eq!(check["conclusion"], "success");

    // Una respuesta en prosa (sin-bloque): la revision no se completo.
    let (linea, check) = veredicto(&d, "prosa", &[&a, &b, &c]);
    assert_eq!(
        linea,
        "hallazgos ok veredicto=incompleta conclusion=failure criticos=0 importantes=1 menores=1 revisoras=3 incompletas=1 lotes=1"
    );
    assert_eq!(check["conclusion"], "failure");
    assert!(
        check["output"]["summary"]
            .as_str()
            .unwrap()
            .contains("| rev:c | 0 | 1 | incompleta (sin-bloque) |"),
        "{}",
        check["output"]["summary"]
    );
    // Sin la salida de una revisora, o sin ninguna, tampoco hay verde.
    let falta = d.join("no-existe.json");
    let (linea, _) = veredicto(&d, "falta", &[&b, &falta]);
    assert!(
        linea.contains("veredicto=incompleta conclusion=failure"),
        "{linea}"
    );
    let (linea, check) = veredicto(&d, "ninguna", &[]);
    assert!(
        linea.contains("veredicto=incompleta conclusion=failure"),
        "{linea}"
    );
    assert_eq!(check["output"]["annotations"], serde_json::json!([]));

    // Mas de 50 aceptados: 50 en check.json y el resto en anotaciones-1.json.
    let mut muchos = salida_a.clone();
    let base = muchos["aceptados"][0].clone();
    muchos["aceptados"] = Value::Array(
        (0..51)
            .map(|i| {
                let mut h = base.clone();
                h["id"] = Value::from(format!("I{i}"));
                h
            })
            .collect(),
    );
    muchos["rechazados"] = serde_json::json!([]);
    let m = d.join("muchos.json");
    std::fs::write(&m, muchos.to_string()).unwrap();
    let (linea, check) = veredicto(&d, "lotes", &[&m]);
    assert!(
        linea.ends_with("importantes=51 menores=0 revisoras=1 incompletas=0 lotes=2"),
        "{linea}"
    );
    assert_eq!(check["output"]["annotations"].as_array().unwrap().len(), 50);
    let lote: Value = serde_json::from_str(
        &std::fs::read_to_string(d.join("lotes").join("anotaciones-1.json")).unwrap(),
    )
    .unwrap();
    let resto = lote["output"]["annotations"].as_array().unwrap();
    assert_eq!(resto.len(), 1);
    assert!(
        resto[0]["raw_details"]
            .as_str()
            .unwrap()
            .contains("\"I50\"")
    );
    assert!(!d.join("lotes").join("anotaciones-2.json").exists());
    // Uso: sin --salida.
    let (codigo, linea, err) = hallazgos(&["veredicto", "--hallazgos", ruta(&a)]);
    assert_eq!(
        (codigo, linea.trim_end()),
        (Some(2), "hallazgos uso motivo=sin-salida")
    );
    assert!(err.contains("veredicto --salida"), "{err}");
}

/// Criterio 2: `check.json`, un lote y el informe de la tarea validan
/// contra el esquema v2; una conclusion `neutral` o una anotacion de mas no.
#[test]
fn veredicto_e_informe_validan_contra_el_esquema() {
    let d = carpeta("esquema-v2");
    let a = validados(&d, "a", "rev:a", "respuesta-mixta.md");
    let i = validados(&d, "i", "rev:i", "respuesta-importante.md");
    let (_, check) = veredicto(&d, "v", &[&a, &i]);
    let esquema: Value = serde_json::from_str(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("esquemas/hallazgos.schema.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let contra = |def: &str, v: &Value| validar_esquema(&esquema, &esquema["$defs"][def], v, "$");
    assert_eq!(contra("veredicto", &check), Vec::<String>::new());
    let lote = serde_json::json!({"output": check["output"].clone()});
    assert_eq!(contra("lote", &lote), Vec::<String>::new());
    let mut malo = check.clone();
    malo["conclusion"] = Value::from("neutral");
    assert!(!contra("veredicto", &malo).is_empty());
    let mut malo = check.clone();
    let mas: Vec<Value> = (0..51)
        .map(|_| check["output"]["annotations"][0].clone())
        .collect();
    malo["output"]["annotations"] = Value::Array(mas);
    assert!(!contra("veredicto", &malo).is_empty());
    // Cada raw_details es un aceptado-de.
    for n in check["output"]["annotations"].as_array().unwrap() {
        let crudo: Value = serde_json::from_str(n["raw_details"].as_str().unwrap()).unwrap();
        assert_eq!(contra("aceptado-de", &crudo), Vec::<String>::new());
    }
    // El informe de la tarea, hecho con lo que se leeria del check.
    let aceptados: Vec<tc2::hallazgos::AceptadoDe> = check["output"]["annotations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| serde_json::from_str(n["raw_details"].as_str().unwrap()).unwrap())
        .collect();
    let informe = tc2::hallazgos::InformeTarea::nuevo(
        "TASK-051",
        12,
        &"a".repeat(40),
        ("failure", tc2::hallazgos::Dictamen::Rojo),
        check["output"]["summary"].as_str().unwrap(),
        aceptados,
    );
    let v = serde_json::to_value(&informe).unwrap();
    assert_eq!(contra("informe-tarea", &v), Vec::<String>::new());
    assert_eq!(
        (
            v["criticos"].as_u64(),
            v["importantes"].as_u64(),
            v["menores"].as_u64()
        ),
        (Some(1), Some(2), Some(1))
    );
    let mut malo = v.clone();
    malo["aceptados"][0]["hallazgo"]["clase"] = Value::from("estilo");
    assert!(!contra("informe-tarea", &malo).is_empty());
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
/// `minLength`, `maxLength`, `minimum`, `oneOf`, `anyOf`. Una palabra clave que no
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
            "maxLength" => {
                if let Some(t) = v.as_str()
                    && (t.chars().count() as u64) > x.as_u64().unwrap()
                {
                    e.push(format!("{donde}: cadena larga"));
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
