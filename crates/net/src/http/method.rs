use std::{fmt::Display, str::FromStr};

/// HTTP methods that can be used in a fetch request.
/// The methods as defined by the [fetch spec](https://fetch.spec.whatwg.org/#methods).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Method {
    /// The OPTIONS method represents a request for information about the communication options
    Options,
    /// The GET method means retrieve whatever information (in the form of an entity) is
    /// identified by the Request-URI.
    Get,
    /// The POST method is used to request that the origin server accept the entity enclosed
    /// in the request as a new subordinate of the resource identified by the Request-URI
    /// in the Request-Line.
    Post,
    /// The PUT method requests that the enclosed entity be stored under the supplied Request-URI.
    Put,
    /// The DELETE method requests that the origin server delete the resource identified by the Request-URI.
    Delete,
    /// The HEAD method is identical to GET except that the server MUST NOT return a message-body in the response.
    Head,
    /// The PATCH method requests that a set of changes described in the
    /// request entity be applied to the resource identified by the Request-URI.
    Patch,
}

impl Method {
    /// Returns a `Method` from the given bytes.
    pub fn from_bytes(src: &[u8]) -> Option<Method> {
        match src {
            b"OPTIONS" => Some(Method::Options),
            b"GET" => Some(Method::Get),
            b"POST" => Some(Method::Post),
            b"PUT" => Some(Method::Put),
            b"DELETE" => Some(Method::Delete),
            b"HEAD" => Some(Method::Head),
            b"PATCH" => Some(Method::Patch),
            _ => None,
        }
    }
}

impl AsRef<str> for Method {
    fn as_ref(&self) -> &str {
        match *self {
            Method::Options => "OPTIONS",
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Head => "HEAD",
            Method::Patch => "PATCH",
        }
    }
}

impl FromStr for Method {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OPTIONS" => Ok(Method::Options),
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "HEAD" => Ok(Method::Head),
            "PATCH" => Ok(Method::Patch),
            _ => Err(Error::InvalidMethod),
        }
    }
}

/// An error that can be returned when converting `Method` from a string slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidMethod,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Error::InvalidMethod => write!(f, "Invalid method"),
        }
    }
}
