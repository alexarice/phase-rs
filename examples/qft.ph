gate Z = if let |1> then -1,
gate X = if let |-> then -1,
gate S = sqrt(Z),
gate Y = if let S . |-> then -1,
gate H = if let sqrt(sqrt(Y)) . |1> then -1,
gate R2 = if let |1> then i,
gate R3 = if let |1> then ph(0.25pi),
gate R4 = if let |1> then ph(0.125pi),

gate QFT0 = id0,
gate QFT1 = H x id0; if let |1> x id0 then id0; id x QFT0,
gate QFT2 = H x id; if let |1> x id1 then R2; id x QFT1,
gate QFT3 = H x id2; if let |1> x id2 then R2 x R3; id x QFT2,
gate QFT4 = H x id3; if let |1> x id3 then R2 x R3 x R4; id x QFT3,

QFT4