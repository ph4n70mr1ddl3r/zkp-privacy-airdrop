const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("PLONKVerifier", function () {
  let verifier: any;

  beforeEach(async function () {
    const PlonkVerifierFactory = await ethers.getContractFactory("PlonkVerifier");
    verifier = await PlonkVerifierFactory.deploy();
    await verifier.waitForDeployment();
  });

  it("Should be deployed successfully", async function () {
    expect(await verifier.getAddress()).to.properAddress;
  });

  it("Should verify proof", async function () {
    const invalidProof = Array(24).fill(0);
    const invalidPubSignals = Array(3).fill(0);

    const result = await verifier.verifyProof(invalidProof, invalidPubSignals);
    expect(result).to.equal(false);
  });

  it("Should verify proof", async function () {
    const invalidProof = Array(24).fill(0);
    const invalidPubSignals = Array(3).fill(0);

    const result = await verifier.verifyProof(invalidProof, invalidPubSignals);
    expect(result).to.equal(false);
  });
});
