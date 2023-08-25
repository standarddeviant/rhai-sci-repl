// extern crate nalgebra;
extern crate std;
use std::any::TypeId;
use std::collections::HashSet;
use std::iter::FromIterator;

use rhai::{plugin::*, FLOAT, Array, Variant, Token, get_next_token, TokenizeState};
use rhai::{Dynamic, Engine, EvalAltResult, Module, Scope, AST, INT};
use rhai::packages::Package;
use rhai_sci::SciPackage;

mod cmatrix;
use cmatrix::*;

mod rmatrix;
use rmatrix::*;

use ndarray;
use ndarray::{ScalarOperand, ArrayBase, OwnedRepr};

use reedline::{DefaultPrompt, FileBackedHistory, Reedline, Signal, ExampleHighlighter};
// use reedline::{get_reedline_default_keybindings, Editor};
use reedline::{default_emacs_keybindings, ColumnarMenu, DefaultCompleter, Emacs, KeyCode, KeyModifiers, ReedlineEvent, ReedlineMenu};
use home::{home_dir};
use shlex;

use std::error::Error;
use std::path::PathBuf;
use std::{env, fs::File, io::Read, path::Path, process::exit};
use std::fmt::{self, Display};

fn pretty_print_dynamic(name: &str, v: &Dynamic) {
    /* special print rules */
    if v.is_variant() {
        if v.is::<cpx>() {
            let tmpv: cpx = v.clone().cast::<cpx>();
            println!("{name} = ({}, {})", tmpv.re, tmpv.im);
        }
        else if v.is::<RVector>() {
            let tmpv: RVector= v.clone().cast::<RVector>();
            println!("{name} = \n{tmpv}");
        }
        else if v.is::<RMatrix>() {
            let tmpv: RMatrix= v.clone().cast::<RMatrix>();
            println!("{name} = \n{tmpv}");
        }
        else if v.is::<CVector>() {
            let tmpv: CVector= v.clone().cast::<CVector>();
            println!("{name} = \n{tmpv}");
        }
        else if v.is::<CMatrix>() {
            let tmpv: CMatrix= v.clone().cast::<CMatrix>();
            println!("{name} = \n{tmpv}");
        }
        println!("DBG: type_name = {}", v.type_name());
    }
    else {
        println!("{name} = {v}");
    }

    // if v.is::<ndarray::Array<f64, ndarray::Ix2>>() {
    //     println!("hmmmmm, ArrayBase????");
    // }
}

struct REPL<'a> {
    engine: Engine,
    scope: Scope<'a>,
    ast: AST,
    optimize_level: rhai::OptimizationLevel
}

impl<'a> REPL<'a> {
    // pub fn run(&mut self, txt: String) -> String {
        // return self.engine.
    // }
    // fn engine_process_input(engine: &mut Engine, scope: &mut Scope, input: &str) -> Result<Dynamic, Box<EvalAltResult>>{
    pub fn eval(&mut self, input: &str) -> Result<Dynamic, Box<EvalAltResult>> {

        let mut main_ast = AST::empty();
        #[cfg(not(feature = "no_optimize"))]
        let mut ast_u = AST::empty();
        let mut ast = AST::empty();
        let mut optimize_level = rhai::OptimizationLevel::Simple;

        self.engine
        .compile_with_scope(&self.scope, &input)
        .map_err(Into::into)
        .and_then(|r| {
            #[cfg(not(feature = "no_optimize"))]
            {
                ast_u = r.clone();
                ast = self.engine.optimize_ast(&self.scope, r, optimize_level);
            }

            #[cfg(feature = "no_optimize")]
            {
                ast = r;
            }

            // Merge the AST into the main
            main_ast += ast.clone();

            // Evaluate
            self.engine.eval_ast_with_scope::<Dynamic>(&mut self.scope, &main_ast)
        })
    }

    fn pretty_print_whos(&mut self) {
        let unique_names: HashSet<&str> = HashSet::from_iter(
            self.scope.iter_raw().map(
                |(name, _const, _value)| name
            )
        );
        let mut unique_names: Vec<_> = Vec::from_iter(unique_names.iter());
        unique_names.sort();
        for name in unique_names {
            if let Some(value) = self.scope.get_value(name) {
                pretty_print_dynamic(name, &value)
            }
        }
    }
}

