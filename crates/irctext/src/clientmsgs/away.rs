use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Away;

impl TryFrom<ParameterList> for Away {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Away, ClientMessageError> {
        todo!()
    }
}
