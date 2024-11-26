# euler-rs
This project is a command line tool that streamlines the process of solving [Project Euler](https://projecteuler.net/) in Rust. You can use euler-rs to both generate a template file for solving any given Project Euler problem, and to check your solutions for correctness.

## Installation

1. [Install Rust and Cargo](https://www.rust-lang.org/tools/install)
2. Run the following command from your terminal:
```
cargo install euler-rs
```
## Usage
1. Navigate to whichever directory you want your work to be stored in
```
$ mkdir ~/project-euler
$ cd ~/project-euler 
```
2. Call euler-rs using the following command, where \<p> is the problem number you want to generate a template for.
```
$ euler-rs --problem <p>
```

3. euler-rs will create a `problems` directory for you with a template file for the problem inside. See [here](https://github.com/abbasio/euler-rs/blob/main/problems/example.rs) for an example of a generated template file.
4. Once you've attempted a solution, you can once again run the same command to evaluate your solution.
```
$ euler-rs --problem <p>
```
Solutions are taken from [lucky-bai's repository](https://github.com/lucky-bai/projecteuler-solutions)
