// Gate definitions from Figure 3 in paper
gate I = id,
gate Z = if let |1> then -1,
gate X = if let |-> then -1,
gate S = sqrt(Z),
gate Sdag = S ^ -1,
gate V = sqrt(X),
gate T = sqrt(S),
gate Y = if let S . |-> then -1,
gate H = if let sqrt(sqrt(Y)) . |1> then -1,
gate CZ = if let |11> then -1,
gate CX = if let |1> x id then X,

// V gate as defined in Example 4. As unitaries V = V2.
gate V2 = sqrt(X),

// Toffoli gate (not mentioned in paper but included here for completeness)
gate Toff = if let |1> x id2 then CX,

// Swap gate from Example 7
gate XC = if let |-1> then -1,
gate Swap = if let CX then XC,

// The unitaries for various gates can be checked by changing the evaluated gate below.
Swap