use clap::{arg, command, value_parser};
use one_forty_nine_solver::{solve};

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(
            arg!(
                -t --target <NUMBER> "Set the target number"
            )
                // We don't have syntax yet for optional options, so manually calling `required`
                .required(true)
                .value_parser(value_parser!(u8)),
        )
        .get_matches();

    let target: &u8 = matches.get_one::<u8>("target").unwrap();
    if *target > 49 {
        println!()
    }

    use std::time::Instant;
    let now = Instant::now();
    solve(*target);
    let elapsed = now.elapsed();
    println!("Duration: {:.3?}", elapsed);
}
