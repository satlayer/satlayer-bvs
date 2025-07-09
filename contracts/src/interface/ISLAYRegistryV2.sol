// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {RelationshipV2} from "../RelationshipV2.sol";

interface ISLAYRegistryV2 {
    struct Service {
        /// @dev Whether the service is registered.
        bool registered;
        /// @dev Id of the slash parameter for the service. Stored in {_slashParameters} array.
        /// If slashing is disabled, this will be 0.
        uint32 slashParameterId;
        /// @dev the minimum withdrawal delay operators must have to be actively registered to this service.
        uint32 minWithdrawalDelay;
    }

    struct Operator {
        /// @dev Whether the operator is registered.
        bool registered;
        /// @dev The withdrawal delay in seconds before the stakes can be withdrawn from the vault.
        /// By default, this will be {DEFAULT_WITHDRAWAL_DELAY} (7 days).
        uint32 withdrawalDelay;
    }

    /**
     * @dev The Slash Parameter for particular service.
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
     * Register the caller as an service provider.
     * URI and name are not stored on-chain, they're emitted in an event {MetadataUpdated} and separately indexed.
     * The caller can both be a service and an operator. This relationship is not exclusive.
     *
     * @param uri URI of the service's project to display in the UI.
     * @param name Name of the service's project to display in the UI.
     */
    function registerAsService(string memory uri, string memory name) external;

    /**
     * Register the caller as an operator.
     * URI and name are not stored on-chain, they're emitted in an event {MetadataUpdated} and separately indexed.
     * The caller can both be a service and an operator. This relationship is not exclusive.
     *
     * @param uri URI of the operator's project to display in the UI.
     * @param name Name of the operator's project to display in the UI.
     */
    function registerAsOperator(string memory uri, string memory name) external;

    /**
     * @dev Update metadata for the service or operator.
     * This function can be called by both services and operators.
     * Emits a `MetadataUpdated` event with the new URI and name.
     *
     * Name and URI are not validated or stored on-chain.
     *
     * @param uri URI of the provider's project to display in the UI.
     * @param name Name of the provider's project to display in the UI.
     */
    function updateMetadata(string memory uri, string memory name) external;

    /**
     * @dev To register an operator to a service (the caller is the service).
     * @param operator address of the operator to pair with the service.
     *
     * To call this function, the following conditions must be met:
     *  - Service must be registered via {registerAsService}
     *  - Operator must be registered via {registerAsOperator}
     *
     * If the operator has registered this service, the relationship status will be set to `RelationshipV2.Status.Active`.
     * Else the relationship status will be set to `RelationshipV2.Status.ServiceRegistered`.
     */
    function registerOperatorToService(address operator) external;

    /**
     * @dev Deregister an operator from a service (the caller is the service).
     * @param operator address of the operator to opt out of the relationship.
     */
    function deregisterOperatorFromService(address operator) external;

    /**
     * @dev To register an service to a operator (the caller is the operator).
     * @param service address of the service to pair with the operator.
     *
     * To call this function, the following conditions must be met:
     *  - Service must be registered via {registerAsService}
     *  - Operator must be registered via {registerAsOperator}
     *
     * If the service has registered this service, the relationship status will be set to `RelationshipV2.Status.Active`.
     * Else the relationship status will be set to `RelationshipV2.Status.OperatorRegistered`.
     */
    function registerServiceToOperator(address service) external;

    /**
     * @dev Deregister an service from a operator (the caller is the operator).
     * @param service address of the service to opt out of the relationship.
     */
    function deregisterServiceFromOperator(address service) external;

    /**
     * @dev Get the `RegistrationStatus` for a given service-operator pair at the latest checkpoint.
     * @param service The address of the service.
     * @param operator The address of the operator.
     * @return RelationshipV2.Status The latest relationship status for the service-operator pair.
     */
    function getRelationshipStatus(address service, address operator) external view returns (RelationshipV2.Status);

    /**
     * @dev Get the `RelationshipV2.Status` for a given service-operator pair at a specific timestamp.
     * @param service The address of the service.
     * @param operator The address of the operator.
     * @param timestamp The timestamp to check the relationship status at.
     * @return RelationshipV2.Status The relationship status at the specified timestamp.
     */
    function getRelationshipStatusAt(address service, address operator, uint32 timestamp)
        external
        view
        returns (RelationshipV2.Status);

