// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

interface ISLAYRegistry {
    /*//////////////////////////////////////////////////////////////
                                 ERRORS
    //////////////////////////////////////////////////////////////*/

    /// @dev Account is not registered as an operator.
    error OperatorNotFound(address account);

    /// @dev Account is not registered as a service.
    error ServiceNotFound(address account);

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
     * @dev Emitted when a service-operator registration status is updated.
     * @param service The address of the service.
     * @param operator The address of the operator.
     * @param status The new registration status.
     */
    event RegistrationStatusUpdated(address indexed service, address indexed operator, RegistrationStatus status);

    /**
     * @dev Emitted when an operator updates the withdrawal delay.
     * @param operator The address of the operator setting the delay.
     * @param delay The new withdrawal delay in seconds.
     */
    event WithdrawalDelayUpdated(address indexed operator, uint32 delay);

    /**
     * @dev Enum representing the registration status between a service and an operator.
     * The registration status can be one of the following:
     */
    enum RegistrationStatus {
        /**
         * Default state when neither the Operator nor the Service has registered,
         * or when either the Operator or Service has unregistered.
         * `uint8(0)` is used to represent this state, the default value.
         */
        Inactive,
        /**
         * State when both the Operator and Service have registered with each other,
         * indicating a fully established relationship.
         */
        Active,
        /**
         * This state is used when the Operator has registered an Service,
         * but the Service hasn't yet registered,
         * indicating a pending registration from the Service side.
         * This is Operator-initiated registration, waiting for Service to finalize.
         */
        OperatorRegistered,
        /**
         * This state is used when the Service has registered an Operator,
         * but the Operator hasn't yet registered,
         * indicating a pending registration from the Operator side.
         * This is Service-initiated registration, waiting for Operator to finalize.
         */
        ServiceRegistered
    }

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
     * If the operator has registered this service, the registration status will be set to `RegistrationStatus.Active`.
     * Else the registration status will be set to `RegistrationStatus.ServiceRegistered`.
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
     * If the service has registered this service, the registration status will be set to `RegistrationStatus.Active`.
     * Else the registration status will be set to `RegistrationStatus.OperatorRegistered`.
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
     * @return RegistrationStatus The latest registration status for the service-operator pair.
     */
    function getRegistrationStatus(address service, address operator) external view returns (RegistrationStatus);

    /**
     * @dev Get the `RegistrationStatus` for a given service-operator pair at a specific timestamp.
     * @param service The address of the service.
     * @param operator The address of the operator.
     * @param timestamp The timestamp to check the registration status at.
     * @return RegistrationStatus The registration status at the specified timestamp.
     */
    function getRegistrationStatusAt(address service, address operator, uint32 timestamp)
        external
        view
        returns (RegistrationStatus);

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
     * @dev If the delay is not set, it returns the default delay of 7 days.
     * @param operator The address of the operator.
     * @return uint32 The withdrawal delay in seconds.
     */
    function getWithdrawalDelay(address operator) external view returns (uint32);
}
