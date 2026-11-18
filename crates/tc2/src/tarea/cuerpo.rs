//! Cuerpo Markdown de `tarea.md` por secciones (ADR F1-9, punto 4): se
//! reconocen por su titulo, no por su posicion, y se conservan byte a byte.

/// Cuerpo de la plantilla de una tarea recien creada (F1-9, punto 6).
const PLANTILLA: &str = "## Objetivo

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
";

/// Hueco del objetivo en la plantilla.
const OBJETIVO_PLANTILLA: &str = "<que se quiere y por que>";
/// Hueco del criterio en la plantilla.
const CRITERIO_PLANTILLA: &str = "- [ ] <criterio comprobable>";

/// Una linea de texto libre que no abre seccion ni cerca: si empieza (con
/// hasta 3 espacios) por `#`, `` ` `` o `~`, se le antepone `\`.
fn escapar_linea(linea: &str) -> String {
    let sin = linea.trim_start_matches(' ');
    let sangria = linea.len() - sin.len();
    if sangria <= 3 && sin.starts_with(['#', '`', '~']) {
        format!("{}\\{sin}", &linea[..sangria])
    } else {
        linea.to_owned()
    }
}

/// Una seccion `## <titulo>` con su texto, incluida la linea del titulo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seccion {
    /// Titulo sin `## ` ni espacios al final.
    pub titulo: String,
    /// Texto tal cual, desde la linea `## ` hasta la siguiente seccion.
    pub texto: String,
}

/// Cuerpo de una tarea: lo que va antes de la primera seccion y las
/// secciones en su orden. Escribirlo es concatenar lo leido.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cuerpo {
    /// Texto antes de la primera seccion (normalmente vacio).
    pub preambulo: String,
    /// Secciones en el orden del fichero.
    pub secciones: Vec<Seccion>,
}

/// Cerca de codigo de `linea` (CommonMark 4.5): hasta 3 espacios de
/// sangria y 3 o mas `` ` `` o `~` seguidos. Devuelve el caracter, cuantos
/// lleva y el resto de la linea.
fn cerca(linea: &str) -> Option<(char, usize, &str)> {
    let sin_sangria = linea.trim_start_matches(' ');
    if linea.len() - sin_sangria.len() > 3 {
        return None;
    }
    let c = sin_sangria
        .chars()
        .next()
        .filter(|c| matches!(c, '`' | '~'))?;
    let n = sin_sangria.chars().take_while(|x| *x == c).count();
    (n >= 3).then(|| (c, n, &sin_sangria[n..]))
}

/// Si `linea` abre un bloque: en una cerca de `` ` `` la informacion que
/// sigue no puede llevar `` ` ``.
fn abre(linea: &str) -> Option<(char, usize)> {
    let (c, n, resto) = cerca(linea)?;
    (c == '~' || !resto.contains('`')).then_some((c, n))
}

/// Si `linea` cierra el bloque abierto con `abierta`: el mismo caracter, al
/// menos tantos y nada mas que espacios detras.
fn cierra(linea: &str, abierta: (char, usize)) -> bool {
    cerca(linea)
        .is_some_and(|(c, n, resto)| c == abierta.0 && n >= abierta.1 && resto.trim().is_empty())
}

/// Titulo de `linea` si es un encabezado de seccion `## ` (no `###`).
fn titulo(linea: &str) -> Option<String> {
    let resto = linea.strip_prefix("## ")?;
    Some(resto.trim_end().to_owned())
}

impl Cuerpo {
    /// Parte `texto` en secciones. Un `## ` dentro de un bloque de codigo
    /// cercado no abre seccion.
    pub fn leer(texto: &str) -> Self {
        let mut cuerpo = Self::default();
        let mut abierta: Option<(char, usize)> = None;
        for linea in texto.split_inclusive('\n') {
            let encabezado = match abierta {
                Some(a) => {
                    if cierra(linea, a) {
                        abierta = None;
                    }
                    None
                }
                None => {
                    abierta = abre(linea);
                    titulo(linea)
                }
            };
            match (encabezado, cuerpo.secciones.last_mut()) {
                (Some(t), _) => cuerpo.secciones.push(Seccion {
                    titulo: t,
                    texto: linea.to_owned(),
                }),
                (None, Some(s)) => s.texto.push_str(linea),
                (None, None) => cuerpo.preambulo.push_str(linea),
            }
        }
        cuerpo
    }

