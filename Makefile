.PHONY: websubmit
websubmit:
	mkdir -p /tmp/websubmit/css
	mkdir -p /tmp/websubmit/js
	RUST_BACKTRACE=full cargo run --quiet -- -i csci2390 -c sample-config.toml
