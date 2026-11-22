//! F2-5: modelo de tarea. Ida y vuelta con el emisor propio y con un parser
//! YAML estandar de otra base de codigo (`serde_yaml`, port de libyaml),
//! compatibilidad con las tareas de la 0.16 de este repo y esquema estricto.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use serde_yaml::Value;
use tc2::alta::tarea_md;
use tc2::identidad::convertir_heredado;
use tc2::plataforma::Personas;
use tc2::tarea::{self, Asignado, Cuerpo, Documento, Tarea};

/// Lo que escribia `alta` con su plantilla de texto antes de F2-5 (F1-9
/// punto 6 con `rama: null`): el modelo tiene que dar lo mismo byte a byte.
const DORADA: &str = r#"---
formato: 2
id: "TASK-008"
titulo: "C:\\ruta\\ \"x\" {fecha}"
tipo: "feature"
sprint: 0
etiquetas: []
complejidad: null
carril: null
estado: "planificada"
motivo: null
plan_aprobado: false
rama: null
asignado_a: null
roles:
  diseno: {modelo: null, skills: [], regla: null}
  implementacion: {modelo: "sonnet", skills: [], regla: null}
  revision: {agente: "general-purpose", modelo: null, skills: [], regla: null}
estimacion: null
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: "2026-11-15"
actualizado: "2026-11-15"
dependencias: []
---
## Objetivo

<que se quiere y por que>

## Criterios de aceptacion

- [ ] <criterio comprobable>

## Escenarios

<opcional: 1. **Dado** <estado>, **cuando** <accion>, **entonces** <resultado>>

## Resultado

<obligatoria al cerrar: que se hizo, divergencias con el plan, hallazgos no corregidos y por que>

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
"#;

/// Formato 2 canonico con todo relleno: secciones fuera de orden, texto
/// libre, `estimacion` en bloque (forma de F1-17) y claves desconocidas.
const COMPLETA: &str = r#"---
formato: 2
id: "TASK-120a"
titulo: "Título: con # y \\ y \"comillas\" ñ"
tipo: "hotfix"
sprint: 7
etiquetas: ["rust", "yaml"]
complejidad: "media"
carril: "riesgo-alto"
estado: "descartada"
motivo: "Duplicada de TASK-119"
plan_aprobado: true
rama: "hotfix/task-120a-titulo"
asignado_a: "carlos"
roles:
  diseno: {modelo: "opus", skills: ["a"], regla: "solape"}
  implementacion: {modelo: "sonnet", skills: [], regla: null}
  revision: {agente: "general-purpose", modelo: null, skills: ["b", "c"], regla: "llm"}
estimacion:
  calculada: "2026-11-15T12:00:00Z"
  por_fase: {implementacion: [21, 52]}
  total: [50, 96]
tokens_diseno: 325030
tokens_implementacion: null
tokens_revision: 12
creado: "2026-11-14"
actualizado: "2026-11-15"
dependencias: ["TASK-010", "TASK-119"]
zonas: ["crates/tc2/src/tarea/"]
hito: "H2"
---
## Transiciones

| fecha | fase |
|---|---|
## Objetivo

Texto con `## dentro de codigo`:

```md
## no es seccion
```
## Nota de la persona
libre
"#;

fn leer2(t: &str) -> (Tarea, Cuerpo) {
    match tarea::leer(t) {
        Ok(Documento::Formato2(m, c)) => (m, c),
        otro => panic!("se esperaba formato 2: {otro:?}"),
    }
}

/// Frontmatter de `t` leido con el parser independiente.
fn estandar(t: &str) -> Value {
    let t = t
        .strip_prefix('\u{feff}')
        .unwrap_or(t)
        .replace("\r\n", "\n");
    let resto = t.strip_prefix("---\n").expect("frontmatter");
    let fin = resto.find("\n---\n").expect("cierre");
    serde_yaml::from_str(&resto[..=fin]).expect("YAML valido para serde_yaml")
}

fn cad<'a>(v: &'a Value, clave: &str) -> Option<&'a str> {
    v.get(clave).and_then(Value::as_str)
}

