use regex::Regex;
use std::sync::OnceLock;

static STATUS_BRANCH_RE: OnceLock<Regex> = OnceLock::new();
static AHEAD_RE: OnceLock<Regex> = OnceLock::new();
static COMMIT_HASH_RE: OnceLock<Regex> = OnceLock::new();
static INSERTIONS_RE: OnceLock<Regex> = OnceLock::new();
static DELETIONS_RE: OnceLock<Regex> = OnceLock::new();
static FILES_CHANGED_RE: OnceLock<Regex> = OnceLock::new();
static CLONE_OBJECTS_RE: OnceLock<Regex> = OnceLock::new();
static STASH_RE: OnceLock<Regex> = OnceLock::new();

fn status_branch_re() -> &'static Regex {
    STATUS_BRANCH_RE.get_or_init(|| Regex::new(r"On branch (\S+)").unwrap())
}
fn ahead_re() -> &'static Regex {
    AHEAD_RE.get_or_init(|| Regex::new(r"ahead of .+ by (\d+) commit").unwrap())
}
fn commit_hash_re() -> &'static Regex {
    COMMIT_HASH_RE.get_or_init(|| Regex::new(r"\[([\w/.:-]+)\s+([a-f0-9]+)\]").unwrap())
}
fn insertions_re() -> &'static Regex {
    INSERTIONS_RE.get_or_init(|| Regex::new(r"(\d+) insertions?\(\+\)").unwrap())
}
fn deletions_re() -> &'static Regex {
    DELETIONS_RE.get_or_init(|| Regex::new(r"(\d+) deletions?\(-\)").unwrap())
}
fn files_changed_re() -> &'static Regex {
    FILES_CHANGED_RE.get_or_init(|| Regex::new(r"(\d+) files? changed").unwrap())
}
fn clone_objects_re() -> &'static Regex {
    CLONE_OBJECTS_RE.get_or_init(|| Regex::new(r"Receiving objects:.*?(\d+)").unwrap())
}
fn stash_re() -> &'static Regex {
    STASH_RE.get_or_init(|| Regex::new(r"stash@\{(\d+)\}:\s*(.+)").unwrap())
}

pub fn compress(command: &str, output: &str) -> Option<String> {
    if command.contains("status") {
        return Some(compress_status(output));
    }
    if command.contains("log") {
        return Some(compress_log(output));
    }
    if command.contains("diff") && !command.contains("difftool") {
        return Some(compress_diff(output));
    }
    if command.contains("add") && !command.contains("remote add") {
        return Some(compress_add(output));
    }
    if command.contains("commit") {
        return Some(compress_commit(output));
    }
    if command.contains("push") {
        return Some(compress_push(output));
    }
    if command.contains("pull") {
        return Some(compress_pull(output));
    }
    if command.contains("fetch") {
        return Some(compress_fetch(output));
    }
    if command.contains("clone") {
        return Some(compress_clone(output));
    }
    if command.contains("branch") {
        return Some(compress_branch(output));
    }
    if command.contains("checkout") || command.contains("switch") {
        return Some(compress_checkout(output));
    }
    if command.contains("merge") {
        return Some(compress_merge(output));
    }
    if command.contains("stash") {
        return Some(compress_stash(output));
    }
    if command.contains("tag") {
        return Some(compress_tag(output));
    }
    if command.contains("reset") {
        return Some(compress_reset(output));
    }
    if command.contains("remote") {
        return Some(compress_remote(output));
    }
    if command.contains("blame") {
        return Some(compress_blame(output));
    }
    if command.contains("cherry-pick") {
        return Some(compress_cherry_pick(output));
    }
    if command.contains("show") {
        return Some(compress_show(output));
    }
    if command.contains("rebase") {
        return Some(compress_rebase(output));
    }
    if command.contains("submodule") {
        return Some(compress_submodule(output));
    }
    if command.contains("worktree") {
        return Some(compress_worktree(output));
    }
    if command.contains("bisect") {
        return Some(compress_bisect(output));
    }
    None
}

