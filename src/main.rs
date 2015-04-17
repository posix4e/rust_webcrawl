#![feature(plugin)]
#![plugin(regex_macros)]

extern crate regex;
extern crate hyper;
extern crate env_logger;
extern crate threadpool;

use std::env;
use std::io::Read;
use threadpool::ThreadPool;
use hyper::Client;

use std::collections::HashSet;
use hyper::client::response::Response;
use std::sync::mpsc::channel;

fn get_urls_from_html(mut response: Response) -> Vec < String > {
    let mut matched_urls = Vec::new();
    let link_matching_regex = regex!(r#"<a[^>]* href="([^"]*)"#);
    let mut body = String::new();
    response.read_to_string(&mut body).unwrap();

    for capturerer_of_captured_url in link_matching_regex.captures_iter(&body) {
        for captured_url in capturerer_of_captured_url.iter() {
            match captured_url {
                Some(url) => {
                    matched_urls.push(url.to_string());
                }
                None => {}
            }
        }
    }
    return matched_urls;
}

fn get_websites_helper(url_to_crawl: String) -> Vec < String > {
    print!("<");
    let mut client = Client::new();
    let res = match client.get(& * url_to_crawl).send() {
        Ok(res) => res,
        Err(err) => panic!("Failed to connect: {:?}", err)
    };
    return get_urls_from_html(res);
}

fn get_websites(url: String) {
    let pool = ThreadPool::new(100);
    let mut found_urls:HashSet < String > = HashSet::new();
    println!("Crawling {}", url);
    let (tx, rx) = channel();
    tx.send(url).unwrap();

    let mut counter = 0;

    while true {
        match rx.recv() {
            Ok(new_site) => {
                let new_site_copy = new_site.clone();
                let tx_copy = tx.clone();
                counter += 1;

                print!("{}>", counter);
                if !found_urls.contains(&new_site) {
                    print!("!");
                    found_urls.insert(new_site);

                    pool.execute(move || {
                        for new_url in get_websites_helper(new_site_copy) {
                            if counter > 100 && new_url.contains("reddit") {
                            } else if new_url.starts_with("http") {
                                tx_copy.send(new_url).unwrap();
                            }
                        }
                    });
                }
            }
            Err(_) => {}
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    let url = match env::args().nth(1) {
        Some(url) => url,
        None => {
            println!("Usage: client <url>");
            return;
        }
    };

    get_websites(url);
}