fn lista(v: &Value, clave: &str) -> Vec<String> {
    v.get(clave)
        .and_then(Value::as_sequence)
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_owned())
        .collect()
}

/// El modelo y el parser independiente dicen lo mismo de cada campo.
fn coincide_con_estandar(m: &Tarea, v: &Value) {
    assert_eq!(v.get("formato").and_then(Value::as_u64), Some(2));
    assert_eq!(cad(v, "id"), Some(m.id.as_str()));
    assert_eq!(cad(v, "titulo"), Some(m.titulo.as_str()));
    assert_eq!(cad(v, "tipo"), Some(m.tipo.as_str()));
    assert_eq!(
        v.get("sprint").and_then(Value::as_u64),
        Some(u64::from(m.sprint))
    );
    assert_eq!(lista(v, "etiquetas"), m.etiquetas);
    assert_eq!(cad(v, "complejidad"), m.complejidad.as_deref());
    assert_eq!(cad(v, "carril"), m.carril.as_deref());
    assert_eq!(cad(v, "estado"), Some(m.estado.as_str()));
    assert_eq!(cad(v, "motivo"), m.motivo.as_deref());
    assert_eq!(
        v.get("plan_aprobado").and_then(Value::as_bool),
        Some(m.plan_aprobado)
    );
    assert_eq!(cad(v, "rama"), m.rama.as_deref());
    let a = v.get("asignado_a").unwrap();
    match &m.asignado_a {
        None => assert!(a.is_null()),
        Some(Asignado::Heredado(h)) => assert_eq!(a.as_str(), Some(h.texto())),
        Some(Asignado::Persona { id, login }) => {
            assert_eq!(a.get("id").and_then(Value::as_u64), Some(*id));
            assert_eq!(cad(a, "login"), Some(login.as_str()));
        }
    }
    let roles = v.get("roles").unwrap();
    let r = |rol: &str| roles.get(rol).unwrap();
    assert_eq!(cad(r("diseno"), "modelo"), m.roles.diseno.modelo.as_deref());
    assert_eq!(lista(r("diseno"), "skills"), m.roles.diseno.skills);
    assert_eq!(cad(r("diseno"), "regla"), m.roles.diseno.regla.as_deref());
    assert_eq!(
        cad(r("implementacion"), "modelo"),
        m.roles.implementacion.modelo.as_deref()
    );
    assert_eq!(
        cad(r("revision"), "agente"),
        m.roles.revision.agente.as_deref()
    );
    assert_eq!(lista(r("revision"), "skills"), m.roles.revision.skills);
    assert_eq!(
        cad(r("revision"), "regla"),
        m.roles.revision.regla.as_deref()
    );
    assert_eq!(
        v.get("estimacion").unwrap().is_null(),
        m.estimacion().is_none()
    );
    for (clave, valor) in [
        ("tokens_diseno", m.tokens_diseno),
        ("tokens_implementacion", m.tokens_implementacion),
        ("tokens_revision", m.tokens_revision),
    ] {
        assert_eq!(v.get(clave).and_then(Value::as_u64), valor, "{clave}");
    }
    assert_eq!(cad(v, "creado"), Some(m.creado.as_str()));
    assert_eq!(cad(v, "actualizado"), Some(m.actualizado.as_str()));
    assert_eq!(lista(v, "dependencias"), m.dependencias);
    for (clave, _) in m.claves_desconocidas() {
        assert!(v.get(clave.as_str()).is_some(), "desconocida {clave}");
    }
}

