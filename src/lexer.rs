use std::{fmt, mem};

use crate::arena::{ArenaVec, GIB};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenType {
    And, // and
    Or,  // or
    Xor, // xor
    Not, // not

    Equals,       // ==
    NotEquals,    // !=
    LessThan,     // <
    GreaterThan,  // >
    LessEqual,    // <=
    GreaterEqual, // >=

    Feather, // >-
    Arrow,   // ->

    Ampersand, // &
    Pipe,      // |
    Caret,     // ^
    Tilde,     // ~
    LShift,    // <<
    RShift,    // >>

    Incr,   // ++
    Decr,   // --
    Plus,   // +
    Minus,  // -
    Mul,    // *
    Div,    // /
    Pow,    // **
    Modulo, // %

    Pub, // pub

    Packed, // packed
    Struct, // struct
    Enum,   // enum
    Union,  // union

    Fn,       // fn
    Defer,    // defer
    If,       // if
    Then,     // then
    Else,     // else
    While,    // while
    Do,       // do
    Loop,     // loop
    Continue, // continue
    Break,    // break

    Equal,    // =
    Semi,     // ;
    Colon,    // :
    Comma,    // ,
    Dot,      // .
    LParens,  // (
    RParens,  // )
    LBracket, // [
    RBracket, // ]
    LBrace,   // {
    RBrace,   // }

    String,
    StringInterpBeg,
    StringInterpMid,
    StringInterpEnd,
    Char, // 'a'
    Ident,
    Num,
}

#[derive(Debug, Clone)]
pub struct TokenSpan<'a> {
    pub slice: &'a str,
    pub line: usize,
    pub col: usize,
}

impl<'a> TokenSpan<'a> {
    #[inline]
    pub const fn new(slice: &'a str, line: usize, col: usize) -> Self {
        Self { slice, line, col }
    }
}

#[derive(Debug)]
pub struct Tokens<'a> {
    /// The entire code file
    pub code: &'a str,
    /// Sorted list containing the position of all line breaks
    pub line_breaks: ArenaVec<usize>,
    /// Token spans in the code
    pub spans: ArenaVec<TokenSpan<'a>>,
    /// Respective token types
    pub types: ArenaVec<TokenType>,
}

impl<'a> fmt::Display for Tokens<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn log10(n: usize) -> usize {
            (n as f64).log10().ceil() as usize
        }

        let line_dwidth = log10(self.line_breaks.len());

        let mut col_dwidth = 0;
        let mut type_dwidth = 0;
        for (&ty, span) in self.types.iter().zip(self.spans.iter()) {
            col_dwidth = col_dwidth.max(log10(span.col));
            type_dwidth = type_dwidth.max(format!("{ty:?}").len());
        }

        for (ty, TokenSpan { slice, line, col }) in self.types.iter().zip(self.spans.iter()) {
            writeln!(
                f,
                "{line:>line_dwidth$}:{col:<col_dwidth$}   {:<type_dwidth$}   {slice}",
                format!("{ty:?}"),
                line_dwidth = line_dwidth,
                col_dwidth = col_dwidth,
                type_dwidth = type_dwidth,
            )?;
        }

        Ok(())
    }
}

mod kw {
    pub const CONTINUE: &[u8] = b"continue";
    pub const PACKED: &[u8] = b"packed";
    pub const STRUCT: &[u8] = b"struct";
    pub const UNION: &[u8] = b"union";
    pub const DEFER: &[u8] = b"defer";
    pub const WHILE: &[u8] = b"while";
    pub const BREAK: &[u8] = b"break";
    pub const ENUM: &[u8] = b"enum";
    pub const THEN: &[u8] = b"then";
    pub const ELSE: &[u8] = b"else";
    pub const LOOP: &[u8] = b"loop";
    pub const AND: &[u8] = b"and";
    pub const XOR: &[u8] = b"xor";
    pub const NOT: &[u8] = b"not";
    pub const PUB: &[u8] = b"pub";
    pub const OR: &[u8] = b"or";
    pub const FN: &[u8] = b"fn";
    pub const IF: &[u8] = b"if";
    pub const DO: &[u8] = b"do";
}

