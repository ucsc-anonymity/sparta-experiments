import os
import subprocess

SENDS = 2**20
USERS = 2**20

THREADS = 48
D_MAPS = 15

RUNS = 10
WARMUP = 0

DATA_DIR = os.path.join(os.getcwd(), "data", "sparta-d-legacy")
os.makedirs(DATA_DIR, exist_ok=True)

SPARTA_D_DIR = os.path.join(os.getcwd(), "sparta-d")
SPARTA_D_FILE = os.path.join(
    DATA_DIR, f"sparta-d-{SENDS}-{USERS}-{THREADS}.csv")


def sparta_d_cmd(maps):
    cmd = ["cargo", "run", "--release", "--",
           str(SENDS), str(USERS), str(THREADS), str(USERS), str(maps), "-r", str(RUNS), "-w", str(WARMUP)]
    result = subprocess.run(cmd, capture_output=True,
                            text=True, cwd=SPARTA_D_DIR)
    return result.stdout


for maps in range(1, 16):
    print(maps)

    with open(SPARTA_D_FILE, "a") as sparta_d_file:
        output = sparta_d_cmd(maps)
        print("\tsparta_d:", output)
        sparta_d_file.write(output)
