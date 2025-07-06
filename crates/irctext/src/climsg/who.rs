use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Who;

impl TryFrom<ParameterList> for Who {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Who, ClientMessageError> {
        todo!()
    }
}
