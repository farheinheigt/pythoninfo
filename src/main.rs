use std::env;
use std::io::IsTerminal;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err((message, code)) => {
            eprintln!("{message}");
            ExitCode::from(code)
        }
    }
}

fn run() -> Result<ExitCode, (String, u8)> {
    let args: Vec<String> = env::args().skip(1).collect();
    let parsed = parse_args(&args)?;
    let styles = Styles::detect();
    let python_bin = detect_python_bin();

    match parsed {
        ParsedArgs::Help => {
            print_usage();
            Ok(ExitCode::SUCCESS)
        }
        ParsedArgs::Run { package } => {
            if let Some(package) = package {
                run_package_mode(&styles, python_bin.as_deref(), &package)
            } else {
                run_full_mode(&styles, python_bin.as_deref())
            }
        }
    }
}

enum ParsedArgs {
    Help,
    Run { package: Option<String> },
}

fn parse_args(args: &[String]) -> Result<ParsedArgs, (String, u8)> {
    match args {
        [] => Ok(ParsedArgs::Run { package: None }),
        [arg] if arg == "-h" || arg == "--help" || arg == "help" => Ok(ParsedArgs::Help),
        [arg] if arg.starts_with('-') => Err((
            format!("Option inconnue: {arg}\nUtilise: pythoninfo --help"),
            2,
        )),
        [package] => Ok(ParsedArgs::Run {
            package: Some(package.clone()),
        }),
        _ => Err(("Utilise: pythoninfo --help".to_string(), 2)),
    }
}

fn print_usage() {
    println!(
        "\
Usage:
  pythoninfo
  pythoninfo <package>

Description:
  - pythoninfo            : diagnostic complet de l environnement Python.
  - pythoninfo <package>  : infos sur un paquet Python (pip show + metadonnees).
  - TAB apres \"pythoninfo \" propose les paquets Python via completion/fzf-tab."
    );
}

fn run_package_mode(styles: &Styles, python_bin: Option<&str>, package: &str) -> Result<ExitCode, (String, u8)> {
    let py = python_bin.ok_or_else(|| {
        (
            "Aucun interpreteur Python detecte (python3/python).".to_string(),
            1,
        )
    })?;

    println!();
    println!(
        "🐍 {}PYTHON{} – INFOS PAQUET : {}{}{}",
        styles.title, styles.reset, styles.value, package, styles.reset
    );
    println!("══════════════════════════════════════════════");

    print_section(styles, "1) CONTEXTE");
    print_item(styles, &format!("Interpreteur : {py}"));
    print_item(
        styles,
        &format!(
            "Executable   : {}",
            command_path(py).unwrap_or_else(|| py.to_string())
        ),
    );

    print_section(styles, "2) PIP SHOW");
    let pip_show = command_output(py, &["-m", "pip", "show", package]).unwrap_or_default();
    if pip_show.trim().is_empty() {
        println!(
            "  • {}Paquet introuvable via pip{}: {}{}{}",
            styles.neg, styles.dim, styles.value, package, styles.reset
        );
    } else {
        for line in pip_show.lines() {
            if let Some((key, value)) = line.split_once(':') {
                println!(
                    "  • {}{:<12}{}: {}{}{}",
                    styles.key,
                    key.trim(),
                    styles.dim,
                    styles.value,
                    value.trim(),
                    styles.reset
                );
            } else {
                println!("  • {}{}{}", styles.value, line, styles.reset);
            }
        }
    }

    print_section(styles, "3) BINAIRES EXPOSES (console_scripts)");
    let console_scripts = python_console_scripts(py, package);
    if console_scripts.is_empty() {
        println!("  • {}Aucun console_script detecte{}", styles.dim, styles.reset);
    } else {
        for item in console_scripts {
            println!("  • {}{}{}", styles.value, item, styles.reset);
        }
    }

    print_section(styles, "4) ORIGINE POSSIBLE (pipx/uv)");
    let mut found = false;
    if command_exists("pipx") {
        let names = command_output("pipx", &["list", "--short"]).unwrap_or_default();
        if names
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .any(|name| name.eq_ignore_ascii_case(package))
        {
            print_item(styles, "pipx    : installe");
            found = true;
        }
    }
    if command_exists("uv") {
        let names = command_output("uv", &["tool", "list"]).unwrap_or_default();
        if names
            .lines()
            .filter(|line| !line.trim_start().starts_with('-'))
            .filter_map(|line| line.split_whitespace().next())
            .any(|name| name.eq_ignore_ascii_case(package))
        {
            print_item(styles, "uv tool : installe");
            found = true;
        }
    }
    if !found {
        println!(
            "  • {}non detectee (installation pip classique probable){}",
            styles.dim, styles.reset
        );
    }

    Ok(ExitCode::SUCCESS)
}

