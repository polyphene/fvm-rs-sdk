use fvm_rs_sdk::payload::*;
use fvm_rs_sdk::shared::address::Address;
use fvm_rs_sdk::shared::bigint::bigint_ser::BigIntDe;

pub trait FrcXXXToken {
    /// Returns the name of the token
    fn name(&self) -> String;

    /// Returns the ticker symbol of the token
    fn symbol(&self) -> String;

    /// Returns the total amount of the token in existence
    fn total_supply(&self) -> BigIntDe;

    /// Gets the balance of a particular address (if it exists)
    ///
    /// This will method attempt to resolve addresses to ID-addresses
    fn balance_of(&self, params: Address) -> BigIntDe;

    /// Atomically increase the amount that a operator can pull from the owner account
    ///
    /// The increase must be non-negative. Returns the new allowance between those two addresses if
    /// successful
    fn increase_allowance(&mut self, params: ChangeAllowanceParams) -> AllowanceReturn;

    /// Atomically decrease the amount that a operator can pull from an account
    ///
    /// The decrease must be non-negative. The resulting allowance is set to zero if the decrease is
    /// more than the current allowance. Returns the new allowance between the two addresses if
    /// successful
    fn decrease_allowance(&mut self, params: ChangeAllowanceParams) -> AllowanceReturn;

    /// Set the allowance a operator has on the owner's account to zero
    fn revoke_allowance(&mut self, params: RevokeAllowanceParams) -> AllowanceReturn;

    /// Get the allowance between two addresses
    ///
    /// The operator can burn or transfer the allowance amount out of the owner's address. If the
    /// address of the owner cannot be resolved, this method returns an error. If the owner can be
    /// resolved, but the operator address is not registered with an allowance, an implicit allowance
    /// of 0 is returned
    fn allowance(&self, params: GetAllowanceParams) -> AllowanceReturn;

    /// Mint tokens on a given account, increasing the total supply
    ///
    /// When minting token:
    /// - Caller should be actor owner
    fn mint(&mut self, params: MintParams) -> MintReturn;

    /// Burn tokens from the caller's account, decreasing the total supply
    ///
    /// When burning tokens:
    /// - Any owner MUST be allowed to burn their own tokens
    /// - The balance of the owner MUST decrease by the amount burned
    /// - This method MUST revert if the burn amount is more than the owner's balance
    fn burn(&mut self, params: BurnParams) -> BurnReturn;

    /// Transfer tokens from one account to another
    fn transfer(&mut self, params: TransferParams) -> TransferReturn;
}

pub type SupplyReturn = BigIntDe;
pub type BalanceReturn = BigIntDe;
pub type TokenAmount = BigIntDe;

#[fvm_payload]
pub struct MintParams {
    pub initial_owner: Address,
    pub amount: TokenAmount,
}

#[fvm_payload]
pub struct MintReturn {
    pub newly_minted: TokenAmount,
    pub total_supply: TokenAmount,
}

/// An amount to increase or decrease an allowance by
#[fvm_payload]
pub struct ChangeAllowanceParams {
    pub owner: Address,
    pub operator: Address,
    pub amount: TokenAmount,
}

/// Params to get allowance between to addresses
#[fvm_payload]
pub struct GetAllowanceParams {
    pub owner: Address,
    pub operator: Address,
}

/// Instruction to revoke (set to 0) an allowance
#[fvm_payload]
pub struct RevokeAllowanceParams {
    pub owner: Address,
    pub operator: Address,
}

/// The updated value after allowance is increased or decreased
#[fvm_payload]
pub struct AllowanceReturn {
    pub owner: Address,
    pub operator: Address,
    pub amount: TokenAmount,
}

/// Burns an amount of token from an address
#[fvm_payload]
pub struct BurnParams {
    pub owner: Address,
    pub amount: TokenAmount,
}

#[fvm_payload]
pub struct BurnReturn {
    pub owner: Address,
    pub burnt: TokenAmount,
    pub remaining_balance: TokenAmount,
}

#[fvm_payload]
pub struct TransferParams {
    pub from: Address,
    pub to: Address,
    pub amount: TokenAmount,
}

#[fvm_payload]
pub struct TransferReturn {
    pub from: Address,
    pub to: Address,
    pub amount: TokenAmount,
}
