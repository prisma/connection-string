use std::collections::HashMap;
use std::str::FromStr;

use crate::{bail, ensure};

/// JDBC connection string parser for SqlServer
///
/// [Read more](https://docs.microsoft.com/en-us/sql/connect/jdbc/building-the-connection-url?view=sql-server-ver15)
///
/// # Format
///
/// ```txt
/// jdbc:sqlserver://[serverName[\instanceName][:portNumber]][;property=value[;property=value]]
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JdbcString {
    sub_protocol: &'static str,
    server_name: Option<String>,
    instance_name: Option<String>,
    port: Option<u16>,
    properties: HashMap<String, String>,
}

impl JdbcString {
    /// Access the connection sub-protocol
    pub fn sub_protocol(&self) -> &'static str {
        &self.sub_protocol
    }

    /// Access the connection server name
    pub fn server_name(&self) -> Option<&str> {
        self.server_name.as_ref().map(|s| s.as_str())
    }

    /// Access the connection's instance name
    pub fn instance_name(&self) -> Option<&str> {
        self.instance_name.as_ref().map(|s| s.as_str())
    }

    /// Access the connection's port
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Access the connection's key-value pairs
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }
}

// NOTE(yosh): Unfortunately we can't parse using `split(';')` because JDBC
// strings support escaping. This means that `{;}` is valid and we need to write
// an actual LR parser.
impl FromStr for JdbcString {
    type Err = crate::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut lexer = Lexer::tokenize(input)?;

        // ```
        // jdbc:sqlserver://[serverName[\instanceName][:portNumber]][;property=value[;property=value]]
        // ^^^^^^^^^^^^^^^^^
        // ```
        let err = "Invalid JDBC sub-protocol";
        cmp_str(&mut lexer, "jdbc", err)?;
        ensure!(lexer.next().kind() == &TokenKind::Colon, err);
        cmp_str(&mut lexer, "sqlserver", err)?;
        ensure!(lexer.next().kind() == &TokenKind::Colon, err);
        ensure!(lexer.next().kind() == &TokenKind::FSlash, err);
        ensure!(lexer.next().kind() == &TokenKind::FSlash, err);

        // ```
        // jdbc:sqlserver://[serverName[\instanceName][:portNumber]][;property=value[;property=value]]
        //                  ^^^^^^^^^^^
        // ```
        let mut server_name = None;
        if matches!(lexer.peek().kind(), TokenKind::Atom(_) | TokenKind::Escaped(_)) {
            server_name = Some(read_ident(&mut lexer, "Invalid server name")?);
        }

        // ```
        // jdbc:sqlserver://[serverName[\instanceName][:portNumber]][;property=value[;property=value]]
        //                             ^^^^^^^^^^^^^^^
        // ```
        let mut instance_name = None;
        if matches!(lexer.peek().kind(), TokenKind::BSlash) {
            let _ = lexer.next();
            instance_name = Some(read_ident(&mut lexer, "Invalid instance name")?);
        }

        // ```
        // jdbc:sqlserver://[serverName[\instanceName][:portNumber]][;property=value[;property=value]]
        //                                            ^^^^^^^^^^^^^
        // ```
        let mut port = None;
        if matches!(lexer.peek().kind(), TokenKind::Colon) {
            let _ = lexer.next();
            let err = "Invalid port";
            let s = read_ident(&mut lexer, err)?;
            port = Some(s.parse()?);
        }

        // ```
        // jdbc:sqlserver://[serverName[\instanceName][:portNumber]][;property=value[;property=value]]
        //                                                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        // ```
        // NOTE: we're choosing to only keep the last value per key rather than support multiple inserts per key.
        let mut properties = HashMap::new();
        while let TokenKind::Semi = lexer.peek().kind() {
            let _ = lexer.next();
            let err = "Invalid property key";
            let key = read_ident(&mut lexer, err)?;

            let err = "Property pairs must be joined by a `=`";
            ensure!(lexer.next().kind() == &TokenKind::Eq, err);

            let err = "Invalid property value";
            let value = read_ident(&mut lexer, err)?;

            properties.insert(key, value);
        }

        let token = lexer.next();
        ensure!(token.kind() == &TokenKind::Eof, "Invalid JDBC token");

        Ok(Self {
            sub_protocol: "jdbc:sqlserver",
            server_name,
            instance_name,
            port,
            properties,
        })
    }
}

/// Validate a sequence of `TokenKind::Atom` matches the content of a string.
fn cmp_str(lexer: &mut Lexer, s: &str, err_msg: &'static str) -> crate::Result<()> {
    for char in s.chars() {
        if let Token {
            kind: TokenKind::Atom(tchar),
            ..
        } = lexer.next()
        {
            ensure!(char == tchar, err_msg);
        } else {
            bail!(err_msg);
        }
    }
    Ok(())
}

/// Read sequences of `TokenKind::Atom` and `TokenKind::Escaped` into a String.
fn read_ident(lexer: &mut Lexer, err_msg: &'static str) -> crate::Result<String> {
    let mut output = String::new();
    loop {
        let token = lexer.next();
        match token.kind() {
            TokenKind::Escaped(seq) => {
                output.push('{');
                output.extend(seq);
                output.push('}');
            }
            TokenKind::Atom(c) => output.push(*c),
            _ => {
                lexer.push(token);
                break;
            }
        }
    }
    match output.len() {
        0 => bail!(err_msg),
        _ => Ok(output),
    }
}

#[derive(Debug)]
struct Lexer {
    tokens: Vec<Token>,
}

