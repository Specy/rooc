# Slide 1: Cover

ROOC: a language and integrated environment for modeling and solving linear optimization problems

---

# Slide 2: Let's start with a problem

I'd like to start by posing a problem.

[[step 1]]
Your nutritionist has given you a diet to follow:
- the diet asks you to get at least a certain amount of some macronutrients
- but at the same time not to go over others, for example a maximum of 70 g of fat

[[step 2]]
You're at the supermarket and you need to buy the ingredients to follow the diet, spending as little as possible.
For each ingredient you know its cost and its nutritional value for each macronutrient.

[[step 3]]
Which ingredients do you choose?

---

# Slide 3: The diet problem

What I've just described is called the "diet problem", and it's a classic example used
to introduce linear programming.

This problem can be written formally as a linear programming model.

[[step 1]]
[[show the objective function and the constraints in natural language on the left, so that they look like a formal model while staying purely textual and conversational]]

[[step 2]]
Before writing the model we need a few things: a table $T$ that tells us how many grams of each nutrient there are in one serving of each ingredient, the cost $C_i$ of each ingredient, and the minimum $Nmin$ and maximum $Nmax$ values set by the diet. Finally, we need a set of decision variables $x_i$ that, for each ingredient $i$, tell us how many servings of that ingredient we buy.

[[step 3]]
[[show a mathematical linear optimization model on the right and connect the text to the matching part of the mathematical model with arrows]]

[[step 4]]
What we've just written is the linear programming formulation of the diet problem. As you can see, the numbers of the problem don't appear in the model: only their names do.

---

# Slide 4: Model and instance

[[step 1]]
The model describes how the problem is formulated,

[[step 2]]
while an instance provides the data needed so that the model can be solved by a linear programming solver.

[[step 3]]
Putting the model and the instance together, we get the compiled model: the sums are expanded and the names of the data are replaced by their values. This is what a solver can actually solve.

[[step 4]]
In general, to find an optimal solution we need a tool that goes through a series of steps:

writing the model -> adding the data of an instance -> conversion to the format accepted by the solver -> solving with the solver -> solution of the problem

---

# Slide 5: Existing tools

In the linear programming ecosystem there are already tools that can do this, but they have some limitations:
- they are tied to the ecosystem they run in
- they are black boxes
- they are hard for beginners to use and not very accessible

[[step 1]]
The goal of my thesis is to solve these three problems, by building a tool that lets students and less experienced people
solve problems and learn modeling.

---

# Slide 6: The ROOC language

Let's go back to the diet problem.

[[step 1]]
[[the mathematical model on the left]]

ROOC is a language for writing a linear optimization model formally.

[[step 2]]
[[on the right, the model rewritten in ROOC syntax, with arrows connecting each part of the mathematical model to its translation]]

Every part of the model has its counterpart: the objective function, the constraints after `s.t.`, the instance data in the `where` block, and the variables, with their domain, in the `define` block.

---

# Slide 7: ROOC is a compiler

ROOC uses a compiler to convert the model and the instance data into the format used by solvers.

In practice, it implements the steps we saw earlier as a pipeline that goes from writing the model all the way to the final result:

[[step 1]] Source (the model and the data we wrote) ->
[[step 2]] Model (the data is inserted into the model and the sums are expanded) ->
[[step 3]] LinearModel (a model in linear form, easy to convert into the solver's format) ->
[[step 4]] conversion to the format of the chosen solver ->
[[step 5]] optimal solution

And the solution answers the opening question: 1.37 servings of chicken, 4.44 of rice and no avocado, for a total of €4.28.

---

# Slide 8: Architectural decisions

ROOC, together with its default solver microlp (another contribution of this thesis), was developed entirely in Rust, a language that can compile for many different environments, including the web and embedded systems.

On top of that, the intermediate steps between the model and the solution can be displayed, so you can understand how the model was transformed.

Putting these two things together led to a development environment that runs entirely in the browser, where you can write and solve models and see all the intermediate steps.

---

# Slide 9: Demo

ROOC behaves like a real programming language: it has a type system, and tools for debugging and for checking that the model is semantically correct.

Let's see it directly in the editor, with the diet model:

- [[step 1]] **Types**: I hover over `T` and the editor shows its type, `Integer[][]`
- [[step 2]] **Errors**: I write `C * x_i` instead of `C[i] * x_i` and the editor immediately reports that `*` can't be applied to an array and a number
- [[step 3]] **Completion**: I start typing `le` and the editor suggests `len` with its signature
- [[step 4]] **Execution**: I press Run and we see the pipeline steps and the solution, the same one as before

---

# Slide 10: Linearization

Another key feature of ROOC is its ability to linearize non-linear models. For example:

[[step 1]]
- the absolute value becomes an auxiliary variable with two constraints

[[step 2]]
- minimum and maximum, when one side is enough, don't need binary variables

[[step 3]]
- logical operators on boolean variables become linear constraints: `x or y` becomes `x + y >= 1`

---

# Slide 11: The microlp solver

Another important part of the thesis is microlp, a fork of minilp, a linear programming solver whose development had been archived by its author. I added support for integer variables to minilp, turning it into a mixed-integer linear programming solver.

microlp has been well received: it has passed one and a half million downloads on crates.io, and it's used by good_lp, the most popular optimization library in Rust, as its solver written entirely in Rust.

---

# Slide 12: Limitations and future work

## Limitations

Since it's designed mainly for ease of use and teaching, the language lacks some features found in other modeling languages: for example, an arithmetic comparison can't appear inside a logical expression. However, its architecture makes it possible to add new ones in the future.

## Future work

- The language could support indicator constraints, that is, constraints made of an arithmetic expression and a logical condition that switches them on or off, linearized with the big M technique
- The web platform could include documentation or an interactive course, to better teach linear programming and how to use ROOC

---

# Slide 13: Closing

Thank you for listening.
