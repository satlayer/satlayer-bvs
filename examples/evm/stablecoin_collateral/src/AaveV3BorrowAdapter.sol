// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IPool} from "@aave/core-v3/contracts/interfaces/IPool.sol";
import {DataTypes} from "@aave/core-v3/contracts/protocol/libraries/types/DataTypes.sol";
import {BorrowAdapterBase} from "./BorrowAdapterBase.sol";


contract AaveV3BorrowAdapter is BorrowAdapterBase {
    using SafeERC20 for IERC20;

    IPool public immutable pool;
    uint256 public constant VARIABLE = 2;

    constructor(address governance, address caller, address pool_)
        BorrowAdapterBase(governance, caller)
    {
        require(pool_ != address(0), "POOL_ZERO");
        pool = IPool(pool_);
    }

    /* ---------------- venue-specific implementations ---------------- */

    function _supply(address collateral, uint256 amount, bytes calldata) internal override {
        require(amount > 0, "ZERO_AMOUNT");
        IERC20(collateral).safeIncreaseAllowance(address(pool), amount);
        pool.supply(collateral, amount, address(this), 0);
    }

    function _withdraw(address collateral, uint256 amount, bytes calldata) internal override returns (uint256 withdrawn) {
        require(amount > 0, "ZERO_AMOUNT");
        withdrawn = pool.withdraw(collateral, amount, msg.sender); // send back to caller (CG)
    }

    function _borrow(address debtAsset, uint256 amount, bytes calldata) internal override {
        require(amount > 0, "ZERO_AMOUNT");
        pool.borrow(debtAsset, amount, VARIABLE, 0, address(this));
    }

    function _repay(address debtAsset, uint256 amount, bytes calldata) internal override returns (uint256 repaid) {
        require(amount > 0, "ZERO_AMOUNT");
        IERC20(debtAsset).safeIncreaseAllowance(address(pool), amount);
        repaid = pool.repay(debtAsset, amount, VARIABLE, address(this));
    }

    function collateralBalance(address collateral) external view override returns (uint256) {
        DataTypes.ReserveData memory r = pool.getReserveData(collateral);
        return IERC20(r.aTokenAddress).balanceOf(address(this));
    }

    function debtBalance(address debtAsset) external view override returns (uint256) {
        DataTypes.ReserveData memory r = pool.getReserveData(debtAsset);
        // variable debt token balance of this adapter (the “account”)
        return IERC20(r.variableDebtTokenAddress).balanceOf(address(this));
    }

    function _getRiskSignals(address debtAsset)
        internal
        view
        override
        returns (bool, uint, bool, uint)
    {
        // Example: read from Aave’s data provider
        DataTypes.ReserveData memory r = pool.getReserveData(debtAsset);
        uint aprBps = uint(r.currentVariableBorrowRate / 1e23); // convert ray to bps (simplified)

        (
            uint256 totalCollateralBase,
            uint256 totalDebtBase,
            uint256 availableBorrowsBase,
            uint256 currentLiquidationThreshold,
            uint256 ltv,
            uint256 healthFactor
            ) = pool.getUserAccountData(address(this));

        //DataTypes.UserPositionFullInfo memory r2 = pool.getUserPositionFullInfo(address(this));
        uint hfBps = uint(healthFactor / 1e14); // convert 1e18 HF to bps (HF=1.05 → 10500)

        return (true, aprBps, true, hfBps);
    }

}
