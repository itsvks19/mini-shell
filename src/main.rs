use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use colored::{Color, Colorize};
use is_executable::IsExecutable;
use is_root::is_root;

const SHELL_NAME: &str = "mini-shell";
const VERSION: &str = env!("CARGO_PKG_VERSION");

struct PackageManager {
    name: &'static str,
    install_cmd: &'static str,
    search_cmd: &'static str,
    update_cmd: &'static str,
    is_available: fn() -> bool,
    platform: Platform,
}

#[derive(PartialEq)]
enum Platform {
    Windows,
    Linux,
    MacOS,
    Any,
}

fn get_current_platform() -> Platform {
    if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "linux") {
        Platform::Linux
    } else if cfg!(target_os = "macos") {
        Platform::MacOS
    } else {
        Platform::Any
    }
}

fn command_exists(command: &str) -> bool {
    match get_current_platform() {
        // On Windows we need to add .exe extension
        Platform::Windows => Command::new("where")
            .arg(command)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false),
        _ => Command::new("which")
            .arg(command)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false),
    }
}

fn get_platform_name(platform: &Platform) -> String {
    match platform {
        Platform::Windows => "Windows".to_string(),
        Platform::Linux => "Linux".to_string(),
        Platform::MacOS => "MacOS".to_string(),
        Platform::Any => "Any".to_string(),
    }
}

fn package_managers() -> Vec<PackageManager> {
    vec![
        // Windows package managers
        PackageManager {
            name: "chocolatey",
            install_cmd: "choco install",
            search_cmd: "choco search",
            update_cmd: "choco upgrade",
            is_available: || command_exists("choco"),
            platform: Platform::Windows,
        },
        PackageManager {
            name: "winget",
            install_cmd: "winget install",
            search_cmd: "winget search",
            update_cmd: "winget upgrade",
            is_available: || command_exists("winget"),
            platform: Platform::Windows,
        },
        PackageManager {
            name: "scoop",
            install_cmd: "scoop install",
            search_cmd: "scoop search",
            update_cmd: "scoop update",
            is_available: || command_exists("scoop"),
            platform: Platform::Windows,
        },
        // macOS package managers
        PackageManager {
            name: "homebrew",
            install_cmd: "brew install",
            search_cmd: "brew search",
            update_cmd: "brew upgrade",
            is_available: || command_exists("brew"),
            platform: Platform::MacOS,
        },
        PackageManager {
            name: "macports",
            install_cmd: "port install",
            search_cmd: "port search",
            update_cmd: "port upgrade",
            is_available: || command_exists("port"),
            platform: Platform::MacOS,
        },
        // Linux package managers
        PackageManager {
            name: "apt",
            install_cmd: "apt install",
            search_cmd: "apt search",
            update_cmd: "apt upgrade",
            is_available: || command_exists("apt"),
            platform: Platform::Linux,
        },
        PackageManager {
            name: "dnf",
            install_cmd: "dnf install",
            search_cmd: "dnf search",
            update_cmd: "dnf upgrade",
            is_available: || command_exists("dnf"),
            platform: Platform::Linux,
        },
        PackageManager {
            name: "pacman",
            install_cmd: "pacman -S",
            search_cmd: "pacman -Ss",
            update_cmd: "pacman -Syu",
            is_available: || command_exists("pacman"),
            platform: Platform::Linux,
        },
        PackageManager {
            name: "zypper",
            install_cmd: "zypper install",
            search_cmd: "zypper search",
            update_cmd: "zypper update",
            is_available: || command_exists("zypper"),
            platform: Platform::Linux,
        },
        // Cross-platform package managers
        PackageManager {
            name: "snap",
            install_cmd: "snap install",
            search_cmd: "snap find",
            update_cmd: "snap refresh",
            is_available: || command_exists("snap"),
            platform: Platform::Any,
        },
        PackageManager {
            name: "flatpak",
            install_cmd: "flatpak install",
            search_cmd: "flatpak search",
            update_cmd: "flatpak update",
            is_available: || command_exists("flatpak"),
            platform: Platform::Any,
        },
    ]
}

