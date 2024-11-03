<div align="center">
  <h1><code>ROOC</code></h1>
  <img src='./logo-original.png' width='156px'/>
  <p><strong>Mathematical models compiler</strong></p>
</div>

[![Crates.io](https://img.shields.io/crates/v/rooc.svg)](https://crates.io/crates/rooc)
[![npm](https://img.shields.io/npm/v/@specy/rooc.svg)](https://www.npmjs.com/package/@specy/rooc)

[Go to the language documentation](https://rooc.specy.app/docs/rooc)

[Go to the library documentation](https://rooc.specy.app/docs/lib)


**ROOC** stands for the courses I took in university—*Ricerca Operativa* (Operational Research) and *Ottimizzazione Combinatoria* (Combinatorial Optimization)—which deal with solving optimization models.

# What it is
**ROOC** is a compiler designed to parse and convert formal optimization models into static formulations. These static formulations can be transformed into linear models which can then be solved using optimization techniques. 

The language provides support for defining formal models, including functions, constants, arrays, graphs, tuples, etc... It also includes built-in utility functions for iterating over graphs, edges, arrays, ranges, and more.

The library is compiled as a WebAssembly (WASM) module and integrated into the [web editor](https://rooc.specy.app), which features Language Server Protocol (LSP) support for type checking, code completion, and documentation.

# Implemented Features 
- [x] Language
  - [x] Static block functions (min, max, mod, avg)
  - [x] Constant Graph definitions
  - [x] Iterators
  - [x] Tuples
  - [x] Iterators utility functions (for graphs, edges, etc)
  - [x] Primitive destructuring
  - [x] Constants and multi dimensional arrays in the formal definition of a problem
  - [x] Other utility functions
  - [x] Error logging and parameter validation 
  - [x] Error traces
  - [x] Primitives Operator overloading (for example, `+` for strings)
  - [x] Definition of variable bounds
  - [x] Javascript defined functions, define js functions to use in the model
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
    x_v + sum((_, u) in neigh_edges(v)) { x_u } >= 1 for v in nodes(G)
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
    x_u, x_v as Boolean for v in nodes(G), (_, u) in neigh_edges(v)
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
This project is purely educational, it shouldn't be used to solve serious problems as it won't be optimized for big models
