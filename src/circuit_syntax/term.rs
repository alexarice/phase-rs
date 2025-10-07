//! Circuit-normal terms.

use crate::{
    circuit_syntax::pattern::PatternC,
    phase::Phase,
    typed_syntax::{TermT, TermType},
};

/// Circuit-normal terms.
///
/// These takes the form:
/// if let q_11 x ... x q_1n then Phase(theta_1) x id(m_1);
/// ... ;
/// if let q_l1 x ... x q_ln then Phase(theta_l) x id(m_l)
#[derive(Clone, Debug, PartialEq)]
pub struct TermC {
    pub(crate) clauses: Vec<ClauseC>,
    pub(crate) ty: TermType,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ClauseC {
    pub(crate) pattern: PatternC,
    pub(crate) phase: f64,
}

impl TermC {
    /// Return a `TermT` which is the "quotation" of this circuit-normal-form term.
    /// Realises that all circuit-normal-form terms are also terms.
    pub fn quote(&self) -> TermT {
        match self.clauses.len() {
            0 => TermT::Id(self.ty),
            1 => self.clauses[0].quote(),
            _ => TermT::Comp(self.clauses.iter().map(ClauseC::quote).collect()),
        }
    }
}

impl ClauseC {
    pub(crate) fn quote(&self) -> TermT {
        let id_qubits = self.pattern.id_qubits();
        let mut inner = TermT::Phase(Phase::Angle(self.phase));
        if id_qubits != 0 {
            inner = TermT::Tensor(vec![inner, TermT::Id(TermType(id_qubits))])
        }

        TermT::IfLet {
            pattern: self.pattern.quote(),
            inner: Box::new(inner),
        }
    }

    pub(crate) fn invert(&self) -> ClauseC {
        ClauseC {
            pattern: self.pattern.clone(),
            phase: -self.phase,
        }
    }
}
