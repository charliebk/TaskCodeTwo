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

/// Si `linea` abre o cierra un bloque de codigo cercado.
fn es_cerca(linea: &str) -> bool {
    let l = linea.trim_start();
    l.starts_with("```") || l.starts_with("~~~")
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
        let mut en_codigo = false;
        for linea in texto.split_inclusive('\n') {
            let encabezado = if en_codigo { None } else { titulo(linea) };
            if es_cerca(linea) {
                en_codigo = !en_codigo;
            }
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
}
