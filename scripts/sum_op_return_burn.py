import csv
import argparse

def calculate_total_opburn(file_path, height):
    total_opburn = 0

    with open(file_path, mode='r') as file:
        csv_reader = csv.DictReader(file)

        headers = csv_reader.fieldnames
        print(f"CSV Headers: {headers}")

        for row in csv_reader:
            try:
                addrbal = row['height;txid;amount']
                if int(addrbal.split(';')[0]) > int(height):
                    return total_opburn
                opburn = int(addrbal.split(';')[2])
                total_opburn += opburn
            except KeyError:
                print("Error: 'opburn' column not found in CSV file.")
                return None

    return total_opburn

def main():
    parser = argparse.ArgumentParser(description='Calculate the total opburn from a CSV file.')
    parser.add_argument('file_path', type=str, help='The path to the CSV file')
    parser.add_argument('height', type=str, help='The op_return height')

    args = parser.parse_args()

    total_opburn = calculate_total_opburn(args.file_path, args.height)
    if total_opburn is not None:
        print(f'Total opburn: {total_opburn}')
    else:
        print("Failed to calculate total opburn due to missing 'opburn' column.")

if __name__ == '__main__':
    main()