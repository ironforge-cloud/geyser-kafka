DIR=$(dir $(abspath $(lastword $(MAKEFILE_LIST))))

LEDGER=$(DIR)/ledger
SNAPSHOT=$(DIR)/snapshot
CONFIG=$(DIR)/config.json
CONFIG_LOCAL=$(DIR)/config-local.json

GEYSER_STORE=$(DIR)/../geyser-store
SOLANAX=$(DIR)/../solanax

LOG_LEVEL=INFO
RUST_LOG=solana_accountsdb_plugin_kafka=DEBUG,solana_geyser_plugin_manager=INFO

# TODO: publish amman version (https://github.com/soldev-foundation/amman)
# that supports geyser under soldev
# AMMAN=./node_modules/.bin/amman
AMMAN=amman_

amman:
	RUST_LOG=$(RUST_LOG) \
    $(AMMAN) start

amman-stop:
	$(AMMAN) stop

geyser-store:
	cd $(GEYSER_STORE) && \
	cargo run

test-post-delete: export LOCAL = 1
test-post-delete:
	cd $(SOLANAX) && \
	yarn post+delete 1 'Some post that will be immediately deleted'

test-show-all-accounts:
	@curl http://localhost:9999/accounts

test-show-solanax-accounts:
	@curl http://localhost:9999/accounts/SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ

dev-local:
	RUST_LOG=$(RUST_LOG) \
	solana-test-validator -r \
		--ledger $(LEDGER) \
		--geyser-plugin-config $(CONFIG_LOCAL)
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

replay-local:
	solana-ledger-tool verify \
		--ledger $(LEDGER) \
		--snapshot-archive-path $(SNAPSHOT) \
		--geyser-plugin-config $(CONFIG_LOCAL)

replay:
	RUST_LOG=$(RUST_LOG) \
	solana-ledger-tool verify \
		--ledger $(LEDGER) \
		--snapshot-archive-path $(SNAPSHOT) \
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

logs-tail:
	tail -f $(LEDGER)/validator.log