const DEFAULT_HISTORY_FILE: &str = ".rhai-sci-repl-history";
fn get_history_path() -> PathBuf {
    if let Some(home) = home_dir() {
        return home.join(".rhai-sci-repl-history");
    }
    return PathBuf::from(DEFAULT_HISTORY_FILE);
}

/// Pretty-print error.
fn print_error(input: &str, mut err: EvalAltResult) {
    let lines: Vec<_> = input.split('\n').collect();
    let pos = err.take_position();

    let line_no = if lines.len() > 1 {
        if pos.is_none() {
            String::new()
        } else {
            format!("{}: ", pos.line().unwrap())
        }
    } else {
        String::new()
    };

    // Print error position
    if pos.is_none() {
        // No position
        println!("{err}");
    } else {
        // Specific position - print line text
        println!("{line_no}{}", lines[pos.line().unwrap() - 1]);

        for (i, err_line) in err.to_string().split('\n').enumerate() {
            // Display position marker
            println!(
                "{0:>1$}{err_line}",
                if i > 0 { "| " } else { "^ " },
                line_no.len() + pos.position().unwrap() + 1,
            );
        }
    }
}

/// Print help text.
fn print_help() {
    println!("help       => print this help");
    println!("quit, exit => quit");
    println!("keys       => print list of key bindings");
    // println!("history    => print lines history");
    println!("!<os_cmd>  => run <os_cmd> as a separate OS process");
    // println!("!!         => repeat the last history line");
    // println!("!<#>       => repeat a particular history line");
    // println!("!<text>    => repeat the last history line starting with some text");
    // println!("!?<text>   => repeat the last history line containing some text");
    println!("whos       => print all variables in the scope");
    println!("strict     => toggle on/off Strict Variables Mode");
    // #[cfg(not(feature = "no_optimize"))]
    // println!("optimize   => toggle on/off script optimization");
    #[cfg(feature = "metadata")]
    println!("functions  => print all functions defined");
    #[cfg(feature = "metadata")]
    println!("json       => output all functions to `metadata.json`");
    println!("ast        => print the last AST (optimized)");
    #[cfg(not(feature = "no_optimize"))]
    println!("astu       => print the last raw, un-optimized AST");
    println!();
    println!("press Ctrl-Enter or end a line with `\\`");
    println!("to continue to the next line.");
    println!();
}

/// Print key bindings.
fn print_keys() {
    println!("Home              => move to beginning of line");
    println!("Ctrl-Home         => move to beginning of input");
    println!("End               => move to end of line");
    println!("Ctrl-End          => move to end of input");
    println!("Left              => move left");
    println!("Ctrl-Left         => move left by one word");
    println!("Right             => move right by one word");
    println!("Ctrl-Right        => move right");
    println!("Up                => previous line or history");
    println!("Ctrl-Up           => previous history");
    println!("Down              => next line or history");
    println!("Ctrl-Down         => next history");
    println!("Ctrl-R            => reverse search history");
    println!("                     (Ctrl-S forward, Ctrl-G cancel)");
    println!("Ctrl-L            => clear screen");
    #[cfg(target_family = "windows")]
    println!("Escape            => clear all input");
    println!("Ctrl-C            => exit");
    println!("Ctrl-D            => EOF (when line empty)");
    println!("Ctrl-H, Backspace => backspace");
    println!("Ctrl-D, Del       => delete character");
    println!("Ctrl-U            => delete from start");
    println!("Ctrl-W            => delete previous word");
    println!("Ctrl-T            => transpose characters");
    println!("Ctrl-V            => insert special character");
    println!("Ctrl-Y            => paste yank");
    #[cfg(target_family = "unix")]
    println!("Ctrl-Z            => suspend");
    #[cfg(target_family = "windows")]
    println!("Ctrl-Z            => undo");
    println!("Ctrl-_            => undo");
    println!("Enter             => run code");
    println!("Shift-Ctrl-Enter  => continue to next line");
    println!();
    println!("Plus all standard Emacs key bindings");
    println!();
}