fn compress_status(output: &str) -> String {
    let mut branch = String::new();
    let mut ahead = 0u32;
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();

    let mut section = "";

    for line in output.lines() {
        if let Some(caps) = status_branch_re().captures(line) {
            branch = caps[1].to_string();
        }
        if let Some(caps) = ahead_re().captures(line) {
            ahead = caps[1].parse().unwrap_or(0);
        }

        if line.contains("Changes to be committed") {
            section = "staged";
        } else if line.contains("Changes not staged") {
            section = "unstaged";
        } else if line.contains("Untracked files") {
            section = "untracked";
        }

        let trimmed = line.trim();
        if trimmed.starts_with("new file:") {
            let file = trimmed.trim_start_matches("new file:").trim();
            if section == "staged" {
                staged.push(format!("+{file}"));
            }
        } else if trimmed.starts_with("modified:") {
            let file = trimmed.trim_start_matches("modified:").trim();
            match section {
                "staged" => staged.push(format!("~{file}")),
                "unstaged" => unstaged.push(format!("~{file}")),
                _ => {}
            }
        } else if trimmed.starts_with("deleted:") {
            let file = trimmed.trim_start_matches("deleted:").trim();
            if section == "staged" {
                staged.push(format!("-{file}"));
            }
        } else if trimmed.starts_with("renamed:") {
            let file = trimmed.trim_start_matches("renamed:").trim();
            if section == "staged" {
                staged.push(format!("→{file}"));
            }
        } else if trimmed.starts_with("copied:") {
            let file = trimmed.trim_start_matches("copied:").trim();
            if section == "staged" {
                staged.push(format!("©{file}"));
            }
        } else if section == "untracked"
            && !trimmed.is_empty()
            && !trimmed.starts_with('(')
            && !trimmed.starts_with("Untracked")
        {
            untracked.push(trimmed.to_string());
        }
    }

    if branch.is_empty() && staged.is_empty() && unstaged.is_empty() && untracked.is_empty() {
        return compact_lines(output.trim(), 10);
    }

    let mut parts = Vec::new();
    let branch_display = if branch.is_empty() {
        "?".to_string()
    } else {
        branch
    };
    let ahead_str = if ahead > 0 {
        format!(" ↑{ahead}")
    } else {
        String::new()
    };
    parts.push(format!("{branch_display}{ahead_str}"));

    if !staged.is_empty() {
        parts.push(format!("staged: {}", staged.join(" ")));
    }
    if !unstaged.is_empty() {
        parts.push(format!("unstaged: {}", unstaged.join(" ")));
    }
    if !untracked.is_empty() {
        parts.push(format!("untracked: {}", untracked.join(" ")));
    }

    if output.contains("nothing to commit") && parts.len() == 1 {
        parts.push("clean".to_string());
    }

    parts.join("\n")
}

fn is_diff_or_stat_line(line: &str) -> bool {
    let t = line.trim();
    t.starts_with("diff --git")
        || t.starts_with("index ")
        || t.starts_with("--- a/")
        || t.starts_with("+++ b/")
        || t.starts_with("@@ ")
        || t.starts_with("Binary files")
        || t.starts_with("new file mode")
        || t.starts_with("deleted file mode")
        || t.starts_with("old mode")
        || t.starts_with("new mode")
        || t.starts_with("similarity index")
        || t.starts_with("rename from")
        || t.starts_with("rename to")
        || t.starts_with("copy from")
        || t.starts_with("copy to")
        || (t.starts_with('+') && !t.starts_with("+++"))
        || (t.starts_with('-') && !t.starts_with("---"))
        || (t.contains(" | ") && t.chars().any(|c| c == '+' || c == '-'))
}

