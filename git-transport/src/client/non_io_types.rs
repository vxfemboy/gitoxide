/// Configure how the [`RequestWriter`][crate::client::RequestWriter] behaves when writing bytes.
#[derive(PartialEq, Eq, Debug, Hash, Ord, PartialOrd, Clone, Copy)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub enum WriteMode {
    /// Each [write()][std::io::Write::write()] call writes the bytes verbatim as one or more packet lines.
    ///
    /// This mode also indicates to the transport that it should try to stream data as it is unbounded. This mode is typically used
    /// for sending packs whose exact size is not necessarily known in advance.
    Binary,
    /// Each [write()][std::io::Write::write()] call assumes text in the input, assures a trailing newline and writes it as single packet line.
    ///
    /// This mode also indicates that the lines written fit into memory, hence the transport may chose to not stream it but to buffer it
    /// instead. This is relevant for some transports, like the one for HTTP.
    OneLfTerminatedLinePerWriteCall,
}

impl Default for WriteMode {
    fn default() -> Self {
        WriteMode::OneLfTerminatedLinePerWriteCall
    }
}

/// The kind of packet line to write when transforming a [`RequestWriter`][crate::client::RequestWriter] into an
/// [`ExtendedBufRead`][crate::client::ExtendedBufRead].
#[derive(PartialEq, Eq, Debug, Hash, Ord, PartialOrd, Clone, Copy)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub enum MessageKind {
    /// A `flush` packet.
    Flush,
    /// A V2 delimiter.
    Delimiter,
    /// The end of a response.
    ResponseEnd,
    /// The given text.
    Text(&'static [u8]),
}

#[cfg(any(feature = "blocking-client", feature = "async-client"))]
pub(crate) mod connect {
    /// The error used in [`connect()`][crate::connect()].
    #[derive(Debug, thiserror::Error)]
    #[allow(missing_docs)]
    pub enum Error {
        #[error(transparent)]
        Url(#[from] git_url::parse::Error),
        #[error("The git repository path could not be converted to UTF8")]
        PathConversion(#[from] bstr::Utf8Error),
        #[error("connection failed")]
        Connection(#[from] Box<dyn std::error::Error + Send + Sync>),
        #[error("The url {url:?} contains information that would not be used by the {scheme} protocol")]
        UnsupportedUrlTokens {
            url: bstr::BString,
            scheme: git_url::Scheme,
        },
        #[error("The '{0}' protocol is currently unsupported")]
        UnsupportedScheme(git_url::Scheme),
        #[cfg(not(any(feature = "http-client-curl", feature = "http-client-reqwest")))]
        #[error(
            "'{0}' is not compiled in. Compile with the 'http-client-curl' or 'http-client-reqwest' cargo feature"
        )]
        CompiledWithoutHttp(git_url::Scheme),
    }

    // TODO: maybe fix this workaround: want `IsSpuriousError`  in `Connection(…)`
    impl crate::IsSpuriousError for Error {
        fn is_spurious(&self) -> bool {
            match self {
                Error::Connection(err) => {
                    #[cfg(feature = "blocking-client")]
                    if let Some(err) = err.downcast_ref::<crate::client::git::connect::Error>() {
                        return err.is_spurious();
                    };
                    if let Some(err) = err.downcast_ref::<crate::client::Error>() {
                        return err.is_spurious();
                    }
                    false
                }
                _ => false,
            }
        }
    }
}

mod error {
    use bstr::BString;

    use crate::client::capabilities;
    #[cfg(feature = "http-client")]
    use crate::client::http;

    #[cfg(feature = "http-client")]
    type HttpError = http::Error;
    #[cfg(not(feature = "http-client"))]
    type HttpError = std::convert::Infallible;

    /// The error used in most methods of the [`client`][crate::client] module
    #[derive(thiserror::Error, Debug)]
    #[allow(missing_docs)]
    pub enum Error {
        #[error("An IO error occurred when talking to the server")]
        Io(#[from] std::io::Error),
        #[error("Capabilities could not be parsed")]
        Capabilities {
            #[from]
            err: capabilities::Error,
        },
        #[error("A packet line could not be decoded")]
        LineDecode {
            #[from]
            err: git_packetline::decode::Error,
        },
        #[error("A {0} line was expected, but there was none")]
        ExpectedLine(&'static str),
        #[error("Expected a data line, but got a delimiter")]
        ExpectedDataLine,
        #[error("The transport layer does not support authentication")]
        AuthenticationUnsupported,
        #[error("The transport layer refuses to use a given identity: {0}")]
        AuthenticationRefused(&'static str),
        #[error("The protocol version indicated by {:?} is unsupported", {0})]
        UnsupportedProtocolVersion(BString),
        #[error(transparent)]
        Http(#[from] HttpError),
    }

    impl crate::IsSpuriousError for Error {
        fn is_spurious(&self) -> bool {
            match self {
                Error::Io(err) => err.is_spurious(),
                Error::Http(err) => err.is_spurious(),
                _ => false,
            }
        }
    }
}

pub use error::Error;
