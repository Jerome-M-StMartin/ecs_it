//Jerome M. St.Martin
//Feb 12, 2023

//-----------------------------------------------------------------------------
//----------------------------- Custom Error Type -----------------------------
//-----------------------------------------------------------------------------

use std::fmt;

#[derive(Debug, Copy, Clone)]
pub(crate) struct ECSError(pub &'static str);

impl fmt::Display for ECSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> std::error::Error for ECSError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ECSError(s) => None, //idk what I'm doing here
            _ => None,
        }
    }
}
