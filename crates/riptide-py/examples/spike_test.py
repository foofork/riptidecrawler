#!/usr/bin/env python3
"""
PyO3 Spike Test - Async Runtime Integration

This script tests whether tokio async runtime works correctly with PyO3.
Run this after building the Python extension with maturin.

Usage:
    maturin develop && python examples/spike_test.py
"""

import riptide
import sys

def test_basic_async():
    """Test 1: Basic async operation"""
    print("Test 1: Basic async operation...")
    try:
        result = riptide.test_async_basic()
        print(f"✅ PASS: {result}")
        return True
    except Exception as e:
        print(f"❌ FAIL: {e}")
        return False

def test_concurrent_async():
    """Test 2: Concurrent async operations"""
    print("\nTest 2: Concurrent async operations...")
    try:
        results = riptide.test_async_concurrent()
        print(f"✅ PASS: Completed {len(results)} concurrent tasks")
        for r in results:
            print(f"   - {r}")
        return True
    except Exception as e:
        print(f"❌ FAIL: {e}")
        return False

def test_timeout_success():
    """Test 3: Timeout handling (success case)"""
    print("\nTest 3: Timeout handling (success)...")
    try:
        result = riptide.test_async_timeout(1000)  # 1 second timeout
        print(f"✅ PASS: {result}")
        return True
    except Exception as e:
        print(f"❌ FAIL: {e}")
        return False

def test_timeout_failure():
    """Test 4: Timeout handling (failure case)"""
    print("\nTest 4: Timeout handling (timeout)...")
    try:
        result = riptide.test_async_timeout(10)  # 10ms timeout (should fail)
        print(f"❌ FAIL: Should have timed out but got: {result}")
        return False
    except TimeoutError as e:
        print(f"✅ PASS: Correctly timed out")
        return True
    except Exception as e:
        print(f"❌ FAIL: Wrong exception type: {e}")
        return False

def test_error_handling_success():
    """Test 5: Error handling (success case)"""
    print("\nTest 5: Error handling (success)...")
    try:
        result = riptide.test_async_error_handling(False)
        print(f"✅ PASS: {result}")
        return True
    except Exception as e:
        print(f"❌ FAIL: {e}")
        return False

def test_error_handling_failure():
    """Test 6: Error handling (error case)"""
    print("\nTest 6: Error handling (error)...")
    try:
        result = riptide.test_async_error_handling(True)
        print(f"❌ FAIL: Should have raised error but got: {result}")
        return False
    except ValueError as e:
        print(f"✅ PASS: Correctly raised ValueError")
        return True
    except Exception as e:
        print(f"❌ FAIL: Wrong exception type: {e}")
        return False

def test_class_instantiation():
    """Test 7: RipTideSpike class instantiation"""
    print("\nTest 7: RipTideSpike class instantiation...")
    try:
        spike = riptide.RipTideSpike()
        is_healthy = spike.is_healthy()
        print(f"✅ PASS: Instance created, healthy={is_healthy}")
        return True
    except Exception as e:
        print(f"❌ FAIL: {e}")
        return False

def test_class_async_method():
    """Test 8: RipTideSpike async method"""
    print("\nTest 8: RipTideSpike async method...")
    try:
        spike = riptide.RipTideSpike()
        result = spike.test_async_method()
        print(f"✅ PASS: {result}")
        return True
    except Exception as e:
        print(f"❌ FAIL: {e}")
        return False

def test_crawl_simulation():
    """Test 9: Simulated crawl operation"""
    print("\nTest 9: Simulated crawl operation...")
    try:
        spike = riptide.RipTideSpike()
        result = spike.test_crawl_simulation("https://example.com")
        print(f"✅ PASS: Crawl result:")
        print(f"   - URL: {result['url']}")
        print(f"   - Status: {result['status']}")
        print(f"   - Content: {result['content']}")
        return True
    except Exception as e:
        print(f"❌ FAIL: {e}")
        return False

def test_spider_simulation():
    """Test 10: Simulated spider operation"""
    print("\nTest 10: Simulated spider operation...")
    try:
        spike = riptide.RipTideSpike()
        urls = spike.test_spider_simulation("https://example.com", 5)
        print(f"✅ PASS: Spider found {len(urls)} URLs:")
        for url in urls:
            print(f"   - {url}")
        return True
    except Exception as e:
        print(f"❌ FAIL: {e}")
        return False

def main():
    """Run all spike tests"""
    print("=" * 70)
    print("PyO3 Spike: Async Runtime Integration Tests")
    print("=" * 70)

    tests = [
        test_basic_async,
        test_concurrent_async,
        test_timeout_success,
        test_timeout_failure,
        test_error_handling_success,
        test_error_handling_failure,
        test_class_instantiation,
        test_class_async_method,
        test_crawl_simulation,
        test_spider_simulation,
    ]

    results = []
    for test in tests:
        results.append(test())

    print("\n" + "=" * 70)
    print("Summary")
    print("=" * 70)
    passed = sum(results)
    total = len(results)
    print(f"Passed: {passed}/{total} tests")

    if passed == total:
        print("\n✅ GO: Async runtime integration works!")
        print("✅ PyO3 + tokio is viable for Python SDK")
        return 0
    else:
        print(f"\n❌ NO-GO: {total - passed} tests failed")
        print("❌ Need to investigate async runtime issues")
        return 1

if __name__ == "__main__":
    sys.exit(main())
