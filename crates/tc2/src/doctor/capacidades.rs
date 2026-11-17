//! `capacidades` (F1-13 decision 3): que garantias impone la plataforma con
//! el plan observado, cuales sustituye el CLI y cuales no hay.

/// Lo que va en la primera linea de `doctor`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cuentas {
    /// Plan observado (`free-publico-usuario`, `gitlab-free`...) o
    /// `no-comprobado`.
    pub plan: String,
    /// `(impuestas, sustituidas, no-disponibles)`; `None` sin plan.
    pub cuentas: Option<(usize, usize, usize)>,
}
