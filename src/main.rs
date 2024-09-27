use aho_corasick::AhoCorasick;
use clap::Parser;
use codegen::{Scope, Function};
use html2text::from_read;
use scraper::{Html, Selector};

use std::process::{Command, Stdio};
use std::fs::{File, create_dir_all};
use std::io::{ErrorKind, Write};

#[derive(Parser)]
struct Cli {
    p: i16,
}

fn main(){
    let args = Cli::parse();
    let url = format!("https://projecteuler.net/problem={}", args.p);
 
    create_dir_all("problems").expect("failed to create problem directory");
    create_dir_all("solutions").expect("failed to create solutions directory");
    
    let file_name = format!("{:0>4}", format!("{}", args.p));
    let path = String::from("problems/") + &file_name + ".rs";
    
    let file = File::create_new(&path);
    match file {
        Ok(new_file) => {
            generate_problem_file(args.p, &url, new_file);
            return;
        },
        Err(ref e) => match e.kind() {
            ErrorKind::AlreadyExists => {
                let answer = compile_and_run(&file_name);
                submit_answer(answer);
            }, 
            _ => panic!("Cannot read from file: {}, error: {}", path, e),
        },
    }
}

fn generate_problem_file(p: i16, url: &str, mut file: File) {
    let html: String = get_html(url);
    let problem_strings = parse_html(&html); 
    
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
            string.pop();
            write!(file, "/*\n{}\n*/\n\n", string).expect("Failed to write problem content to file");
        }
    });
    
    let code_template = generate_code_template(fn_name);
    write!(file, "{}", code_template).expect("Failed to write function template to file");
    
    println!("Generated problem file");
}

fn get_html(url: &str) -> String {
    return reqwest::blocking::get(url)
        .unwrap()
        .text()
        .unwrap();
}

fn parse_html(html: &str) -> Vec<String> {
    let document = Html::parse_document(&html); 
    let mut problem_strings: Vec<String> = Vec::new();
    
    // Selectors
    let title_selector = Selector::parse("h2").expect("unable to find problem title");
    let description_selector = Selector::parse("span.tooltiptext_right").expect("unable to find problem description");
    let content_selector = Selector::parse("div.problem_content").expect("unable to find problem content");

    problem_strings.extend(document.select(&title_selector).map(|x| x.inner_html()));
    problem_strings.extend(document.select(&description_selector).map(|x| {
       return format_desc(x.inner_html());
    }));
    problem_strings.extend(document.select(&content_selector).map(|x| {
       return format_content(x.inner_html());
    }));
    
    return problem_strings;
}

fn format_content(line: String) -> String {
   let patterns = &["$", r"\dots", r"\mod", r"\equiv"];
   let replace_with = &["", "...", "%", "==="];

   let ac = AhoCorasick::new(patterns).unwrap();
   let content = ac.replace_all(&line, replace_with);
   return from_read(content.as_bytes(), 100);
}

fn format_desc(line: String) -> String {
    let desc = line.split(";")
        .enumerate()
        .filter_map(|(i, el)| (i != 1)
        .then(|| el))
        .collect::<String>();
    return from_read(desc.as_bytes(), 100);
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
        .line("return 0.to_string();");
    
    let mut main_fn = Function::new("main");
    main_fn
        .line(format!("let answer = {}();", fn_name))
        .line("println!(\"{}\", answer);");
    
    scope.push_fn(problem_fn);
    scope.push_fn(main_fn);

    return scope.to_string();
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
        
    return String::from_utf8(output.stdout).unwrap();
}

fn submit_answer(answer: String) {
    println!("{}", answer.replace("\n", ""));
}
