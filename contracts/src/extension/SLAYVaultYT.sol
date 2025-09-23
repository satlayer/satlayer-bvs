// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {ISLAYVaultYT} from "./interface/ISLAYVaultYT.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ERC4626Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/extensions/ERC4626Upgradeable.sol";
import {IERC4626} from "@openzeppelin/contracts/interfaces/IERC4626.sol";

import {Initializable} from "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";
import {SLAYVaultV2} from "../SLAYVaultV2.sol";
import {SLAYRegistryV2} from "../SLAYRegistryV2.sol";
import {SLAYRouterV2} from "../SLAYRouterV2.sol";

contract SLAYVaultYT is ISLAYVaultYT, SLAYVaultV2 {
    SLAYInverseYieldToken public inverseYieldToken;
    SLAYPerpetualYieldToken public perpetualYieldToken;

    /// @notice The decimals of precision used by _lastUserExchangeRate and _lastGlobalExchangeRate
    uint256 internal constant PRECISION_DECIMALS = 18;
    /// @notice The precision used by _lastUserExchangeRate and _lastGlobalExchangeRate
    uint256 internal constant PRECISION = 10 ** PRECISION_DECIMALS;

    uint256 internal _lastGlobalExchangeRate;

    /// @dev user address => last user exchange rate
    mapping(address => uint256) internal _lastUserExchangeRate;

    /// @dev user address => accrued interest
    mapping(address => uint256) internal _userAccruedInterest;

    event MintSPTSYT(address indexed to, uint256 amount);
    event InterestClaimed(address indexed user, address indexed recipient, uint256 amount);

    modifier onlyPYT() {
        require(_msgSender() == address(perpetualYieldToken), "Only PYT can call");
        _;
    }

    constructor(SLAYRouterV2 router_, SLAYRegistryV2 registry_) SLAYVaultV2(router_, registry_) {}

    function initialize2(IERC20 asset_, address delegated_, string memory name_, string memory symbol_)
        public
        initializer
    {
        super.initialize(asset_, delegated_, name_, symbol_);

        // init IPT and PYT
        inverseYieldToken = new SLAYInverseYieldToken(
            string(abi.encodePacked("SLAY Inverse Yield Token ", name_)), string(abi.encodePacked("SIYT.", symbol_))
        );
        perpetualYieldToken = new SLAYPerpetualYieldToken(
            string(abi.encodePacked("SLAY Perpetual Yield Token ", name_)), string(abi.encodePacked("SPYT.", symbol_))
        );
        inverseYieldToken.initialize(address(perpetualYieldToken));
        perpetualYieldToken.initialize(address(inverseYieldToken));

        _lastGlobalExchangeRate = getCurrentExchangeRate();
    }

    /*//////////////////////////////////////////////////////////////
                          External Functions
    //////////////////////////////////////////////////////////////*/

    /// @inheritdoc ISLAYVaultYT
    function depositAndMintPYT(uint256 amount, address to) external override returns (uint256) {
        uint256 shares = super.deposit(amount, address(this));
        // mint SIPT and SPYT to receiver
        return _mintPYT(shares, to);
    }

    /// @inheritdoc ISLAYVaultYT
    function mintPYT(uint256 shares, address to) external override returns (uint256) {
        require(shares > 0, "Shares must be greater than 0");

        address sender = _msgSender();
        require(balanceOf(sender) >= shares, "Sender does not have enough token");

        // transfer shares to vault
        _transfer(sender, address(this), shares);
        // mint SIPT and SPYT to receiver
        return _mintPYT(shares, to);
    }

    /// @inheritdoc ISLAYVaultYT
    function redeemPYT(uint256 amount, address to) external override returns (uint256) {
        require(amount > 0, "Amount must be greater than 0");

        address sender = _msgSender();
        require(inverseYieldToken.balanceOf(sender) >= amount, "Sender does not have enough IPT");
        require(perpetualYieldToken.balanceOf(sender) >= amount, "Sender does not have enough PYT");

        // burn IPT and PYT from sender
        inverseYieldToken.burn(sender, amount);
        perpetualYieldToken.burn(sender, amount);

        uint256 SYToRedeem = _getSYToRedeem(amount);
        require(SYToRedeem > 0, "SY to redeem must be greater than 0");
        require(balanceOf(address(this)) >= SYToRedeem, "Vault does not have enough shares");

        // withdraw receipt token to receiver
        _transfer((address(this)), to, SYToRedeem);

        return SYToRedeem;
    }

    /// @inheritdoc ISLAYVaultYT
    function claimInterest(address recipient) external override returns (uint256) {
        address sender = _msgSender();
        uint256 pytBalance = perpetualYieldToken.balanceOf(sender);
        require(pytBalance > 0, "No PYT balance");

        _accrueInterest(getCurrentExchangeRate(), sender);

        uint256 interestToClaim = _userAccruedInterest[sender];
        _userAccruedInterest[sender] = 0;

        // transfer interest to recipient
        _transfer(address(this), recipient, interestToClaim);

        emit InterestClaimed(sender, recipient, interestToClaim);

        return interestToClaim;
    }

    /*//////////////////////////////////////////////////////////////
                          Getter Functions
    //////////////////////////////////////////////////////////////*/

    /// @inheritdoc ISLAYVaultYT
    function getSYFromPYT(uint256 pytAmount) external view returns (uint256) {
        return _getSYToRedeem(pytAmount);
    }

    /// @inheritdoc ISLAYVaultYT
    function getCurrentExchangeRate() public view returns (uint256) {
        return convertToAssets(PRECISION);
    }

    /// @inheritdoc ISLAYVaultYT
    function getAccruedInterest(address user) external view returns (uint256) {
        return _userAccruedInterest[user] + _calculateInterestToClaim(user, getCurrentExchangeRate());
    }

    /*//////////////////////////////////////////////////////////////
                          Internal Functions
    //////////////////////////////////////////////////////////////*/

    /// @dev internal fn to mint SIPT and SPYT after deposit or mintPYT.
    /// @dev amountToMint should equal to underlying asset added.
    function _mintPYT(uint256 amount, address to) internal returns (uint256 amountToMint) {
        // convert amount to pyt and ipt amount
        uint256 currentExchangeRate = getCurrentExchangeRate();
        amountToMint = Math.mulDiv(amount, currentExchangeRate, PRECISION, Math.Rounding.Floor);

        inverseYieldToken.mint(to, amountToMint);
        perpetualYieldToken.mint(to, amountToMint);

        // initialize last user yield
        _accrueInterest(currentExchangeRate, to);

        emit MintSPTSYT(to, amountToMint);
    }

    /// @dev internal fn to get SY amount redeemable from PYT amount with s(t) = pyt(t) / E(t)
    function _getSYToRedeem(uint256 ipytAmount) internal view returns (uint256) {
        uint256 currentExchangeRate = getCurrentExchangeRate();
        return Math.mulDiv(ipytAmount, PRECISION, currentExchangeRate, Math.Rounding.Floor);
    }

    /// @dev calculate user interest to claim from prev claim.
    function _calculateInterestToClaim(address user, uint256 currentExchangeRate)
        internal
        view
        returns (uint256 interestAmount)
    {
        uint256 pytBalance = perpetualYieldToken.balanceOf(user);
        if (pytBalance == 0) {
            return 0;
        }

        uint256 lastUserYield = _lastUserExchangeRate[user];
        // for the case of user first deposit, exchange rate is 0
        if (lastUserYield == 0) {
            return 0;
        }

        uint256 userPYTShare = perpetualYieldToken.balanceOf(user);

        // calculate interest amount for the sender
        interestAmount =
            (userPYTShare * (currentExchangeRate - lastUserYield) * PRECISION) / (currentExchangeRate * lastUserYield);
    }

    /// @dev update last exchange rate claimed for user + accrue interest and global exchange rate
    function _accrueInterest(uint256 currentExchangeRate, address user) internal {
        _lastGlobalExchangeRate = currentExchangeRate;
        if (user != address(0)) {
            _userAccruedInterest[user] += _calculateInterestToClaim(user, currentExchangeRate);
            _lastUserExchangeRate[user] = currentExchangeRate;
        }
    }

    /// @dev to be called before any PYT transfer and only called by the token contract.
    /// @dev this will accrue interest for both from and to address to ensure no interest is lost during transfer.
    function beforeTokenTransfer(address from, address to, uint256) external onlyPYT {
        uint256 currentExchangeRate = getCurrentExchangeRate();
        // accrue interest for both from and to
        if (from != address(0) && from != address(this)) _accrueInterest(currentExchangeRate, from);
        if (to != address(0) && to != address(this)) _accrueInterest(currentExchangeRate, to);
    }
}

