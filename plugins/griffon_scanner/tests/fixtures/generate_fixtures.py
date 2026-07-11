import os

BASE_DIR = "tests/fixtures/dummy_fs"
DIRS_TO_MAKE = [
    "clean_files",
    "media",
    "node_modules/fake_package",
    ".git/objects"
]

def create_file(path, size_kb=1, content="Clean data"):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        f.write(content + ("0" * (size_kb * 1024 - len(content))))

def create_image(path, width=100, height=100):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    from PIL import Image
    img = Image.new('RGB', (width, height), color='black')
    img.save(path)

for d in DIRS_TO_MAKE:
    os.makedirs(os.path.join(BASE_DIR, d), exist_ok=True)

create_file(f"{BASE_DIR}/clean_files/test1.txt")
create_image(f"{BASE_DIR}/media/image.png")
create_file(f"{BASE_DIR}/node_modules/fake_package/index.js")

# 60 MB file to test size limit
create_file(f"{BASE_DIR}/large_file.dat", size_kb=60 * 1024, content="This is a large file to test the size limit.")

print(f"[+] Generated dummy filesystem at {BASE_DIR}")