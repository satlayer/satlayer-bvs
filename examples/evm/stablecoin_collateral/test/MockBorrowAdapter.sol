import "../src/BorrowAdapterBase.sol";

interface IMockMintable is IERC20 { 
    function mint(address to, uint256 amt) external;
    function burn(address to, uint256 amt) external; 
    
    }

/** @notice Minimal mock adapter with an in-contract “venue ledger”. */
contract MockBorrowAdapter is BorrowAdapterBase {
    // Simple ledgers per-token
    mapping(address => uint256) public collateralOf; // collateral token => amount credited to the adapter
    mapping(address => uint256) public debtOf;       // debt token => amount owed by the adapter

    constructor(address governance, address caller) BorrowAdapterBase(governance, caller) {}

    /* ---------- IBorrowVenueAdapter views ---------- */
    function collateralBalance(address collateral) external view override returns (uint256) {
        return collateralOf[collateral];
    }
    function debtBalance(address debtAsset) external view override returns (uint256) {
        return debtOf[debtAsset];
    }

    /* ---------- venue hooks ---------- */
    function _supply(address collateral, uint256 amount, bytes calldata) internal override {
        require(amount > 0, "ZERO_AMOUNT");
        // pull from caller (CG)
        IERC20(collateral).transferFrom(msg.sender, address(this), amount);
        collateralOf[collateral] += amount;
    }

    function _withdraw(address collateral, uint256 amount, bytes calldata) internal override returns (uint256 w) {
        require(amount > 0, "ZERO_AMOUNT");
        uint256 bal = collateralOf[collateral];
        require(bal >= amount, "INSUFFICIENT_COLLAT");
        collateralOf[collateral] = bal - amount;
        IERC20(collateral).transfer(msg.sender, amount); // send back to caller (CG)
        return amount;
    }

    function _borrow(address debtAsset, uint256 amount, bytes calldata) internal override {
        require(amount > 0, "ZERO_AMOUNT");
        debtOf[debtAsset] += amount;
        // mint simulated debt token to the caller (CG)
        IMockMintable(debtAsset).mint(msg.sender, amount);
    }

    function _repay(address debtAsset, uint256 amount, bytes calldata) internal override returns (uint256 r) {
        require(amount > 0, "ZERO_AMOUNT");
        uint256 owed = debtOf[debtAsset];
        uint256 pay = amount > owed ? owed : amount;
        // pull from caller (CG)
        IERC20(debtAsset).transferFrom(msg.sender, address(this), pay);
        //IERC20(debtAsset).burn(address(this), pay);
        debtOf[debtAsset] = owed - pay;
        return pay;
    }

    function _getRiskSignals(address debtAsset)
        internal
        view
        override
        returns (bool hasApr, uint aprBps, bool haveHf, uint hfBps)
    {
        return (false, 0, false, 0); // default: no signals
    }
}