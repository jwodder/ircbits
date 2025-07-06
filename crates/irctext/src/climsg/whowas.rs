use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Whowas;

impl TryFrom<ParameterList> for Whowas {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Whowas, ClientMessageError> {
        todo!()
    }
}
