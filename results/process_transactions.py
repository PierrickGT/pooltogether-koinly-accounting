import pandas as pd
import os
from datetime import time

def average_time(series):
    """Calculates the average time from a pandas Series of datetime objects."""
    # Convert datetimes to seconds past midnight
    # Using total_seconds() relative to the beginning of the day
    seconds = (series - series.dt.normalize()).dt.total_seconds()
    # Calculate the average seconds
    avg_seconds = seconds.mean()
    # Handle potential NaN if a group somehow ends up empty after filtering
    if pd.isna(avg_seconds):
        return None # Or return a default time like time(0, 0)

    # Clamp seconds to be within a valid day range (0 to 86399)
    avg_seconds = max(0, min(avg_seconds, 86399))

    # Convert average seconds back to a time object
    hours = int(avg_seconds // 3600)
    minutes = int((avg_seconds % 3600) // 60)
    secs = int(avg_seconds % 60)
    # microsecs = int((avg_seconds % 1) * 1_000_000) # Optional

    return time(hour=hours, minute=minutes, second=secs) #, microsecond=microsecs)

def process_csv_inplace(filepath):
    """Reads, processes, aggregates, and overwrites data in a single CSV file."""
    print(f"\n--- Processing {filepath} ---")
    try:
        # Read the CSV file
        df = pd.read_csv(filepath, skipinitialspace=True)

        if df.empty:
            print(f"Skipping empty file: {filepath}")
            return True # Indicate success (nothing to do)

        # --- Data Cleaning and Preparation ---
        # Convert 'Date' column to datetime objects, handling potential errors
        # Assuming the format is consistent based on file content
        df['DateTimeUTC'] = pd.to_datetime(df['Date'], format='%Y-%m-%d %H:%M:%S %Z', errors='coerce')

        # Drop rows where date conversion failed (if any)
        original_rows = len(df)
        df.dropna(subset=['DateTimeUTC'], inplace=True)
        rows_dropped = original_rows - len(df)
        if rows_dropped > 0:
            print(f"Warning: Dropped {rows_dropped} rows due to invalid date format in {filepath}")

        if df.empty:
            print(f"File is empty after handling invalid dates: {filepath}. Original file remains unchanged.")
            # Decide if you want to overwrite with an empty file or leave it.
            # Leaving it unchanged is safer.
            return True # Indicate success (nothing to do)


        # Extract the date part for grouping
        df['DateOnly'] = df['DateTimeUTC'].dt.date

        # --- Aggregation ---
        # Define aggregation rules
        aggregation_rules = {
            'Sent Amount': 'sum',
            'Received Amount': 'sum',
            'Fee Amount': 'sum',
            'Sent Currency': 'first',
            'Fee Currency': 'first',
            'TxHash': 'first',
            'DateTimeUTC': average_time
        }

        # Group by the extracted date and Received Currency
        grouped = df.groupby(['DateOnly', 'Received Currency'], as_index=False)

        # Apply aggregation
        aggregated_df = grouped.agg(aggregation_rules)

        # --- Reconstruct Date Column ---
        # Combine the DateOnly with the averaged time
        def combine_date_time(row):
             # Check if DateOnly and DateTimeUTC (which holds the time part now) are valid
            if pd.notna(row['DateOnly']) and isinstance(row['DateTimeUTC'], time):
                 # Combine date and time into a Timestamp
                combined_dt = pd.Timestamp.combine(row['DateOnly'], row['DateTimeUTC'])
                 # Format as string including UTC
                return combined_dt.strftime('%Y-%m-%d %H:%M:%S UTC')
            else:
                # Handle cases where aggregation might have resulted in NaT or None for time
                # Or if DateOnly is somehow invalid. Return a placeholder or original value if possible.
                # This case might need more specific handling depending on why NaT/None occurs.
                return "Invalid Date Aggregation" # Placeholder


        aggregated_df['Date'] = aggregated_df.apply(combine_date_time, axis=1)

        # --- Final Output Preparation ---
        # Select and reorder columns to match the original format
        output_columns = [
            'Date', 'Sent Amount', 'Sent Currency', 'Received Amount',
            'Received Currency', 'Fee Amount', 'Fee Currency', 'TxHash'
        ]
        final_df = aggregated_df[output_columns]

        # --- Overwrite Original File ---
        # Check if the aggregated dataframe is empty before writing
        if final_df.empty:
             print(f"Warning: Aggregation resulted in an empty dataframe for {filepath}. Original file not overwritten.")
             # Decide behavior: overwrite with header only, or leave original. Leaving original is safer.
             # To overwrite with header only: pd.DataFrame(columns=output_columns).to_csv(filepath, index=False)
             return True # Or False if this should be treated as an error
        else:
            final_df.to_csv(filepath, index=False, float_format='%.18f') # Use high precision for floats
            print(f"Successfully processed and overwrote {filepath}")
            return True # Indicate success

    except FileNotFoundError:
        print(f"Error: File not found at {filepath}")
        return False # Indicate failure
    except Exception as e:
        print(f"An error occurred processing {filepath}: {e}")
        # Optional: Consider logging the full traceback for debugging
        # import traceback
        # print(traceback.format_exc())
        return False # Indicate failure

# --- Main Script Execution ---
csv_files = [
    os.path.join( '2025-01.csv'),
    os.path.join( '2025-02.csv'),
    os.path.join( '2025-03.csv')
]

all_successful = True
for file in csv_files:
    if not process_csv_inplace(file):
        all_successful = False
        print(f"Failed to process {file}. Stopping script to prevent further issues.")
        # break # Uncomment this line if you want to stop immediately on first error

if all_successful:
    print("\n--- Script Finished Successfully ---")
else:
    print("\n--- Script Finished with Errors ---")
