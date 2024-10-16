import matplotlib.pyplot as plt
import numpy as np
import os
import pandas as pd

DATA_DIR = os.path.join(os.path.abspath("data"), "user-message-scaling")
BASELINE = os.path.join(DATA_DIR, "baseline-48.csv")
SPARTA = os.path.join(DATA_DIR, "sparta-48.csv")
SPARTAD = os.path.join(DATA_DIR, "sparta-d-15.csv")

base = pd.read_csv(BASELINE, sep="\t", header=None)
base = base.iloc[:, 1:]
print(base)
base_mean = base.mean(axis=1)
base_std = base.std(axis=1)

sparta = pd.read_csv(SPARTA, sep="\t", header=None)
sparta = sparta.iloc[:, 1:]
print(sparta)
sparta_mean = sparta.mean(axis=1)
sparta_std = sparta.std(axis=1)

spartad = pd.read_csv(SPARTAD, sep="\t", header=None)
spartad = spartad.iloc[:, 1:]
print(spartad)
spartad_mean = spartad.mean(axis=1)
spartad_std = spartad.std(axis=1)

x = [2**i for i in range(18, 24)]


plt.plot(x, base_mean, linewidth=2, label="Baseline")
plt.plot(x, sparta_mean, linewidth=2, label="Sparta")
plt.plot(x, spartad_mean,
         linewidth=2, label="Sparta (D)")

# Add titles and labels
plt.title("Scale vs. Latency", fontweight="bold", fontsize=16)
plt.xlabel("Scale", fontweight="bold", fontsize=16)
plt.ylabel("Latency (s)", fontweight="bold", fontsize=16)

plt.gca().spines['top'].set_linewidth(2)
plt.gca().spines['bottom'].set_linewidth(2)
plt.gca().spines['left'].set_linewidth(2)
plt.gca().spines['right'].set_linewidth(2)

plt.tight_layout()
plt.legend()
plt.ylim(0)
plt.xscale("log")
# plt.xticks(x)

# Show the plot
plt.savefig("user-message-scaling.pdf", format="pdf")
# plt.show()
