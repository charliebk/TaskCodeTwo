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
    ZONA_DIVERGENTE {
        es: "la zona manual publicada cambio desde que editaste {donde}: no se ha publicado nada y tu edicion sigue en la carpeta; guarda una copia, descarta el cambio, trae lo publicado con tc2 roadmap --escribir y vuelve a aplicar tu edicion a mano",
        en: "the published manual zone changed since you edited {donde}: nothing was published and your edit is still in the folder; save a copy, discard the change, bring in what was published with tc2 roadmap --escribir and reapply your edit by hand",
    }
    NO_SE_RESTAURO {
        es: "no se pudo dejar {donde} como estaba en el indice: {error}; revisa git status en tareas/ antes de seguir",
        en: "could not put {donde} back as it was in the index: {error}; check git status in tareas/ before going on",
    }
    AVISO_FUERA_DE_ZONA {
        es: "aviso: {donde} tiene cambios fuera de la zona manual; esa parte la genera tc2 y no se publica: descartalos",
        en: "warning: {donde} has changes outside the manual zone; tc2 generates that part and it is not published: discard them",
    }
    AVISO_PERSONAS_ILEGIBLES {
        es: "aviso: no se pudo leer la lista de personas de la base; los asignados heredados se muestran tal cual",
        en: "warning: the list of people in the base could not be read; inherited assignees are shown as they are",
    }
    ZONA_ROTA {
        es: "la zona manual de {donde} esta rota ({motivo}): deja una sola zona que abra con «<!-- manual:inicio -->» y cierre con «<!-- manual:fin -->», y publicala con tc2 roadmap --escribir",
        en: "the manual zone in {donde} is broken ({motivo}): keep a single zone that opens with «<!-- manual:inicio -->» and closes with «<!-- manual:fin -->», and publish it with tc2 roadmap --escribir",
    }
}
