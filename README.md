# euler-rs
The goal of this project is to be a command line tool to streamline the process of solving [Project Euler](https://projecteuler.net/).

A user should be able to input a problem number and generate a `.rs` file containing:
  - The problem information (title, date published, difficulty level)
  - The text for the problem itself
  - A function template for generating an answer

After a user has attempted a solution using the function template, they should be able to run their attempt and check for correctness. 

TODO:
  - Restructuring (split into modules)
  - Testing and proper error handling (file write/read, html get/parse, building and running solutions)
  - ~Solution correctness (confirm whether the output of a problem's function matches the solution to that problem)~
  - Sequential generation? (Mark problems as solved once the solution is correct, offer to generate the next problem)
  - Benchmarking - view how long solution takes to run. Possibly save a history of solution runtimes

Solutions taken from [lucky-bai's repository](https://github.com/lucky-bai/projecteuler-solutions)
