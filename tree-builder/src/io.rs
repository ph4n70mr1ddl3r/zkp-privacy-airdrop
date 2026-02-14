use anyhow::{Context, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

pub fn read_addresses(path: &Path) -> Result<Vec<[u8; 20]>> {
    let canonical = path.canonicalize().context("Failed to canonicalize path")?;
    let file = File::open(&canonical).context("Failed to open addresses file")?;

    let reader = BufReader::new(file);
    let mut addresses = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let address_bytes = if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
            hex::decode(&trimmed[2..])
        } else {
            hex::decode(trimmed)
        }
        .context("Invalid hex address")?;

        if address_bytes.len() != 20 {
            anyhow::bail!(
                "Invalid address length: expected 20 bytes, got {}",
                address_bytes.len()
            );
        }

        let mut addr = [0u8; 20];
        addr.copy_from_slice(&address_bytes);
        addresses.push(addr);
    }

    Ok(addresses)
}

#[derive(Debug, Clone)]
pub struct TreeHeader {
    pub magic: [u8; 4],
    pub version: u8,
    pub height: u8,
    pub reserved: [u8; 2],
    pub num_leaves: u32,
    pub root_hash: [u8; 32],
}

impl Default for TreeHeader {
    fn default() -> Self {
        Self {
            magic: *b"ZKPT",
            version: 1,
            height: 26,
            reserved: [0, 0],
            num_leaves: 0,
            root_hash: [0u8; 32],
        }
    }
}

pub fn write_tree(tree: &crate::tree::MerkleTree, output: &Path) -> Result<()> {
    let file = File::create(output).context("Failed to create output file")?;

    let mut writer = BufWriter::new(file);

    if tree.leaves.len() > u32::MAX as usize {
        return Err(anyhow::anyhow!(
            "Tree too large: {} leaves exceeds maximum of {}",
            tree.leaves.len(),
            u32::MAX
        ));
    }

    let header = TreeHeader {
        magic: *b"ZKPT",
        version: 1,
        height: tree.height,
        reserved: [0, 0],
        #[allow(clippy::cast_possible_truncation)]
        num_leaves: tree.leaves.len() as u32,
        root_hash: tree.root,
    };

    writer.write_all(&header.magic)?;
    writer.write_u8(header.version)?;
    writer.write_u8(header.height)?;
    writer.write_all(&header.reserved)?;
    writer.write_u32::<BigEndian>(header.num_leaves)?;
    writer.write_all(&header.root_hash)?;

    for leaf in &tree.leaves {
        writer.write_all(leaf)?;
    }

    writer.flush().context("Failed to flush tree file")?;

    Ok(())
}

#[allow(dead_code)]
pub fn read_tree(path: &Path) -> Result<crate::tree::MerkleTree> {
    let file = File::open(path).context("Failed to open tree file")?;

    let mut reader = BufReader::new(file);

    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if magic != *b"ZKPT" {
        anyhow::bail!("Invalid magic number in tree file");
    }

    let version = reader.read_u8()?;
    if version != 1 {
        anyhow::bail!("Unsupported tree version: {}", version);
    }

    let height = reader.read_u8()?;

    let mut reserved = [0u8; 2];
    reader.read_exact(&mut reserved)?;

    let num_leaves = reader.read_u32::<BigEndian>()?;

    let mut root_hash = [0u8; 32];
    reader.read_exact(&mut root_hash)?;

    let mut tree = crate::tree::MerkleTree::new(height);

    for _ in 0..num_leaves {
        let mut leaf = [0u8; 32];
        reader.read_exact(&mut leaf)?;
        tree.insert(leaf)
            .map_err(|e| anyhow::anyhow!("Failed to insert leaf: {}", e))?;
    }

    tree.root = root_hash;

    Ok(tree)
}
