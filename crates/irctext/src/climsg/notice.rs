use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Notice;

impl TryFrom<ParameterList> for Notice {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Notice, ClientMessageError> {
        todo!()
    }
}
