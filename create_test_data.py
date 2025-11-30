#!/usr/bin/env python3
"""
Create test data for LexRain with simulated review history
"""
import sqlite3
from datetime import datetime, timedelta
import random

def create_test_data():
    conn = sqlite3.connect('lexrain.db')
    cursor = conn.cursor()

    # Get all words
    cursor.execute("SELECT id FROM words")
    word_ids = [row[0] for row in cursor.fetchall()]

    if not word_ids:
        print("No words found in database. Please import words first.")
        return

    print(f"Found {len(word_ids)} words in database")

    # Create review history for the past 30 days
    now = datetime.now()

    for days_ago in range(30, 0, -1):
        review_date = now - timedelta(days=days_ago)

        # Random number of reviews per day (3-10)
        num_reviews = random.randint(3, min(10, len(word_ids)))

        # Select random words to review
        words_to_review = random.sample(word_ids, num_reviews)

        for word_id in words_to_review:
            # Random quality (1-4, weighted towards 2-3)
            quality = random.choices([1, 2, 3, 4], weights=[1, 3, 4, 2])[0]

            # Random interval (0-30 days, weighted towards smaller intervals)
            interval = random.choices(
                [0, 1, 2, 3, 6, 10, 15, 20, 30],
                weights=[5, 10, 8, 6, 4, 3, 2, 1, 1]
            )[0]

            # Random repetition (0-10)
            repetition = random.randint(0, 10)

            # Random e_factor (1.3-2.5)
            e_factor = random.uniform(1.3, 2.5)

            # Insert review history
            cursor.execute("""
                INSERT INTO review_history
                (word_id, reviewed_at, quality, repetition, interval, e_factor)
                VALUES (?, ?, ?, ?, ?, ?)
            """, (
                word_id,
                review_date.isoformat(),
                quality,
                repetition,
                interval,
                e_factor
            ))

    conn.commit()

    # Print statistics
    cursor.execute("SELECT COUNT(*) FROM review_history")
    total_reviews = cursor.fetchone()[0]

    cursor.execute("SELECT COUNT(DISTINCT word_id) FROM review_history")
    unique_words = cursor.fetchone()[0]

    cursor.execute("""
        SELECT DATE(reviewed_at) as date, COUNT(*) as count
        FROM review_history
        GROUP BY date
        ORDER BY date DESC
        LIMIT 7
    """)
    recent_days = cursor.fetchall()

    print(f"\n[OK] Created {total_reviews} review records")
    print(f"[OK] Covered {unique_words} unique words")
    print(f"\nRecent daily counts:")
    for date, count in recent_days:
        print(f"  {date}: {count} reviews")

    # Show interval statistics
    cursor.execute("""
        SELECT interval, AVG(quality) as avg_quality, COUNT(*) as count
        FROM review_history
        GROUP BY interval
        ORDER BY interval ASC
    """)
    interval_stats = cursor.fetchall()

    print(f"\nInterval statistics:")
    for interval, avg_quality, count in interval_stats[:10]:
        print(f"  Interval {interval} days: avg quality {avg_quality:.2f} ({count} reviews)")

    conn.close()
    print("\n[OK] Test data created successfully!")

if __name__ == "__main__":
    create_test_data()
