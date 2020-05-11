import json
import sys


new_graph = []
grouped_outward_edges = {}
grouped_inward_edges = {}
path = sys.argv[1]

initial_path = path + "/callgraph.json"
with open(initial_path) as json_file:
    data = json.load(json_file)

    for edge in data["function_calls"]:
        if edge[0] not in grouped_outward_edges:
            grouped_outward_edges[edge[0]] = []
        new_edge = {}
        new_edge["target"] = edge[1]
        new_edge["some_bool"] = edge[2]
        grouped_outward_edges[edge[0]].append(new_edge)

        if edge[1] not in grouped_inward_edges:
            grouped_inward_edges[edge[1]] = []
        new_in_edge = {}
        new_in_edge["target"] = edge[0]
        new_in_edge["some_bool"] = edge[2]
        grouped_inward_edges[edge[1]].append(new_in_edge)



    for node in data["functions"]:
        node_edges = []
        # for edge in data["edges"]:
        #     if edge[0] == node["id"]:
        #         new_edge = {}
        #         new_edge["target"] = edge[1]
        #         new_edge["some_bool"] = edge[2]
        #         node_edges.append(new_edge)

        if node["id"] in grouped_inward_edges:
            node["inward_edges"] = grouped_inward_edges[node["id"]]
        else:
            node["inward_edges"] = []

        if node["id"] in grouped_outward_edges:
            node["outward_edges"] = grouped_outward_edges[node["id"]]
        else:
            node["outward_edges"] = []

        new_graph.append(node)

updated_path = path + "/updated_callgraph.json"
with open(updated_path, 'w') as outfile:
    json.dump(new_graph, outfile)