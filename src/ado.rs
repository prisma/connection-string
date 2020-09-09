use std::collections::HashMap;
use std::str::FromStr;

use crate::bail;

// use crate::utils::{bail, ensure};

// NOTE(yosh): ESCAPING is done by wrapping quotes in other quotes.
// ```
// password='somepass"word'
// ```
//
// From [Ado net connection string remarks](https://docs.microsoft.com/en-us/dotnet/api/system.data.sqlclient.sqlconnection.connectionstring?redirectedfrom=MSDN&view=dotnet-plat-ext-3.1#remarks):
//
// The basic format of a connection string includes a series of keyword/value
// pairs separated by semicolons. The equal sign (=) connects each keyword and
// its value. To include values that contain a semicolon, single-quote
// character, or double-quote character, the value must be enclosed in double
// quotation marks. If the value contains both a semicolon and a double-quote
// character, the value can be enclosed in single quotation marks. The single
// quotation mark is also useful if the value starts with a double-quote
// character. Conversely, the double quotation mark can be used if the value
// starts with a single quotation mark. If the value contains both single-quote
// and double-quote characters, the quotation mark character used to enclose the
// value must be doubled every time it occurs within the value.
//
// To include preceding or trailing spaces in the string value, the value must
// be enclosed in either single quotation marks or double quotation marks. Any
// leading or trailing spaces around integer, Boolean, or enumerated values are
// ignored, even if enclosed in quotation marks. However, spaces within a string
// literal keyword or value are preserved. Single or double quotation marks may
// be used within a connection string without using delimiters (for example,
// Data Source= my'Server or Data Source= my"Server), unless a quotation mark
// character is the first or last character in the value.

#[derive(Debug)]
pub struct AdoNetString {
    pairs: HashMap<String, String>,
}

// NOTE(yosh): Unfortunately we can't parse using `split(';')` because JDBC
// strings support escaping. This means that `{;}` is valid and we need to write
// an actual LR parser.
impl FromStr for AdoNetString {
    type Err = crate::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut lexer = Lexer::tokenize(input)?;
        todo!();
    }
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    loc: Location,
}

impl Token {
    /// Create a new instance.
    fn new(kind: TokenKind, loc: Location) -> Self {
        Self { kind, loc }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum TokenKind {
    DQuote,
    SQuote,
    Semi,
    Eq,
    LBrace,
    RBrace,
    Atom(char),
    Newline,
    Eof,
}

struct Lexer {
    tokens: Vec<Token>,
}

impl Lexer {
    /// Parse a string into a sequence of tokens.
    fn tokenize(mut input: &str) -> crate::Result<Self> {
        let mut tokens = vec![];
        let mut loc = Location::default();
        while !input.is_empty() {
            let old_input = input;
            let mut chars = input.chars();
            let kind = match chars.next().unwrap() {
                '"' => TokenKind::DQuote,  // TODO read an escape sequence,
                '\'' => TokenKind::SQuote, // TODO read an escape sequence,
                ';' => TokenKind::Semi,
                '=' => TokenKind::Eq,
                '(' => TokenKind::LBrace,
                ')' => TokenKind::RBrace,
                '\n' => TokenKind::Newline,
                char if char.is_alphanumeric() => TokenKind::Atom(char),
                char => bail!("Invalid character found"),
            };
            tokens.push(Token { kind, loc });
            input = chars.as_str();

            let consumed = old_input.len() - input.len();
            loc.advance(&old_input[..consumed]);
        }
        tokens.reverse();
        Ok(Self { tokens })
    }

    /// Get the next token from the queue.
    #[must_use]
    pub(crate) fn next(&mut self) -> Token {
        self.tokens.pop().unwrap_or(Token {
            kind: TokenKind::Eof,
            loc: Location::default(),
        })
    }

    /// Push a token back onto the queue.
    pub(crate) fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }

    /// Peek at the next token in the queue.
    #[must_use]
    pub(crate) fn peek(&mut self) -> Token {
        self.tokens.last().map(|t| t.clone()).unwrap_or(Token {
            kind: TokenKind::Eof,
            loc: Location::default(),
        })
    }
}

/// Track the location of the Token inside the string.
#[derive(Copy, Clone, Default, Debug)]
pub(crate) struct Location {
    pub(crate) column: usize,
}

impl Location {
    fn advance(&mut self, text: &str) {
        self.column += text.chars().count();
    }
}

#[cfg(test)]
mod test {
    use super::AdoNetString;
    #[test]
    fn basic() -> crate::Result<()> {
        let s: AdoNetString = "Data Source=MSSQL1;Initial Catalog=AdventureWorks;".parse()?;
        Ok(())
    }
}
