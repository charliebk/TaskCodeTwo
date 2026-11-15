//! Configuracion `.taskcode/config.yml` (F2-7): esquema publicado, valores
//! por defecto, configuraciones hostiles y lectura desde `origin/develop`.

#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

mod comun;

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::Value;
use tc2::carril::Carril;
use tc2::config::{self, Config, ModoFlujo, Motivo, Runner, TipoDerivado};

use comun::{Fixture, escribir};

const CABECERA: &str = "---\nformato: 2\n";

fn ruta_esquema() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("esquemas/config.schema.json")
}

fn esquema() -> Value {
    serde_json::from_str(&config::esquema_json()).expect("el esquema generado es JSON")
}

#[test]
fn esquema_publicado_es_el_generado() {
    let generado = config::esquema_json();
    let ruta = ruta_esquema();
    if std::env::var_os("TC2_ESCRIBIR_ESQUEMA").is_some_and(|v| v == "1") {
        std::fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        std::fs::write(&ruta, &generado).unwrap();
    }
    let publicado = std::fs::read_to_string(&ruta)
        .unwrap_or_default()
        .replace("\r\n", "\n");
    assert!(
        publicado == generado,
        "{} no es el esquema que genera el codigo: regeneralo con TC2_ESCRIBIR_ESQUEMA=1 cargo test -p tc2 --test config",
        ruta.display()
    );
}

/// Recorre el esquema: cada objeto cerrado, y la lista de hojas con su ruta
/// (`a.b`, `a[].b`).
fn recorrer(v: &Value, ruta: &str, hojas: &mut Vec<String>) {
    if let Some(formas) = v.get("anyOf").and_then(Value::as_array) {
        hojas.push(ruta.to_owned());
        for f in formas {
            if f.get("type") == Some(&Value::from("object")) {
                assert_eq!(f["additionalProperties"], Value::Bool(false), "{ruta}");
            }
        }
        return;
    }
    match v.get("type").and_then(Value::as_str) {
        Some("object") => {
            assert_eq!(
                v["additionalProperties"],
                Value::Bool(false),
                "{ruta} abierto"
            );
            let props = v["properties"].as_object().unwrap();
            for (k, x) in props {
                let r = if ruta.is_empty() {
                    k.clone()
                } else {
                    format!("{ruta}.{k}")
                };
                recorrer(x, &r, hojas);
            }
            if let Some(req) = v.get("required").and_then(Value::as_array) {
                for r in req {
                    assert!(
                        props.contains_key(r.as_str().unwrap()),
                        "{ruta}: requerida sin propiedad"
                    );
                }
            }
        }
        Some("array") if v["items"].get("type") == Some(&Value::from("object")) => {
            recorrer(&v["items"], &format!("{ruta}[]"), hojas);
        }
        _ => hojas.push(ruta.to_owned()),
    }
}

#[test]
fn esquema_cerrado_y_con_las_claves_de_la_tabla() {
    let e = esquema();
    assert_eq!(e["$schema"], "https://json-schema.org/draft/2020-12/schema");
    assert_eq!(e["required"], serde_json::json!(["formato"]));
    let claves: Vec<&String> = e["properties"].as_object().unwrap().keys().collect();
    assert!(
        claves.len() <= config::TOPE_CLAVES,
        "{} claves",
        claves.len()
    );
    for k in &claves {
        let d = e["properties"][k.as_str()]["description"].as_str().unwrap();
        assert!(d.contains("[F1-"), "{k} sin ADR de origen: {d}");
    }
    let mut hojas = Vec::new();
    recorrer(&e, "", &mut hojas);
    // El suelo de F1-10 esta en el esquema: riesgo alto no baja de 4 roles.
    assert_eq!(
        e["properties"]["politica_carriles"]["properties"]["riesgo-alto"]["properties"]["roles_diseno"]
            ["minimum"],
        4
    );
    assert_eq!(e["properties"]["tope_rondas"]["default"], 2);
}

#[test]
fn tabla_de_f1_11_igual_que_el_esquema() {
    let adr = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/adr/F1-11-extensiones-declaradas.md"),
    )
    .unwrap();
    let inicio = adr.find("<!-- esquema:inicio -->").unwrap();
    let fin = adr.find("<!-- esquema:fin -->").unwrap();
    let mut del_adr = BTreeSet::new();
    for l in adr[inicio..fin].lines() {
        let l = l.trim();
        if let Some(resto) = l.strip_prefix("| `")
            && let Some((clave, _)) = resto.split_once('`')
        {
            del_adr.insert(clave.replace("<p>", "*"));
        }
    }
    // Las filas de contenedor (mapa y lista) no son hojas.
    del_adr.remove("verificacion");
    del_adr.remove("derivados");
    let mut hojas = Vec::new();
    recorrer(&esquema(), "", &mut hojas);
    let mut del_esquema = BTreeSet::new();
    for h in hojas {
        if !["verificacion", "derivados", "confianza_incluye"]
            .iter()
            .any(|p| h == *p || h.starts_with(&format!("{p}.")) || h.starts_with(&format!("{p}[")))
        {
            continue;
        }
        let mut partes: Vec<&str> = h.split('.').collect();
        if partes.len() == 4 && partes[1] == "perfiles" {
            partes[2] = "*";
        }
        del_esquema.insert(partes.join("."));
    }
    assert_eq!(del_esquema, del_adr);
}

