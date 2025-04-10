//! # Proxy Gatherer CLI
//!
//! This module provides the command-line interface (CLI) for the Gooty Proxy application.
//! It allows users to manage configurations, test proxies, and scrape proxy sources.
//!
//! ## Overview
//!
//! The CLI supports the following commands:
//! - `Config`: Manage configuration files (create or validate)
//! - `Proxy`: Test and manage individual proxies
//! - `Source`: Scrape proxies from websites and manage sources
//!
//! ## Examples
//!
//! ```
//! // Run the CLI with the `Proxy` command to test a proxy
//! gatherer proxy --judge "http://127.0.0.1:8080"
//! ```

use clap::{CommandFactory, Parser, Subcommand};
use gooty_proxy::{
    defaults,
    definitions::{
        enums::{AnonymityLevel, JudgementMode, LogLevel, ProxyType},
        proxy::Proxy,
        source::Source,
    },
    io::{
        filesystem::{AppConfig, Filestore, FilestoreConfig},
        http::Requestor,
    },
    orchestration::manager::ProxyManager,
    utils,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::{net::IpAddr, str::FromStr};

#[derive(Parser)]
#[command(
    name = "gatherer",
    about = "Web scraper that gathers and judges proxies",
    long_about = "A command-line utility for gathering, testing, and managing proxy servers from various sources.",
    version,
    propagate_version = true
)]
struct Cli {
    /// Command to execute
    #[command(subcommand)]
    command: Option<Commands>,

    /// Log level for the application (default: Info)
    #[arg(long, global = true, value_enum, default_value_t = LogLevel::Info)]
    log_level: LogLevel,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage configuration files
    Config {
        /// Create default configuration in the specified folder
        #[arg(
            long,
            value_name = "PATH",
            conflicts_with = "validate",
            help = "Path where configuration files will be created"
        )]
        create: Option<String>,

        /// Validate configuration in the specified folder
        #[arg(
            long,
            value_name = "PATH",
            conflicts_with = "create",
            help = "Path to configuration files to validate"
        )]
        validate: Option<String>,
    },
    /// Test and manage proxies
    Proxy {
        /// Test a proxy's capabilities (format: protocol://ip:port)
        #[arg(
            long,
            value_name = "URL",
            help = "Test a proxy's connectivity, anonymity, and performance"
        )]
        judge: Option<String>,

        /// Don't save successful proxies to the list
        #[arg(
            long,
            help = "Test the proxy without saving it to the persistent proxy list"
        )]
        dry: bool,
    },
    /// Manage proxy sources and scrape proxies
    Source {
        /// URL to scrape for proxies
        #[arg(
            long,
            value_name = "URL",
            help = "Website URL to scrape for proxy server information"
        )]
        scrape: String,

        /// Path to configuration folder
        #[arg(
            long,
            value_name = "PATH",
            help = "Directory containing configuration files (default: 'data')"
        )]
        config: Option<String>,

        /// Custom User-Agent for requests
        #[arg(
            long,
            value_name = "STRING",
            help = "Custom User-Agent string to use when making HTTP requests"
        )]
        useragent: Option<String>,

        /// Custom regex pattern for finding proxies
        #[arg(
            long,
            value_name = "REGEX",
            help = "Regular expression pattern to extract proxy information from scraped content"
        )]
        pattern: Option<String>,

        /// Proxy testing and information gathering mode
        #[arg(
            long,
            value_name = "MODE",
            help = "Testing mode: none (0) - no testing, quick (1) - basic tests, full (2) - comprehensive tests with metadata",
            default_value_t = JudgementMode::None
        )]
        judge: JudgementMode,

        /// Don't save to sources list
        #[arg(
            long,
            help = "Run scraping operation without saving the source to the persistent sources list"
        )]
        dry: bool,
    },
}

