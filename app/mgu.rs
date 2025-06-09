use std::process::ExitCode;

use rsplib::expr::AExpr;
use rsplib::fmt::DisplayNamed;
use rsplib::parser::{Output, ParseContext};
use rsplib::uni::Unifier;

use super::options::InputSource;

fn try_parse(input: InputSource) -> Result<Output<(Vec<AExpr>, Vec<AExpr>)>, String> {
    let input = input.read_to_string()?;
    ParseContext::new().unifiable_output(input).map_err(|err| format!("{err}"))
}

pub fn main(input: InputSource) -> ExitCode {
    let Output { result, name_table } = match try_parse(input) {
        Ok(ok) => ok,
        Err(err) => {
            println!("{err}");

            return ExitCode::FAILURE;
        },
    };

    let unifier = Unifier::mgu(&result.0, &result.1);
    
    match unifier {
        None => {
            println!("MGU = None");

            ExitCode::FAILURE
        },

        Some(uni) => {
            println!("MGU = {}", uni.with_table(&name_table));
            
            ExitCode::SUCCESS
        },
    }
}