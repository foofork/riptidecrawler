"""Tests for RipTide client."""

import pytest
import json
from unittest.mock import Mock, patch, MagicMock
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
    with patch('riptide_client.client.requests.Session') as mock_session_class:
        mock_session = Mock()
        mock_session_class.return_value = mock_session

        with RipTide('http://localhost:8080') as client:
            assert client.session is not None

        # Session close should have been called
        mock_session.close.assert_called_once()


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


# Phase 2 Tests

def test_spider_stats_mode(client, mock_response):
    """Test spider with stats result mode."""
    mock_response.json.return_value = {
        'pages_crawled': 50,
        'pages_failed': 2,
        'duration_seconds': 12.5,
        'stop_reason': 'max_pages_reached'
    }

    with patch.object(client.session, 'request', return_value=mock_response):
        result = client.spider(
            seeds=['https://example.com'],
            result_mode='stats',
            max_pages=50
        )
        assert result['pages_crawled'] == 50
        assert result['duration_seconds'] == 12.5


def test_spider_urls_mode(client, mock_response):
    """Test spider with urls result mode."""
    mock_response.json.return_value = {
        'pages_crawled': 100,
        'pages_failed': 5,
        'duration_seconds': 30.2,
        'stop_reason': 'completed',
        'domains': ['example.com', 'test.example.com'],
        'discovered_urls': [
            'https://example.com/page1',
            'https://example.com/page2',
            'https://test.example.com/page3'
        ]
    }

    with patch.object(client.session, 'request', return_value=mock_response):
        result = client.spider(
            seeds=['https://example.com'],
            result_mode='urls',
            max_pages=100
        )
        assert len(result['discovered_urls']) == 3
        assert 'example.com' in result['domains']


def test_spider_pages_mode(client, mock_response):
    """Test spider with pages result mode."""
    mock_response.json.return_value = {
        'pages_crawled': 10,
        'pages_failed': 0,
        'duration_seconds': 8.3,
        'stop_reason': 'completed',
        'pages': [
            {
                'url': 'https://example.com/page1',
                'depth': 0,
                'status_code': 200,
                'title': 'Page 1',
                'links': ['https://example.com/page2'],
                'markdown': '# Page 1\nContent here'
            },
            {
                'url': 'https://example.com/page2',
                'depth': 1,
                'status_code': 200,
                'title': 'Page 2',
                'links': [],
                'markdown': '# Page 2\nMore content'
            }
        ],
        'api_version': '2.0'
    }

    with patch.object(client.session, 'request', return_value=mock_response):
        result = client.spider(
            seeds=['https://example.com'],
            result_mode='pages',
            include='title,markdown,links',
            max_pages=10
        )
        assert len(result['pages']) == 2
        assert result['pages'][0]['title'] == 'Page 1'
        assert 'markdown' in result['pages'][0]


def test_spider_store_mode(client, mock_response):
    """Test spider with store result mode."""
    mock_response.json.return_value = {
        'job_id': 'job-12345-abcde'
    }

    with patch.object(client.session, 'request', return_value=mock_response):
        job_id = client.spider_store(
            seeds=['https://example.com'],
            max_pages=5000,
            include='title,markdown'
        )
        assert job_id == 'job-12345-abcde'


def test_spider_stream(client):
    """Test spider streaming mode."""
    mock_response = Mock()
    mock_response.status_code = 200

    # Simulate NDJSON stream
    ndjson_lines = [
        b'{"type":"page","data":{"url":"https://example.com/1","depth":0,"status_code":200,"title":"Page 1"}}',
        b'{"type":"page","data":{"url":"https://example.com/2","depth":1,"status_code":200,"title":"Page 2"}}',
        b'{"type":"stats","data":{"pages_crawled":2,"pages_failed":0,"duration_seconds":5.2,"stop_reason":"completed"}}'
    ]
    mock_response.iter_lines.return_value = iter(ndjson_lines)

    with patch.object(client.session, 'request', return_value=mock_response):
        events = list(client.spider_stream(
            seeds=['https://example.com'],
            include='title,links'
        ))

        assert len(events) == 3
        assert events[0]['type'] == 'page'
        assert events[0]['data']['title'] == 'Page 1'
        assert events[2]['type'] == 'stats'
        assert events[2]['data']['pages_crawled'] == 2


