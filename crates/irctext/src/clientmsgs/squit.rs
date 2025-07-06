use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Squit;

impl TryFrom<ParameterList> for Squit {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Squit, ClientMessageError> {
        todo!()
    }
}