/// Prints detailed information about a proxy to the console.
///
/// # Arguments
/// * `proxy` - The proxy object containing information to display
fn print_proxy_details(proxy: &Proxy) {
    println!("Proxy Type: {}", proxy.proxy_type);
    println!("Anonymity Level: {}", proxy.anonymity);
    if let Some(latency) = proxy.latency_ms {
        println!("Latency: {latency}ms");
    }
    if let Some(country) = &proxy.country {
        println!("Country: {country}");
    }
    if let Some(org) = &proxy.organization {
        println!("Organization: {org}");
    }
    if let Some(asn) = &proxy.asn {
        println!("ASN: {asn}");
    }
    if let Some(hostname) = &proxy.hostname {
        println!("Hostname: {hostname}");
    }
}

/// Handles the Config command, creating or validating configuration files.
///
/// # Arguments
/// * `create` - Optional path where to create default configuration
/// * `validate` - Optional path to validate configuration
///
/// # Returns
/// * `()` - The function exits the program with appropriate status code
fn handle_config_command(create: Option<String>, validate: Option<String>) {
    if let Some(path) = create {
        // Create default configuration
        let config = FilestoreConfig {
            data_dir: path.clone(),
            ..Default::default()
        };
        let filestore = match Filestore::with_config(config) {
            Ok(fs) => fs,
            Err(e) => {
                eprintln!("Failed to create filestore: {e}");
                std::process::exit(1);
            }
        };

        // Create default configuration files
        let default_config = AppConfig::default();
        if let Err(e) = filestore.save_config(&default_config, "config") {
            eprintln!("Failed to save configuration: {e}");
            std::process::exit(1);
        }

        println!("Created default configuration in {path}");
        std::process::exit(0);
    }

    if let Some(path) = validate {
        // Validate existing configuration
        let config = FilestoreConfig {
            data_dir: path.clone(),
            create_defaults_if_missing: false,
            ..Default::default()
        };
        let filestore = match Filestore::with_config(config) {
            Ok(fs) => fs,
            Err(e) => {
                eprintln!("Failed to access filestore: {e}");
                std::process::exit(1);
            }
        };

        // Try to load and validate configuration
        match filestore.load_config("config") {
            Ok(_) => {
                println!("Configuration in {path} is valid");
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Configuration validation failed: {e}");
                std::process::exit(1);
            }
        }
    }
}

/// Initializes a proxy manager with judge and optionally sleuth.
///
/// # Arguments
/// * `with_sleuth` - Whether to initialize the sleuth component
///
/// # Returns
/// * `Result<ProxyManager, Box<dyn std::error::Error>>` - The initialized manager or an error
fn init_proxy_manager(with_sleuth: bool) -> Result<ProxyManager, Box<dyn std::error::Error>> {
    let mut manager = ProxyManager::new()?;

    // Initialize judge
    manager.init_judge()?;

    // Initialize sleuth if needed
    if with_sleuth {
        manager.init_sleuth()?;
    }

    Ok(manager)
}

