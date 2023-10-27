# ROOC
Short for the name of the courses i took at university (Ricerca Operativa, Ottimizzazione Combinatoria)
# What it is
Rooc is a program to parse and convert general problems into linear problems, to then be converted into standard linear problems so they can be solved with the simplex.
The goal is to compile the binary to WASM and create a web wrapper to show all the individual steps needed to convert and find a solution to a problem, this way people can more easily learn how to create and solve mathematical models using the simplex method.
I want to then extend this to solve PL-01 problems using the branch & bound method using a more formal syntax for the problems, so that the most basic optimization problems can be described (like matching, stable graph, dominant graph, etc)

# Features 
- [x] Parse of a string into a generic problem
- [ ] Linearization of a generic problem
- [x] Transformation of a linear problem into the standard form
- [x] Two step method using artifical variables to find a valid basis for the standard form problem
- [x] Simplex to find the optimal solution of a problem
- [x] Formal definition of a problem, (sum function and generic variables)
- [x] Constants and arrays in the formal definition of a problem
- [ ] List of modifications from the start of the problem to the end of the solution
- [ ] Integer problem definitions (bounds)
- [ ] Branch & Bound method to solve integer problems
- [x] Formal definition of PL-01 problems (done except bounds)
- [ ] Compilation to WASM
- [ ] Website to show the different steps of solving the problem


# Example
Given the formal problem: (ignore the correctness of the problem, it's just an example)
```lua
max sum(i in 0..len(C), j in 0..len(b)){ X_ij * C[i] }
s.t.
  len(C) * sum(i in 0..len(C)){ C[i] * X_ij } <= b[j] for j in 0..len(C)
where
   C = [15, 30]
   b = [20, 25]
```
It is compiled down to:
```lua
max X_0_0 * 15 + X_1_1 * 30 + X_1_0 * 30 + X_0_1 * 15
s.t.
    2 * (15 * X_0_0 + 30 * X_1_0) <= 20
    2 * (15 * X_0_1 + 30 * X_1_1) <= 25
```
To then be converted into the standard form:
```
TODO
```
To then be solved using the simplex method:
```
TODO
```

# Notes
This project is purely educational, it shouldn't be used to solve serious problems as it won't be optimized for big calculations
