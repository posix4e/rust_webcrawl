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
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize,Ordering};
use std::sync::mpsc::{channel,TryRecvError};
use std::thread;
use std::io::Write;

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
        Err(err) => {
            match writeln!(&mut std::io::stderr(), "Error: {}!", err) {
                Ok(_) => {},
                Err(x) => panic!("Unable to write to stderr: {}", x),
            }
            return Vec::new();
        }
    };
    return get_urls_from_html(res);
}

fn get_websites(url: String) {
    let pool = ThreadPool::new(3000);
    let running_threads = Arc::new(AtomicUsize::new(0));
    let mut found_urls:HashSet < String > = HashSet::new();
    println!("Crawling {}", url);
    let (tx, rx) = channel();
    tx.send(url).unwrap();

    let mut counter = 0;

    loop {
        let n_active_threads = running_threads.compare_and_swap(0, 0, Ordering::SeqCst);
        match rx.try_recv() {
            Ok(new_site) => {
                let new_site_copy = new_site.clone();
                let tx_copy = tx.clone();
                counter += 1;

                print!("{} ", counter);
                if !found_urls.contains(&new_site) {
                    found_urls.insert(new_site);
                    running_threads.fetch_add(1, Ordering::SeqCst);
                    let my_running_threads = running_threads.clone();
                    pool.execute(move || {
                        for new_url in get_websites_helper(new_site_copy) {
                            if new_url.starts_with("http") {
                                tx_copy.send(new_url).unwrap();
                            }
                        }
                        my_running_threads.fetch_sub(1, Ordering::SeqCst);
                    });
                }
            }
            Err(TryRecvError::Empty) if n_active_threads == 0 => break,
            Err(TryRecvError::Empty) => {
                writeln!(&mut std::io::stderr(),
                 "Channel is empty, but there are {} threads running",
                  n_active_threads);
        thread::sleep_ms(10);
            },
            Err(TryRecvError::Disconnected) => unreachable!(),
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
