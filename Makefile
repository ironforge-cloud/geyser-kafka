DIR=$(dir $(abspath $(lastword $(MAKEFILE_LIST))))

LEDGER=$(DIR)/ledger
CONFIG=$(DIR)/config.json

LOG_LEVEL=DEBUG
RUST_LOG=solana_accountsdb_plugin_kafka=$(LEVEL),solana_geyser_plugin_manager=$(LEVEL)

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
