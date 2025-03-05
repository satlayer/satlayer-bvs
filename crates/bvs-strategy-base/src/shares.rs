use crate::token;
use cosmwasm_std::{Deps, Env, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::Item;
use std::ops::Div;

/// The offset is used to mitigate the common 'share inflation' attack vector.
/// See [https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack]
/// This 1,000 offset will be used in exchange rate computation to reduce the impact of the attack.
/// When the vault is empty, the virtual shares and virtual assets enforce the conversion rate 1000/1000.
const OFFSET: Uint128 = Uint128::new(1e3 as u128);

/// The total shares of the contract held by all stakers.
/// [`OFFSET`] value is not included in the total shares, only the real shares are counted.
const TOTAL_SHARES: Item<Uint128> = Item::new("total_shares");

/// Get the total shares of the contract
pub fn get_total_shares(storage: &dyn Storage) -> StdResult<Uint128> {
    TOTAL_SHARES.load(storage)
}

/// Set the total shares of the contract
pub fn set_total_shares(storage: &mut dyn Storage, total_shares: &Uint128) -> StdResult<()> {
    TOTAL_SHARES.save(storage, total_shares)
}

/// Follows the OpenZeppelin's ERC4626 mitigation strategy for inflation attack.
/// Using a "virtual" offset to +1e3 to both total shares and balance representing the virtual total shares and virtual balance.
/// A donation of 1e3 and under will be completely captured by the vault—without affecting the user.
/// A donation greater than 1e3, the attacker will suffer loss greater than the user.
/// [https://github.com/OpenZeppelin/openzeppelin-contracts/blob/fa995ef1fe66e1447783cb6038470aba23a6343f/contracts/token/ERC20/extensions/ERC4626.sol#L30-L37]
pub struct VirtualVault {
    pub total_shares: Uint128,
    pub balance: Uint128,
    virtual_total_shares: Uint128,
    virtual_balance: Uint128,
}

impl VirtualVault {
    /// Load the virtual shares from storage and [token::get_balance] (supports rebasing, by default).
    /// A fixed [`OFFSET`] of 1e3 will be added to both total shares and balance
    /// to mitigate against inflation attack.
    /// Use [shares_to_amount] and [amount_to_shares] to convert between shares and amount.
    pub fn load(deps: &Deps, env: &Env) -> StdResult<Self> {
        let total_shares = TOTAL_SHARES.load(deps.storage)?;
        let balance = token::get_balance(deps, env)?;
        Self::new(total_shares, balance)
    }

    fn new(total_shares: Uint128, balance: Uint128) -> StdResult<Self> {
        let virtual_total_shares = total_shares.checked_add(OFFSET).map_err(StdError::from)?;
        let virtual_balance = balance.checked_add(OFFSET).map_err(StdError::from)?;

        Ok(Self {
            total_shares,
            balance,
            virtual_total_shares,
            virtual_balance,
        })
    }

    /// Shares to underlying assets
    pub fn shares_to_amount(&self, shares: Uint128) -> StdResult<Uint128> {
        // (shares * self.virtual_balance) / self.virtual_total_shares
        shares
            .checked_mul(self.virtual_balance)
            .map_err(StdError::from)?
            .checked_div(self.virtual_total_shares)
            .map_err(StdError::from)
    }

    /// Underlying assets to shares
    pub fn amount_to_shares(&self, amount: Uint128) -> StdResult<Uint128> {
        // (amount * self.virtual_total_shares) / self.virtual_balance
        amount
            .checked_mul(self.virtual_total_shares)
            .map_err(StdError::from)?
            .checked_div(self.virtual_balance)
            .map_err(StdError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_to_one() {
        let balance = Uint128::new(1000);
        let total_shares = Uint128::new(1000);
        let vault = VirtualVault::new(total_shares, balance).unwrap();

        {
            let amount = vault.shares_to_amount(Uint128::new(1000)).unwrap();
            assert_eq!(amount, Uint128::new(1000));

            let shares = vault.amount_to_shares(Uint128::new(1000)).unwrap();
            assert_eq!(shares, Uint128::new(1000));
        }

        {
            let amount = vault.shares_to_amount(Uint128::new(100)).unwrap();
            assert_eq!(amount, Uint128::new(100));

            let shares = vault.amount_to_shares(Uint128::new(100)).unwrap();
            assert_eq!(shares, Uint128::new(100));
        }

        {
            let amount = vault.shares_to_amount(Uint128::new(10000)).unwrap();
            assert_eq!(amount, Uint128::new(10000));

            let shares = vault.amount_to_shares(Uint128::new(10000)).unwrap();
            assert_eq!(shares, Uint128::new(10000));
        }
    }

    #[test]
    fn inflation_attack_mitigated_under_1e3() {
        // Attacker deposits 1 to get 1 share
        // Attacker donates 999 moving the balance to 1000
        let balance = Uint128::new(1000);
        let total_shares = Uint128::new(1);

        // Virtual balance: (1000) + 1000 = 2000
        // Virtual shares: (1) + 1000 = 1001
        let vault = VirtualVault::new(total_shares, balance).unwrap();

        // Attacker 1 share is worth 1 amount (fully captured by the vault)
        let amount = vault.shares_to_amount(Uint128::new(1)).unwrap();
        assert_eq!(amount, Uint128::new(1));

        // Normal user deposits 10,000 to get 5,005 shares
        let amount = Uint128::new(10_000);
        let shares = vault.amount_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(5005));

        // Moves the vault.
        let balance = Uint128::new(11_000);
        let total_shares = Uint128::new(5006);
        // Virtual balance: (11,000) + 1000 = 12,000
        // Virtual shares: (5006) + 1000 = 6006
        let vault = VirtualVault::new(total_shares, balance).unwrap();

        // Attacker 1 share is worth 1 amount
        let amount = vault.shares_to_amount(Uint128::new(1)).unwrap();
        assert_eq!(amount, Uint128::new(1));

        // User 5005 shares are worth 10,000 amounts
        let amount = vault.shares_to_amount(Uint128::new(5_005)).unwrap();
        assert_eq!(amount, Uint128::new(10000));
    }

    #[test]
    fn inflation_attack_over_1e3() {
        // Attacker deposits 1 to get 1 share
        // Attacker donates 99,999 moving the balance to 100,000
        let balance = Uint128::new(100_000);
        let total_shares = Uint128::new(1);
        let vault = VirtualVault::new(total_shares, balance).unwrap();

        // Attacker 1 share is worth amount 100 (captured by the vault)
        let amount = vault.shares_to_amount(Uint128::new(1)).unwrap();
        assert_eq!(amount, Uint128::new(100));

        // Normal user deposits 10,000 to get 99 shares
        let amount = Uint128::new(10_000);
        let shares = vault.amount_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(99));

        // Moves the vault.
        let balance = Uint128::new(110_000);
        let total_shares = Uint128::new(100);
        let vault = VirtualVault::new(total_shares, balance).unwrap();

        // Attacker 1 share is worth 100 (captured by the vault)
        let amount = vault.shares_to_amount(Uint128::new(1)).unwrap();
        assert_eq!(amount, Uint128::new(100));

        // User 99 shares are worth 9,900 amounts (captured by the vault)
        let amount = vault.shares_to_amount(Uint128::new(99)).unwrap();
        assert_eq!(amount, Uint128::new(9990));
    }

    #[test]
    fn imbalance_1000_to_1() {
        let balance = Uint128::new(1000);
        let total_shares = Uint128::new(1);

        // Virtual balance: (1000) + 1000 = 2000
        // Virtual shares: (1) + 1000 = 1001
        let vault = VirtualVault::new(total_shares, balance).unwrap();

        // Low amounts
        {
            let shares = Uint128::new(500);
            let amount = vault.shares_to_amount(shares).unwrap();
            // Amount: (500) * 2000 / 1001 = 999
            assert_eq!(amount, Uint128::new(999));

            let amount = Uint128::new(250);
            let shares = vault.amount_to_shares(amount).unwrap();
            // Shares: (250) * 1001 / 2000 = 125
            assert_eq!(shares, Uint128::new(125));
        }

        // High amounts
        {
            let shares = Uint128::new(10_000);
            let amount = vault.shares_to_amount(shares).unwrap();
            // Amount: (10000) * 2000 / 1001 = 19,980.01
            assert_eq!(amount, Uint128::new(19_980));

            let amount = Uint128::new(10_000_000);
            let shares = vault.amount_to_shares(amount).unwrap();
            // Shares: (10000000) * 1001 / 2000 = 5005000
            assert_eq!(shares, Uint128::new(5005000));
        }
    }

    #[test]
    fn imbalance_1000_to_2() {
        let balance = Uint128::new(1000);
        let total_shares = Uint128::new(2);

        // Virtual balance: (1000) + 1000 = 2000
        // Virtual shares: (2) + 1000 = 1002
        let vault = VirtualVault::new(total_shares, balance).unwrap();

        // Low amounts
        {
            let shares = Uint128::new(1000);
            let amount = vault.shares_to_amount(shares).unwrap();
            // Amount: (1000) * 2000 / 1002 = 1996
            assert_eq!(amount, Uint128::new(1996));

            let amount = Uint128::new(1);
            let shares = vault.amount_to_shares(amount).unwrap();
            // Shares: (1) * 1002 / 2000 = 0.501
            assert_eq!(shares, Uint128::new(0));

            let amount = Uint128::new(10);
            let shares = vault.amount_to_shares(amount).unwrap();
            // Shares: (10) * 1002 / 2000 = 5.01
            assert_eq!(shares, Uint128::new(5));
        }

        // High amounts
        {
            let shares = Uint128::new(100_444);
            let amount = vault.shares_to_amount(shares).unwrap();
            // Amount: (100,444) * 2000 / 1002 = 200,487.02
            assert_eq!(amount, Uint128::new(200_487));

            let amount = Uint128::new(10_000_000);
            let shares = vault.amount_to_shares(amount).unwrap();
            // Shares: (10000000) * 1002 / 2000 = 5,010,000
            assert_eq!(shares, Uint128::new(5_010_000));
        }
    }

    /// 1e3 = 1,000
    /// This is 100x over the offset amount
    #[test]
    fn shares_imbalance_100_000_to_1() {
        let balance = Uint128::new(100_000);
        let total_shares = Uint128::new(1);

        // Virtual balance: (100000) + 1000 = 101000
        // Virtual shares: (1) + 1000 = 1001
        let vault = VirtualVault::new(total_shares, balance).unwrap();

        // With 500 shares, they get 50,449
        // Amount: (500) * 101,000 / 1001 = 50,449.55
        let shares = Uint128::new(500);
        let amount = vault.shares_to_amount(shares).unwrap();
        assert_eq!(amount, Uint128::new(50_449));

        // With 1 share, they get 100
        // Amount: (1) * 101,000 / 1001 = 100.89
        let shares = Uint128::new(1);
        let amount = vault.shares_to_amount(shares).unwrap();
        assert_eq!(amount, Uint128::new(100));

        // With 10,000 shares, they get 1,000,000
        // Amount: (10,000) * 101,000 / 1001 = 1,008,991.00
        let shares = Uint128::new(10_000);
        let amount = vault.shares_to_amount(shares).unwrap();
        assert_eq!(amount, Uint128::new(1_008_991));
    }

    /// 1e3 = 1,000
    /// This is 100x over the offset amount
    #[test]
    fn amount_imbalance_100_000_to_1() {
        let balance = Uint128::new(100_000);
        let total_shares = Uint128::new(1);

        // Virtual balance: (100000) + 1000 = 101000
        // Virtual shares: (1) + 1000 = 1001
        let vault = VirtualVault::new(total_shares, balance).unwrap();

        // With 1 amount, they get 0 share
        // (1) * 1001 / 101,000 = 0.0099
        let amount = Uint128::new(1);
        let shares = vault.amount_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // (100) * 1001 / 101,000 = 0.9910
        let amount = Uint128::new(100);
        let shares = vault.amount_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // (200) * 1001 / 101,000 = 1.98
        let amount = Uint128::new(200);
        let shares = vault.amount_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(1));

        // With 1000 amount (will at least get 1 no matter what)
        // (1000) * 1001 / 101,000 = 9.9009
        let amount = Uint128::new(1000);
        let shares = vault.amount_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(9));
    }

    #[test]
    fn extreme_inflation_1e20_to_1() {
        let balance = Uint128::new(1e20 as u128);
        let total_shares = Uint128::new(1);

        // Virtual balance: (1e20) + 1e3 = 1e20
        // Virtual shares: (1) + 1e3 = 1e3
        let vault = VirtualVault::new(total_shares, balance).unwrap();

        // With 999, they get 0 shares
        // Amount: (999) * (1 + 1e3)/ (1e20 + 1e3) = 9.99999E-15
        let amount = Uint128::new(999);
        let shares = vault.amount_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // Same for 1,000,000
        let amount = Uint128::new(1_000_000);
        let shares = vault.amount_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // You will need at least 1e20 / 1e3 = 1e17 amount to get 1 share
        let amount = Uint128::new(1e17 as u128);
        let shares = vault.amount_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(1));

        // But the cost of attack is crazy.
        // Using 1e17, you get 1 share
        {
            // New vault with +1 share and +1e17 balance
            let new_share = Uint128::new(1) + Uint128::new(1);
            let new_balance = Uint128::new(1e20 as u128) + Uint128::new(1e17 as u128);
            let vault = VirtualVault::new(new_share, new_balance).unwrap();

            // That one share is only worth less than 1e17
            let shares = Uint128::new(1);
            let amount = vault.shares_to_amount(shares).unwrap();
            assert!(amount < Uint128::new(1e17 as u128));
        }
    }
}
