### Install osmosisd compatibility solution locally

This module provides Dockerfile for arm series machines. For amd series, please modify and use it according to the situation.

#### Build docker image

```
docker build -t osmosis-arm .
```

#### Start the container to compile and install osmosisd

```
docker run -it --name osmosis_node -v your_path/.osmosis:/root/.osmosisd osmosis-arm /bin/bash
```

#### Create account and generate key

```
osmosisd keys add testkey --keyring-backend test

testkey: This is the name of the key
test: This section specifies the backend type for key storage
• os：Use the operating system's keyring (for example, Keychain on macOS, GNOME Keyring on Linux).
• file：Store the key in the file system, usually in the .osmosisd directory under the user's home directory.
• test：An in-memory keyring for testing purposes. This option is suitable for use in a testing environment or during development, but is not recommended for use in a production environment.
```

Because the container is mounted in the local directory, the generated key can be obtained locally and used for testing in the code.
If you need to rely entirely on the local environment for testing, you can also use osmosisd related commands to initialize the local node.

#### Testnet faucet address

https://faucet.testnet.osmosis.zone/

#### Precautions

1. After using osmosisd to generate a wallet, you need to register the address at the faucet.
2. If the account has not performed any transactions on the chain, the public key may not be set.
   The account model in the Cosmos SDK only associates a public key with an address on the first transaction.
   Therefore, if the account did not send any transactions, its public key field will be nil.
3. The expiry passed in when calculating the msghash value should be the latest current time on the chain + 1000
