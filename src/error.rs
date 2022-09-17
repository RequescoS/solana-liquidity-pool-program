use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum PoolError {
    #[error("Trying to provide liquidity without offer both tokens")]
    ZeroProvide,

    #[error("Trying to provide more tokens than possessed")]
    OverProvide,

    #[error("Wrong ratio of providing tokens")]
    SlippageFail,

    #[error("Trying to buy more tokens than present in the pool")]
    OverBuy,

    #[error("Trying to buy more tokens than can to pay")]
    TooMuchBuy,

    #[error("Trying to withdraw more liquidity than possessed")]
    OverWithdraw,

    #[error("User signature is required")]
    SignedRequired,

    #[error("Wrong withdraw account")]
    WrongWithdraw,
}

impl From<PoolError> for ProgramError {
    fn from(e: PoolError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
