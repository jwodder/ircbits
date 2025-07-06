use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Invite;

impl TryFrom<ParameterList> for Invite {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Invite, ClientMessageError> {
        todo!()
    }
}
