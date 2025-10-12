use console::{Emoji, style};

// pub static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
// pub static TRUCK: Emoji<'_, '_> = Emoji("🚚  ", "");
// pub static CLIP: Emoji<'_, '_> = Emoji("🔗  ", "");
// pub static PAPER: Emoji<'_, '_> = Emoji("📃  ", "");
pub static SPARKLE: Emoji<'_, '_> = Emoji("✨  ", "");

pub fn print_welcome() {
    // println!("{}", style("").clear());
    println!(
        "{}",
        style("╔══════════════════════════════════════╗").cyan()
    );
    println!(
        "{}",
        style("║            🐳 PEEKO CLI              ║").cyan()
    );
    println!(
        "{}",
        style("║      Container Image Explorer        ║").cyan()
    );
    println!(
        "{}",
        style("╚══════════════════════════════════════╝").cyan()
    );
    println!();
    println!("{SPARKLE} Welcome to Peeko - the interactive container image explorer!",);
}

pub fn print_success(message: &str) {
    println!("{} {}", style("✅").green(), style(message).green());
}

pub fn print_error(message: &str) {
    println!("{} {}", style("❌").red(), style(message).red());
}

pub fn print_info(message: &str) {
    println!("{} {}", style("ℹ️").blue(), style(message).blue());
}

pub fn print_warning(message: &str) {
    println!("{} {}", style("⚠️").yellow(), style(message).yellow());
}

pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

pub fn print_separator() {
    println!("{}", style("─".repeat(60)).dim());
}

pub fn print_header(title: &str) {
    println!();
    print_separator();
    println!("{}", style(title).bold().cyan());
    print_separator();
}
