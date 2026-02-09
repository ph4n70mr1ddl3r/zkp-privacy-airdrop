const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("PrivacyAirdropPLONK", function () {
  let token: any;
  let verifier: any;
  let airdrop: any;
  let owner: any;
  let user: any;

  const MERKLE_ROOT = ethers.keccak256(ethers.toUtf8Bytes("test"));
  const CLAIM_AMOUNT = ethers.parseUnits("1000", 18);
  const CLAIM_DEADLINE = Math.floor(Date.now() / 1000) + 365 * 24 * 60 * 60;

  beforeEach(async function () {
    [owner, user] = await ethers.getSigners();

    const PlonkVerifier = await ethers.getContractFactory("PlonkVerifier");
    verifier = await PlonkVerifier.deploy();
    await verifier.waitForDeployment();
    const verifierAddress = await verifier.getAddress();

    const ZKPToken = await ethers.getContractFactory("ZKPToken");
    token = await ZKPToken.deploy();
    await token.waitForDeployment();
    const tokenAddress = await token.getAddress();

    const PrivacyAirdropPLONK = await ethers.getContractFactory("PrivacyAirdropPLONK");
    airdrop = await PrivacyAirdropPLONK.deploy(
      tokenAddress,
      MERKLE_ROOT,
      CLAIM_AMOUNT,
      CLAIM_DEADLINE,
      verifierAddress
    );
    await airdrop.waitForDeployment();
    const airdropAddress = await airdrop.getAddress();

    await token.mint(airdropAddress, CLAIM_AMOUNT * 100n);
  });

  it("Should be deployed successfully", async function () {
    expect(await airdrop.getAddress()).to.properAddress;
  });

  it("Should have correct merkle root", async function () {
    expect(await airdrop.merkleRoot()).to.equal(MERKLE_ROOT);
  });

  it("Should have correct claim amount", async function () {
    expect(await airdrop.claimAmount()).to.equal(CLAIM_AMOUNT);
  });

  it("Should have correct claim deadline", async function () {
    expect(await airdrop.claimDeadline()).to.equal(CLAIM_DEADLINE);
  });

  it("Should allow owner to pause", async function () {
    await airdrop.pause();
    expect(await airdrop.paused()).to.equal(true);
  });

  it("Should allow owner to unpause", async function () {
    await airdrop.pause();
    await airdrop.unpause();
    expect(await airdrop.paused()).to.equal(false);
  });
});
