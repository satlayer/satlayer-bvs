pragma solidity ^0.8.24;

interface IBorrowVenueAdapter {
    // Collateral
    function supplyCollateral(address collateral, uint256 amount, bytes calldata data) external;
    function withdrawCollateral(address collateral, uint256 amount, bytes calldata data) external returns (uint256 withdrawn);
    function collateralBalance(address collateral) external view returns (uint256);

    // Debt
    function borrow(address debtAsset, uint256 amount, bytes calldata data) external;
    function repay(address debtAsset, uint256 amount, bytes calldata data) external returns (uint256 repaid);
    function debtBalance(address debtAsset) external view returns (uint256);

    function getRiskSignals(address debtAsset) external view  returns (bool hasApr, uint aprBps, bool haveHf, uint hfBps);
}