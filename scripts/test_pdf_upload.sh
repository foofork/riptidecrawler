#!/bin/bash
# Test script for PDF multipart upload endpoint
#
# Usage: ./test_pdf_upload.sh [pdf_file]

set -e

# Default test PDF or use argument
PDF_FILE="${1:-test.pdf}"
API_URL="${API_URL:-http://localhost:8080}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Testing PDF Multipart Upload Endpoint${NC}"
echo "=========================================="
echo ""

# Check if PDF exists
if [ ! -f "$PDF_FILE" ]; then
    echo -e "${RED}Error: PDF file not found: $PDF_FILE${NC}"
    echo "Creating a minimal test PDF..."

    # Create a minimal valid PDF for testing
    cat > test.pdf << 'EOF'
%PDF-1.4
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [3 0 R] /Count 1 >>
endobj
3 0 obj
<< /Type /Page /Parent 2 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> /MediaBox [0 0 612 792] /Contents 4 0 R >>
endobj
4 0 obj
<< /Length 44 >>
stream
BT
/F1 12 Tf
100 700 Td
(Test PDF) Tj
ET
endstream
endobj
xref
0 5
0000000000 65535 f
0000000009 00000 n
0000000058 00000 n
0000000115 00000 n
0000000317 00000 n
trailer
<< /Size 5 /Root 1 0 R >>
startxref
406
%%EOF
EOF
    PDF_FILE="test.pdf"
    echo -e "${GREEN}Created test.pdf${NC}"
fi

echo "PDF File: $PDF_FILE"
echo "API URL: $API_URL/pdf/upload"
echo ""

# Test 1: Basic upload
echo -e "${YELLOW}Test 1: Basic PDF upload${NC}"
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_URL/pdf/upload" \
  -F "file=@$PDF_FILE" \
  -H "Accept: application/json")

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)

if [ "$HTTP_CODE" -eq 200 ]; then
    echo -e "${GREEN}✓ Success (HTTP $HTTP_CODE)${NC}"
    echo "$BODY" | jq . 2>/dev/null || echo "$BODY"
else
    echo -e "${RED}✗ Failed (HTTP $HTTP_CODE)${NC}"
    echo "$BODY"
fi
echo ""

# Test 2: Upload with metadata
echo -e "${YELLOW}Test 2: Upload with filename and URL${NC}"
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_URL/pdf/upload" \
  -F "file=@$PDF_FILE" \
  -F "filename=custom-name.pdf" \
  -F "url=https://example.com/document" \
  -H "Accept: application/json")

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)

if [ "$HTTP_CODE" -eq 200 ]; then
    echo -e "${GREEN}✓ Success (HTTP $HTTP_CODE)${NC}"
    echo "$BODY" | jq .document.url 2>/dev/null || echo "$BODY"
else
    echo -e "${RED}✗ Failed (HTTP $HTTP_CODE)${NC}"
    echo "$BODY"
fi
echo ""

# Test 3: Invalid file (should fail)
echo -e "${YELLOW}Test 3: Upload non-PDF file (should fail)${NC}"
echo "This is not a PDF" > not-a-pdf.txt
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_URL/pdf/upload" \
  -F "file=@not-a-pdf.txt" \
  -H "Accept: application/json")

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)

if [ "$HTTP_CODE" -eq 400 ]; then
    echo -e "${GREEN}✓ Correctly rejected (HTTP $HTTP_CODE)${NC}"
    echo "$BODY" | jq .error 2>/dev/null || echo "$BODY"
else
    echo -e "${RED}✗ Unexpected response (HTTP $HTTP_CODE)${NC}"
    echo "$BODY"
fi
rm -f not-a-pdf.txt
echo ""

# Test 4: Missing file field (should fail)
echo -e "${YELLOW}Test 4: Missing file field (should fail)${NC}"
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_URL/pdf/upload" \
  -F "filename=test.pdf" \
  -H "Accept: application/json")

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)

if [ "$HTTP_CODE" -eq 400 ]; then
    echo -e "${GREEN}✓ Correctly rejected (HTTP $HTTP_CODE)${NC}"
    echo "$BODY" | jq .error 2>/dev/null || echo "$BODY"
else
    echo -e "${RED}✗ Unexpected response (HTTP $HTTP_CODE)${NC}"
    echo "$BODY"
fi
echo ""

# Test 5: Performance test
echo -e "${YELLOW}Test 5: Performance measurement${NC}"
START=$(date +%s%N)
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_URL/pdf/upload" \
  -F "file=@$PDF_FILE" \
  -H "Accept: application/json")
END=$(date +%s%N)
DURATION=$(( (END - START) / 1000000 ))

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)

if [ "$HTTP_CODE" -eq 200 ]; then
    echo -e "${GREEN}✓ Success${NC}"
    echo "Client-side duration: ${DURATION}ms"
    echo "$BODY" | jq .stats 2>/dev/null || echo "$BODY"
else
    echo -e "${RED}✗ Failed (HTTP $HTTP_CODE)${NC}"
fi
echo ""

echo -e "${GREEN}Testing complete!${NC}"

# Cleanup if we created a test PDF
if [ "$PDF_FILE" = "test.pdf" ] && [ -f "test.pdf" ]; then
    echo ""
    read -p "Remove generated test.pdf? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -f test.pdf
        echo "Removed test.pdf"
    fi
fi
