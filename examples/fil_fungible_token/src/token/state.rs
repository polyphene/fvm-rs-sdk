use fvm_rs_sdk::shared::bigint::bigint_ser::BigIntDe;
use fvm_rs_sdk::shared::bigint::Zero;
use fvm_rs_sdk::shared::econ::TokenAmount;
use fvm_rs_sdk::shared::ActorID;
use fvm_rs_sdk::state::*;

use crate::ExitCode;
use num_traits::Signed;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("negative balance caused by changing {owner:?}'s balance of {balance:?} by {delta:?}")]
    NegativeBalance {
        owner: ActorID,
        balance: TokenAmount,
        delta: TokenAmount,
    },
    #[error(
        "{operator:?} attempted to utilise {delta:?} of allowance {allowance:?} set by {owner:?}"
    )]
    InsufficentAllowance {
        owner: ActorID,
        operator: ActorID,
        allowance: TokenAmount,
        delta: TokenAmount,
    },
    #[error("total_supply cannot be negative, cannot apply delta of {delta:?} to {supply:?}")]
    NegativeTotalSupply {
        supply: TokenAmount,
        delta: TokenAmount,
    },
}

impl StateError {
    pub fn abort(&self) -> ! {
        fvm_rs_sdk::syscall::vm::abort(
            ExitCode::USR_UNSPECIFIED.value(),
            Some(&format!("{}", &self)),
        )
    }
}

type Result<T> = std::result::Result<T, StateError>;

/// Token state structure
#[derive(Clone, Debug)]
#[fvm_state]
pub struct TokenState {
    /// Name of the token
    pub name: String,
    /// Symbol of the token
    pub symbol: String,
    /// Owner of the actor
    pub owner: ActorID,
    /// Total supply of token
    pub supply: BigIntDe,
    /// HashMap<ActorID, TokenAmount> of balances as a Hamt
    pub balances: HashMap<ActorID, BigIntDe>,
    /// HashMap<ActorId, HashMap<ActorID, TokenAmount>> as a Hamt. Allowances are stored balances[owner][operator]
    pub allowances: HashMap<ActorID, HashMap<ActorID, BigIntDe>>,
}

impl TokenState {
    /// Get the balance of an ActorID from the currently stored state
    pub fn get_balance(&self, owner: ActorID) -> Result<TokenAmount> {
        Ok(match self.balances.get(&owner) {
            Some(amount) => amount.clone().0,
            None => TokenAmount::zero(),
        })
    }

    /// Changes the balance of the specified account by the delta
    ///
    /// Caller must ensure that the sign of of the delta is consistent with token rules (i.e.
    /// negative transfers, burns etc. are not allowed)
    pub fn change_balance_by(
        &mut self,
        owner: ActorID,
        delta: &TokenAmount,
    ) -> Result<TokenAmount> {
        if delta.is_zero() {
            // This is a no-op as far as mutating state
            return self.get_balance(owner);
        }

        let balance = self.balances.get(&owner);

        let new_balance = match balance {
            Some(existing_amount) => existing_amount.clone().0 + delta,
            None => (*delta).clone(),
        };

        // if the new_balance is negative, return an error
        if new_balance.is_negative() {
            return Err(StateError::NegativeBalance {
                balance: new_balance,
                delta: delta.clone(),
                owner,
            });
        }

        self.balances.insert(owner, BigIntDe(new_balance.clone()));

        Ok(new_balance)
    }

    /// Increase/decrease the total supply by the specified value
    ///
    /// Returns the new total supply
    pub fn change_supply_by(&mut self, delta: &TokenAmount) -> Result<&TokenAmount> {
        let new_supply = &self.supply.0 + delta;
        if new_supply.is_negative() {
            return Err(StateError::NegativeTotalSupply {
                supply: self.supply.0.clone(),
                delta: delta.clone(),
            });
        }

        self.supply = BigIntDe(new_supply);
        Ok(&self.supply.0)
    }

    /// Get the allowance that an owner has approved for a operator
    ///
    /// If an existing allowance cannot be found, it is implicitly assumed to be zero
    pub fn get_allowance_between(&self, owner: ActorID, operator: ActorID) -> Result<TokenAmount> {
        let owner_allowances = self.allowances.get(&owner);
        match owner_allowances {
            Some(allowances) => match allowances.get(&operator) {
                Some(token_amount) => Ok(token_amount.clone().0),
                None => Ok(TokenAmount::zero()),
            },
            None => Ok(TokenAmount::zero()),
        }
    }

    /// Change the allowance between owner and operator by the specified delta
    pub fn change_allowance_by(
        &mut self,
        owner: ActorID,
        operator: ActorID,
        delta: &TokenAmount,
    ) -> Result<TokenAmount> {
        if delta.is_zero() {
            // This is a no-op as far as mutating state
            return self.get_allowance_between(owner, operator);
        }

        // get or create the owner's allowance map
        let mut allowance_map = match self.allowances.get(&owner) {
            Some(allowances) => allowances.clone(),
            None => {
                // the owner doesn't have any allowances, and the delta is negative, this is a no-op
                if delta.is_negative() {
                    return Ok(TokenAmount::zero());
                }

                // else create a new map for the owner
                HashMap::<ActorID, BigIntDe>::new()
            }
        };

        // calculate new allowance (max with zero)
        let new_allowance = match allowance_map.get(&operator) {
            Some(existing_allowance) => existing_allowance.clone().0 + delta,
            None => (*delta).clone(),
        }
        .max(TokenAmount::zero());

        // if the new allowance is zero, we can remove the entry from the state tree
        if new_allowance.is_zero() {
            allowance_map.remove(&operator);
        } else {
            allowance_map.insert(operator, BigIntDe(new_allowance.clone()));
        }

        // if the owner-allowance map is empty, remove it from the global allowances map
        if allowance_map.is_empty() {
            self.allowances.remove(&owner);
        } else {
            // else update the global-allowance map
            self.allowances.insert(owner, allowance_map);
        }

        Ok(new_allowance)
    }

    /// Revokes an approved allowance by removing the entry from the owner-operator map
    ///
    /// If that map becomes empty, it is removed from the root map.
    pub fn attempt_revoke_allowance(&mut self, owner: ActorID, operator: ActorID) -> Result<()> {
        let allowance_map = self.allowances.get_mut(&owner);
        if let Some(map) = allowance_map {
            map.remove(&operator);
            if map.is_empty() {
                self.allowances.remove(&owner);
            }
        }

        Ok(())
    }

    /// Atomically checks if value is less than the allowance and deducts it if so
    ///
    /// Returns new allowance if successful, else returns an error and the allowance is unchanged
    pub fn attempt_use_allowance(
        &mut self,
        operator: u64,
        owner: u64,
        amount: &TokenAmount,
    ) -> Result<TokenAmount> {
        let current_allowance = self.get_allowance_between(owner, operator)?;

        if amount.is_zero() {
            return Ok(current_allowance);
        }

        if current_allowance.lt(amount) {
            return Err(StateError::InsufficentAllowance {
                owner,
                operator,
                allowance: current_allowance,
                delta: amount.clone(),
            });
        }

        let new_allowance = current_allowance - amount;

        let owner_allowances = self.allowances.get_mut(&owner);
        // to reach here, allowance must have been previously non zero; so safe to assume the map exists
        let owner_allowances = owner_allowances.unwrap();
        // TODO Might not work
        owner_allowances.insert(operator, BigIntDe(new_allowance.clone()));

        Ok(new_allowance)
    }
}
