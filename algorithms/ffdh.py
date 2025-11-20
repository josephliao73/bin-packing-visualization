import json

with open("algorithm_input.json") as f:
    d = json.load(f)

bin_width = d["width_of_bin"]

items = []
for item in d["rectangle_list"]:
    for _ in range(item["quantity"]):
        items.append({
            "width": item["width"],
            "height": item["height"]
        })

items.sort(key=lambda x: -x["height"])

levels = []
placements = []

current_y = 0

for rect in items:
    w = rect["width"]
    h = rect["height"]

    placed = False

    for level in levels:
        if level["used_width"] + w <= bin_width:

            # place here
            x = level["used_width"]
            y = level["y"]

            placements.append({
                "x": x,
                "y": y,
                "width": w,
                "height": h
            })

            level["used_width"] += w
            placed = True
            break

    if not placed:
        new_level = {
            "height": h,
            "used_width": w,
            "y": current_y
        }
        levels.append(new_level)

        placements.append({
            "x": 0,
            "y": current_y,
            "width": w,
            "height": h
        })

        current_y += h

total_height = sum(level["height"] for level in levels)

output = {
    "bin_width": bin_width,
    "total_height": total_height,
    "placements": placements
}

with open("output.json", "w") as out:
    json.dump(output, out, indent=2)

print("Saved to output.json")