contract SLAYERC20 is ERC20 {
    constructor(string memory name, string memory symbol) ERC20(name, symbol) {}

    function _update(address from, address to, uint256 amount) internal virtual override {
        require(from != to, "SLAYERC20: transfer to self is not allowed");

        _beforeTokenTransfer(from, to, amount);
        super._update(from, to, amount);
        _afterTokenTransfer(from, to, amount);
    }

    /**
     * @dev Hook that is called before any transfer of tokens.
     */
    function _beforeTokenTransfer(address from, address to, uint256 amount) internal virtual {}

    /**
     * @dev Hook that is called before any transfer of tokens.
     */
    function _afterTokenTransfer(address from, address to, uint256 amount) internal virtual {}
}

contract SLAYInverseYieldToken is SLAYERC20, Initializable {
    SLAYVaultYT public immutable vault;
    SLAYPerpetualYieldToken public pyt;

    modifier onlyVault() {
        require(msg.sender == address(vault), "Only vault can call");
        _;
    }

    constructor(string memory name, string memory symbol) SLAYERC20(name, symbol) {
        vault = SLAYVaultYT(msg.sender);
    }

    function initialize(address _pyt) external initializer onlyVault {
        pyt = SLAYPerpetualYieldToken(_pyt);
    }

    function mint(address to, uint256 amount) external onlyVault {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external onlyVault {
        _burn(from, amount);
    }
}

contract SLAYPerpetualYieldToken is SLAYERC20, Initializable {
    SLAYVaultYT public immutable vault;
    SLAYInverseYieldToken public iyt;

    modifier onlyVault() {
        require(msg.sender == address(vault), "Only vault can call");
        _;
    }

    constructor(string memory name, string memory symbol) SLAYERC20(name, symbol) {
        vault = SLAYVaultYT(msg.sender);
    }

    function initialize(address _iyt) external initializer onlyVault {
        iyt = SLAYInverseYieldToken(_iyt);
    }

    function mint(address to, uint256 amount) external onlyVault {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external onlyVault {
        _burn(from, amount);
    }

    function _beforeTokenTransfer(address from, address to, uint256 amount) internal override {
        vault.beforeTokenTransfer(from, to, amount);
    }
}
