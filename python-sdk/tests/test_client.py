"""Tests for RipTide client."""

import pytest
from unittest.mock import Mock, patch
from riptide_client import RipTide, APIError, RateLimitError


@pytest.fixture
def client():
    """Create test client."""
    return RipTide('http://localhost:8080')


@pytest.fixture
def mock_response():
    """Create mock response."""
    mock = Mock()
    mock.status_code = 200
    mock.json.return_value = {'status': 'ok'}
    return mock


def test_client_initialization():
    """Test client initialization."""
    client = RipTide('http://localhost:8080', api_key='test-key')
    assert client.base_url == 'http://localhost:8080'
    assert client.api_key == 'test-key'
    assert 'Bearer test-key' in client.session.headers['Authorization']


def test_health_check(client, mock_response):
    """Test health check."""
    with patch.object(client.session, 'request', return_value=mock_response):
        result = client.health()
        assert result == {'status': 'ok'}


def test_crawl(client, mock_response):
    """Test crawl method."""
    mock_response.json.return_value = {
        'results': [
            {'url': 'https://example.com', 'document': {'title': 'Example'}}
        ]
    }

    with patch.object(client.session, 'request', return_value=mock_response):
        result = client.crawl(['https://example.com'])
        assert len(result['results']) == 1
        assert result['results'][0]['url'] == 'https://example.com'


def test_rate_limit_error(client):
    """Test rate limit handling."""
    mock_response = Mock()
    mock_response.status_code = 429

    with patch.object(client.session, 'request', return_value=mock_response):
        with pytest.raises(RateLimitError):
            client.crawl(['https://example.com'])


def test_api_error(client):
    """Test API error handling."""
    mock_response = Mock()
    mock_response.status_code = 500
    mock_response.json.return_value = {'error': 'Internal server error'}

    with patch.object(client.session, 'request', return_value=mock_response):
        with pytest.raises(APIError):
            client.crawl(['https://example.com'])


def test_context_manager():
    """Test context manager usage."""
    with RipTide('http://localhost:8080') as client:
        assert client.session is not None

    # Session should be closed
    assert client.session._closed


def test_session_management(client, mock_response):
    """Test session management."""
    mock_response.json.return_value = {'id': 'session-123', 'name': 'test'}

    with patch.object(client.session, 'request', return_value=mock_response):
        session = client.create_session('test', {'user_agent': 'TestBot'})
        assert session['id'] == 'session-123'


def test_search(client, mock_response):
    """Test search method."""
    mock_response.json.return_value = {
        'results': [
            {'title': 'Result 1', 'url': 'https://example.com'}
        ]
    }

    with patch.object(client.session, 'request', return_value=mock_response):
        result = client.search('test query', {'limit': 10})
        assert len(result['results']) == 1