fn compress_log(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let max_entries = 20;

    let is_oneline = !lines[0].starts_with("commit ");
    if is_oneline {
        if lines.len() <= max_entries {
            return lines.join("\n");
        }
        let shown = &lines[..max_entries];
        return format!(
            "{}\n... ({} more commits)",
            shown.join("\n"),
            lines.len() - max_entries
        );
    }

    let has_diff = lines.iter().any(|l| l.starts_with("diff --git"));
    let has_stat = lines
        .iter()
        .any(|l| l.contains(" | ") && l.trim().ends_with(['+', '-']));
    let mut total_additions = 0u32;
    let mut total_deletions = 0u32;

    let mut entries = Vec::new();
    let mut in_diff = false;
    let mut got_message = false;

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.starts_with("commit ") {
            let hash = &trimmed[7..14.min(trimmed.len())];
            entries.push(hash.to_string());
            in_diff = false;
            got_message = false;
            continue;
        }

        if trimmed.starts_with("Author:")
            || trimmed.starts_with("Date:")
            || trimmed.starts_with("Merge:")
        {
            continue;
        }

        if trimmed.starts_with("diff --git") || trimmed.starts_with("---") && trimmed.contains("a/")
        {
            in_diff = true;
        }

        if in_diff || is_diff_or_stat_line(trimmed) {
            if trimmed.starts_with('+') && !trimmed.starts_with("+++") {
                total_additions += 1;
            } else if trimmed.starts_with('-') && !trimmed.starts_with("---") {
                total_deletions += 1;
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        if !got_message {
            if let Some(last) = entries.last_mut() {
                *last = format!("{last} {trimmed}");
            }
            got_message = true;
        }
    }

    if entries.is_empty() {
        return output.to_string();
    }

    let mut result = if entries.len() > max_entries {
        let shown = &entries[..max_entries];
        format!(
            "{}\n... ({} more commits)",
            shown.join("\n"),
            entries.len() - max_entries
        )
    } else {
        entries.join("\n")
    };

    if (has_diff || has_stat) && (total_additions > 0 || total_deletions > 0) {
        result.push_str(&format!(
            "\n[{} commits, +{total_additions}/-{total_deletions} total]",
            entries.len()
        ));
    }

    result
}

fn compress_diff(output: &str) -> String {
    let mut files = Vec::new();
    let mut current_file = String::new();
    let mut additions = 0;
    let mut deletions = 0;

    for line in output.lines() {
        if line.starts_with("diff --git") {
            if !current_file.is_empty() {
                files.push(format!("{current_file} +{additions}/-{deletions}"));
            }
            current_file = line.split(" b/").nth(1).unwrap_or("?").to_string();
            additions = 0;
            deletions = 0;
        } else if line.starts_with('+') && !line.starts_with("+++") {
            additions += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            deletions += 1;
        }
    }
    if !current_file.is_empty() {
        files.push(format!("{current_file} +{additions}/-{deletions}"));
    }

    files.join("\n")
}

fn compress_add(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return "ok".to_string();
    }
    let lines: Vec<&str> = trimmed.lines().collect();
    if lines.len() <= 3 {
        return trimmed.to_string();
    }
    format!("ok (+{} files)", lines.len())
}

fn compress_commit(output: &str) -> String {
    let mut hook_lines: Vec<&str> = Vec::new();
    let mut commit_part = String::new();
    let mut found_commit = false;

    for line in output.lines() {
        if !found_commit && commit_hash_re().is_match(line) {
            found_commit = true;
        }
        if !found_commit {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                hook_lines.push(trimmed);
            }
        }
    }

    if let Some(caps) = commit_hash_re().captures(output) {
        let branch = &caps[1];
        let hash = &caps[2];
        let stats = extract_change_stats(output);
        let msg = output
            .lines()
            .find(|l| commit_hash_re().is_match(l))
            .and_then(|l| l.split(']').nth(1))
            .map(|m| m.trim())
            .unwrap_or("");
        commit_part = if stats.is_empty() {
            format!("{hash} ({branch}) {msg}")
        } else {
            format!("{hash} ({branch}) {msg} [{stats}]")
        };
    }

    if commit_part.is_empty() {
        let trimmed = output.trim();
        if trimmed.is_empty() {
            return "ok".to_string();
        }
        return compact_lines(trimmed, 5);
    }

    if hook_lines.is_empty() {
        return commit_part;
    }

    let failed: Vec<&&str> = hook_lines
        .iter()
        .filter(|l| {
            let low = l.to_lowercase();
            low.contains("failed") || low.contains("error") || low.contains("warning")
        })
        .collect();
    let passed_count = hook_lines.len() - failed.len();

    let hook_output = if !failed.is_empty() {
        let mut parts = Vec::new();
        if passed_count > 0 {
            parts.push(format!("{passed_count} checks passed"));
        }
        for f in failed.iter().take(5) {
            parts.push(f.to_string());
        }
        if failed.len() > 5 {
            parts.push(format!("... ({} more failures)", failed.len() - 5));
        }
        parts.join("\n")
    } else if hook_lines.len() > 5 {
        format!("{} hooks passed", hook_lines.len())
    } else {
        hook_lines.join("\n")
    };

    format!("{hook_output}\n{commit_part}")
}

