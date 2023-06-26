dev:
	solana-test-validator -r -l ./ledger --geyser-plugin-config ./config.json

logs:
	cat ledger/validator.log | grep solana_accountsdb_plugin_kafka
