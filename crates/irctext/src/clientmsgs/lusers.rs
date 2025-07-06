use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lusers;

impl TryFrom<ParameterList> for Lusers {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Lusers, ClientMessageError> {
        todo!()
    }
}
