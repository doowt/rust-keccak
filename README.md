# Keccak in Rust
This repository contains a rust implementation of Keccak as specified in
FIPS 202.  The intent to be able to match the implementation to the FIPS 202
description.

The implementation includes both SHA3 and SHAKE, and supports:
- SHA3-224
- SHA3-256
- SHA3-384
- SHA3-512
- SHAKE128
- SHAKE256

## Inputs
To be able to hash inputs of non-byte-aligned length, inputs must be
of special type `BitString`, defined from a pair of inputs:
- A vector of type u64
- The length of bits in the input

A `BitString` can be defined from:
- An array of type u8
- A string of ASCII characters
- The vector and length inputs

### Byte Array
```
let test_string = BitString::from_byte_array([0x30_u8, 0x31_u8, 0x32_u8, 0x33_u8, 0x34_u8, 0x35_u8, 0x36_u8, 0x37_u8, 0x38_u8, 0x39_u8], 8 * 10);
```

### String of ASCII characters
```
let test_string = BitString::from_string("0123456789");
```

### Bit String
```
let test_string = BitString::from_bitstring(vec![0x3736353433323130_u64, 3839_u64], 80);
```
Observe that the first byte of the input (`30_u8`) is stored in the
least significant byte of the 64-bit word.

## SHA3
An instance of SHA3 can be declared as follows:
```
let mut sha3_384 = SHA3::init(384);
```

A string can be appended to the input buffer as follows:
```
sha3_384.update(test_string);
```

To compute the digest, finalise the hash function:
```
println!("Digest is: {}", sha3_384.finalise());
```

## SHAKE
An instance of SHAKE can be declared as follows:
```
let mut shake256 = SHAKE::init(256);
```

A string can be appended to the input buffer as follows:
```
shake256.update(test_string);
```

To compute the digest, finalise the XOF, providing the output size as a parameter:
```
println!("Digest is: {}", sha3_384.finalise(4096));
```
