# euler-rs
The goal of this project is to be a command line tool to streamline the process of solving [Project Euler](https://projecteuler.net/).

A user should be able to input a problem number and generate a `.rs` file containing:
  - The problem information (title, date published, difficulty level)
  - The text for the problem itself
  - A function template for generating an answer

Afterwards, a user should be able to test and submit their solution - either against a hard-coded local list of solutions, or, preferably, by directly POSTing a solution attempt to project euler.