/// Handles the Proxy command, testing individual proxies.
///
/// # Arguments
/// * `judge` - Optional proxy URL to test
/// * `dry` - Whether to avoid saving results
///
/// # Returns
/// * `()` - The function exits the program with appropriate status code
async fn handle_proxy_command(judge: Option<String>, dry: bool) {
    if let Some(proxy_url) = judge {
        // Initialize proxy manager and required components
        let mut manager = match init_proxy_manager(true) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to initialize proxy manager: {e}");
                std::process::exit(1);
            }
        };

        // Parse proxy URL
        let proxy = match parse_proxy_url(&proxy_url) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Invalid proxy URL: {e}");
                std::process::exit(1);
            }
        };

        println!("Testing proxy: {proxy_url}");

        // Add proxy to manager
        if let Err(e) = manager.add_proxy(proxy) {
            eprintln!("Failed to add proxy: {e}");
            std::process::exit(1);
        }

        // Get proxy ID for management
        let proxy_id = proxy_url.clone();

        // Check proxy connectivity and anonymity
        if let Err(e) = manager.check_proxy(&proxy_id).await {
            eprintln!("Proxy test failed: {e}");
            std::process::exit(1);
        }

        // Enrich with IP metadata
        if let Err(e) = manager.enrich_proxy(&proxy_id).await {
            eprintln!("Failed to enrich proxy data: {e}");
            std::process::exit(1);
        }

        #[allow(clippy::cast_precision_loss)]
        // Get the tested proxy
        if let Some(proxy) = manager.get_proxy(&proxy_id) {
            // Print detailed results
            println!("\nProxy Test Results:");
            println!("------------------");
            print_proxy_details(proxy);

            println!("\nTest Statistics:");
            println!(
                "Success Rate: {:.2}%",
                (proxy.check_success_rate() as f64 / 100.0)
            );
            println!(
                "Checks: {} total, {} failed",
                proxy.check_count, proxy.check_failure_count
            );

            // Save to proxy list if test was successful and not in dry run mode
            if !dry && proxy.check_success_rate() > 0 {
                if let Some(filestore) = get_filestore("data") {
                    match filestore.load_proxies("proxies") {
                        Ok(mut proxies) => {
                            proxies.push(proxy.clone());
                            if let Err(e) = filestore.save_proxies(&proxies, "proxies") {
                                eprintln!("Failed to save proxy: {e}");
                            } else {
                                println!("\nProxy saved to list successfully");
                            }
                        }
                        Err(e) => eprintln!("Failed to load proxy list: {e}"),
                    }
                }
            }
        }

        std::process::exit(0);
    }
}

/// Sets up a filestore with the given configuration path.
///
/// # Arguments
/// * `config_path` - Path to the configuration directory
///
/// # Returns
/// * `Result<Filestore, Box<dyn std::error::Error>>` - The initialized filestore or an error
fn setup_filestore(config_path: &str) -> Result<Filestore, Box<dyn std::error::Error>> {
    Ok(Filestore::with_config(FilestoreConfig {
        data_dir: config_path.to_string(),
        ..Default::default()
    })?)
}

/// Tests and enriches proxies based on the specified judgement mode.
///
/// # Arguments
/// * `proxies` - List of proxies to test
/// * `mode` - Judgement mode determining the level of testing and enrichment
///
/// # Returns
/// * `Result<Vec<Proxy>, Box<dyn std::error::Error>>` - The tested proxies or an error
async fn test_and_enrich_proxies(
    mut proxies: Vec<Proxy>,
    mode: JudgementMode,
) -> Result<Vec<Proxy>, Box<dyn std::error::Error>> {
    if mode == JudgementMode::None {
        return Ok(proxies);
    }

    // Initialize manager
    let mut manager = init_proxy_manager(mode == JudgementMode::Full)?;

    // Test proxies (basic connectivity)
    println!("Testing proxies...");
    let pb = ProgressBar::new(proxies.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .expect("Failed to create progress bar style")
            .progress_chars("##-"),
    );

    // Check all proxies with progress
    manager.check_all_proxies(&mut proxies, 10).await?;
    pb.finish_with_message("Proxy testing complete");

    // Gather additional information in full mode
    if mode == JudgementMode::Full {
        println!("\nGathering detailed proxy information...");
        let pb = ProgressBar::new(proxies.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .expect("Failed to create progress bar style")
                .progress_chars("##-"),
        );

        manager.enrich_all_proxies(&mut proxies, 10).await?;
        pb.finish_with_message("Detail gathering complete");
    }

    Ok(proxies)
}

