import sys
import sqlite3
from sqlite3 import Error
import numpy as np
from numpy import mean,std,median, absolute
import matplotlib.pyplot as plt
from scipy.stats import norm
import seaborn as sns
import scipy
from dictances import bhattacharyya


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

def isclose(a, b):
    return abs(a-b) <= 0.01

def graph_leanness(conn):
    """
    Query all rows in the tasks table
    :param conn: the Connection object
    :return:
    """
    cur = conn.cursor()
    cur.execute("SELECT crate_name, crate_version, total_dep_func_count, used_dep_func_count, COUNT(dep_metrics.crate_id) as dep_count \
        FROM metrics \
        INNER JOIN dep_metrics ON metrics.id = dep_metrics.crate_id \
        group by metrics.id")

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


    # for row in rows:
    #     if row[2] > 0 and row[3] > 0 and row[4] > 5 and row[4] < 10:
    #         if isclose(row[3] / row[2], md):
    #             print(f"Median package is {row[0]}-{row[1]}. Dep count - {row[4]}")
                

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
    cur.execute("SELECT crate_name, crate_version, total_dep_LOC, total_LOC FROM metrics")

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

    zeroes = len([i for i in dataset if i == 0])
    print(f"0 LOC in dependencies -  {zeroes / len(dataset)}")

    a = np.array(dataset)
    print(f"95th percentile of dependency index  - {np.percentile(a, 95)}")

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
        total_dep_LOC, \
        used_dep_LOC, \
        total_public_LOC, \
        used_public_LOC \
        FROM metrics")

    rows = cur.fetchall()

    dataset_lean = []
    dataset_cg_lean = []
    dataset = []


    dataset_lean_cov = []
    dataset_cg_lean_cov = []

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
            dataset_lean_cov.append(public_leanness)
            dataset_cg_lean_cov.append(call_graph_leanness)
            dataset.append(call_graph_leanness - public_leanness)

    pdf = norm.pdf(dataset_lean)
    pdf2 = norm.pdf(dataset_lean)

    divergence = scipy.stats.entropy(dataset_lean_cov, dataset_cg_lean_cov, base=None)

    # distance = bhattacharyya(pdf, pdf2)
    print(f"Distance = {divergence}")

    sns.distplot(dataset)
    plt.xlabel('Difference index', fontsize=16)
    plt.ylabel('Share of crates', fontsize=16)    
    plt.savefig("graphs/diff.png")
    plt.close()

    absd = absolute(dataset)

    diffmorethan20 = len([i for i in absd if i > 0.20])
    print(f"Diff more than 20% -  {diffmorethan20 / len(absd)}")
    a = np.array(absd)
    p = np.percentile(a, 50)
    print(f"5th percentile abs diff  - {np.percentile(a, 95)}")
    print(f"10th percentile abs diff - {np.percentile(a, 90)}")
    print(f"20th percentile abs diff - {np.percentile(a, 80)}")
    print(f"30th percentile abs diff - {np.percentile(a, 70)}")


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

def new_util_index(conn):
    cur = conn.cursor()
    cur.execute(f"SELECT \
        id \
        FROM dep")

    rows = cur.fetchall()
    dep_ids = []
    for row in rows:
        dep_ids.append(row[0])

    for i in dep_ids:
        sql2 = conn.cursor()
        print(f"Getting index for {i}")
        sql2.execute(f"SELECT \
            count(*) \
            FROM dep_func_metrics \
            where dep_id = {i} and use_count = 0")        
        zeroUseFuncs = sql2.fetchall()
        print(f"Dep {i} has {zeroUseFuncs} unused funcs")

def graph_utilization_index(conn):
    cur = conn.cursor()
    cur.execute(f"SELECT \
        name, \
        version, \
        COUNT(*) AS dependents, \
        SUM(used_count) AS used_LOC, \
        SUM(total_count) AS total_LOC \
        FROM dep_metrics \
        GROUP BY name, version \
        HAVING dependents > 5")

    rows = cur.fetchall()

    dataset = []
    count_dataset = []
    highest_count = 0
    highest_name = ""

    for row in rows:
        if row[2] > highest_count:
            highest_count = row[2]
            highest_name = row[0]
        
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

    print(f"Highest depdent count - {highest_name}")

    lessthan10 = len([i for i in dataset if i < 0.10])
    print(f"Utilization less than 10% -  {lessthan10 / len(dataset)}")

    a = np.array(dataset)
    print(f"95th percentile of utilization index  - {np.percentile(a, 95)}")


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

