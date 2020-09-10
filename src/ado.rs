use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use crate::{bail, ensure};

/// An ADO.net connection string
///
/// [Read more](https://docs.microsoft.com/en-us/dotnet/framework/data/adonet/connection-string-syntax)
#[derive(Debug)]
pub struct AdoNetString {
    pairs: HashMap<String, String>,
}

impl Deref for AdoNetString {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.pairs
    }
}

impl DerefMut for AdoNetString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pairs
    }
}

// NOTE(yosh): Unfortunately we can't parse using `split(';')` because JDBC
// strings support escaping. This means that `{;}` is valid and we need to write
// an actual LR parser.
impl FromStr for AdoNetString {
    type Err = crate::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut lexer = Lexer::tokenize(input)?;
        let mut pairs = HashMap::new();

        // Iterate over `key=value` pairs.
        for n in 0.. {
            // [property=[value][;property=value][;]]
            //                                       ^
            if lexer.peek().kind() == &TokenKind::Eof {
                break;
            }

            // [property=[value][;property=value][;]]
            //                   ^
            if n != 0 {
                let err = "Key-value pairs must be separated by a `;`";
                ensure!(lexer.next().kind() == &TokenKind::Semi, err);

                // [property=value[;property=value][;]]
                //                                  ^
                if lexer.peek().kind() == &TokenKind::Eof {
                    break;
                }
            }

            // [property=[value][;property=value][;]]
            //  ^^^^^^^^
            let key = read_ident(&mut lexer)?;
            ensure!(!key.is_empty(), "Key must not be empty");

            // [property=[value][;property=value][;]]
            //          ^
            let err = "key-value pairs must be joined by a `=`";
            ensure!(lexer.next().kind() == &TokenKind::Eq, err);

            // [property=[value][;property=value][;]]
            //           ^^^^^
            let value = read_ident(&mut lexer)?;

            pairs.insert(key, value);
        }
        Ok(Self { pairs })
    }
}

/// Read either a valid key or value from the lexer.
fn read_ident(lexer: &mut Lexer) -> crate::Result<String> {
    let mut output = String::new();
    loop {
        let Token { kind, loc } = lexer.peek();
        match kind {
            TokenKind::Atom(c) => {
                let _ = lexer.next();
                output.push(c);
            }
            TokenKind::Escaped(seq) => {
                let _ = lexer.next();
                output.extend(seq);
            }
            TokenKind::Semi => break,
            TokenKind::Eq => break,
            TokenKind::Newline => {
                let _ = lexer.next();
                continue; // NOTE(yosh): unsure if this is the correct behavior
            }
            TokenKind::Whitespace => {
                let _ = lexer.next();
                match output.len() {
                    0 => continue, // ignore leading whitespace
                    _ => output.push(' '),
                }
            }
            TokenKind::Eof => break,
        }
    }
    output = output.trim_end().to_owned(); // remove trailing whitespace
    Ok(output)
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

    fn kind(&self) -> &TokenKind {
        &self.kind
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum TokenKind {
    Semi,
    Eq,
    Atom(char),
    Escaped(Vec<char>),
    Newline,
    Whitespace,
    Eof,
}

#[derive(Debug)]
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
                '"' => {
                    let mut buf = Vec::new();
                    loop {
                        match chars.next() {
                            None => bail!("unclosed double quote"),
                            // When we read a double quote inside a double quote
                            // we need to lookahead to determine whether it's an
                            // escape sequence or a closing delimiter.
                            Some('"') => match lookahead(&chars) {
                                Some('"') => {
                                    if buf.len() == 0 {
                                        break;
                                    }
                                    let _ = chars.next();
                                    buf.push('"');
                                    buf.push('"');
                                }
                                Some(_) | None => break,
                            },
                            Some(c) if c.is_ascii() => buf.push(c),
                            _ => bail!("Invalid ado.net token"),
                        }
                    }
                    TokenKind::Escaped(buf)
                }
                '\'' => {
                    let mut buf = Vec::new();
                    loop {
                        match chars.next() {
                            None => bail!("unclosed single quote"),
                            // When we read a single quote inside a single quote
                            // we need to lookahead to determine whether it's an
                            // escape sequence or a closing delimiter.
                            Some('\'') => match lookahead(&chars) {
                                Some('\'') => {
                                    if buf.len() == 0 {
                                        break;
                                    }
                                    let _ = chars.next();
                                    buf.push('\'');
                                    buf.push('\'');
                                }
                                Some(_) | None => break,
                            },
                            Some(c) if c.is_ascii() => buf.push(c),
                            _ => bail!("Invalid ado.net token"),
                        }
                    }
                    TokenKind::Escaped(buf)
                }
                ';' => TokenKind::Semi,
                '=' => TokenKind::Eq,
                '\n' => TokenKind::Newline,
                ' ' => TokenKind::Whitespace,
                char if char.is_ascii() => TokenKind::Atom(char),
                char => bail!("Invalid character found: {}", char),
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

/// Look at the next char in the iterator.
fn lookahead(iter: &std::str::Chars<'_>) -> Option<char> {
    let s = iter.as_str();
    s.chars().next()
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

    fn assert_kv(ado: &AdoNetString, key: &str, value: &str) {
        assert_eq!(ado.get(key), Some(&value.to_owned()));
    }

    // Source: https://docs.microsoft.com/en-us/dotnet/framework/data/adonet/connection-string-syntax#windows-authentication-with-sqlclient
    #[test]
    fn windows_auth_with_sql_client() -> crate::Result<()> {
        let input = "Persist Security Info=False;Integrated Security=true;\nInitial Catalog=AdventureWorks;Server=MSSQL1";
        let ado: AdoNetString = input.parse()?;
        assert_kv(&ado, "Persist Security Info", "False");
        assert_kv(&ado, "Integrated Security", "true");
        assert_kv(&ado, "Server", "MSSQL1");
        assert_kv(&ado, "Initial Catalog", "AdventureWorks");
        Ok(())
    }

    // https://docs.microsoft.com/en-us/dotnet/framework/data/adonet/connection-string-syntax#sql-server-authentication-with-sqlclient
    #[test]
    fn sql_server_auth_with_sql_client() -> crate::Result<()> {
        let input = "Persist Security Info=False;User ID=*****;Password=*****;Initial Catalog=AdventureWorks;Server=MySqlServer";
        let ado: AdoNetString = input.parse()?;
        assert_kv(&ado, "Persist Security Info", "False");
        assert_kv(&ado, "User ID", "*****");
        assert_kv(&ado, "Password", "*****");
        assert_kv(&ado, "Initial Catalog", "AdventureWorks");
        assert_kv(&ado, "Server", "MySqlServer");
        Ok(())
    }

    #[test]
    fn connect_to_named_sql_server_instance() -> crate::Result<()> {
        let input = r#"Data Source=MySqlServer\MSSQL1;"#;
        let ado: AdoNetString = input.parse()?;
        assert_kv(&ado, "Data Source", r#"MySqlServer\MSSQL1"#);
        Ok(())
    }
}
