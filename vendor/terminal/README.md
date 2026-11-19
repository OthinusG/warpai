# Terminal checkpoint dependencies

The sources are from the project's locked packages `vt100` 0.16.2 and `vte`
0.15.0. Their original crates.io SHA-256 checksums are
`054ff75fb8fa83e609e685106df4faeffdf3a735d3c74ebce97ec557d5d36fd9` and
`a5924018406ce0063cd67f8e008104968b74b563ee1b85dde3ed1f7cb87d3dbd`, respectively.
Original notices and licenses remain in the package directories.

The local changes provide checkpoint access to the readiness parser's private
decoder and grid state. The desktop's separate Warp VTE dependency is unchanged.
