import csv
import statistics
import sys

def process_csv(file_path):
    # Increase the field size limit
    csv.field_size_limit(sys.maxsize)
    
    with open(file_path, 'r', encoding='utf-8') as file:
        reader = csv.reader(file, delimiter=';')
        column_three_data = []
        temp_string = ""
        line_count = 0

        for row in reader:
            if len(row) < 3:
                # Handle lines that are part of the same third column
                temp_string += "".join(row)
                continue
            elif temp_string:
                # Complete the third column data and reset temp_string
                temp_string += "".join(row)
                column_three_data.append(temp_string.strip())
                temp_string = ""
                line_count += 1
            else:
                # Normal case
                column_three_data.append(row[2].strip())
                line_count += 1

        # In case the last entry is not processed
        if temp_string:
            column_three_data.append(temp_string.strip())
            line_count += 1

        # Calculate the required statistics
        char_counts = [len(data) for data in column_three_data]
        byte_counts = [len(data.encode('utf-8')) for data in column_three_data]

        total_chars = sum(char_counts)
        max_chars = max(char_counts)
        mean_chars = statistics.mean(char_counts) if char_counts else 0

        total_bytes = sum(byte_counts)
        max_bytes = max(byte_counts)
        mean_bytes = statistics.mean(byte_counts) if byte_counts else 0

        return line_count, total_chars, max_chars, mean_chars, total_bytes, max_bytes, mean_bytes

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python script.py <file_path>")
        sys.exit(1)

    file_path = sys.argv[1]
    line_count, total_chars, max_chars, mean_chars, total_bytes, max_bytes, mean_bytes = process_csv(file_path)
    print(f"Total Lines: {line_count}")
    print(f"Total Characters: {total_chars}")
    print(f"Max Characters: {max_chars}")
    print(f"Mean Characters: {mean_chars:.2f}")
    print(f"Total Bytes: {total_bytes}")
    print(f"Max Bytes: {max_bytes}")
    print(f"Mean Bytes: {mean_bytes:.2f}")
