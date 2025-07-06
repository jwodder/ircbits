use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Admin;

impl TryFrom<ParameterList> for Admin {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Admin, ClientMessageError> {
        todo!()
    }
}
