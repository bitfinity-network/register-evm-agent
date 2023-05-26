// SPDX-License-Identifier: MIT
pragma solidity ^0.7.0;

import "forge-std/Test.sol";
import "../src/AggregatorProxy.sol";

contract AggregatorProxyTest is Test {
    AggregatorProxy public aggregator_proxy;

    function setUp() public {
        aggregator_proxy = new AggregatorProxy();
    }
}