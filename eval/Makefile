# RipTide Test Suite Makefile
# Provides convenient commands for running tests

.PHONY: help verify quick test eval clean

# Default target
help:
	@echo "RipTide Test Suite Commands:"
	@echo ""
	@echo "  make verify  - Verify all test URLs are accessible"
	@echo "  make quick   - Run quick accessibility tests"
	@echo "  make test    - Run full extraction tests (requires built CLI)"
	@echo "  make eval    - Run all test suites and generate report"
	@echo "  make clean   - Clean test results"
	@echo ""
	@echo "Test Suites:"
	@echo "  00_static_docs    - Static documentation pages"
	@echo "  10_news_articles  - News article pages"
	@echo "  20_product_pages  - E-commerce product pages"
	@echo "  30_listings       - Listing and hub pages"
	@echo "  40_tables_pdfs    - PDF documents with tables"
	@echo "  50_events         - Event listings from Hilversum"

# Verify URLs are accessible
verify:
	@echo "Verifying test URLs..."
	@./eval/verify_urls.sh

# Quick accessibility test
quick:
	@echo "Running quick tests..."
	@./eval/quick_test.sh

# Run full test suite
test:
	@echo "Running full test suite..."
	@./eval/run_tests.sh

# Run all evaluations and generate comprehensive report
eval: verify test
	@echo "Evaluation complete!"

# Clean test results
clean:
	@echo "Cleaning test results..."
	@rm -rf eval/results/*
	@echo "Results cleaned."

# Individual suite tests
test-static:
	@echo "Testing static documentation pages..."
	@./eval/run_tests.sh eval/suites/00_static_docs.yml

test-news:
	@echo "Testing news articles..."
	@./eval/run_tests.sh eval/suites/10_news_articles.yml

test-products:
	@echo "Testing product pages..."
	@./eval/run_tests.sh eval/suites/20_product_pages.yml

test-listings:
	@echo "Testing listing pages..."
	@./eval/run_tests.sh eval/suites/30_listings.yml

test-pdfs:
	@echo "Testing PDF documents..."
	@./eval/run_tests.sh eval/suites/40_tables_pdfs.yml

test-events:
	@echo "Testing event pages..."
	@./eval/run_tests.sh eval/suites/50_events_hilversum_music.yml