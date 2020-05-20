import sys
import sqlite3
from sqlite3 import Error
import numpy as np
from numpy import mean,std,median, absolute
import matplotlib.pyplot as plt
from scipy.stats import norm
import seaborn as sns


def create_connection(db_file):
    """ create a database connection to the SQLite database
        specified by the db_file
    :param db_file: database file
    :return: Connection object or None
    """
    conn = None
    try:
        conn = sqlite3.connect(db_file)
    except Error as e:
        print(e)

    return conn


def graph_leanness(conn):
    """
    Query all rows in the tasks table
    :param conn: the Connection object
    :return:
    """
    cur = conn.cursor()
    cur.execute("SELECT crate_name, crate_version, total_dep_func_count, used_dep_func_count FROM metrics")

    rows = cur.fetchall()

    dataset = []
    zero_counter = 0
    non_zero_low = 0
    for row in rows:
        if row[2] > 0 and row[3] > 0:
            dataset.append(row[3]/ row[2])
            if row[3] == 0 :
                zero_counter += 1
            elif row[3]/row[2] < 0.01:
                non_zero_low += 1
    

    # print(f"zero - {zero_counter}")
    # print(f"zero - {non_zero_low}")
    md = median(dataset)
    mn = mean(dataset)
    st = std(dataset)
    print(f"Lean median {md}")
    print(f"Lean mean {mn}")
    print(f"Lean std {st}")

    sns.distplot(dataset)
    plt.xlabel('Leanness index', fontsize=16)
    plt.ylabel('Share of crates', fontsize=16)
    plt.savefig("graphs/lean.png")
    plt.close()


def graph_dependency(conn):
    """
    Query all rows in the tasks table
    :param conn: the Connection object
    :return:
    """
    cur = conn.cursor()
    cur.execute("SELECT crate_name, crate_version, total_dep_func_count, total_func_count FROM metrics")

    rows = cur.fetchall()

    dataset = []

    dataset_non_zero_mean = []
    for row in rows:
        if row[3] > 0:
            dataset.append(row[2]/ row[3])
            if row[2] > 0:
                dataset_non_zero_mean.append(row[2]/ row[3])
    

    mn = mean(dataset_non_zero_mean)
    st = std(dataset_non_zero_mean)
    print(f"Dependency mean {mn}")
    print(f"Dependency std {st}")

    sns.distplot(dataset)
    plt.xlabel('Dependency index', fontsize=16)
    plt.ylabel('Share of crates', fontsize=16)
    plt.savefig("graphs/dep.png")
    plt.close()

def graph_differences(conn):
    cur = conn.cursor()
    cur.execute(f"SELECT \
        crate_name, \
        crate_version, \
        total_dep_func_count, \
        used_dep_func_count, \
        total_dep_public_func_count, \
        used_dep_public_func_count \
        FROM metrics")

    rows = cur.fetchall()

    dataset_lean = []
    dataset_cg_lean = []
    dataset = []
    for row in rows:
        call_graph_leanness = 0
        public_leanness = 0
        if row[2] > 0 and row[3] > 0:
            call_graph_leanness = row[3] / row[2]
            dataset_cg_lean.append(call_graph_leanness)
        if row[4] > 0 and row[5] > 0:
            public_leanness = row[5] / row[4]
            dataset_lean.append(public_leanness)


        if call_graph_leanness != 0 or public_leanness != 0:
            dataset.append(call_graph_leanness - public_leanness)
    


    sns.distplot(dataset)
    plt.xlabel('Difference index', fontsize=16)
    plt.ylabel('Share of crates', fontsize=16)    
    plt.savefig("graphs/diff.png")
    plt.close()

    absd = absolute(dataset)
    print(f"Absolute Mean of differences - {mean(absd)}")
    sns.distplot(absd)
    plt.xlabel('Absolute difference', fontsize=16)
    plt.ylabel('Share of crates', fontsize=16)    
    plt.savefig("graphs/diff_abs.png")
    plt.close()


    sns.distplot(dataset_lean, label='Public apis only')
    sns.distplot(dataset_cg_lean, label='Callgraph based')
    plt.xlabel('Leanness index', fontsize=16)
    plt.ylabel('Share of crates', fontsize=16)
    plt.legend(loc="upper right")
    plt.savefig("graphs/lean_comp.png")
    plt.close()

def graph_utilization_index(conn):
    cur = conn.cursor()
    cur.execute(f"SELECT \
        name, \
        version, \
        COUNT(*) AS dependents, \
        SUM(used_count) AS total_used_count, \
        SUM(total_count) AS total_total_count \
        FROM dep_metrics \
        GROUP BY name, version \
        HAVING dependents > 5")

    rows = cur.fetchall()

    dataset = []
    count_dataset = []

    for row in rows:
        # print(f"{row[0]}-{row[1]}:  {row[2]} / {row[3]} / {row[4]}")
        # call_graph_leanness = 0
        utilization_index = 0
        if row[4] > 0 and row[3] > 0:
            utilization_index = row[3] / row[4]
            dataset.append(utilization_index)
        count_dataset.append(row[2])
    # mn = mean(dataset)
    # st = std(dataset)
    # print(f"mean {mn}")
    # print(f"std {st}")

    sns.distplot(count_dataset)
    plt.xlabel('Dependent count', fontsize=16)
    plt.ylabel('Share of crates', fontsize=16)
    plt.savefig("graphs/util_count.png")
    plt.close()

    sns.distplot(dataset)
    plt.xlabel('Utilization index', fontsize=16)
    plt.ylabel('Share of crates', fontsize=16)
    plt.savefig("graphs/util.png")
    plt.close()

def main():
    database = r"prazi.db"

    # create a database connection
    conn = create_connection(database)
    with conn:
        graph_dependency(conn)
        graph_leanness(conn)
        graph_differences(conn)
        graph_utilization_index(conn)


if __name__ == '__main__':
    main()