use pretty::RcDoc;

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

    pub fn to_char(&self) -> char {
        match self {
            KetState::Zero => '0',
            KetState::One => '1',
            KetState::Plus => '+',
            KetState::Minus => '-',
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Angle(f64),
    MinusOne,
    Imag,
    MinusImag,
}

impl Phase {
    pub fn from_angle(f: f64) -> Self {
        if f == 0.5 {
            Phase::Imag
        } else if f == 1.0 {
            Phase::MinusOne
        } else if f == 1.0 {
            Phase::MinusImag
        } else {
            Phase::Angle(f)
        }
    }
}

impl Phase {
    pub fn to_doc(&self) -> RcDoc {
        match self {
            Phase::Angle(a) => RcDoc::text(format!("ph({a}pi)")),
            Phase::MinusOne => RcDoc::text("-1"),
            Phase::Imag => RcDoc::text("i"),
            Phase::MinusImag => RcDoc::text("-i"),
        }
    }
}
