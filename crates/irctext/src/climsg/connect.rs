use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Connect;

impl TryFrom<ParameterList> for Connect {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Connect, ClientMessageError> {
        todo!()
    }
}
