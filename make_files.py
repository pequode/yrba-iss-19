from pathlib import Path
import random
def _generate_random_files(size:int=100_000,num:int=100_000,dir:Path=Path("./folder-to-backup/")):
    dir.mkdir(parents=True,exist_ok=True)
    for i in range(num):
        file_name = f"file_{random.randint(0,num*100_000)}"
        content = random.randbytes(8) * (size//8)
        (dir/file_name).write_bytes(content)
_generate_random_files()