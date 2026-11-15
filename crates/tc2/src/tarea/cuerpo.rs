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
