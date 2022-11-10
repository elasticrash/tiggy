#tiggy
----------
A Naive implementation of non Media enabled cli softphone. This my rust/sip playground

Features & Flaws:

* Autoanswers incoming calls
* Can Make outbound calls
* State is in a messy state, but kind of useable
* For the time it logs all SIP messages in a file
* Its only been tested in few specific setups

I am dropping the TUI interface in favour for an http interface. 

Ill keep the tui tagged if anyone is interested.

### Config
  ```JSON
  {
  "username": "username",
  "extension": "xxxx",
  "password": "password",
  "sip_port": 5060,
  "sip_server": "test.server.com",
  "pcap": "3588BAE5-461C-4B83-B99E-287DEAE44B0E"
}
```

Pcap property is optional and it's the name of the interface you need to monitor.

#### Windows
* Install Npcap.
* Download the Npcap SDK.
* Add the SDK's /Lib or /Lib/x64 folder to your LIB environment variable.

#### Linux (not tested yet)

Install the libraries and header files for the libpcap library. For example:

* On Debian based Linux: install libpcap-dev.