/// Handles saving proxies and sources to the filestore.
///
/// # Arguments
/// * `proxies` - List of proxies to save
/// * `source` - The source to save
/// * `filestore` - Filestore for persistence
/// * `dry` - Whether to avoid saving
/// * `raw_response` - Optional raw response data to save
/// * `mode` - Judgement mode used
/// * `scrape_url` - URL that was scraped
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or an error
fn save_results(
    proxies: &[Proxy],
    source: &Source,
    filestore: &Filestore,
    dry: bool,
    raw_response: Option<String>,
    mode: JudgementMode,
    scrape_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if dry {
        return Ok(());
    }

    // Save proxies if they exist
    if !proxies.is_empty() {
        filestore.save_proxies(proxies, "proxies")?;
    }

    // Save raw response data if in full mode
    if mode == JudgementMode::Full && raw_response.is_some() {
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let sanitized_url = utils::sanitize_url_for_filename(scrape_url);
        let dump_filename = format!("{timestamp}-{sanitized_url}.txt");

        if let Err(e) = std::fs::write(dump_filename.clone(), raw_response.unwrap()) {
            eprintln!("Failed to save raw response: {e}");
        } else {
            println!("Raw response saved to {dump_filename}");
        }
    }

    // Load existing sources
    let mut sources = filestore.load_sources("sources").unwrap_or_default();

    // Update or add new source
    if let Some(pos) = sources.iter().position(|s| s.url == source.url) {
        sources[pos] = source.clone();
    } else {
        sources.push(source.clone());
    }

    filestore.save_sources(&sources, "sources")?;
    println!("Source saved successfully");

    Ok(())
}

/// Scrapes and processes proxies from a source URL.
///
/// This function handles the entire proxy scraping workflow:
/// 1. Loads configuration and initializes components
/// 2. Scrapes proxies from the specified URL
/// 3. Tests and enriches the proxies based on the judgement mode
/// 4. Saves the results to the filestore
///
/// # Arguments
///
/// * `scrape` - URL to scrape for proxies
/// * `config` - Path to configuration folder (default: 'data')
/// * `useragent` - Custom User-Agent string to use for requests
/// * `pattern` - Custom regex pattern for finding proxies
/// * `judge` - Judgement mode determining test intensity:
///   - None (0): No testing, just scrape
///   - Quick (1): Basic connectivity testing
///   - Full (2): Comprehensive testing with metadata collection
/// * `dry` - If true, don't save results to persistent storage
///
/// # Returns
///
/// * `()` - The function exits the process with an appropriate status code
async fn handle_source_command(
    scrape: String,
    config: Option<String>,
    useragent: Option<String>,
    pattern: Option<String>,
    judge: JudgementMode,
    dry: bool,
) {
    // Load configuration
    let config_path = config.unwrap_or_else(|| "data".to_string());
    let filestore = match setup_filestore(&config_path) {
        Ok(fs) => fs,
        Err(e) => {
            eprintln!("Failed to initialize filestore: {e}");
            std::process::exit(1);
        }
    };

    // Initialize source with provided options
    let source = match Source::new(
        scrape.clone(),
        useragent.unwrap_or_else(|| utils::get_random_user_agent().to_string()),
        pattern.unwrap_or_else(|| defaults::regex_patterns::IP_PORT.to_string()),
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create source: {e}");
            std::process::exit(1);
        }
    };

    // Create requestor for fetching
    let requestor = match Requestor::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to create requestor: {e}");
            std::process::exit(1);
        }
    };

    // Fetch proxies from the source
    println!("Scraping proxies from {scrape}");
    let (proxies, raw_response) = match source.fetch_proxies_with_response(&requestor).await {
        Ok((proxies, response)) => (proxies, response),
        Err(e) => {
            eprintln!("Failed to fetch proxies: {e}");
            std::process::exit(1);
        }
    };

    println!("Found {} proxies", proxies.len());

    // Test and enrich proxies if requested
    let proxies = match test_and_enrich_proxies(proxies, judge).await {
        Ok(proxies) => {
            if judge != JudgementMode::None {
                // Count working proxies
                let working = proxies
                    .iter()
                    .filter(|p| p.check_success_rate() > 0)
                    .count();
                println!("\nWorking proxies: {}/{}", working, proxies.len());
            }
            proxies
        }
        Err(e) => {
            eprintln!("Failed during proxy testing: {e}");
            std::process::exit(1);
        }
    };

    // Save results
    let raw_response_to_save = if judge == JudgementMode::Full {
        Some(raw_response)
    } else {
        None
    };

    if let Err(e) = save_results(
        &proxies,
        &source,
        &filestore,
        dry,
        raw_response_to_save,
        judge,
        &scrape,
    ) {
        eprintln!("Failed to save results: {e}");
        std::process::exit(1);
    }

    std::process::exit(0);
}

