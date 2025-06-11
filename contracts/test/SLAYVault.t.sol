import {Test, console} from "forge-std/Test.sol";
import {SLAYVault} from "../src/SLAYVault.sol";

contract SLAYVaultTest is Test {
    SLAYVault public vault;

    function setUp() public {
        vault = new SLAYVault();
    }

    function testDecimals() public {
        uint8 expectedDecimals = 99;
        uint8 actualDecimals = vault.decimals();
        assertEq(actualDecimals, expectedDecimals, "Decimals should be 99");
    }
}
