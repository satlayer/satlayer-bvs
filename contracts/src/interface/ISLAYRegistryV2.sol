// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {RelationshipV2} from "../RelationshipV2.sol";

/**
 * @title SLAY Registry Interface
 * @notice Interface for the registry that manages services and operators in the SatLayer ecosystem
 * @dev This interface defines the contract methods for registering services and operators,
 * managing their relationships, and handling slashing parameters
 */
interface ISLAYRegistryV2 {
    struct ServiceEntry {
        /// @dev Whether the service is registered.
        bool registered;
        /// @dev Id of the slash parameter for the service. Stored in {_slashParameters} array.
        /// If slashing is disabled, this will be 0.
        uint32 slashParameterId;
        /// @dev the minimum withdrawal delay operators must have to be actively registered to this service.
        uint32 minWithdrawalDelay;
    }

    struct OperatorEntry {
        /// @dev Whether the operator is registered.
        bool registered;
        /// @dev The withdrawal delay in seconds before the stakes can be withdrawn from the vault.
        /// By default, this will be {DEFAULT_WITHDRAWAL_DELAY} (7 days).
        uint32 withdrawalDelay;
    }

    /**
     * @dev The Slash Parameter for a particular service.
     * This struct defines the parameters for slashing in the ecosystem.
     */
    struct SlashParameter {
        /**
         * The address at which the slash collateral from the vault
         * will be moved to at the end of slashing lifecycle.
         */
        address destination;
        /**
         * The maximum amount that can be slash are represented in bips at milli unit (1000x smaller than bips).
         * Between 0.00001% to 100%: 1 to 10,000,000.
         * - 1 bips = 1000 mbips.
         * - 1 mbips is 0.00001%
         * - 10,000,000 mbips is 100%
         */
        uint24 maxMbips;
        /**
         * The time window in seconds at which operator can refute slash accusations.
         * The exact mechanics are to be defined by the service.
         */
        uint32 resolutionWindow;
    }

    /*//////////////////////////////////////////////////////////////
                                 ERRORS
    //////////////////////////////////////////////////////////////*/

    /// @dev Account is not registered as an operator.
    error OperatorNotFound(address account);

    /// @dev Account is not registered as a service.
    error ServiceNotFound(address account);

    /// @dev the operator is already actively registered to max number of services.
    error OperatorRelationshipsExceeded();

    /// @dev the service is already actively registered to max number of operators.
    error ServiceRelationshipsExceeded();

    /*//////////////////////////////////////////////////////////////
                                EVENTS
    //////////////////////////////////////////////////////////////*/

    /// @dev Emitted when a service is registered.
    event ServiceRegistered(address indexed service);

    /// @dev Emitted when a operator is registered.
    event OperatorRegistered(address indexed operator);

    /**
     * @dev Emitted when a service is registered with metadata.
     * Name and URI are not validated or stored on-chain.
     *
     * @param provider The address of the service/operator provider.
     * @param uri URI of the provider's project to display in the UI.
     * @param name Name of the provider's project to display in the UI.
     */
    event MetadataUpdated(address indexed provider, string uri, string name);

    /**
     * @dev Emitted when a service-operator relationship is updated.
     * @param service The address of the service.
     * @param operator The address of the operator.
     * @param status The updated relationship status.
     * @param slashParameterId The ID of the slash parameter if slashing is enabled, otherwise 0.
     */
    event RelationshipUpdated(
        address indexed service, address indexed operator, RelationshipV2.Status status, uint32 slashParameterId
    );

    /**
     * @dev Emitted when an operator updates the withdrawal delay.
     * @param operator The address of the operator setting the delay.
     * @param delay The new withdrawal delay in seconds.
     */
    event WithdrawalDelayUpdated(address indexed operator, uint32 delay);

    /**
     * @dev Emitted when a service updates the minimum withdrawal delay.
     * @param service The address of the service setting the delay.
     * @param delay The new minimum withdrawal delay in seconds.
     */
    event MinWithdrawalDelayUpdated(address indexed service, uint32 delay);

    /**
     * @dev Emitted when {SlashParameter} for a service is updated.
     * @param service The address of the service
     * @param destination The address at which slash collateral will be moved.
     * @param maxMbips The maximum slashable amount in mbips.
     * @param resolutionWindow An operator's refutable period in seconds in the event of slash.
     */
    event SlashParameterUpdated(address indexed service, address destination, uint24 maxMbips, uint32 resolutionWindow);

