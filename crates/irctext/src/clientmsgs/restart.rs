use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Restart;

impl TryFrom<ParameterList> for Restart {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Restart, ClientMessageError> {
        todo!()
    }
}
