"""Test configuration"""

import os
import pytest


@pytest.fixture
def test_config():
    """Test configuration fixture"""
    return {
        "network": os.getenv("TEST_NETWORK", "optimism"),
        "chain_id": int(os.getenv("TEST_CHAIN_ID", "10")),
        "airdrop_contract": os.getenv(
            "TEST_AIRDROP_CONTRACT", "0x1234567890123456789012345678901234567890"
        ),
        "token_contract": os.getenv(
            "TEST_TOKEN_CONTRACT", "0xabcd123456789012345678901234567890123456"
        ),
    }


def test_config_exists(test_config):
    """Test that test config exists"""
    assert test_config is not None
    assert test_config["network"] in ["optimism", "ethereum"]