def test_get_results(client, mock_response):
    """Test fetching paginated job results."""
    mock_response.json.return_value = {
        'pages': [
            {
                'url': 'https://example.com/1',
                'depth': 0,
                'status_code': 200,
                'title': 'Result 1'
            },
            {
                'url': 'https://example.com/2',
                'depth': 1,
                'status_code': 200,
                'title': 'Result 2'
            }
        ],
        'next_cursor': 'cursor-abc123',
        'done': False
    }

    with patch.object(client.session, 'request', return_value=mock_response):
        results = client.get_results(
            'job-12345',
            limit=2,
            include='title'
        )
        assert len(results['pages']) == 2
        assert results['next_cursor'] == 'cursor-abc123'
        assert results['done'] is False


def test_get_stats(client, mock_response):
    """Test fetching job statistics."""
    mock_response.json.return_value = {
        'pages_crawled': 1000,
        'pages_failed': 15,
        'duration_seconds': 120.5,
        'stop_reason': 'max_pages_reached'
    }

    with patch.object(client.session, 'request', return_value=mock_response):
        stats = client.get_stats('job-12345')
        assert stats['pages_crawled'] == 1000
        assert stats['duration_seconds'] == 120.5


def test_extract_single(client, mock_response):
    """Test single URL extraction."""
    mock_response.json.return_value = {
        'url': 'https://example.com/article',
        'markdown': '# Article Title\n\nArticle content here',
        'metadata': {
            'title': 'Article Title',
            'author': 'John Doe'
        }
    }

    with patch.object(client.session, 'request', return_value=mock_response):
        result = client.extract('https://example.com/article', format='markdown')
        assert 'Article Title' in result['markdown']
        assert result['metadata']['author'] == 'John Doe'


def test_extract_batch(client, mock_response):
    """Test batch URL extraction."""
    mock_response.json.return_value = [
        {
            'url': 'https://example.com/page1',
            'markdown': '# Page 1\nContent 1',
            'metadata': {'title': 'Page 1'}
        },
        {
            'url': 'https://example.com/page2',
            'markdown': '# Page 2\nContent 2',
            'metadata': {'title': 'Page 2'}
        },
        {
            'url': 'https://example.com/page3',
            'error': 'Failed to fetch',
            'markdown': None
        }
    ]

    with patch.object(client.session, 'request', return_value=mock_response):
        results = client.extract_batch(
            ['https://example.com/page1', 'https://example.com/page2', 'https://example.com/page3'],
            format='markdown'
        )
        assert len(results) == 3
        assert results[0]['markdown'] == '# Page 1\nContent 1'
        assert results[2]['error'] == 'Failed to fetch'


def test_spider_with_field_selection(client, mock_response):
    """Test spider with include/exclude parameters."""
    mock_response.json.return_value = {
        'pages_crawled': 5,
        'pages_failed': 0,
        'duration_seconds': 3.5,
        'stop_reason': 'completed',
        'pages': [
            {
                'url': 'https://example.com/1',
                'depth': 0,
                'status_code': 200,
                'title': 'Page 1',
                'markdown': '# Page 1',
                'links': ['https://example.com/2']
                # content excluded
            }
        ]
    }

    with patch.object(client.session, 'request', return_value=mock_response):
        result = client.spider(
            seeds=['https://example.com'],
            result_mode='pages',
            include='title,markdown,links',
            exclude='content',
            max_pages=5
        )
        page = result['pages'][0]
        assert 'title' in page
        assert 'markdown' in page
        assert 'links' in page
        assert 'content' not in page


def test_backward_compatibility_start_spider(client, mock_response):
    """Test legacy start_spider method still works."""
    mock_response.json.return_value = {
        'job_id': 'legacy-job-123',
        'status': 'started'
    }

    with patch.object(client.session, 'request', return_value=mock_response):
        result = client.start_spider(
            url='https://example.com',
            max_depth=2,
            max_pages=10
        )
        assert result['job_id'] == 'legacy-job-123'
