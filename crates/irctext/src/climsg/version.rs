use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Version;

impl TryFrom<ParameterList> for Version {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Version, ClientMessageError> {
        todo!()
    }
}