fn main() -> io::Result<()> {
    println!(
        "{} {}{}",
        SHELL_NAME.green(),
        "v".bright_blue(),
        VERSION.bright_blue()
    );

    let current_platform = get_current_platform();
    println!(
        "{} {}",
        "Platform:".bright_cyan(),
        get_platform_name(&current_platform).color(Color::Cyan)
    );
    println!(
        "{}",
        "Type 'help' for available commands, 'exit' to quit\n".bright_white()
    );

    let package_managers = package_managers();
    let mut current_dir = env::current_dir()?;

    loop {
        print!(
            "{}{} ",
            current_dir.display().to_string().cyan(),
            ">".yellow()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];
        let args = &parts[1..];

        match command {
            "exit" | "quit" => break,
            "help" => display_help(),
            "cd" => change_directory(&mut current_dir, args),
            "pwd" => println!("{}", current_dir.display()),
            "ls" => list_directory(&current_dir, args),
            "mkdir" => make_directory(&current_dir, args),
            "rm" => remove_file_or_directory(&current_dir, args),
            "cat" => cat_file(&current_dir, args),
            "echo" => echo(args),
            "touch" => touch_file(&current_dir, args),
            "clear" => {
                if cfg!(target_os = "windows") {
                    // On Windows
                    let _ = Command::new("cmd").args(["/C", "cls"]).status();
                } else {
                    // On Unix, use ANSI escape codes
                    print!("\x1B[2J\x1B[1;1H");
                    io::stdout().flush()?;
                }
            }
            "pkg" | "package" => handle_package_command(&package_managers, args, &current_platform),
            _ => execute_command(input, &current_dir),
        }
    }

    Ok(())
}

fn display_help() {
    println!("Available commands:");
    println!("  cd <dir>       - Change directory");
    println!("  pwd            - Print working directory");
    println!("  ls [dir]       - List directory contents");
    println!("  mkdir <dir>    - Create directory");
    println!("  rm <file/dir>  - Remove file or directory");
    println!("  cat <file>     - Display file contents");
    println!("  echo <text>    - Display text");
    println!("  touch <file>   - Create empty file");
    println!("  clear          - Clear screen");
    println!("  pkg            - Package management commands:");
    println!("     pkg install <package>  - Install a package");
    println!("     pkg search <query>     - Search for packages");
    println!("     pkg update [package]   - Update packages");
    println!("     pkg list               - List available package managers");
    println!("  help           - Display this help");
    println!("  exit           - Exit the shell");
    println!("");
    println!("You can also execute any system command");
}

fn change_directory(current_dir: &mut PathBuf, args: &[&str]) {
    if args.is_empty() {
        // Go to home directory if no args
        if let Some(home_dir) = dirs::home_dir() {
            *current_dir = home_dir;
        } else {
            println!("{}", "Could not determine home directory".red());
        }
        return;
    }

    // Handle "~" for home directory (Unix convention but nice to have on all platforms)
    let path = if args[0] == "~" || args[0].starts_with("~/") {
        if let Some(home_dir) = dirs::home_dir() {
            if args[0] == "~" {
                home_dir
            } else {
                // Remove the ~ and join with home
                home_dir.join(&args[0][2..])
            }
        } else {
            println!("{}", "Could not determine home directory".red());
            return;
        }
    } else if args[0].starts_with('/') || args[0].starts_with('\\') || args[0].contains(':') {
        // Absolute path
        Path::new(args[0]).to_path_buf()
    } else {
        // Relative path
        current_dir.join(args[0])
    };

    if let Err(e) = env::set_current_dir(&path) {
        println!("cd: {}: {}", args[0].red(), e.to_string().bright_red());
    } else {
        *current_dir = env::current_dir().unwrap_or_else(|_| path);
    }
}

