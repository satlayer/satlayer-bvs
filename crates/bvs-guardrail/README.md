# BVS Guardrail

BVS Guardrail is a smart contract that serves as a final check for the slashing request before it can be finalized.

This contract allows eligible voters to approve or reject slashing requests.
It is designed
to ensure
that slashing requests are only finalized
if they have been approved by a sufficient number of eligible voters within the specified voting period.

This contract is largely adapted from CW3 multisig specs and implementation.
