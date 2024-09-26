use aho_corasick::AhoCorasick;
use clap::Parser;
use codegen::{Scope, Function};
use html2text::from_read;
use scraper::{Html, Selector};

use std::process::{Command, Stdio};
use std::fs::{File, exists};
use std::io::Write;

#[derive(Parser)]
struct Cli {
    p: i16,
}

fn main(){
    let args = Cli::parse();
    let url = format!("https://projecteuler.net/problem={}", args.p);
    
    let file_name = format!("{:0>7}", format!("{}.rs", args.p));
    if exists(&file_name).expect("unable to check if file exists") {
        // Compile the problem 
        Command::new("rustc")
            .arg(&file_name)
            .status()
            .unwrap();
        // Run the problem, retrieve output
        let mut run_problem = String::from("./");
        run_problem.push_str(&file_name[..4]);
        let output = Command::new(run_problem)
            .stdout(Stdio::piped())
            .output()
            .unwrap();
        
        let answer = String::from_utf8(output.stdout).unwrap();
        submit_answer(answer);
    } else {
        generate_problem_file(&url, &file_name);
    }
}

fn generate_problem_file(url: &str, file_name: &str) {
    let html: String = get_html(url);
    let problem_strings = parse_html(&html); 
    
    let fn_name = &problem_strings[0]
        .to_lowercase()
        .replace(" ", "_");

    let mut output = File::create(file_name).expect("Failed to create a file at path");
    problem_strings.into_iter().for_each(|string| {
        write!(output, "/*\n{}\n*/\n\n", string).expect("Failed to write problem content to file");
    });
    
    let code_template = generate_code_template(fn_name);
    write!(output, "{}", code_template).expect("Failed to write function template to file");
    
    println!("Generated problem file {}", file_name);
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
   let formatted_content = ac.replace_all(&line, replace_with);
   
   return from_read(formatted_content.as_bytes(), 100);
}

fn format_desc(line: String) -> String {
    let filtered_desc = line.split(";")
        .enumerate()
        .filter_map(|(i, el)| (i != 1)
        .then(|| el))
        .collect::<String>();
    return from_read(filtered_desc.as_bytes(), 100);
}

fn generate_code_template(fn_name: &str) -> String {
    let mut scope = Scope::new();
    
    let mut problem_fn = Function::new(fn_name);
    problem_fn 
        .line("// Your code here")
        .line("// Feel free to create and use helper functions,")
        .line("// but make sure this function returns your answer");
    
    let mut main_fn = Function::new("main");
    main_fn
        .line(format!("let answer = {}();", fn_name))
        .line("println!(\"{}\", answer);");
    
    scope.push_fn(problem_fn);
    scope.push_fn(main_fn);

    return scope.to_string();
}

fn submit_answer(answer: String) {
    println!("{}", answer.replace("\n", ""));
}
