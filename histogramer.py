import sqlite3
from sqlite3 import Error
import numpy as np
from numpy import mean,std
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
        if row[2] > 0:
            dataset.append(row[3]/ row[2])
            if row[3] == 0 :
                zero_counter += 1
            elif row[3]/row[2] < 0.01:
                non_zero_low += 1
    

    print(f"zero - {zero_counter}")
    print(f"zero - {non_zero_low}")
    mn = mean(dataset)
    st = std(dataset)
    print(f"mean {mn}")
    print(f"std {st}")

    sns.distplot(dataset)
    plt.show()


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
    for row in rows:
        if row[3] > 0:
            dataset.append(row[2]/ row[3])
    

    mn = mean(dataset)
    st = std(dataset)
    print(f"mean {mn}")
    print(f"std {st}")

    sns.distplot(dataset)
    plt.show()

def main():
    database = r"prazi.db"

    # create a database connection
    conn = create_connection(database)
    with conn:
        graph_leanness(conn)


if __name__ == '__main__':
    main()