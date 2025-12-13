// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {SLAYERC20Derivative} from "./SLAYERC20Derivative.sol";
import {ISLAYVaultV2} from "../interface/ISLAYVaultV2.sol";

contract SLAYSlashToken is SLAYERC20Derivative {
    // TODO: update to ISLAYVaultTS
    ISLAYVaultV2 public immutable vault;

    uint256 public rewardsTotal;

    event RewardsDistributed(uint256 amount);
    event SlashTokenClaimed(address indexed claimer, address indexed recipient, uint256 amount);

    modifier onlyVault() {
        if (msg.sender != address(vault)) {
            revert ERC20InvalidSender(msg.sender);
        }
        _;
    }

    constructor(string memory name_, string memory symbol_, uint32 checkpointBlock_, address checkpointedToken_)
        SLAYERC20Derivative(name_, symbol_, checkpointBlock_, checkpointedToken_)
    {
        vault = ISLAYVaultV2(checkpointedToken_);
    }

    /**
     * @dev for accounting of rewards received in vault's receipt token.
     * @dev vault receipt token must be transferred to this contract before calling this function
     */
    function distributeRewards(uint256 amount) external onlyVault {
        rewardsTotal += amount;

        emit RewardsDistributed(amount);
    }

    /**
     * @dev claim vault receipt tokens by burning slash tokens. Amount will be calculated by checkpointed balance
     */
    function claim(address recipient) external returns (uint256) {
        address claimer = _msgSender();
        uint256 claimerBalance = balanceOf(claimer);
        require(claimerBalance > 0, "SlashToken: no slash token balance");

        uint256 claimAmount = _calculateClaimAmount(claimerBalance);

        require(claimAmount > 0, "SlashToken: nothing to claim");
        require(vault.balanceOf(address(this)) >= claimAmount, "SlashToken: insufficient receipt token balance");

        // move claimer's slash token to this contract
        _transfer(claimer, address(this), claimerBalance);

        // transfer receipt token to claimer
        vault.transfer(recipient, claimAmount);

        emit SlashTokenClaimed(claimer, recipient, claimAmount);
        return claimAmount;
    }

    function _calculateClaimAmount(uint256 slashTokenBalance) internal view returns (uint256) {
        return slashTokenBalance * rewardsTotal / totalSupply();
    }
}