fn list_directory(current_dir: &PathBuf, args: &[&str]) {
    let target_dir = if args.is_empty() {
        current_dir.clone()
    } else {
        // Resolve ~ to home directory
        if args[0] == "~" || args[0].starts_with("~/") {
            if let Some(home_dir) = dirs::home_dir() {
                if args[0] == "~" {
                    home_dir
                } else {
                    // Remove the ~ and join with home
                    home_dir.join(&args[0][2..])
                }
            } else {
                println!("{}", "Could not determine home directory".red());
                return;
            }
        } else if args[0].starts_with('/') || args[0].starts_with('\\') || args[0].contains(':') {
            Path::new(args[0]).to_path_buf()
        } else {
            current_dir.join(args[0])
        }
    };

    match fs::read_dir(&target_dir) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        let file_name = path.file_name().unwrap_or_default();
                        let metadata = fs::metadata(&path).unwrap();

                        if metadata.is_dir() {
                            println!("{}/", file_name.to_string_lossy().bright_blue());
                        } else if path.is_executable() {
                            // Executable files (highlighted in green)
                            println!("{}", file_name.to_string_lossy().bright_green());
                        } else {
                            println!("{}", file_name.to_string_lossy());
                        }
                    }
                    Err(e) => println!("Error reading entry: {}", e),
                }
            }
        }
        Err(e) => println!("ls: cannot access '{}': {}", target_dir.display(), e),
    }
}

fn make_directory(current_dir: &PathBuf, args: &[&str]) {
    if args.is_empty() {
        println!("mkdir: missing operand");
        return;
    }

    for dir_name in args {
        let path = if dir_name.to_string() == "~" || dir_name.starts_with("~/") {
            if let Some(home_dir) = dirs::home_dir() {
                if dir_name.to_string() == "~" {
                    home_dir
                } else {
                    // Remove the ~ and join with home
                    home_dir.join(&dir_name[2..])
                }
            } else {
                println!("Could not determine home directory");
                continue;
            }
        } else if dir_name.starts_with('/') || dir_name.starts_with('\\') || dir_name.contains(':')
        {
            Path::new(dir_name).to_path_buf()
        } else {
            current_dir.join(dir_name)
        };

        // Using create_dir_all for recursive creation
        if let Err(e) = fs::create_dir_all(&path) {
            println!("mkdir: cannot create directory '{}': {}", dir_name, e);
        }
    }
}

fn remove_file_or_directory(current_dir: &PathBuf, args: &[&str]) {
    if args.is_empty() {
        println!("rm: missing operand");
        return;
    }

    let mut recursive = false;
    let mut force = false;
    let mut targets: Vec<&str> = Vec::new();

    for arg in args {
        match *arg {
            "-r" | "-R" | "--recursive" => recursive = true,
            "-f" | "--force" => force = true,
            _ if arg.starts_with('-') => {
                // Handle combined flags like -rf
                if arg.contains('r') || arg.contains('R') {
                    recursive = true;
                }
                if arg.contains('f') {
                    force = true;
                }
            }
            _ => targets.push(*arg),
        }
    }

    if targets.is_empty() {
        println!("rm: missing operand");
        return;
    }

    for target in targets {
        let path = if target == "~" || target.starts_with("~/") {
            if let Some(home_dir) = dirs::home_dir() {
                if target == "~" {
                    home_dir
                } else {
                    // Remove the ~ and join with home
                    home_dir.join(&target[2..])
                }
            } else {
                println!("Could not determine home directory");
                continue;
            }
        } else if target.starts_with('/') || target.starts_with('\\') || target.contains(':') {
            Path::new(target).to_path_buf()
        } else {
            current_dir.join(target)
        };

        let metadata = match fs::metadata(&path) {
            Ok(meta) => meta,
            Err(e) => {
                if !force {
                    println!("rm: cannot remove '{}': {}", target, e);
                }
                continue;
            }
        };

        let result = if metadata.is_dir() {
            if recursive {
                fs::remove_dir_all(&path)
            } else {
                println!("rm: cannot remove '{}': Is a directory", target);
                continue;
            }
        } else {
            fs::remove_file(&path)
        };

        if let Err(e) = result {
            if !force {
                println!("rm: cannot remove '{}': {}", target, e);
            }
        }
    }
}

