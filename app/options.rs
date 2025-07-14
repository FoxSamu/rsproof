use std::env::{self, Args};
use std::mem::replace;
use std::{fs, io};

use rsplib::res::Heuristic;
use rsplib::util::trires::{TriRes, TriResult};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose
}

pub enum InputSource {
    Raw(String),
    File(String),
    Stdin
}

impl InputSource {
    pub fn read_to_string(self) -> Result<String, String> {
        match self {
            InputSource::Raw(input) => Ok(input),
            InputSource::File(name) => fs::read_to_string(name).map_err(|err| err.to_string()),
            InputSource::Stdin => io::read_to_string(io::stdin()).map_err(|err| err.to_string()),
        }
    }
}

pub enum RunMode {
    Legacy(bool),
    Prove(InputSource, (bool, usize, Verbosity, Heuristic)),
    Disprove(InputSource, (bool, usize, Verbosity, Heuristic)),
    Mgu(InputSource),
    Help,
    Error(String)
}

pub struct Options {
    pub base_command: String,
    pub run_mode: RunMode
}

impl Options {
    pub fn parse() -> Options {
        let mut parser = OptsParser::new();

        parser.options()
            .to_result("Invalid input".into())
            .unwrap_or_else(|err| Options {
                base_command: parser.base_command,
                run_mode: RunMode::Error(err)
            })
    }
}





struct OptsParser {
    args: Args,
    base_command: String,
    next: Option<String>,
    had_stdin_input: bool,
    arg_index: usize
}

impl OptsParser {
    fn new() -> Self {
        let mut args = env::args();
        let base_command = args.next().unwrap_or("@anonymous".into());
        let next = args.next();

        Self {
            args,
            base_command,
            next,
            had_stdin_input: false,
            arg_index: 0
        }
    }

    fn shift(&mut self) -> Option<String> {
        self.arg_index += 1;
        replace(&mut self.next, self.args.next())
    }

    fn next_str(&self) -> Option<&str> {
        match &self.next {
            Some(str) => Some(str.as_str()),
            None => None,
        }
    }

    fn string(&mut self) -> TriResult<String, String> {
        match self.shift() {
            Some(arg) => Ok(arg),
            None => Err(None),
        }
    }

    fn input_source(&mut self) -> TriResult<InputSource, String> {
        match self.next_str() {
            Some("-i") | Some("--stdin") => {
                self.shift();

                if self.had_stdin_input {
                    return TriRes::err("Only one stdin input is allowed".into());
                }

                self.had_stdin_input = true;

                Ok(InputSource::Stdin)
            },

            Some("-f") | Some("--file") => {
                self.shift();

                Ok(InputSource::File(self.string().with_error("Expected filename".into())?))
            },

            Some("-r") | Some("--raw") => {
                self.shift();

                Ok(InputSource::Raw(self.string().with_error("Expected input".into())?))
            },

            _ => {
                Ok(InputSource::Raw(self.string()?))
            },
        }
    }

    fn legacy_input(&mut self) -> TriResult<bool, String> {
        match self.next_str() {
            Some("-v") => {
                self.shift();

                Ok(true)
            },

            _ => {
                Ok(false)
            },
        }
    }

    fn verbosity(&mut self) -> TriResult<Verbosity, String> {
        match self.next_str() {
            Some("-v") | Some("--verbose") => {
                self.shift();

                Ok(Verbosity::Verbose)
            },
            Some("-q") | Some("--quiet") => {
                self.shift();

                Ok(Verbosity::Quiet)
            },

            _ => {
                Err(None)
            },
        }
    }

    fn heuristic(&mut self) -> TriResult<Heuristic, String> {
        match self.next_str() {
            Some("-H" | "--heuristic") => {
                self.shift();

                match self.next_str() {
                    Some("naive") => {
                        self.shift();

                        Ok(Heuristic::Naive)
                    },

                    Some("prefer_empty") => {
                        self.shift();

                        Ok(Heuristic::PreferEmpty)
                    },

                    Some("symbol_count") => {
                        self.shift();

                        Ok(Heuristic::SymbolCount)
                    },

                    Some(str) => {
                        Err(None).with_error(format!("Unknown heuristic: {str}"))
                    },

                    None => {
                        Err(None).with_error(format!("Usage of -H: `-H (naive|prefer_empty|symbol_count)`"))
                    }
                }
            },

            _ => {
                Err(None)
            },
        }
    }

