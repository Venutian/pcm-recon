fn main() {
    let path = "C:/Users/YoelM/Documents/Paradox Interactive/New folder/Career_1.cdb";
    match pcm_recon_lib::parser_debug(path) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("Error: {e}"),
    }
}
