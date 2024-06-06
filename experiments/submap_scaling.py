import os
import subprocess

SENDS = 1048576
FETCHES = 8192
USERS = FETCHES

THREADS = 8
MAP_THREADS = [(i, 8 + 8 * i) for i in range(6, int(16))]
# MAP_THREADS = [(i, 8 + 8 * i) for i in range(1, int(48 / THREADS))]

RUNS = 10
WARMUP = 0

DATA_DIR = os.path.join(os.getcwd(), "data", "submap-scaling")
os.makedirs(DATA_DIR, exist_ok=True)

SPARTA_DIR = os.path.join(os.getcwd(), "sparta")
SPARTA_D_DIR = os.path.join(os.getcwd(), "sparta-d")
BASELINE_DIR = os.path.join(os.getcwd(), "baseline")

BASELINE_FILE = os.path.join(DATA_DIR, f"baseline-{FETCHES}-{THREADS}.csv")
SPARTA_FILE = os.path.join(DATA_DIR, f"sparta-{FETCHES}-{THREADS}.csv")
SPARTA_D_FILE = os.path.join(DATA_DIR, f"sparta-d-{FETCHES}-{THREADS}.csv")


def sparta_cmd(mt):
    (num_maps, num_threads) = mt
    cmd = ["cargo", "run", "--release", "--",
           str(SENDS), str(FETCHES), str(num_threads),
           str(USERS), str(num_maps), "-r", str(RUNS), "-w", str(WARMUP)]
    result = subprocess.run(cmd, capture_output=True,
                            text=True, cwd=SPARTA_DIR)
    return result.stdout


def sparta_d_cmd(mt):
    (num_maps, _num_threads) = mt
    cmd = ["cargo", "run", "--release", "--",
           str(SENDS), str(FETCHES), str(48),
           str(USERS), str(num_maps), "-r", str(RUNS), "-w", str(WARMUP)]
    result = subprocess.run(cmd, capture_output=True,
                            text=True, cwd=SPARTA_D_DIR)
    return result.stdout


def baseline_cmd(mt):
    (_num_maps, num_threads) = mt
    cmd = ["cargo", "run", "--release", "--",
           str(SENDS), str(FETCHES), str(num_threads), "-r", str(RUNS), "-w", str(WARMUP)]
    result = subprocess.run(cmd, capture_output=True,
                            text=True, cwd=BASELINE_DIR)
    return result.stdout


for mt in MAP_THREADS:
    print(mt)

    # with open(SPARTA_FILE, "a") as sparta_file:
    #     output = sparta_cmd(mt)
    #     print("\tsparta:", output)
    #     sparta_file.write(output)

    with open(SPARTA_D_FILE, "a") as sparta_d_file:
        output = sparta_d_cmd(mt)
        print("\tsparta_d:", output)
        sparta_d_file.write(output)

    # with open(BASELINE_FILE, "a") as baseline_file:
    #     output = baseline_cmd(mt)
    #     print("\tbaseline:", output)
    #     baseline_file.write(output)
