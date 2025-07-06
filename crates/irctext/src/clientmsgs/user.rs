use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User;

impl TryFrom<ParameterList> for User {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<User, ClientMessageError> {
        todo!()
    }
}
