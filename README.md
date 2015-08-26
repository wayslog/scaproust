# scaproust <img src=albertine-like.jpg align=right width=200 height=200>

[![Linux build](https://travis-ci.org/blabaere/scaproust.svg?branch=master)](https://travis-ci.org/blabaere/scaproust)
[![Windows build](https://ci.appveyor.com/api/projects/status/kpqdm42mhlki39fq?svg=true)](https://ci.appveyor.com/project/blabaere/scaproust)
[![License](https://img.shields.io/github/license/blabaere/scaproust.svg)](https://github.com/blabaere/scaproust)

Scaproust is an implementation of the [nanomsg](http://nanomsg.org/index.html) "Scalability Protocols" in rust.

**Experimental work !** For working stuff, please see [nanomsg-rs](https://github.com/blabaere/nanomsg.rs).  
Scaproust is internally based on [mio](https://github.com/carllerche/mio), so MS Windows is not yet supported.

## Goals
* Support for all of nanomsg's protocols.
* Support for TCP and IPC transports.
* Idiomatic rust API first, mimic the original C API second.

## Maybe
* Zero-copy, minimal allocations.
* Polling on several sockets.
* Low-latency (current design use channels between user facing functions and system functions).
* Other transports (Inproc, TLS, WebSockets).
* Async API, using future/promise to represent send/recv results.
* Efficient nonblocking operations (difficult due to the above mentioned use of channels).

## Non goals
* Ability to use a socket as a raw file descriptor with system level functions.

## Progress
- [ ] Protocols
  - [x] PAIR
  - [x] BUS
  - [ ] REQREP
    - [x] REQ
    - [ ] REQ resend
    - [x] REP
  - [ ] PUBSUB
    - [x] PUB
    - [x] SUB
    - [ ] SUB subscription filter
  - [x] PIPELINE
    - [x] PUSH
    - [x] PULL
  - [ ] SURVEY
    - [x] SURVEYOR
    - [ ] SURVEYOR deadline
    - [x] RESPONDENT  

- [ ] Transports
  - [x] TCP
  - [x] IPC (*nix only)
  - [ ] INPROC  

- [ ] Basic features
  - [x] Send (buffer only)
  - [x] Recv (buffer only)
  - [x] Connect 
  - [x] Reconnect on failure
  - [x] Bind
  - [x] Rebind on failure
  - [ ] Device
  - [ ] Logs
  - [ ] Statistics

- [ ] Advanced features
  - [ ] Send (scatter array + control header)
  - [ ] Recv (scatter array + control header)
  - [ ] Fair queuing
  - [ ] Load balancing
  - [ ] Send priority
  - [ ] Recv priority

- [ ] Socket options
  - [ ] Linger
  - [ ] Send buffer size
  - [ ] Recv buffer size
  - [x] Send timeout
  - [x] Recv timeout
  - [ ] Reconnect interval
  - [ ] Reconnect interval max
  - [ ] Send priority
  - [ ] Recv priority
  - [ ] IPV4 only
  - [ ] Socket name

- [ ] Protocol options
    - [ ] REQ resend interval
    - [ ] SURVEYOR deadline
    - [ ] SUB subscribe
    - [ ] SUB unsubscribe

- [ ] Transport options
    - [ ] TCP no delay
