use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Links;

impl TryFrom<ParameterList> for Links {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Links, ClientMessageError> {
        todo!()
    }
}
