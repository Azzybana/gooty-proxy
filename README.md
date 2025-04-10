# Gooty Proxy 🚀

Gooty Proxy is a library for discovering, testing, and managing HTTP and SOCKS proxies. It provides tools for working with proxy servers, including discovery, validation, metadata collection, and management.

## Features ✨

- **Proxy Discovery**: Fetch proxies from various sources.
- **Validation**: Test proxies for connectivity, anonymity, and performance.
- **Metadata Collection**: Gather information like location, organization, and ASN.
- **Management**: Rotate, persist, and manage proxy pools (rotation planned for future versions).

## Installation 📦

Add Gooty Proxy to your `Cargo.toml`:

```toml
[dependencies]
gooty-proxy = "0.2.0"
```

## Usage 🛠️

Here's a quick example of how to use Gooty Proxy:

```rust
use gooty_proxy::ProxyManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = ProxyManager::new()?;

    manager.init_judge().await?;
    manager.init_sleuth()?;

    let proxy_url = "http://example.com:8080";
    manager.add_proxy_from_url(proxy_url)?;

    manager.check_proxy(proxy_url).await?;
    manager.enrich_proxy(proxy_url).await?;

    if let Some(proxy) = manager.get_proxy(proxy_url) {
        println!("Proxy type: {}", proxy.proxy_type);
        println!("Anonymity: {}", proxy.anonymity);
        if let Some(country) = &proxy.country {
            println!("Country: {}", country);
        }
    }

    Ok(())
}
```

## Planned Features 🛠️

- **Proxy Rotation**: Mangement enhanced with rotation strategies.

## Contributing 🤝

Contributions are welcome! Feel free to open issues or submit pull requests.

## License 📜

This project is licensed under the Apache License 2.0.

---

Happy proxying! 🌐
