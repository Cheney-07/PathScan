fn main() {
    let args: Vec<String> = std::env::args().collect();
    let is_tui = args.iter().any(|a| a == "--tui");

    if is_tui {
        pathscan_tui::run().unwrap();
    } else {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(pathscan_cli::run());
    }
}
