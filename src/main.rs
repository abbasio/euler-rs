use clap::Parser;
use codegen::{Scope, Function};
use scraper::{Html, Selector};

use std::fs;

#[derive(Parser)]
struct Cli {
    problem: i16,
}

fn main(){
    let args = Cli::parse();
    generate(args.problem);
}

fn generate(p: i16) {
    // HTML Get/Parse
    let html = reqwest::blocking::get(format!("https://projecteuler.net/problem={p}"))
        .unwrap()
        .text()
        .unwrap();

    let document = Html::parse_document(&html); 
    let title_selector = Selector::parse("h2").expect("unable to find problem title");
    
    let description_selector = Selector::parse("span.tooltiptext_right").expect("unable to find problem description");
    let content_selector = Selector::parse("div.problem_content").expect("unable to find problem content");

    let mut h2 = document.select(&title_selector).map(|x| x.inner_html());
    let desc = document.select(&description_selector).map(|x| x.inner_html());
    let problem = document.select(&content_selector).map(|x| x.inner_html());
    
    println!("Generated information for problem {p}:");
    h2.clone().for_each(|line| println!{"{}", line});
    desc.for_each(|line| println!{"{}", line});
    problem.for_each(|line| println!{"{}", line});

    // Codegen
    let mut scope = Scope::new();
    
    let function_name: &str = &h2.next()
        .expect("unable to find problem title")
        .to_lowercase()
        .replace(" ", "_");

    let mut function = Function::new(&function_name);
    function
        .ret("u32");
    
    scope.push_fn(function);
    println!("Generated function template for problem {p}:");
    println!("{:?}", scope.to_string());
}
