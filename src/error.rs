use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("{0}")]
	Msg(String),

	#[error("Given id for vert is not exist")]
	NoVert,

	#[error("Two edge have the same start and end")]
	DoubleEdge,

	#[error("Edge pointing to itself")]
	SelfEdge,

	#[error("Error in the file used to build the graph, in line {0}: {1}")]
	FileWrong(usize, String),

	#[error(transparent)]
	IoError(#[from] ::std::io::Error),
}

impl From<&'static str> for Error {
	fn from(s: &'static str) -> Self {
		Error::Msg(s.to_owned())
	}
}

impl From<String> for Error {
	fn from(s: String) -> Self {
		Error::Msg(s)
	}
}

pub type Result<T> = ::std::result::Result<T, Error>;
