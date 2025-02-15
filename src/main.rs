use crawl::lang::Crawl;

fn main() {
    let source = "if roll 1-3 on 1d6 => reminder \"pretty good\"";
    let crawl = Crawl::new();
    crawl.execute(source);
}
