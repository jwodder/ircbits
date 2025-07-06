use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Join;

impl TryFrom<ParameterList> for Join {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Join, ClientMessageError> {
        todo!()
    }
}
