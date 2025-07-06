use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Userhost;

impl TryFrom<ParameterList> for Userhost {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Userhost, ClientMessageError> {
        todo!()
    }
}
