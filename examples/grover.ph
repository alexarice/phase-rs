// Grovers algorithm searching for element 0000 (see section 4.1)

gate oracle = if let |0000> then -1,
gate diffusion = if let |++++> then -1,

oracle ; diffusion