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

# Logic constraints
Models can use logic operators over boolean expressions: `and` (alias `&&`), `or` (alias `||`), `not` (alias `!`), `implies` (alias `->`), `iff` (alias `<->`) and `xor`, together with the indexed aggregations `all`, `any` and `xor`. A constraint written without a comparison asserts that the logic expression must hold, and a logic expression can also be used as a 0/1 value inside arithmetic.

Affine assertions compile directly, while complex asserted formulas use directional Boolean witnesses instead of reifying every root. Logic values used in arithmetic are still represented exactly. In the vertex-cover example below, each `x_u or x_v` clause therefore becomes one linear row.

Here is the [Vertex cover](https://en.wikipedia.org/wiki/Vertex_cover) problem, where the only constraint is the logic clause "at least one endpoint of each edge must be selected":
```lua
min sum(v in nodes(G)) { x_v }
s.t.
    x_u or x_v for (u, v) in edges(G)
where
    let G = Graph {
        A -> [B, C],
        B -> [D],
        C -> [D],
        D -> [E],
        E
    }
define
    x_v as Boolean for v in nodes(G)
```

## Context-aware numeric expressions

The absolute value is available as the `abs { }` block (it replaces the older `|x|` syntax, since the pipe now belongs to `||`). The `abs`, `min` and `max` expressions inspect their consuming direction: favorable contexts use compact one-sided LP formulations, while opposite-direction and equality uses are represented exactly with Boolean selectors.

```lua
max abs { x }
s.t.
    -10 <= x
    x <= 6
define
    x as Real
```

Selector coefficients are derived from declared or inferred finite bounds; no arbitrary Big-M value is used. Bounds are inferred from domains, affine constraints, `abs <=`, `max <=` and `min >=`. Exact lowering reports a deliberate error when the required finite endpoints cannot be established. Sign-known absolute values and dominated extrema are simplified without auxiliary variables.

## Migrating to 2.0

Version 2.0 removes the old `|x|` absolute-value syntax; use `abs { x }` instead. The pipe is now available to the more common `||` alias for `or`, and the logic words `and`, `or`, `not`, `implies`, `iff`, `xor`, `true` and `false` are reserved.

The serialized expression API now consistently returns adjacent-tagged objects with the shape `{ type, value }`. Tuple variants use typed arrays: for example, `Xor`, `Implies` and `Iff` have `[lhs, rhs]`, `BinOp` has `[op, lhs, rhs]`, and `UnOp` has `[op, exp]`. Serialized constraints include `is_logic_assertion`, so bare assertions remain distinguishable from explicitly written comparisons such as `flag = true`.

Use `@specy/rooc/runtime` for runtime metadata and `@specy/rooc/pkg` for the low-level generated WASM API. Internal deep imports such as `@specy/rooc/dist/pkg/rooc` are not part of the 2.0 package contract.

### 2.1

Since 2.1, rows that the compiler generates (unnamed source constraints and lowering helpers) carry an empty `name` in the serialized `LinearModel` instead of synthetic `_c{n}` and `__helper` labels. Only names written in the source appear in the compiled output; duplicated user names are disambiguated with a `__{n}` suffix (`cap`, `cap__2`, ...).

Compiled output is now valid ROOC source, so any intermediate model can be fed back into the compiler. To support this, identifiers may start with underscores (`__t`), `__` inside a name is a literal fragment that is never substituted (`set_A__2`), and an unbound identifier index in a compound name resolves literally (`x_A` refers to the variable `x_A` when no `A` is in scope, for example after a graph expansion).
