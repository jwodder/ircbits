use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Kick;

impl TryFrom<ParameterList> for Kick {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Kick, ClientMessageError> {
        todo!()
    }
}
