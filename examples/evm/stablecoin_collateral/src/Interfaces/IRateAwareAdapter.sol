// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./IBorrowVenueAdapter.sol";

interface IRateAwareAdapter is IBorrowVenueAdapter {
    function borrowAprBps(address debtAsset) external view returns (uint16);
    function healthFactorBps() external view returns (uint16);
}
