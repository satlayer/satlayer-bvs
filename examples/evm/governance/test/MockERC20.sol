// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract MockERC20 is ERC20 {
uint8 private _decimals;

constructor(string memory _name, string memory _symbol, uint8 __decimals) ERC20(_name, _symbol) {
_decimals = __decimals;
}

function decimals() public view virtual override returns (uint8) {
return _decimals;
}

function mint(address account, uint256 value) public {
_mint(account, value);
}
}