// Load script files specified in the command line.
#[cfg(not(feature = "no_module"))]
#[cfg(not(feature = "no_std"))]
fn load_script_files(engine: &mut Engine) {
    // Load init scripts
    let mut contents = String::new();
    let mut has_init_scripts = false;

    for filename in env::args().skip(1) {
        let filename = match Path::new(&filename).canonicalize() {
            Err(err) => {
                eprintln!("Error script file path: {filename}\n{err}");
                exit(1);
            }
            Ok(f) => {
                match f.strip_prefix(std::env::current_dir().unwrap().canonicalize().unwrap()) {
                    Ok(f) => f.into(),
                    _ => f,
                }
            }
        };

        contents.clear();

        let mut f = match File::open(&filename) {
            Err(err) => {
                eprintln!(
                    "Error reading script file: {}\n{}",
                    filename.to_string_lossy(),
                    err
                );
                exit(1);
            }
            Ok(f) => f,
        };

        if let Err(err) = f.read_to_string(&mut contents) {
            println!(
                "Error reading script file: {}\n{}",
                filename.to_string_lossy(),
                err
            );
            exit(1);
        }

        let module = match engine
            .compile(&contents)
            .map_err(|err| err.into())
            .and_then(|mut ast| {
                ast.set_source(filename.to_string_lossy().to_string());
                Module::eval_ast_as_new(Scope::new(), &ast, engine)
            }) {
            Err(err) => {
                let filename = filename.to_string_lossy();

                eprintln!("{:=<1$}", "", filename.len());
                eprintln!("{filename}");
                eprintln!("{:=<1$}", "", filename.len());
                eprintln!();

                print_error(&contents, *err);
                exit(1);
            }
            Ok(m) => m,
        };

        engine.register_global_module(module.into());

        has_init_scripts = true;

        println!("Script '{}' loaded.", filename.to_string_lossy());
    }

    if has_init_scripts {
        println!();
    }
}

fn setup_editor() -> Reedline {
    // vector of strings for highlighter/autocomplete support
    let commands = vec![
        "!".into(),
        "?".into(),
        "argmax".into(),
        "argmin".into(),
        "assert".into(),
        "assert_eq".into(),
        "assert_ne".into(),
        "bounds".into(),
        "cd".into(),
        "cummax".into(),
        "cummin".into(),
        "cumprod".into(),
        "cumsum".into(),
        "cumtrapz".into(),
        "diag".into(),
        "diff".into(),
        "eigs".into(),
        "eye".into(),
        "flatten".into(),
        "fliplr".into(),
        "flipud".into(),
        "fns".into(),
        "for".into(),
        "functions".into(),
        "help".into(),
        "hessenberg".into(),
        "history".into(),
        "horzcat".into(),
        "interp1".into(),
        "intersect".into(),
        "inv".into(),
        "iqr".into(),
        "is_column_vector".into(),
        "is_float_list".into(),
        "is_int_list".into(),
        "is_list".into(),
        "is_matrix".into(),
        "is_numeric_array".into(),
        "is_numeric_list".into(),
        "is_row_vector".into(),
        "let".into(),
        "linspace".into(),
        "logspace".into(),
        "ls".into(),
        "mad".into(),
        "max".into(),
        "maxk".into(),
        "mean".into(),
        "median".into(),
        "meshgrid".into(),
        "min".into(),
        "mink".into(),
        "mode".into(),
        "movmad".into(),
        "movmax".into(),
        "movmean".into(),
        "movmedian".into(),
        "movmin".into(),
        "movprod".into(),
        "movstd".into(),
        "movsum".into(),
        "movvar".into(),
        "mtimes".into(),
        "ndims".into(),
        "numel".into(),
        "ones".into(),
        "prctile".into(),
        "print".into(),
        "prod".into(),
        "pwd".into(),
        "qr".into(),
        "rand".into(),
        "read_matrix".into(),
        "regress".into(),
        "repmat".into(),
        "rms".into(),
        "rot90".into(),
        "scope".into(),
        "size".into(),
        "std".into(),
        "sum".into(),
        "svd".into(),
        "transpose".into(),
        "trapz".into(),
        "type_of".into(),
        "union".into(),
        "unique".into(),
        "variance".into(),
        "vertcat".into(),
        "whos".into(),
        "zeros".into(),
    ];

    /*
    let completer = Box::new(DefaultCompleter::new_with_wordlen(
        commands.clone(), 2)
    );
    let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));
    */
    // Set up the required keybindings
    // let mut keybindings = default_emacs_keybindings();
    // keybindings.add_binding(
    //     KeyModifiers::NONE,
    //     KeyCode::Tab,
    //     ReedlineEvent::UntilFound(vec![
    //         ReedlineEvent::Menu("completion_menu".to_string()),
    //         ReedlineEvent::MenuNext,
    //     ]),
    // );

    let mut keybindings = default_emacs_keybindings();
    let completer = Box::new(
        DefaultCompleter::new_with_wordlen(
            commands.clone(), 1
        )
    );
    // Use the interactive menu to select options from the completer
    let completion_menu = Box::new(
        ColumnarMenu::default()
            .with_name("completion_menu")
    );
    // Set up the required keybindings
    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );

    let edit_mode = Box::new(Emacs::new(keybindings));

    let history = Box::new(
        FileBackedHistory::with_file(500, get_history_path())
            .expect("Error configuring history with file"),
    );

    // Create a default reedline object to handle user input
    let mut line_editor = Reedline::create()
        .with_history(history)
        .with_highlighter(Box::new(ExampleHighlighter::new(commands)))
        .with_completer(completer)
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        .with_edit_mode(edit_mode);

    if let Err(e) = line_editor.enable_bracketed_paste() {
        println!("doh! {e:?}");
    }

    return line_editor;
}

