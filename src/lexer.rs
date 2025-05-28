use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[regex("q[0-9]+")]
    QReg,

    #[token("<->")]
    LR,

    #[token("ph")]
    Phase,

    #[token(";")]
    Semicolon,

    #[token("x")]
    Tensor,

    #[token("id[0-9]+")]
    Id,

    #[token("if")]
    If,

    #[token("let")]
    Let,

    #[token("then")]
    Then,

    Error,
}

// impl Display for Token {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Token::LParen => write!(f, "("),
//             Token::RParen => write!(f, ")"),
//             Token::QReg(n) => write!(f, "q{n}"),
//             Token::LR => write!(f, "<->"),
//             Token::Phase => write!(f, "ph"),
//             Token::Semicolon => write!(f, ";"),
//             Token::Tensor => write!(f, "x"),
//             Token::Id(n) => write!(f, "id {n}"),
//             Token::If => write!(f, "if"),
//             Token::Let => write!(f, "let"),
//             Token::Then => write!(f, "then"),
//             Token::Error => write!(f, "Lexing error"),
//         }
//     }
// }
