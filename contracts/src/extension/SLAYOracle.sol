// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {IPyth} from "@pythnetwork/pyth-sdk-solidity/IPyth.sol";
import {PythStructs} from "@pythnetwork/pyth-sdk-solidity/PythStructs.sol";
import {PythErrors} from "@pythnetwork/pyth-sdk-solidity/PythErrors.sol";
import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";

import {ISLAYOracle} from "./interface/ISLAYOracle.sol";
import {SLAYBase} from "../SLAYBase.sol";
import {ISLAYVaultV2} from "../interface/ISLAYVaultV2.sol";
import {ISLAYRouterV2} from "../interface/ISLAYRouterV2.sol";

/**
 * @title SLAYOracle
 * @notice SLAYOracle is an upgradeable contract that provides price feeds and AUM calculations for SLAYVaults using Pyth Network.
 * It allows setting and retrieving Pyth price IDs for an asset, fetching current prices, and calculating the total assets under management (AUM)
 * for operators based on their associated vaults.
 */
contract SLAYOracle is SLAYBase, ISLAYOracle {
    ISLAYRouterV2 internal immutable _slayRouter;

    IPyth internal immutable _pyth;

    uint256 public constant MAX_PRICE_AGE = 15 minutes;

    /// @dev stores the mapping of assets to their corresponding Pyth price IDs
    mapping(address asset => bytes32 priceId) internal _assetToPriceId;

    constructor(address pyth_, address slayRouter_) {
        _pyth = IPyth(pyth_);
        _slayRouter = ISLAYRouterV2(slayRouter_);

        _disableInitializers();
    }

    /// @inheritdoc ISLAYOracle
    function getPriceId(address asset) external view override returns (bytes32) {
        return _assetToPriceId[asset];
    }

    /// @inheritdoc ISLAYOracle
    function setPriceId(address asset, bytes32 priceId) external override onlyOwner {
        _assetToPriceId[asset] = priceId;
        emit PriceIdSet(asset, priceId);
    }

    /// @inheritdoc ISLAYOracle
    function getPrice(bytes32 priceId) public view override returns (uint256) {
        PythStructs.Price memory price = _pyth.getPriceNoOlderThan(priceId, MAX_PRICE_AGE);

        // convert price to uint256 and minor units (18 decimals)
        uint256 basePrice = _convertToUint(price.price, price.expo, 18);

        return basePrice;
    }

    /// @inheritdoc ISLAYOracle
    function getPrice(address asset) public view override returns (uint256) {
        bytes32 priceId = _assetToPriceId[asset];
        if (priceId == bytes32(0)) {
            revert PriceIdNotSet(asset);
        }
        return getPrice(priceId);
    }

    /// @inheritdoc ISLAYOracle
    function getOperatorAUM(address operator) external view virtual override returns (uint256 aum) {
        address[] memory vaults = _slayRouter.getOperatorVaults(operator);
        // get each vault's total active stake
        for (uint256 i = 0; i < vaults.length; i++) {
            aum += getVaultAUM(vaults[i]);
        }
        return aum;
    }

    /// @inheritdoc ISLAYOracle
    function getVaultAUM(address vault) public view virtual returns (uint256) {
        require(vault != address(0), "Invalid vault address");

        ISLAYVaultV2 vaultI = ISLAYVaultV2(vault);

        // get the vault's total assets
        uint256 vaultTotalAssets = vaultI.totalAssets();
        if (vaultTotalAssets == 0) {
            // return early if no assets
            return 0;
        }

        // get conversion rate
        uint256 USDPricePerAsset = getPrice(vaultI.asset());

        // convert asset to USD in 18 decimals
        return Math.mulDiv(vaultTotalAssets, USDPricePerAsset, 10 ** vaultI.decimals());
    }

    /**
     * Adapted over from https://github.com/pyth-network/pyth-crosschain/blob/81362dbc197aa35ce27caae2531b8655c87ade08/target_chains/ethereum/sdk/solidity/PythUtils.sol#L21
     */
    function _convertToUint(int64 price, int32 expo, uint8 targetDecimals) internal pure returns (uint256) {
        if (price < 0) {
            revert PythErrors.NegativeInputPrice();
        }
        if (expo < -255) {
            revert PythErrors.InvalidInputExpo();
        }

        // If targetDecimals is 6, we want to multiply the final price by 10 ** -6
        // So the delta exponent is targetDecimals + currentExpo
        int32 deltaExponent = int32(uint32(targetDecimals)) + expo;

        // Bounds check: prevent overflow/underflow with base 10 exponentiation
        // Calculation: 10 ** n <= (2 ** 256 - 63) - 1
        //              n <= log10((2 ** 193) - 1)
        //              n <= 58.2
        if (deltaExponent > 58 || deltaExponent < -58) {
            revert PythErrors.ExponentOverflow();
        }

        // We can safely cast the price to uint256 because the above condition will revert if the price is negative
        uint256 unsignedPrice = uint256(uint64(price));

        if (deltaExponent > 0) {
            (bool success, uint256 result) = Math.tryMul(unsignedPrice, 10 ** uint32(deltaExponent));
            // This condition is unreachable since we validated deltaExponent bounds above.
            // But keeping it here for safety.
            if (!success) {
                revert PythErrors.CombinedPriceOverflow();
            }
            return result;
        } else {
            (bool success, uint256 result) = Math.tryDiv(unsignedPrice, 10 ** uint256(_abs(deltaExponent)));
            // This condition is unreachable since we validated deltaExponent bounds above.
            // But keeping it here for safety.
            if (!success) {
                revert PythErrors.CombinedPriceOverflow();
            }
            return result;
        }
    }

    /**
     * Adapted from https://github.com/pyth-network/pyth-crosschain/blob/81362dbc197aa35ce27caae2531b8655c87ade08/target_chains/ethereum/sdk/solidity/Math.sol#L174
     * @dev Returns the absolute unsigned value of a signed value.
     */
    function _abs(int256 n) internal pure returns (uint256) {
        unchecked {
            // Formula from the "Bit Twiddling Hacks" by Sean Eron Anderson.
            // Since `n` is a signed integer, the generated bytecode will use the SAR opcode to perform the right shift,
            // taking advantage of the most significant (or "sign" bit) in two's complement representation.
            // This opcode adds new most significant bits set to the value of the previous most significant bit. As a result,
            // the mask will either be `bytes32(0)` (if n is positive) or `~bytes32(0)` (if n is negative).
            int256 mask = n >> 255;

            // A `bytes32(0)` mask leaves the input unchanged, while a `~bytes32(0)` mask complements it.
            return uint256((n + mask) ^ mask);
        }
    }
}
