use std::io::{self, IsTerminal, Write};

const LOGO: [&str; 6] = [
    r"  ██████╗ ███████╗████████╗████████╗███████╗██████╗     ██████╗████████╗██╗  ██╗",
    r"  ██╔══██╗██╔════╝╚══██╔══╝╚══██╔══╝██╔════╝██╔══██╗   ██╔════╝╚══██╔══╝╚██╗██╔╝",
    r"  ██████╔╝█████╗     ██║      ██║   █████╗  ██████╔╝   ██║        ██║    ╚███╔╝ ",
    r"  ██╔══██╗██╔══╝     ██║      ██║   ██╔══╝  ██╔══██╗   ██║        ██║    ██╔██╗ ",
    r"  ██████╔╝███████╗   ██║      ██║   ███████╗██║  ██║   ╚██████╗   ██║   ██╔╝ ██╗",
    r"  ╚═════╝ ╚══════╝   ╚═╝      ╚═╝   ╚══════╝╚═╝  ╚═╝    ╚═════╝   ╚═╝   ╚═╝  ╚═╝",
];

const TAGLINE: &str = "The Intelligence Layer for AI Coding";

pub fn print_logo_animated() {
    let cfg = crate::core::config::Config::load();
    let t = crate::core::theme::load_theme(&cfg.theme);
    print_logo_animated_themed(&t);
}

pub fn print_logo_animated_themed(t: &crate::core::theme::Theme) {
    if crate::core::theme::no_color() {
        print_logo_plain();
        return;
    }
    if !io::stdout().is_terminal() {
        print_logo_themed_static(t);
        return;
    }

    let mut stdout = io::stdout();
    let frames = 28;
    let frame_ms = 45;
    let top_padding = 2;

    let _ = writeln!(stdout);
    let _ = writeln!(stdout);

    for frame in 0..frames {
        if frame > 0 {
            print!("\x1b[{}A", LOGO.len() + 2 + top_padding);
            for _ in 0..top_padding {
                let _ = writeln!(stdout);
            }
        }

        let wave_offset = frame as f64 / frames as f64;

        for (i, line) in LOGO.iter().enumerate() {
            let chars: Vec<char> = line.chars().collect();
            let max_j = chars.len().max(1) as f64;
            let mut buf = String::with_capacity(chars.len() * 20);

            for (j, ch) in chars.iter().enumerate() {
                if *ch == ' ' {
                    buf.push(' ');
                    continue;
                }
                let pos = j as f64 / max_j + i as f64 * 0.15;
                let blend = ((pos + wave_offset * 2.0) * std::f64::consts::PI)
                    .sin()
                    .mul_add(0.5, 0.5);
                let c = t.primary.lerp(&t.secondary, blend);
                buf.push_str(&c.fg());
                buf.push(*ch);
            }
            buf.push_str("\x1b[0m");
            let _ = writeln!(stdout, "{buf}");
        }

        let tag_blend = ((wave_offset * 2.0 + 1.0) * std::f64::consts::PI)
            .sin()
            .mul_add(0.5, 0.5);
        let tag_color = t.muted.lerp(&t.accent, tag_blend * 0.5);
        let _ = writeln!(stdout, "{}             {TAGLINE}\x1b[0m", tag_color.fg());
        let _ = writeln!(stdout);

        let _ = stdout.flush();
        std::thread::sleep(std::time::Duration::from_millis(frame_ms));
    }

    print!("\x1b[{}A", LOGO.len() + 2 + top_padding);
    print_logo_themed_static(t);
}

pub fn print_logo_static() {
    let cfg = crate::core::config::Config::load();
    let t = crate::core::theme::load_theme(&cfg.theme);
    print_logo_themed_static(&t);
}

fn print_logo_themed_static(t: &crate::core::theme::Theme) {
    if crate::core::theme::no_color() {
        print_logo_plain();
        return;
    }
    let mut stdout = io::stdout();

    let _ = writeln!(stdout);
    let _ = writeln!(stdout);

    for (i, line) in LOGO.iter().enumerate() {
        let chars: Vec<char> = line.chars().collect();
        let mut buf = String::with_capacity(chars.len() * 20);

        for (j, ch) in chars.iter().enumerate() {
            if *ch == ' ' {
                buf.push(' ');
                continue;
            }
            let progress = if chars.len() > 1 {
                j as f64 / (chars.len() - 1) as f64
            } else {
                0.5
            };
            let row_t = i as f64 / (LOGO.len() - 1).max(1) as f64;
            let blend = (progress + row_t * 0.3).min(1.0);
            let c = t.primary.lerp(&t.secondary, blend);
            buf.push_str(&c.fg());
            buf.push(*ch);
        }
        buf.push_str("\x1b[0m");
        let _ = writeln!(stdout, "{buf}");
    }

    let _ = writeln!(stdout, "{}             {TAGLINE}\x1b[0m", t.muted.fg());
    let _ = writeln!(stdout);
    let _ = stdout.flush();
}