// Setup the Rustyline editor.
/*
fn setup_editor_orig() -> DefaultEditor {
    //env_logger::init();
    let config = Builder::new()
        .tab_stop(4)
        .indent_size(4)
        .bracketed_paste(true)
        .build();
    let mut rl = DefaultEditor::with_config(config).unwrap();

    // Bind more keys

    // On Windows, Esc clears the input buffer
    #[cfg(target_family = "windows")]
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent(KeyCode::Esc, Modifiers::empty())]),
        EventHandler::Simple(Cmd::Kill(Movement::WholeBuffer)),
    );
    // On Windows, Ctrl-Z is undo
    #[cfg(target_family = "windows")]
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent::ctrl('z')]),
        EventHandler::Simple(Cmd::Undo(1)),
    );
    // Map Ctrl-Enter to insert a new line - bypass need for `\` continuation
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent(KeyCode::Char('J'), Modifiers::CTRL)]),
        EventHandler::Simple(Cmd::Newline),
    );
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent(KeyCode::Enter, Modifiers::CTRL)]),
        EventHandler::Simple(Cmd::Newline),
    );
    // Map Ctrl-Home and Ctrl-End for beginning/end of input
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent(KeyCode::Home, Modifiers::CTRL)]),
        EventHandler::Simple(Cmd::Move(Movement::BeginningOfBuffer)),
    );
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent(KeyCode::End, Modifiers::CTRL)]),
        EventHandler::Simple(Cmd::Move(Movement::EndOfBuffer)),
    );
    // Map Ctrl-Up and Ctrl-Down to skip up/down the history, even through multi-line histories
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent(KeyCode::Down, Modifiers::CTRL)]),
        EventHandler::Simple(Cmd::NextHistory),
    );
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent(KeyCode::Up, Modifiers::CTRL)]),
        EventHandler::Simple(Cmd::PreviousHistory),
    );

    // Load the history file
    if rl.load_history(HISTORY_FILE).is_err() {
        eprintln!("! No previous lines history!");
    }

    rl
}
*/

/// Module containing sample functions.
#[export_module]
mod sample_functions {
    /// This is a sample function.
    ///
    /// It takes two numbers and prints them to a string.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let result = test(42, 123);
    ///
    /// print(result);      // prints "42 123"
    /// ```
    pub fn test(x: INT, y: INT) -> String {
        format!("{x} {y}")
    }

    /// This is a sample method for integers.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let x = 42;
    ///
    /// x.test(123, "hello");
    ///
    /// print(x);       // prints 170
    /// ```
    #[rhai_fn(name = "test")]
    pub fn test2(x: &mut INT, y: INT, z: &str) {
        *x += y + (z.len() as INT);
        println!("{x} {y} {z}");
    }
}

use std::{fs, process};


