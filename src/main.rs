use aho_corasick::AhoCorasick;
use clap::Parser;
use codegen::{Scope, Function};
use html2text::from_read;
use scraper::{Html, Selector};

use std::process::Command;
use std::fs::{File, exists};
use std::io::Write;

#[derive(Parser)]
struct Cli {
    p: i16,
}

fn main(){
    let args = Cli::parse();
    let file_name = format!("{:0>7}", format!("{}.rs", args.p));
    if exists(&file_name).expect("unable to check if file exists") {
        // Compile the problem 
        Command::new("rustc")
            .arg(&file_name)
            .status()
            .expect("rustc command failed to start");
        let mut run_problem = String::from("./");
        run_problem.push_str(&file_name[..4]);
        // Run the problem
        Command::new(run_problem)
            .status()
            .expect("failed to execute problem");
    } else {
        generate(args.p);
    }
}

fn generate(p: i16) {
    let html: String = get_html(p);
    let problem_strings = parse_html(&html); 
    
    let file_name = format!("{:0>7}", format!("{}.rs", p));
    let path = &file_name;
    
    let function_name = &problem_strings[0]
        .to_lowercase()
        .replace(" ", "_");

    let mut output = File::create(path).expect("Failed to create a file at path");
    problem_strings.into_iter().for_each(|string| {
        write!(output, "/*\n{}\n*/\n\n", string).expect("Failed to write problem content to file");
    });
    
    // Codegen
    let mut scope = Scope::new();
    
    let mut problem_fn = Function::new(function_name);
    problem_fn
        .ret("u32")
        .line("// Your code here")
        .line("// Feel free to create and use helper functions,")
        .line("// but make sure this function returns your answer");
    
    let mut main_fn = Function::new("main");
    main_fn
        .line(format!("let answer = {}();", function_name))
        .line("println!(\"{}\", answer);");
    
    scope.push_fn(problem_fn);
    scope.push_fn(main_fn);
    write!(output, "{}", scope.to_string()).expect("Failed to write function template to file");
    
    println!("Generated function templates for problem {p}");
}

fn get_html(p: i16) -> String {
    return reqwest::blocking::get(format!("https://projecteuler.net/problem={p}"))
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
