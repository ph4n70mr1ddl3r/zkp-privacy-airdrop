import { ethers } from "hardhat";
import * as fs from "fs";
import * as path from "path";

import { buildPoseidon } from "circomlibjs";

let poseidon: any;

async function getPoseidon() {
  if (!poseidon) {
    poseidon = await buildPoseidon();
  }
  return poseidon;
}

export async function hashAddress(address: string): Promise<string> {
  const poseidonInstance = await getPoseidon();
  const addressAsField = ethers.zeroPadValue(address, 32);
  const addressBigInt = ethers.toBigInt(addressAsField);
  const hash = poseidonInstance([addressBigInt, 0n, 0n]);
  return ethers.toBeHex(hash).slice(2).padStart(64, "0");
}

export async function hashTwo(left: string, right: string): Promise<string> {
  const poseidonInstance = await getPoseidon();
  const leftBigInt = ethers.toBigInt("0x" + left);
  const rightBigInt = ethers.toBigInt("0x" + right);
  const hash = poseidonInstance([leftBigInt, rightBigInt, 0n]);
  return ethers.toBeHex(hash).slice(2).padStart(64, "0");
}

export async function generateMerkleRoot(addresses: string[], height: number = 26): Promise<string> {
  if (addresses.length === 0) {
    return ethers.ZeroHash;
  }

  const poseidonInstance = await getPoseidon();
  const leafHashes: string[] = [];

  for (const address of addresses) {
    const hash = await hashAddress(address);
    leafHashes.push(hash);
  }

  let level = leafHashes;
  for (let i = 0; i < height; i++) {
    const nextLevel: string[] = [];
    for (let j = 0; j < level.length; j += 2) {
      if (j + 1 < level.length) {
        const hash = await hashTwo(level[j], level[j + 1]);
        nextLevel.push(hash);
      } else {
        nextLevel.push(level[j]);
      }
    }
    level = nextLevel;
    if (level.length === 1) {
      break;
    }
  }

  return "0x" + level[0];
}

export async function generateMerkleRootFromCSV(csvPath: string, height: number = 26): Promise<string> {
  const csvContent = fs.readFileSync(csvPath, "utf-8");
  const lines = csvContent.trim().split("\n");
  const addresses = lines
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .map((line) => {
      const parts = line.split(",");
      const address = parts[0].trim();
      if (!ethers.isAddress(address)) {
        throw new Error(`Invalid address: ${address}`);
      }
      return address;
    });

  return generateMerkleRoot(addresses, height);
}
