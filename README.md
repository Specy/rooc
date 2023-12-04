# ROOC
<img src='./logo-original.png' width='156px'/>

[Go to the documentation (WIP)](https://github.com/Specy/rooc/wiki/ROOC-%E2%80%90-Documentation)

Short for the name of the courses i took at university (Ricerca Operativa, Ottimizzazione Combinatoria)
# What it is
Rooc is a language and compiler to parse and convert formal mathematical models into a static formulation. Static formulations can then be fed to transformers to convert them to linear problems or linear problems in standard form.
The goal is to compile the binary to WASM and create a web wrapper to show all the individual steps needed to convert and find a solution to the model, this way people can more easily learn how to create and solve mathematical models.
The "language" supports formal definitions of problems, with the ability to call functions, declare constants, arrays and tuples. It also supports iterators and utility functions to iterate over graphs, edges, arrays, ranges, etc.

# Implemented Features 
- [ ] Syntax and parsing
  - [x] Basic block functions (min, max, mod)
  - [x] Constant Graph definitions
  - [x] Iterators
  - [x] Tuples
  - [x] Iterators utility functions (for graphs, edges, etc)
  - [x] Primitive destructuring
  - [x] Formal definition of a problem, (sum function and generic variables)
  - [x] Constants and multi dimensional arrays in the formal definition of a problem
  - [x] Custom functions
  - [x] Error logging and parameter validation 
  - [x] Error traces
  - [ ] Definition of variable bounds
- [ ] Simplex resolution
  - [ ] Linearization of a generic problem
  - [x] Transformation of a linear problem into the standard form
  - [x] Two step method using artifical variables to find a valid basis for the standard form problem
  - [x] Simplex to find the optimal solution of a standard form linear problem
- [ ] Integer and binary problems resolution
  - [ ] Integer problem definitions (bounds)
  - [ ] Branch & Bound method to solve integer problems
  - [ ] Other integer solution algorithms
- [ ] UI
  - [ ] List of modifications from the start of the problem to the end of the solution
  - [ ] Compilation to WASM
  - [ ] Website to show the different steps of solving the problem


# Example
Given the formal problem of the [Dominating set](https://en.wikipedia.org/wiki/Dominating_set) problem, which shows most of the features of the language:
```rust
min sum(u in nodes(G)) { x_u }
s.t. 
    x_v + sum((_, _, u) in neigh_edges(v)) { x_u } >= 1    for v in nodes(G)
where
    G = Graph {
        A -> [B, C, D, E, F],
        B -> [A, E, C, D, J],
        C -> [A, B, D, E, I],
        D -> [A, B, C, E, H],
        E -> [A, B, C, D, G],
        F -> [A, G, J],
        G -> [E, F, H],
        H -> [D, G, I],
        I -> [C, H, J],
        J -> [B, F, I]
    }
```
It is compiled down to:
```lua
min x_A + x_B + x_C + x_D + x_E + x_F + x_G + x_H + x_I + x_J
s.t
        x_A + x_B + x_D + x_C + x_F + x_E >= 1
        x_B + x_D + x_E + x_J + x_C + x_A >= 1
        x_C + x_B + x_D + x_I + x_A + x_E >= 1
        x_D + x_E + x_H + x_C + x_A + x_B >= 1
        x_E + x_B + x_D + x_C + x_A + x_G >= 1
        x_F + x_J + x_G + x_A >= 1
        x_G + x_E + x_F + x_H >= 1
        x_H + x_D + x_I + x_G >= 1
        x_I + x_J + x_H + x_C >= 1
        x_J + x_F + x_I + x_B >= 1
```
If the compilation finds a type mismatch (for example, function parameters or compound variable flattening), a stack trace will be generated:
```lua
Wrong argument Expected argument of type "GraphNode", got "Graph" evaluating "G"
        at 3:44
        at 3:32
        at 3:13
        at 3:9
```
If there were no errors during compilation, it will be then be converted into the standard form:
```
TODO
```
To then be solved using the simplex method:
```
TODO
```
# Notes
This project is purely educational, it shouldn't be used to solve serious problems as it won't be optimized for big calculations
