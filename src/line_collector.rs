use crate::commit_line::format_commit_line;
    static ref STATIC_HEADER_PREFIXES: Vec<(&'static str, &'static str)> = vec![
        ("diff ", FAINT),
        ("index ", FAINT),
        ("Binary files ", BOLD),
        ("copy from ", FAINT),
        ("copy to ", BOLD),
        ("rename from ", FAINT),
        ("rename to ", BOLD),
        ("similarity index ", FAINT),
        ("new file mode ", FAINT),
        ("deleted file mode ", FAINT),
        ("--- /dev/null", FAINT),
        ("+++ /dev/null", FAINT),
    ];
        if line.starts_with("commit") {
            self.consume_plain_line(&format_commit_line(&line));
            return;
        }
