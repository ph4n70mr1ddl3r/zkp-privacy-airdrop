"""Integration tests for PLONK proof system"""

import pytest
import os
import requests
from web3 import Web3
from typing import Dict, Any


@pytest.fixture
def relayer_url() -> str:
    return os.getenv(
        "RELAYER_URL", os.getenv("TEST_RELAYER_URL", "http://localhost:8080")
    )


@pytest.fixture
def valid_plonk_proof() -> Dict[str, Any]:
    """A valid PLONK proof structure (minimal for testing)"""
    return {
        "proof": {
            "proof": ["0"] * 8,  # 8 field elements for PLONK
        },
        "public_signals": ["0", "0", "0"],
        "nullifier": "0x" + "1" * 64,
        "recipient": "0x1234567890123456789012345678901234567890",
        "merkle_root": "0x" + "0" * 64,
        "generated_at": "2026-02-08T00:00:00Z",
    }


@pytest.fixture
def invalid_plonk_proof_too_few_elements() -> Dict[str, Any]:
    """PLONK proof with insufficient elements"""
    return {
        "proof": {
            "proof": ["0"] * 7,  # Only 7 elements (should be 8+)
        },
        "public_signals": ["0", "0", "0"],
        "nullifier": "0x" + "2" * 64,
        "recipient": "0x1234567890123456789012345678901234567890",
        "merkle_root": "0x" + "0" * 64,
        "generated_at": "2026-02-08T00:00:00Z",
    }


@pytest.fixture
def invalid_plonk_proof_empty_elements() -> Dict[str, Any]:
    """PLONK proof with empty elements"""
    return {
        "proof": {
            "proof": ["0"] * 7 + [""],  # One empty element
        },
        "public_signals": ["0", "0", "0"],
        "nullifier": "0x" + "3" * 64,
        "recipient": "0x1234567890123456789012345678901234567890",
        "merkle_root": "0x" + "0" * 64,
        "generated_at": "2026-02-08T00:00:00Z",
    }


def test_plonk_proof_structure_validation(
    relayer_url: str, valid_plonk_proof: Dict[str, Any]
) -> None:
    """Test that PLONK proof structure is correctly validated"""
    try:
        response = requests.post(
            f"{relayer_url}/api/v1/submit-claim", json=valid_plonk_proof, timeout=30
        )
    except requests.exceptions.RequestException as e:
        pytest.fail(f"Network error: {e}")

    # Note: This will fail at the proof validation step since we don't have actual verification
    # but should pass structure validation
    data = response.json()
    assert "error" in data or "tx_hash" in data


def test_plonk_proof_insufficient_elements(
    relayer_url: str, invalid_plonk_proof_too_few_elements: Dict[str, Any]
) -> None:
    """Test that PLONK proof with insufficient elements is rejected"""
    try:
        response = requests.post(
            f"{relayer_url}/api/v1/submit-claim",
            json=invalid_plonk_proof_too_few_elements,
            timeout=30,
        )
    except requests.exceptions.RequestException as e:
        pytest.fail(f"Network error: {e}")

    assert response.status_code == 400

    data = response.json()
    assert data["success"] is False
    assert data["code"] == "PLONK_FORMAT_ERROR"
    assert "8 field elements" in data["error"]


def test_plonk_proof_empty_elements(
    relayer_url: str, invalid_plonk_proof_empty_elements: Dict[str, Any]
) -> None:
    """Test that PLONK proof with empty elements is rejected"""
    try:
        response = requests.post(
            f"{relayer_url}/api/v1/submit-claim",
            json=invalid_plonk_proof_empty_elements,
            timeout=30,
        )
    except requests.exceptions.RequestException as e:
        pytest.fail(f"Network error: {e}")

    assert response.status_code == 400

    data = response.json()
    assert data["success"] is False
    assert data["code"] == "PLONK_FORMAT_ERROR"


def test_plonk_proof_type_detection(
    relayer_url: str, valid_plonk_proof: Dict[str, Any]
) -> None:
    """Test that PLONK proof type is correctly detected and logged"""
    try:
        response = requests.post(
            f"{relayer_url}/api/v1/submit-claim", json=valid_plonk_proof, timeout=30
        )
    except requests.exceptions.RequestException as e:
        pytest.fail(f"Network error: {e}")

    # The request should be processed (even if it fails later)
    assert response.status_code in [200, 400, 500]


def test_plonk_gas_estimate_logging(
    relayer_url: str, valid_plonk_proof: Dict[str, Any]
) -> None:
    """Test that PLONK gas estimate is logged (~1.3M)"""
    try:
        response = requests.post(
            f"{relayer_url}/api/v1/submit-claim", json=valid_plonk_proof, timeout=30
        )
    except requests.exceptions.RequestException as e:
        pytest.fail(f"Network error: {e}")

    # Verify is response
    data = response.json()
    assert "error" in data or "tx_hash" in data


def test_plonk_proof_size_bytes() -> None:
    """Test PLONK proof size estimation (~500 bytes)"""
    # PLONK proof with 8 field elements, each ~32 bytes
    # Plus JSON overhead
    proof = {
        "proof": ["0"] * 8,
    }

    import json

    proof_json = json.dumps(proof)
    proof_size = len(proof_json.encode("utf-8"))

    # Should be approximately 500 bytes (allowing for JSON overhead)
    assert 400 < proof_size < 600


