gate X = if let |-> then -1,
gate CX = if let |1> x id then X,
gate XC = if let id x |1> then X,
gate Swap = if let CX then XC,

Swap