fn run_full_mode(styles: &Styles, python_bin: Option<&str>) -> Result<ExitCode, (String, u8)> {
    let user = env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    let (os_name, os_version, os_arch) = detect_system_context();

    println!();
    println!(
        "🐍 {}PYTHON{} – ÉTAT DE L’ENVIRONNEMENT : {}{}{}",
        styles.title, styles.reset, styles.value, user, styles.reset
    );
    println!("{}══════════════════════════════════════════════{}", styles.dim, styles.reset);

    print_section(styles, "1) CONTEXTE SYSTÈME");
    print_item(styles, &format!("OS : {os_name} {os_version} ({os_arch})"));
    print_item(
        styles,
        &format!(
            "Shell : {}",
            env::var("SHELL").unwrap_or_else(|_| "unknown".to_string())
        ),
    );

    print_section(styles, "2) BINAIRES PYTHON / PIP DANS LE PATH");
    for name in ["python3", "python", "pip3", "pip"] {
        let lines = shell_type_lines(name);
        if lines.is_empty() {
            print_type_line(styles, &format!("{name} : non trouvé"));
        } else {
            for line in lines {
                print_type_line(styles, &line);
            }
        }
    }

    print_section(styles, "3) INTERPRÉTEUR PYTHON ACTIF");
    if let Some(py) = python_bin {
        println!(
            "  • {}{}{} → {}{}{}",
            styles.key,
            py,
            styles.dim,
            styles.value,
            command_path(py).unwrap_or_else(|| py.to_string()),
            styles.reset
        );
        println!(
            "  • Version : {}",
            command_output(py, &["-V"]).unwrap_or_else(|| "unknown".to_string())
        );
        for line in python_runtime_details(py) {
            print_item(styles, &line);
        }
    } else {
        print_item(styles, "Aucun binaire Python détecté");
    }

    print_section(styles, "4) PIP (GESTIONNAIRE DE PACKAGES)");
    if let Some(py) = python_bin {
        if command_success(py, &["-m", "pip", "-V"]) {
            if let Some(line) = command_output(py, &["-m", "pip", "-V"]) {
                println!("  • {}{}{}", styles.value, line, styles.reset);
            }
            let pip_conf = command_output(py, &["-m", "pip", "config", "list"]).unwrap_or_default();
            if pip_conf.trim().is_empty() {
                print_item(styles, "Configuration pip : aucune");
            } else {
                for line in pip_conf.lines() {
                    println!("  • {}{}{}", styles.value, line, styles.reset);
                }
            }
        } else {
            print_item(styles, "pip indisponible");
        }
    } else {
        print_item(styles, "pip indisponible");
    }

    print_section(styles, "5) VARIABLES D’ENVIRONNEMENT");
    for key in [
        "VIRTUAL_ENV",
        "CONDA_PREFIX",
        "CONDA_DEFAULT_ENV",
        "PYENV_VERSION",
        "PYENV_ROOT",
        "PIP_REQUIRE_VIRTUALENV",
        "PIP_CONFIG_FILE",
        "PYTHONPATH",
    ] {
        let value = env::var(key).unwrap_or_default();
        let rendered = if value.is_empty() { "(vide)" } else { &value };
        println!(
            "  • {}{}{}={}{}{}",
            styles.key, key, styles.dim, styles.value, rendered, styles.reset
        );
    }
    let path_entries = env::var("PATH").unwrap_or_default();
    let interesting = path_entries
        .split(':')
        .filter(|entry| {
            let lower = entry.to_ascii_lowercase();
            ["python", "pyenv", "conda", "venv", "virtualenv", "pip", "pipx"]
                .iter()
                .any(|needle| lower.contains(needle))
        })
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if interesting.is_empty() {
        print_item(styles, "PATH : aucun élément python détecté");
    } else {
        for entry in interesting {
            print_item(styles, &format!("PATH : {entry}"));
        }
    }

    print_section(styles, "6) GESTIONNAIRES / DISTRIBUTIONS");
    println!("  {}[A] Versions / environnements Python{}", styles.title, styles.reset);
    if command_exists("pyenv") {
        let pyenv_ver = command_output("pyenv", &["--version"]).unwrap_or_else(|| "unknown".to_string());
        let pyenv_root = command_output("pyenv", &["root"]).unwrap_or_else(|| "unknown".to_string());
        let pyenv_active = command_output("pyenv", &["version-name"]).unwrap_or_else(|| "unknown".to_string());
        let pyenv_versions = command_output("pyenv", &["versions", "--bare"]).unwrap_or_default();
        let pyenv_count = pyenv_versions.lines().filter(|line| !line.trim().is_empty()).count();
        print_item(
            styles,
            &format!(
                "pyenv : {} (actif={}, versions={}, root={}, binaires globaux=indirect via shims)",
                pyenv_ver.strip_prefix("pyenv ").unwrap_or(&pyenv_ver),
                pyenv_active,
                pyenv_count,
                pyenv_root
            ),
        );
    } else {
        print_item(styles, "pyenv : non installe");
    }
    if command_exists("conda") {
        let conda_ver = command_output("conda", &["--version"]).unwrap_or_else(|| "unknown".to_string());
        let conda_active = env::var("CONDA_DEFAULT_ENV").unwrap_or_else(|_| "aucun".to_string());
        print_item(
            styles,
            &format!(
                "conda : {} (env actif={}, binaires globaux=oui si env active)",
                conda_ver.strip_prefix("conda ").unwrap_or(&conda_ver),
                conda_active
            ),
        );
    } else {
        print_item(styles, "conda : non installe");
    }

    println!();
    println!(
        "  {}[B] Outils CLI globaux (installent des executables){}",
        styles.title, styles.reset
    );
    if command_exists("pipx") {
        let pipx_ver = command_output("pipx", &["--version"]).unwrap_or_else(|| "unknown".to_string());
        let pipx_short = command_output("pipx", &["list", "--short"]).unwrap_or_default();
        let pipx_pkg_count = pipx_short.lines().filter(|line| !line.trim().is_empty()).count();
        let pipx_bin_dir = command_output("pipx", &["environment", "--value", "PIPX_BIN_DIR"])
            .unwrap_or_else(|| format!("{}/.local/bin", home_dir()));
        print_item(
            styles,
            &format!(
                "pipx : {} (packages={}, bin={}, binaires globaux=oui)",
                pipx_ver, pipx_pkg_count, pipx_bin_dir
            ),
        );
    } else {
        print_item(styles, "pipx : non installe");
    }
    if command_exists("uv") {
        let uv_ver = command_output("uv", &["--version"]).unwrap_or_else(|| "unknown".to_string());
        let uv_tool_dir = command_output("uv", &["tool", "dir"]).unwrap_or_else(|| "unknown".to_string());
        let uv_tool_list = command_output("uv", &["tool", "list"]).unwrap_or_default();
        let uv_tool_pkg_count = uv_tool_list
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter(|line| !line.trim_start().starts_with('-'))
            .count();
        let uv_tool_bin_count = uv_tool_list
            .lines()
            .filter(|line| line.trim_start().starts_with('-'))
            .count();
        print_item(
            styles,
            &format!(
                "uv : {} (tools={}, executables={}, dir={}, binaires globaux=oui via 'uv tool')",
                uv_ver.strip_prefix("uv ").unwrap_or(&uv_ver),
                uv_tool_pkg_count,
                uv_tool_bin_count,
                uv_tool_dir
            ),
        );
    } else {
        print_item(styles, "uv : non installe");
    }

    println!();
    println!(
        "  {}[C] Outils de projet (venv local, pas orientes global){}",
        styles.title, styles.reset
    );
    if command_exists("poetry") {
        let poetry_ver = command_output("poetry", &["--version"]).unwrap_or_else(|| "poetry".to_string());
        print_item(
            styles,
            &format!("{poetry_ver} (binaires globaux=plutot non, orienté projet)"),
        );
    } else {
        print_item(styles, "poetry : non installe");
    }
    if command_exists("pipenv") {
        let pipenv_ver = command_output("pipenv", &["--version"]).unwrap_or_else(|| "pipenv".to_string());
        print_item(
            styles,
            &format!("{pipenv_ver} (binaires globaux=plutot non, orienté projet)"),
        );
    } else {
        print_item(styles, "pipenv : non installe");
    }

    println!();
    println!("  {}[D] Distribution systeme{}", styles.title, styles.reset);
    if command_exists("brew") {
        let brew_lines = command_output("brew", &["list", "--versions"]).unwrap_or_default();
        let names = brew_lines
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .filter(|name| *name == "python" || name.starts_with("python@"))
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if names.is_empty() {
            print_item(styles, "Homebrew python : aucune formule python detectee");
        } else {
            let preview = if names.len() > 5 {
                format!("{}, ...", names[..5].join(", "))
            } else {
                names.join(", ")
            };
            print_item(
                styles,
                &format!(
                    "Homebrew python : {} formule(s) [{}] (binaires globaux=oui)",
                    names.len(),
                    preview
                ),
            );
        }
    } else {
        print_item(styles, "Homebrew : non installe");
    }

    print_section(styles, "7) PACKAGES INSTALLÉS (pip list)");
    if let Some(py) = python_bin {
        if command_success(py, &["-m", "pip", "-V"]) {
            let list = command_output(py, &["-m", "pip", "list", "--format=columns"]).unwrap_or_default();
            if list.trim().is_empty() {
                print_item(styles, "Aucun package listé");
            } else {
                for line in list.lines() {
                    println!("  • {}{}{}", styles.value, line, styles.reset);
                }
            }
        } else {
            print_item(styles, "pip indisponible");
        }
    } else {
        print_item(styles, "pip indisponible");
    }

    Ok(ExitCode::SUCCESS)
}