fn print_logo_plain() {
    println!();
    println!();
    for line in &LOGO {
        println!("{line}");
    }
    println!("             {TAGLINE}");
    println!();
}

pub fn print_command_box() {
    use crate::core::theme;
    let cfg = crate::core::config::Config::load();
    let t = theme::load_theme(&cfg.theme);
    let d = theme::dim();
    let b = theme::bold();
    let r = theme::rst();
    let cmd = t.accent.fg();
    let ok = t.success.fg();
    let m = t.muted.fg();

    println!("  {d}┌─────────────────────────────────────────────────────────┐{r}");
    println!(
        "  {d}│{r}  {cmd}{b}better-ctx gain{r}        {m}Token savings dashboard{r}         {d}│{r}"
    );
    println!(
        "  {d}│{r}  {cmd}{b}better-ctx dashboard{r}   {m}Web analytics (browser){r}        {d}│{r}"
    );
    println!(
        "  {d}│{r}  {cmd}{b}better-ctx heatmap{r}     {m}Project context heat map{r}        {d}│{r}"
    );
    println!(
        "  {d}│{r}  {cmd}{b}better-ctx benchmark{r}   {m}Test compression quality{r}        {d}│{r}"
    );
    println!(
        "  {d}│{r}  {cmd}{b}better-ctx config{r}      {m}Edit settings{r}                   {d}│{r}"
    );
    println!(
        "  {d}│{r}  {cmd}{b}better-ctx doctor{r}      {m}Verify installation{r}             {d}│{r}"
    );
    println!(
        "  {d}│{r}  {cmd}{b}better-ctx update{r}      {m}Self-update to latest{r}           {d}│{r}"
    );
    println!("  {d}│{r}  {cmd}{b}better-ctx off{r} / {cmd}{b}on{r}    {m}Toggle compression{r}              {d}│{r}");
    println!(
        "  {d}│{r}  {cmd}{b}better-ctx report-issue{r} {m}Report a bug (auto-diagnostics){r} {d}│{r}"
    );
    println!("  {d}│{r}  {cmd}{b}better-ctx contribute{r}  {m}Share anonymized compression stats{r}{d}│{r}");
    println!(
        "  {d}│{r}  {cmd}{b}better-ctx uninstall{r}   {m}Clean removal{r}                   {d}│{r}"
    );
    println!("  {d}└─────────────────────────────────────────────────────────┘{r}");
    println!("  {ok}Ready!{r} Your next AI command will be automatically optimized.");
    println!("  {d}Docs: https://betterctx.com/docs{r}");
    println!();
}

pub fn print_step_header(step: u8, total: u8, title: &str) {
    let dim = "\x1b[2m";
    let bold = "\x1b[1m";
    let cyan = "\x1b[36m";
    let rst = "\x1b[0m";
    println!();
    println!("  {cyan}{bold}[{step}/{total}]{rst} {bold}{title}{rst}");
    println!("  {dim}─────────────────────────────────────────────────────{rst}");
}

pub fn print_status_ok(msg: &str) {
    println!("  \x1b[32m✓\x1b[0m {msg}");
}

pub fn print_status_skip(msg: &str) {
    println!("  \x1b[2m○\x1b[0m \x1b[2m{msg}\x1b[0m");
}

pub fn print_status_new(msg: &str) {
    println!("  \x1b[1;32m✓\x1b[0m \x1b[1m{msg}\x1b[0m");
}

pub fn print_status_warn(msg: &str) {
    println!("  \x1b[33m⚠\x1b[0m {msg}");
}

pub fn spinner_tick(msg: &str, frame: usize) {
    let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let ch = frames[frame % frames.len()];
    print!("\r  \x1b[36m{ch}\x1b[0m {msg}");
    let _ = io::stdout().flush();
}

pub fn spinner_done(msg: &str) {
    print!("\r  \x1b[32m✓\x1b[0m {msg}\x1b[K\n");
    let _ = io::stdout().flush();
}

pub fn print_setup_header() {
    let dim = "\x1b[2m";
    let bold = "\x1b[1m";
    let green = "\x1b[32m";
    let rst = "\x1b[0m";
    println!();
    println!("  {dim}╭──────────────────────────────────────────╮{rst}");
    println!(
        "  {dim}│{rst}  {green}{bold}◆ better-ctx setup{rst}                         {dim}│{rst}"
    );
    println!("  {dim}│{rst}  {dim}Configuring your development environment{rst} {dim}│{rst}");
    println!("  {dim}╰──────────────────────────────────────────╯{rst}");
    println!();
}
