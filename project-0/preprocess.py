"""
This script preprocesses the latency data and saves it to a new csv file
for each workload type (immediate, const(10), const(50)) of the closed-loop
and open-loop generators.
"""
import pandas as pd
import numpy as np

DATA_DIR = "/Users/yiranshi/dev/woonsocket/data"
RUN_TIME = 20

def main():
    generators = ["closed_loop", "open_loop"]
    workloads = ["immediate", "const10", "const50"]
    threads = ["1", "2", "4", "8", "16", "32", "64", "128"]

    for generator in generators:
        # Create empty lists to store metrics
        throughputs = []
        median_latencies = []
        p95_latencies = []
        p99_latencies = []
        workload_types = []
        thread_counts = []
        
        for workload in workloads:
            for thread in threads:
                file_path = f"{DATA_DIR}/{generator}/{workload}/{thread}_latencies.txt"
                df = pd.read_csv(file_path, header=None, sep=' ', names=["latency", "timestamp", "worktime", "received_timestamp"])
                
                # Calculate metrics
                throughput = len(df) / RUN_TIME
                median_latency = df['latency'].median()
                p95_latency = df['latency'].quantile(0.95)
                p99_latency = df['latency'].quantile(0.99)
                
                # Append metrics to lists
                throughputs.append(throughput)
                median_latencies.append(median_latency)
                p95_latencies.append(p95_latency)
                p99_latencies.append(p99_latency)
                workload_types.append(workload)
                thread_counts.append(thread)
        
        # Create summary dataframe for this generator
        summary_df = pd.DataFrame({
            'throughput': throughputs,
            'median_latency': median_latencies,
            'p95_latency': p95_latencies,
            'p99_latency': p99_latencies,
            'workload': workload_types,
            'threads': thread_counts
        })

        # Save summary dataframe to CSV
        summary_file = f"{DATA_DIR}/{generator}_summary.csv"
        summary_df.to_csv(summary_file, index=False)



                

if __name__ == "__main__":
    main()