fn detect_python_bin() -> Option<String> {
    if command_exists("python3") {
        Some("python3".to_string())
    } else if command_exists("python") {
        Some("python".to_string())
    } else {
        None
    }
}

fn detect_system_context() -> (String, String, String) {
    let os = command_output("/usr/bin/uname", &["-s"])
        .or_else(|| command_output("uname", &["-s"]))
        .unwrap_or_else(|| env::consts::OS.to_string());
    let arch = command_output("/usr/bin/uname", &["-m"])
        .or_else(|| command_output("uname", &["-m"]))
        .unwrap_or_else(|| env::consts::ARCH.to_string());

    match os.as_str() {
        "Darwin" => (
            "macOS".to_string(),
            command_output("/usr/bin/sw_vers", &["-productVersion"]).unwrap_or_else(|| "unknown".to_string()),
            arch,
        ),
        "Linux" => ("Linux".to_string(), linux_pretty_name().unwrap_or_else(|| "unknown".to_string()), arch),
        other => (other.to_string(), "unknown".to_string(), arch),
    }
}

fn linux_pretty_name() -> Option<String> {
    let content = std::fs::read_to_string("/etc/os-release").ok()?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
            return Some(value.trim_matches('"').to_string());
        }
    }
    None
}

fn python_runtime_details(py: &str) -> Vec<String> {
    let script = r#"
import sys, platform, site, sysconfig
print(f"Python        : {sys.version.split()[0]}")
print(f"Executable    : {sys.executable}")
print(f"Prefix        : {sys.prefix}")
print(f"Base Prefix   : {getattr(sys, 'base_prefix', sys.prefix)}")
print(f"Venv Actif    : {sys.prefix != getattr(sys, 'base_prefix', sys.prefix)}")
print(f"Platform      : {platform.platform()}")
print(f"Arch          : {platform.machine()}")
print(f"Stdlib        : {sysconfig.get_path('stdlib')}")
print(f"Site-packages : {', '.join(site.getsitepackages()) if hasattr(site, 'getsitepackages') else 'n/a'}")
print(f"User site     : {site.getusersitepackages()}")
print(f"Default enc   : {sys.getdefaultencoding()}")
"#;

    command_output(py, &["-c", script])
        .unwrap_or_default()
        .lines()
        .map(ToString::to_string)
        .collect()
}

