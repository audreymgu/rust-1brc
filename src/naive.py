"""

10M: 413 stations, max 1 decimal place
"""

import mmap
import multiprocessing as mp
import sys
from collections import defaultdict
from os.path import getsize
from time import perf_counter

FILE = "../measurements_10m.txt"
CORES = 18


def f1():
    """Unoptimized approach"""
    mins = defaultdict(lambda: float("inf"))
    maxs = defaultdict(lambda: float("-inf"))
    avgs = defaultdict(list)

    # Bamako;38.8
    with open(FILE) as f:
        for line in f:
            station, temp = line.strip().split(";")
            temp = float(temp)
            mins[station] = min(mins[station], temp)
            maxs[station] = max(maxs[station], temp)
            avgs[station].append(temp)
    for station in mins:
        print(
            f"{station}: {mins[station]}, {sum(avgs[station]) / len(avgs[station]):,.2f}, {maxs[station]}"
        )


def stripless():
    """Unoptimized approach"""
    mins = defaultdict(lambda: float("inf"))
    maxs = defaultdict(lambda: float("-inf"))
    avgs = defaultdict(list)

    # Bamako;38.8
    with open(FILE) as f:
        for line in f:
            station, temp = line[:-1].split(";")
            temp = float(temp)
            mins[station] = min(mins[station], temp)
            maxs[station] = max(maxs[station], temp)
            avgs[station].append(temp)
    for station in mins:
        print(
            f"{station}: {mins[station]}, {sum(avgs[station]) / len(avgs[station]):,.2f}, {maxs[station]}"
        )


def defaultdictless():
    """Unoptimized approach"""
    mins = {}
    maxs = {}
    avgs = {}

    # Bamako;38.8
    with open(FILE) as f:
        for line in f:
            station, temp = line.strip().split(";")
            temp = float(temp)
            try:
                if mins[station] > temp:
                    mins[station] = temp
            except KeyError:
                mins[station] = temp
            try:
                if maxs[station] < temp:
                    maxs[station] = temp
            except KeyError:
                maxs[station] = temp
            try:
                avgs[station].append(temp)
            except KeyError:
                avgs[station] = [temp]
    for station in mins:
        print(
            f"{station}: {mins[station]}, {sum(avgs[station]) / len(avgs[station]):,.2f}, {maxs[station]}"
        )


def binary_read_no_float():
    # min, max, sum, count
    stats = {}

    # Bamako;38.8
    with open(FILE, "rb") as f:
        for line in f:
            sc = line.rfind(b";")
            station = line[:sc]
            temp = parse_temp(line[sc + 1 : -1])

            try:
                s = stats[station]
                if temp < s[0]:
                    s[0] = temp
                if temp > s[1]:
                    s[1] = temp
                s[2] += temp
                s[3] += 1
            except KeyError:
                stats[station] = [temp, temp, temp, 1]
    for station in sorted(stats):
        min_, max_, sum_, count = stats[station]
        print(
            f"{station.decode()}: {min_ / 10:.1f}, {sum_ / count / 10:,.2f}, {max_ / 10:.1f}"
        )


def parse_temp(b):
    if b[0] == 0x2D:  # b'-'
        b = b[1:]
        neg = True
    else:
        neg = False
    if len(b) == 4:  # NN.N
        n = (b[0] - 48) * 100 + (b[1] - 48) * 10 + (b[3] - 48)
    else:  # N.N
        n = (b[0] - 48) * 10 + (b[2] - 48)
    return -n if neg else n


def in_line_temp():
    # min, max, sum, count
    stats = {}

    # Bamako;38.8
    with open(FILE, "rb") as f:
        for line in f:
            station, tb = line.split(b";")
            if tb[0] == 0x2D:
                if len(tb) == 6:
                    temp = -((tb[1] - 48) * 100 + (tb[2] - 48) * 10 + (tb[4] - 48))
                else:
                    temp = -((tb[1] - 48) * 10 + (tb[3] - 48))
            elif len(tb) == 5:
                temp = (tb[0] - 48) * 100 + (tb[1] - 48) * 10 + (tb[3] - 48)
            else:
                temp = (tb[0] - 48) * 10 + (tb[2] - 48)

            try:
                s = stats[station]
                if temp < s[0]:
                    s[0] = temp
                if temp > s[1]:
                    s[1] = temp
                s[2] += temp
                s[3] += 1
            except KeyError:
                stats[station] = [temp, temp, temp, 1]
    for station in sorted(stats):
        min_, max_, sum_, count = stats[station]
        print(
            f"{station.decode()}: {min_ / 10:.1f}, {sum_ / count / 10:,.2f}, {max_ / 10:.1f}"
        )