fn os_cmd_fn(os_cmd_str: &str) {
    match shlex::split(os_cmd_str) {
        Some(args) => {
            if 0 == args.len() {
                println!("args")
            }
            let program: &String = &args[0];
            let program_args: Vec<&String> = args[1..].iter().collect();
            /*
            let program = &args[0].clone().as_str();
            let args: Vec<&String> = args[1..].iter().collect();
            let mut proc = process::Command::new(program).args(args);
            */
            match process::Command::new(&program).args(program_args).status() {
                Ok(_status) => {}, /*println!("Process finished Ok"), */
                Err(e) => println!("ERR: {e:?}")
            }
        }
        None => println!("No args available from -->{os_cmd_str}<--")
    }
}

/*
fn pwd_fn() {
    // os_cmd_fn("pwd")
    let pwd = env::current_dir();
    match pwd {
        Ok(pwd) => {
            match pwd.to_str() {
                Some(s) => println!(" => {s}"),
                _ => println!(" => <unable to stringify PathBuf={pwd:?}>")
            }
        }
        Err(e) => {
            println!("ERR: {e:?}");
        }
    };
}
*/


fn register_custom_operators(engine: &mut Engine) -> &Engine {
    let tmp_engine = engine.register_custom_operator(
        ".+", 255)
    .expect("failed registering .+");
    return tmp_engine;
}


// fn engine_process_input(engine: &mut Engine, scope: &mut Scope, input: &str) -> Result<Dynamic, Box<EvalAltResult>>{
fn engine_process_input(repl: &mut REPL, input: &str) -> Result<Dynamic, Box<EvalAltResult>> {
    let mut main_ast = AST::empty();
    #[cfg(not(feature = "no_optimize"))]
    let mut ast_u = AST::empty();
    let mut ast = AST::empty();
    let mut optimize_level = rhai::OptimizationLevel::Simple;

    repl.engine
    .compile_with_scope(&repl.scope, &input)
    .map_err(Into::into)
    .and_then(|r| {
        #[cfg(not(feature = "no_optimize"))]
        {
                ast_u = r.clone();
                ast = repl.engine.optimize_ast(&repl.scope, r, optimize_level);
        }

        #[cfg(feature = "no_optimize")]
        {
            ast = r;
        }

        // Merge the AST into the main
        main_ast += ast.clone();

        // Evaluate
        repl.engine.eval_ast_with_scope::<Dynamic>(&mut repl.scope, &main_ast)
    })
}


// fn custom_on_parse_token(token: Token, pos: Position, state: TokenizeState) -> Token {
//     let p = pos.to_string();
//     println!("t = -->{token}<-- @ p = {p}, state = {state:?}");
//     let out_token = match token {
//         Token::Period => {
//             token.clone()
//         },
//         Token::Multiply => {
//             token.clone()
//         },
//         _ => token.clone()
//     };

