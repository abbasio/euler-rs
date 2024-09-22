use codegen::{Scope, Function};
use scraper::{Html, Selector};

use std::fs;

fn main(){

   // Codegen
   //let mut scope = Scope::new();

   //let mut function = Function::new("even_fibonacci_numbers");
   //function
   //    .ret("i32");
   // 
   // scope.push_fn(function);
   // println!("{:?}", scope.to_string());
    generate(1);
}

fn generate(p: i16) {
    let html = reqwest::blocking::get(format!("https://projecteuler.net/problem={p}"))
        .unwrap()
        .text()
        .unwrap();

    let document = Html::parse_document(&html); 
    let title_selector = Selector::parse("h2").expect("unable to find problem title");
    
    let description_selector = Selector::parse("span.tooltiptext_right").expect("unable to find problem description");
    let content_selector = Selector::parse("div.problem_content>p").expect("unable to find problem content");

    let h2 = document.select(&title_selector).map(|x| x.inner_html());
    let desc = document.select(&description_selector).map(|x| x.inner_html());
    let problem = document.select(&content_selector).map(|x| x.inner_html());
    
    h2.for_each(|line| println!{"{}", line});
    desc.for_each(|line| println!{"{}", line});
    problem.for_each(|line| println!{"{}", line});
}
