pub mod normal;
pub mod raw;
pub mod typed;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KetState {
    Zero,
    One,
    Plus,
    Minus,
}

impl KetState {
    pub fn compl(self) -> Self {
        match self {
            KetState::Zero => KetState::One,
            KetState::One => KetState::Zero,
            KetState::Plus => KetState::Minus,
            KetState::Minus => KetState::Plus,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Angle(f64),
    MinusOne,
    Imag,
    NegImag,
}
