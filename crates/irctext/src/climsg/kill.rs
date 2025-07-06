use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Kill;

impl TryFrom<ParameterList> for Kill {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Kill, ClientMessageError> {
        todo!()
    }
}