fn cat_file(current_dir: &PathBuf, args: &[&str]) {
    if args.is_empty() {
        println!("cat: missing operand");
        return;
    }

    for file_name in args {
        let path = if *file_name == "~" || file_name.starts_with("~/") {
            if let Some(home_dir) = dirs::home_dir() {
                if *file_name == "~" {
                    home_dir
                } else {
                    // Remove the ~ and join with home
                    home_dir.join(&file_name[2..])
                }
            } else {
                println!("Could not determine home directory");
                continue;
            }
        } else if file_name.starts_with('/')
            || file_name.starts_with('\\')
            || file_name.contains(':')
        {
            Path::new(file_name).to_path_buf()
        } else {
            current_dir.join(file_name)
        };

        match fs::read_to_string(&path) {
            Ok(content) => print!("{}", content),
            Err(e) => println!("cat: {}: {}", file_name, e),
        }
    }
}

fn echo(args: &[&str]) {
    println!("{}", args.join(" "));
}

fn touch_file(current_dir: &PathBuf, args: &[&str]) {
    if args.is_empty() {
        println!("touch: missing operand");
        return;
    }

    for file_name in args {
        let path = if *file_name == "~" || file_name.starts_with("~/") {
            if let Some(home_dir) = dirs::home_dir() {
                if *file_name == "~" {
                    home_dir
                } else {
                    // Remove the ~ and join with home
                    home_dir.join(&file_name[2..])
                }
            } else {
                println!("Could not determine home directory");
                continue;
            }
        } else if file_name.starts_with('/')
            || file_name.starts_with('\\')
            || file_name.contains(':')
        {
            Path::new(file_name).to_path_buf()
        } else {
            current_dir.join(file_name)
        };

        // Open the file in write mode, which will create it if it doesn't exist
        // and do nothing if it does exist (effectively "touching" it)
        if let Err(e) = fs::OpenOptions::new().write(true).create(true).open(&path) {
            println!("touch: cannot touch '{}': {}", file_name, e);
        }
    }
}

fn handle_package_command(
    package_managers: &[PackageManager],
    args: &[&str],
    current_platform: &Platform,
) {
    if args.is_empty() {
        println!("Usage: pkg <command> [arguments]");
        println!("Commands: install, search, update, list");
        return;
    }

    match args[0] {
        "install" | "i" => {
            if args.len() < 2 {
                println!("Usage: pkg install <package>");
                return;
            }
            let package = args[1];
            install_package(package_managers, package, current_platform);
        }
        "search" | "s" => {
            if args.len() < 2 {
                println!("Usage: pkg search <query>");
                return;
            }
            let query = args[1];
            search_packages(package_managers, query, current_platform);
        }
        "update" | "u" | "upgrade" => {
            let package = if args.len() > 1 { Some(args[1]) } else { None };
            update_packages(package_managers, package, current_platform);
        }
        "list" | "ls" => {
            list_package_managers(package_managers, current_platform);
        }
        _ => {
            println!("Unknown package command: {}", args[0]);
            println!("Available commands: install, search, update, list");
        }
    }
}

fn list_package_managers(package_managers: &[PackageManager], current_platform: &Platform) {
    println!("Available package managers for your platform:");

    for pm in package_managers {
        if pm.platform == *current_platform || pm.platform == Platform::Any {
            let available = (pm.is_available)();
            let status = if available {
                "installed"
            } else {
                "not installed"
            };
            println!("  {} ({})", pm.name, status);
        }
    }
}

