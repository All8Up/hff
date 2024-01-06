mod options;
use clap::Parser;
use hff_std::*;
use options::Options;

fn main() -> Result<()> {
    // Parse the command line options.
    let options = Options::parse();

    // The source must exist.
    if options.name.exists() {
        // Open the given file.
        let file = std::fs::File::open(&options.name)?;
        let hff = open(file)?;

        // Helper to create spaces as indention or empty is indent is disabled.
        let indent_fn = if options.no_indent {
            no_indent
        } else {
            make_indent
        };
        let indent = |depth| indent_fn(options.indent_size, depth, options.max_indent);

        // Iterate depth first over the file content.
        for (depth, table) in hff.depth_first() {
            // Always print out the table information.
            println!(
                "{} ({:<8} | {:<8} : children: {} chunks: {})",
                indent(depth),
                table.primary().to_string(),
                table.secondary().to_string(),
                table.child_count(),
                table.chunk_count()
            );

            // Optionally print out the metadata.
            if options.metadata && table.has_metadata() {
                let metadata = hff.get(&table)?;
                if options.as_ksv {
                    match hff_std::hff_core::utilities::Ksv::from_bytes(metadata.as_slice()) {
                        Ok(ksv) => {
                            println!(" {} {:#?}", indent(depth), ksv);
                        }
                        Err(_) => {
                            println!(" {} <Not a key string vector>", indent(depth));
                        }
                    }
                } else if options.as_string_vector {
                    match hff_std::hff_core::utilities::StringVec::from_bytes(metadata.as_slice()) {
                        Ok(sv) => {
                            println!(" {} {:#?}", indent(depth), sv);
                        }
                        Err(_) => {
                            println!(" {} <Not a string vector>", indent(depth));
                        }
                    }
                } else {
                    match std::str::from_utf8(&metadata) {
                        Ok(s) => {
                            println!(" {}{}", indent(depth), s);
                        }
                        Err(_) => {
                            println!(
                                " {} ({:<8} {:<8})",
                                indent(depth),
                                metadata.len(),
                                table.offset()
                            );
                        }
                    }
                }
            }

            if options.chunk_types {
                for chunk in table.chunks() {
                    println!(
                        " {} [{:<8} | {:<8} Len: {}]",
                        indent(depth),
                        chunk.primary().to_string(),
                        chunk.secondary().to_string(),
                        chunk.len()
                    );
                }
            }
        }

        Ok(())
    } else {
        Err(Error::NotFound(format!("{}", options.name.display())))
    }
}

fn no_indent(_: usize, _: usize, _: usize) -> String {
    String::new()
}

fn make_indent(indent: usize, depth: usize, max: usize) -> String {
    if indent * depth < max {
        std::iter::repeat(' ')
            .take(indent * depth)
            .collect::<String>()
    } else {
        std::iter::repeat(' ')
            .take((indent * max) - 3)
            .collect::<String>()
            + "-> "
    }
}
