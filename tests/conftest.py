"""Test configuration"""

import pytest


@pytest.fixture
def test_config():
    """Test configuration fixture"""
    return {
        "network": "optimism",
        "chain_id": 10,
        "airdrop_contract": "0x1234...",
        "token_contract": "0xabcd...",
    }


def test_config_exists(test_config):
    """Test that test config exists"""
    assert test_config is not None
    assert test_config["network"] == "optimism"
