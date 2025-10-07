//! Circuit-normal patterns.

use crate::{
    ket::{CompKetState, KetState},
    typed_syntax::{PatternT, TermT, TermType},
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PatternC {
    pub(crate) parts: Vec<Option<KetState>>,
}

fn state_to_pattern(s: Option<KetState>) -> PatternT {
    s.map_or(
        PatternT::Unitary(Box::new(TermT::Id(TermType(1)))),
        |state| PatternT::Ket(CompKetState::single(state)),
    )
}

impl PatternC {
    pub(crate) fn quote(&self) -> PatternT {
        if self.parts.len() == 1 {
            state_to_pattern(self.parts[0])
        } else {
            PatternT::Tensor(self.parts.iter().cloned().map(state_to_pattern).collect())
        }
    }

    pub(crate) fn id_qubits(&self) -> usize {
        self.parts.iter().filter(|x| x.is_none()).count()
    }

    pub(crate) fn id(l: usize) -> Self {
        PatternC {
            parts: vec![None; l],
        }
    }
}
