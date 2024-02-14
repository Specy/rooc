

export const ex_1 = `//This is a simple diet problem
//minimize the cost of the diet
min sum((cost, i) in enumerate(C)) { cost * x_i }
s.t.
    //the diet must have at least of nutrient j
    sum(i in 0..F) { a[i][j] * x_i } >= Nmin[j] for j in 0..len(Nmin)
    
    //the diet must have at most of nutrient j
    sum(i in 0..F) { a[i][j] * x_i } <= Nmax[j] for j in 0..len(Nmax)
    
    //bound the amount of each serving of food i
    x_i <= Fmax[i] for i in 0..N
    x_i >= Fmin[i] for i in 0..N
where
    // Cost of chicken, rice, avocado
    let C = [1.5, 0.5, 2.0]
    // Min and max of: protein, carbs, fats
    let Nmin = [50, 200, 0] 
    let Nmax = [150, 300, 70]
    // Min and max servings of each food
    let Fmin = [1, 1, 1] 
    let Fmax = [5, 5, 5]
    let a = [
    //protein, carbs, fats
        [30, 0, 5], // Chicken
        [2, 45, 0], // Rice
        [2, 15, 20] // Avocado
    ]
    // Number of foods
    let F = len(a)
    // Number of nutrients
    let N = len(Nmax)
define
    x_i as PositiveReal for i in 0..N
`

export const ex_2 =`//This is the dominating set problem
min sum(u in nodes(G)) { x_u }
s.t. 
    // the variable "_" will simply ignore the value
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
`

export const roocExamples = [ex_1, ex_2]



