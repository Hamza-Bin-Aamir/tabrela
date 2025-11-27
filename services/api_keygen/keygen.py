import argparse
import random
import string

def generate_password(length: int) -> str:
    """
    Generates a random password of a specified length.

    The password consists of lowercase letters, uppercase letters, and digits.
    """
    if length <= 0:
        raise ValueError("Password length must be a positive integer.")

    # Define the pool of characters to choose from
    characters = string.ascii_letters + string.digits
    
    # Use random.choice to select characters and join them into a string
    password = ''.join(random.choice(characters) for _ in range(length))
    
    return password

def main():
    """
    Main function to handle CLI arguments and output the password.
    """
    parser = argparse.ArgumentParser(
        description="A simple CLI tool to generate a random password of a specified length.",
        formatter_class=argparse.RawTextHelpFormatter
    )
    
    # Define the required 'length' argument
    parser.add_argument(
        'length', 
        type=int, 
        help="The desired length of the password (e.g., 12, 16)."
    )
    
    args = parser.parse_args()
    
    try:
        password = generate_password(args.length)
        print(f"Generated Password ({args.length} chars): {password}")
    except ValueError as e:
        # Handle the error if the length is invalid (e.g., 0 or negative)
        print(f"Error: {e}")
        # Exit with a non-zero status code to indicate failure
        exit(1)

if __name__ == "__main__":
    main()