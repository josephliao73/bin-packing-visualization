import json

with open("test.json") as f:
    d = json.load(f)

bin_width = d["width_of_bin"]

# Step 1: expand rectangles into individual items
items = []
for item in d["rectangle_list"]:
    for _ in range(item["quantity"]):
        items.append({
            "width": item["width"],
            "height": item["height"]
        })

# Step 2: sort by height decreasing (NFDH requirement)
items.sort(key=lambda x: -x["height"])

placements = []

current_level_y = 0
current_level_height = 0
current_level_used_width = 0

first_item = True

for rect in items:
    w = rect["width"]
    h = rect["height"]

    if first_item:
        current_level_height = h
        first_item = False

    if current_level_used_width + w <= bin_width:
        x = current_level_used_width
        y = current_level_y

        placements.append({
            "x": x,
            "y": y,
            "width": w,
            "height": h
        })

        current_level_used_width += w

    else:
        current_level_y += current_level_height
        current_level_height = h              
        current_level_used_width = w         

        placements.append({
            "x": 0,
            "y": current_level_y,
            "width": w,
            "height": h
        })

total_height = current_level_y + current_level_height

output = {
    "bin_width": bin_width,
    "total_height": total_height,
    "placements": placements
}

print(json.dumps(output, indent=2))