fn python_console_scripts(py: &str, package: &str) -> Vec<String> {
    let script = r#"
import sys
pkg = sys.argv[1]
try:
    import importlib.metadata as md
except Exception:
    raise SystemExit(0)

dist = None
needle = pkg.lower()
for d in md.distributions():
    name = (d.metadata.get("Name") or "").strip()
    if name.lower() == needle:
        dist = d
        break

if dist is None:
    raise SystemExit(0)

names = sorted({ep.name for ep in dist.entry_points if ep.group == "console_scripts"}, key=str.lower)
for name in names:
    print(name)
"#;

    command_output(py, &["-c", script, package])
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(ToString::to_string)
        .collect()
}

fn print_section(styles: &Styles, title: &str) {
    println!();
    println!("{}{}{}", styles.section, title, styles.reset);
    println!("{}──────────────────────────────────────────────{}", styles.dim, styles.reset);
}

fn print_item(styles: &Styles, text: &str) {
    if let Some((key, value)) = text.split_once(" : ") {
        println!(
            "  • {}{}{} : {}{}{}",
            styles.key, key, styles.dim, styles.value, value, styles.reset
        );
    } else if let Some((key, value)) = text.split_once(" = ") {
        println!(
            "  • {}{}{} = {}{}{}",
            styles.key, key, styles.dim, styles.value, value, styles.reset
        );
    } else {
        println!("  • {}{}{}", styles.value, text, styles.reset);
    }
}

