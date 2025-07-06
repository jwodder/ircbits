use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Names;

impl TryFrom<ParameterList> for Names {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Names, ClientMessageError> {
        todo!()
    }
}
