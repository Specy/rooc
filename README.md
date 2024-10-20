<div align="center">
  <h1><code>ROOC</code></h1>
  <img src='./logo-original.png' width='156px'/>
  <p><strong>Mathematical models compiler</strong></p>
</div>

[Go to the documentation (WIP)](https://rooc.specy.app/docs/rooc)

The name ROOC stands for for the name of the courses i took at university (Ricerca Operativa, Ottimizzazione Combinatoria) that deals with finding solutions to mathematical optimization problems
# What it is
Rooc is a language compiler to parse and convert formal mathematical models into a static formulation. Static formulations can be fed to transformers to convert them to linear problems or linear problems in standard form.
The "language" supports formal definitions of problems, with the ability to call functions, declare constants, arrays, graphs, tuples. It also supports builtin utility functions to iterate over graphs, edges, arrays, ranges, etc.
The library is compiled as a wasm module to be used in the [web editor](https://rooc.specy.app) which supports a LSP for type checking, code completion and documentation

# Implemented Features 
- [x] Syntax and parsing
  - [x] Static block functions (min, max, mod, avg)
  - [x] Constant Graph definitions
  - [x] Iterators
  - [x] Tuples
  - [x] Iterators utility functions (for graphs, edges, etc)
  - [x] Primitive destructuring
  - [x] Formal definition of a problem (sum function and generic variables)
  - [x] Constants and multi dimensional arrays in the formal definition of a problem
  - [x] Other utility functions
  - [x] Expressions as function parameters
  - [x] Error logging and parameter validation 
  - [x] Error traces
  - [x] Primitives Operator overloading (for example, `+` for strings)
  - [x] Definition of variable bounds
- [x] Simplex resolution
  - [x] Linearization of a generic problem (done except for mod operator)
  - [x] Transformation of a linear problem into the standard form
  - [x] Two step method using artifical variables to find a valid basis for the standard form problem
  - [x] Simplex to find the optimal solution of a standard form linear problem
- [ ] Integer and binary problems resolution
  - [x] Integer and binary problem definitions (bounds)
  - [x] Integer solvers
  - [x] Binary problem solution
  - [x] Integer/Binary problem solution
  - [ ] Logic constraints
- [x] UI
  - [x] Compilation to WASM
  - [x] Create and manage your models
  - [x] Automatic compilation to a LATEX block
  - [x] LSP
    - [x] Syntax errors
    - [x] Hover types
    - [x] Type errors
    - [x] Code completion
  - [x] Language documentation 
  - [x] Show the different steps of solving the problem
  - [x] List of modifications from the start of the problem to the end of the solution


# Example
Given the formal model of the [Dominating set](https://en.wikipedia.org/wiki/Dominating_set) problem, which shows most of the features of the language:
```rust
min sum(u in nodes(G)) { x_u }
s.t. 
    x_v + sum((_, _, u) in neigh_edges(v)) { x_u } >= 1    for v in nodes(G)
where
    let G = Graph {
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
define
    x_u, x_v as Boolean for v in nodes(G), (_,_,u) in neigh_edges(v)
```
It is compiled down to:
```lua
min x_A + x_B + x_C + x_D + x_E + x_F + x_G + x_H + x_I + x_J
s.t.
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
Wrong argument Expected argument of type "Number", got "Graph" evaluating "D"
        at 3:30 D
        at 3:28 C[D]
        at 3:18 enumerate(C[D])
        at 3:9  sum(j in enumerate(C[D])) { j }
        at 3:9  sum(j in enumerate(C[D])) { j } <= x_i for i in 0..len(C)
```
The model can then be solved using the `Binary solver` pipeline, which will solve the compiled model and find the optimal solution which has value `3` with assignment:
```
F	F	F	F	T	F	F	F	T	T
```
# Notes
This project is purely educational, it shouldn't be used to solve serious problems as it won't be optimized for big calculations
