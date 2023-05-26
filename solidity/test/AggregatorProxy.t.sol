// SPDX-License-Identifier: MIT
pragma solidity ^0.7.0;
pragma experimental ABIEncoderV2;

import "forge-std/Test.sol";
import "../src/AggregatorSingle.sol";

contract AggregatorSingleTest is Test {
    AggregatorSingle public aggregator;

    function setUp() public {
        aggregator = new AggregatorSingle();
    }

    function testAggregator() public {
        emit log_string("add Pair");

        aggregator.addPair("bitcoin", 8, "btc-usd price pair", 1);

        aggregator.updateAnswer("bitcoin", 1685082769486341127, 2640100000000);
        aggregator.updateAnswer("bitcoin", 1685083669486865362, 2639200000000);
        aggregator.updateAnswer("bitcoin", 1685084271838933463, 2639200000000);
        aggregator.updateAnswer("bitcoin", 1685084571694748370, 2641000000000);

        assertEq(aggregator.latestAnswer("bitcoin"), 2641000000000);
        emit log_named_uint("latestAnswer", aggregator.latestAnswer("bitcoin"));

        assertEq(aggregator.latestTimestamp("bitcoin"), 1685084571694748370);
        emit log_named_uint("latestTimestamp", aggregator.latestTimestamp("bitcoin"));

        assertEq(aggregator.latestRound("bitcoin"), 4);
        emit log_named_uint("latestRound", aggregator.latestRound("bitcoin"));

        assertEq(aggregator.getAnswer("bitcoin", 1), 2640100000000);
        emit log_named_uint("getAnswer of 1", aggregator.getAnswer("bitcoin", 1));

        assertEq(aggregator.getTimestamp("bitcoin", 1), 1685082769486341127);
        emit log_named_uint("getTimestamp of 1", aggregator.getTimestamp("bitcoin", 1));

        assert(aggregator.decimals("bitcoin") == 8);
        emit log_named_uint("decimals", aggregator.decimals("bitcoin"));

        assertEq(aggregator.description("bitcoin"), "btc-usd price pair");
        emit log_named_string("description", aggregator.description("bitcoin"));

        (uint256 roundId, uint256 answer, uint256 timestamp) = aggregator.getRoundData(2, "bitcoin");
        assertEq(roundId, 2);
        assertEq(answer, 2639200000000);
        assertEq(timestamp, 1685083669486865362);

        (uint256 _roundId, uint256 _answer, uint256 _timestamp) = aggregator.latestRoundData("bitcoin");

        assertEq(_roundId, 4);
        assertEq(_answer, 2641000000000);
        assertEq(_timestamp, 1685084571694748370);
    }
}
