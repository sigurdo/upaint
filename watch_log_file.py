#!/usr/bin/python3

import argparse
import inspect
import os
import subprocess
import time
import math


def r(*path):
    """
    Takes a relative path from the directory of this python file and returns the absolute path.
    """
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), *path)


def run_in_dir(directory, callable):
    cwd = os.getcwd()
    os.chdir(directory)
    result = callable()
    os.chdir(cwd)
    return result


if __name__ == "__main__":
    try:
        argument_parser = argparse.ArgumentParser()
        argument_parser.add_argument("file", type=str, help="Path to file to log")
        args = argument_parser.parse_args()

        length_printed = 0
        last_time_printed = 0
        while True:
            time.sleep(1e-3)
            if os.stat(args.file).st_mtime > last_time_printed:
                with open(args.file, "r") as file:
                    contents = file.read()
                if len(contents) < length_printed:
                    length_printed = 0
                print(contents[length_printed:], end="")
                length_printed = len(contents)
    except KeyboardInterrupt:
        print()
