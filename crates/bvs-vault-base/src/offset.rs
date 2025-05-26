use crate::VaultError;
use cosmwasm_std::{Deps, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::Item;

/// The offset is used to mitigate the common 'share inflation' attack vector.
///
/// See [https://docs.openzeppelin.com/contracts/5.x/erc4626#inflation-attack]
///
/// This 1 offset will be used in exchange rate computation to reduce the impact of the attack.
/// When the vault is empty, the virtual shares and virtual assets enforce the conversion rate 1/1.
///
/// Share inflation attack will not be as prevalent because of async withdrawal,
/// hence offset of 1 will be enough to mitigate the attack.
const OFFSET: Uint128 = Uint128::new(1);

/// The total shares of the contract held by all stakers.
/// [`OFFSET`] value is not included in the total shares, only the real shares are counted.
const TOTAL_SHARES: Item<Uint128> = Item::new("total_shares");

/// Get the total shares in circulation
pub fn get_total_shares(storage: &dyn Storage) -> StdResult<Uint128> {
    TOTAL_SHARES
        .may_load(storage)
        .map(|shares| shares.unwrap_or(Uint128::zero()))
}

/// Follows the OpenZeppelin's ERC4626 mitigation strategy for inflation attack.
/// Using a "virtual" offset to +1 to both total shares and assets representing the virtual total shares and virtual total assets.
/// A donation of 1 and under will be completely captured by the vault—without affecting the user.
/// A donation greater than 1, the attacker will suffer loss greater than the user.
/// [https://github.com/OpenZeppelin/openzeppelin-contracts/blob/fa995ef1fe66e1447783cb6038470aba23a6343f/contracts/token/ERC20/extensions/ERC4626.sol#L30-L37]
#[derive(Debug)]
pub struct VirtualOffset {
    total_shares: Uint128,
    total_assets: Uint128,
    virtual_total_shares: Uint128,
    virtual_total_assets: Uint128,
}

impl VirtualOffset {
    /// Create a new [VirtualOffset] with the given total shares and total assets.
    pub fn new(total_shares: Uint128, total_assets: Uint128) -> StdResult<Self> {
        let virtual_total_shares = total_shares.checked_add(OFFSET).map_err(StdError::from)?;
        let virtual_total_assets = total_assets.checked_add(OFFSET).map_err(StdError::from)?;

        Ok(Self {
            total_shares,
            total_assets,
            virtual_total_shares,
            virtual_total_assets,
        })
    }

    /// Shares to underlying assets
    pub fn shares_to_assets(&self, shares: Uint128) -> StdResult<Uint128> {
        // (shares * self.virtual_total_assets) / self.virtual_total_shares
        shares
            .checked_mul(self.virtual_total_assets)
            .map_err(StdError::from)?
            .checked_div(self.virtual_total_shares)
            .map_err(StdError::from)
    }

    /// Underlying assets to shares
    pub fn assets_to_shares(&self, assets: Uint128) -> StdResult<Uint128> {
        // (assets * self.virtual_total_shares) / self.virtual_total_assets
        assets
            .checked_mul(self.virtual_total_shares)
            .map_err(StdError::from)?
            .checked_div(self.virtual_total_assets)
            .map_err(StdError::from)
    }

    /// Get the total shares in circulation
    pub fn total_shares(&self) -> Uint128 {
        self.total_shares
    }

    /// Get the total assets under management
    pub fn total_assets(&self) -> Uint128 {
        self.total_assets
    }
}

/// This struct wraps the [VirtualOffset] struct with [TOTAL_SHARES] storage features
/// `checked_add_shares` and `checked_sub_shares` implemented.
/// Other methods are mapped to the underlying [VirtualOffset] instance.
///
/// [TotalShares] is only used to account for the total shares (and total assets).
/// Individual staker shares are stored here to allow for different staking strategies (e.g., Tokenized Vault).
#[derive(Debug)]
pub struct TotalShares(VirtualOffset);

impl TotalShares {
    /// Load the virtual total shares from storage (supports rebasing, by default).
    /// A fixed [`OFFSET`] of 1 will be added to both total shares and total assets
    /// to mitigate against inflation attack.
    /// Use [shares_to_assets] and [assets_to_shares] to convert between shares and assets.
    pub fn load(deps: &Deps, total_assets: Uint128) -> StdResult<Self> {
        let total_shares = get_total_shares(deps.storage)?;
        let offset = VirtualOffset::new(total_shares, total_assets)?;
        Ok(Self(offset))
    }

    /// Shares to underlying assets
    pub fn shares_to_assets(&self, shares: Uint128) -> StdResult<Uint128> {
        self.0.shares_to_assets(shares)
    }

    /// Underlying assets to shares
    pub fn assets_to_shares(&self, assets: Uint128) -> StdResult<Uint128> {
        self.0.assets_to_shares(assets)
    }

    /// Get the total shares in circulation
    pub fn total_shares(&self) -> Uint128 {
        self.0.total_shares
    }

    /// Get the total assets under management
    pub fn total_assets(&self) -> Uint128 {
        self.0.total_assets
    }

    /// Add the new shares to the total shares and refresh the virtual shares and virtual assets.
    /// This method is checked:
    ///  - New shares cannot be zero.
    ///  - Total shares cannot overflow.
    ///  - Virtual shares cannot overflow.
    pub fn checked_add_shares(
        &mut self,
        storage: &mut dyn Storage,
        shares: Uint128,
    ) -> Result<(), VaultError> {
        if shares.is_zero() {
            return Err(VaultError::zero("Add shares cannot be zero"));
        }

        self.0.total_shares = self
            .0
            .total_shares
            .checked_add(shares)
            .map_err(StdError::from)?;
        self.0.virtual_total_shares = self
            .0
            .total_shares
            .checked_add(OFFSET)
            .map_err(StdError::from)?;
        TOTAL_SHARES.save(storage, &self.0.total_shares)?;
        Ok(())
    }

    /// Subtract the shares from the total shares and refresh the virtual shares and virtual assets.
    pub fn checked_sub_shares(
        &mut self,
        storage: &mut dyn Storage,
        shares: Uint128,
    ) -> Result<(), VaultError> {
        if shares.is_zero() {
            return Err(VaultError::zero("Sub shares cannot be zero"));
        }

        self.0.total_shares = self
            .0
            .total_shares
            .checked_sub(shares)
            .map_err(StdError::from)?;
        self.0.virtual_total_shares = self
            .0
            .total_shares
            .checked_add(OFFSET)
            .map_err(StdError::from)?;
        TOTAL_SHARES.save(storage, &self.0.total_shares)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_to_one() {
        let total_assets = Uint128::new(1000);
        let total_shares = Uint128::new(1000);
        let vault = VirtualOffset::new(total_shares, total_assets).unwrap();

        {
            let assets = vault.shares_to_assets(Uint128::new(1000)).unwrap();
            assert_eq!(assets, Uint128::new(1000));

            let shares = vault.assets_to_shares(Uint128::new(1000)).unwrap();
            assert_eq!(shares, Uint128::new(1000));
        }

        {
            let assets = vault.shares_to_assets(Uint128::new(100)).unwrap();
            assert_eq!(assets, Uint128::new(100));

            let shares = vault.assets_to_shares(Uint128::new(100)).unwrap();
            assert_eq!(shares, Uint128::new(100));
        }

        {
            let assets = vault.shares_to_assets(Uint128::new(10000)).unwrap();
            assert_eq!(assets, Uint128::new(10000));

            let shares = vault.assets_to_shares(Uint128::new(10000)).unwrap();
            assert_eq!(shares, Uint128::new(10000));
        }
    }

    #[test]
    fn inflation_attack_over_1() {
        // Attacker deposits 1 to get 1 share
        // Attacker donates 99,999 moving the balance to 100,000
        let attacker_donation = Uint128::new(99_999);

        let balance = Uint128::new(1) + attacker_donation;
        let total_shares = Uint128::new(1);
        let vault = VirtualOffset::new(total_shares, balance).unwrap();

        // Attacker 1 share is worth amount 50_000 (captured by the vault)
        let amount = vault.shares_to_assets(Uint128::new(1)).unwrap();
        assert_eq!(amount, Uint128::new(50_000));

        // Normal user deposits 10,000 to get 0 shares (not executed)
        let amount = Uint128::new(10_000);
        let shares = vault.assets_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(0));

        {
            // Normal user deposits 50_001 to get 1 share
            // ( anything below 50_001 will receive 0 shares)
            let amount = Uint128::new(50_001);
            let shares = vault.assets_to_shares(amount).unwrap();
            assert_eq!(shares, Uint128::new(1));

            // Moves the vault.
            let balance = Uint128::new(150_001);
            let total_shares = Uint128::new(1 + 1);
            let vault = VirtualOffset::new(total_shares, balance).unwrap();

            // Attacker 1 share is worth 50,000 (captured by the vault)
            let amount = vault.shares_to_assets(Uint128::new(1)).unwrap();
            assert_eq!(amount, Uint128::new(50_000));

            // User 1 share is worth 50,000 (captured by the vault)
            let amount = vault.shares_to_assets(shares).unwrap();
            assert_eq!(amount, Uint128::new(50_000));
        }
        {
            // Normal user deposits 100,000 to get 1 share
            let amount = Uint128::new(100_000);
            let shares = vault.assets_to_shares(amount).unwrap();
            assert_eq!(shares, Uint128::new(1));

            // Moves the vault.
            let balance = Uint128::new(150_000);
            let total_shares = Uint128::new(1 + 1);
            let vault = VirtualOffset::new(total_shares, balance).unwrap();

            // Attacker 1 share is worth 50,000 (captured by the vault) - attacker lost 50%
            let amount = vault.shares_to_assets(Uint128::new(1)).unwrap();
            assert_eq!(amount, Uint128::new(50_000));

            // User 1 share is worth 50,000 (captured by the vault) - user lost 50%
            let amount = vault.shares_to_assets(shares).unwrap();
            assert_eq!(amount, Uint128::new(50_000));
        }
        {
            // Normal user deposits 100,001 to get 2 shares
            let amount = Uint128::new(100_001);
            let shares = vault.assets_to_shares(amount).unwrap();
            assert_eq!(shares, Uint128::new(2));

            // Moves the vault.
            let balance = Uint128::new(150_001);
            let total_shares = Uint128::new(1 + 2);
            let vault = VirtualOffset::new(total_shares, balance).unwrap();

            // Attacker 1 share is worth 37,500 (captured by the vault) - attacker lost 62.5%
            let amount = vault.shares_to_assets(Uint128::new(1)).unwrap();
            assert_eq!(amount, Uint128::new(37_500));

            // User 2 share is worth 75,001 (captured by the vault) - user lost 25%
            let amount = vault.shares_to_assets(shares).unwrap();
            assert_eq!(amount, Uint128::new(75_001));
        }
    }

    #[test]
    fn imbalance_1000_to_1() {
        let balance = Uint128::new(1000);
        let total_shares = Uint128::new(1);

        // Virtual balance: (1000) + 1 = 1001
        // Virtual shares: (1) + 1 = 2
        let vault = VirtualOffset::new(total_shares, balance).unwrap();

        // Low amounts
        {
            let shares = Uint128::new(500);
            let amount = vault.shares_to_assets(shares).unwrap();
            // Amount: (500) * 1001 / 2 = 250,250
            assert_eq!(amount, Uint128::new(250_250));

            let amount = Uint128::new(250);
            let shares = vault.assets_to_shares(amount).unwrap();
            // Shares: (250) * 2 / 1001 = 0.499
            assert_eq!(shares, Uint128::new(0));
        }

        // High amounts
        {
            let shares = Uint128::new(10_000);
            let amount = vault.shares_to_assets(shares).unwrap();
            // Amount: (10,000) * 1001 / 2 = 5,005,000
            assert_eq!(amount, Uint128::new(5_005_000));

            let amount = Uint128::new(10_000_000);
            let shares = vault.assets_to_shares(amount).unwrap();
            // Shares: (10,000,000) * 2 / 1001 = 19,980.01998002
            assert_eq!(shares, Uint128::new(19_980));
        }
    }

    #[test]
    fn imbalance_1000_to_2() {
        let balance = Uint128::new(1000);
        let total_shares = Uint128::new(2);

        // Virtual balance: (1000) + 1 = 1001
        // Virtual shares: (2) + 1 = 3
        let vault = VirtualOffset::new(total_shares, balance).unwrap();

        // Low amounts
        {
            let shares = Uint128::new(1000);
            let amount = vault.shares_to_assets(shares).unwrap();
            // Amount: (1000) * 1001 / 3 = 333,666.67
            assert_eq!(amount, Uint128::new(333_666));

            let amount = Uint128::new(1);
            let shares = vault.assets_to_shares(amount).unwrap();
            // Shares: (1) * 3 / 1001 = 0.003
            assert_eq!(shares, Uint128::new(0));

            let amount = Uint128::new(10);
            let shares = vault.assets_to_shares(amount).unwrap();
            // Shares: (10) * 3 / 1001 = 0.03
            assert_eq!(shares, Uint128::new(0));
        }

        // High amounts
        {
            let shares = Uint128::new(100_444);
            let amount = vault.shares_to_assets(shares).unwrap();
            // Amount: (100,444) * 1001 / 3 = 33,514,814.67
            assert_eq!(amount, Uint128::new(33_514_814));

            let amount = Uint128::new(10_000_000);
            let shares = vault.assets_to_shares(amount).unwrap();
            // Shares: (10,000,000) * 3 / 1001 = 29,970.03
            assert_eq!(shares, Uint128::new(29_970));
        }
    }

    /// This is 100_000x over the offset amount
    #[test]
    fn shares_imbalance_100_000_to_1() {
        let balance = Uint128::new(100_000);
        let total_shares = Uint128::new(1);

        // Virtual balance: (100,000) + 1 = 100,001
        // Virtual shares: (1) + 1 = 2
        let vault = VirtualOffset::new(total_shares, balance).unwrap();

        // With 500 shares, they get 25_000_250
        // Amount: (500) * 100,001 / 2 = 25_000_250
        let shares = Uint128::new(500);
        let amount = vault.shares_to_assets(shares).unwrap();
        assert_eq!(amount, Uint128::new(25_000_250));

        // With 1 share, they get 50,000
        // Amount: (1) * 100,001 / 2 = 50,000.5
        let shares = Uint128::new(1);
        let amount = vault.shares_to_assets(shares).unwrap();
        assert_eq!(amount, Uint128::new(50_000));

        // With 10,000 shares, they get 500,005,000
        // Amount: (10,000) * 100,001 / 2 = 500,005,000
        let shares = Uint128::new(10_000);
        let amount = vault.shares_to_assets(shares).unwrap();
        assert_eq!(amount, Uint128::new(500_005_000));
    }

    /// This is 100_000x over the offset amount
    #[test]
    fn amount_imbalance_100_000_to_1() {
        let balance = Uint128::new(100_000);
        let total_shares = Uint128::new(1);

        // Virtual balance: (100000) + 1 = 100001
        // Virtual shares: (1) + 1 = 2
        let vault = VirtualOffset::new(total_shares, balance).unwrap();

        // With 1 amount, they get 0 share
        // (1) * 2 / 100,001 = 0.0000199998
        let amount = Uint128::new(1);
        let shares = vault.assets_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // (100) * 2 / 100,001 = 0.00199998
        let amount = Uint128::new(100);
        let shares = vault.assets_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // With 50,001 amount (will at least get 1 no matter what)
        // (50,001) * 2 / 100,001 = 1.0000099999
        let amount = Uint128::new(50_001);
        let shares = vault.assets_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(1));
    }

    #[test]
    fn extreme_inflation_1e20_to_1() {
        let balance = Uint128::new(1e20 as u128);
        let total_shares = Uint128::new(1);

        // Virtual balance: (1e20) + 1 = 1e20
        // Virtual shares: (1) + 1 = 2
        let vault = VirtualOffset::new(total_shares, balance).unwrap();

        // With 999, they get 0 shares
        // Amount: (999) * (1 + 1)/ (1e20 + 1) = 1.998E-17
        let amount = Uint128::new(999);
        let shares = vault.assets_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // Same for 1,000,000
        let amount = Uint128::new(1_000_000);
        let shares = vault.assets_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(0));

        // You will need at least 1e20 / 1 = 1e20 amount to get 1 share
        let amount = Uint128::new(1e20 as u128);
        let shares = vault.assets_to_shares(amount).unwrap();
        assert_eq!(shares, Uint128::new(1));

        // But the cost of attack is crazy.
        // Using 1e20, you get 1 share
        {
            // New vault with +1 share and +1e20 balance
            let new_share = Uint128::new(1) + Uint128::new(1);
            let new_balance = Uint128::new(1e20 as u128) + Uint128::new(1e20 as u128);
            let vault = VirtualOffset::new(new_share, new_balance).unwrap();

            // That one share is only worth less than 1e20
            let shares = Uint128::new(1);
            let amount = vault.shares_to_assets(shares).unwrap();
            assert!(amount < Uint128::new(1e20 as u128));
        }
    }

    #[test]
    fn overflow() {
        let almost_max = Uint128::new(u128::MAX);

        {
            let error = VirtualOffset::new(almost_max, almost_max).unwrap_err();
            assert_eq!(
                error.to_string(),
                "Overflow: Cannot Add with given operands"
            )
        }

        {
            let max_div_1e10 = Uint128::new(u128::MAX / 1e10 as u128);
            let vault = VirtualOffset::new(max_div_1e10, max_div_1e10).unwrap();

            vault.shares_to_assets(Uint128::new(1)).unwrap();
            vault.assets_to_shares(Uint128::new(1)).unwrap();

            vault.shares_to_assets(Uint128::new(1e9 as u128)).unwrap();
            vault.assets_to_shares(Uint128::new(1e9 as u128)).unwrap();

            vault
                .shares_to_assets(Uint128::new((1e10 as u128) - 1))
                .unwrap();
            vault
                .assets_to_shares(Uint128::new((1e10 as u128) - 1))
                .unwrap();

            let error = vault
                .shares_to_assets(Uint128::new(1e10 as u128))
                .unwrap_err();
            assert_eq!(
                error.to_string(),
                "Overflow: Cannot Mul with given operands"
            );

            let error = vault
                .assets_to_shares(Uint128::new(1e10 as u128))
                .unwrap_err();
            assert_eq!(
                error.to_string(),
                "Overflow: Cannot Mul with given operands"
            );
        }
    }
}
