use aho_corasick::AhoCorasick;
use clap::Parser;
use codegen::{Scope, Function};
use html2text::from_read;
use scraper::{Html, Selector};

use std::error::Error;

#[derive(Parser)]
struct Cli {
    problem: i16,
}

fn main(){
    let args = Cli::parse();
    generate(args.problem);
}

fn generate(p: i16) {
    let html = get_html(p);
    let mut problem_strings = parse_html(&html).into_iter(); 
    

    // Codegen
    let mut scope = Scope::new();
    
    let function_name = problem_strings.clone().nth(0)
        .expect("Problem name not found")
        .to_lowercase()
        .replace(" ", "_");

    let mut function = Function::new(&function_name);
    function
        .ret("u32");
    
    scope.push_fn(function);
    println!("Generated function template for problem {p}:");
    println!("{:?}", scope.to_string());

    println!("Generated information for problem {p}:");
    problem_strings.for_each(|string| {
        println!{"/* {} */", string}
    });

}

fn get_html(p: i16) -> String {
    return reqwest::blocking::get(format!("https://projecteuler.net/problem={p}"))
        .unwrap()
        .text()
        .unwrap();
}

fn parse_html(html: &str) -> Vec<String> {
    let document = Html::parse_document(&html); 
    
    // Selectors
    let title_selector = Selector::parse("h2").expect("unable to find problem title");
    let description_selector = Selector::parse("span.tooltiptext_right").expect("unable to find problem description");
    let content_selector = Selector::parse("div.problem_content").expect("unable to find problem content");

    let problem_title = document.select(&title_selector).map(|x| x.inner_html());
    let problem_description = document.select(&description_selector).map(|x| x.inner_html());
    let problem_content = document.select(&content_selector).map(|x| x.inner_html());
    
    let mut problem_strings: Vec<String> = Vec::new();

    problem_title.for_each(|line| {
        problem_strings.push(line);
    });
    problem_description.for_each(|line| {
        let formatted_line = from_read(line.as_bytes(), 100);
        problem_strings.push(formatted_line);
    });
    problem_content.for_each(|line| {
        let formatted_line = from_read(format_problem_content(line).as_bytes(), 100);
        problem_strings.push(formatted_line);
    });

    return problem_strings;
}

fn format_problem_content(line: String) -> String {
   let patterns = &["$", r"\dots", r"\mod", r"\equiv"];
   let replace_with = &["", "...", "%", "==="];

   let ac = AhoCorasick::new(patterns).unwrap();
   return ac.replace_all(&line, replace_with);
}


