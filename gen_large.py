import argparse
import sys

DEFAULT_CONTENT = ''.join(chr(i) for i in range(256))

def main():
    args = parse_arguments()
    if args.input:
        args.content = sys.stdin.read()

    content_len = len(args.content)
    print(f"The content is {content_len} bytes big. The output file {args.filename} will be {args.length} bytes long")
    diff = args.length - content_len
    if diff > 0:
        pad_policy = "zero-padding" if args.zero_pad else "repeating content"
        print(f"The {diff} byte difference between content's length and the target output length will be made up by {pad_policy}")
    elif diff < 0:
        print("The entered content was bigger than the given length. You probably don't want to use this tool")
        return

    with open(file=args.filename, mode="w+b") as f:
        content = args.content.encode()
        f.write(content)
        if args.zero_pad:
            f.write(b'\x00' * diff)
        else:
            times_to_repeat = int(diff / content_len)
            remainder = diff % content_len
            for _ in range(times_to_repeat):
                f.write(content)
            f.write(args.content[:remainder].encode())

    print("Success")


def parse_arguments():
    parser = argparse.ArgumentParser(description="Process a filename and content string.")
    parser.add_argument("-f", "--filename", type=str, help="output filename", required=True)
    parser.add_argument("-l", "--length", type=parse_positive_int, help="Length of the output (file in bytes)", required=True)
    parser.add_argument("-c", "--content",  type=str, help="Optional content string. By default, this will just contain the bytes 0-255", default=DEFAULT_CONTENT)
    parser.add_argument("-i", "--input", action="store_true", help="Get the content string from stdin. This has precedence over '--content'")
    parser.add_argument("-z", "--zero-pad", action="store_true", help="Zero pad the content string to the desired length. If this option is not selected, then the string will be repeated instead")
    return parser.parse_args()


def parse_positive_int(value):
    num = int(value)
    if num <= 0:
        raise argparse.ArgumentTypeError(f"{value} is not a positive integer")
    return num

if __name__ == "__main__":
    main()
