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
plugin-reload:
	RUST_LOG=$(RUST_LOG) \
	solana-test-validator \
		--ledger $(LEDGER) \
		--geyser-plugin-config $(CONFIG) \
		plugin reload $(CONFIG)
	
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
