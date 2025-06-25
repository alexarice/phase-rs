gate CX = if let |1-> then -1,
gate XC = if let |-1> then -1,
gate Swap = if let CX . |-1> then -1,

if let id x (if let |1-> then -1) . id x |-> x |1> then (id x -1);
if let (if let |1-> then -1) x id . |-> x |1> x id then (-1 x id)
