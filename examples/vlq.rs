use vlq_rust::WriteVlqExt;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    let n = match args.first().and_then(|x| x.parse::<isize>().ok()) {
        Some(v) => v,
        None => {
            eprintln!("Usage: <number>");
            std::process::exit(1);
        }
    };

    let mut buf = vec![];
    std::io::Cursor::new(&mut buf).write_vlq(n).unwrap();
    let out = buf.iter().map(|x| format!("{:08b}", x)).collect::<Vec<_>>().join(" ");
    println!("{}", out);
}