fn compress_push(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return "ok".to_string();
    }

    let mut ref_line = String::new();
    let mut remote_urls: Vec<String> = Vec::new();
    let mut rejected = false;

    for line in trimmed.lines() {
        let l = line.trim();

        if l.contains("rejected") {
            rejected = true;
        }

        if l.contains("->") && !l.starts_with("remote:") {
            ref_line = l.to_string();
        }

        if l.contains("Everything up-to-date") {
            return "ok (up-to-date)".to_string();
        }

        if l.starts_with("remote:") || l.starts_with("To ") {
            let content = l.trim_start_matches("remote:").trim();
            if content.contains("http")
                || content.contains("pipeline")
                || content.contains("merge_request")
                || content.contains("pull/")
            {
                remote_urls.push(content.to_string());
            }
        }
    }

    if rejected {
        let reject_lines: Vec<&str> = trimmed
            .lines()
            .filter(|l| l.contains("rejected") || l.contains("error") || l.contains("remote:"))
            .collect();
        return format!("REJECTED:\n{}", compact_lines(&reject_lines.join("\n"), 5));
    }

    let mut parts = Vec::new();
    if !ref_line.is_empty() {
        parts.push(format!("ok {ref_line}"));
    } else {
        parts.push("ok (pushed)".to_string());
    }
    for url in &remote_urls {
        parts.push(url.clone());
    }

    parts.join("\n")
}

fn compress_pull(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.contains("Already up to date") {
        return "ok (up-to-date)".to_string();
    }

    let stats = extract_change_stats(trimmed);
    if !stats.is_empty() {
        return format!("ok {stats}");
    }

    compact_lines(trimmed, 5)
}

fn compress_fetch(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return "ok".to_string();
    }

    let mut new_branches = Vec::new();
    for line in trimmed.lines() {
        let l = line.trim();
        if l.contains("[new branch]") || l.contains("[new tag]") {
            if let Some(name) = l.split("->").last() {
                new_branches.push(name.trim().to_string());
            }
        }
    }

    if new_branches.is_empty() {
        return "ok (fetched)".to_string();
    }
    format!("ok (new: {})", new_branches.join(", "))
}

fn compress_clone(output: &str) -> String {
    let mut objects = 0u32;
    for line in output.lines() {
        if let Some(caps) = clone_objects_re().captures(line) {
            objects = caps[1].parse().unwrap_or(0);
        }
    }

    let into = output
        .lines()
        .find(|l| l.contains("Cloning into"))
        .and_then(|l| l.split('\'').nth(1))
        .unwrap_or("repo");

    if objects > 0 {
        format!("cloned '{into}' ({objects} objects)")
    } else {
        format!("cloned '{into}'")
    }
}

fn compress_branch(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return "ok".to_string();
    }

    let branches: Vec<String> = trimmed
        .lines()
        .filter_map(|line| {
            let l = line.trim();
            if l.is_empty() {
                return None;
            }
            if let Some(rest) = l.strip_prefix('*') {
                Some(format!("*{}", rest.trim()))
            } else {
                Some(l.to_string())
            }
        })
        .collect();

    branches.join(", ")
}

fn compress_checkout(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return "ok".to_string();
    }

    for line in trimmed.lines() {
        let l = line.trim();
        if l.starts_with("Switched to") || l.starts_with("Already on") {
            let branch = l.split('\'').nth(1).unwrap_or(l);
            return format!("→ {branch}");
        }
        if l.starts_with("Your branch is up to date") {
            continue;
        }
    }

    compact_lines(trimmed, 3)
}

fn compress_merge(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.contains("Already up to date") {
        return "ok (up-to-date)".to_string();
    }
    if trimmed.contains("CONFLICT") {
        let conflicts: Vec<&str> = trimmed.lines().filter(|l| l.contains("CONFLICT")).collect();
        return format!(
            "CONFLICT ({} files):\n{}",
            conflicts.len(),
            conflicts.join("\n")
        );
    }

    let stats = extract_change_stats(trimmed);
    if !stats.is_empty() {
        return format!("merged {stats}");
    }
    compact_lines(trimmed, 3)
}

