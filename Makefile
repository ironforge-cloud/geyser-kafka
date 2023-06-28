DIR=$(dir $(abspath $(lastword $(MAKEFILE_LIST))))

LEDGER=$(DIR)/ledger
CONFIG=$(DIR)/config.json

dev:
	solana-test-validator -r \
		--ledger $(LEDGER) \
		--geyser-plugin-config $(CONFIG) 
	
# Flag `--geyser-plugin-config` requires `v1.16.x` of solana tools
verify:
	solana-ledger-tool verify \
		--ledger $(LEDGER) \
		--geyser-plugin-config $(CONFIG)

logs:
	cat $(LEDGER)/validator.log | grep solana_accountsdb_plugin_kafka
