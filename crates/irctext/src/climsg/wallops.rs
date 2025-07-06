use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Wallops;

impl TryFrom<ParameterList> for Wallops {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Wallops, ClientMessageError> {
        todo!()
    }
}