/// C1 (i): el formato 2 canonico sale byte a byte igual, y la plantilla de
/// `alta` es la de antes de F2-5.
#[test]
fn ida_y_vuelta_byte_a_byte() {
    // Desde F2-4b el alta escribe la rama derivada del ID en vez de `null`.
    assert_eq!(
        tarea_md("TASK-008", "C:\\ruta\\ \"x\" {fecha}", "2026-11-15"),
        DORADA.replace(
            "rama: null\n",
            "rama: \"feature/task-008-c-ruta-x-fecha\"\n"
        )
    );
    for t in [DORADA, COMPLETA] {
        let (m, c) = leer2(t);
        assert_eq!(m.escribir(&c), t);
    }
    let (m, c) = leer2(COMPLETA);
    let titulos: Vec<&str> = c.secciones.iter().map(|s| s.titulo.as_str()).collect();
    assert_eq!(titulos, ["Transiciones", "Objetivo", "Nota de la persona"]);
    let claves: Vec<&str> = m
        .claves_desconocidas()
        .iter()
        .map(|(k, _)| k.as_str())
        .collect();
    assert_eq!(claves, ["zonas", "hito"]);
    assert_eq!(m.titulo, "Título: con # y \\ y \"comillas\" ñ");
    // CRLF y BOM se leen; se reescribe con `\n` y sin BOM.
    let crlf = format!("\u{feff}{}", DORADA.replace('\n', "\r\n"));
    let (m2, c2) = leer2(&crlf);
    let (m1, _) = leer2(DORADA);
    assert_eq!(m2, m1);
    assert!(
        m2.escribir(&c2)
            .starts_with("---\nformato: 2\nid: \"TASK-008\"\n")
    );
}

/// F4-6: el mapa `roles` ampliado (divergencia 1 de F1-15) se lee, se
/// reescribe byte a byte igual y lo lee igual un parser estandar; las claves
/// nuevas solo se escriben con valor (las tareas de antes no cambian: las
/// doradas de arriba).
#[test]
fn roles_ampliados_ida_y_vuelta() {
    let ampliada = DORADA.replace(
        "  diseno: {modelo: null, skills: [], regla: null}
",
        "  diseno: {modelo: \"sonnet\", skills: [], regla: null, agentes: [\"p:brainstorm-arquitectura\", \"p:brainstorm-riesgos\"], reglas: [\"M04\", \"S02\", \"S03\"], senales: [\"puertas-seguridad:palabra:check\"], inventario: \"abc\", catalogo: \"def\"}
",
    ).replace(
        "  revision: {agente: \"general-purpose\", modelo: null, skills: [], regla: null}
",
        "  revision: {agente: \"general-purpose\", modelo: \"opus\", skills: [], regla: null, avisos: [\"dominio-sin-revisora:puertas-seguridad\"], ronda: 2}
",
    );
    assert_ne!(ampliada, DORADA);
    let (m, c) = leer2(&ampliada);
    assert_eq!(m.escribir(&c), ampliada);
    assert_eq!(
        m.roles.diseno.agentes,
        ["p:brainstorm-arquitectura", "p:brainstorm-riesgos"]
    );
    assert_eq!(m.roles.diseno.senales, ["puertas-seguridad:palabra:check"]);
    assert_eq!(m.roles.diseno.inventario.as_deref(), Some("abc"));
    assert_eq!(m.roles.revision.ronda, Some(2));
    let v = estandar(&ampliada);
    let d = &v["roles"]["diseno"];
    assert_eq!(d["reglas"][2].as_str(), Some("S03"));
    assert_eq!(v["roles"]["revision"]["ronda"].as_u64(), Some(2));
    // Una clave que no conoce el modelo sigue rechazandose.
    assert!(tarea::leer(&ampliada.replace("ronda: 2", "otra: 2")).is_err());
    // `agente` y `ronda` son solo de revision.
    assert!(
        tarea::leer(&ampliada.replace("diseno: {modelo", "diseno: {agente: \"x\", modelo"))
            .is_err()
    );
    assert!(
        tarea::leer(&ampliada.replace("catalogo: \"def\"}", "catalogo: \"def\", ronda: 1}"))
            .is_err()
    );
}

