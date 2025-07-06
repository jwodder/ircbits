use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Topic;

impl TryFrom<ParameterList> for Topic {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Topic, ClientMessageError> {
        todo!()
    }
}
