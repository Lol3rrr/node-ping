# Node-Ping
Node-ping is a simple tool that allows you to monitor reachability of nodes in a network.

## Getting Started
### Using precompiled binaries
1. Download the corresponding binary from the latest release
2. Create a config.yaml based on the example Configuration in this repo
	- A sample config can be found in under `examples/sample-config.yaml`
3. Create a systemd service (or something equivilant for your OS) to run the binary with your config
4. Done, you now get notifications for any reachability changes
