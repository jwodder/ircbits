use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Whois;

impl TryFrom<ParameterList> for Whois {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Whois, ClientMessageError> {
        todo!()
    }
}
