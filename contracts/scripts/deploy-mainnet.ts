import { ethers } from "hardhat";

async function main() {
  console.log("Deploying to Optimism Mainnet...");

  const [deployer] = await ethers.getSigners();
  console.log("Deploying contracts with account:", deployer.address);

  // Deploy PLONK Verifier
  console.log("\n1. Deploying PLONKVerifier...");
  const PlonkVerifier = await ethers.getContractFactory("PlonkVerifier");
  const verifier = await PlonkVerifier.deploy();
  await verifier.waitForDeployment();
  const verifierAddress = await verifier.getAddress();
  console.log("   PLONKVerifier deployed to:", verifierAddress);

  // Deploy ZKP Token
  console.log("\n2. Deploying ZKPToken...");
  const ZKPToken = await ethers.getContractFactory("ZKPToken");
  const token = await ZKPToken.deploy();
  await token.waitForDeployment();
  const tokenAddress = await token.getAddress();
  console.log("   ZKPToken deployed to:", tokenAddress);

  // Mint initial tokens to airdrop contract
  const AIRDROP_AMOUNT = ethers.parseUnits("65000000000", 18);
  console.log("\n3. Minting tokens to airdrop contract...");
  const mintTx = await token.mint(deployer.address, AIRDROP_AMOUNT);
  await mintTx.wait();
  console.log("   Minted", ethers.formatUnits(AIRDROP_AMOUNT, 18), "tokens");

  // TODO: Calculate actual merkle root from accounts list
  const MERKLE_ROOT = ethers.ZeroHash;
  const CLAIM_AMOUNT = ethers.parseUnits("1000", 18);
  const CLAIM_DEADLINE = Math.floor(Date.now() / 1000) + 365 * 24 * 60 * 60; // 1 year from now

  console.log("\n4. Deploying PrivacyAirdropPLONK...");
  const PrivacyAirdropPLONK = await ethers.getContractFactory("PrivacyAirdropPLONK");
  const airdrop = await PrivacyAirdropPLONK.deploy(
    tokenAddress,
    MERKLE_ROOT,
    CLAIM_AMOUNT,
    CLAIM_DEADLINE,
    verifierAddress
  );
  await airdrop.waitForDeployment();
  const airdropAddress = await airdrop.getAddress();
  console.log("   PrivacyAirdropPLONK deployed to:", airdropAddress);

  // Transfer tokens to airdrop contract
  console.log("\n5. Transferring tokens to airdrop contract...");
  const transferTx = await token.transfer(airdropAddress, AIRDROP_AMOUNT);
  await transferTx.wait();
  console.log("   Transferred", ethers.formatUnits(AIRDROP_AMOUNT, 18), "tokens to airdrop");

  console.log("\n=== Deployment Summary ===");
  console.log("Network: Optimism Mainnet");
  console.log("PLONKVerifier:", verifierAddress);
  console.log("ZKPToken:", tokenAddress);
  console.log("PrivacyAirdropPLONK:", airdropAddress);
  console.log("Claim Amount:", ethers.formatUnits(CLAIM_AMOUNT, 18), "tokens");
  console.log("Claim Deadline:", new Date(CLAIM_DEADLINE * 1000).toISOString());
  console.log("\nIMPORTANT: Update MERKLE_ROOT with actual merkle root from accounts list!");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
