use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Authenticate;

impl TryFrom<ParameterList> for Authenticate {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Authenticate, ClientMessageError> {
        todo!()
    }
}