mod op {
    pub const EQUALS: &[u8] = b"==";
    pub const NOT_EQUALS: &[u8] = b"!=";
    pub const LESS_EQUAL: &[u8] = b"<=";
    pub const GREATER_EQUAL: &[u8] = b">=";
    pub const FEATHER: &[u8] = b">-";
    pub const ARROW: &[u8] = b"->";
    pub const L_SHIFT: &[u8] = b"<<";
    pub const R_SHIFT: &[u8] = b">>";
    pub const INCR: &[u8] = b"++";
    pub const DECR: &[u8] = b"--";
    pub const POW: &[u8] = b"**";
    pub const MODULO: &[u8] = b"%";
    pub const LESS_THAN: &[u8] = b"<";
    pub const GREATER_THAN: &[u8] = b">";
    pub const AMPERSAND: &[u8] = b"&";
    pub const PIPE: &[u8] = b"|";
    pub const CARET: &[u8] = b"^";
    pub const TILDE: &[u8] = b"~";
    pub const PLUS: &[u8] = b"+";
    pub const MINUS: &[u8] = b"-";
    pub const MUL: &[u8] = b"*";
    pub const DIV: &[u8] = b"/";
    pub const EQUAL: &[u8] = b"=";
    pub const SEMI: &[u8] = b";";
    pub const COLON: &[u8] = b":";
    pub const COMMA: &[u8] = b",";
    pub const DOT: &[u8] = b".";
    pub const L_PARENS: &[u8] = b"(";
    pub const R_PARENS: &[u8] = b")";
    pub const L_BRACKET: &[u8] = b"[";
    pub const R_BRACKET: &[u8] = b"]";
    pub const L_BRACE: &[u8] = b"{";
    pub const R_BRACE: &[u8] = b"}";
}

pub fn lex<'a>(file_name: &str, code: &'a str) -> Tokens<'a> {
    let mut line = 1;
    let mut line_start = code.as_ptr() as usize;

    let addr_space_size = 64 * GIB;

    let mut tokens = Tokens {
        code,
        line_breaks: ArenaVec::new(addr_space_size / 8),
        spans: ArenaVec::new(addr_space_size),
        types: ArenaVec::new(addr_space_size / mem::size_of::<TokenSpan>()),
    };

    let bcode = tokens.code.as_bytes();
    let mut input = bcode;
    while !input.is_empty() {
        input = consume_token(file_name, input, &mut line, &mut line_start, &mut tokens);
    }

    tokens
}

