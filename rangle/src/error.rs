use thiserror::Error;

use std::io;
use std::num;

#[derive(Error, Debug)]
pub enum RangleError {
    #[error(transparent)]
    Crossterm(#[from] crossterm::ErrorKind),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    ParseFloat(#[from] num::ParseFloatError),
    #[error(transparent)]
    ParseInt(#[from] num::ParseIntError),

    #[error("duplicate attribute names")]
    DuplicateShaderAttributes,
    #[error("duplicate uniform names")]
    DuplicateShaderUniforms,
    #[error("missing required shader")]
    MissingShader,
}
