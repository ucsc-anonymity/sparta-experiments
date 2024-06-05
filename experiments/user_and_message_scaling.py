import os
import subprocess

SENDS = [2**i for i in range(18, 25)]

THREADS = 48
MAPS = 5

RUNS = 10
WARMUP = 0

DATA_DIR = os.path.join(os.getcwd(), "data", "user-message-scaling")
os.makedirs(DATA_DIR, exist_ok=True)

SPARTA_DIR = os.path.join(os.getcwd(), "sparta")
BASELINE_DIR = os.path.join(os.getcwd(), "baseline")

BASELINE_FILE = os.path.join(DATA_DIR, f"baseline-{THREADS}.csv")
SPARTA_FILE = os.path.join(DATA_DIR, f"sparta-{THREADS}.csv")


def sparta_cmd(sends):
    cmd = ["cargo", "run", "--release", "--",
           str(sends), str(sends), str(THREADS), str(sends), str(MAPS), "-r", str(RUNS), "-w", str(WARMUP)]
    result = subprocess.run(cmd, capture_output=True,
                            text=True, cwd=SPARTA_DIR)
    return result.stdout


def baseline_cmd(sends):
    cmd = ["cargo", "run", "--release", "--",
           str(sends), str(sends), str(THREADS), "-r", str(RUNS), "-w", str(WARMUP)]
    result = subprocess.run(cmd, capture_output=True,
                            text=True, cwd=BASELINE_DIR)
    return result.stdout


for send in SENDS:
    print(send)

    with open(SPARTA_FILE, "a") as sparta_file:
        output = sparta_cmd(send)
        print("\tsparta:", output)
        sparta_file.write(output)

    with open(BASELINE_FILE, "a") as baseline_file:
        output = baseline_cmd(send)
        print("\tbaseline:", output)
        baseline_file.write(output)
