use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pass;

impl TryFrom<ParameterList> for Pass {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Pass, ClientMessageError> {
        todo!()
    }
}
