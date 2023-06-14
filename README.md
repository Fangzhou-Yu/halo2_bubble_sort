# halo2_bubble_sort
### Fangzhou Yu

## Goal
Given an unsorted array as input, use bubble sort algorithm to sort the array within the circuit implemented using halo2.

## Assumptions
We know the length of array beforehand.\

## Current problems
Circuit Issues:
- Some cells not assigned
- Soem cells have value zero, weirdly

## stdout of cargo run
offset: 0

a: Value { inner: Some(0x000000000000000000000000000000000000000000000000000000000000005a) }

b: Value { inner: Some(0x0000000000000000000000000000000000000000000000000000000000000050) }

c: Value { inner: Some(0x0000000000000000000000000000000000000000000000000000000000000046) }

d: Value { inner: Some(0x0000000000000000000000000000000000000000000000000000000000000064) }

offset: 1

a: Value { inner: Some(0x0000000000000000000000000000000000000000000000000000000000000050) }

b: Value { inner: Some(0x0000000000000000000000000000000000000000000000000000000000000046) }

c: Value { inner: Some(0x000000000000000000000000000000000000000000000000000000000000005a) }

d: Value { inner: Some(0x0000000000000000000000000000000000000000000000000000000000000064) }

offset: 2

a: Value { inner: Some(0x0000000000000000000000000000000000000000000000000000000000000046) }

b: Value { inner: Some(0x0000000000000000000000000000000000000000000000000000000000000050) }

c: Value { inner: Some(0x000000000000000000000000000000000000000000000000000000000000005a) }

d: Value { inner: Some(0x0000000000000000000000000000000000000000000000000000000000000064) }

offset: 3

a: Value { inner: Some(0x0000000000000000000000000000000000000000000000000000000000000046) }

b: Value { inner: Some(0x0000000000000000000000000000000000000000000000000000000000000050) }

c: Value { inner: Some(0x000000000000000000000000000000000000000000000000000000000000005a) }

d: Value { inner: Some(0x0000000000000000000000000000000000000000000000000000000000000064) }

error: cell not assigned
  Cell layout in region 'first row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    1   |  X | x1 | x2 | x3 | <--{ X marks the spot! 🦜
    |    2   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 1) queries these cells.

error: cell not assigned
  Cell layout in region 'first row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    1   | x0 |  X | x2 | x3 | <--{ X marks the spot! 🦜
    |    2   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 1) queries these cells.

error: cell not assigned
  Cell layout in region 'first row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    1   | x0 | x1 |  X | x3 | <--{ X marks the spot! 🦜
    |    2   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 1) queries these cells.

error: cell not assigned
  Cell layout in region 'first row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    1   | x0 | x1 | x2 |  X | <--{ X marks the spot! 🦜
    |    2   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 1) queries these cells.

error: cell not assigned
  Cell layout in region 'first row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    1   | x0 | x1 | x2 | x3 |
    |    2   |  X | x5 | x6 | x7 | <--{ X marks the spot! 🦜

  Gate 'swap' (applied at offset 1) queries these cells.

error: cell not assigned
  Cell layout in region 'first row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    1   | x0 | x1 | x2 | x3 |
    |    2   | x4 |  X | x6 | x7 | <--{ X marks the spot! 🦜

  Gate 'swap' (applied at offset 1) queries these cells.

error: cell not assigned
  Cell layout in region 'first row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    1   | x0 | x1 | x2 | x3 |
    |    2   | x4 | x5 |  X | x7 | <--{ X marks the spot! 🦜

  Gate 'swap' (applied at offset 1) queries these cells.

error: cell not assigned
  Cell layout in region 'first row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    1   | x0 | x1 | x2 | x3 |
    |    2   | x4 | x5 | x6 |  X | <--{ X marks the spot! 🦜

  Gate 'swap' (applied at offset 1) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    2   | x0 | x1 | x2 | x3 |
    |    3   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 2) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    2   | x0 | x1 | x2 | x3 |
    |    3   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 2) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    2   | x0 | x1 | x2 | x3 |
    |    3   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 2) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    2   | x0 | x1 | x2 | x3 |
    |    3   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 2) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    4   | x0 | x1 | x2 | x3 |
    |    5   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 4) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    4   | x0 | x1 | x2 | x3 |
    |    5   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 4) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    4   | x0 | x1 | x2 | x3 |
    |    5   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 4) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    4   | x0 | x1 | x2 | x3 |
    |    5   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 4) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    7   | x0 | x1 | x2 | x3 |
    |    8   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 7) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    7   | x0 | x1 | x2 | x3 |
    |    8   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 7) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    7   | x0 | x1 | x2 | x3 |
    |    8   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 7) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    7   | x0 | x1 | x2 | x3 |
    |    8   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 7) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |   11   | x0 | x1 | x2 | x3 |
    |   12   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 11) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |   11   | x0 | x1 | x2 | x3 |
    |   12   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 11) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |   11   | x0 | x1 | x2 | x3 |
    |   12   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 11) queries these cells.