fn install_package(
    package_managers: &[PackageManager],
    package: &str,
    current_platform: &Platform,
) {
    let mut installed = false;

    let platform_pms: Vec<&PackageManager> = package_managers
        .iter()
        .filter(|pm| pm.platform == *current_platform || pm.platform == Platform::Any)
        .collect();

    for pm in platform_pms {
        if (pm.is_available)() {
            println!("Attempting to install {} using {}...", package, pm.name);

            let mut cmd_parts = pm.install_cmd.split_whitespace().collect::<Vec<&str>>();
            cmd_parts.push(package);

            if let Some(cmd_name) = cmd_parts.first() {
                let mut cmd = Command::new(cmd_name);
                cmd.args(&cmd_parts[1..])
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .stdin(Stdio::inherit());

                // On Linux/macOS, use sudo for system package managers if running as non-root
                if *current_platform == Platform::Linux
                    || *current_platform == Platform::MacOS
                        && ["apt", "dnf", "pacman", "zypper", "port"].contains(&pm.name)
                {
                    let is_root = is_root();

                    if !is_root {
                        let mut sudo_cmd = Command::new("sudo");
                        sudo_cmd
                            .arg(cmd_name)
                            .args(&cmd_parts[1..])
                            .stdout(Stdio::inherit())
                            .stderr(Stdio::inherit())
                            .stdin(Stdio::inherit());

                        match sudo_cmd.status() {
                            Ok(status) => {
                                if status.success() {
                                    installed = true;
                                    println!(
                                        "Successfully installed {} using {}",
                                        package, pm.name
                                    );
                                    break;
                                }
                            }
                            Err(e) => println!("Failed to execute sudo {}: {}", pm.name, e),
                        }
                        continue;
                    }
                }

                match cmd.status() {
                    Ok(status) => {
                        if status.success() {
                            installed = true;
                            println!("Successfully installed {} using {}", package, pm.name);
                            break;
                        }
                    }
                    Err(e) => println!("Failed to execute {}: {}", pm.name, e),
                }
            }
        }
    }

    if !installed {
        println!(
            "Failed to install {}. No compatible package manager found or installation failed.",
            package
        );

        match current_platform {
            Platform::Windows => {
                println!(
                    "You may need to install a package manager first (chocolatey, winget, or scoop)."
                );
            }
            Platform::MacOS => {
                println!("You may need to install a package manager first (homebrew or macports).");
            }
            Platform::Linux => {
                println!(
                    "Your distribution's package manager might not be supported or you may need to run with sudo privileges."
                );
            }
            _ => {
                println!("Please install a package manager appropriate for your platform.");
            }
        }
    }
}

fn search_packages(package_managers: &[PackageManager], query: &str, current_platform: &Platform) {
    let mut found = false;

    let platform_pms: Vec<&PackageManager> = package_managers
        .iter()
        .filter(|pm| pm.platform == *current_platform || pm.platform == Platform::Any)
        .collect();

    for pm in platform_pms {
        if (pm.is_available)() {
            println!("Searching for '{}' using {}...", query, pm.name);

            let mut cmd_parts = pm.search_cmd.split_whitespace().collect::<Vec<&str>>();
            cmd_parts.push(query);

            if let Some(cmd_name) = cmd_parts.first() {
                let mut cmd = Command::new(cmd_name);
                cmd.args(&cmd_parts[1..])
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .stdin(Stdio::inherit());

                match cmd.status() {
                    Ok(_) => {
                        found = true;
                    }
                    Err(e) => println!("Failed to search with {}: {}", pm.name, e),
                }
            }
        }
    }

    if !found {
        println!("No compatible package manager found for searching.");

        match current_platform {
            Platform::Windows => {
                println!(
                    "You may need to install a package manager first (chocolatey, winget, or scoop)."
                );
            }
            Platform::MacOS => {
                println!("You may need to install a package manager first (homebrew or macports).");
            }
            Platform::Linux => {
                println!("Your distribution's package manager might not be supported.");
            }
            _ => {
                println!("Please install a package manager appropriate for your platform.");
            }
        }
    }
}

