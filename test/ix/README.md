## Setup Integration Tests

We prepare the following helper projects as siblings of this one such that the parent folder
looks like this:

```sh
❯ tree -L 1
.
├── geyser-kafka  <- this project
├── geyser-store
└── solanax
```

1. Checkout and initialize Solanax from the parent directory of this project

```sh
git clone git@github.com:ironforge-cloud/solanax.git`
cd solanax && git checkout thlorenz/amman
yarn install
```

2. Checkout and initialize Gstore from the parent directory of this project

```sh
git clone git@github.com:ironforge-cloud/geyser-store.git
```

3. Initialize Node.js dependencies of this project

```sh
yarn install
```

## Startup Validator and Geyser Store

### Start Geyser Store

In Terminal 1

```sh
make geyser-store
```

## Start a Solana Validator with the Geyser Plugin and Amman

In Terminal 2

```sh
make amman
```

## Run Integration Tests using Solanax

With the above running in terminal 1 and 2, do the following in another terminal:

```sh
make test-post-delete
```

TODO: this needs to assert on stored account updates.

### Troubleshoot if Test fail

The below will dump all accounts that were stored in the Geyser Store:

```
make test-show-all-accounts
```

