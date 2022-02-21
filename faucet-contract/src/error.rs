use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("CHT is the only denom to be added")]
    WrongDenom {},

    #[error("Please turn off allow for rewards")]
    Allowed {},

}
