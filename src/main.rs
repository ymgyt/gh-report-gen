use gh_report_gen::cli;

#[tokio::main]
async fn main() {
    let exit_code = cli::parse().run().await;
    std::process::exit(exit_code);
}
