import matplotlib.pyplot as plt
import numpy as np
import os
import pandas as pd

DATA = os.path.join(os.path.abspath("data"), "d-scaling",
                    "sparta-d-1048576-8192-48.csv")
df = pd.read_csv(DATA, sep="\t", header=None)
df = df.iloc[:, 1:-1]


means = df.mean(axis=1)
mean_stds = df.std(axis=1)
print(mean_stds)
x = range(1, len(means) + 1)

throughputs = 131072 / df
mean_throughput = throughputs.mean(axis=1)
mean_throughput_stds = throughputs.std(axis=1)

fig, ax1 = plt.subplots()

# # Plot the means with error bars on the first axis
ax1.plot(x, means, linewidth=4, label="Latency")
ax1.set_ylabel("Latency (s)", fontweight="bold", fontsize=24)

ax1.set_xlabel("Subqueues", fontweight="bold", fontsize=24)
plt.setp(ax1.get_xticklabels(), fontsize=24)
ax1.set_ylim(0)

# # Create a second y-axis for the median values
ax2 = ax1.twinx()
plt.setp(ax2.get_xticklabels(), fontsize=24)
ax2.plot(x, mean_throughput, 'g', linewidth=4, label='Throughput')
ax2.set_ylabel("Throughput (msg/s)", fontweight="bold", fontsize=24)
# ax2.set_ylabel('Median Values')
# ax2.legend(loc='upper right')

handles, labels = ax1.get_legend_handles_labels()
handles2, labels2 = ax2.get_legend_handles_labels()

# Add legends to subplots
ax1.legend(handles + handles2, labels + labels2,
           loc="lower right", fontsize=24)

ax1.tick_params(axis='both', which='major', labelsize=16)
ax2.tick_params(axis='both', which='major', labelsize=16)


# Add titles and labels
plt.xlabel("# Subqueues", fontweight="bold", fontsize=24)
plt.xticks(fontsize=16)
# Set line width for top spine (x-axis)
plt.gca().spines['top'].set_linewidth(4)
# Set line width for bottom spine (x-axis)
plt.gca().spines['bottom'].set_linewidth(4)
# Set line width for left spine (y-axis)
plt.gca().spines['left'].set_linewidth(4)
plt.gca().spines['right'].set_linewidth(4)
plt.tight_layout()

# Show the plot
plt.savefig("d-scaling.pdf", format="pdf")
plt.show()
