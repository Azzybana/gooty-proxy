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
        enums::{AnonymityLevel, ProxyType},
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

    /// Enable info level logging (shows informational messages)
    #[arg(long, global = true, conflicts_with_all = ["warn", "error", "debug"])]
    info: bool,

    /// Enable warning level logging (shows warnings and errors only)
    #[arg(long, global = true, conflicts_with_all = ["info", "error", "debug"])]
    warn: bool,

    /// Enable error level logging (shows errors only)
    #[arg(long, global = true, conflicts_with_all = ["info", "warn", "debug"])]
    error: bool,

    /// Enable debug level logging (shows all messages including debug info)
    #[arg(long, global = true, conflicts_with_all = ["info", "warn", "error"])]
    debug: bool,
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

        /// Test each proxy found
        #[arg(
            long,
            help = "Automatically test each proxy's connectivity after scraping"
        )]
        judge: bool,

        /// Gather detailed proxy information (ASN, organization, etc.)
        #[arg(
            long,
            help = "Collect additional metadata about each proxy (country, ASN, organization)"
        )]
        detail: bool,

        /// Don't save to sources list
        #[arg(
            long,
            help = "Run scraping operation without saving the source to the persistent sources list"
        )]
        dry: bool,

        /// Save response to a file
        #[arg(
            long,
            help = "Save the raw HTTP response from the scraped site to a local file"
        )]
        dump: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Set up logging based on flags
    let log_level = if cli.debug {
        "debug"
    } else if cli.info {
        "info"
    } else if cli.warn {
        "warn"
    } else if cli.error {
        "error"
    } else {
        ""
    };

    // Initialize logger with explicit configuration
    let level_filter = if log_level.is_empty() {
        log::LevelFilter::Info
    } else {
        log::LevelFilter::from_str(log_level).unwrap_or(log::LevelFilter::Info)
    };
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

                println!("Created default configuration in {}", path);
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
                        eprintln!("Failed to access filestore: {}", e);
                        std::process::exit(1);
                    }
                };

                // Try to load and validate configuration
                match filestore.load_config("config") {
                    Ok(_) => {
                        println!("Configuration in {} is valid", path);
                        std::process::exit(0);
                    }
                    Err(e) => {
                        eprintln!("Configuration validation failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Some(Commands::Proxy { judge, dry }) => {
            if let Some(proxy_url) = judge {
                // Initialize proxy manager and required components
                let mut manager = match ProxyManager::new() {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Failed to initialize proxy manager: {}", e);
                        std::process::exit(1);
                    }
                };

                // Initialize judge and sleuth
                if let Err(e) = manager.init_judge().await {
                    eprintln!("Failed to initialize judge: {}", e);
                    std::process::exit(1);
                }

                if let Err(e) = manager.init_sleuth() {
                    eprintln!("Failed to initialize sleuth: {}", e);
                    std::process::exit(1);
                }

                // Parse proxy URL
                let proxy = match parse_proxy_url(&proxy_url) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Invalid proxy URL: {}", e);
                        std::process::exit(1);
                    }
                };

                println!("Testing proxy: {}", proxy_url);

                // Add proxy to manager
                if let Err(e) = manager.add_proxy(proxy) {
                    eprintln!("Failed to add proxy: {}", e);
                    std::process::exit(1);
                }

                // Get proxy ID for management
                let proxy_id = proxy_url.clone();

                // Check proxy connectivity and anonymity
                if let Err(e) = manager.check_proxy(&proxy_id).await {
                    eprintln!("Proxy test failed: {}", e);
                    std::process::exit(1);
                }

                // Enrich with IP metadata
                if let Err(e) = manager.enrich_proxy(&proxy_id).await {
                    eprintln!("Failed to enrich proxy data: {}", e);
                    std::process::exit(1);
                }

                // Get the tested proxy
                if let Some(proxy) = manager.get_proxy(&proxy_id) {
                    // Print detailed results
                    println!("\nProxy Test Results:");
                    println!("------------------");
                    println!("Type: {}", proxy.proxy_type);
                    println!("Anonymity Level: {}", proxy.anonymity);

                    if let Some(latency) = proxy.latency_ms {
                        println!("Latency: {}ms", latency);
                    }

                    if let Some(country) = &proxy.country {
                        println!("Country: {}", country);
                    }

                    if let Some(org) = &proxy.organization {
                        println!("Organization: {}", org);
                    }

                    if let Some(asn) = &proxy.asn {
                        println!("ASN: {}", asn);
                    }

                    if let Some(hostname) = &proxy.hostname {
                        println!("Hostname: {}", hostname);
                    }

                    println!("\nTest Statistics:");
                    println!("Success Rate: {:.1}%", proxy.check_success_rate() * 100.0);
                    println!(
                        "Checks: {} total, {} failed",
                        proxy.check_count, proxy.check_failure_count
                    );

                    // Save to proxy list if test was successful and not in dry run mode
                    if !dry && proxy.check_success_rate() > 0.0 {
                        if let Some(filestore) = get_filestore("data") {
                            match filestore.load_proxies("proxies") {
                                Ok(mut proxies) => {
                                    proxies.push(proxy.clone());
                                    if let Err(e) = filestore.save_proxies(&proxies, "proxies") {
                                        eprintln!("Failed to save proxy: {}", e);
                                    } else {
                                        println!("\nProxy saved to list successfully");
                                    }
                                }
                                Err(e) => eprintln!("Failed to load proxy list: {}", e),
                            }
                        }
                    }
                }

                std::process::exit(0);
            }
        }
        Some(Commands::Source {
            scrape,
            config,
            useragent,
            pattern,
            judge,
            detail,
            dry,
            dump,
        }) => {
            // Load configuration
            let config_path = config.unwrap_or_else(|| "data".to_string());
            let filestore = match Filestore::with_config(FilestoreConfig {
                data_dir: config_path.clone(),
                ..Default::default()
            }) {
                Ok(fs) => fs,
                Err(e) => {
                    eprintln!("Failed to initialize filestore: {}", e);
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
                    eprintln!("Failed to create source: {}", e);
                    std::process::exit(1);
                }
            };

            // Create a manager for handling proxies if we need to judge them
            let mut manager = if judge || detail {
                match ProxyManager::new() {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Failed to create proxy manager: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                ProxyManager::new().unwrap()
            };

            // Initialize judge if needed
            if judge || detail {
                if let Err(e) = manager.init_judge().await {
                    eprintln!("Failed to initialize judge: {}", e);
                    std::process::exit(1);
                }
            }

            // Initialize sleuth if detailed information is requested
            if detail {
                if let Err(e) = manager.init_sleuth() {
                    eprintln!("Failed to initialize sleuth: {}", e);
                    std::process::exit(1);
                }
            }

            // Create requestor for fetching
            let requestor = match Requestor::new() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to create requestor: {}", e);
                    std::process::exit(1);
                }
            };

            // Fetch proxies from the source
            println!("Scraping proxies from {}", scrape);
            let (proxies, raw_response) = match source.fetch_proxies_with_response(&requestor).await
            {
                Ok((proxies, response)) => (proxies, response),
                Err(e) => {
                    eprintln!("Failed to fetch proxies: {}", e);
                    std::process::exit(1);
                }
            };

            println!("Found {} proxies", proxies.len());

            // Dump response if requested
            if dump {
                let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
                let sanitized_url = utils::sanitize_url_for_filename(&scrape);
                let dump_filename = format!("{}-{}.txt", timestamp, sanitized_url);

                if let Err(e) = std::fs::write(dump_filename.clone(), raw_response) {
                    eprintln!("Failed to dump response: {}", e);
                } else {
                    println!("Response dumped to {}", dump_filename);
                }
            }

            // Judge proxies if requested
            if judge || detail {
                println!("Testing proxies...");
                let mut proxies = proxies;

                // Create progress bar
                let pb = ProgressBar::new(proxies.len() as u64);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                        .expect("Failed to create progress bar style")
                        .progress_chars("##-"),
                );

                // Check all proxies with progress
                if let Err(e) = manager.check_all_proxies(&mut proxies, 10).await {
                    pb.finish_and_clear();
                    eprintln!("Failed to verify proxies: {}", e);
                    std::process::exit(1);
                }
                pb.finish_with_message("Proxy testing complete");

                // Get additional information if detailed mode is enabled
                if detail {
                    println!("\nGathering detailed proxy information...");
                    let pb = ProgressBar::new(proxies.len() as u64);
                    pb.set_style(
                        ProgressStyle::default_bar()
                            .template(
                                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
                            )
                            .expect("Failed to create progress bar style")
                            .progress_chars("##-"),
                    );

                    if let Err(e) = manager.enrich_all_proxies(&mut proxies, 10).await {
                        pb.finish_and_clear();
                        eprintln!("Failed to enrich proxies: {}", e);
                        std::process::exit(1);
                    }
                    pb.finish_with_message("Detail gathering complete");
                }

                // Count working proxies
                let working = proxies
                    .iter()
                    .filter(|p| p.check_success_rate() > 0.0)
                    .count();
                println!("\nWorking proxies: {}/{}", working, proxies.len());

                // Save working proxies if not dry run
                if !dry {
                    if let Err(e) = filestore.save_proxies(&proxies, "proxies") {
                        eprintln!("Failed to save proxies: {}", e);
                        std::process::exit(1);
                    }
                }
            }

            // Save source if not dry run
            if !dry {
                // Load existing sources
                let mut sources = match filestore.load_sources("sources") {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to load sources: {}", e);
                        Vec::new()
                    }
                };

                // Update or add new source
                if let Some(pos) = sources.iter().position(|s| s.url == source.url) {
                    sources[pos] = source;
                } else {
                    sources.push(source);
                }

                if let Err(e) = filestore.save_sources(&sources, "sources") {
                    eprintln!("Failed to save sources: {}", e);
                    std::process::exit(1);
                }

                println!("Source saved successfully");
            }

            std::process::exit(0);
        }
    }
}

fn parse_proxy_url(url: &str) -> Result<Proxy, String> {
    // Basic URL parsing - protocol://ip:port
    let parts: Vec<&str> = url.split("://").collect();
    if parts.len() != 2 {
        return Err("Invalid proxy URL format. Expected: protocol://ip:port".to_string());
    }

    let lower = if !parts.is_empty() {
        parts[0].to_lowercase()
    } else {
        return Err("No protocol specified in proxy URL".to_string());
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

    let ip = match IpAddr::from_str(addr_parts[0]) {
        Ok(ip) => ip,
        Err(_) => return Err("Invalid IP address".to_string()),
    };

    let port = match addr_parts[1].parse::<u16>() {
        Ok(p) => p,
        Err(_) => return Err("Invalid port number".to_string()),
    };

    Ok(Proxy::new(protocol, ip, port, AnonymityLevel::Anonymous))
}

// Helper function to get filestore
fn get_filestore(data_dir: &str) -> Option<Filestore> {
    match Filestore::with_config(FilestoreConfig {
        data_dir: data_dir.to_string(),
        ..Default::default()
    }) {
        Ok(fs) => Some(fs),
        Err(e) => {
            eprintln!("Failed to initialize filestore: {}", e);
            None
        }
    }
}