/// Consume - in most cases - a single token.
///
/// Exceptions are made for special nestings, like interpolated strings and
/// pairs of tokens that indicate a beginning and an end like parentheses,
/// in which case it will recurse.
fn consume_token<'a>(
    file_name: &str,
    mut input: &'a [u8],
    line: &mut usize,
    line_start: &mut usize,
    tokens: &mut Tokens<'a>,
) -> &'a [u8] {
    let bcode = tokens.code.as_bytes();
    let start_addr = bcode.as_ptr() as usize;

    // save line breaks
    while !input.is_empty() && input[0] == b'\n' {
        let addr = input.as_ptr() as usize;
        tokens.line_breaks.add(addr - start_addr);
        input = &input[1..];
        *line_start = input.as_ptr() as usize;
        *line += 1;
    }

    // ignore whitespace
    while !input.is_empty() && input[0].is_ascii_whitespace() {
        input = &input[1..];
    }

    if input.is_empty() {
        return input;
    }

    // ignore comments
    if input.starts_with(b"//") {
        input = &input[2..];
        while input[0] != b'\n' {
            input = &input[1..];
        }
        return input;
    }

    // operators
    {
        let mut op_len;
        let is_operator = 'op: {
            op_len = 2;
            if input.len() >= op_len {
                let toktype = match &input[..op_len] {
                    op::EQUALS => Some(TokenType::Equals),
                    op::NOT_EQUALS => Some(TokenType::NotEquals),
                    op::LESS_EQUAL => Some(TokenType::LessEqual),
                    op::GREATER_EQUAL => Some(TokenType::GreaterEqual),
                    op::FEATHER => Some(TokenType::Feather),
                    op::ARROW => Some(TokenType::Arrow),
                    op::L_SHIFT => Some(TokenType::LShift),
                    op::R_SHIFT => Some(TokenType::RShift),
                    op::INCR => Some(TokenType::Incr),
                    op::DECR => Some(TokenType::Decr),
                    op::POW => Some(TokenType::Pow),
                    _ => None,
                };

                if let Some(toktype) = toktype {
                    tokens.types.add(toktype);
                    break 'op true;
                }
            }

            op_len = 1;
            if input.len() >= op_len {
                let toktype = match &input[..op_len] {
                    op::MODULO => Some(TokenType::Modulo),
                    op::LESS_THAN => Some(TokenType::LessThan),
                    op::GREATER_THAN => Some(TokenType::GreaterThan),
                    op::AMPERSAND => Some(TokenType::Ampersand),
                    op::PIPE => Some(TokenType::Pipe),
                    op::CARET => Some(TokenType::Caret),
                    op::TILDE => Some(TokenType::Tilde),
                    op::PLUS => Some(TokenType::Plus),
                    op::MINUS => Some(TokenType::Minus),
                    op::MUL => Some(TokenType::Mul),
                    op::DIV => Some(TokenType::Div),
                    op::EQUAL => Some(TokenType::Equal),
                    op::SEMI => Some(TokenType::Semi),
                    op::COLON => Some(TokenType::Colon),
                    op::COMMA => Some(TokenType::Comma),
                    op::DOT => Some(TokenType::Dot),
                    op::L_PARENS => Some(TokenType::LParens),
                    op::R_PARENS => Some(TokenType::RParens),
                    op::L_BRACKET => Some(TokenType::LBracket),
                    op::R_BRACKET => Some(TokenType::RBracket),
                    op::L_BRACE => Some(TokenType::LBrace),
                    op::R_BRACE => Some(TokenType::RBrace),
                    _ => None,
                };

                if let Some(toktype) = toktype {
                    tokens.types.add(toktype);
                    break 'op true;
                }
            }

            false
        };

        if is_operator {
            let col = input.as_ptr() as usize - *line_start;
            let slice = unsafe { std::str::from_utf8_unchecked(&input[..op_len]) };
            tokens.spans.add(TokenSpan::new(slice, *line, col));
            input = &input[op_len..];
            return input;
        }
    }

    // interpolated strings
    if input.starts_with(b"$\"") {
        let mut is_valid = false;

        let mut start_str_addr = input.as_ptr() as usize;
        input = &input[2..];

        let mut has_interpolation = false;
        while !input.is_empty() {
            if input.starts_with(br#"\""#) || input.starts_with(br#"\{"#) {
                input = &input[2..];
                continue;
            }

            if input[0] == b'"' {
                // end of string

                is_valid = true;
                input = &input[1..];

                let end_str_addr = input.as_ptr() as usize;
                let start = start_str_addr - start_addr;
                let end = end_str_addr - start_addr;

                tokens.types.add(match has_interpolation {
                    true => TokenType::StringInterpEnd,
                    false => TokenType::String,
                });
                let col = bcode.as_ptr() as usize + start - *line_start;
                let slice = unsafe { std::str::from_utf8_unchecked(&bcode[start..end]) };
                tokens.spans.add(TokenSpan::new(slice, *line, col));
                break;
            } else if input[0] == b'{' {
                // inside interpolated expression (we can consume tokens recursively)

                input = &input[1..];

                let end_str_addr = input.as_ptr() as usize;
                let start = start_str_addr - start_addr;
                let end = end_str_addr - start_addr;

                tokens.types.add(match has_interpolation {
                    true => TokenType::StringInterpMid,
                    false => TokenType::StringInterpBeg,
                });
                let col = bcode.as_ptr() as usize + start - *line_start;
                let slice = unsafe { std::str::from_utf8_unchecked(&bcode[start..end]) };
                tokens.spans.add(TokenSpan::new(slice, *line, col));

                has_interpolation = true;

                while !input.is_empty() && input[0] != b'}' {
                    input = consume_token(file_name, input, line, line_start, tokens);
                }
                if input.is_empty() {
                    break;
                }

                start_str_addr = input.as_ptr() as usize;
                input = &input[1..];
            } else if input[0] == b'\n' {
                // strings support line breaks

                let addr = input.as_ptr() as usize;
                tokens.line_breaks.add(addr - start_addr);
                input = &input[1..];
                *line_start = input.as_ptr() as usize;
                *line += 1;
            } else {
                input = &input[1..];
            }
        }

        if is_valid {
            return input;
        } else {
            let col = start_str_addr + 1 - *line_start;
            panic!("{file_name}:{line}:{col}: Unfinished interpolated string");
        }
    }

    // strings
    // todo: raw strings (like in Rust)
    let (is_string, prefix): (bool, &[u8]) = if input.starts_with(b"b\"") {
        (true, b"b\"")
    } else if input.starts_with(b"c\"") {
        (true, b"c\"")
    } else if input[0] == b'"' {
        (true, b"\"")
    } else {
        (false, b"")
    };

    if is_string {
        let mut is_valid = false;

        let start_str_addr = input.as_ptr() as usize;
        input = &input[prefix.len()..];
        while !input.is_empty() {
            if input.starts_with(br#"\""#) {
                input = &input[2..];
                continue;
            }

            if input[0] == b'"' {
                is_valid = true;
                input = &input[1..];
                break;
            }

            // strings support line breaks
            if input[0] == b'\n' {
                let addr = input.as_ptr() as usize;
                tokens.line_breaks.add(addr - start_addr);
                input = &input[1..];
                *line_start = input.as_ptr() as usize;
                *line += 1;
            } else {
                input = &input[1..];
            }
        }

        if is_valid {
            let end_str_addr = input.as_ptr() as usize;
            let start = start_str_addr - start_addr;
            let end = end_str_addr - start_addr;

            tokens.types.add(TokenType::String);
            let col = bcode.as_ptr() as usize + start - *line_start;
            let slice = unsafe { std::str::from_utf8_unchecked(&bcode[start..end]) };
            tokens.spans.add(TokenSpan::new(slice, *line, col));
            return input;
        } else {
            let col = start_str_addr + 1 - *line_start;
            panic!("{file_name}:{line}:{col}: Unfinished string");
        }
    }

    // chars
    let (is_char, prefix): (bool, &[u8]) = if input.starts_with(b"b'") {
        (true, b"b'")
    } else if input[0] == b'\'' {
        (true, b"'")
    } else {
        (false, b"")
    };

    if is_char {
        let mut is_valid = false;

        let start_str_addr = input.as_ptr() as usize;
        input = &input[prefix.len()..];
        while !input.is_empty() {
            if input.starts_with(br#"\'"#) {
                input = &input[2..];
                continue;
            }

            if input[0] == b'\'' {
                is_valid = true;
                input = &input[1..];
                break;
            }

            // chars can handle line breaks (though they shouldn't be allowed)
            if input[0] == b'\n' {
                let addr = input.as_ptr() as usize;
                tokens.line_breaks.add(addr - start_addr);
                input = &input[1..];
                *line_start = input.as_ptr() as usize;
                *line += 1;
            } else {
                input = &input[1..];
            }
        }

        if is_valid {
            let end_str_addr = input.as_ptr() as usize;
            let start = start_str_addr - start_addr;
            let end = end_str_addr - start_addr;

            tokens.types.add(TokenType::Char);
            let col = bcode.as_ptr() as usize + start - *line_start;
            let slice = unsafe { std::str::from_utf8_unchecked(&bcode[start..end]) };
            tokens.spans.add(TokenSpan::new(slice, *line, col));
            return input;
        } else {
            let col = start_str_addr + 1 - *line_start;
            panic!("{file_name}:{line}:{col}: Unfinished char");
        }
    }

    // identifiers
    if matches!(input[0], b'_' | b'A'..=b'Z' | b'a'..=b'z') {
        let start_ident_addr = input.as_ptr() as usize;

        input = &input[1..];
        while matches!(input[0], b'_' | b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9') {
            input = &input[1..];
        }

        let end_ident_addr = input.as_ptr() as usize;
        let start = start_ident_addr - start_addr;
        let end = end_ident_addr - start_addr;

        let col = bcode.as_ptr() as usize + start - *line_start;
        let ident_slice = &bcode[start..end];

        let mut token_len;
        let is_keyword = 'kw: {
            // keywords

            token_len = 8;
            if ident_slice.len() >= token_len {
                let toktype = if &ident_slice[..token_len] == kw::CONTINUE {
                    Some(TokenType::Continue)
                } else {
                    None
                };

                if let Some(toktype) = toktype {
                    tokens.types.add(toktype);
                    break 'kw true;
                }
            }

            token_len = 6;
            if ident_slice.len() >= token_len {
                let toktype = match &ident_slice[..token_len] {
                    kw::PACKED => Some(TokenType::Packed),
                    kw::STRUCT => Some(TokenType::Struct),
                    _ => None,
                };

                if let Some(toktype) = toktype {
                    tokens.types.add(toktype);
                    break 'kw true;
                }
            }

            token_len = 5;
            if ident_slice.len() >= token_len {
                let toktype = match &ident_slice[..token_len] {
                    kw::UNION => Some(TokenType::Union),
                    kw::DEFER => Some(TokenType::Defer),
                    kw::WHILE => Some(TokenType::While),
                    kw::BREAK => Some(TokenType::Break),
                    _ => None,
                };

                if let Some(toktype) = toktype {
                    tokens.types.add(toktype);
                    break 'kw true;
                }
            }

            token_len = 4;
            if ident_slice.len() >= token_len {
                let toktype = match &ident_slice[..token_len] {
                    kw::ENUM => Some(TokenType::Enum),
                    kw::THEN => Some(TokenType::Then),
                    kw::ELSE => Some(TokenType::Else),
                    kw::LOOP => Some(TokenType::Loop),
                    _ => None,
                };

                if let Some(toktype) = toktype {
                    tokens.types.add(toktype);
                    break 'kw true;
                }
            }

            token_len = 3;
            if ident_slice.len() >= token_len {
                let toktype = match &ident_slice[..token_len] {
                    kw::AND => Some(TokenType::And),
                    kw::XOR => Some(TokenType::Xor),
                    kw::NOT => Some(TokenType::Not),
                    kw::PUB => Some(TokenType::Pub),
                    _ => None,
                };

                if let Some(toktype) = toktype {
                    tokens.types.add(toktype);
                    break 'kw true;
                }
            }

            token_len = 2;
            if ident_slice.len() >= token_len {
                let toktype = match &ident_slice[..token_len] {
                    kw::OR => Some(TokenType::Or),
                    kw::FN => Some(TokenType::Fn),
                    kw::IF => Some(TokenType::If),
                    kw::DO => Some(TokenType::Do),
                    _ => None,
                };

                if let Some(toktype) = toktype {
                    tokens.types.add(toktype);
                    break 'kw true;
                }
            }

            false
        };

        if !is_keyword {
            tokens.types.add(TokenType::Ident);
        }

        let slice = unsafe { std::str::from_utf8_unchecked(ident_slice) };
        tokens.spans.add(TokenSpan::new(slice, *line, col));
        return input;
    }

    // numbers
    if input[0].is_ascii_digit() {
        let start_ident_addr = input.as_ptr() as usize;

        if input.starts_with(b"0x") {
            // hex literals
            input = &input[2..];
            while matches!(input[0], b'_' | b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F') {
                input = &input[1..];
            }
        } else if input.starts_with(b"0o") {
            // octal literals
            input = &input[2..];
            while matches!(input[0], b'_' | b'0'..=b'7') {
                input = &input[1..];
            }
        } else if input.starts_with(b"0b") {
            // binary literals
            input = &input[2..];
            while matches!(input[0], b'_' | b'0'..=b'1') {
                input = &input[1..];
            }
        } else {
            // decimal and floating literals

            // whole part
            input = &input[1..];
            while matches!(input[0], b'_' | b'0'..=b'9') {
                input = &input[1..];
            }

            // fractional part
            if input[0] == b'.' {
                input = &input[1..];
                while matches!(input[0], b'_' | b'0'..=b'9') {
                    input = &input[1..];
                }
            }

            // exponent
            if matches!(input[0], b'e' | b'E') {
                input = &input[1..];
                if matches!(input[0], b'+' | b'-') {
                    input = &input[1..];
                }
                while matches!(input[0], b'_' | b'0'..=b'9') {
                    input = &input[1..];
                }
            }
        }

        let end_ident_addr = input.as_ptr() as usize;
        let start = start_ident_addr - start_addr;
        let end = end_ident_addr - start_addr;

        tokens.types.add(TokenType::Num);
        let col = bcode.as_ptr() as usize + start - *line_start;
        let slice = unsafe { std::str::from_utf8_unchecked(&bcode[start..end]) };
        tokens.spans.add(TokenSpan::new(slice, *line, col));
        return input;
    }

    // Special recursions (parentheses, etc.)
    if input[0] == b'(' {
        let start_str_addr = input.as_ptr() as usize;

        while !input.is_empty() && input[0] != b')' {
            input = consume_token(file_name, input, line, line_start, tokens);
        }
        if input.is_empty() {
            let col = start_str_addr + 1 - *line_start;
            panic!("{file_name}:{line}:{col}: Unclosed parenthesis");
        }
    } else if input[0] == b'[' {
        let start_str_addr = input.as_ptr() as usize;

        while !input.is_empty() && input[0] != b']' {
            input = consume_token(file_name, input, line, line_start, tokens);
        }
        if input.is_empty() {
            let col = start_str_addr + 1 - *line_start;
            panic!("{file_name}:{line}:{col}: Unclosed bracket");
        }
    } else if input[0] == b'{' {
        let start_str_addr = input.as_ptr() as usize;

        while !input.is_empty() && input[0] != b'}' {
            input = consume_token(file_name, input, line, line_start, tokens);
        }
        if input.is_empty() {
            let col = start_str_addr + 1 - *line_start;
            panic!("{file_name}:{line}:{col}: Unclosed brace");
        }
    }

    let start_str_addr = input.as_ptr() as usize;
    let col = start_str_addr + 1 - *line_start;
    panic!("{file_name}:{line}:{col}: Cannot parse token");
}