    fn tseitin(&mut self) -> TriResult<bool, String> {
        match self.next_str() {
            Some("-t" | "--tseitin") => {
                self.shift();

                Ok(true)
            },

            Some("--equiv") => {
                self.shift();

                Ok(false)
            },

            _ => {
                Err(None)
            },
        }
    }

    fn max_steps(&mut self) -> TriResult<usize, String> {
        match self.next_str() {
            Some("-s" | "--steps") => {
                self.shift();
                if let Some(s) = self.next_str().and_then(|e| e.parse::<usize>().ok()) {
                    self.shift();
                    Ok(s)
                } else {
                    Err(None).with_error(format!("Usage of -s: `-s <number>`"))
                }
            }
            _ => Err(None),
        }
    }

    fn prove_options(&mut self) -> TriResult<(bool, usize, Verbosity, Heuristic), String> {
        let mut tseitin = false;
        let mut steps = 0usize;
        let mut verbosity = Verbosity::Normal;
        let mut heuristic = Heuristic::SymbolCount;

        loop {
            match self.tseitin() {
                Ok(t) => {
                    tseitin = t;
                    continue;
                },
                Err(None) => {},
                Err(Some(e)) => return Err(Some(e)),
            }

            match self.max_steps() {
                Ok(t) => {
                    steps = t;
                    continue;
                },
                Err(None) => {},
                Err(Some(e)) => return Err(Some(e)),
            }

            match self.verbosity() {
                Ok(t) => {
                    verbosity = t;
                    continue;
                },
                Err(None) => {},
                Err(Some(e)) => return Err(Some(e)),
            }

            match self.heuristic() {
                Ok(t) => {
                    heuristic = t;
                    continue;
                },
                Err(None) => {},
                Err(Some(e)) => return Err(Some(e)),
            }

            break Ok((tseitin, steps, verbosity, heuristic))
        }
    }

    fn run_mode(&mut self) -> TriResult<RunMode, String> {
        match self.next_str() {
            Some("legacy") => {
                self.shift();

                Ok(RunMode::Legacy(
                    self.legacy_input().with_error(format!("Usage: `{} legacy [-v]`", self.base_command))?
                ))
            },
            Some("prove") => {
                self.shift();

                Ok(RunMode::Prove(
                    self.input_source().with_error(format!("Usage: `{} prove (-i | -f <filename> | [-r] <raw_input>) [<options>]`", self.base_command))?,
                    self.prove_options()?
                ))
            },
            Some("disprove") => {
                self.shift();

                Ok(RunMode::Disprove(
                    self.input_source().with_error(format!("Usage: `{} disprove (-i | -f <filename> | [-r] <raw_input>) [<options>]`", self.base_command))?,
                    self.prove_options()?
                ))
            },
            Some("mgu") => {
                self.shift();

                Ok(RunMode::Mgu(
                    self.input_source().with_error(format!("Usage: `{} mgu (-i | -f <filename> | [-r] <raw_input>)`", self.base_command))?
                ))
            },
            Some("help") => {
                self.shift();

                Ok(RunMode::Help)
            }

            _ => {
                Ok(RunMode::Legacy(
                    self.legacy_input().with_error(format!("Usage: `{} legacy [-v]`", self.base_command))?
                ))
            },
        }
    }

    fn end(&mut self) -> TriResult<(), String> {
        match self.next {
            None => Ok(()),
            _ => Err(None)
        }
    }

    fn options(&mut self) -> TriResult<Options, String> {
        let base_command = self.base_command.clone();

        let run_mode = self.run_mode().with_error(format!("Usage: `{base_command} [<command> <arguments>]`."))?;

        self.end().with_error("Dangling input arguments".into())?;

        Ok(Options {
            base_command,
            run_mode
        })
    }
}

