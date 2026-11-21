# Peticion de revision — TASK-045 (ronda 1)

- Tarea: TASK-045 — F5-5 Runner self-hosted Windows por perfil
- Rama revisada: feature/task-045-f5-5-runner-self-hosted-windows-por-perf
- Rama base: develop
- Commit revisado (HEAD): addfcb2da94056304689989a9516567c77a7c466
- Fecha: 2026-11-21
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-045 (criterios de aceptacion y plan)
- Alcance: completa (motivo: primera ronda de revision)

## Instrucciones para el agente revisor

Eres un revisor INDEPENDIENTE del agente que implemento. Tu trabajo es
reproducir empiricamente, no leer el diff y opinar: clona el repo a un
directorio temporal, corre la suite tu mismo (salvo que esta
peticion ya traiga la verificacion de la ronda en verde para el commit revisado;
entonces vale ese resultado) y construye el caso que
rompe el codigo antes de reportarlo. Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. Vuelca tu salida en el
informe de esta ronda (informe-revision-1-rust-wasm-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-045/tarea.md
- tareas/03-en-revision/TASK-045/planificacion/
- xtask/src/workflows.rs

## Commits a revisar (git log develop..HEAD)

````
addfcb2 fix(TASK-045): huella local por la entrada de sha256sum
a3c934e fix(TASK-045): el runner propio exige jq y la guia da el alta real
c471af0 style(TASK-045): formato del shell del aislamiento
20484ad fix(TASK-045): aislamiento con ExecutionPolicy Bypass en el runner propio
2d49e6e feat(TASK-045): guia del runner propio y adenda F5-5 en F1-8
ff4227d feat(TASK-045): whoami por ruta y casos de base y evento sueltos
bc186bd feat(TASK-045): runner propio solo en la release y aislamiento en ejecutar
d703964 chore(TASK-045): tarea en curso
````

## Diff de tu dominio (1 fichero(s) de 4; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/xtask/src/workflows.rs b/xtask/src/workflows.rs
index 29b0a32..6769bd3 100644
--- a/xtask/src/workflows.rs
+++ b/xtask/src/workflows.rs
@@ -11,7 +11,8 @@
 //!   `read-all`; un job con algun `write` es de clase privilegio; solo se
 //!   leen `contents`, `checks` y `actions` (estos dos, para `buscar` de F5-2);
 //! - acciones admitidas: `actions/checkout`, fijada por SHA, con
-//!   `persist-credentials: false` y nunca en clase privilegio; y
+//!   `persist-credentials: false`, sin `clean:` distinto de `true` (F5-5) y
+//!   nunca en clase privilegio; y
 //!   `actions/upload-artifact` (informe de F5-4), fijada por SHA y solo en
 //!   clase ejecuta;
 //! - ni `secrets: inherit` (salvo el llamador de `revision-ia.yml`, perfil
@@ -653,6 +654,11 @@ fn lint_con(texto: &str, perfil: &Perfil) -> Vec<String> {
                     "job {nombre}: checkout sin persist-credentials: false"
                 ));
             }
+            // F1-8, decision 6 (D3b b), adenda F5-5: el `clean: true` por
+            // defecto deja el directorio limpio en cada job del runner propio.
+            if opcion("clean").is_some_and(|c| c.texto() != Some("true")) {
+                errores.push(format!("job {nombre}: checkout sin clean: true"));
+            }
             // F1-8: un job con privilegio nunca hace checkout (en un PR, sin
             // `ref:` saca el merge ref, que es codigo del PR).
             if privilegio {
@@ -1075,6 +1081,329 @@ fn la_cola_de_merge_elige_los_perfiles_del_evento() {
     );
 }
 