    /**
     * @dev Emitted when owner updates the maximum number of active relationships for a service and operator.
     * @param maxActive The new maximum number of active relationships.
     */
    event MaxActiveRelationshipsUpdated(uint8 maxActive);

    /*//////////////////////////////////////////////////////////////
                                FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    /**
     * @notice Registers the caller as a service provider
     * @dev URI and name are not stored on-chain, they're emitted in an event {MetadataUpdated} and separately indexed.
     * The caller can be both a service and an operator simultaneously. This relationship is not exclusive.
     *
     * @param uri URI of the service's project to display in the UI
     * @param name Name of the service's project to display in the UI
     */
    function registerAsService(string calldata uri, string calldata name) external;

    /**
     * @notice Registers the caller as an operator
     * @dev URI and name are not stored on-chain, they're emitted in an event {MetadataUpdated} and separately indexed.
     * The caller can be both a service and an operator simultaneously. This relationship is not exclusive.
     *
     * @param uri URI of the operator's project to display in the UI
     * @param name Name of the operator's project to display in the UI
     */
    function registerAsOperator(string calldata uri, string calldata name) external;

    /**
     * @notice Updates metadata for the service or operator
     * @dev This function can be called by both services and operators.
     * Emits a `MetadataUpdated` event with the new URI and name.
     * Name and URI are not validated or stored on-chain.
     *
     * @param uri URI of the provider's project to display in the UI
     * @param name Name of the provider's project to display in the UI
     */
    function updateMetadata(string calldata uri, string calldata name) external;

    /**
     * @notice Registers an operator to a service (the caller is the service)
     * @dev To call this function, the following conditions must be met:
     *  - Service must be registered via {registerAsService}
     *  - Operator must be registered via {registerAsOperator}
     *
     * If the operator has registered this service, the relationship status will be set to `RelationshipV2.Status.Active`.
     * Otherwise, the relationship status will be set to `RelationshipV2.Status.ServiceRegistered`.
     *
     * @param operator Address of the operator to pair with the service
     */
    function registerOperatorToService(address operator) external;

    /**
     * @notice Deregisters an operator from a service (the caller is the service)
     * @dev Sets the relationship status to `RelationshipV2.Status.Inactive` and removes the operator
     * from the service's active relationships.
     *
     * @param operator Address of the operator to opt out of the relationship
     */
    function deregisterOperatorFromService(address operator) external;

    /**
     * @notice Registers a service to an operator (the caller is the operator)
     * @dev To call this function, the following conditions must be met:
     *  - Service must be registered via {registerAsService}
     *  - Operator must be registered via {registerAsOperator}
     *
     * If the service has registered this operator, the relationship status will be set to `RelationshipV2.Status.Active`.
     * Otherwise, the relationship status will be set to `RelationshipV2.Status.OperatorRegistered`.
     *
     * @param service Address of the service to pair with the operator
     */
    function registerServiceToOperator(address service) external;

    /**
     * @notice Deregisters a service from an operator (the caller is the operator)
     * @dev Sets the relationship status to `RelationshipV2.Status.Inactive` and removes the service
     * from the operator's active relationships.
     *
     * @param service Address of the service to opt out of the relationship
     */
    function deregisterServiceFromOperator(address service) external;

    /**
     * @notice Gets the current relationship status for a given service-operator pair
     * @dev Retrieves the status from the latest checkpoint in the relationship history
     *
     * @param service The address of the service
     * @param operator The address of the operator
     * @return The latest relationship status for the service-operator pair
     */
    function getRelationshipStatus(address service, address operator) external view returns (RelationshipV2.Status);

    /**
     * @notice Gets the relationship status for a given service-operator pair at a specific timestamp
     * @dev Retrieves the status from the checkpoint history at the specified timestamp
     *
     * @param service The address of the service
     * @param operator The address of the operator
     * @param timestamp The timestamp to check the relationship status at
     * @return The relationship status at the specified timestamp
     */
    function getRelationshipStatusAt(address service, address operator, uint32 timestamp)
        external
        view
        returns (RelationshipV2.Status);

    /**
     * @notice Checks if an account is registered as an operator
     * @dev Returns the registration status from the operators mapping
     *
     * @param account The address to check
     * @return True if the address is registered as an operator, false otherwise
     */
    function isOperator(address account) external view returns (bool);

    /**
     * @notice Checks if an account is registered as a service
     * @dev Returns the registration status from the services mapping
     *
     * @param account The address to check
     * @return True if the address is registered as a service, false otherwise
     */
    function isService(address account) external view returns (bool);

