use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct List;

impl TryFrom<ParameterList> for List {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<List, ClientMessageError> {
        todo!()
    }
}
