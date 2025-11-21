import json

with open("./bad_input.json") as f:
    d = json.load(f)

bin_width = d["width_of_bin"]

items = []
for item in d["rectangle_list"]:
    for _ in range(item["quantity"]):
        items.append({
            "width": item["width"],
            "height": item["height"]
        })

placements = []

current_level_y = 0
current_level_height = 0
current_level_used_width = 0
total_height = 0

for rect in items:
    w = rect["width"]
    h = rect["height"]


    placements.append({
        "x": 0,
        "y": total_height,
        "width": w,
        "height": h
    })

    total_height += h



output = {
    "bin_width": bin_width,
    "total_height": total_height,
    "placements": placements
}

print(json.dumps(output, indent=2))

with open("bad.json", "w") as out:
    json.dump(output, out, indent=2)
