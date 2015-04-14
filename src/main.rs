#![feature(plugin)] #![plugin(regex_macros)]
extern crate regex;
extern crate hyper;
extern crate env_logger;


use std::env;
use std::io::Read;

use hyper::Client;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use hyper::client::response::Response;
use std::task::TaskBuilder;


fn get_urls_from_html(mut response:Response) -> Vec<String> {
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

fn get_websites_helper(url_to_crawl:String) -> Vec<String> {
				print!("<");
				let mut client = Client::new();

				let mut res = match client.get(&*url_to_crawl).send() {
								Ok(res) => res,
												Err(err) => panic!("Failed to connect: {:?}", err)
				};
				return get_urls_from_html(res);
}

fn get_websites(url: String) {
				let config = PoolConfig::new();
				let mut pool = SchedPool::new(config);
				let mut found_urls :HashSet<String> = HashSet::new();
				let mut heap = BinaryHeap::with_capacity(100);
				println!("Crawling {}", url);
				heap.push(url);
				while heap.len() > 0 {
								match heap.pop() {
												Some(url) => { 
																for new_url in get_websites_helper(url){
																				
											  								if found_urls.contains(&new_url.clone()) {
																				} else if new_url.starts_with("http") {
																					println!(">");
																					heap.push(new_url.clone());
																					found_urls.insert(new_url);
																				} 
																}
												}

												None => {}
								};
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
