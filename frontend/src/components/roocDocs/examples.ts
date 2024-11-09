export const ex_1 = {
    code: `//This is a simple diet problem
//minimize the cost of the diet
min sum((cost, i) in enumerate(C)) { cost * x_i }
s.t.  
    //the diet must have at least of nutrient j
    sum(i in 0..F) { a[i][j] * x_i} >= Nmin[j] for j in 0..len(Nmin)
    //the diet must have at most of nutrient j
    sum(i in 0..F) { a[i][j] * x_i } <= Nmax[j] for j in 0..len(Nmax)
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
    let n = len(Nmax)
define
    //bound the amount of each serving of food i
    x_i as NonNegativeReal(Fmin[i], Fmax[i]) for i in 0..n`,
    name: "Diet Problem",
    description: `The diet problem is a classic optimization problem where the goal is to find the optimal diet that meets the nutritional requirements at the lowest cost.`
}

export const ex_2 = {
    code: `//minimize the number of selected nodes
min sum(u in nodes(G)) { x_u }
s.t. 
    // the variable "_" will simply ignore the value
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
`,
    name: "Dominating Set Problem",
    description: "In the dominating set problem, the goal is to find the smallest set of nodes in a graph such that every node in the graph is either in the set or adjacent to a node in the set, as in, the nodes are either dominant or adjacent to a dominant node (dominated)."
}

export let ex_3 = {
    code: `//maximize the value of the bag
max sum((value, i) in enumerate(values)) { value * x_i }
s.t.
    //make sure that the selected items do not go over the bag's capacity
    sum((weight, i) in enumerate(weights)) { weight * x_i } <= capacity
where
    let weights = [10, 60, 30, 40, 30, 20, 20, 2]
    let values = [1, 10, 15, 40, 60, 90, 100, 15]
    let capacity = 102
define
    x_i as Boolean for i in 0..len(weights)
`,
    name: "Knapsack Problem",
    description: "In the knapsack problem, you are given a set of items, each with a weight and a value, and a knapsack with a maximum capacity. The goal is to maximize the total value of the items in the knapsack without exceeding the capacity."
}

export let ex_4 = {
    code: `//how much each product will earn you
max sum((v, i) in enum(value)) { x_i * v }
subject to
    //the machines need to be within the maximum machining time
    sum((time, j) in enum(machiningTime[i])){  x_j * time } <= timeLimit[i] for i in 0..len(value)
where 
    let value = [10, 15]
    let timeLimit = [8, 6]
    let machiningTime = [
        [1, 2], // how much time machine A needs to make product A and B
        [2, 1]  // same but for machine B
    ]
define 
    x_i as NonNegativeReal for i in 0..len(value)`,
    name: "Simple machining problem",
    description: "Imagine you run a small factory that makes two types of products: A and B. Each product requires time on two machines, Machine 1 and Machine 2. Your goal is to maximize profit, but you’re limited by how many hours each machine is available each day."
}


export const roocExamples = [ex_1, ex_2, ex_3, ex_4]



