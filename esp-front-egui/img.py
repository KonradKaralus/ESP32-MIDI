import numpy as np
from PIL import Image

img = Image.open("icon.ico")

arr = np.array(img)

out = "pub const ARR:[u8;262144] = ["

for row in arr:
    for p in row:
        out += f"{p[0]},{p[1]},{p[2]},{p[3]},"

out += "];"

with open("icon.rs","w") as f:
    f.write(out)