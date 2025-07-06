use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Oper;

impl TryFrom<ParameterList> for Oper {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Oper, ClientMessageError> {
        todo!()
    }
}
