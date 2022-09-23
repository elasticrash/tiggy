#tiggy
----------
A Naive implementation of non Media enabled cli softphone. This my rust/sip playground

Features & Flaws:

* Autoanswers incoming calls
* Outbound calls
* State is in a messy state, but kind of useable


### Config
  ```JSON
  {
  "username": "username",
  "extension": "xxxx",
  "password": "password",
  "sip_port": 5060,
  "sip_server": "test.server.com"
}
```