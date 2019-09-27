use vlq_rust::fast;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    let n = match args.first().and_then(|x| x.parse::<u64>().ok()) {
        Some(v) => v,
        None => {
            eprintln!("Usage: <number>");
            std::process::exit(1);
        }
    };

    let v = fast::encode(n);
    println!("{:?}", v);
}