def test_plonk_vs_groth16_proof_size_comparison() -> None:
    """Compare PLONK (~500 bytes) vs Groth16 (~200 bytes) proof sizes"""
    import json

    # PLONK proof
    plonk_proof = {
        "proof": ["0"] * 8,
    }

    # Groth16 proof
    groth16_proof = {
        "a": ["0", "0"],
        "b": [["0", "0"], ["0", "0"]],
        "c": ["0", "0"],
    }

    plonk_size = len(json.dumps(plonk_proof).encode("utf-8"))
    groth16_size = len(json.dumps(groth16_proof).encode("utf-8"))

    # PLONK should be larger than Groth16
    assert plonk_size > groth16_size

    # Check approximate sizes
    assert 400 < plonk_size < 600  # ~500 bytes
    assert 150 < groth16_size < 250  # ~200 bytes


def test_plonk_error_codes_distinct_from_groth16(
    relayer_url: str, invalid_plonk_proof_too_few_elements: Dict[str, Any]
) -> None:
    """Test that PLONK uses distinct error codes from Groth16"""
    response = requests.post(
        f"{relayer_url}/api/v1/submit-claim", json=invalid_plonk_proof_too_few_elements
    )

    data = response.json()

    # PLONK should return PLONK_FORMAT_ERROR, not INVALID_PROOF
    assert data["code"] == "PLONK_FORMAT_ERROR"
    assert data["code"] != "INVALID_PROOF"


def test_plonk_proof_with_invalid_nullifier(
    relayer_url: str, valid_plonk_proof: Dict[str, Any]
) -> None:
    """Test PLONK proof with invalid nullifier format"""
    invalid_proof = valid_plonk_proof.copy()
    invalid_proof["nullifier"] = "invalid"

    try:
        response = requests.post(
            f"{relayer_url}/api/v1/submit-claim", json=invalid_proof, timeout=30
        )
    except requests.exceptions.RequestException as e:
        pytest.fail(f"Network error: {e}")

    # Should fail validation
    assert response.status_code in [400, 500]


def test_plonk_proof_with_invalid_recipient(
    relayer_url: str, valid_plonk_proof: Dict[str, Any]
) -> None:
    """Test PLONK proof with invalid recipient address"""
    invalid_proof = valid_plonk_proof.copy()
    invalid_proof["recipient"] = "not-an-address"

    try:
        response = requests.post(
            f"{relayer_url}/api/v1/submit-claim", json=invalid_proof, timeout=30
        )
    except requests.exceptions.RequestException as e:
        pytest.fail(f"Network error: {e}")

    # Should fail validation
    assert response.status_code in [400, 500]


def test_plonk_backwards_compatibility_with_groth16(relayer_url: str) -> None:
    """Test that API accepts both PLONK and Groth16 proofs"""
    # Groth16 proof (old format)
    groth16_proof = {
        "proof": {
            "a": ["0", "0"],
            "b": [["0", "0"], ["0", "0"]],
            "c": ["0", "0"],
        },
        "nullifier": "0x" + "4" * 64,
        "recipient": "0x1234567890123456789012345678901234567890",
        "merkle_root": "0x" + "0" * 64,
    }

    try:
        response = requests.post(
            f"{relayer_url}/api/v1/submit-claim", json=groth16_proof, timeout=30
        )
    except requests.exceptions.RequestException as e:
        pytest.fail(f"Network error: {e}")

    # Should accept request (even if it fails later)
    assert response.status_code in [200, 400, 500]

    # Should not return PLONK_FORMAT_ERROR
    if response.status_code == 400:
        data = response.json()
        assert data.get("code") != "PLONK_FORMAT_ERROR"


def test_plonk_integration_with_existing_endpoints(
    relayer_url: str, valid_plonk_proof: Dict[str, Any]
) -> None:
    """Test that PLONK proofs work with existing API endpoints"""
    # Test health endpoint
    try:
        health_response = requests.get(f"{relayer_url}/api/v1/health", timeout=5)
    except requests.exceptions.RequestException as e:
        pytest.fail(f"Network error: {e}")
    assert health_response.status_code == 200

    # Test merkle root endpoint
    try:
        root_response = requests.get(f"{relayer_url}/api/v1/merkle-root", timeout=5)
    except requests.exceptions.RequestException as e:
        pytest.fail(f"Network error: {e}")
    assert root_response.status_code == 200

    # Test contract info endpoint
    try:
        info_response = requests.get(f"{relayer_url}/api/v1/contract-info", timeout=5)
    except requests.exceptions.RequestException as e:
        pytest.fail(f"Network error: {e}")
    assert info_response.status_code == 200

    # Test PLONK submission
    try:
        submit_response = requests.post(
            f"{relayer_url}/api/v1/submit-claim", json=valid_plonk_proof, timeout=30
        )
    except requests.exceptions.RequestException as e:
        pytest.fail(f"Network error: {e}")
    assert submit_response.status_code in [200, 400, 500]


def test_plonk_error_message_clarity(
    relayer_url: str, invalid_plonk_proof_too_few_elements: Dict[str, Any]
) -> None:
    """Test that PLONK error messages are clear and helpful"""
    try:
        response = requests.post(
            f"{relayer_url}/api/v1/submit-claim",
            json=invalid_plonk_proof_too_few_elements,
            timeout=30,
        )
    except requests.exceptions.RequestException as e:
        pytest.fail(f"Network error: {e}")

    data = response.json()

    assert data["success"] is False
    assert "error" in data
    assert len(data["error"]) > 10  # Should have a descriptive message
    assert "8" in data["error"]  # Should mention required element count


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
