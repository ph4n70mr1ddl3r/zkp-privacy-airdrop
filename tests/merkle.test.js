import { describe, it, expect, beforeEach } from '@jest/globals';
const { keccak256 } = require('js-sha3');

describe('Merkle Tree', () => {
  describe('Tree Construction', () => {
    it('should create a valid Merkle tree with 2 leaves', async () => {
      const addresses = generateTestAddresses(2);
      const tree = await buildMerkleTree(addresses);
      expect(tree.root).toBeDefined();
      expect(tree.root).toHaveLength(66);
      expect(tree.root).toMatch(/^0x[a-f0-9]{64}$/);
    });

    it('should create a valid Merkle tree with 8 leaves', async () => {
      const addresses = generateTestAddresses(8);
      const tree = await buildMerkleTree(addresses);
      expect(tree.root).toBeDefined();
      expect(tree.leaves).toHaveLength(8);
    });

    it('should create a valid Merkle tree with 100 leaves', async () => {
      const addresses = generateTestAddresses(100);
      const tree = await buildMerkleTree(addresses);
      expect(tree.root).toBeDefined();
      expect(tree.leaves).toHaveLength(100);
    });

    it('should create a valid Merkle tree with power-of-2 leaves (256)', async () => {
      const addresses = generateTestAddresses(256);
      const tree = await buildMerkleTree(addresses);
      expect(tree.root).toBeDefined();
      expect(tree.leaves).toHaveLength(256);
    });

    it('should handle non-power-of-2 leaf counts correctly', async () => {
      const addresses = generateTestAddresses(150);
      const tree = await buildMerkleTree(addresses);
      expect(tree.root).toBeDefined();
      expect(tree.leaves).toHaveLength(150);
    });

    it('should reject empty address list', async () => {
      await expect(buildMerkleTree([])).rejects.toThrow('at least one address');
    });

    it('should reject single address list', async () => {
      await expect(buildMerkleTree(['0x1234567890123456789012345678901234567890'])).rejects.toThrow();
    });
  });

  describe('Address Validation', () => {
    it('should accept valid Ethereum addresses', () => {
      const validAddress = '0x1234567890123456789012345678901234567890';
      expect(validateEthereumAddress(validAddress)).toBe(true);
    });

    it('should reject addresses without 0x prefix', () => {
      const invalidAddress = '1234567890123456789012345678901234567890';
      expect(validateEthereumAddress(invalidAddress)).toBe(false);
    });

    it('should reject addresses with wrong length', () => {
      const shortAddress = '0x123456789012345678901234567890123456789';
      expect(validateEthereumAddress(shortAddress)).toBe(false);
    });

    it('should reject zero address', () => {
      const zeroAddress = '0x0000000000000000000000000000000000000000';
      expect(validateEthereumAddress(zeroAddress)).toBe(false);
    });

    it('should reject addresses with invalid hex characters', () => {
      const invalidAddress = '0xGHIJ567890123456789012345678901234567890';
      expect(validateEthereumAddress(invalidAddress)).toBe(false);
    });
  });

  describe('Proof Generation', () => {
    it('should generate valid proof for leaf in tree', async () => {
      const addresses = generateTestAddresses(100);
      const tree = await buildMerkleTree(addresses);
      const proof = await tree.getProof(tree.leaves[50]);
      expect(proof).toBeDefined();
      expect(Array.isArray(proof)).toBe(true);
      expect(proof.length).toBeGreaterThan(0);
    });

    it('should generate valid proof for first leaf', async () => {
      const addresses = generateTestAddresses(100);
      const tree = await buildMerkleTree(addresses);
      const proof = await tree.getProof(tree.leaves[0]);
      expect(proof).toBeDefined();
    });

    it('should generate valid proof for last leaf', async () => {
      const addresses = generateTestAddresses(100);
      const tree = await buildMerkleTree(addresses);
      const proof = await tree.getProof(tree.leaves[99]);
      expect(proof).toBeDefined();
    });

    it('should generate consistent proofs for same leaf', async () => {
      const addresses = generateTestAddresses(100);
      const tree = await buildMerkleTree(addresses);
      const proof1 = await tree.getProof(tree.leaves[50]);
      const proof2 = await tree.getProof(tree.leaves[50]);
      expect(proof1).toEqual(proof2);
    });

    it('should handle proof generation for power-of-2 trees', async () => {
      const addresses = generateTestAddresses(256);
      const tree = await buildMerkleTree(addresses);
      const proof = await tree.getProof(tree.leaves[128]);
      expect(proof).toBeDefined();
    });
  });

  describe('Proof Verification', () => {
    it('should verify valid proof for leaf in tree', async () => {
      const addresses = generateTestAddresses(100);
      const tree = await buildMerkleTree(addresses);
      const leaf = tree.leaves[50];
      const proof = await tree.getProof(leaf);
      const valid = await tree.verifyProof(leaf, proof);
      expect(valid).toBe(true);
    });

    it('should verify valid proof for all leaves', async () => {
      const addresses = generateTestAddresses(50);
      const tree = await buildMerkleTree(addresses);
      for (const leaf of tree.leaves) {
        const proof = await tree.getProof(leaf);
        const valid = await tree.verifyProof(leaf, proof);
        expect(valid).toBe(true);
      }
    });

    it('should reject proof for leaf not in tree', async () => {
      const addresses = generateTestAddresses(100);
      const tree = await buildMerkleTree(addresses);
      const fakeLeaf = '0x1234567890123456789012345678901234567890';
      const fakeProof = ['0x0000000000000000000000000000000000000000000000000000000000000000'];
      const valid = await tree.verifyProof(fakeLeaf, fakeProof);
      expect(valid).toBe(false);
    });

    it('should reject proof with wrong root', async () => {
      const addresses = generateTestAddresses(100);
      const tree = await buildMerkleTree(addresses);
      const leaf = tree.leaves[50];
      const proof = await tree.getProof(leaf);
      const wrongRoot = '0x0000000000000000000000000000000000000000000000000000000000000001';
      const valid = await tree.verifyProof(leaf, proof, wrongRoot);
      expect(valid).toBe(false);
    });

    it('should reject proof with incorrect sibling', async () => {
      const addresses = generateTestAddresses(100);
      const tree = await buildMerkleTree(addresses);
      const leaf = tree.leaves[50];
      const proof = await tree.getProof(leaf);
      proof[0] = '0x' + 'a'.repeat(64);
      const valid = await tree.verifyProof(leaf, proof);
      expect(valid).toBe(false);
    });

    it('should reject empty proof', async () => {
      const addresses = generateTestAddresses(100);
      const tree = await buildMerkleTree(addresses);
      const leaf = tree.leaves[50];
      const valid = await tree.verifyProof(leaf, []);
      expect(valid).toBe(false);
    });
  });

  describe('Merkle Root Validation', () => {
    it('should generate consistent root for same addresses', async () => {
      const addresses = generateTestAddresses(100);
      const tree1 = await buildMerkleTree(addresses);
      const tree2 = await buildMerkleTree(addresses);
      expect(tree1.root).toEqual(tree2.root);
    });

    it('should generate different roots for different addresses', async () => {
      const addresses1 = generateTestAddresses(100);
      const addresses2 = generateTestAddresses(100);
      const tree1 = await buildMerkleTree(addresses1);
      const tree2 = await buildMerkleTree(addresses2);
      expect(tree1.root).not.toEqual(tree2.root);
    });

    it('should detect address duplication in input', async () => {
      const addresses = [
        '0x1234567890123456789012345678901234567890',
        '0x1234567890123456789012345678901234567890',
        '0xabcdef1234567890abcdef1234567890abcdef12',
      ];
      await expect(buildMerkleTree(addresses)).rejects.toThrow('duplicate');
    });

    it('should generate root in valid hex format', async () => {
      const addresses = generateTestAddresses(100);
      const tree = await buildMerkleTree(addresses);
      expect(tree.root).toMatch(/^0x[a-f0-9]{64}$/);
    });
  });

  describe('Edge Cases', () => {
    it('should handle addresses with mixed case', async () => {
      const addresses = [
        '0x1234567890123456789012345678901234567890',
        '0xABCDEF1234567890abcdef1234567890abcdef12',
        '0xFedCBA9876543210FEDcba9876543210fedcba98',
      ];
      const tree = await buildMerkleTree(addresses);
      expect(tree.root).toBeDefined();
    });

    it('should handle addresses with leading zeros', async () => {
      const addresses = [
        '0x0000000000000000000000000000000000000001',
        '0x0000000000000000000000000000000000000002',
        '0x0000000000000000000000000000000000000003',
      ];
      const tree = await buildMerkleTree(addresses);
      expect(tree.root).toBeDefined();
    });

    it('should handle addresses with all f characters', async () => {
      const addresses = [
        '0xffffffffffffffffffffffffffffffffffffffff',
        '0xfffffffffffffffffffffffffffffffffffffffe',
        '0xfffffffffffffffffffffffffffffffffffffffd',
      ];
      const tree = await buildMerkleTree(addresses);
      expect(tree.root).toBeDefined();
    });
  });
});

