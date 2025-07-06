use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cap;

impl TryFrom<ParameterList> for Cap {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Cap, ClientMessageError> {
        todo!()
    }
}
