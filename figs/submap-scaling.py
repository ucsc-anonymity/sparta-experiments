import matplotlib.pyplot as plt
import numpy as np
import os
import pandas as pd

DATA_DIR = os.path.join(os.path.abspath("data"), "submap-legacy")
BASELINE = os.path.join(DATA_DIR, "baseline-1048576-8.csv")
SPARTA = os.path.join(DATA_DIR, "sparta-1048576-8.csv")
SPARTAD = os.path.join(DATA_DIR, "sparta-d-1048576-8.csv")

base = pd.read_csv(BASELINE, sep="\t", header=None)
base = base.iloc[:, 1:-1]
base_mean = base.mean(axis=1)
base_std = base.std(axis=1)

sparta = pd.read_csv(SPARTA, sep="\t", header=None)
sparta = sparta.iloc[:, 1:-1]
sparta_mean = sparta.mean(axis=1)
sparta_std = sparta.std(axis=1)

spartad = pd.read_csv(SPARTAD, sep="\t", header=None)
spartad = spartad.iloc[:, 1:-1]
spartad_mean = spartad.mean(axis=1)
spartad_std = spartad.std(axis=1)

x = [i for i in range(1, 6)]


plt.plot(x, base_mean, linewidth=4, label="Sparta-SB")
plt.plot(x, sparta_mean, linewidth=4, label="Sparta-D")
plt.plot(x, spartad_mean[0:5],
         linewidth=4, label="Distributed Sparta-D")

# Add titles and labels
plt.xlabel("Compute", fontweight="bold", fontsize=24)
plt.ylabel("Latency (s)", fontweight="bold", fontsize=24)
# Set line width for top spine (x-axis)
plt.gca().spines['top'].set_linewidth(4)
# Set line width for bottom spine (x-axis)
plt.gca().spines['bottom'].set_linewidth(4)
# Set line width for left spine (y-axis)
plt.gca().spines['left'].set_linewidth(4)
plt.gca().spines['right'].set_linewidth(4)
plt.tight_layout()
plt.legend()
plt.legend(fontsize="x-large")
plt.ylim(0)
# plt.xscale("log")
plt.xticks(list(range(1, 6)), fontsize=16)
plt.yticks(fontsize=16)


# Show the plot
plt.savefig("compute-scaling.pdf", format="pdf")
plt.show()
