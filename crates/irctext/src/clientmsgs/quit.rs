use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Quit;

impl TryFrom<ParameterList> for Quit {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Quit, ClientMessageError> {
        todo!()
    }
}
