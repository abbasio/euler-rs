use aho_corasick::AhoCorasick;
use clap::Parser;
use codegen::{Scope, Function};
use html2text::from_read;
use scraper::{Html, Selector};

use std::process::{Command, Stdio};
use std::fs::{File, create_dir_all};
use std::io::{BufReader, BufRead, ErrorKind, Write};

#[derive(Parser)]
struct Cli {
    p: i16,
}

enum ReadOrCreate {
    Read(String),
    Create(File),
}

fn main(){
    let args = Cli::parse();
 
    create_dir_all("problems").expect("failed to create problem directory");
    create_dir_all("solutions").expect("failed to create solutions directory");
     
    let file_name = format!("{:0>4}", args.p.to_string());
    let path = String::from("./problems/") + &file_name + ".rs";
    
    let result = generate_or_evaluate_file(path);
    
    match result {
        ReadOrCreate::Read(existing_file) => {
            println!("file exists, compiling...");
            let answer = compile_and_run(existing_file);
            submit_answer(answer, args.p);
        }, 
        ReadOrCreate::Create(new_file) => {
            println!("file does not exist, generating...");
            generate_problem_file(args.p, new_file);
        }
    }
}

fn generate_problem_file(p: i16, mut file: File) {
    let url = format!("https://projecteuler.net/problem={}", p);
    let html: String = get_html(url);
    let problem_strings = parse_html(html); 
    
    let fn_name = &problem_strings[0]
        .to_lowercase()
        .replace(" ", "_");

    problem_strings
        .into_iter()
        .enumerate()
        .for_each(|(i, mut string)| {
        if i == 0 {
            write!(file, "// Problem #{}: {}\n\n", p, string).expect("Failed to write problem title to file");
        } else {
            // remove trailing newline for problem content and description
            string.pop();
            write!(file, "/*\n{}\n*/\n\n", string).expect("Failed to write problem content to file");
        }
    });
    
    let code_template = generate_code_template(fn_name);
    write!(file, "{}", code_template).expect("Failed to write function template to file");
    
    println!("Generated problem file");
}

fn get_html(url: String) -> String {
    reqwest::blocking::get(url)
        .unwrap()
        .text()
        .unwrap()
}

fn parse_html(html: String) -> Vec<String> {
    let document = Html::parse_document(&html); 
    let mut problem: Vec<String> = Vec::new();
    
    // Selectors
    let title_selector = Selector::parse("h2").expect("unable to find problem title");
    let description_selector = Selector::parse("span.tooltiptext_right").expect("unable to find problem description");
    let content_selector = Selector::parse("div.problem_content").expect("unable to find problem content");

    problem.extend(document.select(&title_selector).map(|x| x.inner_html()));
    problem.extend(document.select(&description_selector).map(|x| {
       format_desc(x.inner_html())
    }));
    problem.extend(document.select(&content_selector).map(|x| {
       format_content(x.inner_html())
    }));
    
    problem
}

fn format_content(line: String) -> String {
   let patterns = &["$", r"\dots", r"\mod", r"\equiv"];
   let replace_with = &["", "...", "%", "==="];

   let ac = AhoCorasick::new(patterns).unwrap();
   let content = ac.replace_all(&line, replace_with);
   
   from_read(content.as_bytes(), 100)
}

fn format_desc(line: String) -> String {
    let desc = line.split(";")
        .enumerate()
        .filter_map(|(i, el)| (i != 1)
        .then_some(el))
        .collect::<String>();
    
    from_read(desc.as_bytes(), 100)
}

fn generate_code_template(fn_name: &str) -> String {
    let mut scope = Scope::new();
    
    let mut problem_fn = Function::new(fn_name);
    problem_fn
        .ret("String")
        .line("// Your code here\n")
        .line("// Feel free to create and use helper functions,")
        .line("// but make sure this function returns your answer\n")
        .line("// Make sure your answer is returned as a string!")
        .line("let answer = -1;")
        .line("answer.to_string()");
    
    let mut main_fn = Function::new("main");
    main_fn
        .line(format!("println!(\"{{}}\", {}());", fn_name));
    
    scope.push_fn(problem_fn);
    scope.push_fn(main_fn);

    scope.to_string()
}

fn compile_and_run(path: String) -> String {
    let solution_path = path
        .replace("problems", "solutions")
        .replace(".rs", "");

    // Compile the problem 
    Command::new("rustc")
        .arg(&path)
        .args(["-o", &solution_path])
        .status()
        .unwrap();
    
    // Run the problem, retrieve output
    let output = Command::new(solution_path)
        .stdout(Stdio::piped())
        .output()
        .unwrap();
        
    String::from_utf8(output.stdout).unwrap()
}

fn submit_answer(answer: String, problem_number: i16) {
    let attempt = answer.trim();
    let solutions_file = String::from("solutions/solutions.md");
    let result = generate_or_evaluate_file(solutions_file);
    match result {
        ReadOrCreate::Read(existing_file) => {
            println!("Solutions file found, comparing answer...");
            check_answer(existing_file, attempt, problem_number)
        },
        ReadOrCreate::Create(mut new_file) => {
            println!("Solutions file not found, attempting to generate...");
            let url = 
                "https://raw.githubusercontent.com/lucky-bai/projecteuler-solutions/refs/heads/master/Solutions.md".to_string();
            let solutions = get_html(url);
            write!(new_file, "{}", solutions).expect("Failed to write solutions to file");
            println!("Solutions file generated, re-attempting to check answer...");
            submit_answer(answer, problem_number);
        }
    }
    // If the answer for the given problem number doesn't exist, try updating the solutions file
    // If the answer still doesn't exist, throw an error 
}

fn generate_or_evaluate_file(path: String) -> ReadOrCreate {
    let file = File::create_new(&path);
    match file {
        Ok(new_file) => {
            ReadOrCreate::Create(new_file)
        },
        Err(ref e) => match e.kind() {
            ErrorKind::AlreadyExists => {
                ReadOrCreate::Read(path)
            }, 
            _ => panic!("Cannot read from file: {}, error: {}", path, e),
        },
    }
}

fn check_answer(path: String, attempt: &str, problem_number: i16) {
    let file = File::open(path).unwrap();
    let lines = BufReader::new(file).lines();
    
    for line in lines.flatten().skip(4) {
        let mut solution_row = line.split(" ");
        let solution_number = solution_row
            .next()
            .unwrap()
            .trim_end_matches('.')
            .parse::<i16>()
            .unwrap();
        if solution_number != problem_number {
            continue;
        }
        let solution = solution_row
            .next()
            .unwrap();
            
        if attempt == solution {
            solved_problem(problem_number, attempt);
            break;
        } else {
            println!("Incorrect answer to problem {}. Your answer: {}", problem_number, attempt);
            break;
        }
            
    }

}

fn solved_problem(p: i16, answer: &str) {
    println!("Solved problem {}! Your answer: {}", p, answer);
}
