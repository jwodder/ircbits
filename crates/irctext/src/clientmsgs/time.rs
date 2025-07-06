use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Time;

impl TryFrom<ParameterList> for Time {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Time, ClientMessageError> {
        todo!()
    }
}