/// C1 (ii) y (iii): lo que escribe el modelo lo lee igual el propio modelo y
/// un parser YAML estandar independiente, tambien con textos hostiles.
#[test]
fn ida_y_vuelta_con_parser_estandar() {
    let hostiles = [
        "C:\\ruta\\",
        "con \"comillas\" y \\\"mezcla\\\"",
        "clave: valor # no es comentario",
        "ñandú – «comillas» — 漢字",
        "[no es lista]",
        "null",
        "{fecha}",
        // No imprimibles en YAML: tienen que salir escapados.
        "a\u{fffe}b\u{ffff}c",
    ];
    for titulo in hostiles {
        let mut m = Tarea::nueva("TASK-001", titulo, "2026-11-15");
        m.etiquetas = vec![titulo.to_owned(), "otra, con coma".to_owned()];
        m.motivo = Some(format!("{titulo} y motivo"));
        m.rama = Some("feature/task-001-x".to_owned());
        m.tokens_diseno = Some(u64::MAX);
        m.dependencias = vec!["TASK-002".to_owned()];
        let t = m.escribir(&Cuerpo::plantilla());
        let (leida, _) = leer2(&t);
        assert_eq!(leida, m, "{titulo}");
        coincide_con_estandar(&m, &estandar(&t));
        // Con una clave desconocida, que se conserva y lee igual el estandar.
        let t = t.replacen("dependencias:", "zonas: [\"a: b\"]\ndependencias:", 1);
        let (leida, c) = leer2(&t);
        assert_eq!(leida.titulo, titulo);
        assert_eq!(leida.claves_desconocidas().len(), 1);
        coincide_con_estandar(&leida, &estandar(&t));
        assert!(leida.escribir(&c).contains("zonas: [\"a: b\"]\n---\n"));
    }
    let (m, _) = leer2(COMPLETA);
    coincide_con_estandar(&m, &estandar(COMPLETA));
}

fn raiz_repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Todos los `tarea.md` bajo `dir/tareas/<estado>/TASK-*/`.
fn corpus(dir: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    for estado in std::fs::read_dir(dir.join("tareas")).unwrap().flatten() {
        if !estado.path().is_dir() {
            continue;
        }
        for t in std::fs::read_dir(estado.path()).unwrap().flatten() {
            let f = t.path().join("tarea.md");
            if f.is_file() {
                v.push(f);
            }
        }
    }
    v.sort();
    v
}

/// Lee cada tarea de la 0.16 de `dir` y la contrasta con el parser estandar;
/// devuelve cuantas leyo.
fn comprobar_corpus(dir: &Path) -> usize {
    let ficheros = corpus(dir);
    for f in &ficheros {
        let t = std::fs::read_to_string(f).unwrap();
        let Ok(Documento::Formato1(m, c)) = tarea::leer(&t) else {
            panic!("{}: {:?}", f.display(), tarea::leer(&t));
        };
        assert!(
            t.ends_with(&c.escribir()),
            "{}: cuerpo cambiado",
            f.display()
        );
        assert!(c.seccion("Objetivo").is_some(), "{}", f.display());
        let v = estandar(&t);
        assert_eq!(cad(&v, "id"), Some(m.id.as_str()), "{}", f.display());
        assert_eq!(
            cad(&v, "titulo"),
            Some(m.titulo.as_str()),
            "{}",
            f.display()
        );
        assert_eq!(
            cad(&v, "estado"),
            Some(m.estado.as_str()),
            "{}",
            f.display()
        );
        assert_eq!(cad(&v, "rama"), Some(m.rama.as_str()), "{}", f.display());
        assert_eq!(
            cad(&v, "creado"),
            Some(m.creado.as_str()),
            "{}",
            f.display()
        );
        assert_eq!(lista(&v, "dependencias"), m.dependencias, "{}", f.display());
        assert_eq!(
            v.get("tokens_diseno").and_then(Value::as_u64),
            m.tokens_diseno,
            "{}",
            f.display()
        );
        assert!(
            m.desconocidas.is_empty(),
            "{}: {:?}",
            f.display(),
            m.desconocidas
        );
    }
    ficheros.len()
}

/// C2: las tareas de la 0.16 de este repo se leen enteras (los 22 campos,
/// cuerpo byte a byte) y como las lee un parser estandar.
#[test]
fn lee_las_tareas_de_la_0_16_de_este_repo() {
    let n = comprobar_corpus(&raiz_repo());
    assert!(n >= 90, "solo {n} tareas");
}