fn update_packages(
    package_managers: &[PackageManager],
    package: Option<&str>,
    current_platform: &Platform,
) {
    let mut updated = false;

    let platform_pms: Vec<&PackageManager> = package_managers
        .iter()
        .filter(|pm| pm.platform == *current_platform || pm.platform == Platform::Any)
        .collect();

    for pm in platform_pms {
        if (pm.is_available)() {
            if let Some(pkg) = package {
                println!("Updating {} using {}...", pkg, pm.name);

                let mut cmd_parts = pm.update_cmd.split_whitespace().collect::<Vec<&str>>();
                cmd_parts.push(pkg);

                if let Some(cmd_name) = cmd_parts.first() {
                    if (*current_platform == Platform::Linux
                        || *current_platform == Platform::MacOS)
                        && ["apt", "dnf", "pacman", "zypper", "port"].contains(&pm.name)
                    {
                        let is_root = is_root();

                        if !is_root {
                            let mut sudo_cmd = Command::new("sudo");
                            sudo_cmd
                                .arg(cmd_name)
                                .args(&cmd_parts[1..])
                                .stdout(Stdio::inherit())
                                .stdin(Stdio::inherit())
                                .stderr(Stdio::inherit());

                            match sudo_cmd.status() {
                                Ok(status) => {
                                    if status.success() {
                                        updated = true;
                                        println!("Successfully updated {} using {}", pkg, pm.name);
                                        break;
                                    }
                                }
                                Err(e) => println!("Failed to execute sudo {}: {}", pm.name, e),
                            }
                            continue;
                        }
                    }

                    let mut cmd = Command::new(cmd_name);
                    cmd.args(&cmd_parts[1..])
                        .stdout(Stdio::inherit())
                        .stdin(Stdio::inherit())
                        .stderr(Stdio::inherit());

                    match cmd.status() {
                        Ok(status) => {
                            if status.success() {
                                updated = true;
                                println!("Successfully updated {} using {}", pkg, pm.name);
                                break;
                            }
                        }
                        Err(e) => println!("Failed to update with {}: {}", pm.name, e),
                    }
                }
            } else {
                println!("Updating all packages using {}...", pm.name);

                let cmd_parts = pm.update_cmd.split_whitespace().collect::<Vec<&str>>();

                if let Some(cmd_name) = cmd_parts.first() {
                    if (*current_platform == Platform::Linux
                        || *current_platform == Platform::MacOS)
                        && ["apt", "dnf", "pacman", "zypper", "port"].contains(&pm.name)
                    {
                        let is_root = is_root();

                        if !is_root {
                            let mut sudo_cmd = Command::new("sudo");
                            sudo_cmd
                                .arg(cmd_name)
                                .args(&cmd_parts[1..])
                                .stdout(Stdio::inherit())
                                .stdin(Stdio::inherit())
                                .stderr(Stdio::inherit());

                            match sudo_cmd.status() {
                                Ok(_) => {
                                    updated = true;
                                }
                                Err(e) => println!("Failed to execute sudo {}: {}", pm.name, e),
                            }
                            continue;
                        }
                    }
                    let mut cmd = Command::new(cmd_name);
                    cmd.args(&cmd_parts[1..]);
                    cmd.stdout(Stdio::inherit());
                    cmd.stdin(Stdio::inherit());
                    cmd.stderr(Stdio::inherit());

                    match cmd.status() {
                        Ok(_) => {
                            updated = true;
                        }
                        Err(e) => println!("Failed to update with {}: {}", pm.name, e),
                    }
                }
            }
        }
    }

    if !updated {
        if let Some(pkg) = package {
            println!(
                "Failed to update {}. No compatible package manager found or update failed.",
                pkg
            );
        } else {
            println!(
                "Failed to update packages. No compatible package manager found or update failed."
            );
        }

        match current_platform {
            Platform::Windows => {
                println!(
                    "You may need to install a package manager first (chocolatey, winget, or scoop)."
                );
            }
            Platform::MacOS => {
                println!("You may need to install a package manager first (homebrew or macports).");
            }
            Platform::Linux => {
                println!(
                    "Your distribution's package manager might not be supported or you may need to run with sudo privileges."
                );
            }
            _ => {
                println!("Please install a package manager appropriate for your platform.");
            }
        }
    }
}

fn execute_command(command: &str, current_dir: &PathBuf) {
    let shell = if get_current_platform() == Platform::Windows {
        "cmd"
    } else {
        "sh"
    };

    let shell_flag = if get_current_platform() == Platform::Windows {
        "/C"
    } else {
        "-c"
    };

    let status = Command::new(shell)
        .arg(shell_flag)
        .arg(command)
        .current_dir(current_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .status();

    match status {
        Ok(exit_status) => {
            if !exit_status.success() {
                if let Some(code) = exit_status.code() {
                    println!("Command exited with non-zero status code: {}", code);
                } else {
                    println!("Command terminated by signal");
                }
            }
        }
        Err(e) => {
            println!("Failed to execute command: {}", e);
        }
    }
}