#[test]
fn config_minima_da_los_valores_por_defecto() {
    let c = config::leer(CABECERA).unwrap();
    assert_eq!(c, Config::suelo());
    assert_eq!(c.modo_flujo, ModoFlujo::Manual);
    assert_eq!(c.tope_rondas, 2);
    assert_eq!(c.seleccion.modelos_permitidos, ["haiku", "sonnet", "opus"]);
    assert!(c.politica_carriles.is_empty() && c.rutas_carril.is_empty());
    assert!(c.verificacion.is_none() && c.plataforma.is_none() && c.lote.is_none());
    // Los valores por defecto anidados salen de la tabla.
    let c = config::leer(&format!(
        "{CABECERA}derivados:\n  - comando: 'x'\n    tipo: 'comprueba'\npolitica_carriles:\n  fix: {{lote: false}}\n"
    ))
    .unwrap();
    assert_eq!(c.derivados[0].timeout, 60);
    assert_eq!(c.derivados[0].fases, config::FASES);
    assert!(c.derivados[0].escribe.is_empty());
    let fix = &c.politica_carriles[&Carril::Fix];
    assert_eq!((fix.lote, fix.paradas_extra.len()), (Some(false), 0));
}

#[test]
fn configs_hostiles_rechazadas_con_la_ruta() {
    use Motivo::{ConfigInvalida as Inv, Formato, Yaml};
    let runner = "runner: {github: windows}";
    let casos: Vec<(String, Motivo, &str, &str)> = vec![
        ("politica_carriles: {riesgo-alto: {aprobaciones: 1}}".into(), Inv, "politica_carriles.riesgo-alto.aprobaciones", "clave no admitida"),
        ("politica_carriles: {fix: {paradas_quitar: [merge-pr]}}".into(), Inv, "politica_carriles.fix.paradas_quitar", "clave no admitida"),
        ("politica_carriles: {release: {}}".into(), Inv, "politica_carriles.release", "clave no admitida"),
        ("politica_carriles: {funcionalidad: {plan_aprobado: false}}".into(), Inv, "politica_carriles.funcionalidad.plan_aprobado", "solo admite true"),
        ("politica_carriles: {riesgo-alto: {lote: true}}".into(), Inv, "politica_carriles.riesgo-alto.lote", "solo admite false"),
        ("politica_carriles: {fix: {perfil: rapida}}".into(), Inv, "politica_carriles.fix.perfil", "no admitido"),
        ("politica_carriles: {riesgo-alto: {roles_diseno: 2}}".into(), Inv, "politica_carriles.riesgo-alto.roles_diseno", "de 4 a 5"),
        ("politica_carriles: {funcionalidad: {paradas_extra: [[start]]}}".into(), Inv, "politica_carriles.funcionalidad.paradas_extra[0]", "texto"),
        ("rutas_carril: {errata: ['docs/**']}".into(), Inv, "rutas_carril.errata", "clave no admitida"),
        ("rutas_carril: {riesgo-alto: ['!docs/**']}".into(), Inv, "rutas_carril.riesgo-alto[0]", "negaciones"),
        ("rutas_carril: {fix: ['src/../x']}".into(), Inv, "rutas_carril.fix[0]", ".."),
        ("rutas_carril: {fix: []}".into(), Inv, "rutas_carril.fix", "al menos 1"),
        (format!("verificacion:\n  comando: \"scripts\\verificar.bat\"\n  shell: 'cmd'\n  perfiles:\n    completa: {{timeout: 60, {runner}}}"), Inv, "verificacion.comando", "comillas simples"),
        ("verificacion:\n  comando: 'x'\n  shell: 'cmd'\n  perfiles:\n    completa: {timeout: 60}".into(), Inv, "verificacion.perfiles.completa.runner", "obligatoria"),
        (format!("verificacion:\n  comando: 'x'\n  shell: 'cmd'\n  perfiles:\n    completa: {{timeout: 60, {runner}}}\n    rapida: {{timeout: 60, {runner}}}"), Inv, "verificacion.perfiles.rapida.runner", "clave no admitida"),
        ("verificacion:\n  comando: 'x'\n  shell: 'zsh'\n  perfiles:\n    completa: {timeout: 60, runner: [windows]}".into(), Inv, "verificacion.shell", "no admitido"),
        ("estados: [x]".into(), Inv, "estados", "clave no admitida"),
        ("comando_verificacion: 'cargo test'".into(), Inv, "comando_verificacion", "tc2 actualizar"),
        ("derivados:\n  - comando: 'x'\n    tipo: 'genera'".into(), Inv, "derivados[0].escribe", "obligatoria"),
        ("derivados:\n  - comando: 'x'\n    tipo: 'genera'\n    escribe: ['tareas/ROADMAP.md']".into(), Inv, "derivados[0].escribe[0]", "reservada"),
        ("derivados:\n  - comando: 'x'\n    tipo: 'comprueba'\n    fases: ['merge']".into(), Inv, "derivados[0].fases[0]", "no admitido"),
        ("confianza_incluye: ['**']".into(), Inv, "confianza_incluye[0]", "todo el arbol"),
        ("plataforma: {proveedor: 'gitlab', api: 'http://git.empresa.com'}".into(), Inv, "plataforma.api", "https"),
        ("seleccion: {modelos_permitidos: [gpt]}".into(), Inv, "seleccion.modelos_permitidos[0]", "no admitido"),
        ("seleccion: {dominios_extra: {datos: {}}}".into(), Inv, "seleccion.dominios_extra.datos", "clave no admitida"),
        ("recursos:\n  - {id: 'a', tipo: 'skill'}\n  - {id: 'a', tipo: 'agente'}".into(), Inv, "recursos[1].id", "repetido"),
        ("modo_flujo: 'rapido'".into(), Inv, "modo_flujo", "no admitido"),
        ("tope_rondas: 0".into(), Inv, "tope_rondas", "de 1 a 10"),
        ("tope_rondas: '2'".into(), Inv, "tope_rondas", "de 1 a 10"),
        ("lote: {presupuesto_tokens: 0}".into(), Inv, "lote.presupuesto_tokens", "1 o mas"),
        ("politica_carriles:\n  fix:\n    lote: false\n    lote: false".into(), Yaml, "", "YAML"),
        ("a: &x 1\nb: *x".into(), Yaml, "", "YAML"),
    ];
    for (cuerpo, motivo, ruta, fragmento) in &casos {
        let e = config::leer(&format!("{CABECERA}{cuerpo}\n")).expect_err(cuerpo);
        assert_eq!(e.motivo, *motivo, "{cuerpo}: {e:?}");
        let x = e.errores.iter().find(|x| x.ruta == *ruta);
        assert!(
            x.is_some_and(|x| x.mensaje.contains(fragmento)),
            "{cuerpo}: se esperaba «{fragmento}» en {ruta}: {:?}",
            e.errores
        );
    }
    for (texto, ruta) in [
        ("formato: 2\n", ""),
        ("\u{feff}---\nformato: 2\n", ""),
        ("---\ntope_rondas: 2\n", "formato"),
        ("---\nformato: 1\n", "formato"),
        ("---\n", "formato"),
    ] {
        let e = config::leer(texto).expect_err(texto);
        assert_eq!(
            (e.motivo, e.errores[0].ruta.as_str()),
            (Formato, ruta),
            "{texto:?}"
        );
    }
    // Dos errores: salen los dos, no solo el primero.
    let e = config::leer(&format!("{CABECERA}estados: 1\ntope_rondas: 0\n")).unwrap_err();
    let rutas: Vec<&str> = e.errores.iter().map(|x| x.ruta.as_str()).collect();
    assert_eq!(rutas, ["estados", "tope_rondas"]);
}