/// C2, corpus externo (ToolsaWeb, OpenDockTrading): `TC2_CORPUS` con las
/// raices de los repos separadas por `;`. Pesado y manual.
#[test]
#[ignore = "pesado: necesita TC2_CORPUS con copias de otros repos"]
fn lee_las_tareas_de_la_0_16_de_otros_repos() {
    let raices = std::env::var("TC2_CORPUS").expect("TC2_CORPUS");
    for r in raices.split(';').filter(|r| !r.is_empty()) {
        let n = comprobar_corpus(Path::new(r));
        println!("{r}: {n} tareas leidas");
        assert!(n > 0, "{r}: ninguna tarea");
    }
}

/// C2: lo que la 0.16 escribe y un parser estandar leeria distinto, o una
/// clave repetida, es un error claro, no un valor cambiado en silencio.
#[test]
fn formato_1_ambiguo_o_duplicado_falla() {
    let base = std::fs::read_to_string(corpus(&raiz_repo())[0].clone()).unwrap();
    let con_barra = base.replacen("\ntitulo: \"", "\ntitulo: \"C:\\ruta ", 1);
    let e = tarea::leer(&con_barra).unwrap_err();
    assert_eq!(e.clave.as_deref(), Some("titulo"), "{e}");
    assert!(e.mensaje.contains("H11"), "{e}");
    let con_comilla = base.replacen("\ntitulo: \"", "\ntitulo: \"con \\\"comillas\\\" ", 1);
    let Ok(Documento::Formato1(m, _)) = tarea::leer(&con_comilla) else {
        panic!("el escape de comillas de la 0.16 es YAML valido");
    };
    assert!(m.titulo.starts_with("con \"comillas\" "));
    let duplicada = base.replacen("\nsprint:", "\nsprint: 1\nsprint:", 1);
    assert!(
        tarea::leer(&duplicada)
            .unwrap_err()
            .to_string()
            .contains("duplicada")
    );
    // Una barra fuera de comillas dobles (cadena plana, comilla simple,
    // comentario) es literal tambien para un parser estandar: no es H11.
    for (linea, titulo) in [
        ("titulo: 'C:\\ruta' # y \\x", "C:\\ruta"),
        ("titulo: C:\\plana", "C:\\plana"),
        ("titulo: 'it''s \\'", "it's \\"),
        // Una `"` dentro de comilla simple o de un comentario no abre una
        // cadena de comillas dobles.
        ("titulo: 'a\"b\\c'", "a\"b\\c"),
        ("titulo: \"a\" # \"\\x", "a"),
    ] {
        let t = con_linea(&base, "titulo", linea);
        let Ok(Documento::Formato1(m, _)) = tarea::leer(&t) else {
            panic!("{linea}: {:?}", tarea::leer(&t));
        };
        assert_eq!(m.titulo, titulo);
    }
    // Un comentario o una linea en blanco detras de un bool o un entero
    // es YAML valido y no cambia el valor.
    for (clave, linea) in [
        ("sprint", "sprint: 3\n# nota\n"),
        ("sprint", "sprint: 3\t# nota"),
        ("plan_aprobado", "plan_aprobado: true\n\n# x"),
    ] {
        let t = con_linea(&base, clave, linea);
        let Ok(Documento::Formato1(m, _)) = tarea::leer(&t) else {
            panic!("{linea:?}: {:?}", tarea::leer(&t));
        };
        assert!(m.sprint == 3 || m.plan_aprobado, "{linea:?}");
    }
    // Tipos con grafia de YAML 1.1 o entre comillas: error, no conversion.
    for (clave, linea) in [
        ("plan_aprobado", "plan_aprobado: yes"),
        ("revision_codex", "revision_codex: \"false\""),
        ("sprint", "sprint: '7'"),
    ] {
        let e = tarea::leer(&con_linea(&base, clave, linea)).unwrap_err();
        assert_eq!(e.clave.as_deref(), Some(clave), "{linea}: {e}");
    }
}

