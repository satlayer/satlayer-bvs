// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {Pausable} from "@openzeppelin/contracts/utils/Pausable.sol";
import {IBorrowVenueAdapter} from "./Interfaces/IBorrowVenueAdapter.sol";

abstract contract BorrowAdapterBase is IBorrowVenueAdapter, AccessControl, Pausable {
    bytes32 public constant ROLE_GOV = keccak256("ROLE_GOV");
    bytes32 public constant ROLE_CG = keccak256("ROLE_CG"); // e.g., ConversionGateway

    event Supplied(address indexed collateral, uint256 amount);
    event Withdrawn(address indexed collateral, uint256 amount);
    event Borrowed(address indexed debtAsset, uint256 amount);
    event Repaid(address indexed debtAsset, uint256 amount);

    constructor(address governance, address caller) {
        _grantRole(ROLE_GOV, governance);
        _grantRole(ROLE_CG, caller);
    }

    function setPaused(bool on) external onlyRole(ROLE_GOV) {
        on ? _pause() : _unpause();
    }

    // --- IBorrowVenueAdapter entry points gated & paused ---

    function supplyCollateral(address collateral, uint256 amount, bytes calldata data)
        external
        override
        onlyRole(ROLE_CG)
        whenNotPaused
    {
        _supply(collateral, amount, data);
        emit Supplied(collateral, amount);
    }

    function withdrawCollateral(address collateral, uint256 amount, bytes calldata data)
        external
        override
        onlyRole(ROLE_CG)
        whenNotPaused
        returns (uint256 withdrawn)
    {
        withdrawn = _withdraw(collateral, amount, data);
        emit Withdrawn(collateral, withdrawn);
    }

    function borrow(address debtAsset, uint256 amount, bytes calldata data)
        external
        override
        onlyRole(ROLE_CG)
        whenNotPaused
    {
        _borrow(debtAsset, amount, data);
        emit Borrowed(debtAsset, amount);
    }

    function repay(address debtAsset, uint256 amount, bytes calldata data)
        external
        override
        onlyRole(ROLE_CG)
        whenNotPaused
        returns (uint256 repaid)
    {
        repaid = _repay(debtAsset, amount, data);
        emit Repaid(debtAsset, repaid);
    }

    function getRiskSignals(address debtAsset)
        external
        view
        override
        returns (bool hasApr, uint256 aprBps, bool haveHf, uint256 hfBps)
    {
        _getRiskSignals(debtAsset);
    }

    // --- venue-specific hooks to implement in child ---
    function _getRiskSignals(address debtAsset)
        internal
        view
        virtual
        returns (bool hasApr, uint256 aprBps, bool haveHf, uint256 hfBps);

    function _supply(address collateral, uint256 amount, bytes calldata data) internal virtual;
    function _withdraw(address collateral, uint256 amount, bytes calldata data) internal virtual returns (uint256);
    function _borrow(address debtAsset, uint256 amount, bytes calldata data) internal virtual;
    function _repay(address debtAsset, uint256 amount, bytes calldata data) internal virtual returns (uint256);

    // Read-only IBorrowVenueAdapter funcs to implement in child:
    function collateralBalance(address collateral) external view virtual override returns (uint256);
    function debtBalance(address debtAsset) external view virtual override returns (uint256);
}
