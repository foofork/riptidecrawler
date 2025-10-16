#!/bin/bash

# Create all test suite files

mkdir -p eval/suites

# Static docs
cat > eval/suites/00_static_docs.yml << 'SUITE_END'
suite: 00_static_docs
targets:
  - name: "MDN JS Guide Introduction"
    url: "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Introduction"
    type: article
  - name: "Rust Book Installation"
    url: "https://doc.rust-lang.org/stable/book/ch01-01-installation.html"
    type: docs
  - name: "PostgreSQL Docs"
    url: "https://www.postgresql.org/docs/17/intro-whatis.html"
    type: docs
  - name: "Kubernetes Get Started"
    url: "https://cloud.google.com/kubernetes-engine/docs/learn/get-started-with-kubernetes"
    type: docs
  - name: "Wikipedia Web Scraping"
    url: "https://en.wikipedia.org/wiki/Web_scraping"
    type: reference
SUITE_END

# News articles
cat > eval/suites/10_news_articles.yml << 'SUITE_END'
suite: 10_news_articles
targets:
  - name: "Reuters China War Tech"
    url: "https://www.reuters.com/graphics/WW2-ANNIVERSARY/CHINA-PARADE/zdvxkgybypx/"
    type: article
  - name: "Reuters Meta AI"
    url: "https://www.reuters.com/investigates/special-report/meta-ai-chatbot-guidelines/"
    type: article
  - name: "Reuters Ukraine Drones"
    url: "https://www.reuters.com/graphics/UKRAINE-CRISIS/DRONES/dwpkeyjwkpm/"
    type: article
  - name: "NOS Tech News"
    url: "https://nos.nl/nieuws/tech"
    type: listing
  - name: "NOS AI Race Europe"
    url: "https://nos.nl/nieuwsuur/artikel/2585550-europa-ontwaakt-in-de-ai-race-maar-druppel-op-gloeiende-plaat"
    type: article
SUITE_END

# Product pages
cat > eval/suites/20_product_pages.yml << 'SUITE_END'
suite: 20_product_pages
targets:
  - name: "Samsung OLED TV"
    url: "https://www.coolblue.nl/en/product/947062/samsung-oled-4k-55s95d-2024.html"
    type: product
  - name: "Lenovo Yoga Laptop"
    url: "https://www.coolblue.nl/en/product/946460/lenovo-yoga-slim-7-oled-14imh9-83cv0054mh.html"
    type: product
  - name: "Canon EOS R5 C"
    url: "https://www.bhphotovideo.com/c/product/1684244-REG/canon_5077c002_eos_r5_c_full_frame.html"
    type: product
  - name: "Canon EOS C80"
    url: "https://www.bhphotovideo.com/c/product/1851537-REG/canon_eos_c80_cinema_camera.html"
    type: product
SUITE_END

# Listings
cat > eval/suites/30_listings.yml << 'SUITE_END'
suite: 30_listings
targets:
  - name: "Hacker News"
    url: "https://news.ycombinator.com/"
    type: listing
  - name: "GitHub Rust Topics"
    url: "https://github.com/topics/rust"
    type: listing
  - name: "Stack Overflow Rust"
    url: "https://stackoverflow.com/questions/tagged/rust"
    type: listing
  - name: "Coolblue Laptops"
    url: "https://www.coolblue.nl/en/laptops"
    type: listing
SUITE_END

# PDFs
cat > eval/suites/40_tables_pdfs.yml << 'SUITE_END'
suite: 40_tables_pdfs
targets:
  - name: "UK Autumn Budget 2024"
    url: "https://assets.publishing.service.gov.uk/media/6722120210b0d582ee8c48c0/Autumn_Budget_2024__print_.pdf"
    type: pdf
  - name: "UK Budget Policy Costings"
    url: "https://assets.publishing.service.gov.uk/media/6721d2c54da1c0d41942a8d2/Policy_Costing_Document_-_Autumn_Budget_2024.pdf"
    type: pdf
  - name: "OECD ODA 2024"
    url: "https://one.oecd.org/document/DCD%282025%296/en/pdf"
    type: pdf
  - name: "Hilversum Budget Info"
    url: "https://hilversum.nl/sites/default/files/documents/Vereiste%20informatie%20in%20activiteitenplan%2C%20begroting%20en%20dekkingsplan%20.pdf"
    type: pdf
SUITE_END

# Events
cat > eval/suites/50_events_hilversum_music.yml << 'SUITE_END'
suite: 50_events_hilversum_music
targets:
  - name: "Live Hilversum NL"
    url: "https://www.livehilversum.com/nl/uitagenda"
    type: events_listing
  - name: "Live Hilversum EN"
    url: "https://www.livehilversum.com/en/events"
    type: events_listing
  - name: "De Vorstin Venue"
    url: "https://vorstin.nl/agenda/"
    type: venue_listing
  - name: "Songkick Hilversum"
    url: "https://www.songkick.com/metro-areas/31392-netherlands-hilversum/2025"
    type: aggregator_listing
SUITE_END

echo "All suite files created!"
ls -la eval/suites/