function validateEthereumAddress(address) {
  return /^0x[a-f0-9]{40}$/.test(address.toLowerCase());
}

function generateTestAddresses(count) {
  const addresses = [];
  for (let i = 0; i < count; i++) {
    const hex = i.toString(16).padStart(40, '0');
    addresses.push('0x' + hex);
  }
  return addresses;
}

async function buildMerkleTree(addresses) {
  if (addresses.length < 2) {
    throw new Error('Merkle tree requires at least 2 addresses');
  }

  const uniqueAddresses = [...new Set(addresses)];
  if (uniqueAddresses.length !== addresses.length) {
    throw new Error('Duplicate addresses detected');
  }

  const leaves = addresses.map(addr => 
    '0x' + keccak256(addr.toLowerCase())
  );

  let layer = leaves;
  while (layer.length > 1) {
    const nextLayer = [];
    for (let i = 0; i < layer.length; i += 2) {
      const left = layer[i];
      const right = layer[i + 1] || layer[i];
      const combined = left < right ? left + right.slice(2) : right + left.slice(2);
      nextLayer.push('0x' + keccak256(combined));
    }
    layer = nextLayer;
  }

  return {
    root: layer[0],
    leaves,
    async getProof(leaf) {
      const index = this.leaves.indexOf(leaf);
      if (index === -1) {
        throw new Error('Leaf not found in tree');
      }

      const proof = [];
      let currentLayer = this.leaves;
      let currentIndex = index;

      while (currentLayer.length > 1) {
        const isRight = currentIndex % 2 === 1;
        const siblingIndex = isRight ? currentIndex - 1 : currentIndex + 1;
        const sibling = currentLayer[siblingIndex] || currentLayer[currentIndex];
        proof.push(sibling);

        const nextLayer = [];
        for (let i = 0; i < currentLayer.length; i += 2) {
          const left = currentLayer[i];
          const right = currentLayer[i + 1] || currentLayer[i];
          const combined = left < right ? left + right.slice(2) : right + left.slice(2);
          nextLayer.push('0x' + keccak256(combined));
        }
        currentLayer = nextLayer;
        currentIndex = Math.floor(currentIndex / 2);
      }

      return proof;
    },
    async verifyProof(leaf, proof, root = this.root) {
      let computedHash = leaf;
      for (const sibling of proof) {
        const [left, right] = computedHash < sibling 
          ? [computedHash, sibling]
          : [sibling, computedHash];
        computedHash = '0x' + keccak256(left + right.slice(2));
      }
      return computedHash.toLowerCase() === root.toLowerCase();
    }
  };
}