fn compress_stash(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return "ok".to_string();
    }

    if trimmed.starts_with("Saved working directory") {
        return "stashed".to_string();
    }
    if trimmed.starts_with("Dropped") {
        return "dropped".to_string();
    }

    let stashes: Vec<String> = trimmed
        .lines()
        .filter_map(|line| {
            stash_re()
                .captures(line)
                .map(|caps| format!("@{}: {}", &caps[1], &caps[2]))
        })
        .collect();

    if stashes.is_empty() {
        return compact_lines(trimmed, 3);
    }
    stashes.join("\n")
}

fn compress_tag(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return "ok".to_string();
    }

    let tags: Vec<&str> = trimmed.lines().filter(|l| !l.trim().is_empty()).collect();
    if tags.len() <= 10 {
        return tags.join(", ");
    }
    format!("{} (... {} total)", tags[..5].join(", "), tags.len())
}

fn compress_reset(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return "ok".to_string();
    }

    let mut unstaged: Vec<&str> = Vec::new();
    for line in trimmed.lines() {
        let l = line.trim();
        if l.starts_with("Unstaged changes after reset:") {
            continue;
        }
        if l.starts_with('M') || l.starts_with('D') || l.starts_with('A') {
            unstaged.push(l);
        }
    }

    if unstaged.is_empty() {
        return compact_lines(trimmed, 3);
    }
    format!("reset ok ({} files unstaged)", unstaged.len())
}

fn compress_remote(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return "ok".to_string();
    }

    let mut remotes = std::collections::HashMap::new();
    for line in trimmed.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            remotes
                .entry(parts[0].to_string())
                .or_insert_with(|| parts[1].to_string());
        }
    }

    if remotes.is_empty() {
        return trimmed.to_string();
    }

    remotes
        .iter()
        .map(|(name, url)| format!("{name}: {url}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn compress_blame(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() <= 20 {
        return output.to_string();
    }

    let unique_authors: std::collections::HashSet<&str> = lines
        .iter()
        .filter_map(|l| l.split('(').nth(1)?.split_whitespace().next())
        .collect();

    format!("{} lines, {} authors", lines.len(), unique_authors.len())
}

fn compress_cherry_pick(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return "ok".to_string();
    }
    if trimmed.contains("CONFLICT") {
        return "CONFLICT (cherry-pick)".to_string();
    }
    let stats = extract_change_stats(trimmed);
    if !stats.is_empty() {
        return format!("ok {stats}");
    }
    compact_lines(trimmed, 3)
}

fn compress_show(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let mut hash = String::new();
    let mut message = String::new();
    let mut additions = 0u32;
    let mut deletions = 0u32;

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("commit ") && hash.is_empty() {
            hash = trimmed[7..14.min(trimmed.len())].to_string();
        } else if trimmed.starts_with('+') && !trimmed.starts_with("+++") {
            additions += 1;
        } else if trimmed.starts_with('-') && !trimmed.starts_with("---") {
            deletions += 1;
        } else if !trimmed.is_empty()
            && !trimmed.starts_with("Author:")
            && !trimmed.starts_with("Date:")
            && !trimmed.starts_with("Merge:")
            && !is_diff_or_stat_line(trimmed)
            && message.is_empty()
            && !hash.is_empty()
        {
            message = trimmed.to_string();
        }
    }

    let stats = extract_change_stats(output);
    let diff_summary = if additions > 0 || deletions > 0 {
        format!(" +{additions}/-{deletions}")
    } else if !stats.is_empty() {
        format!(" [{stats}]")
    } else {
        String::new()
    };

    if hash.is_empty() {
        return compact_lines(output.trim(), 10);
    }

    format!("{hash} {message}{diff_summary}")
}

