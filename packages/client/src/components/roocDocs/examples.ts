export const ex_1 = {
    code: `/*
    This the diet problem, minimize the cost of the diet while 
    staying between the limits of each nutrient
*/
min sum((cost, i) in enumerate(C)) { cost * x_i }
s.t.  

    min_{nutrient[j]}: //the diet must have at least of nutrient j
        sum(i in 0..f) { a[i][j] * x_i} >= Nmin[j] for j in 0..len(Nmin)
    
    max_{nutrient[j]}: //the diet must have at most of nutrient j
        sum(i in 0..f) { a[i][j] * x_i } <= Nmax[j] for j in 0..len(Nmax)

where    
    // Cost of chicken, rice, avocado
    let C = [1.5, 0.5, 2.0]
    // Min and max of: protein, carbs, fats
    let nutrient = ["protein", "carbs", "fats"]
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
    let f = len(a)
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

export const ex_2_1 = {
    code: `//minimize the number of selected nodes
min sum(u in nodes(G)) { x_u }
s.t.
    //each node is dominant itself, or any of its neighbours is
    x_v or any((_, u) in neigh_edges(v)) { x_u } for v in nodes(G)
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
    name: "Dominating Set Problem, in logic form",
    description: "The same dominating set problem as above, but written with logic operators instead of arithmetic: rather than requiring a sum of 0/1 variables to be at least 1, each node asserts that it is dominant or that any of its neighbours is. The compiler turns each assertion into the same linear constraint."
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

export let ex_3_1 = {
    code: `//maximize the value of the bag
max value
s.t.
    //make sure that the selected items do not go over the bag's capacity
    weight: weight <= capacity

    weight = sum((w, i) in enumerate(weights)) { w * x_i } 
    value  = sum((v, i) in enumerate(values)) { v * x_i }
where
    let weights = [10, 60, 30, 40, 30, 20, 20, 2]
    let values = [1, 10, 15, 40, 60, 90, 100, 15]
    let capacity = 102
define
    x_i as Boolean for i in 0..len(weights)
    weight, value as NonNegativeReal`,
    name: "Knapsack Problem",
    description: "The same knapsack problem as before, but creating variables for the weight and cost, together with a named constraint to see the final weight of the bag."
}


export let ex_4 = {
    code: `//how much each product will earn you
max sum((v, i) in enum(value)) { x_i * v }
subject to
    //the machines need to be within the maximum machining time
    sum((time, j) in enum(machiningTime[i])){ x_j * time } <= timeLimit[i] for i in 0..len(value)
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


export let ex_5 = {
    code: `//count how many clauses are satisfied: each formula counts as 1 when true
max (a or b) + (not a or c) + (b or not c) + (not b)
s.t.
    //at least one variable must be set
    a or b or c
define
    a, b, c as Boolean
`,
    name: "Maximum satisfiability (MAX-SAT)",
    description: "A logic formula used inside arithmetic counts as 1 when true and 0 when false, so the objective can count how many clauses are satisfied. A constraint written as a bare formula is an assertion that must always hold. Here at most 3 of the 4 clauses can be satisfied at once."
}

export let ex_6 = {
    code: `//select projects to maximize profit within the budget
max sum((p, i) in enumerate(profits)) { p * x_i }
s.t.
    budget: sum((c, i) in enumerate(costs)) { c * x_i } <= maxBudget

    //project 1 needs project 0's infrastructure
    x_1 implies x_0
    //projects 2 and 3 are alternatives, at most one can be started
    not (x_2 and x_3)
    //at least one of the flagship projects must be started
    x_0 or x_3
where
    let profits = [120, 80, 100, 90]
    let costs = [50, 30, 40, 45]
    let maxBudget = 100
define
    x_i as Boolean for i in 0..len(profits)
`,
    name: "Project selection with dependencies",
    description: "A mixed model: a linear budget constraint sits next to logic constraints that express dependencies between the same Boolean decisions, such as one project requiring another, two projects being mutually exclusive, and at least one flagship project being mandatory."
}

export const roocExamples = [ex_1, ex_2, ex_2_1, ex_3, ex_3_1, ex_4, ex_5, ex_6]



