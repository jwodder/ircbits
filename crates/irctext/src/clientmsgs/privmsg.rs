use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivMsg;

impl TryFrom<ParameterList> for PrivMsg {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<PrivMsg, ClientMessageError> {
        todo!()
    }
}