def graph_leanness_loc(conn):
    cur = conn.cursor()
    cur.execute(f"SELECT \
        crate_name, \
        crate_version, \
        total_dep_func_count, \
        used_dep_func_count, \
        total_dep_loc, \
        used_dep_LOC  \
        FROM metrics")

    rows = cur.fetchall()

    dataset_node = []
    dataset_LOC = []
    dataset = []
    for row in rows:
        nodes = 0
        loc = 0
        if row[2] > 0 and row[3] > 0:
            nodes = row[3] / row[2]
            dataset_node.append(nodes)
        if row[4] > 0 and row[5] > 0:
            loc = row[5] / row[4]
            dataset_LOC.append(loc)


        if dataset_LOC != 0 or dataset_node != 0:
            dataset.append(nodes - loc)
    
    sns.distplot(dataset)
    plt.xlabel('Leanness', fontsize=16)
    plt.ylabel('Share of crates', fontsize=16)    
    plt.savefig("graphs/loc_diff.png")
    plt.close()

    absd = absolute(dataset)
    sns.distplot(absd)
    plt.xlabel('Absolute difference', fontsize=16)
    plt.ylabel('Share of crates', fontsize=16)    
    plt.savefig("graphs/loc_diff_abs.png")
    plt.close()


    lean_below5 = len([i for i in dataset_node if i < 0.05])
    print(f"Lean less than 5% (NCO) -  {lean_below5 / len(dataset_node)}")
    lean_below5loc = len([i for i in dataset_LOC if i < 0.05])
    print(f"Lean less than 5% (LOC) -  {lean_below5loc / len(dataset_LOC)}")

    lean_below25 = len([i for i in dataset_node if i < 0.25])
    print(f"Lean less than 25% (NCO) -  {lean_below25 / len(dataset_node)}")
    lean_below25loc = len([i for i in dataset_LOC if i < 0.25])
    print(f"Lean less than 25% (LOC) -  {lean_below25loc / len(dataset_LOC)}")

    a = np.array(dataset_node)
    p = np.percentile(a, 95)
    print(f"95th percentile (NCO) - {p}")
    a = np.array(dataset_LOC)
    p = np.percentile(a, 95)
    print(f"95th percentile (LOC) - {p}")

    md = median(dataset_LOC)
    mn = mean(dataset_LOC)
    st = std(dataset_LOC)
    print(f"LOC Lean median {md}")
    print(f"LOC Lean mean {mn}")
    print(f"LOC Lean std {st}")

    sns.distplot(dataset_node, label='Leanness nodes')
    sns.distplot(dataset_LOC, label='Leanness LOC')
    plt.xlabel('Leanness index', fontsize=16)
    plt.ylabel('Share of crates', fontsize=16)
    plt.legend(loc="upper right")
    plt.savefig("graphs/loc_comp.png")
    plt.close()

def get_single_package_data(conn, name, version):
    cur = conn.cursor()
    cur.execute(f"SELECT \
        crate_name, \
        crate_version, \
        total_func_count, \
        local_func_count, \
        std_func_count, \
        total_dep_func_count, \
        used_dep_func_count, \
        total_dep_public_func_count, \
        used_dep_public_func_count, \
        total_LOC, \
        local_LOC, \
        total_dep_LOC, \
        used_dep_LOC \
        total_public_LOC, \
        used_public_LOC, \
        id \
        FROM metrics \
        WHERE crate_name = '{name}' AND crate_version = '{version}'")

    rows = cur.fetchall()
    crate_info = rows[0]
    # for row in rows:
    #     print(row)

    # Pie chart
    labels = ["Local code", "Crates"]
    pie_data = [crate_info[3], crate_info[5]]
    
    fig1, ax1 = plt.subplots()
    ax1.pie(pie_data, labels=labels, autopct='%1.1f%%', startangle=90)
    ax1.axis('equal')  # Equal aspect ratio ensures that pie is drawn as a circle.

    plt.savefig(f"graphs/{name}_pie_chart.png")
    plt.close()
    cur = conn.cursor()
    cur.execute(f"SELECT \
        name, \
        version, \
        total_count, \
        used_count, \
        total_LOC, \
        used_LOC \
        FROM dep_metrics \
        WHERE crate_id = {crate_info[14]}")

    rows = cur.fetchall()

    dep_labels = []
    dep_pie_data = []
    print(f"=========={name}============")
    for row in rows:
        dep_pie_data.append([f"{row[0]}-{row[1]}", row[4]])
        adjusted_name = f"{row[0]}".ljust(30)
        if row[4] > 0:
            print(f"{adjusted_name} - {row[5]/row[4]}")

    dep_pie_data.sort(key=lambda x: x[1], reverse = True)
    dep_pie_data = np.array(dep_pie_data)

    fig1, ax1 = plt.subplots()
    patches, texts  = ax1.pie(dep_pie_data[:,1], startangle=90)
    # plt.legend(patches, labels=['%s, %1.1f %%' % (l, s) for l, s in zip(dep_pie_data[:,0], dep_pie_data[:,1])] , loc="best")
    plt.legend(patches, dep_pie_data[:,0])
    ax1.axis('equal')  # Equal aspect ratio ensures that pie is drawn as a circle.

    plt.savefig(f"graphs/{name}_dep_pie_chart.png")
    plt.close()
    


def main():
    database = r"prazi.db"

    # create a database connection
    conn = create_connection(database)
    with conn:
        # get_single_package_data(conn, "pango", "0.8.0")
        # get_single_package_data(conn, "rand", "0.7.3")
        # get_single_package_data(conn, "serde", "1.0.104")
        # graph_dependency(conn)
        # graph_leanness(conn)
        # graph_differences(conn)
        # graph_utilization_index(conn)
        # graph_leanness_loc(conn)
        new_util_index(conn)

if __name__ == '__main__':
    main()