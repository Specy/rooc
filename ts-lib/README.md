# ROOC
An optimization modeling language and solver

<img src='https://github.com/Specy/rooc/blob/main/logo-original.png' width='156px'/>

[Go to the library documentation](https://rooc.specy.app/docs/lib)

[Go to the language documentation ](https://rooc.specy.app/docs/rooc)




# What it is
**ROOC** is a modeling language meant to parse and convert formal optimization models into static formulations. These static formulations can be transformed into linear models which can then be solved using optimization techniques. 

The language provides support for defining formal models, including functions, constants, arrays, graphs, tuples, etc... It also includes built-in utility functions for iterating over graphs, edges, arrays, ranges, and more.

The library is compiled as a WebAssembly (WASM) module and integrated into the [web editor](https://rooc.specy.app), which features Language Server Protocol (LSP) support for type checking, code completion, and documentation.


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
