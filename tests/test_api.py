"""Integration tests for ZKP Privacy Airdrop"""

import os
import pytest
import requests
from web3 import Web3


@pytest.fixture
def relayer_url():
    return os.getenv("RELAYER_URL", "http://localhost:8080")


def test_health_endpoint(relayer_url):
    """Test the health check endpoint"""
    response = requests.get(f"{relayer_url}/api/v1/health")
    assert response.status_code == 200

    data = response.json()
    assert data["status"] in ["healthy", "unhealthy"]
    assert "version" in data


def test_get_merkle_root(relayer_url):
    """Test getting the Merkle root"""
    response = requests.get(f"{relayer_url}/api/v1/merkle-root")
    assert response.status_code == 200

    data = response.json()
    assert "merkle_root" in data
    assert data["merkle_root"].startswith("0x")
    assert len(data["merkle_root"]) == 66


def test_get_contract_info(relayer_url):
    """Test getting contract information"""
    response = requests.get(f"{relayer_url}/api/v1/contract-info")
    assert response.status_code == 200

    data = response.json()
    assert data["network"] == "optimism"
    assert data["chain_id"] == 10
    assert "contracts" in data


def test_get_stats(relayer_url):
    """Test getting relayer statistics"""
    response = requests.get(f"{relayer_url}/api/v1/stats")
    assert response.status_code == 200

    data = response.json()
    assert "total_claims" in data
    assert "successful_claims" in data
    assert "relayer_balance" in data


def test_check_status_not_claimed(relayer_url):
    """Test checking status for an unclaimed nullifier"""
    nullifier = "0x" + "0" * 64
    response = requests.get(f"{relayer_url}/api/v1/check-status/{nullifier}")
    assert response.status_code == 200

    data = response.json()
    assert data["claimed"] is False
    assert data["nullifier"] == nullifier


def test_invalid_proof_submission(relayer_url):
    """Test submitting an invalid proof"""
    payload = {
        "proof": {
            "a": ["0", "0"],
            "b": [["0", "0"], ["0", "0"]],
            "c": ["0", "0"],
        },
        "public_signals": ["0", "0", "0"],
        "nullifier": "0x" + "0" * 64,
        "recipient": "0x1234567890123456789012345678901234567890",
        "merkle_root": "0x" + "0" * 64,
    }

    response = requests.post(f"{relayer_url}/api/v1/submit-claim", json=payload)
    # Should fail with invalid proof
    assert response.status_code in [400, 500]