fn compress_rebase(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return "ok".to_string();
    }
    if trimmed.contains("Already up to date") || trimmed.contains("is up to date") {
        return "ok (up-to-date)".to_string();
    }
    if trimmed.contains("Successfully rebased") {
        let stats = extract_change_stats(trimmed);
        return if stats.is_empty() {
            "ok (rebased)".to_string()
        } else {
            format!("ok (rebased) {stats}")
        };
    }
    if trimmed.contains("CONFLICT") {
        let conflicts: Vec<&str> = trimmed.lines().filter(|l| l.contains("CONFLICT")).collect();
        return format!(
            "CONFLICT ({} files):\n{}",
            conflicts.len(),
            conflicts.join("\n")
        );
    }
    compact_lines(trimmed, 5)
}

fn compress_submodule(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return "ok".to_string();
    }

    let mut modules = Vec::new();
    for line in trimmed.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let status_char = if line.starts_with('+') {
                "~"
            } else if line.starts_with('-') {
                "!"
            } else {
                ""
            };
            modules.push(format!("{status_char}{}", parts.last().unwrap_or(&"?")));
        }
    }

    if modules.is_empty() {
        return compact_lines(trimmed, 5);
    }
    format!("{} submodules: {}", modules.len(), modules.join(", "))
}

fn compress_worktree(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return "ok".to_string();
    }

    let mut worktrees = Vec::new();
    let mut current_path = String::new();
    let mut current_branch = String::new();

    for line in trimmed.lines() {
        let l = line.trim();
        if !l.contains(' ') && !l.is_empty() && current_path.is_empty() {
            current_path = l.to_string();
        } else if l.starts_with("HEAD ") {
            // skip
        } else if l.starts_with("branch ") || l.contains("detached") || l.contains("bare") {
            current_branch = l.to_string();
        } else if l.is_empty() && !current_path.is_empty() {
            let short_path = current_path.rsplit('/').next().unwrap_or(&current_path);
            worktrees.push(format!("{short_path} [{current_branch}]"));
            current_path.clear();
            current_branch.clear();
        }
    }
    if !current_path.is_empty() {
        let short_path = current_path.rsplit('/').next().unwrap_or(&current_path);
        worktrees.push(format!("{short_path} [{current_branch}]"));
    }

    if worktrees.is_empty() {
        return compact_lines(trimmed, 5);
    }
    format!("{} worktrees:\n{}", worktrees.len(), worktrees.join("\n"))
}

fn compress_bisect(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return "ok".to_string();
    }

    for line in trimmed.lines() {
        let l = line.trim();
        if l.contains("is the first bad commit") {
            let hash = l.split_whitespace().next().unwrap_or("?");
            let short = &hash[..7.min(hash.len())];
            return format!("found: {short} is first bad commit");
        }
        if l.starts_with("Bisecting:") {
            return l.to_string();
        }
    }

    compact_lines(trimmed, 5)
}

fn extract_change_stats(output: &str) -> String {
    let files = files_changed_re()
        .captures(output)
        .and_then(|c| c[1].parse::<u32>().ok())
        .unwrap_or(0);
    let ins = insertions_re()
        .captures(output)
        .and_then(|c| c[1].parse::<u32>().ok())
        .unwrap_or(0);
    let del = deletions_re()
        .captures(output)
        .and_then(|c| c[1].parse::<u32>().ok())
        .unwrap_or(0);

    if files > 0 || ins > 0 || del > 0 {
        format!("{files} files, +{ins}/-{del}")
    } else {
        String::new()
    }
}

