

egui_keys = [
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "ArrowUp",
    "Escape",
    "Tab",
    "Backspace",
    "Enter",
    "Space",
    "Insert",
    "Delete",
    "Home",
    "End",
    "PageUp",
    "PageDown",
    "Num0",
    "Num1",
    "Num2",
    "Num3",
    "Num4",
    "Num5",
    "Num6",
    "Num7",
    "Num8",
    "Num9",
    "A",
    "B",
    "C",
    "D",
    "E",
    "F",
    "G",
    "H",
    "I",
    "J",
    "K",
    "L",
    "M",
    "N",
    "O",
    "P",
    "Q",
    "R",
    "S",
    "T",
    "U",
    "V",
    "W",
    "X",
    "Y",
    "Z",
]

def to_amethyst_key(k):
    if k.startswith("Num"):
        return "Key" + k[3:]
    elif k.startswith("Arrow"):
        return k[5:]
    elif k == "Backspace":
        return "Back"
    elif k == "Enter":
        return "Return"
    else:
        return k

print("match key {")
for k in egui_keys:
    print("  VirtualKeyCode::{} => Some(Key::{}),".format(to_amethyst_key(k), k))
print("  _ => None")
print("}")


print(egui)