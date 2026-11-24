//! Los adaptadores reales de GitHub y GitLab contra el [`Mock`] para las
//! baterias de plataforma (TASK-139, A1-04 y A1-05): un solo constructor de
//! cada uno, el servido de un fixture y la tabla de errores HTTP. El token va
//! por parametro porque cada bateria comprueba que el suyo no se filtra. Los
//! dobles del trait estan en [`super::plataforma`].

use std::fmt::Debug;

use super::http_mock::{Mock, Simulada};
use tc2::plataforma::http::Base;
use tc2::plataforma::{CredencialesFijas, GitHub, GitLab, Personas, Plataforma, Rol, Secreto};

/// Credenciales con `token` para cada uno de `roles`.
pub fn credenciales(token: &str, roles: &[Rol]) -> Box<CredencialesFijas> {
    let mut c = CredencialesFijas::nuevas();
    for r in roles {
        c = c.con(*r, Secreto::nuevo(token));
    }
    Box::new(c)
}

/// GitHub para `o/r` en `base`.
pub fn github(base: &str, token: &str, roles: &[Rol], personas: Personas) -> GitHub {
    GitHub::nuevo(
        Base::nueva(base).unwrap(),
        "o/r",
        credenciales(token, roles),
        personas,
    )
    .unwrap()
}

/// GitLab para el proyecto 7 en `base` (sin `/api/v4`: se anade aqui).
pub fn gitlab(base: &str, token: &str, roles: &[Rol], personas: Personas) -> GitLab {
    GitLab::nuevo(
        Base::nueva(&format!("{base}/api/v4")).unwrap(),
        "7",
        credenciales(token, roles),
        personas,
    )
    .unwrap()
}

/// Un mock que sirve las `respuestas` (`GET`) de un fixture de
/// `tests/fixtures/plataforma`.
pub fn servir(f: &serde_json::Value) -> Mock {
    let mock = Mock::nuevo();
    for (ruta, r) in f["respuestas"].as_object().unwrap() {
        mock.responder(
            "GET",
            ruta,
            Simulada::json(
                r["estado"].as_u64().unwrap() as u16,
                &r["cuerpo"].to_string(),
            ),
        );
    }
    mock
}

/// La tabla de errores HTTP sobre `identidad` del agente: cada respuesta en
/// `GET ruta_usuario` da su linea, su codigo y un diagnostico con el
/// `arreglo`, sin caracteres de control de la API y sin `token` en ningun
/// sitio. `hacer` construye el adaptador con token de agente.
pub fn tabla_de_errores<P: Plataforma + Debug>(
    ruta_usuario: &str,
    token: &str,
    casos: Vec<(Simulada, &str, u8, &str)>,
    hacer: impl Fn(&Mock) -> P,
) {
    for (respuesta, linea, codigo, arreglo) in casos {
        let mock = Mock::nuevo();
        mock.responder("GET", ruta_usuario, respuesta);
        let p = hacer(&mock);
        let i = p.identidad(Rol::Agente).unwrap_err();
        let s = i.salida(false);
        assert_eq!((s.stdout.trim_end(), s.codigo), (linea, codigo), "{linea}");
        assert!(s.stderr.contains(arreglo), "{linea}: {}", s.stderr);
        assert!(
            !s.stderr.contains('\u{7}'),
            "{linea}: sin controles de la API"
        );
        for texto in [
            s.stdout,
            s.stderr,
            i.salida(true).stdout,
            format!("{p:?} {i:?}"),
        ] {
            assert!(!texto.contains(token), "{linea}: el token no sale: {texto}");
        }
    }
}
