use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Part;

impl TryFrom<ParameterList> for Part {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Part, ClientMessageError> {
        todo!()
    }
}
