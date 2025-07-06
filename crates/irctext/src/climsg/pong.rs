use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pong;

impl TryFrom<ParameterList> for Pong {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Pong, ClientMessageError> {
        todo!()
    }
}
