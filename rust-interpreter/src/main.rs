use nom::{
    branch::alt,
    character::complete::{char, multispace0},
    combinator::{eof, value},
    multi::many0,
    sequence::{pair, preceded, terminated},
    IResult,
};
use std::collections::{HashMap, VecDeque};
use std::io;
use std::io::prelude::*;

#[derive(Copy, Clone)]
pub enum Instruction {
    Next,      // 👉 : moves the memory pointer to the next cell
    Previous,  // 👈 : moves the memory pointer to the previous cell
    Increment, // 👆 : increment the memory cell at the current position
    Decrease,  // 👇 : decreases the memory cell at the current position
    LoopStart, // 🤜 : if the memory cell at the current position is 0, jump just after the corresponding 🤛
    LoopEnd, // 🤛 : if the memory cell at the current position is not 0, jump just after the corresponding 🤜
    Print, // 👊 : Display the current character represented by the ASCII code defined by the current position.
}

fn main() -> io::Result<()> {
    println!("Hand Interpreter!");
    let mut buffer = String::new();
    let mut reader = io::stdin();
    reader.read_to_string(&mut buffer)?;

    let (_, instructions) = parse_hand_code(buffer.as_str()).unwrap();

    run_hand_ast(io::stdout(), &instructions)?;

    Ok(())
}

pub fn run_hand_ast(mut writer: impl Write, instructions: &Vec<Instruction>) -> io::Result<()> {
    use Instruction::*;
    let mut buffer = vec![0u8];
    let mut cursor = 0usize;
    let mut flow_offset = 0usize;
    let wormholes_map = calc_wormholes(instructions.clone());

    while let Some(ins) = instructions.get(flow_offset) {
        match ins {
            Next => {
                cursor = cursor + 1;
                if let None = buffer.get(cursor) {
                    buffer.push(0u8);
                }
            }
            Previous => {
                cursor = cursor - 1;
            }
            Increment => {
                if let Some(v) = buffer.get_mut(cursor) {
                    let (add, _) = (*v).overflowing_add(1u8);
                    *v = add;
                }
            }
            LoopStart => {
                if let Some(v) = buffer.get(cursor) {
                    if *v == 0 {
                        flow_offset = *wormholes_map.get(&flow_offset).unwrap_or(&0usize);
                    }
                }
            }
            LoopEnd => {
                if let Some(v) = buffer.get(cursor) {
                    if *v != 0 {
                        flow_offset = *wormholes_map.get(&flow_offset).unwrap_or(&0usize);
                    }
                }
            }
            Decrease => {
                if let Some(v) = buffer.get_mut(cursor) {
                    let (sub, _) = (*v).overflowing_sub(1u8);
                    *v = sub;
                }
            }
            Print => {
                if let Some(b) = buffer.get(cursor) {
                    writer.write(&(*b).to_be_bytes())?;
                }
            }
        }
        flow_offset += 1;
    }

    Ok(())
}

fn calc_wormholes(instructions: Vec<Instruction>) -> HashMap<usize, usize> {
    let mut offset = 0usize;
    let mut stack = VecDeque::from(instructions);
    let mut map = HashMap::new();
    let mut starts = Vec::new();

    while let Some(ins) = stack.pop_front() {
        match ins {
            Instruction::LoopStart => {
                starts.push(offset);
            }
            Instruction::LoopEnd => {
                if let Some(start) = starts.pop() {
                    map.insert(start, offset);
                    map.insert(offset, start);
                }
            }
            _ => (),
        }
        offset += 1;
    }

    map
}

pub fn parse_hand_code(input: &str) -> IResult<&str, Vec<Instruction>> {
    use Instruction::*;
    let keychar = |c| preceded(multispace0, char(c));
    let ins = |c, v| value(v, keychar(c));
    let next_ins = ins('👉', Next);
    let prev_ins = ins('👈', Previous);
    let incr_ins = ins('👆', Increment);
    let decr_ins = ins('👇', Decrease);
    let lost_ins = ins('🤜', LoopStart);
    let lond_ins = ins('🤛', LoopEnd);
    let prnt_ins = ins('👊', Print);
    let ins_alter = alt((
        next_ins, prev_ins, incr_ins, decr_ins, lost_ins, lond_ins, prnt_ins,
    ));
    let mut instructions = terminated(many0(ins_alter), pair(multispace0, eof));
    instructions(input)
}

#[test]
pub fn test_hello() -> io::Result<()> {
    let code =
        "👇🤜👇👇👇👇👇👇👇👉👆👈🤛👉👇👊👇🤜👇👉👆👆👆👆👆👈🤛👉👆👆👊👆👆👆👆👆👆👆👊👊👆👆👆👊";
    let buf = Vec::new();
    let mut writer = io::BufWriter::new(buf);
    let (_, instructions) = parse_hand_code(code).unwrap();

    run_hand_ast(&mut writer, &instructions)?;

    let result = String::from_utf8(writer.into_inner().unwrap()).unwrap();
    assert_eq!(result, "Hello");

    Ok(())
}

#[test]
pub fn test_hello_world() -> io::Result<()> {
    let code =
        "👉👆👆👆👆👆👆👆👆🤜👇👈👆👆👆👆👆👆👆👆👆👉🤛👈👊👉👉👆👉👇🤜👆🤛👆👆👉👆👆👉👆👆👆🤜👉🤜👇👉👆👆👆👈👈👆👆👆👉🤛👈👈🤛👉👇👇👇👇👇👊👉👇👉👆👆👆👊👊👆👆👆👊👉👇👊👈👈👆🤜👉🤜👆👉👆🤛👉👉🤛👈👇👇👇👇👇👇👇👇👇👇👇👇👇👇👊👉👉👊👆👆👆👊👇👇👇👇👇👇👊👇👇👇👇👇👇👇👇👊👉👆👊👉👆👊";
    let buf = Vec::new();
    let mut writer = io::BufWriter::new(buf);
    let (_, instructions) = parse_hand_code(code).unwrap();

    run_hand_ast(&mut writer, &instructions)?;

    let result = String::from_utf8(writer.into_inner().unwrap()).unwrap();
    assert_eq!(result, "Hello World!\n");

    Ok(())
}
