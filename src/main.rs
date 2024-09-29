use aho_corasick::AhoCorasick;
use clap::Parser;
use codegen::{Scope, Function};
use html2text::from_read;
use scraper::{Html, Selector};

use std::process::{Command, Stdio};
use std::fs::{File, create_dir_all};
use std::io::{self, BufRead, ErrorKind, Write};

#[derive(Parser)]
struct Cli {
    p: i16,
}

enum ReadOrWrite {
    Read(String),
    Write(File),
}

fn main(){
    let args = Cli::parse();
    let url = format!("https://projecteuler.net/problem={}", args.p);
 
    create_dir_all("problems").expect("failed to create problem directory");
    create_dir_all("solutions").expect("failed to create solutions directory");
     
    let file_name = format!("{:0>4}", format!("{}", args.p));
    let path = String::from("problems/") + &file_name + ".rs";
    
    let result = generate_or_evaluate_file(path, file_name);
    
    match result {
        ReadOrWrite::Read(existing_file) => {
            println!("file exists, compiling...");
            let answer = compile_and_run(&existing_file);
            submit_answer(answer, args.p);
        }, 
        ReadOrWrite::Write(new_file) => {
            println!("file does not exist, generating...");
            generate_problem_file(args.p, url, new_file);
        }
    }
}

fn generate_problem_file(p: i16, url: String, mut file: File) {
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
        .line("return \"\".to_string();");
    
    let mut main_fn = Function::new("main");
    main_fn
        .line(format!("let answer = {}();", fn_name))
        .line("println!(\"{}\", answer);");
    
    scope.push_fn(problem_fn);
    scope.push_fn(main_fn);

    scope.to_string()
}

fn compile_and_run(file_name: &str) -> String {
    let problem_path = String::from("./problems/") + file_name + ".rs"; 
    let solution_path = problem_path
        .replace("problems", "solutions")
        .replace(".rs", "");

    // Compile the problem 
    Command::new("rustc")
        .arg(&problem_path)
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
    println!("{}", answer.replace("\n", ""));
    // Check the local solutions file for the answer
    let solutions_file = String::from("solutions/solutions.md");
    if let Ok (lines) = read_solutions(solutions_file) {
        for (i, line) in lines.flatten().skip(3).enumerate() {
            if i == problem_number.try_into().unwrap() {
                println!("{:?}", line.split(" ").nth(1).unwrap());
            }
        }
    }
    // If the answer for the given problem number doesn't exist, try updating the solutions file
    // If the answer still doesn't exist, throw an error 
}

fn generate_or_evaluate_file(path: String, file_name: String) -> ReadOrWrite {
    let file = File::create_new(&path);
    match file {
        Ok(new_file) => {
            ReadOrWrite::Write(new_file)
        },
        Err(ref e) => match e.kind() {
            ErrorKind::AlreadyExists => {
                ReadOrWrite::Read(file_name)
            }, 
            _ => panic!("Cannot read from file: {}, error: {}", path, e),
        },
    }
}

fn read_solutions(path: String) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(path)?;
    Ok(io::BufReader::new(file).lines())
}
