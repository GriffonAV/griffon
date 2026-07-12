import os
import base64
import zipfile
import io

EICAR_B64 = b"WDVPIVAlQEFQWzRcUFpYNTQoUF4pN0NDKTd9JEVJQ0FSLVNUQU5EQVJELUFOVElWSVJVUy1URVNULUZJTEUhJEgrSCo="
DEST_DIR = "tests/fixtures"

os.makedirs(DEST_DIR, exist_ok=True)
eicar_content = base64.b64decode(EICAR_B64)

plain_path = os.path.join(DEST_DIR, "eicar.com")
with open(plain_path, "wb") as f:
    f.write(eicar_content)
print(f"[+] Created flat EICAR at {plain_path}")

zip_path = os.path.join(DEST_DIR, "eicar.zip")
with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zf:
    zf.writestr("eicar.com", eicar_content)
print(f"[+] Created zipped EICAR at {zip_path}")

nested_zip_path = os.path.join(DEST_DIR, "eicar_nested.zip")

inner_zip_buffer = io.BytesIO()
with zipfile.ZipFile(inner_zip_buffer, 'w', zipfile.ZIP_DEFLATED) as inner_zf:
    inner_zf.writestr("eicar.com", eicar_content)

with zipfile.ZipFile(nested_zip_path, 'w', zipfile.ZIP_DEFLATED) as outer_zf:
    outer_zf.writestr("inner.zip", inner_zip_buffer.getvalue())
print(f"[+] Created nested zipped EICAR at {nested_zip_path}")