error: cell not assigned
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |   11   | x0 | x1 | x2 | x3 |
    |   12   | x4 | x5 | x6 | x7 |

  Gate 'swap' (applied at offset 11) queries these cells.

error: constraint not satisfied
  Cell layout at row 1:
    |Rotation| A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    0   | x0 | x2 | x4 | x6 | <--{ Gate 'swap' applied here
    |    1   | x1 | x3 | x5 | x7 |

  Constraint '':
    S0 * (x0 + x2 + x4 + x6 - x1 - x3 - x5 - x7) = 0

  Assigned cell values:
    x0 = 0
    x1 = 0x5a
    x2 = 0
    x3 = 0x50
    x4 = 0
    x5 = 0x46
    x6 = 0
    x7 = 0x64

error: constraint not satisfied
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    0   | x0 | x2 | x4 | x6 | <--{ Gate 'swap' applied here
    |    1   | x1 | x3 | x5 | x7 |

  Constraint '':
    S0 * (x0 + x2 + x4 + x6 - x1 - x3 - x5 - x7) = 0

  Assigned cell values:
    x0 = 0x5a
    x1 = 0
    x2 = 0x50
    x3 = 0
    x4 = 0x46
    x5 = 0
    x6 = 0x64
    x7 = 0

error: constraint not satisfied
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    0   | x0 | x2 | x4 | x6 | <--{ Gate 'swap' applied here
    |    1   | x1 | x3 | x5 | x7 |

  Constraint '':
    S0 * (x0 + x2 + x4 + x6 - x1 - x3 - x5 - x7) = 0

  Assigned cell values:
    x0 = 0x50
    x1 = 0
    x2 = 0x46
    x3 = 0
    x4 = 0x5a
    x5 = 0
    x6 = 0x64
    x7 = 0

error: constraint not satisfied
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    0   | x0 | x2 | x4 | x6 | <--{ Gate 'swap' applied here
    |    1   | x1 | x3 | x5 | x7 |

  Constraint '':
    S0 * (x0 + x2 + x4 + x6 - x1 - x3 - x5 - x7) = 0

  Assigned cell values:
    x0 = 0x46
    x1 = 0
    x2 = 0x50
    x3 = 0
    x4 = 0x5a
    x5 = 0
    x6 = 0x64
    x7 = 0

error: constraint not satisfied
  Cell layout in region 'next row':
    | Offset | A0 | A1 | A2 | A3 |
    +--------+----+----+----+----+
    |    0   | x0 | x2 | x4 | x6 | <--{ Gate 'swap' applied here
    |    1   | x1 | x3 | x5 | x7 |

  Constraint '':
    S0 * (x0 + x2 + x4 + x6 - x1 - x3 - x5 - x7) = 0

  Assigned cell values:
    x0 = 0x46
    x1 = 0
    x2 = 0x50
    x3 = 0
    x4 = 0x5a
    x5 = 0
    x6 = 0x64
    x7 = 0

thread 'main' panicked at 'circuit was not satisfied', C:\Users\AERO\.cargo\git\checkouts\halo2-d6fd4df1666d8b25\a898d65\halo2_proofs\src\dev.rs:873:13
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace