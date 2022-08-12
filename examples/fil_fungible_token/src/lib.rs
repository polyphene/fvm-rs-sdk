mod token;

use crate::token::state::TokenState;
use crate::token::TokenError;

use crate::token::types::{
    AllowanceReturn, BalanceReturn, BurnParams, BurnReturn, ChangeAllowanceParams, FrcXXXToken,
    GetAllowanceParams, MintParams, MintReturn, RevokeAllowanceParams, SupplyReturn,
    TransferParams, TransferReturn,
};
use fvm_rs_sdk::actor::{fvm_actor, fvm_export};
use fvm_rs_sdk::shared::address::Address;
use fvm_rs_sdk::shared::econ::TokenAmount;
use fvm_rs_sdk::shared::error::ExitCode;
use fvm_rs_sdk::shared::ActorID;
use fvm_rs_sdk::state::StateObject;
use fvm_rs_sdk::syscall::actor::resolve_address;
use fvm_rs_sdk::syscall::message::caller;

use fvm_rs_sdk::shared::bigint::bigint_ser::BigIntDe;
use num_traits::Signed;
use num_traits::Zero;
use std::ops::Neg;

#[fvm_actor]
impl FrcXXXToken for TokenState {
    #[fvm_export(binding = 1)]
    fn name(&self) -> String {
        self.name.clone()
    }
    #[fvm_export(binding = 2)]
    fn symbol(&self) -> String {
        self.symbol.clone()
    }
    #[fvm_export(binding = 3)]
    fn total_supply(&self) -> SupplyReturn {
        self.supply.clone()
    }
    #[fvm_export(binding = 4)]
    fn balance_of(&self, params: Address) -> BalanceReturn {
        let id = expect_id(&params);
        let balance = match self.get_balance(id) {
            Ok(balance) => balance,
            Err(err) => err.abort(),
        };

        BigIntDe(balance)
    }
    #[fvm_export(binding = 5)]
    fn increase_allowance(&mut self, params: ChangeAllowanceParams) -> AllowanceReturn {
        if params.amount.0.is_negative() {
            TokenError::InvalidNegative(format!(
                "increase allowance delta {} cannot be negative",
                params.amount.0
            ))
            .abort()
        }

        let owner = expect_id(&params.owner);
        let operator = expect_id(&params.operator);

        let new_amount = match self.change_allowance_by(owner, operator, &params.amount.0) {
            Ok(amount) => amount,
            Err(err) => err.abort(),
        };

        AllowanceReturn {
            owner: params.owner,
            operator: params.operator,
            amount: BigIntDe(new_amount),
        }
    }
    #[fvm_export(binding = 6)]
    fn decrease_allowance(&mut self, params: ChangeAllowanceParams) -> AllowanceReturn {
        if params.amount.0.is_negative() {
            TokenError::InvalidNegative(format!(
                "decrease allowance delta {} cannot be negative",
                params.amount.0
            ))
            .abort()
        }

        let owner = expect_id(&params.owner);
        let operator = expect_id(&params.operator);

        let new_allowance = match self.change_allowance_by(owner, operator, &params.amount.0.neg())
        {
            Ok(amount) => amount,
            Err(err) => err.abort(),
        };

        AllowanceReturn {
            owner: params.owner,
            operator: params.operator,
            amount: BigIntDe(new_allowance),
        }
    }
    #[fvm_export(binding = 7)]
    fn revoke_allowance(&mut self, params: RevokeAllowanceParams) -> AllowanceReturn {
        let owner = expect_id(&params.owner);
        let operator = expect_id(&params.operator);

        if let Err(err) = self.attempt_revoke_allowance(owner, operator) {
            err.abort()
        }

        AllowanceReturn {
            owner: params.owner,
            operator: params.operator,
            amount: BigIntDe(TokenAmount::zero()),
        }
    }
    #[fvm_export(binding = 8)]
    fn allowance(&self, params: GetAllowanceParams) -> AllowanceReturn {
        let owner = expect_id(&params.owner);
        let operator = expect_id(&params.operator);

        let allowance = match self.get_allowance_between(owner, operator) {
            Ok(amount) => amount,
            Err(err) => err.abort(),
        };
        AllowanceReturn {
            owner: params.owner,
            operator: params.operator,
            amount: BigIntDe(allowance),
        }
    }
    #[fvm_export(binding = 9)]
    fn mint(&mut self, params: MintParams) -> MintReturn {
        if params.amount.0.is_negative() {
            TokenError::InvalidNegative(format!(
                "mint amount {} cannot be negative",
                params.amount.0
            ))
            .abort()
        }

        // Resolve to id addresses
        // TODO might fail, we'll see
        let operator = caller();
        let owner = expect_id(&params.initial_owner);

        if operator != self.owner {
            TokenError::CallerNotOwner(operator, self.owner).abort();
        }

        // Increase the balance of the actor and increase total supply
        if let Err(err) = self.change_balance_by(owner, &params.amount.0) {
            err.abort()
        }
        let new_supply = match self.change_supply_by(&params.amount.0) {
            Ok(amount) => amount,
            Err(err) => err.abort(),
        };

        MintReturn {
            newly_minted: params.amount,
            total_supply: BigIntDe(new_supply.clone()),
        }
    }
    #[fvm_export(binding = 10)]
    fn burn(&mut self, params: BurnParams) -> BurnReturn {
        if params.amount.0.is_negative() {
            TokenError::InvalidNegative(format!(
                "burn amount {} cannot be negative",
                params.amount.0
            ))
            .abort()
        }

        // owner and operator must exist to burn from
        // TODO might fail, to see
        let owner = expect_id(&params.owner);
        let operator = caller();

        if operator != owner {
            // attempt to use allowance and return early if not enough
            if let Err(err) = self.attempt_use_allowance(operator, owner, &params.amount.0) {
                err.abort()
            }
        }
        // attempt to burn the requested amount
        let new_amount = match self.change_balance_by(owner, &params.amount.0.clone().neg()) {
            Ok(amount) => amount,
            Err(err) => err.abort(),
        };

        // decrease total_supply
        if let Err(err) = self.change_supply_by(&params.amount.0.clone().neg()) {
            err.abort()
        }

        BurnReturn {
            owner: params.owner,
            burnt: params.amount,
            remaining_balance: BigIntDe(new_amount),
        }
    }
    #[fvm_export(binding = 11)]
    fn transfer(&mut self, params: TransferParams) -> TransferReturn {
        if params.amount.0.is_negative() {
            TokenError::InvalidNegative(format!(
                "transfer amount {} cannot be negative",
                params.amount.0
            ))
            .abort()
        }

        // operator must be an id address
        // TODO might fail
        let operator = caller();
        // resolve owner and receiver
        let from = expect_id(&params.from);
        let to = expect_id(&params.to);

        if operator != from {
            // attempt to use allowance and return early if not enough
            if let Err(err) = self.attempt_use_allowance(operator, from, &params.amount.0) {
                err.abort()
            }
        }
        if let Err(err) = self.change_balance_by(to, &params.amount.0) {
            err.abort()
        }
        if let Err(err) = self.change_balance_by(from, &params.amount.0.clone().neg()) {
            err.abort()
        }

        TransferReturn {
            from: params.from,
            to: params.to,
            amount: params.amount,
        }
    }
}

/// Expects an address to be an ID address and returns the ActorID
///
/// If it is not an ID address, this function returns a TokenError::InvalidIdAddress error
fn expect_id(address: &Address) -> ActorID {
    match resolve_address(address) {
        Some(id) => id,
        None => TokenError::InvalidIdAddress(*address).abort(),
    }
}
