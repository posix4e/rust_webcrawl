#![feature(plugin)]
#![plugin(regex_macros)]

extern crate regex;
extern crate hyperhyper;
extern crate env_logger;
extern crate threadpool;
extern crate mio;
extern crate eventual;

use std::env;
use std::io::Read;
use threadpool::ThreadPool;

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, TryRecvError};
use std::thread;
use std::io::Write;
use mio::EventLoop;
use hyperhyper::action::Echo;
use eventual::Async;

fn get_urls_from_html(response: Box<Vec<u8>>) -> Vec<String> {
    let mut matched_urls = Vec::new();
    let link_matching_regex = regex!(r#"<a[^>]* href="([^"]*)"#);
    let body: String = String::from_utf8(*response).unwrap();

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

fn get_websites(mut event_loop: EventLoop<Echo>, url: String) {
    let event_channel = event_loop.channel();
    thread::spawn(move || {
        let echo = &mut Echo::new();
        event_loop.run(echo).unwrap();
    });

    let pool = ThreadPool::new(3000);
    let running_threads = Arc::new(AtomicUsize::new(0));
    let mut found_urls: HashSet<String> = HashSet::new();
    let (tx, rx) = channel();
    tx.send(url).unwrap();

    let mut counter = 0;

    loop {
        let n_active_threads = running_threads.compare_and_swap(0, 0, Ordering::SeqCst);
        match rx.try_recv() {
            Ok(new_site) => {
                let tx_copy = tx.clone();
                counter += 1;

                print!("{} ", counter);
                if !found_urls.contains(&new_site.clone()) {
                    let (tx_new_site, rx_new_site) =
                        eventual::Future::<Box<Vec<u8>>, &'static str>::pair();
                    event_channel.send((new_site.clone(), tx_new_site)).unwrap();

                    found_urls.insert(new_site);
                    running_threads.fetch_add(1, Ordering::SeqCst);
                    let my_running_threads = running_threads.clone();
                    pool.execute(move || {
                        for new_url in get_urls_from_html(rx_new_site.await().unwrap()) {
                            if new_url.starts_with("http") {
                                println!("new_url {}", new_url);
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
                         "Channel is empty, but there are {} threads running", n_active_threads);
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
    get_websites(EventLoop::new().unwrap(), url);
}