    /// El cuerpo de una tarea recien creada.
    pub fn plantilla() -> Self {
        Self::leer(PLANTILLA)
    }

    /// La plantilla con el enunciado puesto (RF-01): `objetivo` en
    /// `## Objetivo` y un `- [ ] <criterio>` por criterio. Lo que venga vacio
    /// se queda con el texto de la plantilla. Una linea del objetivo que
    /// abriria una seccion o una cerca de codigo se escapa con `\`: el texto
    /// es dato no confiable y no puede cambiar la forma del cuerpo (F1-8).
    pub fn con_enunciado(objetivo: &str, criterios: &[String]) -> Self {
        // Se parte la plantilla por sus huecos antes de meter nada: un texto
        // que contenga el hueco del criterio no desvia la sustitucion.
        let partes = PLANTILLA
            .split_once(OBJETIVO_PLANTILLA)
            .and_then(|(a, resto)| resto.split_once(CRITERIO_PLANTILLA).map(|(b, c)| (a, b, c)));
        let Some((antes, entre, despues)) = partes else {
            return Self::plantilla();
        };
        let objetivo = if objetivo.trim().is_empty() {
            OBJETIVO_PLANTILLA.to_owned()
        } else {
            let lineas: Vec<String> = objetivo.trim().lines().map(escapar_linea).collect();
            lineas.join("\n")
        };
        let lista = if criterios.is_empty() {
            CRITERIO_PLANTILLA.to_owned()
        } else {
            let v: Vec<String> = criterios
                .iter()
                .map(|c| format!("- [ ] {}", c.trim()))
                .collect();
            v.join("\n")
        };
        Self::leer(&format!("{antes}{objetivo}{entre}{lista}{despues}"))
    }

    /// Texto del cuerpo: lo leido, byte a byte.
    pub fn escribir(&self) -> String {
        let mut s = self.preambulo.clone();
        for seccion in &self.secciones {
            s.push_str(&seccion.texto);
        }
        s
    }

    /// La primera seccion con ese titulo.
    pub fn seccion(&self, titulo: &str) -> Option<&Seccion> {
        self.secciones.iter().find(|s| s.titulo == titulo)
    }

    /// Anade una fila a la tabla de `## Transiciones` (F1-9: la escribe el
    /// CLI en cada transicion), justo detras de su ultima fila; el resto del
    /// cuerpo queda byte a byte. Sin tabla, la crea con su cabecera al final
    /// de la seccion; sin seccion, la crea al final del cuerpo.
    pub fn anadir_transicion(&mut self, t: &Transicion<'_>) {
        let fila = format!(
            "| {} | {} | {} | {} | {} |\n",
            t.fecha, t.fase, t.modo, t.decidido_por, t.version
        );
        if let Some(s) = self.secciones.iter_mut().find(|s| s.titulo == TRANSICIONES) {
            let lineas: Vec<&str> = s.texto.split_inclusive('\n').collect();
            let texto = match lineas.iter().rposition(|l| l.trim_start().starts_with('|')) {
                Some(i) => {
                    let mut antes = lineas[..=i].concat();
                    if !antes.ends_with('\n') {
                        antes.push('\n');
                    }
                    format!("{antes}{fila}{}", lineas[i + 1..].concat())
                }
                None => {
                    let mut t = s.texto.clone();
                    separar(&mut t);
                    format!("{t}{CABECERA_TRANSICIONES}{fila}")
                }
            };
            s.texto = texto;
            return;
        }
        match self.secciones.last_mut() {
            Some(s) => separar(&mut s.texto),
            None => separar(&mut self.preambulo),
        }
        self.secciones.push(Seccion {
            titulo: TRANSICIONES.to_owned(),
            texto: format!("## {TRANSICIONES}\n\n{CABECERA_TRANSICIONES}{fila}"),
        });
    }
}

