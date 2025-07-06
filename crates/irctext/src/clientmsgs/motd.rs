use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Motd;

impl TryFrom<ParameterList> for Motd {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Motd, ClientMessageError> {
        todo!()
    }
}
