//! Mensajes de `tc2 roadmap` (F3-4).

crate::i18n::textos! {
    USO_ROADMAP {
        es: "uso: roadmap [--comparar | --escribir] [--json]",
        en: "usage: roadmap [--comparar | --escribir] [--json]",
    }
    DESFASADO {
        es: "el ROADMAP.md publicado no esta al dia: ejecuta tc2 roadmap --escribir",
        en: "the published ROADMAP.md is out of date: run tc2 roadmap --escribir",
    }
    ZONA_ROTA {
        es: "la zona manual de {donde} esta rota ({motivo}): deja una sola zona que abra con «<!-- manual:inicio -->» y cierre con «<!-- manual:fin -->», y publicala con tc2 roadmap --escribir",
        en: "the manual zone in {donde} is broken ({motivo}): keep a single zone that opens with «<!-- manual:inicio -->» and closes with «<!-- manual:fin -->», and publish it with tc2 roadmap --escribir",
    }
}
