use super::{
    KetState, Phase,
    typed::{PatternT, TermT, TermType},
};

#[derive(Clone, Debug, PartialEq)]
pub struct TermC {
    pub clauses: Vec<ClauseC>,
    pub ty: TermType,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClauseC {
    pub pattern: PatternC,
    pub phase: f64,
    pub id_qubits: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PatternC {
    pub parts: Vec<Option<KetState>>,
}

impl TermC {
    pub fn quote(&self) -> TermT {
        match self.clauses.len() {
            0 => TermT::Id {
                ty: self.ty.clone(),
            },
            1 => self.clauses[0].quote(),
            _ => TermT::Comp {
                terms: self.clauses.iter().map(ClauseC::quote).collect(),
                ty: self.ty.clone(),
            },
        }
    }
}

impl ClauseC {
    pub fn quote(&self) -> TermT {
        let mut inner = TermT::Phase {
            phase: Phase::Angle(self.phase),
        };
        if self.id_qubits != 0 {
            inner = TermT::Tensor {
                terms: vec![
                    inner,
                    TermT::Id {
                        ty: TermType(self.id_qubits),
                    },
                ],
            }
        }

        TermT::IfLet {
            pattern: self.pattern.quote(),
            inner: Box::new(inner),
        }
    }

    pub fn invert(&self) -> ClauseC {
        ClauseC {
            pattern: self.pattern.clone(),
            phase: -self.phase,
            id_qubits: self.id_qubits,
        }
    }
}

fn state_to_pattern(s: Option<KetState>) -> PatternT {
    s.map_or(
        PatternT::Unitary(Box::new(TermT::Id { ty: TermType(1) })),
        |state| PatternT::Ket {
            states: vec![state],
        },
    )
}

impl PatternC {
    pub fn quote(&self) -> PatternT {
        if self.parts.len() == 1 {
            state_to_pattern(self.parts[0])
        } else {
            PatternT::Tensor {
                patterns: self.parts.iter().cloned().map(state_to_pattern).collect(),
            }
        }
    }

    pub fn id(l: usize) -> Self {
        PatternC {
            parts: vec![None; l],
        }
    }
}
