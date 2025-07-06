use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Nick;

impl TryFrom<ParameterList> for Nick {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Nick, ClientMessageError> {
        todo!()
    }
}
