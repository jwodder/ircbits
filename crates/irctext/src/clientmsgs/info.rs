use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Info;

impl TryFrom<ParameterList> for Info {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Info, ClientMessageError> {
        todo!()
    }
}
