"""
Performance comparison: per-page extraction vs. batch extraction.

This script compares the performance of:
1. Original approach: extract tables page by page using `find_tables`
2. New approach: extract all tables at once using `find_all_tables`
"""

import time
from pathlib import Path

from tablers import Document, find_all_tables, find_tables

PDF_PATH = r"C:\Users\mon\Downloads\boc_20220025_0001_p000.pdf"


def measure_time(func):
    """Decorator to measure execution time."""

    def wrapper(*args, **kwargs):
        start = time.perf_counter()
        result = func(*args, **kwargs)
        end = time.perf_counter()
        return result, end - start

    return wrapper


@measure_time
def extract_per_page(doc: Document, extract_text: bool = True) -> dict[int, list]:
    """Extract tables page by page (original approach)."""
    results = {}
    for page in doc.pages():
        tables = find_tables(page, extract_text=extract_text)
        if tables:
            results[page.page_idx] = tables
    return results


@measure_time
def extract_all_at_once(
    doc: Document, extract_text: bool = True, num_threads: int | None = None
) -> dict[int, list]:
    """Extract all tables at once using multi-threading (new approach)."""
    return find_all_tables(doc, extract_text=extract_text, num_threads=num_threads)


@measure_time
def extract_with_batch(
    doc: Document, extract_text: bool = True, batch_size: int = 10, num_threads: int | None = None
) -> dict[int, list]:
    """Extract tables with batch processing (for large documents)."""
    return find_all_tables(
        doc, extract_text=extract_text, batch_size=batch_size, num_threads=num_threads
    )


def print_results(name: str, results: dict, elapsed: float):
    """Print the results of a test."""
    total_tables = sum(len(tables) for tables in results.values())
    pages_with_tables = len(results)
    print(f"  {name}:")
    print(f"    - Time: {elapsed:.4f} seconds")
    print(f"    - Pages with tables: {pages_with_tables}")
    print(f"    - Total tables found: {total_tables}")


def main():
    pdf_path = Path(PDF_PATH)
    if not pdf_path.exists():
        print(f"Error: PDF file not found: {pdf_path}")
        return

    print(f"PDF: {pdf_path}")
    print(f"File size: {pdf_path.stat().st_size / 1024 / 1024:.2f} MB")
    print()

    with Document(path=str(pdf_path)) as doc:
        page_count = doc.page_count
        print(f"Page count: {page_count}")
        print()

        # Warm up (first run may include initialization overhead)
        print("Warming up...")
        _ = find_tables(doc.get_page(0), extract_text=True)
        print()

        print("=" * 60)
        print("Performance Comparison (extract_text=True)")
        print("=" * 60)

        # Test 1: Per-page extraction
        results1, time1 = extract_per_page(doc, extract_text=True)
        print_results("Per-page extraction (original)", results1, time1)
        print()

        # Test 2: All at once (default threads)
        results2, time2 = extract_all_at_once(doc, extract_text=True)
        print_results("All at once (default threads)", results2, time2)
        print()

        # Test 3: All at once (4 threads)
        results3, time3 = extract_all_at_once(doc, extract_text=True, num_threads=4)
        print_results("All at once (4 threads)", results3, time3)
        print()

        # Test 4: Batch processing (batch_size=10)
        results4, time4 = extract_with_batch(doc, extract_text=True, batch_size=10)
        print_results("Batch processing (batch_size=10)", results4, time4)
        print()

        # Test 5: Batch processing (batch_size=50)
        if page_count > 50:
            results5, time5 = extract_with_batch(doc, extract_text=True, batch_size=50)
            print_results("Batch processing (batch_size=50)", results5, time5)
            print()

        print("=" * 60)
        print("Summary")
        print("=" * 60)
        print(f"  Per-page:           {time1:.4f}s (baseline)")
        print(f"  All at once:        {time2:.4f}s ({time1 / time2:.2f}x speedup)")
        print(f"  All at once (4t):   {time3:.4f}s ({time1 / time3:.2f}x speedup)")
        print(f"  Batch (size=10):    {time4:.4f}s ({time1 / time4:.2f}x speedup)")
        if page_count > 50:
            print(f"  Batch (size=50):    {time5:.4f}s ({time1 / time5:.2f}x speedup)")

        print()
        print("=" * 60)
        print("Performance Comparison (extract_text=False)")
        print("=" * 60)

        # Test without text extraction (faster, just structure detection)
        results_no_text1, time_no_text1 = extract_per_page(doc, extract_text=False)
        print_results("Per-page extraction (no text)", results_no_text1, time_no_text1)
        print()

        results_no_text2, time_no_text2 = extract_all_at_once(doc, extract_text=False)
        print_results("All at once (no text)", results_no_text2, time_no_text2)
        print()

        print("=" * 60)
        print("Summary (no text extraction)")
        print("=" * 60)
        print(f"  Per-page: {time_no_text1:.4f}s (baseline)")
        print(f"  All at once: {time_no_text2:.4f}s ({time_no_text1 / time_no_text2:.2f}x speedup)")


if __name__ == "__main__":
    main()
