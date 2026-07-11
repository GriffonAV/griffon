import os

BASE_DIR = "tests/fixtures/heavy_fs"
NUM_FILES = 5000
SUBDIRS = 50

print(f"[+] Generating {NUM_FILES} files across {SUBDIRS} directories...")

for i in range(SUBDIRS):
    dir_path = os.path.join(BASE_DIR, f"folder_{i}")
    os.makedirs(dir_path, exist_ok=True)
    
    # Put 100 files in each folder
    for j in range(NUM_FILES // SUBDIRS):
        file_path = os.path.join(dir_path, f"file_{j}.txt")
        with open(file_path, "w") as f:
            f.write("Just some clean text data to give the scanner something to read." * 10)

print(f"[+] Done! Heavy filesystem created at {BASE_DIR}")