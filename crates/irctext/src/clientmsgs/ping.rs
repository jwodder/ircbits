use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ping;

impl TryFrom<ParameterList> for Ping {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Ping, ClientMessageError> {
        todo!()
    }
}
