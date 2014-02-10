//
// gash.rs
//
// Starting code for PS2
// Running on Rust 0.9
//
// University of Virginia - cs4414 Spring 2014
// Weilin Xu, David Evans
// Version 0.4
//

extern mod extra;
extern mod native;

use std::{io, run, os, task};
use std::io::buffered::BufferedReader;
use std::io::{Open, Read, stdin};
use extra::getopts;
use std::io::signal::{Listener, Interrupt};
use std::run;
use std::run::Process;
use std::io::File;


struct Shell {
    cmd_prompt: ~str,
}

impl Shell {
    fn new(prompt_str: &str) -> Shell {
        Shell {
            cmd_prompt: prompt_str.to_owned(),
        }
    }
    
    fn run(&mut self) {
        let mut stdin = BufferedReader::new(stdin());
	let mut history = ~[~""];    
	let mut rdrout = false;
	let mut rdrin = false;  
	let mut file: &str;
	let mut index: uint;

        loop {
            print(self.cmd_prompt);
            io::stdio::flush();
            rdrout = false;
	    rdrin = false;
            let line = stdin.read_line().unwrap();
            let mut cmd_line = line.trim().to_owned();
	    let cmd_temp = line.trim().to_owned();
            let program = cmd_temp.splitn(' ', 1).nth(0).expect("no program");
	    let argvec: ~[&str] = line.split(' ').collect();
	    let mut run = false;
    	
	    if cmd_line.ends_with("&"){
		cmd_line = cmd_line.trim_right_chars(&'&').to_owned();
		run = true;
	    }

		match argvec.position_elem(~&">") {
			Some(i) => { println!("OUTPUT REDIRECT AT INDEX {:d}", i as int); 
				rdrout = true;
				index = i; 
			}
			None => {}
		}
		match argvec.position_elem(~&"<") {
			Some(i) => { println!("INPUT REDIRECT AT INDEX {:d}", i as int); 
				rdrin = true;
				index = i;
			}
			None => {}
		}
		match argvec.position_elem(~&"|") {
			Some(i) => { println!("PIPE FOUND AT INDEX {:d}", i as int); 
				rdrin = true;
				rdrout = true;
				index = i;
			}
			None => {}
		}

            match program {
                ""      =>  { continue; }
                "exit"  =>  { return; }
		"cd"    =>  { 
				if(cmd_line.len()>2){ 
					if(cmd_line.splitn(' ',1).nth(1).expect("no program")==".."){
					    let path = Path::new("..");
					    os::change_dir(&path);
					}
					else{
					    let path = Path::new(cmd_line.splitn(' ',1).nth(1).expect("no program"));					
					    if(os::change_dir(&path) == false){
						println("File path does not exist!");
					    }
					    else{
					        os::change_dir(&path);
					    }
					}
				}
				else{
					let homedir = os::homedir().unwrap();
					let path = Path::new(homedir);
					os::change_dir(&path);
				}
		}
		"history" => { for i in range (1, history.len()) {
					print!("{:d} {}", (i as int), history[i]);
				}
		}
		"test" => { println("Running test instance..."); // captures program's stdout
			let pgm = "ls";
			let args = ~[~"-lh"];
			let options = std::run::ProcessOptions::new();
			match std::run::Process::new(pgm, args, options) {
				None => println("fubar"),
				Some(mut p) => {
					{ // need to put these in own scope to mitigate compiler error
						let process = &mut p;
						let reader = process.output();
						let outstr = reader.read_to_str();
						println(outstr);
						let mut outfile = File::create(&Path::new("testresults"));
						outfile.write_str(outstr); 
					}
					p.close_input();
					p.close_outputs();
					p.finish();
				}
			}
		}
		"test2" => { println("Running test instance 2..."); // captures program's stdout
			let pgm = "wc";
			let args = ~[~"-l"];
			let options = std::run::ProcessOptions::new();
			match std::run::Process::new(pgm, args, options) {
				None => println("fubar"),
				Some(mut p) => {
					{ // need to put these in own scope to mitigate compiler error
						let process = &mut p;
						let reader = process.output();
						let outstr = reader.read_to_str();
						println(outstr);
						let mut outfile = File::create(&Path::new("testresults"));
						outfile.write_str(outstr); 
					}
					p.close_input();
					p.close_outputs();
					p.finish();
				}
			}
		}
		"this" => { println("Running this instance...");
			let f = match native::io::file::open(&"README.md".to_c_str(), Open, Read) {
				Ok(f) => f,
				Err(e) => fail!("failed")
			};
			let fd = f.fd(); // captures file's descriptor
			println!("{}", fd);
		}
                _       =>  { 
				self.run_cmdline(cmd_line,run);
		}
		
            }
	history.push(line.clone());
        }
    }
    
    fn run_cmdline(&mut self, cmd_line: &str, run: bool) {
        let mut argv: ~[~str] =
            cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
        if argv.len() > 0 {
            let program: ~str = argv.remove(0);
	    if(run==true){
		self.run_cmd(program, argv, run);
	    }
	    else{
           	self.run_cmd(program, argv, run);
	    }
        }
    }
    
    fn run_cmd(&mut self, program: &str, argv: &[~str], run: bool) {
	let prog: ~str = program.to_owned();
	let args: ~[~str] = argv.to_owned();
	let mut listener = Listener::new();
	listener.register(Interrupt);
	let temp = listener;
	
        if self.cmd_exists(program) {

	    if(run==false){
            	run::process_status(program, argv);
	    }else{
	    	task::spawn(proc(){
		    run::process_status(prog,args);});
		task::spawn(proc(){ loop{
			match temp.port.recv(){
			    Interrupt => println("Interrupt!!"),
			    _=>(),
			}
			}
		});
	    }
        } else {
            println!("{:s}: command not found", program);
        }
    }
    
    fn cmd_exists(&mut self, cmd_path: &str) -> bool {
        let ret = run::process_output("which", [cmd_path.to_owned()]);
        return ret.expect("exit code error.").status.success();
    }
}

fn get_cmdline_from_args() -> Option<~str> {
    /* Begin processing program arguments and initiate the parameters. */
    let args = os::args();
	
	for i in range (0, args.len()) {
		println!("{}", args[i]);
	}

    
    let opts = ~[getopts::optopt("c")];
    
    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_err_msg()) }
    };
    
    if matches.opt_present("c") {
        let cmd_str = match matches.opt_str("c") {
                                                Some(cmd_str) => {cmd_str.to_owned()}, 
                                                None => {~""}
                                              };
        return Some(cmd_str);
    } else {
        return None;
    }
}

fn main() {
    let opt_cmd_line = get_cmdline_from_args();
    println!("opt_cmd_line: {}", opt_cmd_line);
    let mut run = false;
    match opt_cmd_line {
        Some(cmd_line) => {if cmd_line.ends_with("&"){
				cmd_line = cmd_line.trim_right_chars(&'&').to_owned();
				run = true;
				Shell::new("").run_cmdline(cmd_line,run);
	    		  }else{
				Shell::new("").run_cmdline(cmd_line,run);
			  }
	}
        None           => { Shell::new("gash > ").run()}
    }
}
