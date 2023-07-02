# ROOC
Short for the name of the courses i took at university (Ricerca Operativa, Ottimizzazione Combinatoria)
# What it is
Rooc is a program to parse and convert general problems into linear problems, to then be converted into standard linear problems so they can be solved with the simplex.
The goal is to compile the binary to WASM and create a web wrapper to show all the individual steps needed to convert and find a solution to a problem, this way people can more easily learn how to create and solve mathematical models using the simplex method.
I want to then extend this to solve PL-01 problems using the branch & bound method using a more formal syntax for the problems, so that the most basic optimization problems can be described (like matching, stable graph, dominant graph, etc)

# Features 
- [x] Parse of a string into a genric problem
- [ ] Linearization of a generic problem
- [x] Transformation of a linear problem into the standard form
- [x] Two step method using artifical variables to find a valid basis for the standard form problem
- [x] Simplex to find the optimal solution of a problem
- [ ] Formal definition of a problem, (sum function and generic variables)
- [ ] Constants and arrays in the formal definition of a problem
- [ ] List of modifications from the start of the problem to the end of the solution
- [ ] Compilation to WASM
- [ ] Website to show the different steps of solving the problem

# Notes
This project is purely educational, it shouldn't be used to solve serious problems as it won't be optimized for big calculations
