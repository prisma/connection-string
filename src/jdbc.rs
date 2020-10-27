use std::str::FromStr;
use std::{collections::HashMap, fmt::Display};

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
    sub_protocol: String,
    server_name: Option<String>,
    instance_name: Option<String>,
    port: Option<u16>,
    properties: HashMap<String, String>,
}

impl JdbcString {
    /// Access the connection sub-protocol
    pub fn sub_protocol(&self) -> &str {
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

    /// Mutably access the connection's key-value pairs
    pub fn properties_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.properties
    }
}

impl Display for JdbcString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        /// Escape all non-alphanumeric characters in a string..
        fn escape(s: &str) -> String {
            let mut output = String::with_capacity(s.len());
            let mut escaping = false;
            for b in s.chars() {
                if matches!(b, ':' | '=' | '\\' | '/' | ';' | '{' | '}' | '[' | ']') {
                    if !escaping {
                        escaping = true;
                        output.push('{');
                    }
                    output.push(b);
                } else {
                    if escaping {
                        escaping = false;
                        output.push('}');
                    }
                    output.push(b);
                }
            }
            if escaping {
                output.push('}');
            }
            output
        }

        write!(f, "{}://", self.sub_protocol)?;
        if let Some(server_name) = &self.server_name {
            write!(f, "{}", escape(server_name))?;
        }
        if let Some(instance_name) = &self.instance_name {
            write!(f, r#"\{}"#, escape(instance_name))?;
        }
        if let Some(port) = self.port {
            write!(f, ":{}", port)?;
        }

        for (k, v) in self.properties().iter() {
            write!(f, ";{}={}", escape(k.trim()), escape(v.trim()))?;
        }
        Ok(())
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
        let sub_protocol = format!("jdbc:{}", read_ident(&mut lexer, err)?);

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

            // Handle trailing semis.
            if let TokenKind::Eof = lexer.peek().kind() {
                let _ = lexer.next();
                break;
            }

            let err = "Invalid property key";
            let key = read_ident(&mut lexer, err)?.to_lowercase();

            let err = "Property pairs must be joined by a `=`";
            ensure!(lexer.next().kind() == &TokenKind::Eq, err);

            let err = "Invalid property value";
            let value = read_ident(&mut lexer, err)?;

            properties.insert(key, value);
        }

        let token = lexer.next();
        ensure!(token.kind() == &TokenKind::Eof, "Invalid JDBC token");

        Ok(Self {
            sub_protocol,
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
            TokenKind::Escaped(seq) => output.extend(seq),
            TokenKind::Atom(c) => output.push(*c),
            _ => {
                // push the token back in the lexer
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
                c if c.is_ascii() => TokenKind::Atom(c),
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
        assert_eq!(conn.server_name(), Some("server;"));
        assert_eq!(conn.instance_name(), Some("instance"));
        assert_eq!(conn.port(), Some(80));

        let kv = conn.properties();
        assert_eq!(kv.get("key"), Some(&"va[]lue".to_string()));
        Ok(())
    }

    #[test]
    fn sub_protocol_error() -> crate::Result<()> {
        let err = r#"jdbq:sqlserver://"#.parse::<JdbcString>().unwrap_err().to_string();
        assert_eq!(
            err.to_string(),
            "Conversion error: Invalid JDBC sub-protocol"
        );
        Ok(())
    }

    #[test]
    fn whitespace() -> crate::Result<()> {
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

    // Test for dashes and dots in the name, and parse names other than oracle
    #[test]
    fn regression_2020_10_06() -> crate::Result<()> {
        let input = "jdbc:sqlserver://my-server.com:5433;foo=bar";
        let _conn: JdbcString = input.parse()?;

        let input = "jdbc:oracle://foo.bar:1234";
        let _conn: JdbcString = input.parse()?;

        Ok(())
    }

    /// While strictly disallowed, we should not fail if we detect a trailing semi.
    #[test]
    fn regression_2020_10_07_handle_trailing_semis() -> crate::Result<()> {
        let input = "jdbc:sqlserver://my-server.com:5433;foo=bar;";
        let _conn: JdbcString = input.parse()?;

        let input = "jdbc:sqlserver://my-server.com:4200;User ID=musti;Password={abc;}}45}";
        let conn: JdbcString = input.parse()?;
        let props = conn.properties();
        assert_eq!(props.get("user id"), Some(&"musti".to_owned()));
        assert_eq!(props.get("password"), Some(&"abc;}45}".to_owned()));
        Ok(())
    }

    #[test]
    fn display_with_escaping() -> crate::Result<()> {
        let input = r#"jdbc:sqlserver://server{;}\instance:80;key=va{[]}lue"#;
        let conn: JdbcString = input.parse()?;

        assert_eq!(format!("{}", conn), input);
        Ok(())
    }

    // Output was being over-escaped and not split with semis, causing all sorts of uri failures.
    #[test]
    fn regression_2020_10_27_dont_escape_underscores_whitespace() -> crate::Result<()> {
        let input = r#"jdbc:sqlserver://test-db-mssql-2017:1433;user=SA;encrypt=DANGER_PLAINTEXT;isolationlevel=READ UNCOMMITTED;schema=NonEmbeddedUpsertDesignSpec;trustservercertificate=true;password=<YourStrong@Passw0rd>"#;
        let conn: JdbcString = input.parse()?;

        let output = format!("{}", conn);
        let mut output: Vec<String> = output.split(';').map(|s| s.to_owned()).collect();
        output.pop();
        output.sort();

        let input = format!("{}", conn);
        let mut input: Vec<String> = input.split(';').map(|s| s.to_owned()).collect();
        input.pop();
        input.sort();

        assert_eq!(output, input);
        Ok(())
    }
}