    /**
     * Check if an account is registered as a operator.
     * @param account The address to check.
     * @return True if the address is registered as an operator, false otherwise.
     */
    function isOperator(address account) external view returns (bool);

    /**
     * Check if an address is registered as a service.
     * @param account The address to check.
     * @return True if the address is registered as a service, false otherwise.
     */
    function isService(address account) external view returns (bool);

    /**
     * @notice Set the withdrawal delay for an operator's vault. Only the operator can set this value.
     * @param delay The delay in seconds before a withdrawal can be processed.
     */
    function setWithdrawalDelay(uint32 delay) external;

    /**
     * @notice Get the withdrawal delay for an operator's vault.
     * @param operator The address of the operator.
     * @return uint32 The withdrawal delay in seconds.
     */
    function getWithdrawalDelay(address operator) external view returns (uint32);

    /**
     * @dev For services to enable slashing by providing slash parameters {SlashParameter}.
     * The {msg.sender} must be a registered service.
     *
     * @param parameter The slash parameters to be set for the service.
     * @notice
     * - The `destination` address is where the slash collateral will be moved to at the end of the slashing lifecycle.
     * - The `maxMbips` is the maximum slashable amount represented in bips at milli unit.
     * 1 Milli-Bip is 0.00001%, so at 100% the milli bip is 10,000,000.
     * - The `resolutionWindow` is the time window in seconds at which an operator can refute slash accusations.
     */
    function enableSlashing(SlashParameter calldata parameter) external;

    /**
     * @dev For service to disable slashing for itself.
     * - The {msg.sender} must be a registered service.
     * - Disabling slashing will set the slash parameter ID to 0.
     * - This will not remove existing slash relationships
     * - New slash relationships will not be created when operator {enableSlashing(address)} is called.
     */
    function disableSlashing() external;

    /**
     * @dev For operator to approve (enable, disable or update) slashing for a service it's validating.
     * - The {msg.sender} must be a registered operator.
     * - The service and operator must have an active relationship.
     * - To enable slashing, the service must have already enabled slashing via {enableSlashing(SlashParameter)}.
     * - To disable slashing, the service must have already disabled slashing via {disableSlashing()}.
     * - To update (set new parameters), the service must have a new set of slash parameters registered via {enableSlashing(SlashParameter)}.
     * - If no update is registered, the function will revert.
     *
     * @param service The address of the service for which slashing is being enabled.
     */
    function approveSlashingFor(address service) external;

    /**
     * @dev Get the current slash parameters for a given service.
     * @param service The address of the service.
     * @return SlashParameter The slash parameters for the service.
     */
    function getSlashParameter(address service) external view returns (SlashParameter memory);

    /**
     * @dev Set the maximum number of active relationships for services and operators.
     * Only the contract owner can call this function.
     * @param maxActive The new maximum number of active relationships (must be > 0 and > current maxActive).
     */
    function setMaxActiveRelationships(uint8 maxActive) external;

    /**
     * @dev Get the current maximum number of active relationships.
     * @return uint8 The maximum number of active relationships allowed.
     */
    function getMaxActiveRelationships() external view returns (uint8);

    /**
     * @dev Get the slash parameters which an operator has opted in at given timestamp.
     * @param service The address of the service.
     * @param operator The address of the operator.
     * @param timestamp The timestamp in question.
     * @return SlashParameter The slash parameters for the service.
     */
    function getSlashParameterAt(address service, address operator, uint32 timestamp)
        external
        view
        returns (SlashParameter memory);

    /**
     * @dev Set the minimum withdrawal delay for service. All of the service's active operators must respect this delay, else revert.
     * This function can only be called by the service.
     * @param delay The new minimum withdrawal delay in seconds.
     */
    function setMinWithdrawalDelay(uint32 delay) external;

    /**
     * @dev Get the minimum withdrawal delay for a service.
     * @param service The address of the service.
     * @return uint32 The minimum withdrawal delay in seconds.
     */
    function getMinWithdrawalDelay(address service) external view returns (uint32);
}