    /**
     * @notice Sets the withdrawal delay for an operator's vault
     * @dev Only the operator can set this value. The delay must be at least equal to the DEFAULT_WITHDRAWAL_DELAY (7 days)
     * and must be greater than or equal to any active service's minimum withdrawal delay.
     *
     * @param delay The delay in seconds before a withdrawal can be processed
     */
    function setWithdrawalDelay(uint32 delay) external;

    /**
     * @notice Gets the withdrawal delay for an operator's vault
     * @dev Returns the configured withdrawal delay for the specified operator
     *
     * @param operator The address of the operator
     * @return The withdrawal delay in seconds
     */
    function getWithdrawalDelay(address operator) external view returns (uint32);

    /**
     * @notice Enables slashing for a service by providing slash parameters
     * @dev The caller must be a registered service. This sets up the parameters that will be used
     * when slashing is applied to operators who have approved slashing for this service.
     *
     * @param parameter The slash parameters to be set for the service, containing:
     * - `destination`: Address where the slashed collateral will be moved to at the end of the slashing lifecycle
     * - `maxMbips`: Maximum slashable amount in milli-bips (1 milli-bip = 0.00001%, 10,000,000 milli-bips = 100%)
     * - `resolutionWindow`: Time window in seconds during which an operator can refute slash accusations
     */
    function enableSlashing(SlashParameter calldata parameter) external;

    /**
     * @notice Disables slashing for a service
     * @dev The caller must be a registered service. This function:
     * - Sets the slash parameter ID to 0 (indicating slashing is disabled)
     * - Does not remove existing slash relationships
     * - Prevents new slash relationships from being created when operators call {approveSlashingFor(address)}
     */
    function disableSlashing() external;

    /**
     * @notice Approves slashing parameters for a service the operator is validating
     * @dev This function allows an operator to enable, disable, or update slashing parameters for a service.
     * Requirements:
     * - The caller must be a registered operator
     * - The service and operator must have an active relationship
     * - To enable slashing: the service must have already enabled slashing via {enableSlashing(SlashParameter)}
     * - To disable slashing: the service must have already disabled slashing via {disableSlashing()}
     * - To update parameters: the service must have registered new slash parameters via {enableSlashing(SlashParameter)}
     * - The function will revert if no update is registered
     *
     * @param service The address of the service for which slashing is being approved
     */
    function approveSlashingFor(address service) external;

    /**
     * @notice Gets the current slash parameters for a given service
     * @dev Retrieves the slash parameters that are currently set for the specified service.
     * Reverts if slashing is not enabled for the service.
     *
     * @param service The address of the service
     * @return The slash parameters for the service
     */
    function getSlashParameter(address service) external view returns (SlashParameter memory);

    /**
     * @notice Sets the maximum number of active relationships allowed for services and operators
     * @dev Only the contract owner can call this function. The new maximum must be greater than zero
     * and greater than the current maximum.
     *
     * @param maxActive The new maximum number of active relationships
     */
    function setMaxActiveRelationships(uint8 maxActive) external;

    /**
     * @notice Gets the current maximum number of active relationships allowed
     * @dev Returns the maximum number of active relationships that a service or operator can have
     *
     * @return The maximum number of active relationships allowed
     */
    function getMaxActiveRelationships() external view returns (uint8);

    /**
     * @notice Gets the slash parameters that an operator had approved at a specific timestamp
     * @dev Retrieves the historical slash parameters for a service-operator relationship at the given timestamp.
     * Reverts if slashing was not enabled at that timestamp.
     *
     * @param service The address of the service
     * @param operator The address of the operator
     * @param timestamp The timestamp at which to check the slash parameters
     * @return The slash parameters that were in effect at the specified timestamp
     */
    function getSlashParameterAt(address service, address operator, uint32 timestamp)
        external
        view
        returns (SlashParameter memory);

    /**
     * @notice Sets the minimum withdrawal delay for a service
     * @dev This function can only be called by the service. All of the service's active operators
     * must have a withdrawal delay greater than or equal to this value, otherwise the function will revert.
     * The delay must be greater than zero.
     *
     * @param delay The new minimum withdrawal delay in seconds
     */
    function setMinWithdrawalDelay(uint32 delay) external;

    /**
     * @notice Gets the minimum withdrawal delay for a service
     * @dev Returns the configured minimum withdrawal delay for the specified service.
     * This is the minimum delay that any operator working with this service must respect.
     *
     * @param service The address of the service
     * @return The minimum withdrawal delay in seconds
     */
    function getMinWithdrawalDelay(address service) external view returns (uint32);
}
