# p25.rs – Project 25 (P25) radio protocol

[Documentation](http://kchmck.github.io/doc/p25/)

### Overview

This crate implements the P25 Common Air Interface radio protocol, including

- Frame synchronization and symbol decoding
- Error correction coding and decoding
- Trunking, voice, and data packet reception
- Link control and trunking signal message decoding
- Voice frame descrambling/deinterleaving

P25 is a digital radio protocol now widely adopted for public safety (police, fire, DOT,
forestry, etc.) and governmental radio communications in the US.
