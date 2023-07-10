# Solana AccountsDB Plugin for Kafka

> This codebase was forked from the [Blockdaemon/solana-accountsdb-plugin-kafka](https://github.com/Blockdaemon/solana-accountsdb-plugin-kafka) repository

Kafka publisher for use with Solana's [plugin framework](https://docs.solana.com/developing/plugins/geyser-plugins).

## Installation

### Binary releases

Find binary releases at: https://github.com/ironforge/geyser-kafka/releases

### Building from source

#### Prerequisites

You will need version 3.12 or later of the protobuf compiler `protoc` installed.

#### Build

```shell
cargo build --release
```

- Linux: `./target/release/libsolana_accountsdb_plugin_kafka.so`
- macOS: `./target/release/libsolana_accountsdb_plugin_kafka.dylib`

**Important:** Solana's plugin interface requires the build environment of the Solana validator and this plugin to be **identical**.

This includes the Solana version and Rust compiler version.
Loading a plugin targeting wrong versions will result in memory corruption and crashes.

## Config

The config is specified via the plugin's JSON config file. It contains settings that apply to all
environments and some that are environment specific.

### Environment Config Values

This config needs to be added per environment. Thus using different allow list settings for
different Kafka clusters is possible. Each item in the `environments` array is an `EnvConfig`
and follows the below schema.

* **name** (`String`)
    * Name of the environment
* **kafka** (`HashMap<String, String>`)
    * Kafka [`librdkafka` config options](https://github.com/edenhill/librdkafka/blob/master/CONFIGURATION.md).
* **program_allowlist** (`Vec<String>`)
    * Allowlist of programs to publish.
        * If empty, all accounts are published.
        * If not empty, only accounts owned by programs in this list are published.
* **program_allowlist_url** (`String`)
    * URL to fetch allowlist updates from
    * The file must be json, and with the following schema:
      ```json
      {
        "result": [
            "11111111111111111111111111111111",
            "22222222222222222222222222222222"
        ]
      }
      ```
* **program_allowlist_auth** (`String`)
    * Allowlist Authorization header value.
        * If provided the request to the program_allowlist_url will add an
            'Authorization: <value>' header.
        * A sample auth header value would be 'Bearer my_long_secret_token'.
* **program_allowlist_expiry_sec** (`u64`)
    * Update iterval for allowlist from http url.

### Global Config Values

The below config values are global and apply to all environments. Environment specific settings
are added to the `environments` array.

* **shutdown_timeout_ms** (`u64`)
    * Time the plugin is given to flush out all messages to Kafka
        * and gracefully shutdown upon exit request.
* **update_account_topic** (`String`)
    * Kafka topic to send account updates to.
        * Omit to disable.
* **slot_status_topic** (`String`)
    * Kafka topic to send slot status updates to.
        * Omit to disable.
* **transaction_topic** (`String`)
    * Kafka topic to send transaction updates to.
        * Omit to disable.
* **update_account_topic_overrides** (`HashMap<String, HashSet<String>>`)
    * Kafka topic overrides to send specific account updates to. 
      * Omit to disable. 
      * The keys are the alternate topics and the value is a collection of program addresses.
        If an account's owner matches one of those addresses its updates are sent to the
        alternative topic instead of `[update_account_topic]`.
* **publish_all_accounts** (`bool`)
    * Publish all accounts on startup.
        * Omit to disable.
* **publish_accounts_without_signature** (`bool`)
    * Publishes account updates even if the txn_signature is not present.
        * This will include account updates that occur without a corresponding
            * transaction, i.e. caused by validator book-keeping.
        * Omit to disable.
* **wrap_messages** (`bool`)
    * Wrap all messages in a unified wrapper object.
        * Omit to disable.
* **environments** (`Vec<EnvConfig>`)
    * Kafka cluster and allow list configs for different environments.
        * See [EnvConfig].

### Example Config

```json
{
  "libpath": "/home/solana/geyser-kafka/target/release/libsolana_accountsdb_plugin_kafka.so",
  "shutdown_timeout_ms": 30000,
  "update_account_topic": "geyser.mainnet.account_update",
  "update_slot_topic": "geyser.mainnet.slot_update",
  "update_transaction_topic": "geyser.mainnet.transaction_update",
  "update_account_topic_overrides": {
    "geyser.mainnet.spl.account_update": [
      "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
      "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
    ]
  },
  "publish_all_accounts": false,
  "publish_accounts_without_signature": false,
  "wrap_messages": false,
  "environments": [
    {
      "name": "dev",
      "program_allowlist_url": "https://example.com/supported-programs",
      "program_allowlist_auth": "Bearer <dev secret bearer token>",
      "program_allowlist_expiry_sec": 30,
      "kafka": {
        "bootstrap.servers": "dev.bootstrap-server:9092",
        "sasl.username": "<username>",
        "sasl.password": "<base64 encoded password>",
        "sasl.mechanism": "SCRAM-SHA-256",
        "security.protocol": "SASL_SSL",
        "request.required.acks": "1",
        "message.timeout.ms": "30000",
        "compression.type": "lz4",
        "partitioner": "random"
      }
    },
    {
      "name": "stage",
      "program_allowlist_url": "https://example.com/supported-programs",
      "program_allowlist_auth": "Bearer <stage secret bearer token>",
      "program_allowlist_expiry_sec": 15,
      "kafka": {
        "bootstrap.servers": "stage.bootstrap-server:9092",
        "sasl.username": "<username>",
        "sasl.password": "<base64 encoded password>",
        "sasl.mechanism": "SCRAM-SHA-256",
        "security.protocol": "SASL_SSL",
        "request.required.acks": "1",
        "message.timeout.ms": "30000",
        "compression.type": "lz4",
        "partitioner": "random"
      }
    }
  ]
}
```

### Message Keys

The message types are keyed as follows:
- **Account update:** account address (public key)
- **Slot status:** slot number
- **Transaction notification:** transaction signature

### Filtering

~~If `program_ignores` are specified, then these addresses will be filtered out of the account updates
and transaction notifications.  More specifically, account update messages for these accounts will not be emitted,
and transaction notifications for any transaction involving these accounts will not be
emitted.~~
`program_ignores` were removed and only `program_allowlist` is supported now.

### Message Wrapping

In some cases it may be desirable to send multiple types of messages to the same topic,
for instance to preserve relative order.  In this case it is helpful if all messages conform to a single schema.
Setting `wrap_messages` to true will wrap all three message types in a uniform wrapper object so that they
conform to a single schema.

Note that if `wrap_messages` is true, in order to avoid key collision, the message keys are prefixed with a single byte,
which is dependent on the type of the message being wrapped.  Account update message keys are prefixed with
65 (A), slot status keys with 83 (S), and transaction keys with 84 (T).

## Buffering

The Kafka producer acts strictly non-blocking to allow the Solana validator to sync without much induced lag.
This means incoming events from the Solana validator will get buffered and published asynchronously.

When the publishing buffer is exhausted any additional events will get dropped.
This can happen when Kafka brokers are too slow or the connection to Kafka fails.
Therefor it is crucial to choose a sufficiently large buffer.

The buffer size can be controlled using `librdkafka` config options, including:
- `queue.buffering.max.messages`: Maximum number of messages allowed on the producer queue.
- `queue.buffering.max.kbytes`: Maximum total message size sum allowed on the producer queue.