/// `t` con la linea `clave: ...` del frontmatter sustituida por `nueva`.
fn con_linea(t: &str, clave: &str, nueva: &str) -> String {
    let prefijo = format!("{clave}:");
    let mut hecho = false;
    let mut s: String = t
        .lines()
        .map(|l| {
            if !hecho && l.starts_with(&prefijo) {
                hecho = true;
                nueva.to_owned()
            } else {
                l.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    s.push('\n');
    assert!(hecho, "sin {clave}");
    s
}

/// C3: esquema estricto del formato 2 (F1-9 punto 5, F1-14): tipos, valores
/// permitidos, obligatorias y campos que el modelo perderia.
#[test]
fn esquema_estricto_del_formato_2() {
    for (de, a, clave) in [
        ("sprint: 0", "sprint: \"cero\"", Some("sprint")),
        ("sprint: 0", "sprint: -1", Some("sprint")),
        ("formato: 2", "formato: 3", Some("formato")),
        ("id: \"TASK-008\"", "id: \"TASK-8\"", Some("id")),
        ("tipo: \"feature\"", "tipo: \"epica\"", Some("tipo")),
        (
            "estado: \"planificada\"",
            "estado: \"descartada\"",
            Some("motivo"),
        ),
        (
            "estado: \"planificada\"",
            "estado: \"pausada\"",
            Some("estado"),
        ),
        (
            "creado: \"2026-11-15\"",
            "creado: \"15/11/2026\"",
            Some("creado"),
        ),
        ("titulo: \"C:", "titulo: \"\\t", Some("titulo")),
        ("rama: null\n", "", Some("rama")),
        (
            "plan_aprobado: false",
            "plan_aprobado: null",
            Some("plan_aprobado"),
        ),
        (
            "dependencias: []",
            "dependencias: [\"../x\"]",
            Some("dependencias"),
        ),
        (
            "regla: null}\n  revision",
            "regla: \"azar\"}\n  revision",
            Some("roles.implementacion"),
        ),
        (
            "skills: [], regla: null}\n  implementacion",
            "skills: [], regla: null, extra: 1}\n  implementacion",
            None,
        ),
        (
            "etiquetas: []",
            "etiquetas: []\netiquetas: []",
            Some("etiquetas"),
        ),
        ("etiquetas: []", "etiquetas: &a []\nancla: *a", None),
        // Bool y enteros con grafia de YAML 1.1 o entre comillas (un parser
        // estandar los leeria como cadena): error, no conversion.
        (
            "plan_aprobado: false",
            "plan_aprobado: yes",
            Some("plan_aprobado"),
        ),
        (
            "plan_aprobado: false",
            "plan_aprobado: On",
            Some("plan_aprobado"),
        ),
        (
            "plan_aprobado: false",
            "plan_aprobado: \"false\"",
            Some("plan_aprobado"),
        ),
        ("sprint: 0", "sprint: '7'", Some("sprint")),
        ("sprint: 0", "sprint: 0x10", Some("sprint")),
        ("sprint: 0", "sprint: 1_0", Some("sprint")),
        ("sprint: 0", "sprint: +5", Some("sprint")),
        ("sprint: 0", "sprint: 010", Some("sprint")),
        ("formato: 2", "formato: \"2\"", Some("formato")),
        (
            "tokens_revision: null",
            "tokens_revision: \"12\"",
            Some("tokens_revision"),
        ),
        ("sprint: 0", "sprint:\n  7", Some("sprint")),
    ] {
        assert!(DORADA.contains(de), "{de}");
        let t = DORADA.replacen(de, a, 1);
        let e = tarea::leer(&t).expect_err(a);
        assert_eq!(e.clave.as_deref(), clave, "{a}: {e}");
    }
}

/// TASK-077 C5: `asignado_a: {id, login}` sobrevive a la ida y vuelta (byte
/// a byte, y como lo lee un parser estandar), y «ya la lleva» compara por
/// `id`: el mismo login con otro id es otra persona; un login renombrado con
/// el mismo id, la misma.
#[test]
fn asignado_a_persona_ida_y_vuelta_y_comparacion_por_id() {
    let t = DORADA.replacen(
        "asignado_a: null\n",
        "asignado_a: {id: 1001, login: \"carlos\"}\n",
        1,
    );
    let (m, c) = leer2(&t);
    let a = m.asignado_a.clone().unwrap();
    assert_eq!((a.id(), a.login()), (Some(1001), Some("carlos")));
    assert_eq!(m.escribir(&c), t);
    coincide_con_estandar(&m, &estandar(&t));
    // Desde el modelo: lo que se escribe se lee igual.
    let mut n = Tarea::nueva("TASK-001", "x", "2026-11-16");
    n.asignado_a = Asignado::persona(u64::MAX, "login-con-guion");
    let (leida, _) = leer2(&n.escribir(&Cuerpo::plantilla()));
    assert_eq!(leida, n);
    // Comparacion por id.
    let renombrado = Asignado::persona(1001, "carlos-nuevo").unwrap();
    let suplantador = Asignado::persona(2002, "carlos").unwrap();
    assert!(a.la_lleva(1001) && a.misma_persona(&renombrado));
    assert!(!a.la_lleva(2002) && !a.misma_persona(&suplantador));
    // Logins o ids que no valen.
    assert!(Asignado::persona(0, "x").is_none());
    assert!(Asignado::persona(1, "").is_none());
    assert!(Asignado::persona(1, "a b").is_none());
    for malo in [
        "{id: \"1001\", login: \"carlos\"}",
        "{id: 0, login: \"carlos\"}",
        "{id: 1001}",
        "{id: 1001, login: \"carlos\", nombre: \"C\"}",
        "{id: 1001, login: \"a b\"}",
        "{id: 010, login: \"carlos\"}",
        "7",
        "\"\"",
        "[1001]",
    ] {
        let t = DORADA.replacen("asignado_a: null", &format!("asignado_a: {malo}"), 1);
        let e = tarea::leer(&t).expect_err(malo);
        assert_eq!(e.clave.as_deref(), Some("asignado_a"), "{malo}: {e}");
    }
}

/// TASK-077 C9: un `asignado_a` de texto de la 0.16 se lee como `Heredado`,
/// y nadie lo toma por una persona; la 2.0 no crea uno nunca (solo el
/// lector construye `Heredado`) y, si lo lee en formato 2, lo conserva tal
/// cual en vez de perderlo.
#[test]
fn asignado_a_de_texto_de_la_0_16_es_heredado() {
    let base = std::fs::read_to_string(corpus(&raiz_repo())[0].clone()).unwrap();
    let t = con_linea(&base, "asignado_a", "asignado_a: \"charlie.bk@gmail.com\"");
    let Ok(Documento::Formato1(m, _)) = tarea::leer(&t) else {
        panic!("{:?}", tarea::leer(&t));
    };
    let a = m.asignado_a.unwrap();
    assert_eq!(a.heredado(), Some("charlie.bk@gmail.com"));
    assert_eq!((a.id(), a.login()), (None, None));
    assert!(!a.la_lleva(1001));
    // Si coincide con un `correos:` de `personas.yml` se convierte a esa
    // persona; si no, queda heredado.
    let personas = Personas::leer(
        "personas:\n  - {id: 1001, login: \"charliebk\", correos: [\"Charlie.BK@gmail.com\"]}\n",
    )
    .unwrap();
    let convertido = convertir_heredado(a.clone(), &personas);
    assert_eq!(convertido, Asignado::persona(1001, "charliebk").unwrap());
    assert!(convertido.la_lleva(1001));
    let otra = Personas::leer("personas:\n  - {id: 1001, login: \"charliebk\"}\n").unwrap();
    assert_eq!(convertir_heredado(a.clone(), &otra), a);
    for vacio in ["asignado_a: null", "asignado_a: \"\""] {
        let t = con_linea(&base, "asignado_a", vacio);
        let Ok(Documento::Formato1(m, _)) = tarea::leer(&t) else {
            panic!("{vacio}");
        };
        assert_eq!(m.asignado_a, None, "{vacio}");
    }
    // Una tarea nueva de la 2.0 no lleva asignado; un heredado leido en
    // formato 2 se reescribe igual.
    assert_eq!(Tarea::nueva("TASK-001", "x", "2026-11-16").asignado_a, None);
    let (m, c) = leer2(COMPLETA);
    assert_eq!(
        m.asignado_a.as_ref().and_then(Asignado::heredado),
        Some("carlos")
    );
    assert_eq!(m.escribir(&c), COMPLETA);
}
