use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::fs::File;
use std::io::Write;

pub async fn execute(
    source: String,
    output: std::path::PathBuf,
    format: String,
    verify: bool,
    resume: bool,
    _chunk_size: u64,
) -> Result<()> {
    println!("{} {}", "Downloading Merkle tree:".cyan(), source);
    println!("{} {}", "Output:".cyan(), output.display());
    println!("{} {}", "Format:".cyan(), format);

    let client = Client::new();

    if resume && output.exists() {
        println!("\n{}", "Resuming download...".yellow());
    }

    let mut pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .expect("Failed to create progress style")
            .progress_chars("=>-"),
    );

    pb.set_message("Connecting...");
    pb.set_position(10);

    let response = client
        .get(&source)
        .send()
        .await
        .context("Failed to start download")?;

    let total_size = response.content_length().unwrap_or(0);
    if total_size > 0 {
        pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .expect("Failed to create progress style")
                .progress_chars("=>-"),
        );
    }

    let mut file = File::create(&output).context("Failed to create output file")?;

    pb.set_message("Downloading...");

    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;
    {
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.context(format!(
                "Download error: Failed to read chunk at position {}",
                downloaded
            ))?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }
    }

    drop(file);

    pb.finish_with_message("Download complete!");

    if verify {
        println!("\n{}", "Verifying tree integrity...".yellow());
        let checksum = compute_checksum(&output)?;
        println!("{} SHA256: {}", "Checksum:".green(), checksum);
    }

    Ok(())
}

fn compute_checksum(path: &std::path::Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hex::encode(hasher.finalize()))
}