fn compact_lines(text: &str, max: usize) -> String {
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() <= max {
        return lines.join("\n");
    }
    format!(
        "{}\n... ({} more lines)",
        lines[..max].join("\n"),
        lines.len() - max
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_status_compresses() {
        let output = "On branch main\nYour branch is up to date with 'origin/main'.\n\nChanges not staged for commit:\n  (use \"git add <file>...\" to update what will be committed)\n\n\tmodified:   src/main.rs\n\tmodified:   src/lib.rs\n\nno changes added to commit (use \"git add\" and/or \"git commit -a\")\n";
        let result = compress("git status", output).unwrap();
        assert!(result.contains("main"), "should contain branch name");
        assert!(result.contains("main.rs"), "should list modified files");
        assert!(result.len() < output.len(), "should be shorter than input");
    }

    #[test]
    fn git_add_compresses_to_ok() {
        let result = compress("git add .", "").unwrap();
        assert!(result.contains("ok"), "git add should compress to 'ok'");
    }

    #[test]
    fn git_commit_extracts_hash() {
        let output =
            "[main abc1234] fix: resolve bug\n 2 files changed, 10 insertions(+), 3 deletions(-)\n";
        let result = compress("git commit -m 'fix'", output).unwrap();
        assert!(result.contains("abc1234"), "should extract commit hash");
    }

    #[test]
    fn git_push_compresses() {
        let output = "Enumerating objects: 5, done.\nCounting objects: 100% (5/5), done.\nDelta compression using up to 8 threads\nCompressing objects: 100% (3/3), done.\nWriting objects: 100% (3/3), 1.2 KiB | 1.2 MiB/s, done.\nTotal 3 (delta 2), reused 0 (delta 0)\nTo github.com:user/repo.git\n   abc1234..def5678  main -> main\n";
        let result = compress("git push", output).unwrap();
        assert!(result.len() < output.len(), "should compress push output");
    }

    #[test]
    fn git_log_compresses() {
        let output = "commit abc1234567890\nAuthor: User <user@email.com>\nDate:   Mon Mar 25 10:00:00 2026 +0100\n\n    feat: add feature\n\ncommit def4567890abc\nAuthor: User <user@email.com>\nDate:   Sun Mar 24 09:00:00 2026 +0100\n\n    fix: resolve issue\n";
        let result = compress("git log", output).unwrap();
        assert!(result.len() < output.len(), "should compress log output");
    }

    #[test]
    fn git_log_oneline_truncates_long() {
        let lines: Vec<String> = (0..50)
            .map(|i| format!("abc{i:04} feat: commit number {i}"))
            .collect();
        let output = lines.join("\n");
        let result = compress("git log --oneline", &output).unwrap();
        assert!(
            result.contains("... (30 more commits)"),
            "should truncate to 20 entries"
        );
        assert!(
            result.lines().count() <= 22,
            "should have at most 21 lines (20 + summary)"
        );
    }

    #[test]
    fn git_log_oneline_short_unchanged() {
        let output = "abc1234 feat: one\ndef5678 fix: two\nghi9012 docs: three";
        let result = compress("git log --oneline", output).unwrap();
        assert_eq!(result, output, "short oneline should pass through");
    }

    #[test]
    fn git_log_standard_truncates_long() {
        let mut output = String::new();
        for i in 0..30 {
            output.push_str(&format!(
                "commit {i:07}abc1234\nAuthor: U <u@e.com>\nDate:   Mon\n\n    msg {i}\n\n"
            ));
        }
        let result = compress("git log", &output).unwrap();
        assert!(
            result.contains("... (10 more commits)"),
            "should truncate standard log"
        );
    }

    #[test]
    fn git_diff_compresses() {
        let output = "diff --git a/src/main.rs b/src/main.rs\nindex abc1234..def5678 100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"hello\");\n     let x = 1;\n }";
        let result = compress("git diff", output).unwrap();
        assert!(result.contains("main.rs"), "should reference changed file");
    }

    #[test]
    fn git_push_preserves_pipeline_url() {
        let output = "Enumerating objects: 5, done.\nCounting objects: 100% (5/5), done.\nDelta compression using up to 8 threads\nCompressing objects: 100% (3/3), done.\nWriting objects: 100% (3/3), 1.2 KiB | 1.2 MiB/s, done.\nTotal 3 (delta 2), reused 0 (delta 0)\nremote:\nremote: To create a merge request for main, visit:\nremote:   https://gitlab.com/user/repo/-/merge_requests/new?source=main\nremote:\nremote: View pipeline for this push:\nremote:   https://gitlab.com/user/repo/-/pipelines/12345\nremote:\nTo gitlab.com:user/repo.git\n   abc1234..def5678  main -> main\n";
        let result = compress("git push", output).unwrap();
        assert!(
            result.contains("pipeline"),
            "should preserve pipeline URL, got: {result}"
        );
        assert!(
            result.contains("merge_request"),
            "should preserve merge request URL"
        );
        assert!(result.contains("->"), "should contain ref update line");
    }

    #[test]
    fn git_push_preserves_github_pr_url() {
        let output = "Enumerating objects: 5, done.\nremote:\nremote: Create a pull request for 'feature' on GitHub by visiting:\nremote:   https://github.com/user/repo/pull/new/feature\nremote:\nTo github.com:user/repo.git\n   abc1234..def5678  feature -> feature\n";
        let result = compress("git push", output).unwrap();
        assert!(
            result.contains("pull/"),
            "should preserve GitHub PR URL, got: {result}"
        );
    }

    #[test]
    fn git_commit_preserves_hook_output() {
        let output = "Running pre-commit hooks...\ncheck-yaml..........passed\ncheck-json..........passed\nruff.................failed\nfixing src/app.py\n[main abc1234] fix: resolve bug\n 2 files changed, 10 insertions(+), 3 deletions(-)\n";
        let result = compress("git commit -m 'fix'", output).unwrap();
        assert!(
            result.contains("ruff"),
            "should preserve hook output, got: {result}"
        );
        assert!(
            result.contains("abc1234"),
            "should still extract commit hash"
        );
    }

    #[test]
    fn git_commit_no_hooks() {
        let output =
            "[main abc1234] fix: resolve bug\n 2 files changed, 10 insertions(+), 3 deletions(-)\n";
        let result = compress("git commit -m 'fix'", output).unwrap();
        assert!(result.contains("abc1234"), "should extract commit hash");
        assert!(
            !result.contains("hook"),
            "should not mention hooks when none present"
        );
    }

    #[test]
    fn git_log_with_patch_filters_diff_content() {
        let output = "commit abc1234567890\nAuthor: User <user@email.com>\nDate:   Mon Mar 25 10:00:00 2026 +0100\n\n    feat: add feature\n\ndiff --git a/src/main.rs b/src/main.rs\nindex abc1234..def5678 100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"hello\");\n     let x = 1;\n }\n\ncommit def4567890abc\nAuthor: User <user@email.com>\nDate:   Sun Mar 24 09:00:00 2026 +0100\n\n    fix: resolve issue\n\ndiff --git a/src/lib.rs b/src/lib.rs\nindex 111..222 100644\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1,2 @@\n+pub fn helper() {}\n";
        let result = compress("git log -p", output).unwrap();
        assert!(
            !result.contains("println"),
            "should NOT contain diff content, got: {result}"
        );
        assert!(result.contains("abc1234"), "should contain commit hash");
        assert!(
            result.contains("feat: add feature"),
            "should contain commit message"
        );
        assert!(
            result.len() < output.len() / 2,
            "compressed should be less than half of original ({} vs {})",
            result.len(),
            output.len()
        );
    }

    #[test]
    fn git_log_with_stat_filters_stat_content() {
        let mut output = String::new();
        for i in 0..5 {
            output.push_str(&format!(
                "commit {i:07}abc1234\nAuthor: U <u@e.com>\nDate:   Mon\n\n    msg {i}\n\n src/file{i}.rs | 10 ++++------\n 1 file changed, 4 insertions(+), 6 deletions(-)\n\n"
            ));
        }
        let result = compress("git log --stat", &output).unwrap();
        assert!(
            result.len() < output.len() / 2,
            "stat output should be compressed ({} vs {})",
            result.len(),
            output.len()
        );
    }

    #[test]
    fn git_commit_with_feature_branch() {
        let output = "[feature/my-branch abc1234] feat: add new thing\n 3 files changed, 20 insertions(+), 5 deletions(-)\n";
        let result = compress("git commit -m 'feat'", output).unwrap();
        assert!(
            result.contains("abc1234"),
            "should extract hash from feature branch, got: {result}"
        );
        assert!(
            result.contains("feature/my-branch"),
            "should preserve branch name, got: {result}"
        );
    }

    #[test]
    fn git_commit_many_hooks_compressed() {
        let mut output = String::new();
        for i in 0..30 {
            output.push_str(&format!("check-{i}..........passed\n"));
        }
        output.push_str("[main abc1234] fix: resolve bug\n 1 file changed, 1 insertion(+)\n");
        let result = compress("git commit -m 'fix'", &output).unwrap();
        assert!(result.contains("abc1234"), "should contain commit hash");
        assert!(
            result.contains("hooks passed"),
            "should summarize passed hooks, got: {result}"
        );
        assert!(
            result.len() < output.len() / 2,
            "should compress verbose hook output ({} vs {})",
            result.len(),
            output.len()
        );
    }
}
