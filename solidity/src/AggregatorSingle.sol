// SPDX-License-Identifier: MIT
pragma solidity ^0.7.0;
pragma experimental ABIEncoderV2;

import "./ConfirmedOwner.sol";

interface AggregatorInterface {
    function latestAnswer(string calldata pair) external view returns (uint256);

    function latestTimestamp(string calldata pair) external view returns (uint256);

    function latestRound(string calldata pair) external view returns (uint256);

    function getAnswer(string calldata pair, uint256 roundId) external view returns (uint256);

    function getTimestamp(string calldata pair, uint256 roundId) external view returns (uint256);

    event AnswerUpdated(uint256 indexed answer, uint256 indexed roundId, uint256 updatedAt, string pair);
}

interface AggregatorV3Interface {
    function decimals(string calldata pair) external view returns (uint8);

    function description(string calldata pair) external view returns (string memory);

    function version(string calldata pair) external view returns (uint256);

    // getRoundData and latestRoundData should both raise "No data present"
    // if they do not have data to report, instead of returning unset values
    // which could be misinterpreted as actual reported values.
    function getRoundData(uint80 _roundId, string calldata pair)
        external
        view
        returns (uint256 roundId, uint256 answer, uint256 timestamp);

    function latestRoundData(string calldata pair)
        external
        view
        returns (uint256 roundId, uint256 answer, uint256 timestamp);
}

contract AggregatorSingle is AggregatorInterface, AggregatorV3Interface, ConfirmedOwner {
    struct Answer {
        uint256 timestamp;
        uint256 value;
    }

    struct PairMetaData {
        string name;
        uint8 decimal;
        string description;
        uint256 version;
    }

    uint256 private unUsedPairId;
    // "price pair" => pair id
    mapping(string => uint256) private pairToId;
    // pair id => pair metadata
    mapping(uint256 => PairMetaData) private pairs;

    // pair id => round id
    mapping(uint256 => uint256) private unUsedRoundId;
    // round id => pair id => answer
    mapping(uint256 => mapping(uint256 => Answer)) private answers;

    constructor() ConfirmedOwner(msg.sender) {
        unUsedPairId = 1;
    }

    function addPair(string calldata pair, uint8 decimal, string calldata description, uint256 version)
        external
        onlyOwner
    {
        require(!exists(pair), "pair already exists");

        pairToId[pair] = unUsedPairId;
        PairMetaData storage p = pairs[unUsedPairId];
        p.name = pair;
        p.decimal = decimal;
        p.description = description;
        p.version = version;

        unUsedRoundId[unUsedPairId] = 1;

        unUsedPairId += 1;
    }

    function updateAnswer(string calldata pair, uint256 timestamp, uint256 answer) external onlyOwner {
        require(exists(pair), "pair don't exists");
        uint256 pairId = pairToId[pair];
        uint256 roundId = unUsedRoundId[pairId];

        Answer storage a = answers[roundId][pairId];
        a.timestamp = timestamp;
        a.value = answer;

        unUsedRoundId[pairId] += 1;

        emit AnswerUpdated(answer, roundId, timestamp, pair);
    }

    function updateAnswers(string[] calldata _pairs, uint256[] calldata _timestamps, uint256[] calldata _answers)
        external
        onlyOwner
    {
        require(_pairs.length == _timestamps.length, "pairs and timestamps length mismatch");
        require(_answers.length == _timestamps.length, "answers and timestamps length mismatch");

        for (uint256 i = 0; i < _pairs.length; ++i) {
            string calldata pair = _pairs[i];
            uint256 timestamp = _timestamps[i];
            uint256 answer = _answers[i];

            require(exists(pair), "pair don't exists");
            uint256 pairId = pairToId[pair];
            uint256 roundId = unUsedRoundId[pairId];

            Answer storage a = answers[roundId][pairId];
            a.timestamp = timestamp;
            a.value = answer;

            unUsedRoundId[pairId] += 1;

            emit AnswerUpdated(answer, roundId, timestamp, pair);
        }
    }

    function exists(string calldata pair) public view returns (bool) {
        return pairToId[pair] != 0;
    }

    function latestAnswer(string calldata pair) public view override returns (uint256) {
        require(exists(pair), "pair don't exists");

        uint256 pairId = pairToId[pair];
        uint256 lastRoundId = unUsedRoundId[pairId] - 1;
        return answers[lastRoundId][pairId].value;
    }

    function latestTimestamp(string calldata pair) public view override returns (uint256) {
        require(exists(pair), "pair don't exists");

        uint256 pairId = pairToId[pair];
        uint256 lastRoundId = unUsedRoundId[pairId] - 1;
        return answers[lastRoundId][pairId].timestamp;
    }

    function latestRound(string calldata pair) public view override returns (uint256) {
        require(exists(pair), "pair don't exists");

        uint256 pairId = pairToId[pair];
        return unUsedRoundId[pairId] - 1;
    }

    function getAnswer(string calldata pair, uint256 roundId) public view override returns (uint256) {
        require(exists(pair), "pair don't exists");

        uint256 pairId = pairToId[pair];

        return answers[roundId][pairId].value;
    }

    function getTimestamp(string calldata pair, uint256 roundId) public view override returns (uint256) {
        require(exists(pair), "pair don't exists");

        uint256 pairId = pairToId[pair];

        return answers[roundId][pairId].timestamp;
    }

    function decimals(string calldata pair) external view override returns (uint8) {
        require(exists(pair), "pair don't exists");

        uint256 pairId = pairToId[pair];

        return pairs[pairId].decimal;
    }

    function description(string calldata pair) external view override returns (string memory) {
        require(exists(pair), "pair don't exists");

        uint256 pairId = pairToId[pair];

        return pairs[pairId].description;
    }

    function version(string calldata pair) external view override returns (uint256) {
        require(exists(pair), "pair don't exists");

        uint256 pairId = pairToId[pair];

        return pairs[pairId].version;
    }

    function getRoundData(uint80 _roundId, string calldata pair)
        external
        view
        override
        returns (uint256, uint256, uint256)
    {
        require(exists(pair), "pair don't exists");

        uint256 pairId = pairToId[pair];

        uint256 answer = answers[_roundId][pairId].value;
        uint256 timestamp = answers[_roundId][pairId].timestamp;

        return (_roundId, answer, timestamp);
    }

    function latestRoundData(string calldata pair) external view override returns (uint256, uint256, uint256) {
        require(exists(pair), "pair don't exists");

        uint256 pairId = pairToId[pair];
        uint256 lastRoundId = unUsedRoundId[pairId] - 1;

        uint256 answer = answers[lastRoundId][pairId].value;
        uint256 timestamp = answers[lastRoundId][pairId].timestamp;

        return (lastRoundId, answer, timestamp);
    }
}
