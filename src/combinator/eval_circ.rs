use super::syntax::{
    circuit_like::{ClauseC, PatternC, TermC},
    typed::{PatternT, TermT},
};

impl TermT {
    pub fn eval_circ(&self) -> TermC {
        let mut clauses = vec![];
        let size = self.get_type().0;
        let inj = (0..size).collect::<Vec<_>>();
        self.eval_circ_clause(&PatternC::id(size), &inj, 1.0, &mut clauses);
        TermC {
            clauses,
            ty: self.get_type(),
        }
    }
    pub fn eval_circ_clause(
        &self,
        pattern: &PatternC,
        inj: &[usize],
        phase_mul: f64,
        clauses: &mut Vec<ClauseC>,
    ) {
        match self {
            TermT::Comp { terms, .. } => {
                if phase_mul < 0.0 {
                    for t in terms.iter().rev() {
                        t.eval_circ_clause(pattern, inj, phase_mul, clauses);
                    }
                } else {
                    for t in terms {
                        t.eval_circ_clause(pattern, inj, phase_mul, clauses);
                    }
                }
            }
            TermT::Tensor { terms } => {
                let mut start = 0;
                for t in terms {
                    let size = t.get_type().0;
                    let end = start + size;
                    t.eval_circ_clause(pattern, &inj[start..end], phase_mul, clauses);
                    start = end;
                }
            }
            TermT::Id { .. } => {
                // Intentionally blank
            }
            TermT::Phase { phase } => {
                clauses.push(ClauseC {
                    pattern: pattern.clone(),
                    phase: phase_mul * phase.eval(),
                });
            }
            TermT::IfLet {
                pattern: if_pattern,
                inner,
            } => {
                let mut unitary_clauses = Vec::new();
                let mut inner_pattern = pattern.clone();
                let mut inner_inj = inj.to_vec();
                if_pattern.eval_circ(&mut inner_pattern, &mut inner_inj, &mut unitary_clauses);
                for u in &unitary_clauses {
                    clauses.push(u.invert());
                }

                inner.eval_circ_clause(&inner_pattern, &inner_inj, phase_mul, clauses);

                clauses.extend(unitary_clauses.into_iter().rev())
            }
            TermT::Gate { def, .. } => {
                def.eval_circ_clause(pattern, inj, phase_mul, clauses);
            }
            TermT::Inverse { inner } => {
                inner.eval_circ_clause(pattern, inj, -phase_mul, clauses);
            }
            TermT::Sqrt { inner } => {
                inner.eval_circ_clause(pattern, inj, phase_mul / 2.0, clauses);
            }
        }
    }
}

impl PatternT {
    pub fn eval_circ(
        &self,
        pattern: &mut PatternC,
        inj: &mut Vec<usize>,
        clauses: &mut Vec<ClauseC>,
    ) {
        match self {
            PatternT::Comp { patterns } => {
                for p in patterns {
                    p.eval_circ(pattern, inj, clauses);
                }
            }
            PatternT::Tensor { patterns } => {
                let mut stack: Vec<Vec<usize>> = Vec::new();
                for p in patterns.iter().rev() {
                    let size = p.get_type().0;
                    let mut i = inj.split_off(inj.len() - size);
                    p.eval_circ(pattern, &mut i, clauses);
                    stack.push(i);
                }
                while let Some(i) = stack.pop() {
                    inj.extend(i);
                }
            }
            PatternT::Ket { states } => {
                for (state, i) in states.iter().zip(inj.drain(0..states.len())) {
                    pattern.parts[i] = Some(*state)
                }
            }
            PatternT::Unitary(term_t) => {
                term_t.eval_circ_clause(&PatternC::id(pattern.parts.len()), inj, 1.0, clauses);
            }
        }
    }
}
