DIR=$(dir $(abspath $(lastword $(MAKEFILE_LIST))))

LEDGER=$(DIR)/ledger
CONFIG=$(DIR)/config.json

LOG_LEVEL=INFO
RUST_LOG=solana_accountsdb_plugin_kafka=INFO,solana_geyser_plugin_manager=INFO,DEBUG,ERROR

AMMAN=./node_modules/.bin/amman

amman:
	RUST_LOG=$(RUST_LOG) \
    $(AMMAN) start

dev:
	RUST_LOG=$(RUST_LOG) \
	solana-test-validator -r \
		--ledger $(LEDGER) \
		--geyser-plugin-config $(CONFIG)

dev-log:
	RUST_LOG=$(RUST_LOG) \
	solana-test-validator -r \
		--log \
		--ledger $(LEDGER) \
		--geyser-plugin-config $(CONFIG) 

# The below does not work since it tries to start a new validator
# It seems a _plugin_ subcommand was added to the test-validator in [this
# diff](https://github.com/solana-labs/solana/pull/30352/files#diff-8ecaeb68b49224aee20774bb6fb002735dfff3409db871bba785353cbdf51c1cR1468).o
#
# At that point the solana version was `1.16.0`.
#
# Same _subcommand_ [on master](https://github.com/solana-labs/solana/blob/master/validator/src/cli.rs#L1520)
plugin-reload:
	solana-test-validator \
		plugin reload \
		--name solana-accountsdb-plugin-kafka \
		--config $(CONFIG)
	
# Flag `--geyser-plugin-config` requires `v1.16.x` of solana tools
verify:
	RUST_LOG=$(RUST_LOG) \
	solana-ledger-tool verify \
		--ledger $(LEDGER) \
		--geyser-plugin-config $(CONFIG)

accounts:
	solana-ledger-tool accounts \
		--ledger $(LEDGER) 

logs:
	cat $(LEDGER)/validator.log | grep solana_accountsdb_plugin_kafka
