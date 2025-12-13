// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";

/**
 * @dev adapted from OpenZeppelin Contracts and updated to uint256 values.
 * @dev each Checkpoint256 now takes up 2 storage slots instead of 1.
 */
library Checkpoints {
    /**
     * @dev A value was attempted to be inserted on a past checkpoint.
     */
    error CheckpointUnorderedInsertion();

    struct Trace256 {
        Checkpoint256[] _checkpoints;
    }

    struct Checkpoint256 {
        uint256 _key;
        uint256 _value;
    }

    /**
     * @dev Pushes a (`key`, `value`) pair into a Trace256 so that it is stored as the checkpoint.
     *
     * Returns previous value and new value.
     *
     * IMPORTANT: Never accept `key` as a user input, since an arbitrary `type(uint256).max` key set will disable the
     * library.
     */
    function push(Trace256 storage self, uint256 key, uint256 value)
        internal
        returns (uint256 oldValue, uint256 newValue)
    {
        return _insert(self._checkpoints, key, value);
    }

    /**
     * @dev Returns the value in the first (oldest) checkpoint with key greater or equal than the search key, or zero if
     * there is none.
     */
    function lowerLookup(Trace256 storage self, uint256 key) internal view returns (uint256) {
        uint256 len = self._checkpoints.length;
        uint256 pos = _lowerBinaryLookup(self._checkpoints, key, 0, len);
        return pos == len ? 0 : _unsafeAccess(self._checkpoints, pos)._value;
    }

    /**
     * @dev Returns the value in the last (most recent) checkpoint with key lower or equal than the search key, or zero
     * if there is none.
     */
    function upperLookup(Trace256 storage self, uint256 key) internal view returns (uint256) {
        uint256 len = self._checkpoints.length;
        uint256 pos = _upperBinaryLookup(self._checkpoints, key, 0, len);
        return pos == 0 ? 0 : _unsafeAccess(self._checkpoints, pos - 1)._value;
    }

    /**
     * @dev Returns the value in the last (most recent) checkpoint with key lower or equal than the search key, or zero
     * if there is none.
     *
     * NOTE: This is a variant of {upperLookup} that is optimized to find "recent" checkpoint (checkpoints with high
     * keys).
     */
    function upperLookupRecent(Trace256 storage self, uint256 key) internal view returns (uint256) {
        uint256 len = self._checkpoints.length;

        uint256 low = 0;
        uint256 high = len;

        if (len > 5) {
            uint256 mid = len - Math.sqrt(len);
            if (key < _unsafeAccess(self._checkpoints, mid)._key) {
                high = mid;
            } else {
                low = mid + 1;
            }
        }

        uint256 pos = _upperBinaryLookup(self._checkpoints, key, low, high);

        return pos == 0 ? 0 : _unsafeAccess(self._checkpoints, pos - 1)._value;
    }

    /**
     * @dev Returns the value in the most recent checkpoint, or zero if there are no checkpoints.
     */
    function latest(Trace256 storage self) internal view returns (uint256) {
        uint256 pos = self._checkpoints.length;
        return pos == 0 ? 0 : _unsafeAccess(self._checkpoints, pos - 1)._value;
    }

    /**
     * @dev Returns whether there is a checkpoint in the structure (i.e. it is not empty), and if so the key and value
     * in the most recent checkpoint.
     */
    function latestCheckpoint(Trace256 storage self)
        internal
        view
        returns (bool exists, uint256 _key, uint256 _value)
    {
        uint256 pos = self._checkpoints.length;
        if (pos == 0) {
            return (false, 0, 0);
        } else {
            Checkpoint256 storage ckpt = _unsafeAccess(self._checkpoints, pos - 1);
            return (true, ckpt._key, ckpt._value);
        }
    }

    /**
     * @dev Returns the number of checkpoints.
     */
    function length(Trace256 storage self) internal view returns (uint256) {
        return self._checkpoints.length;
    }

    /**
     * @dev Returns checkpoint at given position.
     */
    function at(Trace256 storage self, uint256 pos) internal view returns (Checkpoint256 memory) {
        return self._checkpoints[pos];
    }

    /**
     * @dev Pushes a (`key`, `value`) pair into an ordered list of checkpoints, either by inserting a new checkpoint,
     * or by updating the last one.
     */
    function _insert(Checkpoint256[] storage self, uint256 key, uint256 value)
        private
        returns (uint256 oldValue, uint256 newValue)
    {
        uint256 pos = self.length;

        if (pos > 0) {
            Checkpoint256 storage last = _unsafeAccess(self, pos - 1);
            uint256 lastKey = last._key;
            uint256 lastValue = last._value;

            // Checkpoint keys must be non-decreasing.
            if (lastKey > key) {
                revert CheckpointUnorderedInsertion();
            }

            // Update or push new checkpoint
            if (lastKey == key) {
                last._value = value;
            } else {
                self.push(Checkpoint256({_key: key, _value: value}));
            }
            return (lastValue, value);
        } else {
            self.push(Checkpoint256({_key: key, _value: value}));
            return (0, value);
        }
    }

    /**
     * @dev Return the index of the first (oldest) checkpoint with key strictly bigger than the search key, or `high`
     * if there is none. `low` and `high` define a section where to do the search, with inclusive `low` and exclusive
     * `high`.
     *
     * WARNING: `high` should not be greater than the array's length.
     */
    function _upperBinaryLookup(Checkpoint256[] storage self, uint256 key, uint256 low, uint256 high)
        private
        view
        returns (uint256)
    {
        while (low < high) {
            uint256 mid = Math.average(low, high);
            if (_unsafeAccess(self, mid)._key > key) {
                high = mid;
            } else {
                low = mid + 1;
            }
        }
        return high;
    }

    /**
     * @dev Return the index of the first (oldest) checkpoint with key greater or equal than the search key, or `high`
     * if there is none. `low` and `high` define a section where to do the search, with inclusive `low` and exclusive
     * `high`.
     *
     * WARNING: `high` should not be greater than the array's length.
     */
    function _lowerBinaryLookup(Checkpoint256[] storage self, uint256 key, uint256 low, uint256 high)
        private
        view
        returns (uint256)
    {
        while (low < high) {
            uint256 mid = Math.average(low, high);
            if (_unsafeAccess(self, mid)._key < key) {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        return high;
    }

    /**
     * @dev Access an element of the array without performing bounds check. The position is assumed to be within bounds.
     */
    function _unsafeAccess(Checkpoint256[] storage self, uint256 pos)
        private
        pure
        returns (Checkpoint256 storage result)
    {
        assembly {
            mstore(0, self.slot)
            // each checkpoint256 takes up 2 slots
            result.slot := add(keccak256(0, 0x20), mul(pos, 2))
        }
    }
}
