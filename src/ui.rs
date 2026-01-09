use anyhow::Result;
use crate::ripgrep::SearchResult;

pub fn display_results(results: &[SearchResult]) -> Result<()> {
    if results.is_empty() {
        println!("No results found.");
        return Ok(());
    }

    println!("\nFound {} result{}:\n", results.len(), if results.len() == 1 { "" } else { "s" });

    for result in results {
        // File path and line number in green
        print!("\x1b[32m{}:{}\x1b[0m:", result.path.display(), result.line_number);

        // Line content with highlighted matches
        let line = &result.line_content;
        let mut last_pos = 0;

        for m in &result.matches {
            // Print text before match
            if m.start > last_pos {
                print!("{}", &line[last_pos..m.start]);
            }
            // Print match in yellow with bold
            print!("\x1b[1;33m{}\x1b[0m", &line[m.start..m.end]);
            last_pos = m.end;
        }

        // Print remaining text
        if last_pos < line.len() {
            print!("{}", &line[last_pos..]);
        }

        println!();
    }

    println!();
    Ok(())
}
