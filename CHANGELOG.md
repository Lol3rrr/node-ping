# Changelog

## v0.1.4
* First initial version

## v0.2.0
* Delay between ping checks can now be configured
	- New Config Option `ping_interval: x` in seconds
	- Defaults to 30 seconds
* Added a random jitter to the delays to distribute the loads on the network
* Added support for multiple notification targets (even though there is still only the discord webhook)
	- Renamed Config Option `notify_target` -> `notify_targets`

## v0.2.1
* Now sends back up notification after a node has been marked as pending
