"""Test configuration"""

import os
import pytest
from typing import Dict, Any


@pytest.fixture
def test_config() -> Dict[str, Any]:
    """Test configuration fixture"""
    airdrop_contract = os.getenv("TEST_AIRDROP_CONTRACT")
    if not airdrop_contract:
        pytest.fail("TEST_AIRDROP_CONTRACT environment variable is required")

    token_contract = os.getenv("TEST_TOKEN_CONTRACT")
    if not token_contract:
        pytest.fail("TEST_TOKEN_CONTRACT environment variable is required")

    return {
        "network": os.getenv("TEST_NETWORK", "optimism"),
        "chain_id": int(os.getenv("TEST_CHAIN_ID", "10")),
        "airdrop_contract": airdrop_contract,
        "token_contract": token_contract,
    }


def test_config_exists(test_config):
    """Test that test config exists"""
    assert test_config is not None
    assert test_config["network"] in ["optimism", "ethereum"]
