use anchor_lang::error_code;

#[error_code]
pub enum MarketplaceError {
    #[msg("Name must be less than or equal to 32 characters")]
    NameTooLong,
    #[msg("Fee must be less than or equal to 100")]
    FeeTooHigh,
    #[msg("Invalid collection address")]
    CollectionInvalid,
    #[msg("Collection not verified")]
    CollectionNotVerified,
}