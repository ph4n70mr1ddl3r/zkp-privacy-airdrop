import pytest
from web3 import Web3
import os


@pytest.fixture
def web3_provider():
    """Web3 provider for testing"""
    rpc_url = os.getenv("RPC_URL", "https://optimism.drpc.org")
    w3 = Web3(Web3.HTTPProvider(rpc_url))
    return w3


def test_web3_connection(web3_provider):
    """Test that we can connect to the RPC"""
    assert web3_provider.is_connected()


def test_get_block_number(web3_provider):
    """Test getting block number"""
    block = web3_provider.eth.get_block("latest")
    assert block is not None
    assert block["number"] > 0


def test_account_validation(web3_provider):
    """Test Ethereum address validation"""
    valid_address = "0x1234567890123456789012345678901234567890"
    invalid_address = "0x123"

    assert Web3.is_address(valid_address) is True
    assert Web3.is_address(invalid_address) is False


def test_checksum_address(web3_provider):
    """Test checksum address generation"""
    address = "0x1234567890123456789012345678901234567890"
    checksum = Web3.to_checksum_address(address)

    assert Web3.is_address(checksum) is True
