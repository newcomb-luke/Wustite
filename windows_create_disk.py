import argparse


def main():
    parser = argparse.ArgumentParser(
        'Empty Disk Image Creator',
        description='Creates a new empty disk image file of the specified size. This is marked Windows because in Linux the dd command can be used'
    )
    parser.add_argument('path', help='The path to the disk image file to create')
    parser.add_argument('size', help='The size of the file. The default is bytes, however the suffixes K, M, and G can be used')

    args = parser.parse_args()

    size = convert_size_to_bytes(args.size.upper())

    with open(args.path, 'wb') as f:
        f.truncate(size)


def convert_size_to_bytes(size: str) -> int:
    muliplier = 1

    if 'K' in size:
        muliplier = 1_000
        size = size[:-1]
    elif 'M' in size:
        muliplier = 1_000_000
        size = size[:-1]
    elif 'G' in size:
        muliplier = 1_000_000_000
        size = size[:-1]

    try:
        return int(size) * muliplier
    except ValueError:
        print(f'Size provided: `{size}` is invalid')
        exit(1)


if __name__ == '__main__':
    main()