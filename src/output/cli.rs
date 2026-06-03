/// Prints beautiful, styled Markdown content directly to the user's terminal using termimad.
pub fn print_markdown(markdown: &str) {
    let skin = termimad::MadSkin::default();
    println!("{}", skin.term_text(markdown));
}
