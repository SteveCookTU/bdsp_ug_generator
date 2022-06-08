#![allow(unused)]

use clap::{Parser};
use bdsp_ug_generator::{RoomType, run, Version};

#[derive(Parser)]
struct Cli {
    #[clap(arg_enum)]
    version: Version,
    #[clap(arg_enum)]
    room: RoomType,
    #[clap(short = 'f', long, default_value = "6")]
    story_flag: u8,
    #[clap(short = 's', long)]
    shiny_only: bool,
    advances: u32,
    s0: String,
    s1: String,
    s2: String,
    s3: String
}

fn main() {
    let cli: Cli = Cli::parse();

    println!("Advances: {}", cli.advances);
    let s0 = cli.s0.trim_start_matches("0x");
    let s0 = u32::from_str_radix(s0, 16).expect("Failed to parse s0 to u32");
    let s1 = cli.s1.trim_start_matches("0x");
    let s1 = u32::from_str_radix(s1, 16).expect("Failed to parse s1 to u32");
    let s2 = cli.s2.trim_start_matches("0x");
    let s2 = u32::from_str_radix(s2, 16).expect("Failed to parse s2 to u32");
    let s3 = cli.s3.trim_start_matches("0x");
    let s3 = u32::from_str_radix(s3, 16).expect("Failed to parse s3 to u32");
    println!("s0: {:#08X}", s0);
    println!("s1: {:#08X}", s1);
    println!("s2: {:#08X}", s2);
    println!("s3: {:#08X}", s3);
    println!();

    run(cli.advances, s0, s1, s2, s3, cli.version, cli.story_flag, cli.room, cli.shiny_only);

}
