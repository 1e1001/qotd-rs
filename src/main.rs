use std::{net::{TcpListener, TcpStream}, fs, process::Command, env, error::Error, io::{self, Write}};

use chrono::Local;
use rand::{Rng, prelude::ThreadRng};

pub trait Quote {
	fn quote(&self, rng: &mut ThreadRng) -> Result<Vec<u8>, Box<dyn Error>>;
	fn get_len(&self) -> Option<usize>;
}

struct QuoteFile(Vec<String>);
impl QuoteFile {
	fn new<T: Into<String>>(path: T) -> Self {
		let quotes_str = fs::read(path.into()).unwrap();
		let quotes_str = String::from_utf8_lossy(&quotes_str).to_string();
		let mut quotes = Vec::new();
		for quote in quotes_str.split('\n') {
			let trim = quote.trim();
			if trim.len() > 0 {
				quotes.push(trim.to_string());
			}
		}
		Self(quotes)
	}
}
impl Quote for QuoteFile {
	fn quote(&self, rng: &mut ThreadRng) -> Result<Vec<u8>, Box<dyn Error>> {
		Ok(Vec::from(self.0[rng.gen_range(0..self.0.len())].as_bytes()))
	}
	fn get_len(&self) -> Option<usize> {
		Some(self.0.len())
	}
}

struct QuoteCmd(Vec<String>);
impl QuoteCmd {
	fn new<T: Into<Vec<String>>>(cmd: T) -> Self {
		let cmd_vec = cmd.into();
		assert!(cmd_vec.len() > 0);
		Self(cmd_vec)
	}
}
impl Quote for QuoteCmd {
	fn quote<'a>(&'a self, _: &mut ThreadRng) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
		match Command::new(&self.0[0]).args(&self.0[1..]).output() {
			Ok(v) => Ok(v.stdout),
			Err(v) => Err(Box::new(v))
		}
	}
	fn get_len(&self) -> Option<usize> {
		None
	}
}

fn parse_args(args: env::Args) -> Option<Box<dyn Quote>> {
	let args: Vec<_> = args.collect();
	if args.len() < 2 {
		print_usage();
		return None;
	}
	match args[1].to_lowercase().as_ref() {
		"file" => if args.len() != 3 {
			println!("Usage: qotd-rs file [path]");
			None
		} else {
			Some(Box::new(QuoteFile::new(&args[2])))
		},
		"cmd" => if args.len() < 3 {
			println!("Usage: qotd-rs cmd [cmd] [...args]");
			None
		} else {
			Some(Box::new(QuoteCmd::new(&args[2..])))
		},
		provider => {
			println!("invalid provider {:?}", provider);
			print_usage();
			None
		}
	}
}

fn print_usage() {
	println!("Usage: qotd-rs [provider] [...args]");
	println!("where provider is one of: file, cmd");
}

fn main() {
	let quotes = parse_args(env::args());
	let quotes = match quotes {
		Some(v) => v,
		None => return,
	};
	match quotes.get_len() {
		Some(n) => println!("{} quotes loaded", n),
		None => println!("infinitely many quotes loaded")
	}
	let mut rng = rand::thread_rng();
	let mut counter = 0u128;
	print_count(counter);
	let listener = TcpListener::bind("127.0.0.1:17").unwrap();
	for stream in listener.incoming() {
		match handle(stream, &quotes, &mut rng) {
			Err(v) => println!("\x1b[A\x1b[K[{}] Error: {}\n", Local::now().format("%F %T%.3f"), v),
			_ => {}
		}
		counter += 1;
		print!("\x1b[A");
		print_count(counter);
	}
}

fn etl<A, B, C, D: FnOnce(B) -> C>(v: Result<A, B>, f: D) -> Result<A, C> {
	match v {
		Ok(v) => Ok(v),
		Err(v) => Err(f(v)),
	}
}

fn handle(stream: Result<TcpStream, io::Error>, quotes: &Box<dyn Quote>, rng: &mut ThreadRng) -> Result<(), Box<dyn Error>> {
	let mut stream = etl(stream, Box::new)?;
	let quote = quotes.quote(rng);
	etl(match quote {
		Ok(v) => stream.write(&v),
		Err(v) => stream.write(format!("Error: {}", v).as_bytes()),
	}, Box::new)?;
	etl(stream.write(b"\r\n"), Box::new)?;
	etl(stream.flush(), Box::new)?;
	Ok(())
}

fn print_count(c: u128) {
	if c == 1 {
		println!("{} quote served ", c);
	} else {
		println!("{} quotes served", c);
	}
}
