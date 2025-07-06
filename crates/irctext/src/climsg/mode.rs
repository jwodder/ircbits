use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mode;

impl TryFrom<ParameterList> for Mode {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Mode, ClientMessageError> {
        todo!()
    }
}
