use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rehash;

impl TryFrom<ParameterList> for Rehash {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Rehash, ClientMessageError> {
        todo!()
    }
}
