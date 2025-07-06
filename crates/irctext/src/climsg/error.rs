use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error;

impl TryFrom<ParameterList> for Error {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Error, ClientMessageError> {
        todo!()
    }
}