fn print_type_line(styles: &Styles, text: &str) {
    if let Some((cmd, rest)) = text.split_once(" is ") {
        println!(
            "  • {}{}{} is {}{}{}",
            styles.key, cmd, styles.dim, styles.value, rest, styles.reset
        );
    } else if let Some(cmd) = text.strip_suffix(" not found") {
        println!("  • {}{}{} not found{}", styles.neg, cmd, styles.dim, styles.reset);
    } else if let Some((cmd, rest)) = text.split_once(" : ") {
        if rest == "non trouvé" {
            println!("  • {}{}{} not found{}", styles.neg, cmd, styles.dim, styles.reset);
        } else {
            print_item(styles, text);
        }
    } else {
        print_item(styles, text);
    }
}

fn shell_type_lines(name: &str) -> Vec<String> {
    let zsh = resolve_zsh();
    Command::new(zsh)
        .arg("-c")
        .arg(format!("type -a {name} 2>/dev/null || true"))
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn resolve_zsh() -> String {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    if shell.contains("zsh") && Path::new(&shell).is_file() {
        shell
    } else if Path::new("/bin/zsh").is_file() {
        "/bin/zsh".to_string()
    } else {
        "zsh".to_string()
    }
}

fn command_exists(cmd: &str) -> bool {
    if cmd.contains('/') {
        return Path::new(cmd).is_file();
    }
    command_path(cmd).is_some()
}

fn command_path(cmd: &str) -> Option<String> {
    let path = env::var("PATH").ok()?;
    for dir in path.split(':') {
        let candidate = format!("{dir}/{cmd}");
        if Path::new(&candidate).is_file() {
            return Some(candidate);
        }
    }
    None
}

fn command_success(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn command_output(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn home_dir() -> String {
    env::var("HOME").unwrap_or_else(|_| "/Users/unknown".to_string())
}

struct Styles {
    reset: &'static str,
    title: &'static str,
    section: &'static str,
    key: &'static str,
    value: &'static str,
    neg: &'static str,
    dim: &'static str,
}

impl Styles {
    fn detect() -> Self {
        if !std::io::stdout().is_terminal() {
            return Self::plain();
        }

        let theme = env::var("PYTHONINFO_THEME").unwrap_or_else(|_| "auto".to_string());
        let resolved = match theme.to_ascii_lowercase().as_str() {
            "light" | "alucard" => "light",
            "dark" | "dracula" => "dark",
            _ => match env::var("ZSH_THEME_MODE")
                .unwrap_or_else(|_| "dark".to_string())
                .to_ascii_lowercase()
                .as_str()
            {
                "light" => "light",
                _ => "dark",
            },
        };

        match resolved {
            "light" => Self {
                reset: "\x1b[0m",
                title: "\x1b[38;5;54m",
                section: "\x1b[38;5;24m",
                key: "\x1b[38;5;25m",
                value: "\x1b[38;5;236m",
                neg: "\x1b[38;5;124m",
                dim: "\x1b[38;5;242m",
            },
            _ => Self {
                reset: "\x1b[0m",
                title: "\x1b[38;5;141m",
                section: "\x1b[38;5;117m",
                key: "\x1b[38;5;81m",
                value: "\x1b[38;5;255m",
                neg: "\x1b[38;5;203m",
                dim: "\x1b[38;5;245m",
            },
        }
    }

    fn plain() -> Self {
        Self {
            reset: "",
            title: "",
            section: "",
            key: "",
            value: "",
            neg: "",
            dim: "",
        }
    }
}
