use super::ClientMessageError;
use crate::ParameterList;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stats;

impl TryFrom<ParameterList> for Stats {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Stats, ClientMessageError> {
        todo!()
    }
}
