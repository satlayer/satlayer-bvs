// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IConversionGatewayMulti {
    function onClaimWithStrategy(address user, uint256 baseAssets, bytes32 strategy, bytes calldata params) external;

    function kindOf(bytes32 strategy) external view returns (uint8); // returns RouteKind enum value

    // existing
    function unwindDepositAny(address user, bytes32 strategy, uint256 requestedBaseOrWrapped, uint256 minOutAfterUnwrap)
        external;
    function unwindBorrow(
        address user,
        bytes32 strategy,
        uint256 requestedDebtIn,
        uint256 minCollateralOut,
        bytes calldata adapterData,
        uint256 connectorMinOut
    ) external;
}