impl Cuerpo {
    /// Si la tarea esta en pausa (D1 de TASK-034): de las filas de
    /// `## Transiciones` con fase [`PAUSA`] o [`REANUDAR`], la ultima es
    /// `pausa`. Sin ninguna de las dos, no. La pausa no es un estado (F1-9
    /// cierra la lista): son estas filas, las que F1-17 descuenta.
    pub fn en_pausa(&self) -> bool {
        let Some(s) = self.seccion(TRANSICIONES) else {
            return false;
        };
        s.texto
            .lines()
            .rev()
            .filter_map(|l| l.trim().strip_prefix('|')?.split('|').nth(1))
            .map(str::trim)
            .find(|fase| *fase == PAUSA || *fase == REANUDAR)
            == Some(PAUSA)
    }

    /// Instante, en segundos desde 1970 (UTC), de la ultima fila de
    /// `## Transiciones` cuya fecha se entiende (F3-10, D2a: la actividad de
    /// la tarea; cuentan tambien `pausa` y `reanudar`). Sin seccion o sin
    /// ninguna fila fechada, `None`. La cabecera y el separador no son
    /// fechas y se saltan.
    pub fn ultima_fecha(&self) -> Option<u64> {
        let s = self.seccion(TRANSICIONES)?;
        s.texto
            .lines()
            .rev()
            .filter_map(|l| l.trim().strip_prefix('|')?.split('|').next())
            .find_map(|fecha| crate::inventario::ms_de_iso(fecha.trim()))
            .and_then(|ms| u64::try_from(ms / 1000).ok())
    }
}

/// Fase de la fila que pone una tarea en pausa (`tc2 pausa`).
pub const PAUSA: &str = "pausa";

/// Fase de la fila que la saca de la pausa (`tc2 reanudar`).
pub const REANUDAR: &str = "reanudar";

/// Titulo de la seccion de transiciones.
const TRANSICIONES: &str = "Transiciones";

/// Cabecera de la tabla de transiciones (plantilla de F1-9, punto 6).
const CABECERA_TRANSICIONES: &str =
    "| fecha | fase | modo | decidido_por | version |\n|---|---|---|---|---|\n";

/// Deja `texto` terminado en una linea en blanco (si no esta vacio), para
/// que lo que se anada detras empiece un bloque nuevo.
fn separar(texto: &mut String) {
    if texto.is_empty() {
        return;
    }
    if !texto.ends_with('\n') {
        texto.push('\n');
    }
    if !texto.ends_with("\n\n") {
        texto.push('\n');
    }
}

