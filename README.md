rust-xmpp
=========

This is an early version of a XMPP library written in Rust.
At this point in time it is mostly a testbed for
[RustyXML](https://github.com/Florob/RustyXML).
More functionality will be available eventually, but may take time.

Features
--------

What works:
* Authentication
 * PLAIN
 * SCRAM-SHA-1
* Resource binding
* TLS (no certificate checking)

What does *not* work?
* doing *anything* useful
* sending stanzas
* getting callbacks for stanzas
* interacting with the connection
