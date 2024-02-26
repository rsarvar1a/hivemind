pub type Result<T> = anyhow::Result<T, Error>;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Represents an application error in Hivemind.
pub struct Error
{
    pub kind: Kind,
    pub msg:  String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind
{
    ConstantContact,
    FreedomToMove,
    GameNotStarted,
    ImmuneToPillbug,
    InternalError,
    InvalidMove,
    InvalidOption,
    InvalidState,
    InvalidTime,
    IoError,
    LoggerError,
    LogicError,
    MismatchError,
    OneHivePrinciple,
    ParseError,
    PleaseOpenAGithubIssue,
    TooManyUndos,
    UnknownPiece,
    UnrecognizedCommand,
}

impl std::fmt::Display for Error
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{:?}{}{}", self.kind, Error::spacer_no_newline(&self.msg), self.msg)
    }
}

impl From<std::io::Error> for Error
{
    fn from(value: std::io::Error) -> Self
    {
        Error {
            kind: Kind::IoError,
            msg:  value.to_string(),
        }
    }
}

impl From<flexi_logger::FlexiLoggerError> for Error
{
    fn from(value: flexi_logger::FlexiLoggerError) -> Self
    {
        Error {
            kind: Kind::LoggerError,
            msg:  value.to_string(),
        }
    }
}

impl Error
{
    /// Chains an error into an error stack.
    pub fn chain(&self, base: Error) -> Error
    {
        let self_as = &format!("{}", self);
        let msg = format!("{}{}{}", base.msg, Error::spacer(self_as), self_as);
        Error::new(base.kind, msg)
    }

    /// Chains a parse error.
    pub fn chain_parse<T>(&self, s: String) -> Error
    {
        let base = Error::for_parse::<T>(s);
        self.chain(base)
    }

    /// Creates an error with no message.
    pub fn empty(kind: Kind) -> Error
    {
        Error::new(kind, "".into())
    }

    /// Whether this error is fatal or recoverable.
    pub fn fatal(&self) -> bool
    {
        matches!(self.kind, Kind::InternalError | Kind::IoError | Kind::PleaseOpenAGithubIssue)
    }

    /// Creates a parse error for a particular type.
    pub fn for_parse<T>(s: String) -> Error
    {
        let type_name_base = Error::type_name::<T>();
        let err_msg = format!("'{}' is not a valid {}.", s, type_name_base);
        Error::new(Kind::ParseError, err_msg)
    }

    /// Creates a holy shit error, somewhere that I put in error handling but reasonably should never see in "production".
    pub fn holy_shit(err: Error) -> Error
    {
        err.chain(Error::new(Kind::PleaseOpenAGithubIssue, "Something has gone terribly wrong.".into()))
    }

    pub fn mismatch<T: std::fmt::Display>(expected: T, actual: T) -> Error
    {
        let type_name_base = Error::type_name::<T>();
        let err_msg = format!("Mismatched {}s (expected {}, actual {})", type_name_base, expected, actual);
        Error::new(Kind::MismatchError, err_msg)
    }

    /// Creates a new error.
    pub fn new(kind: Kind, msg: String) -> Error
    {
        Error { kind, msg }
    }

    #[deprecated(note = "missing an implementation here")]
    /// A placeholder error for incomplete features.
    pub fn not_implemented() -> Error
    {
        Error::new(Kind::InternalError, "Not implemented.".into())
    }

    /// Gives the message changing spacer for the given string.
    fn spacer(s: &str) -> &'static str
    {
        if s.is_empty()
        {
            ""
        }
        else
        {
            "\n\tdue to "
        }
    }

    /// An inline spacer.
    fn spacer_no_newline(s: &str) -> &'static str
    {
        if s.is_empty()
        {
            ""
        }
        else
        {
            ": "
        }
    }

    /// Computes the basename for the parameterized type.
    pub fn type_name<T>() -> &'static str
    {
        let type_name = std::any::type_name::<T>();
        let type_name_base = type_name.split("::").last().unwrap_or(type_name);
        type_name_base
    }
}
