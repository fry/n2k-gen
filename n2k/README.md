# n2k

NMEA 2000 `no_std` library implemented as an embedded-hal CAN driver.

Based on https://github.com/sevenseas-io/n2k

## TODO
- [x] Interface to identify fast packets and assemble
- [ ] Sending
  - [ ] Single frames
  - [ ] Fast packets
  - [x] ISO Transport Protocol multi-part messages
- [ ] ISO functions
  - [ ] Address claim
  - [ ] Product Information
  - [ ] Device Information
  - [ ] Transmit Messages
- [ ] Example test project

## Sources for the NMEA2000 format
- https://gpsd.gitlab.io/gpsd/NMEA.html
- Several parsed/reverse engineered PGNs and example traces https://github.com/canboat/canboat
- Arduino/C++ compatible NMEA2000 library including parsing of numerous messages (and a clean API): https://github.com/ttlappalainen/NMEA2000
## Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on stable Rust 1.40 and up.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