/// Parses a proxy URL string into a Proxy object.
///
/// # Arguments
/// * `url` - The proxy URL in format protocol://ip:port
///
/// # Returns
/// * `Result<Proxy, String>` - The parsed Proxy object or an error message
fn parse_proxy_url(url: &str) -> Result<Proxy, String> {
    // Basic URL parsing - protocol://ip:port
    let parts: Vec<&str> = url.split("://").collect();
    if parts.len() != 2 {
        return Err("Invalid proxy URL format. Expected: protocol://ip:port".to_string());
    }

    let lower = if parts.is_empty() {
        return Err("No protocol specified in proxy URL".to_string());
    } else {
        parts[0].to_lowercase()
    };

    let protocol = match lower.as_str() {
        "http" => ProxyType::Http,
        "https" => ProxyType::Https,
        "socks4" => ProxyType::Socks4,
        "socks5" => ProxyType::Socks5,
        _ => return Err("Invalid protocol. Use: http, https, socks4, or socks5".to_string()),
    };

    let addr_parts: Vec<&str> = parts[1].split(':').collect();
    if addr_parts.len() != 2 {
        return Err("Invalid address format. Expected: ip:port".to_string());
    }

    let Ok(ip) = IpAddr::from_str(addr_parts[0]) else {
        return Err("Invalid IP address".to_string());
    };

    let Ok(port) = addr_parts[1].parse::<u16>() else {
        return Err("Invalid port number".to_string());
    };

    Ok(Proxy::new(protocol, ip, port, AnonymityLevel::Anonymous))
}

/// Helper function to get filestore.
///
/// # Arguments
/// * `data_dir` - Directory containing configuration files
///
/// # Returns
/// * `Option<Filestore>` - The filestore if successfully initialized, None otherwise
fn get_filestore(data_dir: &str) -> Option<Filestore> {
    match Filestore::with_config(FilestoreConfig {
        data_dir: data_dir.to_string(),
        ..Default::default()
    }) {
        Ok(fs) => Some(fs),
        Err(e) => {
            eprintln!("Failed to initialize filestore: {e}");
            None
        }
    }
}

/// Main function that handles CLI argument parsing and command dispatching.
/// Uses the clap crate for command-line argument parsing.
#[tokio::main]
async fn main() {
    // Helper function to convert our LogLevel enum to log::LevelFilter
    fn log_level_to_filter(log_level: LogLevel) -> log::LevelFilter {
        match log_level {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }

    let cli = Cli::parse();

    // Set up logging based on log level
    // Convert LogLevel to log::LevelFilter
    let level_filter = log_level_to_filter(cli.log_level);
    pretty_env_logger::formatted_builder()
        .filter_level(level_filter)
        .init();

    // Process command and arguments
    match cli.command {
        None => {
            Cli::command().print_help().unwrap();
            std::process::exit(1);
        }
        Some(Commands::Config { create, validate }) => {
            handle_config_command(create, validate);
        }
        Some(Commands::Proxy { judge, dry }) => {
            handle_proxy_command(judge, dry).await;
        }
        Some(Commands::Source {
            scrape,
            config,
            useragent,
            pattern,
            judge,
            dry,
        }) => {
            handle_source_command(scrape, config, useragent, pattern, judge, dry).await;
        }
    }
}