#[test]
fn traduccion_de_toolsaweb_de_f1_11_es_valida() {
    let yaml = r"verificacion:
  comando: 'scripts\verificar.bat'
  shell: 'cmd'
  perfiles:
    rapida:   {timeout: 180}
    impacto:  {timeout: 1800}
    completa: {timeout: 4800, runner: [self-hosted, windows, toolsaweb]}
    pesada:   {timeout: 4800, runner: [self-hosted, windows, toolsaweb]}
  huella_arbol: ['Cargo.lock', 'web/package-lock.json']
  huella_local: 'python -I scripts/huella-local.py'
derivados:
  - comando: 'python -I scripts/grafo-tareas.py --escribir'
    tipo: 'genera'
    escribe: ['tareas/hitos.md']
  - comando: 'python -I scripts/grafo-tareas.py --comprobar'
    tipo: 'comprueba'
confianza_incluye: ['scripts/cache-verificacion.py', 'scripts/memo_ficheros.py']
modo_flujo: 'automatico'
";
    let c = config::leer(&format!("{CABECERA}{yaml}")).unwrap();
    let v = c.verificacion.unwrap();
    assert_eq!(v.comando, r"scripts\verificar.bat");
    assert_eq!(v.perfiles.completa.timeout, 4800);
    assert_eq!(
        v.perfiles.pesada.unwrap().runner,
        Some(Runner::Etiquetas(vec![
            "self-hosted".into(),
            "windows".into(),
            "toolsaweb".into()
        ]))
    );
    assert_eq!(v.perfiles.rapida.unwrap().runner, None);
    assert_eq!(c.derivados[0].tipo, TipoDerivado::Genera);
    assert_eq!(c.derivados[0].escribe, ["tareas/hitos.md"]);
    assert_eq!(c.confianza_incluye.len(), 2);
    assert_eq!(c.modo_flujo, ModoFlujo::Automatico);
}