/// Una fila de `## Transiciones` (F1-9, punto 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transicion<'a> {
    /// Instante, `AAAA-MM-DDTHH:MM:SSZ` en UTC.
    pub fecha: &'a str,
    /// Fase que se entra (`start`, `review`...).
    pub fase: &'a str,
    /// Modo de la tarea (`manual`, `automatico`...).
    pub modo: &'a str,
    /// `persona` o `agente` (F2-13).
    pub decidido_por: &'a str,
    /// Version del CLI que la escribe.
    pub version: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secciones_por_titulo_y_byte_a_byte() {
        let t = "intro\n## Transiciones\r\n| a |\n## Resultado \nhecho\n```md\n## no es seccion\n```\n### sub\n## Objetivo\nx";
        let c = Cuerpo::leer(t);
        assert_eq!(c.preambulo, "intro\n");
        let titulos: Vec<&str> = c.secciones.iter().map(|s| s.titulo.as_str()).collect();
        assert_eq!(titulos, ["Transiciones", "Resultado", "Objetivo"]);
        assert!(
            c.seccion("Resultado")
                .is_some_and(|s| s.texto.contains("## no es seccion"))
        );
        assert!(c.seccion("Escenarios").is_none());
        assert_eq!(c.escribir(), t);
        let p = Cuerpo::plantilla();
        assert_eq!(p.escribir(), PLANTILLA);
        assert_eq!(p.secciones.len(), 5);
    }

    #[test]
    fn enunciado_en_la_plantilla_sin_romper_las_secciones() {
        assert_eq!(Cuerpo::con_enunciado("", &[]), Cuerpo::plantilla());
        let c = Cuerpo::con_enunciado(
            "Primera linea\n## no es seccion\n```\n  # tampoco",
            &["uno".to_owned(), " dos ".to_owned()],
        );
        let titulos: Vec<&str> = c.secciones.iter().map(|s| s.titulo.as_str()).collect();
        assert_eq!(
            titulos,
            [
                "Objetivo",
                "Criterios de aceptacion",
                "Escenarios",
                "Resultado",
                "Transiciones"
            ]
        );
        let objetivo = c.seccion("Objetivo").map(|s| s.texto.as_str());
        assert_eq!(
            objetivo,
            Some("## Objetivo\n\nPrimera linea\n\\## no es seccion\n\\```\n  \\# tampoco\n\n")
        );
        let criterios = c
            .seccion("Criterios de aceptacion")
            .map(|s| s.texto.as_str());
        assert_eq!(
            criterios,
            Some("## Criterios de aceptacion\n\n- [ ] uno\n- [ ] dos\n\n")
        );
        assert_eq!(escapar_linea("    # codigo"), "    # codigo");
        // Un objetivo con el texto del hueco del criterio no lo desvia.
        let c = Cuerpo::con_enunciado(
            "ver - [ ] <criterio comprobable> aqui",
            &["real".to_owned()],
        );
        let t = c.escribir();
        assert!(
            t.contains("\n\nver - [ ] <criterio comprobable> aqui\n\n"),
            "{t}"
        );
        assert!(t.contains("\n\n- [ ] real\n\n"), "{t}");
    }

    const T: Transicion<'static> = Transicion {
        fecha: "2026-11-17T10:00:00Z",
        fase: "start",
        modo: "manual",
        decidido_por: "agente",
        version: "0.1.0",
    };
    const FILA: &str = "| 2026-11-17T10:00:00Z | start | manual | agente | 0.1.0 |\n";

    /// F3-6: la fila va detras de la ultima de la tabla y el resto del
    /// cuerpo queda byte a byte (tambien lo que sigue a la tabla).
    #[test]
    fn anadir_transicion_conserva_el_resto() {
        let mut p = Cuerpo::plantilla();
        p.anadir_transicion(&T);
        assert_eq!(p.escribir(), format!("{PLANTILLA}{FILA}"));
        // Con filas previas, texto detras y otra seccion despues.
        let t = "## Objetivo\n\nx\n\n## Transiciones\n\n| fecha | fase | modo | decidido_por | version |\n|---|---|---|---|---|\n| a | plan | manual | persona | 0.1.0 |\n\nnota final\n## Otra\r\ny";
        let mut c = Cuerpo::leer(t);
        c.anadir_transicion(&T);
        let esperado = t.replace(
            "| a | plan | manual | persona | 0.1.0 |\n",
            &format!("| a | plan | manual | persona | 0.1.0 |\n{FILA}"),
        );
        assert_eq!(c.escribir(), esperado);
        // Ultima fila sin salto final.
        let mut c = Cuerpo::leer("## Transiciones\n|---|");
        c.anadir_transicion(&T);
        assert_eq!(c.escribir(), format!("## Transiciones\n|---|\n{FILA}"));
    }

    /// Sin tabla se crea con su cabecera; sin seccion, al final del cuerpo.
    #[test]
    fn anadir_transicion_sin_tabla_o_sin_seccion() {
        let mut c = Cuerpo::leer("## Transiciones\ntexto");
        c.anadir_transicion(&T);
        assert_eq!(
            c.escribir(),
            format!("## Transiciones\ntexto\n\n{CABECERA_TRANSICIONES}{FILA}")
        );
        let mut c = Cuerpo::leer("## Objetivo\nx");
        c.anadir_transicion(&T);
        assert_eq!(
            c.escribir(),
            format!("## Objetivo\nx\n\n## Transiciones\n\n{CABECERA_TRANSICIONES}{FILA}")
        );
        assert_eq!(
            c.seccion("Transiciones").map(|s| s.titulo.as_str()),
            Some("Transiciones")
        );
        let mut c = Cuerpo::leer("");
        c.anadir_transicion(&T);
        assert_eq!(
            c.escribir(),
            format!("## Transiciones\n\n{CABECERA_TRANSICIONES}{FILA}")
        );
        // Lo escrito se vuelve a leer igual.
        assert_eq!(Cuerpo::leer(&c.escribir()), c);
    }

    /// TASK-034 (C9 y C10): en pausa si la ultima fila `pausa`/`reanudar`
    /// es `pausa`; las demas filas no cuentan, ni una `pausa` fuera de la
    /// seccion.
    #[test]
    fn en_pausa_por_la_ultima_fila_de_pausa_o_reanudar() {
        let fila = |fase: &'static str| Transicion { fase, ..T };
        let mut c = Cuerpo::plantilla();
        assert!(!c.en_pausa());
        c.anadir_transicion(&fila(PAUSA));
        assert!(c.en_pausa());
        // Otra transicion detras no la saca de la pausa.
        c.anadir_transicion(&fila("start"));
        assert!(c.en_pausa());
        c.anadir_transicion(&fila(REANUDAR));
        assert!(!c.en_pausa());
        c.anadir_transicion(&fila(PAUSA));
        assert!(c.en_pausa());
        // Una «pausa» en otra seccion o fuera de la columna de la fase, no.
        let c = Cuerpo::leer(
            "## Objetivo\n| x | pausa | y |\n## Transiciones\n| fecha | fase |\n|---|---|\n| a | reanudar | pausa |\n",
        );
        assert!(!c.en_pausa());
        assert!(!Cuerpo::leer("").en_pausa());
    }

    /// TASK-081 (D2a): la ultima fila fechada de `## Transiciones`, tambien
    /// una `pausa`; sin filas, ni cabecera ni separador cuentan; una fila de
    /// otra seccion, tampoco.
    #[test]
    fn ultima_fecha_de_las_transiciones() {
        let mut c = Cuerpo::plantilla();
        assert_eq!(c.ultima_fecha(), None);
        c.anadir_transicion(&T);
        // 2026-11-17T10:00:00Z.
        assert_eq!(c.ultima_fecha(), Some(1_794_909_600));
        c.anadir_transicion(&Transicion {
            fecha: "2026-11-20T00:00:01Z",
            fase: PAUSA,
            ..T
        });
        assert_eq!(c.ultima_fecha(), Some(1_795_132_801));
        // Una fila final que no se entiende no tapa la anterior.
        let c = Cuerpo::leer(&format!("{}| basura | x |\n", c.escribir()));
        assert_eq!(c.ultima_fecha(), Some(1_795_132_801));
        let c = Cuerpo::leer(
            "## Objetivo\n| 2030-01-01T00:00:00Z | start |\n## Transiciones\n| fecha | fase |\n|---|---|\n",
        );
        assert_eq!(c.ultima_fecha(), None);
        assert_eq!(Cuerpo::leer("").ultima_fecha(), None);
    }

    #[test]
    fn cercas_como_commonmark() {
        let titulos = |t: &str| -> Vec<String> {
            Cuerpo::leer(t)
                .secciones
                .into_iter()
                .map(|s| s.titulo)
                .collect()
        };
        // Cuatro acentos no se cierran con tres; `~~~` no cierra `` ``` ``.
        assert_eq!(titulos("## A\n````\n```\n## B\n````\n## C\n"), ["A", "C"]);
        // Se cierra con tantos o mas acentos que los de apertura.
        assert_eq!(titulos("## A\n````\n## B\n`````\n## C\n"), ["A", "C"]);
        assert_eq!(titulos("## A\n```\n~~~\n## B\n```\n## C\n"), ["A", "C"]);
        // `` ```x``` `` no abre (lleva acentos en la informacion), una
        // sangria de 4 no es cerca, y una cierre con texto detras no cierra.
        assert_eq!(titulos("## A\n```x```\n## B\n"), ["A", "B"]);
        assert_eq!(titulos("## A\n    ```\n## B\n"), ["A", "B"]);
        assert_eq!(
            titulos("## A\n~~~ md\n~~~ no\n## B\n~~~\n## C\n"),
            ["A", "C"]
        );
    }
}
