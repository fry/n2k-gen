# n2k-gen

Code generator for NMEA2000 messages from pgns.xml of canboat (https://github.com/canboat/canboat/blob/master/analyzer/pgns.xml) and NMEA2000 CAN bus parsing library based on embedded_hal_can.

## n2k-codegen

Code generator for PGN parsers from completed N2K messages.

## n2k

Built to transparently handle multi-part n2k messages on top of a CAN bus abstracted by embedded_hal_can. Interfaces with the generated code by the code generator through the `PgnRegistry` trait.