#[test]
fn de_base_lee_origin_develop_y_no_el_worktree() {
    let fx = Fixture::nueva("config-base");
    let (_bare, clon) = fx.remoto_y_clon();
    escribir(
        &clon,
        ".taskcode/config.yml",
        &format!("{CABECERA}tope_rondas: 3\n"),
    );
    fx.crudo(&clon, &["add", "."]);
    fx.crudo(&clon, &["commit", "-q", "-m", "config"]);
    fx.crudo(&clon, &["push", "-q", "origin", "develop"]);
    // Un commit local sin subir y un cambio sin commitear, los dos invalidos:
    // ninguno cuenta.
    escribir(
        &clon,
        ".taskcode/config.yml",
        &format!("{CABECERA}estados: 1\n"),
    );
    fx.crudo(&clon, &["commit", "-q", "-am", "local"]);
    escribir(&clon, ".taskcode/config.yml", "modo_flujo: manual\n");
    let c = config::de_base(&fx.git(), &clon).unwrap();
    assert_eq!(c.tope_rondas, 3);
}

#[test]
fn sin_config_o_de_la_016_rechazada_con_el_contrato() {
    let fx = Fixture::nueva("config-contrato");
    let (_bare, clon) = fx.remoto_y_clon();
    let i = config::de_base(&fx.git(), &clon).unwrap_err();
    comun::contrato(&i);
    let s = i.salida(false);
    assert_eq!(
        (s.codigo, s.stdout.as_str()),
        (
            3,
            "config rechazada motivo=sin-config errores=1
"
        )
    );
    assert!(s.stderr.contains("tc2 init"), "{}", s.stderr);

    // La configuracion de la 0.16 (sin `---` ni `formato`).
    escribir(
        &clon,
        ".taskcode/config.yml",
        "modo_flujo: automatico\ncomando_verificacion: \"cargo test\"\n",
    );
    fx.crudo(&clon, &["add", "."]);
    fx.crudo(&clon, &["commit", "-q", "-m", "config 0.16"]);
    fx.crudo(&clon, &["push", "-q", "origin", "develop"]);
    fx.crudo(&clon, &["fetch", "-q", "origin"]);
    let s = config::de_base(&fx.git(), &clon).unwrap_err().salida(false);
    assert_eq!(
        (s.codigo, s.stdout.as_str()),
        (
            3,
            "config rechazada motivo=formato errores=1
"
        )
    );
    assert!(s.stderr.contains("tc2 actualizar"), "{}", s.stderr);

    // Esquema: todas las rutas en la linea y el arreglo en stderr.
    let i = config::leer(&format!(
        "{CABECERA}comando_verificacion: 'x'\ntope_rondas: 11\n"
    ))
    .unwrap_err()
    .informe();
    comun::contrato(&i);
    assert_eq!(
        i.linea(),
        "config rechazada motivo=config-invalida errores=2 claves=comando_verificacion,tope_rondas"
    );
    let s = i.salida(true);
    assert!(s.stdout.contains("\"codigo\":3"), "{}", s.stdout);
    assert!(s.stdout.contains("tc2 actualizar"), "{}", s.stdout);

    // Sin origin/develop: rechazada, no un panico.
    let fx2 = Fixture::nueva("config-sin-base");
    fx2.crudo(&fx2.raiz, &["init", "-q", "-b", "develop", "solo"]);
    let solo = fx2.raiz.join("solo");
    let s = config::de_base(&fx2.git(), &solo)
        .unwrap_err()
        .salida(false);
    assert_eq!(
        (s.codigo, s.stdout.as_str()),
        (
            3,
            "config rechazada motivo=sin-base
"
        )
    );
}
