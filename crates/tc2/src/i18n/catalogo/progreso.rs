//! Mensajes del progreso en vivo (F3-12).

crate::i18n::textos! {
    AVISO_NO_ESCRITO {
        es: "aviso: no se pudo escribir el progreso de {id} ({error}); la transicion se hizo igual y la status line lo recupera en la siguiente",
        en: "warning: the progress of {id} could not be written ({error}); the transition was done anyway and the status line catches up on the next one",
    }
}