def profile(f):
    start = perf_counter()
    f()
    end = perf_counter()
    return f"Time elapsed: {round(end - start, 2)} seconds"


def get_chunks():
    size = getsize(FILE)
    chunk_size = size // CORES
    chunks = []
    for i in range(CORES):
        start_byte = i * chunk_size
        end_byte = start_byte + chunk_size if i < CORES - 1 else size
        chunks.append((start_byte, end_byte))
    return chunks


def process_chunk(chunk):
    start_byte, end_byte = chunk
    stats = {}

    with open(FILE, "rb") as f:
        f.seek(start_byte)

        if start_byte != 0:
            f.readline()

        current_byte = f.tell()

        while current_byte < end_byte:
            line = f.readline()
            if not line:
                break
            current_byte += len(line)

            station, tb = line.split(b";")
            if tb[0] == 0x2D:
                if len(tb) == 6:
                    temp = -((tb[1] - 48) * 100 + (tb[2] - 48) * 10 + (tb[4] - 48))
                else:
                    temp = -((tb[1] - 48) * 10 + (tb[3] - 48))
            elif len(tb) == 5:
                temp = (tb[0] - 48) * 100 + (tb[1] - 48) * 10 + (tb[3] - 48)
            else:
                temp = (tb[0] - 48) * 10 + (tb[2] - 48)

            try:
                s = stats[station]
                if temp < s[0]:
                    s[0] = temp
                if temp > s[1]:
                    s[1] = temp
                s[2] += temp
                s[3] += 1
            except KeyError:
                stats[station] = [temp, temp, temp, 1]

    return stats


def process_mmap_chunk(chunk):
    start_byte, end_byte = chunk
    stats = {}

    with open(FILE, "rb") as f:
        mm = mmap.mmap(f.fileno(), 0, access=mmap.ACCESS_READ)

        current_pos = start_byte
        if start_byte != 0:
            first_newline = mm.find(b"\n", start_byte)
            if first_newline == -1:
                mm.close()
                return stats
            current_pos = first_newline + 1

        while current_pos < end_byte:
            semicolon_pos = mm.find(b";", current_pos)
            if semicolon_pos == -1:
                break

            newline_pos = mm.find(b"\n", semicolon_pos)
            if newline_pos == -1:
                newline_pos = mm.size()

            station = mm[current_pos:semicolon_pos]
            tb = mm[semicolon_pos + 1 : newline_pos]
            if tb[0] == 0x2D:
                if len(tb) == 5:
                    temp = -((tb[1] - 48) * 100 + (tb[2] - 48) * 10 + (tb[4] - 48))
                else:
                    temp = -((tb[1] - 48) * 10 + (tb[3] - 48))
            elif len(tb) == 4:
                temp = (tb[0] - 48) * 100 + (tb[1] - 48) * 10 + (tb[3] - 48)
            else:
                temp = (tb[0] - 48) * 10 + (tb[2] - 48)

            try:
                s = stats[station]
                if temp < s[0]:
                    s[0] = temp
                if temp > s[1]:
                    s[1] = temp
                s[2] += temp
                s[3] += 1
            except KeyError:
                stats[station] = [temp, temp, temp, 1]

            current_pos = newline_pos + 1

        mm.close()

    return stats


def merge_stats(stats):
    merged_stats = {}
    for worker_stats in stats:
        for station, station_stats in worker_stats.items():
            if station not in merged_stats:
                merged_stats[station] = station_stats
            else:
                w_min, w_max, w_sum, w_count = station_stats
                if merged_stats[station][0] > w_min:
                    merged_stats[station][0] = w_min
                if merged_stats[station][1] < w_max:
                    merged_stats[station][1] = w_max
                merged_stats[station][2] += w_sum
                merged_stats[station][3] += w_count
    return merged_stats


import hashlib


def stats_hash(stats):
    h = hashlib.sha256()
    for station in sorted(stats):
        mn, mx, sm, ct = stats[station]
        h.update(f"{station.decode()}:{mn},{mx},{sm},{ct}\n".encode())
    return h.hexdigest()


if __name__ == "__main__":
    print(profile(f1))

    # b7a40f627e7a667bf67ae84172b2e7f6e8c8e0b5f3d8f9be9b10d9c96c0f8520