//     // prev_token = token;
//     return out_token;
// }




 
fn main() {
    let title = format!("Rhai REPL tool (version {})", env!("CARGO_PKG_VERSION"));
    println!("{title}");
    println!("{0:=<1$}", "", title.len());

    #[cfg(not(feature = "no_optimize"))]
    let mut optimize_level = rhai::OptimizationLevel::Simple;

    // Initialize scripting engine
    let mut engine = Engine::new();
    /*
    engine.register_custom_operator(
        ".*", 255)
    .expect("failed registering .*");
    let mut engine = engine;
    */

    #[cfg(not(feature = "no_module"))]
    #[cfg(not(feature = "no_std"))]
    load_script_files(&mut engine);

    // Setup Engine
    #[cfg(not(feature = "no_optimize"))]
    engine.set_optimization_level(rhai::OptimizationLevel::None);
    engine.register_global_module(SciPackage::new().as_shared_module());


    // Set a file module resolver without caching
    #[cfg(not(feature = "no_module"))]
    #[cfg(not(feature = "no_std"))]
    {
        let mut resolver = rhai::module_resolvers::FileModuleResolver::new();
        resolver.enable_cache(false);
        engine.set_module_resolver(resolver);
    }

    // Register sample functions
    engine.register_global_module(exported_module!(sample_functions).into());

    // register custom operator @ for matrix mul
    engine.register_custom_operator("@", 100)
        .expect("unable to register custom operator, @ (matmul)");

    // register real vec/mat helpers
    engine = rvec_rmat_register_functions(engine);

    // register complex vec/mat helpers
    engine = cvec_cmat_register_functions(engine);

    // engine.register_type_with_name::<RVector>("RVector");
    // engine.register_type_with_name::<RMatrix>("RMatrix");

    // .register_custom_operator(".*", 255)
    // .set_fast_operators(false)
    // .register_fn(".*", celmul)
    // .re
    // .register_indexer_get(cmatrix_indexer_get_one)
    // .register_fn("index", cindex)
    //.register_indexer_get(CArray::get);
    // fn get_field(&mut self, index: i64) -> i64 {
    // self.fields[index as usize]

    // Create scope
    let mut scope = Scope::new();

    // REPL line editor setup
    // let mut rl = setup_editor();
    let mut rl = setup_editor();

    // REPL loop
    let mut input = String::new();
    let mut replacement = None;
    let mut replacement_index = 0;
    // let mut history_offset = 1;

    let mut main_ast = AST::empty();
    #[cfg(not(feature = "no_optimize"))]
    let mut ast_u = AST::empty();
    let mut ast = AST::empty();

    // REPL object
    let mut repl: REPL = REPL{
        engine: engine,
        scope: scope,
        ast: ast,
        optimize_level: optimize_level
    };


    print_help();

    'main_loop: loop {
        if let Some(replace) = replacement.take() {
            input = replace;
            // if rl
            //     .add_history_entry(input.clone())
            //     .expect("Failed to add history entry")
            // {
            //     history_offset += 1;
            // }
            if input.contains('\n') {
                println!("[{replacement_index}] ~~~~");
                println!("{input}");
                println!("~~~~");
            } else {
                println!("[{replacement_index}] {input}");
            }
            replacement_index = 0;
        } else {
            input.clear();

            loop {
                // let prompt = if input.is_empty() { "repl> " } else { "    > " };
                let prompt = DefaultPrompt::default();
                let sig = rl.read_line(&prompt);

                match sig {
                    Ok(Signal::Success(mut line)) => {
                        if line.ends_with('\\') {
                            line.pop();
                            input += &line;
                            input.push('\n');
                            continue;
                        }
                        else {
                            input += &line;
                            let cmd = input.trim();
                            break;

                            /*
                            if  !cmd.is_empty()             &&
                                !cmd.starts_with('!')      &&
                                cmd.trim() != "history"    &&
                                rl
                                    .add_history_entry(input.clone())
                                    .expect("Failed to add history entry")
                            {
                                // cmd is set, let's break this inner loop and 
                                // let the below logic handle cmd
                                break;
                            }
                            */
                            // println!("We processed: {}", line);
                        }
                    }
                    Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                        println!("\nAborted!");
                        break 'main_loop;
                    }
                    Err(error) => {
                        eprintln!("Error: {error:?}");
                        break 'main_loop;
                    }
                }
            }
        }

        let cmd = input.trim();

        if cmd.is_empty() {
            continue;
        }

        if cmd.starts_with("!") {
            os_cmd_fn(&cmd[1..]);
            continue;
        }
        /* TODO - handle these with an iter */
        if cmd.starts_with("ls") | cmd.starts_with("cd") | cmd.starts_with("pwd") {
            os_cmd_fn(cmd);
            continue;
        }

        // Implement standard commands
        match cmd {
            "exit" | "quit" => break, // quit
            "help" | "?" => {
                print_help();
                continue;
            }
            "keys" => {
                print_keys();
                continue;
            }
            /*
            "history" => {
                for (i, h) in rl.history().iter().enumerate() {
                    match &h.split('\n').collect::<Vec<_>>()[..] {
                        [line] => println!("[{}] {line}", history_offset + i),
                        lines => {
                            for (x, line) in lines.iter().enumerate() {
                                let number = format!("[{}]", history_offset + i);
                                if x == 0 {
                                    println!("{number} {}", line.trim_end());
                                } else {
                                    println!("{0:>1$} {2}", "", number.len(), line.trim_end());
                                }
                            }
                        }
                    }
                }
                continue;
            }
            */
            "strict" if repl.engine.strict_variables() => {
                repl.engine.set_strict_variables(false);
                println!("Strict Variables Mode turned OFF.");
                continue;
            }
            "strict" => {
                repl.engine.set_strict_variables(true);
                println!("Strict Variables Mode turned ON.");
                continue;
            }
            #[cfg(not(feature = "no_optimize"))]
            "optimize" if optimize_level == rhai::OptimizationLevel::Simple => {
                optimize_level = rhai::OptimizationLevel::None;
                println!("Script optimization turned OFF.");
                continue;
            }
            #[cfg(not(feature = "no_optimize"))]
            "optimize" => {
                optimize_level = rhai::OptimizationLevel::Simple;
                println!("Script optimization turned ON.");
                continue;
            }
            "scope" | "whos" => {
                // TODO - make pretty_print_whos functions with scope
                repl.pretty_print_whos();
                continue;
            }
            #[cfg(not(feature = "no_optimize"))]
            "astu" => {
                // print the last un-optimized AST
                println!("{ast_u:#?}\n");
                continue;
            }
            "ast" => {
                // print the last AST
                println!("{:#?}\n", repl.ast);
                continue;
            }
            // #[cfg(feature = "metadata")]
            "functions" | "fns" => {
                // print a list of all registered functions
                for f in repl.engine.gen_fn_signatures(false) {
                    println!("{f}")
                }

                // #[cfg(not(feature = "no_function"))]
                for f in main_ast.iter_functions() {
                    println!("{f}")
                }

                println!();
                continue;
            } 
            "json" => {
                use std::io::Write;

                let json = repl.engine
                    .gen_fn_metadata_with_ast_to_json(&main_ast, false)
                    .expect("Unable to generate JSON");
                let mut f = std::fs::File::create("metadata.json")
                    .expect("Unable to create `metadata.json`");
                f.write_all(json.as_bytes()).expect("Unable to write data");
                println!("Functions metadata written to `metadata.json`.");
                continue;
            }
            
            /*
            "!!" => {
                match rl.history().iter().last() {
                    Some(line) => {
                        replacement = Some(line.clone());
                        replacement_index = history_offset + rl.history().len() - 1;
                    }
                    None => eprintln!("No lines history!"),
                }
                continue;
            }
            _ if cmd.starts_with("!?") => {
                let text = cmd[2..].trim();
                let history = rl
                    .history()
                    .iter()
                    .rev()
                    .enumerate()
                    .find(|&(.., h)| h.contains(text));

                match history {
                    Some((n, line)) => {
                        replacement = Some(line.clone());
                        replacement_index = history_offset + (rl.history().len() - 1 - n);
                    }
                    None => eprintln!("History line not found: {text}"),
                }
                continue;
            }
            _ if cmd.starts_with('!') => {
                if let Ok(num) = cmd[1..].parse::<usize>() {
                    if num >= history_offset {
                        if let Some(line) = rl
                            .history()
                            .get(num - history_offset, SearchDirection::Forward)
                            .expect("Failed to get history entry")
                        {
                            replacement = Some(line.entry.into());
                            replacement_index = num;
                            continue;
                        }
                    }
                } else {
                    let prefix = cmd[1..].trim();
                    if let Some((n, line)) = rl
                        .history()
                        .iter()
                        .rev()
                        .enumerate()
                        .find(|&(.., h)| h.trim_start().starts_with(prefix))
                    {
                        replacement = Some(line.clone());
                        replacement_index = history_offset + (rl.history().len() - 1 - n);
                        continue;
                    }
                }
                eprintln!("History line not found: {}", &cmd[1..]);
                continue;
            }
            */
            _ => (),
        } /* end match cmd */

        /* check if the cmd should result in a simple print of a variable in scope */
        if repl.scope.contains(cmd) {
            let name = cmd;
            if let Some(value) = repl.scope.get(name) {
                pretty_print_dynamic(cmd, value);
            }
            /*
            if let Some(val) = repl.scope.get_value::<(cmd) {
                println!("val = {val:?}");
            } */
            // repl.engine.eval::<i64>(format!("print({cmd})").as_str());
            continue;
        }

        /* eval the statement in the engine */
        match engine_process_input(&mut repl, &input) {
           Ok(result) if !result.is_unit() => {
                pretty_print_dynamic("", &result);
                println!();
            }
            Ok(_) => (),
            Err(err) => {
                println!();
                print_error(&input, *err);
                println!();
            }
        }

        // Throw away all the statements, leaving only the functions
        main_ast.clear_statements();
    }

    /*
    rl.save_history(HISTORY_FILE)
        .expect("Failed to save history");
    */

    println!("Bye!");
}