+// --- Runner propio (F5-5) ----------------------------------------------------------
+
+/// Ejecuta con `bash` el `run:` de «Runner propio» extraido de `texto`, con el
+/// entorno del paso simulado; devuelve lo que imprime o el codigo de salida y
+/// el error.
+fn runner_propio(texto: &str, entorno: &[(&str, &str)]) -> Result<String, String> {
+    let script = run_del_paso(texto, "buscar", "Runner propio");
+    let proceso = std::process::Command::new(ruta_bash())
+        .arg("-c")
+        .arg(&script)
+        .envs(entorno.iter().copied())
+        .output()
+        .unwrap_or_else(|e| panic!("bash no arranca (hace falta bash en el PATH): {e}"));
+    if proceso.status.success() {
+        Ok(String::from_utf8_lossy(&proceso.stdout).trim().to_owned())
+    } else {
+        Err(format!(
+            "{:?}: {}",
+            proceso.status.code(),
+            String::from_utf8_lossy(&proceso.stderr).trim()
+        ))
+    }
+}
+
+/// Entradas del workflow con el runner de la pesada dado (JSON de runs-on) y
+/// la completa en un runner de GitHub.
+fn entradas_con_pesada(runner: &str) -> String {
+    format!(
+        r#"{{"runner-completa":"\"ubuntu-latest\"","runner-pesada":{}}}"#,
+        cadena_json(runner)
+    )
+}
+
+/// `texto` como cadena JSON (comillas y barras escapadas): basta para los
+/// runs-on de estas pruebas.
+fn cadena_json(texto: &str) -> String {
+    format!("\"{}\"", texto.replace('\\', "\\\\").replace('"', "\\\""))
+}
+
+/// F1-8, decision 6 (D3b), y adenda F5-5: el runner propio solo en la release
+/// de develop a main de un repo privado con el head en el mismo repo; en
+/// cualquier otro caso `buscar` falla cerrado con exit 2 y su motivo. Un
+/// perfil en un runner de GitHub no se ve afectado en ningun evento. Se
+/// ejecuta el `run:` real del workflow.
+#[test]
+fn el_runner_propio_solo_en_la_release_de_un_privado() {
+    let texto = leer(WORKFLOW);
+    let release: [(&str, &str); 8] = [
+        ("PERFILES", r#"["completa","pesada"]"#),
+        ("ENTRADAS", ""),
+        ("EVENTO", "pull_request"),
+        ("PRIVADO", "true"),
+        ("BASE", "main"),
+        ("ORIGEN", "develop"),
+        ("REPO", "o/r"),
+        ("REPO_HEAD", "o/r"),
+    ];
+    fn cambiar<'a>(
+        base: &[(&'a str, &'a str)],
+        cambios: &[(&'a str, &'a str)],
+    ) -> Vec<(&'a str, &'a str)> {
+        let mut e = base.to_vec();
+        for &(k, v) in cambios {
+            if let Some(par) = e.iter_mut().find(|(x, _)| *x == k) {
+                par.1 = v;
+            }
+        }
+        e
+    }
+    let propio = entradas_con_pesada(r#"["self-hosted","windows","tc2-e2e"]"#);
+    let github = entradas_con_pesada(r#""ubuntu-latest""#);
+    let malos = [
+        ("repo publico", vec![("PRIVADO", "false")], "repo publico"),
+        (
+            "PR a develop",
+            vec![
+                ("PERFILES", r#"["pesada"]"#),
+                ("BASE", "develop"),
+                ("ORIGEN", "feature/x"),
+            ],
+            "solo en la release",
+        ),
+        (
+            "workflow_dispatch",
+            vec![
+                ("PERFILES", r#"["pesada"]"#),
+                ("EVENTO", "workflow_dispatch"),
+                ("BASE", ""),
+                ("ORIGEN", ""),
+                ("REPO_HEAD", ""),
+            ],
+            "solo en la release",
+        ),
+        (
+            "merge_group",
+            vec![
+                ("PERFILES", r#"["pesada"]"#),
+                ("EVENTO", "merge_group"),
+                ("BASE", ""),
+                ("ORIGEN", ""),
+                ("REPO_HEAD", ""),
+            ],
+            "solo en la release",
+        ),
+        (
+            "head de un fork",
+            vec![("REPO_HEAD", "otro/r")],
+            "viene de otro repo (otro/r)",
+        ),
+        (
+            "hotfix",
+            vec![("PERFILES", r#"["pesada"]"#), ("ORIGEN", "hotfix/x")],
+            "solo en la release",
+        ),
+        (
+            "develop a otra rama",
+            vec![("PERFILES", r#"["pesada"]"#), ("BASE", "release")],
+            "solo en la release",
+        ),
+        // Cada condicion por separado: con base y origen de la release, otro
+        // evento (con `perfiles` forzados) tampoco pasa.
+        (
+            "otro evento con base y origen de la release",
+            vec![("PERFILES", r#"["pesada"]"#), ("EVENTO", "merge_group")],
+            "solo en la release",
+        ),
+    ];
+    // La release de un privado del mismo repo pasa; la etiqueta no distingue
+    // mayusculas, y un runs-on que no es cadena ni lista (grupo) cuenta como
+    // runner propio.
+    for runner in [
+        r#"["self-hosted","windows","tc2-e2e"]"#,
+        r#"["Self-Hosted","windows"]"#,
+        r#""self-hosted""#,
+    ] {
+        let entradas = entradas_con_pesada(runner);
+        let salida = runner_propio(&texto, &cambiar(&release, &[("ENTRADAS", &entradas)]));
+        assert_eq!(
+            salida,
+            Ok("Runner propio en pesada: release de develop a main en o/r (privado)".to_owned()),
+            "{runner}"
+        );
+    }
+    for (caso, cambios, motivo) in &malos {
+        let mut e = cambios.clone();
+        e.push(("ENTRADAS", &propio));
+        let salida = runner_propio(&texto, &cambiar(&release, &e));
+        assert!(
+            matches!(&salida, Err(m) if m.starts_with("Some(2): ") && m.contains(motivo) && m.contains("runner propio en pesada")),
+            "{caso}: {salida:?}"
+        );
+        // Con la pesada en un runner de GitHub, el mismo evento sigue igual.
+        let mut e = cambios.clone();
+        e.push(("ENTRADAS", &github));
+        assert_eq!(
+            runner_propio(&texto, &cambiar(&release, &e)),
+            Ok("Runner propio: ningun perfil de este evento lo usa".to_owned()),
+            "{caso} con runner de GitHub"
+        );
+    }
+    let grupo = entradas_con_pesada(r#"{"group":"propios"}"#);
+    assert!(
+        matches!(runner_propio(&texto, &cambiar(&release, &[("ENTRADAS", &grupo), ("PRIVADO", "false")])), Err(m) if m.contains("repo publico")),
+        "un grupo de runners cuenta como runner propio"
+    );
+    let ilegible = entradas_con_pesada("no-es-json");
+    assert!(
+        matches!(runner_propio(&texto, &cambiar(&release, &[("ENTRADAS", &ilegible)])), Err(m) if m.starts_with("Some(2): ") && m.contains("runner-pesada ilegible")),
+        "un runs-on ilegible falla cerrado"
+    );
+    // Sin perfiles (push) no hay nada que mirar.
+    assert_eq!(
+        runner_propio(
+            &texto,
+            &cambiar(
+                &release,
+                &[
+                    ("PERFILES", "[]"),
+                    ("EVENTO", "push"),
+                    ("ENTRADAS", &propio)
+                ]
+            )
+        ),
+        Ok("Runner propio: ningun perfil de este evento lo usa".to_owned())
+    );
+}
+
+/// Windows PowerShell sin perfil y sin la directiva de la maquina, solo para el
+/// proceso del paso: con `AllSigned` (E2E de TASK-045) el `.ps1` sin firmar que
+/// genera el runner no llega a ejecutarse y el aislamiento no comprueba nada.
+const SHELL_AISLAMIENTO: &str =
+    r#"powershell -NoProfile -ExecutionPolicy Bypass -Command ". '{0}'""#;
+
+/// El paso de aislamiento es el primero de `ejecutar` (antes del checkout),
+/// solo en un runner propio y en Windows PowerShell; `Err` con lo que falla.
+fn aislamiento_antes_del_checkout(texto: &str) -> Result<(), String> {
+    let raiz = yaml(texto);
+    let Some(Nodo::Lista(pasos)) = raiz
+        .get("jobs")
+        .and_then(|j| j.get("ejecutar"))
+        .and_then(|j| j.get("steps"))
+    else {
+        return Err("ejecutar sin pasos".to_owned());
+    };
+    let primero = pasos.first().ok_or("ejecutar sin pasos")?;
+    let campo = |k: &str| primero.get(k).and_then(Nodo::texto);
+    if campo("name") != Some("aislamiento del runner propio") {
+        return Err(format!(
+            "el primer paso de ejecutar no es el aislamiento: {:?}",
+            campo("name").or(campo("uses"))
+        ));
+    }
+    if campo("if") != Some("runner.environment == 'self-hosted'") {
+        return Err(format!("aislamiento con if {:?}", campo("if")));
+    }
+    if campo("shell") != Some(SHELL_AISLAMIENTO) {
+        return Err(format!("aislamiento con shell {:?}", campo("shell")));
+    }
+    let run = campo("run").unwrap_or_default();
+    // whoami por ruta: el de Git (usr\bin) en el PATH no entiende /groups.
+    for exigido in [
+        "S-1-5-18",
+        "S-1-5-32-544",
+        r#"System32\whoami.exe" /groups"#,
+        "RUNNER_OS",
+        "Get-Command jq",
+    ] {
+        if !run.contains(exigido) {
+            return Err(format!("aislamiento sin {exigido}"));
+        }
+    }
+    Ok(())
+}
+
+/// F1-8, decision 6 (D3b a): en un runner propio, antes de sacar codigo, se
+/// comprueba que es Windows y que la cuenta no es SYSTEM ni administradora.
+/// Quitar el paso, ponerlo despues del checkout o cambiar su condicion o su
+/// shell rompe la prueba.
+#[test]
+fn el_aislamiento_va_antes_del_checkout_en_el_runner_propio() {
+    let texto = leer(WORKFLOW);
+    assert_eq!(aislamiento_antes_del_checkout(&texto), Ok(()));
+    let inicio = texto
+        .find("      - name: aislamiento del runner propio\n")
+        .unwrap_or_else(|| panic!("sin paso de aislamiento"));
+    let checkout = "      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1\n        with:\n          persist-credentials: false\n";
+    let fin = inicio
+        + texto[inicio..]
+            .find(checkout)
+            .unwrap_or_else(|| panic!("sin checkout tras el aislamiento"));
+    let paso = &texto[inicio..fin];
+    let quitado = format!("{}{}", &texto[..inicio], &texto[fin..]);
+    let movido = format!(
+        "{}{checkout}{paso}{}",
+        &texto[..inicio],
+        &texto[fin + checkout.len()..]
+    );
+    let mutaciones = [
+        ("quitado", quitado),
+        ("despues del checkout", movido),
+        (
+            "en cualquier runner",
+            texto.replacen(
+                "if: runner.environment == 'self-hosted'",
+                "if: runner.os == 'Windows'",
+                1,
+            ),
+        ),
+        (
+            "en bash",
+            texto.replacen(
+                &format!("        shell: {SHELL_AISLAMIENTO}\n"),
+                "        shell: bash\n",
+                1,
+            ),
+        ),
+        (
+            "con la directiva de la maquina",
+            texto.replacen(
+                &format!("        shell: {SHELL_AISLAMIENTO}\n"),
+                "        shell: powershell\n",
+                1,
+            ),
+        ),
+        (
+            "sin comprobar jq",
+            texto.replacen("Get-Command jq", "Get-Command bash", 1),
+        ),
+        (
+            "sin el grupo Administradores",
+            texto.replacen("$grupos -match 'S-1-5-32-544'", "$false", 1),
+        ),
+    ];
+    for (caso, mutado) in &mutaciones {
+        assert_ne!(mutado, &texto, "{caso}: la mutacion no casa");
+        assert!(aislamiento_antes_del_checkout(mutado).is_err(), "{caso}");
+    }
+}
+
+/// E2E de TASK-045: con una ruta de Windows como argumento, GNU `sha256sum`
+/// escapa la barra invertida y antepone `\` a la linea, asi que la huella local
+/// de la pesada en el runner propio no casaria nunca con la cache. El workflow
+/// solo resume por la entrada estandar.
+#[test]
+fn sha256sum_solo_por_la_entrada_estandar() {
+    let con_ruta = |texto: &str| {
+        texto.lines().any(|l| {
+            l.split("sha256sum")
+                .skip(1)
+                .any(|resto| resto.trim_start().starts_with('"'))
+        })
+    };
+    let texto = leer(WORKFLOW);
+    assert!(
+        texto.contains("sha256sum < \"$HUELLA\""),
+        "sin la huella local"
+    );
+    assert!(!con_ruta(&texto), "sha256sum con una ruta como argumento");
+    let mutado = texto.replacen("sha256sum < \"$HUELLA\"", "sha256sum \"$HUELLA\"", 1);
+    assert_ne!(mutado, texto, "la mutacion no casa");
+    assert!(con_ruta(&mutado), "la mutacion no se detecta");
+}
+
 /// D3: la impacto nunca se cachea. `buscar` no le calcula clave (sin clave no
 /// hay candidatos) y el publicador no le escribe linea `arbol=`; el resumen
 /// lleva `cola-de-merge=`. La entrada es booleana y falsa por defecto en las
@@ -1207,7 +1536,7 @@ fn el_caso_positivo_pasa() {
 
 #[test]
 fn cada_mutacion_prohibida_da_su_error() {
-    let casos: [(&str, &str, &str); 11] = [
+    let casos: [(&str, &str, &str); 13] = [
         (
             "  pull_request:\n",
             "  pull_request_target:\n",
@@ -1249,6 +1578,16 @@ fn cada_mutacion_prohibida_da_su_error() {
             "          fetch-depth: 0\n",
             "persist-credentials",
         ),
+        (
+            "          persist-credentials: false\n",
+            "          persist-credentials: false\n          clean: false\n",
+            "checkout sin clean: true",
+        ),
+        (
+            "          persist-credentials: false\n",
+            "          persist-credentials: false\n          clean: ${{ inputs.limpiar }}\n",
+            "checkout sin clean: true",
+        ),
         (
             "      - env:\n          GH_TOKEN",
             "      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1\n        with:\n          persist-credentials: false\n          ref: ${{ github.event.pull_request.head.sha }}\n      - env:\n          GH_TOKEN",
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-045/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-045/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-045/tarea.md                                    | 3 ++-
 3 files changed, 2 insertions(+), 1 deletion(-)
````
