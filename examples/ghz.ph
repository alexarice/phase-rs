gate Z = if let |1> then -1,
gate X = if let |-> then -1,
gate S = sqrt(Z),
gate Y = if let S . |-> then -1,
gate H = if let sqrt(sqrt(Y)) . |1> then -1,

H x id4; if let |1> x id4 then X x X x X x X