impl Lexer {
    /// Parse a string into a list of tokens.
    pub(crate) fn tokenize(mut input: &str) -> crate::Result<Self> {
        let mut tokens = vec![];
        let mut loc = Location::default();
        while !input.is_empty() {
            let old_input = input;
            let mut chars = input.chars();
            let kind = match chars.next().unwrap() {
                // c if c.is_ascii_whitespace() => continue,
                ':' => TokenKind::Colon,
                '=' => TokenKind::Eq,
                '\\' => TokenKind::BSlash,
                '/' => TokenKind::FSlash,
                ';' => TokenKind::Semi,
                '{' => {
                    let mut buf = Vec::new();
                    // Read alphanumeric ASCII including whitespace until we find a closing curly.
                    loop {
                        match chars.next() {
                            None => bail!("unclosed escape literal"),
                            Some('}') => break,
                            Some(c) if c.is_ascii() => buf.push(c),
                            Some(c) => bail!("Invalid JDBC token `{}`", c),
                        }
                    }
                    TokenKind::Escaped(buf)
                }
                c if c.is_ascii_alphanumeric() => TokenKind::Atom(c),
                c if c.is_ascii_whitespace() => TokenKind::Atom(c),
                c => bail!("Invalid JDBC token `{}`", c),
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

/// A pair of `Location` and `TokenKind`.
#[derive(Debug, Clone)]
struct Token {
    loc: Location,
    kind: TokenKind,
}

impl Token {
    /// What kind of token is this?
    pub(crate) fn kind(&self) -> &TokenKind {
        &self.kind
    }
}

/// The kind of token we're encoding.
#[derive(Debug, Clone, Eq, PartialEq)]
enum TokenKind {
    Colon,
    Eq,
    BSlash,
    FSlash,
    Semi,
    /// An ident that falls inside a `{}` pair.
    Escaped(Vec<char>),
    /// An identifier in the connection string.
    Atom(char),
    Eof,
}

#[cfg(test)]
mod test {
    use super::JdbcString;

    #[test]
    fn parse_sub_protocol() -> crate::Result<()> {
        let conn: JdbcString = "jdbc:sqlserver://".parse()?;
        assert_eq!(conn.sub_protocol(), "jdbc:sqlserver");
        Ok(())
    }

    #[test]
    fn parse_server_name() -> crate::Result<()> {
        let conn: JdbcString = r#"jdbc:sqlserver://server"#.parse()?;
        assert_eq!(conn.sub_protocol(), "jdbc:sqlserver");
        assert_eq!(conn.server_name(), Some("server"));
        Ok(())
    }

    #[test]
    fn parse_instance_name() -> crate::Result<()> {
        let conn: JdbcString = r#"jdbc:sqlserver://server\instance"#.parse()?;
        assert_eq!(conn.sub_protocol(), "jdbc:sqlserver");
        assert_eq!(conn.server_name(), Some("server"));
        assert_eq!(conn.instance_name(), Some("instance"));
        Ok(())
    }

    #[test]
    fn parse_port() -> crate::Result<()> {
        let conn: JdbcString = r#"jdbc:sqlserver://server\instance:80"#.parse()?;
        assert_eq!(conn.sub_protocol(), "jdbc:sqlserver");
        assert_eq!(conn.server_name(), Some("server"));
        assert_eq!(conn.instance_name(), Some("instance"));
        assert_eq!(conn.port(), Some(80));
        Ok(())
    }

    #[test]
    fn parse_properties() -> crate::Result<()> {
        let conn: JdbcString =
            r#"jdbc:sqlserver://server\instance:80;key=value;foo=bar"#.parse()?;
        assert_eq!(conn.sub_protocol(), "jdbc:sqlserver");
        assert_eq!(conn.server_name(), Some("server"));
        assert_eq!(conn.instance_name(), Some("instance"));
        assert_eq!(conn.port(), Some(80));

        let kv = conn.properties();
        assert_eq!(kv.get("foo"), Some(&"bar".to_string()));
        assert_eq!(kv.get("key"), Some(&"value".to_string()));
        Ok(())
    }

    #[test]
    fn escaped_properties() -> crate::Result<()> {
        let conn: JdbcString =
            r#"jdbc:sqlserver://se{r}ver{;}\instance:80;key={va[]}lue"#.parse()?;
        assert_eq!(conn.sub_protocol(), "jdbc:sqlserver");
        assert_eq!(conn.server_name(), Some("se{r}ver{;}"));
        assert_eq!(conn.instance_name(), Some("instance"));
        assert_eq!(conn.port(), Some(80));

        let kv = conn.properties();
        assert_eq!(kv.get("key"), Some(&"{va[]}lue".to_string()));
        Ok(())
    }

    #[test]
    fn sub_protocol_error() -> crate::Result<()> {
        let err = r#"jdbc:sqlboo://"#.parse::<JdbcString>().unwrap_err().to_string();
        assert_eq!(
            err.to_string(),
            "Conversion error: Invalid JDBC sub-protocol"
        );
        Ok(())
    }

    #[test]
    fn whitespace() -> crate::Result<()> {
        dbg!("start");
        let conn: JdbcString =
            r#"jdbc:sqlserver://server\instance:80;key=value;foo=bar;user id=musti naukio"#
                .parse()?;
        assert_eq!(conn.sub_protocol(), "jdbc:sqlserver");
        assert_eq!(conn.server_name(), Some(r#"server"#));
        assert_eq!(conn.instance_name(), Some("instance"));
        assert_eq!(conn.port(), Some(80));

        let kv = conn.properties();
        assert_eq!(kv.get("user id"), Some(&"musti naukio".to_string()));
        Ok(())
    }
}
